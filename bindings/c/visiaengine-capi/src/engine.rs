//! VeEngine 实体（计划 v1.4 §3）：headless 出图真身（I2）。
//! 装载=CPU 解析 → MeshCore 上传；拾取=REND-21..24 复用；策略=输入→相机。
//! bytes-first load 口为批 7 预留（[FFI-R:EP-附]/P2）：I2 走 path，J1 前升 bytes。

use visiaengine_core::{EntityId, Scene};

use crate::ffi::enc_entity; // CAPI-10 反标键=宿主可见位形（CAPI-08 编码单源）
use visiaengine_io_text::GLYPH_ATLAS_PX;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MaterialId, MeshCandidate, MeshDesc,
    MeshId, PointMark, PointTableDesc, RenderBackend, StrokeSeg, StrokeTableDesc, Viewport,
    ViewportRect, pick_meshes, ray_ground_intersect, screen_to_ray_ortho, screen_to_ray_persp,
};
use visiaengine_render_wgpu::HeadlessBackend;
use visiaengine_render_wgpu::MultiClearPolicy;
use visiaengine_render_wgpu::surface::Swapchain;

/// 输入→相机引擎策略（非事件透传）：按下后移动=orbit，滚轮=共享 zoom。
const ORBIT_RATE: f64 = 0.005;
const ZOOM_RATE: f64 = 0.1;
const FIT_PAD: f64 = 1.25;

struct DrawItem {
    mesh: MeshId,
    material: MaterialId,
    entity: EntityId,
    origin: [f64; 3],
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

/// 渲染目标态（CAPI-06：headless 与窗口互斥共存于同一引擎，attach 失败不动现态）。
enum TargetMode {
    Headless,
    Window(Swapchain),
}

/// 投影模式（geo=ortho fit、gltf=persp，viewport 变更重建）。
#[derive(Clone, Copy, PartialEq)]
enum Proj {
    Persp,
    Ortho,
}

/// CAPI-23 推进器状态：from 起飞快照 / to 目标 / 起止时钟 / 时长。
struct Fly {
    from: CameraRig,
    to: CameraRig,
    start_ms: f64,
    dur_s: f64,
}

/// 双平台单调毫秒钟：native=进程锚点差值（Instant 单调语义保留），wasm=Date.now()
/// （std::time 在 wasm32-unknown-unknown 运行时不可用——Web 是本 SDK 主舞台，
/// 此 shim 免 flyTo 触 capi_guard -4 假死）。
#[must_use]
fn now_ms() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::LazyLock;
        static T0: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);
        #[allow(clippy::cast_precision_loss)]
        return T0.elapsed().as_secs_f64() * 1000.0;
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
}

pub struct Engine {
    pub w: u32,
    pub h: u32,
    rig: CameraRig,
    mode: Proj,
    zoom: f64,
    down: bool,
    last: (f32, f32),
    backend: HeadlessBackend,
    target: TargetMode,
    scene: Scene, // EntityId 生成器（slot+gen 与 core slab 同空间，pick/entity_at 共用）
    items: Vec<DrawItem>,
    /// geo 扩片族命令（GEO-24/WGPU-17/18：mount 期建表，逐帧随 items 重放）
    extra_cmds: Vec<DrawCommand>,
    /// CAPI-12: load_geojson 保留面（每载一 doc，行=feature 下标 GEO-17）
    geo_docs: Vec<visiaengine_geo::GeoDocument>,
    /// CAPI-10: entity（C u64 位形，与 pick 出口同编码；含 gen=ABA 免疫）→ (doc 序号, 行)
    attr_of: std::collections::HashMap<u64, (usize, usize)>,
    /// CAPI-13: 隐藏实体位形表（render/pick 过滤域；枚举域不变，删除随 CAPI-16 清理）
    hidden: Vec<u64>,
    /// CAPI-20: 剖面裁切世界面（None/空=全保留；render 逐帧入 Frame.clip，
    /// pick 命中点 keeps 谓词负侧=排除重试 [WGPU-21 管线面同一世界帧]）
    clip: Option<visiaengine_render::ClipSetup>,
    /// CAPI-23：飞行推进器（墙钟住 engine——render crate 零 std::time 的分层定案；
    /// None=idle/done 合并态）
    fly: Option<Fly>,
    /// ⑤b 二波：小地图（None=关；旧宿主零感知）
    map: Option<MapView>,
    frame_cache: Option<Vec<u8>>,
    /// CAPI-19：点云云级 meta（位形→单行 AttrSet；attr_* 缀查域，geo 图先查）。
    pcl_meta: std::collections::HashMap<u64, visiaengine_core::AttrSet>,
    /// CAPI-21：标注字体（None=文字管线休眠——样式/口在，零输出 [GEO-25/CAPI-22]）
    font: Option<visiaengine_io_text::FontFace>,
    /// 跨装载常驻的字形图集（dirty→render 前全量重传 [WGPU-24]）
    glyphs: visiaengine_io_text::GlyphCache,

    /// CAPI-17 事件锚（fn/user 裸指针 usize 形；触发域=owner 线程，
    /// 与 CAPI-03 亲和门同谱，故 Send 包装安全）。
    evt: Option<EvtSink>,
}

/// ⑤b 二波 CAPI-25：小地图视口（比例表+世界半高；None=关=旧单帧路，主 target 派生跟随）。
#[derive(Clone, Copy, Debug)]
pub struct MapView {
    pub fx: f32,
    pub fy: f32,
    pub fw: f32,
    pub fh: f32,
    pub zoom: f64,
}

/// SAFETY: 锚仅在 owner 线程被 gate 后触发（回调线程契约见 CAPI-17 条款体）。
struct EvtAnchor(usize, usize);
unsafe impl Send for EvtAnchor {}

/// 事件汇双形：native=C 锚（Mutex 表需 Send）；wasm32=JS 闭包（RefCell 单线程，
/// js_sys::Function 非 Send 合法）——Engine 的 Send 域随 cfg 保持精确。
#[cfg(not(target_arch = "wasm32"))]
type EvtSink = EvtAnchor;
#[cfg(target_arch = "wasm32")]
type EvtSink = Box<dyn FnMut(u32, u64, u64)>;

impl Engine {
    #[must_use]
    pub fn new_headless(w: u32, h: u32) -> Option<Self> {
        Some(Self {
            w,
            h,
            rig: CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            mode: Proj::Persp,
            zoom: 1.0,
            down: false,
            last: (0.0, 0.0),
            backend: HeadlessBackend::new(w, h)?,
            target: TargetMode::Headless,
            scene: Scene::new(),
            items: Vec::new(),
            extra_cmds: Vec::new(),
            geo_docs: Vec::new(),
            attr_of: std::collections::HashMap::new(),
            hidden: Vec::new(),
            clip: None,
            fly: None,
            map: None,
            font: None,
            glyphs: visiaengine_io_text::GlyphCache::new(),
            frame_cache: None,
            evt: None,
            pcl_meta: std::collections::HashMap::new(),
        })
    }

    #[allow(clippy::too_many_arguments)] // 私有上传汇点（normals 参数化后 +1，拆门面反而碎）
    fn upload(
        &mut self,
        entity: EntityId,
        positions: &[[f32; 3]],
        indices: &[u32],
        normals: Option<&[[f32; 3]]>,
        mat: &MaterialDesc,
        uv: &[[f32; 2]],
        origin: [f64; 3],
    ) -> Result<(), String> {
        let flat: Vec<[f32; 3]>;
        let nrm = match normals {
            Some(n) => n,
            None => {
                flat = vec![[0.0f32, 0.0, 1.0]; positions.len()];
                &flat[..]
            }
        };
        let mesh = self
            .backend
            .create_mesh(&MeshDesc {
                positions,
                normals: nrm,
                indices,
                uv,
            })
            .map_err(|e| format!("create_mesh: {e:?}"))?;
        let material = self
            .backend
            .create_material_desc(mat)
            .map_err(|e| format!("create_material_desc: {e:?}"))?;
        self.items.push(DrawItem {
            mesh,
            material,
            entity,
            origin,
            positions: positions.to_vec(),
            indices: indices.to_vec(),
        });
        Ok(())
    }

    // ===== 装载（CAPI-04）=====

    /// CAPI-17：注册锚（cb=0 → 摘除）。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_event_anchor(&mut self, cb: usize, user: usize) {
        self.evt = if cb == 0 {
            None
        } else {
            Some(EvtAnchor(cb, user))
        };
    }

    /// CAPI-17 wasm 形：闭包注册（js_sys::Function 由 wasm 桥包入）。
    #[cfg(target_arch = "wasm32")]
    pub fn set_event_fn(&mut self, f: Option<Box<dyn FnMut(u32, u64, u64)>>) {
        self.evt = f;
    }

    /// CAPI-17：事件触发点（owner 线程内同步直调；重入=调用方禁区）。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn emit(&mut self, event: u32, a: u64, b: u64) {
        if let Some(EvtAnchor(f, u)) = self.evt {
            let f: unsafe extern "C" fn(*mut std::ffi::c_void, u32, u64, u64) =
                unsafe { std::mem::transmute(f) };
            unsafe { f(u as *mut _, event, a, b) };
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn emit(&mut self, event: u32, a: u64, b: u64) {
        if let Some(f) = self.evt.as_mut() {
            f(event, a, b);
        }
    }

    /// glTF path 口（CAPI-04）。
    pub fn load_gltf(&mut self, path: &str) -> Result<usize, String> {
        let doc = visiaengine_io_gltf::load_gltf(path).map_err(|e| format!("{path}: {e}"))?;
        self.mount_gltf(&doc)
    }

    /// glTF bytes 口（批 7 js 面复用；[FFI-R:EP-附] bytes-first 兑现）。
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    pub fn load_gltf_bytes(&mut self, data: &[u8]) -> Result<usize, String> {
        let doc = visiaengine_io_gltf::load_gltf_bytes(data).map_err(|e| format!("bytes: {e}"))?;
        self.mount_gltf(&doc)
    }

    /// primitive 粒度=实体（与 GLTF-01 家族对齐；world 烘入上传顶点，
    /// DrawItem.origin=0——headless demo 小坐标域，rebase 恒等合法）。
    fn mount_gltf(&mut self, doc: &visiaengine_io_gltf::GltfDocument) -> Result<usize, String> {
        let n = doc.entities().len();
        if n == 0 {
            return Err("no primitives".to_string());
        }
        // GLTF-11 纹理槽位 → GPU 纹理 id（去重按 image 槽位；同图多 primitive 共享）
        let mut tex_ids = Vec::with_capacity(doc.textures().len());
        for t in doc.textures() {
            tex_ids.push(
                self.backend
                    .upload_texture(&visiaengine_render::TextureDesc {
                        rgba: &t.rgba,
                        width: t.width,
                        height: t.height,
                    })
                    .map_err(|e| format!("upload_texture: {e:?}"))?,
            );
        }
        let mut done = 0usize;
        for e in doc.entities() {
            done += 1; // CAPI-17：逐实体进度（终态 done==n，与返回计数同谱）
            self.emit(crate::ffi::VE_EVT_LOAD_PROGRESS, done as u64, n as u64);
            let id = self.scene.spawn();
            // 局部顶点不变；headless persp 相机 ±10 域，twoprim/hierarchy 同尺度合法
            let mat = MaterialDesc {
                base_color: e.mesh.base_color,
                texture: e.mesh.texture.and_then(|s| tex_ids.get(s).copied()),
                repeat: [1.0, 1.0],
                // mock-up [4ab①]：(1-metallic)*roughness 反向强度→Lambert 系数（WGPU-14）
                specular: (1.0 - e.mesh.metallic_factor) * e.mesh.roughness_factor,
            };
            self.upload(
                id,
                &e.mesh.positions,
                &e.mesh.indices,
                Some(&e.mesh.normals),
                &mat,
                &e.mesh.uv,
                [0.0; 3],
            )?;
        }
        Ok(n)
    }

    /// GeoJSON path 口（CAPI-04）。
    pub fn load_geojson(&mut self, path: &str) -> Result<usize, String> {
        let doc = visiaengine_geo::load_geojson(path).map_err(|e| format!("{path}: {e}"))?;
        self.mount_geo(doc)
    }

    /// GeoJSON bytes 口（js 面；Lenient 策略——脏件丢弃不因数据拖死整层，
    /// 报告导出=M2）。
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    pub fn load_geojson_bytes(&mut self, data: &[u8]) -> Result<usize, String> {
        let (doc, _rep) =
            visiaengine_geo::parse_geojson_lenient(data).map_err(|e| format!("bytes: {e}"))?;
        self.mount_geo(doc)
    }

    /// feature 粒度=实体（一件可多 part 共享 id）；layer bbox 中心
    /// =origin（D7 shifted 纪律，geo_viewer 同款）；正交 fit 取景。
    fn mount_geo(&mut self, doc: visiaengine_geo::GeoDocument) -> Result<usize, String> {
        let [x0, y0, x1, y1] = doc.layer_bbox().ok_or("empty layer")?;
        let origin = [(x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0];
        let radius = ((x1 - x0) / 2.0).max((y1 - y0) / 2.0) * FIT_PAD;
        // 退化层（单点/单线=半径 0）与 eye 同下限——ortho zoom=0=必崩既有洞，
        // S2 标签单点测试抓回（存量 geo 测试全多点故未踏）
        self.zoom = radius.max(10.0);
        self.rig = CameraRig::look_at(
            [origin[0], origin[1], radius.max(10.0)],
            origin,
            [0.0, 1.0, 0.0],
        );
        self.mode = Proj::Ortho;
        let neg = [-origin[0], -origin[1]];
        let doc_no = self.geo_docs.len(); // CAPI-12: 本次装载在保留面中的序号
        let mut rows: Vec<(u64, (usize, usize))> = Vec::new();
        let mut n = 0usize;
        // GEO-25：标签请求（字体休眠=整段零收集，零输出不报错）
        let mut label_marks: Vec<visiaengine_render::LabelMark> = Vec::new();
        let cols = doc.attrs();
        for (row, f) in doc.features().iter().enumerate() {
            let id = self.scene.spawn();
            rows.push((enc_entity(id), (doc_no, row))); // CAPI-10: 键=宿主可见位形；行=GEO-17 展平下标
            n += 1;
            // CAPI-17：feature 粒度进度（total=features.len，终态与 n 等）
            self.emit(
                crate::ffi::VE_EVT_LOAD_PROGRESS,
                n as u64,
                doc.features().len() as u64,
            );
            if let (Some(face), Some(field)) = (self.font.as_ref(), f.style.text_field.as_deref()) {
                // 列语义优先（列在且行值缺=skip 不落字面量）；列不存在=常量文本
                let is_col = cols.names().any(|c| c == field);
                let text: Option<&str> = if is_col {
                    cols.str_value(row, field)
                } else {
                    Some(field)
                };
                if let (Some(t), Some([x0, y0, x1, y1])) = (text, f.world_bbox) {
                    let (ax, ay) = ((x0 + x1) / 2.0 + neg[0], (y0 + y1) / 2.0 + neg[1]);
                    let (quads, pen) = visiaengine_io_text::layout(
                        t,
                        face,
                        &mut self.glyphs,
                        f.style.text_size_px,
                    );
                    if !quads.is_empty() {
                        let lin = visiaengine_core::srgb_to_linear(f.style.text_color);
                        let anchor = [ax as f32, ay as f32, 0.1]; // 恒顶 0.1（无深度竞争，纯序语义）
                        label_marks.extend(quads.iter().map(|q| {
                            visiaengine_render::LabelMark::new(
                                anchor,
                                lin,
                                [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                                [
                                    q.size_px[0],
                                    q.size_px[1],
                                    q.top_left_px[0] - pen / 2.0, // MapLibre 默认 center 锚
                                    q.top_left_px[1],
                                ],
                            )
                        }));
                    }
                }
            }
            let parts = visiaengine_geo::tessellate(&f.kind.shifted(neg), &f.style)
                .map_err(|e| format!("tessellate: {e}"))?;
            // [PIT-8 提取位] LineStrip→StrokeSeg / Marker→PointMark 转换与
            // geo_viewer/geo_pipeline 同形三处——第三消费者已现，4de 后抽公共 helper。
            for gp in parts {
                match gp {
                    visiaengine_geo::GeoPart::Fill(p) => {
                        if p.positions.is_empty() || p.indices.len() < 3 {
                            continue;
                        }
                        let mat = MaterialDesc {
                            base_color: p.color,
                            texture: None,
                            repeat: [1.0, 1.0],
                            specular: 0.0,
                        };
                        self.upload(id, &p.positions, &p.indices, None, &mat, &[], origin)?;
                    }
                    visiaengine_geo::GeoPart::Strokes(strips) => {
                        let mut segs = Vec::new();
                        for s in strips {
                            for w in s.pts.windows(2) {
                                segs.push(StrokeSeg::new(
                                    [w[0][0], w[0][1], 0.0],
                                    [w[1][0], w[1][1], 0.0],
                                    s.color,
                                    s.width_px,
                                ));
                            }
                        }
                        if segs.is_empty() {
                            continue;
                        }
                        let table = self
                            .backend
                            .create_strokes(&StrokeTableDesc { data: &segs })
                            .map_err(|e| format!("strokes: {e}"))?;
                        self.extra_cmds.push(DrawCommand::DrawStrokes {
                            table,
                            origin,
                            transform: IDENTITY,
                        });
                    }
                    visiaengine_geo::GeoPart::Markers(ms) => {
                        let marks: Vec<PointMark> = ms
                            .iter()
                            .map(|m| {
                                PointMark::new([m.pos[0], m.pos[1], 0.0], m.color, m.radius_px)
                            })
                            .collect();
                        if marks.is_empty() {
                            continue;
                        }
                        let table = self
                            .backend
                            .create_points(&PointTableDesc { data: &marks })
                            .map_err(|e| format!("points: {e}"))?;
                        self.extra_cmds.push(DrawCommand::DrawPoints {
                            table,
                            origin,
                            transform: IDENTITY,
                        });
                    }
                }
            }
        }
        if !label_marks.is_empty() {
            let table = self
                .backend
                .create_labels(&visiaengine_render::LabelTableDesc { data: &label_marks })
                .map_err(|e| format!("labels: {e}"))?;
            self.extra_cmds.push(DrawCommand::DrawLabels {
                table,
                origin,
                transform: IDENTITY,
            });
        }
        self.geo_docs.push(doc); // CAPI-12 保留面（部分失败=未 push，反标同步不发布）
        for (id, r) in rows {
            self.attr_of.insert(id, r);
        }
        Ok(n)
    }

    // ===== 输入/相机（CAPI-05）=====

    pub fn apply_input(&mut self, kind: u32, px: f32, py: f32, wheel: f32) -> Option<bool> {
        // CAPI-23 中断语义（MapLibre A 派）：指针/滚轮即 cancel；键(5)与未知不打断
        if matches!(kind, 1 | 2 | 4) {
            self.fly = None;
        }
        // CAPI-25 配套：小图区内指针/滚轮=消费 no-op（v0 无拖图语义；导航走 navigate_click）
        if matches!(kind, 1..=4)
            && self
                .map_rect()
                .is_some_and(|r| r.contains(px.max(0.0) as u32, py.max(0.0) as u32))
        {
            return Some(true);
        }
        match kind {
            2 => {
                self.down = true;
                self.last = (px, py);
                Some(true)
            }
            3 => {
                self.down = false;
                Some(true)
            }
            1 => {
                if !self.down {
                    return Some(false);
                }
                let (dx, dy) = (f64::from(px - self.last.0), f64::from(py - self.last.1));
                self.last = (px, py);
                self.rig.orbit_delta(-dx * ORBIT_RATE, dy * ORBIT_RATE);
                Some(true)
            }
            4 => {
                self.zoom *= (f64::from(wheel) * ZOOM_RATE).exp();
                self.rig.zoom = self.zoom.max(0.001);
                Some(true)
            }
            5 => Some(false),
            _ => None,
        }
    }

    // ===== attach（CAPI-06）=====

    /// 裸句柄建面并接管渲染目标（configure 成功才算 attach 完成）。
    ///
    /// # Safety
    /// 见 `Swapchain::from_raw_handles`（宿主窗口存活义务）。
    pub unsafe fn attach_raw(
        &mut self,
        display: Option<raw_window_handle::RawDisplayHandle>,
        window: raw_window_handle::RawWindowHandle,
    ) -> Result<(), String> {
        // 先建后换：失败保全 headless 现目标（attach 原子性，CAPI-06）
        let sw =
            unsafe { self.backend.attach_surface(display, window) }.map_err(|e| e.to_string())?;
        self.target = TargetMode::Window(sw);
        Ok(())
    }

    // ===== 渲染/回读（CAPI-04）=====

    pub fn render(&mut self) -> Result<(), String> {
        self.advance_fly();
        let mut commands = vec![DrawCommand::ClearColor {
            rgba: [0.05, 0.07, 0.10, 1.0],
        }];
        for it in &self.items {
            if self.hidden.contains(&enc_entity(it.entity)) {
                continue; // CAPI-13 过滤域
            }
            commands.push(DrawCommand::DrawMesh {
                mesh: it.mesh,
                material: it.material,
                origin: it.origin,
                transform: IDENTITY,
            });
        }
        if self.glyphs.take_dirty() {
            let g = GLYPH_ATLAS_PX;
            self.backend
                .set_glyph_atlas(self.glyphs.pixels(), g, g)
                .map_err(|e| format!("atlas: {e}"))?;
        }
        commands.append(&mut self.extra_cmds.clone());
        let (hw, hh) = self.half_extents();
        let proj = match self.mode {
            Proj::Persp => self
                .rig
                .perspective(
                    self.rig.fov_y as f32,
                    self.w as f32 / self.h as f32,
                    0.1,
                    1000.0,
                )
                .ok_or("persp degenerate")?,
            Proj::Ortho => self
                .rig
                .ortho_frame(hw as f32, self.w as f32, self.h as f32, 0.1, 1000.0)
                .ok_or("ortho degenerate")?,
        };
        let camera = match self.mode {
            Proj::Persp => Camera::perspective(
                self.rig.fov_y as f32,
                self.w as f32 / self.h as f32,
                0.1,
                1000.0,
            ),
            Proj::Ortho => Camera::ortho(hw as f32, hh as f32, 0.1, 1000.0),
        };
        let eye = self.rig.eye();
        let frame = Frame {
            viewport: Viewport::new(self.w, self.h, 1.0),
            camera,
            view_rot: self.rig.view_rotation(),
            eye,
            proj,
            // ortho：世界宽 2·hw 铺 W px → px_world_scale=2·hw/W（REND-29 精确路）
            px_world_scale: if matches!(self.mode, Proj::Ortho) {
                2.0 * hw as f32 / self.w as f32
            } else {
                1.0
            },
            shadow: None,
            clip: self.clip,
            commands,
        };
        // ⑤b 二波：map=Some ⇒ 主全幅+小图角窗两投；None ⇒ 旧单帧路（canary 构造保真）
        if let (Some(r), Some(mrig)) = (self.map_rect(), self.map_rig()) {
            let (rw, rh) = (r.width.max(1), r.height.max(1));
            let proj = mrig
                .ortho_frame(mrig.zoom as f32, rw as f32, rh as f32, 0.1, 1000.0)
                .ok_or("map ortho degenerate")?;
            let map_frame = Frame {
                viewport: Viewport::new(rw, rh, 1.0),
                camera: Camera::ortho(
                    mrig.zoom as f32,
                    (mrig.zoom * f64::from(rh) / f64::from(rw.max(1))) as f32,
                    0.1,
                    1000.0,
                ),
                view_rot: mrig.view_rotation(),
                eye: mrig.eye(),
                proj,
                px_world_scale: (2.0 * mrig.zoom / f64::from(rw)) as f32,
                shadow: None,
                clip: self.clip,
                commands: frame.commands.clone(),
            };
            let passes = [(frame, ViewportRect::full(self.w, self.h)), (map_frame, r)];
            return match &mut self.target {
                TargetMode::Headless => {
                    let img = self
                        .backend
                        .render_to_pixels_rects(&passes, MultiClearPolicy::FirstClearRestLoad)
                        .ok_or("render failed")?;
                    self.frame_cache = Some(img.rgba);
                    Ok(())
                }
                TargetMode::Window(sw) => self
                    .backend
                    .render_swapchain_multi(&passes, sw)
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
            };
        }
        match &mut self.target {
            TargetMode::Headless => {
                let img = self
                    .backend
                    .render_to_pixels(&frame)
                    .ok_or("render failed")?;
                self.frame_cache = Some(img.rgba);
                Ok(())
            }
            TargetMode::Window(sw) => self
                .backend
                .render_swapchain(&frame, sw)
                .map(|_| ())
                .map_err(|e| e.to_string()),
        }
    }

    /// readback 缓冲需求（未渲染=0，调用方据此给 -2/-5 归因）。
    #[must_use]
    pub fn frame_bytes(&self) -> Option<usize> {
        self.frame_cache.as_ref().map(Vec::len)
    }

    pub fn copy_frame(&self, buf: &mut [u8]) {
        if let Some(f) = &self.frame_cache {
            buf[..f.len()].copy_from_slice(f);
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.w = w;
        self.h = h;
        self.backend.resize(Viewport::new(w, h, 1.0));
        if let TargetMode::Window(sw) = &mut self.target {
            sw.resize(w, h); // 下帧 configure 重配（CAPI-06 resize 路径）
        }
        self.frame_cache = None;
    }

    fn half_extents(&self) -> (f64, f64) {
        let aspect = f64::from(self.w) / f64::from(self.h);
        (self.zoom, self.zoom / aspect.max(1e-6))
    }

    pub fn entity_count(&self) -> u32 {
        let mut ids: Vec<EntityId> = self.items.iter().map(|i| i.entity).collect();
        ids.dedup();
        ids.len() as u32
    }

    pub fn entity_at(&self, index: u32) -> Option<EntityId> {
        let mut seen: Vec<EntityId> = Vec::new();
        for it in &self.items {
            if !seen.contains(&it.entity) {
                seen.push(it.entity);
            }
        }
        seen.into_iter().nth(index as usize)
    }

    /// 屏幕拾取（复用 REND-21..24；ortho/persp 双路）。命中=EntityId。
    pub fn pick(&self, px: f32, py: f32) -> Option<EntityId> {
        // ⑤b 二波路由：小图区优先（视觉顶层=输入顶层）；其余=主 rig 原语义
        if let (Some(r), Some(mrig)) = (self.map_rect(), self.map_rig())
            && let Some((lx, ly)) = r.local(px.max(0.0) as u32, py.max(0.0) as u32)
        {
            let (id, _) = self.pick_geo(&mrig, true, lx, ly, r.width, r.height)?;
            return Some(id);
        }
        let ortho = matches!(self.mode, Proj::Ortho);
        let (id, _) = self.pick_geo(&self.rig, ortho, px, py, self.w, self.h)?;
        Some(id)
    }

    /// 拾取核（rig/投影形/局部 px 全参数化：主视口与小图共用；返回 (位形, 命中点 xy)）。
    fn pick_geo(
        &self,
        rig: &CameraRig,
        ortho: bool,
        px: f32,
        py: f32,
        w: u32,
        h: u32,
    ) -> Option<(EntityId, [f64; 2])> {
        let ray = if ortho {
            screen_to_ray_ortho(rig, px, py, w as f32, h.max(1) as f32)?
        } else {
            screen_to_ray_persp(rig, px, py, w as f32, h.max(1) as f32)?
        };
        let mut skip: Vec<EntityId> = Vec::new();
        loop {
            let cands: Vec<MeshCandidate> = self
                .items
                .iter()
                .filter(|it| {
                    !self.hidden.contains(&enc_entity(it.entity)) && !skip.contains(&it.entity)
                })
                .map(|it| MeshCandidate {
                    entity: it.entity,
                    positions: &it.positions,
                    indices: &it.indices,
                    world: &IDENTITY,
                })
                .collect();
            let hit = pick_meshes(ray, &cands)?;
            if let Some(c) = &self.clip {
                let p = hit.point;
                if !c.keeps([p.x, p.y, p.z]) {
                    skip.push(hit.entity);
                    continue;
                }
            }
            return Some((hit.entity, [hit.point.x, hit.point.y]));
        }
    }

    /// CAPI-13: 显隐切换（严格存在域校验；render/pick 过滤、枚举域不变）。
    /// CAPI-23：起飞/改道。位姿六分量+fov，**near/far 恒当前 rig 现值**（[Momus-A1]
    /// 深度域不飞行）；`dur_ms=0`=瞬移形（立即落位，非错误值）；飞行中重入=改道
    /// （from=当前位姿快照，t 归零——首帧连续免跳变 [Momus-A3]）。
    /// 域拒：非有限 / dist≤0 / zoom≤0 / fov∉(0,π)。
    #[allow(clippy::too_many_arguments)]
    pub fn fly_to(
        &mut self,
        target: [f64; 3],
        yaw: f64,
        pitch: f64,
        dist: f64,
        zoom: f64,
        fov: f64,
        dur_ms: u64,
    ) -> Result<(), String> {
        let ok = target.iter().all(|v| v.is_finite())
            && [yaw, pitch, dist, zoom, fov].iter().all(|v| v.is_finite())
            && dist > 0.0
            && zoom > 0.0
            && fov > 0.0
            && fov < std::f64::consts::PI;
        if !ok {
            return Err("fly_to: pose domain (finite/dist>0/zoom>0/fov in (0,pi))".into());
        }
        let to = CameraRig {
            target,
            yaw,
            pitch,
            dist,
            zoom,
            fov_y: fov,
            near: self.rig.near,
            far: self.rig.far,
        };
        self.fly_rig(to, dur_ms);
        Ok(())
    }

    /// 推进器落位核（fly_to/navigate_click 共路；域检由调用方完成）。
    fn fly_rig(&mut self, to: CameraRig, dur_ms: u64) {
        if dur_ms == 0 {
            self.rig = to;
            self.zoom = to.zoom;
            self.fly = None;
            return;
        }
        self.fly = Some(Fly {
            from: self.rig,
            to,
            start_ms: now_ms(),
            dur_s: dur_ms as f64 / 1000.0,
        });
    }

    /// CAPI-24 语义源：不在飞=done（idle/done 合并单主值域）。
    #[must_use]
    pub fn fly_state_done(&self) -> bool {
        self.fly.is_none()
    }

    /// 飞行中进度 [0,1)（done/idle=None——out 不写路的引擎形）。
    #[must_use]
    pub fn fly_progress(&self) -> Option<f64> {
        self.fly
            .as_ref()
            .map(|f| ((now_ms() - f.start_ms) / 1000.0 / f.dur_s).clamp(0.0, 1.0))
    }

    /// 每帧推进（render 首行）：t>=1 直落 to 原值（端点精确 REND-16 同谱）；
    /// 否则 fly_sample 采样落 rig（zoom 同步——ortho 族消费面）。
    fn advance_fly(&mut self) {
        let Some(f) = &self.fly else { return };
        let t = (now_ms() - f.start_ms) / 1000.0 / f.dur_s;
        let rig = if t >= 1.0 {
            f.to
        } else {
            CameraRig::fly_sample(&f.from, &f.to, t, visiaengine_render::Easing::CubicInOut)
        };
        self.rig = rig;
        self.zoom = rig.zoom;
        if t >= 1.0 {
            self.fly = None;
        }
    }

    /// CAPI-20：世界面系数组 [nx,ny,nz,d]（法向指保留侧，`dot(n,P)+d≥0` 保留，
    /// AND 组合）。n=0=唯一清空形；退化/非有限/超 4 拒（ClipSetup::new 同源门）。
    pub fn set_clips(&mut self, planes: &[[f64; 4]]) -> Result<(), String> {
        if planes.len() > visiaengine_render::ClipSetup::MAX_PLANES {
            return Err("set_clips: n>MAX(4)".into());
        }
        if planes.is_empty() {
            self.clip = None;
            return Ok(());
        }
        self.clip = Some(
            visiaengine_render::ClipSetup::new(planes)
                .ok_or("set_clips: degenerate plane (zero normal / non-finite)")?,
        );
        Ok(())
    }

    /// CAPI-25：开/关小地图（比例表 0..1，fw/fh>0，zoom>0；域拒零副作用；None=清空旧路）。
    // W3 接 visiaengine_set_map 三口后即活（先行暂死注记）。
    #[allow(dead_code)]
    pub fn set_map(&mut self, rect: Option<(f32, f32, f32, f32)>, zoom: f64) -> Result<(), String> {
        let Some((fx, fy, fw, fh)) = rect else {
            self.map = None;
            return Ok(());
        };
        let ok = [fx, fy, fw, fh].iter().all(|v| v.is_finite())
            && (0.0..=1.0).contains(&fx)
            && (0.0..=1.0).contains(&fy)
            && fw > 0.0
            && fh > 0.0
            && fx + fw <= 1.0 + 1e-6
            && fy + fh <= 1.0 + 1e-6
            && zoom.is_finite()
            && zoom > 0.0;
        if !ok {
            return Err("set_map: frac/zoom domain".into());
        }
        self.map = Some(MapView {
            fx,
            fy,
            fw,
            fh,
            zoom,
        });
        Ok(())
    }

    /// 当前小地图像素 rect（比例表 resize 自动跟的求值点）。
    #[must_use]
    pub fn map_rect(&self) -> Option<ViewportRect> {
        self.map
            .map(|m| ViewportRect::from_frac(m.fx, m.fy, m.fw, m.fh, self.w, self.h))
    }

    /// 小地图顶视 rig（跟随主 target；pitch≈86° 近正视，up+Y 与视向近正交免退化）。
    #[must_use]
    fn map_rig(&self) -> Option<CameraRig> {
        let m = self.map?;
        let [tx, ty, _] = self.rig.target;
        Some(CameraRig::orbit(
            [tx, ty, 0.0],
            0.0,
            1.5,
            m.zoom * 2.0,
            m.zoom,
            std::f64::consts::FRAC_PI_3,
            0.1,
            m.zoom * 20.0,
        ))
    }

    /// CAPI-26：小地图点击导航（两阶段：实体命中优先→地面 z=0 兜底；主 rig 保角保距换 target）。
    #[allow(dead_code)]
    pub fn navigate_click(&mut self, px: f32, py: f32, dur_ms: u64) -> Result<(), String> {
        let rect = self.map_rect().ok_or("navigate_click: map not set")?;
        let (lx, ly) = rect
            .local(px.max(0.0) as u32, py.max(0.0) as u32)
            .ok_or("navigate_click: outside map rect")?;
        let mrig = self.map_rig().ok_or("navigate_click: no map")?;
        let ray = screen_to_ray_ortho(&mrig, lx, ly, rect.width as f32, rect.height.max(1) as f32)
            .ok_or("navigate_click: ray degenerate")?;
        let (tx, ty) = if let Some(hit) =
            self.pick_geo(&mrig, true, lx, ly, rect.width, rect.height)
        {
            (hit.1[0], hit.1[1])
        } else {
            let g = ray_ground_intersect(ray).ok_or("navigate_click: no ground under cursor")?;
            (g.x, g.y)
        };
        let mut to = self.rig;
        to.target = [tx, ty, 0.0];
        self.fly_rig(to, dur_ms);
        Ok(())
    }

    /// CAPI-27：主相机位姿读回（宿主 HUD/到达断言）。
    #[must_use]
    #[allow(dead_code)]
    pub fn camera_pose(&self) -> CameraRig {
        self.rig
    }

    /// CAPI-21：装载/替换标注字体（bytes=TTF/OTF 全式；解析失败=Err 不动现存字体）。
    pub fn set_font(&mut self, bytes: &[u8]) -> Result<(), String> {
        let f =
            visiaengine_io_text::FontFace::from_bytes(bytes).map_err(|e| format!("font: {e:?}"))?;
        self.font = Some(f);
        self.glyphs = visiaengine_io_text::GlyphCache::new(); // 换字体=图集作废重建
        Ok(())
    }

    /// CAPI-20 读回（归一化后系数；截断/计数语义住 FFI 面）。
    #[must_use]
    pub fn clips(&self) -> Vec<[f64; 4]> {
        self.clip
            .map(|c| c.planes[..c.count].to_vec())
            .unwrap_or_default()
    }

    pub fn set_visible(&mut self, entity: u64, visible: bool) -> Result<(), String> {
        if !self.items.iter().any(|i| enc_entity(i.entity) == entity) {
            return Err(format!("unknown entity bits {entity:#018x}"));
        }
        self.hidden.retain(|h| *h != entity);
        if !visible {
            self.hidden.push(entity);
        }
        Ok(())
    }

    /// CAPI-14: 显隐查询（未知位形=None，与 setter 同域）。
    #[must_use]
    pub fn is_visible(&self, entity: u64) -> Option<bool> {
        if !self.items.iter().any(|i| enc_entity(i.entity) == entity) {
            return None;
        }
        Some(!self.hidden.contains(&entity))
    }

    /// CAPI-15: 程序化加网格（调用期拷贝；退化零提交，spawn 回滚不外泄槽位）。
    pub fn add_mesh(
        &mut self,
        positions: &[[f32; 3]],
        normals: Option<&[[f32; 3]]>,
        indices: &[u32],
        base_color: [f32; 4],
        origin: [f64; 3],
    ) -> Result<u64, String> {
        if positions.is_empty() || indices.is_empty() {
            return Err("add_mesh: empty geometry".into());
        }
        if let Some(n) = normals
            && n.len() != positions.len()
        {
            return Err("add_mesh: normals length mismatch".into());
        }
        if let Some(mx) = indices.iter().max()
            && (*mx as usize) >= positions.len()
        {
            return Err("add_mesh: index out of range".into());
        }
        let id = self.scene.spawn();
        let mat = MaterialDesc {
            base_color,
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0, // 存量 Flat 默认（WGPU-14 零回归族）
        };
        match self.upload(id, positions, indices, normals, &mat, &[], origin) {
            Ok(()) => Ok(enc_entity(id)),
            Err(e) => {
                let _ = self.scene.despawn(id); // 代际 +1：失败位形永不复用撞出
                Err(e)
            }
        }
    }

    /// CAPI-19: 点云文件装载（path 便捷口=bytes 形之文件皮；wasm 走 bytes 直入）。
    pub fn load_pcl(
        &mut self,
        path: &str,
        lenient: bool,
    ) -> Result<(u64, visiaengine_io_points::PclReport), String> {
        let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
        self.load_pcl_bytes(&bytes, lenient)
    }

    /// mount_pcl：单实体单 DrawPoints（origin=f64 bbox 中心，D7 零新数学）；
    /// 云级 meta 注册 pcl_meta（attr 缀查域）；失败 spawn 回滚同 add_mesh 谱。
    pub fn load_pcl_bytes(
        &mut self,
        bytes: &[u8],
        lenient: bool,
    ) -> Result<(u64, visiaengine_io_points::PclReport), String> {
        let cloud = visiaengine_io_points::parse_pcl(bytes, lenient).map_err(|e| e.to_string())?;
        let id = self.scene.spawn();
        let marks: Vec<PointMark> = cloud
            .positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let c = cloud.colors.get(i).copied().unwrap_or([0.7, 0.75, 0.8]);
                PointMark::new(*p, c, cloud.radius_px)
            })
            .collect();
        let table = match self.backend.create_points(&PointTableDesc { data: &marks }) {
            Ok(t) => t,
            Err(e) => {
                let _ = self.scene.despawn(id);
                return Err(format!("create_points: {e:?}"));
            }
        };
        self.extra_cmds.push(DrawCommand::DrawPoints {
            table,
            origin: cloud.origin,
            transform: IDENTITY,
        });
        let bits = enc_entity(id);
        self.pcl_meta.insert(bits, cloud.meta);
        self.emit(
            crate::ffi::VE_EVT_LOAD_PROGRESS,
            cloud.report.kept,
            cloud.report.kept,
        );
        Ok((bits, cloud.report))
    }

    /// CAPI-18: 点云直通（单实体单 DrawPoints；origin=[0,0,0] 宿主系局部——
    /// 非有限照收=宿主责任域，load 侧才挂 RepairPolicy；失败 despawn 回滚同 CAPI-15）。
    pub fn add_points(&mut self, raw: &[crate::ffi::VePointMark]) -> Result<u64, String> {
        let marks: Vec<PointMark> = raw
            .iter()
            .map(|m| PointMark::new(m.pos, m.color, m.radius_px))
            .collect();
        let id = self.scene.spawn();
        let table = match self.backend.create_points(&PointTableDesc { data: &marks }) {
            Ok(t) => t,
            Err(e) => {
                let _ = self.scene.despawn(id); // 代际 +1：失败位形不复用（add_mesh 同谱）
                return Err(format!("create_points: {e:?}"));
            }
        };
        self.extra_cmds.push(DrawCommand::DrawPoints {
            table,
            origin: [0.0; 3],
            transform: IDENTITY,
        });
        Ok(enc_entity(id))
    }

    /// CAPI-22：世界锚单标签（add_points 同谱=items 外管理域，位形可枚举、
    /// v0 删除不入 remove 面=明账）。**字体未载=显式拒**（数据口无休眠义，
    /// 与 GEO-25 样式休眠成对：样式休眠/口拒）。空文本/size 域外=拒零提交。
    pub fn add_label(
        &mut self,
        world: [f64; 3],
        text: &str,
        rgba_srgb: [f32; 4],
        size_px: f32,
    ) -> Result<u64, String> {
        let Some(face) = self.font.as_ref() else {
            return Err("add_label: font not loaded (CAPI-21 first)".into());
        };
        if text.is_empty() || size_px <= 0.0 || !size_px.is_finite() {
            return Err("add_label: empty text / size domain".into());
        }
        let (quads, pen) = visiaengine_io_text::layout(text, face, &mut self.glyphs, size_px);
        if quads.is_empty() {
            return Err("add_label: no raster quads".into());
        }
        let lin = visiaengine_core::srgb_to_linear(rgba_srgb);
        let marks: Vec<visiaengine_render::LabelMark> = quads
            .iter()
            .map(|q| {
                visiaengine_render::LabelMark::new(
                    [world[0] as f32, world[1] as f32, world[2] as f32],
                    lin,
                    [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                    [
                        q.size_px[0],
                        q.size_px[1],
                        q.top_left_px[0] - pen / 2.0,
                        q.top_left_px[1],
                    ],
                )
            })
            .collect();
        let id = self.scene.spawn();
        let table = match self
            .backend
            .create_labels(&visiaengine_render::LabelTableDesc { data: &marks })
        {
            Ok(t) => t,
            Err(e) => {
                let _ = self.scene.despawn(id);
                return Err(format!("create_labels: {e:?}"));
            }
        };
        self.extra_cmds.push(DrawCommand::DrawLabels {
            table,
            origin: [0.0; 3],
            transform: IDENTITY,
        });
        Ok(enc_entity(id))
    }

    /// CAPI-16: 删除实体（items/attr_of/hidden 三面清理；旧句柄再入=双销毁同谱）。
    pub fn remove_entity(&mut self, entity: u64) -> Result<(), String> {
        let Some(pos) = self
            .items
            .iter()
            .position(|i| enc_entity(i.entity) == entity)
        else {
            return Err(format!("unknown entity bits {entity:#018x}"));
        };
        let item = self.items.remove(pos);
        self.scene
            .despawn(item.entity)
            .map_err(|e| format!("remove_entity: {e:?}"))?;
        self.attr_of.remove(&entity);
        self.hidden.retain(|h| *h != entity);
        Ok(())
    }

    /// CAPI-10: entity 键控三型属性读（位形与 pick 出口同编码）；
    /// 缺失四径（无行/无列/异型/空格）一律 None 非零值。
    #[must_use]
    pub fn attr_f64(&self, entity: u64, name: &str) -> Option<f64> {
        if let Some(&(d, r)) = self.attr_of.get(&entity) {
            return self.geo_docs.get(d)?.attr_f64(r, name);
        }
        self.pcl_meta.get(&entity).and_then(|m| m.f64(0, name))
    }

    #[must_use]
    pub fn attr_str(&self, entity: u64, name: &str) -> Option<&str> {
        if let Some(&(d, r)) = self.attr_of.get(&entity) {
            return self.geo_docs.get(d)?.attr_str(r, name);
        }
        self.pcl_meta
            .get(&entity)
            .and_then(|m| m.str_value(0, name))
    }

    #[must_use]
    pub fn attr_bool(&self, entity: u64, name: &str) -> Option<bool> {
        if let Some(&(d, r)) = self.attr_of.get(&entity) {
            return self.geo_docs.get(d)?.attr_bool(r, name);
        }
        self.pcl_meta.get(&entity).and_then(|m| m.bool(0, name))
    }
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
impl Engine {
    /// Canvas 引擎构造（批 7 J1：async 设备 + canvas surface 一气呵成）。
    pub async fn new_canvas(canvas: &web_sys::HtmlCanvasElement) -> Option<Self> {
        let w = canvas.width().max(1);
        let h = canvas.height().max(1);
        let mut backend = HeadlessBackend::new_async(w, h).await?;
        let sw = backend.attach_canvas(canvas).ok()?;
        Some(Self {
            w,
            h,
            rig: CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            mode: Proj::Persp,
            zoom: 1.0,
            down: false,
            last: (0.0, 0.0),
            backend,
            target: TargetMode::Window(sw),
            scene: Scene::new(),
            items: Vec::new(),
            extra_cmds: Vec::new(),
            geo_docs: Vec::new(),
            attr_of: std::collections::HashMap::new(),
            hidden: Vec::new(),
            clip: None,
            fly: None,
            map: None,
            font: None,
            glyphs: visiaengine_io_text::GlyphCache::new(),
            frame_cache: None,
            evt: None,
            pcl_meta: std::collections::HashMap::new(),
        })
    }
}

const IDENTITY: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

#[cfg(test)]
mod attr_tests {
    //! CAPI-10/11/12（RED 先行）：保留面 + entity→(doc,row) 反标 + 三型读口。
    use super::*;
    use std::fs;

    /// 最小两层件：A 三型齐（str/f64/bool），B 仅 name（空格= None 契约面）。
    const GEO: &str = r#"{"type":"FeatureCollection","features":[
     {"type":"Feature","properties":{"name":"buildingA","height":12.5,"active":true},
      "geometry":{"type":"Polygon","coordinates":[[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]]}},
     {"type":"Feature","properties":{"name":"buildingB"},
      "geometry":{"type":"Polygon","coordinates":[[[2.0,2.0],[3.0,2.0],[3.0,3.0],[2.0,2.0]]]}}
    ]}"#;

    fn tmp_layer(tag: &str, body: &str) -> String {
        let dir = std::env::temp_dir().join(format!(
            "ve-attr-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0),
        ));
        fs::create_dir_all(dir.clone()).unwrap();
        let path = dir.join("layer.geojson");
        fs::write(&path, body).unwrap();
        path.to_str().unwrap().to_owned()
    }

    fn loaded() -> Engine {
        let mut e = Engine::new_headless(8, 8).expect("headless engine");
        e.load_geojson(&tmp_layer("a", GEO)).expect("load geojson");
        e
    }

    #[test]
    fn attr_three_types_hit_via_entity() {
        // spec: CAPI-10
        let e = loaded();
        let a = enc_entity(e.entity_at(0).expect("feature 0 entity"));
        let b = enc_entity(e.entity_at(1).expect("feature 1 entity"));
        assert_eq!(e.attr_str(a, "name"), Some("buildingA"));
        assert_eq!(e.attr_f64(a, "height"), Some(12.5));
        assert_eq!(e.attr_bool(a, "active"), Some(true));
        assert_eq!(e.attr_str(b, "name"), Some("buildingB"));
    }

    #[test]
    fn attr_missing_is_none_never_zero() {
        // spec: CAPI-10
        let e = loaded();
        let b = enc_entity(e.entity_at(1).expect("feature 1 entity"));
        assert_eq!(e.attr_f64(b, "height"), None); // 行在，格空
        assert_eq!(e.attr_f64(b, "nope"), None); // 列不存在
        assert_eq!(e.attr_f64(b, "name"), None); // 异型（str 列问 f64）
        assert_eq!(e.attr_bool(b, "name"), None); // 异型（str 列问 bool）
    }

    #[test]
    fn attr_str_content_exact() {
        // spec: CAPI-11
        let e = loaded();
        let a = enc_entity(e.entity_at(0).expect("entity"));
        assert_eq!(e.attr_str(a, "name"), Some("buildingA"));
        assert_eq!(e.attr_str(a, "missing"), None);
    }

    #[test]
    fn attr_multi_load_rows_independent() {
        // spec: CAPI-12
        let mut e = loaded();
        let old = enc_entity(e.entity_at(0).expect("old entity"));
        let second = r#"{"type":"FeatureCollection","features":[
          {"type":"Feature","properties":{"name":"towerC","height":99.0},
           "geometry":{"type":"Polygon","coordinates":[[[5.0,5.0],[6.0,5.0],[6.0,6.0],[5.0,5.0]]]}}
        ]}"#;
        assert_eq!(e.load_geojson(&tmp_layer("b", second)), Ok(1));
        assert_eq!(e.attr_str(old, "name"), Some("buildingA")); // 旧行不串
        let c = enc_entity(e.entity_at(2).expect("new entity"));
        assert_eq!(e.attr_f64(c, "height"), Some(99.0));
        assert_eq!(e.attr_str(c, "name"), Some("towerC"));
        assert_eq!(e.attr_bool(old, "active"), Some(true)); // 跨 doc 列隔离
    }
}

#[cfg(test)]
mod mut_spec_tests {
    //! CAPI-13..16（RED 先行）：显隐过滤 / 查询 / 程序化增 / 删-ABA。
    use super::*;
    use crate::ffi::enc_entity;

    const TWOPRIM: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/data/twoprim.glb"
    );

    fn eng() -> Engine {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        assert_eq!(e.load_gltf(TWOPRIM).unwrap(), 2, "twoprim=2 实体");
        e
    }

    fn quad() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0.0, 0.0, 1.0]; 4],
            vec![0, 1, 2, 0, 2, 3],
        )
    }

    // spec: CAPI-20
    #[test]
    fn clip_filters_pick_retry_and_set_domain() {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        let (pos, nrm, idx) = quad();
        // 注：capi pick 域=positions×IDENTITY（origin 住 render 路=既有事实）——
        // 测试把 z 烘进 positions 令两帧合一，回避该缺口不带病断言
        let flat: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 8.0 - 4.0, p[1] * 8.0 - 4.0, 0.0])
            .collect();
        let mut frontz = flat.clone();
        for q in &mut frontz {
            q[2] = 2.0;
        }
        let front = e
            .add_mesh(
                &frontz,
                Some(&nrm),
                &idx,
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
            )
            .expect("front");
        let back = e
            .add_mesh(
                &flat,
                Some(&nrm),
                &idx,
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
            )
            .expect("back");
        let hitk = |e: &Engine| e.pick(80.0, 60.0).map(enc_entity);
        assert_eq!(hitk(&e), Some(front), "默认命中前墙");
        // 世界面 z≤1 保留：前墙(z=2)命中点被裁 → 重试环命中后墙
        assert_eq!(e.set_clips(&[[0.0, 0.0, -1.0, 1.0]]), Ok(()));
        assert_eq!(hitk(&e), Some(back), "剖掉前墙必须重试命中后墙（非 miss）");
        assert_eq!(e.render(), Ok(()), "clip 入帧渲染路通");
        assert_eq!(e.clips(), vec![[0.0, 0.0, -1.0, 1.0]]);
        // n=0 清空=唯一清除形 → 前墙复中
        assert_eq!(e.set_clips(&[]), Ok(()));
        assert_eq!(hitk(&e), Some(front), "清空复原");
        assert!(e.clips().is_empty());
        // 退化拒（与 FFI 同源）：零法向/非有限/>4
        assert!(e.set_clips(&[[0.0, 0.0, 0.0, 1.0]]).is_err());
        assert!(e.set_clips(&[[f64::NAN, 0.0, 0.0, 0.0]]).is_err());
        assert!(e.set_clips(&[[0.0, 1.0, 0.0, 0.0]; 5]).is_err());
    }

    fn geo(fc: &str) -> visiaengine_geo::GeoDocument {
        visiaengine_geo::parse_geojson_lenient(fc.as_bytes())
            .unwrap()
            .0
    }

    const DEJAVU: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/data/DejaVuSans.ttf"
    ));

    // spec: GEO-25
    #[test]
    fn geo_labels_mount_only_with_font_and_text_style() {
        let fc = r#"{"type":"FeatureCollection","features":[
          {"type":"Feature","properties":{"text-field":"{name}"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}
        ]}"#;
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        // 无字体：样式在也零输出（挂载即过，不报错）
        assert_eq!(e.mount_geo(geo(fc)).unwrap(), 1);
        assert_eq!(e.label_cmd_count(), 0, "无字体=零标签输出");
        // 注字体+重装：产表入列
        e.set_font(DEJAVU).expect("font");
        let mut e2 = Engine::new_headless(160, 120).expect("adapter");
        e2.set_font(DEJAVU).expect("font");
        assert_eq!(e2.mount_geo(geo(fc)).unwrap(), 1);
        assert_eq!(e2.label_cmd_count(), 1, "一 doc 一标签表命令");
        assert_eq!(e2.render(), Ok(()), "标签帧渲染路通");
        // 列在（首行 name 建列）但该行为空→列语义 skip，不落字面量 "{name}" 文本
        let miss = r#"{"type":"FeatureCollection","features":[
          {"type":"Feature","properties":{"text-field":"{name}","name":"X"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}},
          {"type":"Feature","properties":{"text-field":"{name}"},"geometry":{"type":"Point","coordinates":[11.0,51.0]}}
        ]}"#;
        let mut e3 = Engine::new_headless(160, 120).expect("adapter");
        e3.set_font(DEJAVU).expect("font");
        e3.mount_geo(geo(miss)).unwrap();
        assert_eq!(e3.label_cmd_count(), 1, "一表合流");
        assert_eq!(
            e3.label_mark_total(),
            1,
            "缺值行=skip（X 一片，第二行零片）"
        );
        // 常量文本（列不存在=字面量形）
        let lit = r#"{"type":"FeatureCollection","features":[
          {"type":"Feature","properties":{"text-field":"Gate"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}
        ]}"#;
        let mut e4 = Engine::new_headless(160, 120).expect("adapter");
        e4.set_font(DEJAVU).expect("font");
        e4.mount_geo(geo(lit)).unwrap();
        assert_eq!(e4.label_cmd_count(), 1, "字面量文本产标");
    }

    impl Engine {
        /// extra_cmds 中标签命令数（GEO-25 零输出/宿主自省口）。
        #[must_use]
        pub fn label_cmd_count(&self) -> usize {
            self.extra_cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::DrawLabels { .. }))
                .count()
        }

        /// 标签表总 quad 数（列/字面量语义分辨口，宿主可自省）。
        #[must_use]
        pub fn label_mark_total(&self) -> usize {
            self.backend.label_mark_total()
        }
    }

    // spec: CAPI-23
    #[test]
    fn fly_to_wallclock_advance_cancel_and_reroute() {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        // dur=0 = 瞬移形（立即落位非错误）；near/far 恒现值 [A1]
        let near0 = e.rig.near;
        assert!(
            e.fly_to([1.0, 2.0, 3.0], 0.3, 0.2, 12.0, 25.0, 1.0, 0)
                .is_ok()
        );
        assert!(e.fly_state_done(), "瞬移后即 done");
        assert!((e.rig.target[0] - 1.0).abs() < 1e-9, "瞬移落位");
        assert!((e.rig.near - near0).abs() < 1e-12, "near 恒 from 现值");
        // 域拒：dist<=0 / 非有限 / fov>=π
        assert!(e.fly_to([0.0; 3], 0.0, 0.0, -1.0, 25.0, 1.0, 500).is_err());
        assert!(
            e.fly_to([0.0; 3], 0.0, 0.0, 10.0, 25.0, f64::NAN, 500)
                .is_err()
        );
        assert!(e.fly_to([0.0; 3], 0.0, 0.0, 10.0, 25.0, 4.0, 500).is_err());
        // 起飞 200ms：进度单调 → 到达落位精确（t>=1 直返 to 原值）
        assert!(
            e.fly_to([5.0, 5.0, 0.0], 0.7, 0.5, 40.0, 30.0, 1.1, 200)
                .is_ok()
        );
        assert!(!e.fly_state_done());
        let p1 = e.fly_progress().expect("t1");
        std::thread::sleep(std::time::Duration::from_millis(120));
        let _ = e.render();
        let p2 = e.fly_progress().unwrap_or(1.0);
        assert!(p2 >= p1, "进度单调 {p1}->{p2}");
        let mut guard = 0;
        while !e.fly_state_done() && guard < 60 {
            let _ = e.render();
            std::thread::sleep(std::time::Duration::from_millis(20));
            guard += 1;
        }
        assert!((e.rig.target[0] - 5.0).abs() < 1e-9, "到达落位精确");
        assert!((e.rig.dist - 40.0).abs() < 1e-9, "dist 全量落位");
        // 输入打断（A 派）：pointer-down cancel → done+无进度+可再飞；键(5)不打断
        assert!(e.fly_to([0.0; 3], 0.0, 0.0, 50.0, 30.0, 1.1, 5000).is_ok());
        assert!(e.apply_input(5, 0.0, 0.0, 0.0).is_some(), "键输入消费");
        assert!(!e.fly_state_done(), "键不打断飞行 [定案 A 派]");
        assert!(
            e.apply_input(2, 10.0, 10.0, 0.0).is_some(),
            "指针按下仍消费"
        );
        assert!(e.fly_state_done(), "cancel 即不在飞（idle/done 合并）");
        assert!(e.fly_progress().is_none(), "cancel 后无进度");
        // 改道连续性：leg1 中途 → leg2 from=当前采样 ⇒ 首帧零跳变
        assert!(
            e.fly_to([9.0, 9.0, 0.0], 0.7, 0.5, 44.0, 30.0, 1.1, 3000)
                .is_ok()
        );
        std::thread::sleep(std::time::Duration::from_millis(60));
        let _ = e.render();
        let mid = e.rig.target;
        assert!(
            e.fly_to([-9.0, -9.0, 0.0], 0.7, 0.5, 44.0, 30.0, 1.1, 3000)
                .is_ok()
        );
        let _ = e.render();
        let d = (e.rig.target[0] - mid[0]).abs();
        assert!(d < 0.5, "改道首帧连续（跳距 {d}）");
    }

    // spec: CAPI-25
    #[test]
    fn map_set_render_routing_and_domains() {
        let mut e = Engine::new_headless(200, 100).expect("adapter");
        let (pos, _nrm, idx) = quad();
        let big: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 20.0, p[1] * 20.0, 0.0])
            .collect();
        e.add_mesh(&big, None, &idx, [0.1, 0.6, 0.2, 1.0], [0.0; 3])
            .expect("ground"); // 首件位形可=0（CAPI-01 合法分工，成败看 rc——此坑第三见记账）
        // 域拒三连（fx+fw>1 / zoom 非正 / 脏 NaN）
        assert!(
            e.set_map(Some((0.7, 0.0, 0.4, 0.4)), 40.0).is_err(),
            "越 1 界须拒"
        );
        assert!(
            e.set_map(Some((0.7, 0.0, 0.25, 0.25)), 0.0).is_err(),
            "zoom 域"
        );
        assert!(
            e.set_map(Some((f32::NAN, 0.0, 0.2, 0.2)), 30.0).is_err(),
            "frac 域"
        );
        assert!(e.map_rect().is_none(), "拒后零残留");
        // navigate 无图=拒（无暗改道）
        assert!(e.navigate_click(150.0, 50.0, 500).is_err());
        // 开图（右 1/4 竖幅）+ 渲染：map 区与主区皆有内容（非底色像素）
        e.set_map(Some((0.72, 0.02, 0.26, 0.96)), 30.0)
            .expect("map on");
        e.render().expect("render dual");
        let img = e.frame_cache.clone().expect("cache");
        let bg = |i: usize| img[i * 4] < 30 && img[i * 4 + 1] < 40 && img[i * 4 + 2] < 60;
        let nonbg = |x: u32, y: u32| !bg((y * 200 + x) as usize);
        let cnt = |x0: u32, x1: u32| -> u32 {
            (10..90)
                .flat_map(|y| (x0..x1).filter(move |x| nonbg(*x, y)))
                .count() as u32
        };
        let (main_c, map_c) = (cnt(10, 130), cnt(150, 195));
        println!("PROBE map main={main_c} map={map_c}");
        assert!(main_c > 200, "主视地面缺席 {main_c}");
        assert!(map_c > 200, "小图顶视地面缺席 {map_c}");
        // 路由：小图区按下=消费且位姿不变；主区拖动=yaw 变
        let yaw0 = e.camera_pose().yaw;
        assert_eq!(
            e.apply_input(2, 180.0, 50.0, 0.0),
            Some(true),
            "小图按下消费"
        );
        let _ = e.apply_input(1, 190.0, 50.0, 0.0);
        assert!((e.camera_pose().yaw - yaw0).abs() < 1e-12, "小图拖 no-op");
        let _ = e.apply_input(2, 60.0, 50.0, 0.0);
        let _ = e.apply_input(1, 90.0, 50.0, 0.0);
        assert!((e.camera_pose().yaw - yaw0).abs() > 1e-6, "主区拖生效");
        // 小图 pick 命中地面实体（顶视中心=主 target 附近）
        let hit = e.pick(180.0, 50.0);
        assert!(hit.is_some(), "小图内 pick 须命中");
        // 关图回旧路（canary 构造位）+ rect 消失
        e.set_map(None, 0.0).expect("clear");
        assert!(e.map_rect().is_none());
        e.render().expect("render legacy again");
    }

    // spec: CAPI-27
    #[test]
    fn camera_pose_readback_is_single_source_of_truth() {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        let p0 = e.camera_pose();
        // idle 读=稳态；瞬移后读=新值（宿主 HUD 断言面）
        assert!(e.fly_state_done());
        assert!((e.camera_pose().target[0] - p0.target[0]).abs() < 1e-12);
        let tgt = [3.0, 4.0, 0.0];
        e.fly_to(tgt, p0.yaw, p0.pitch, p0.dist, p0.zoom, p0.fov_y, 0)
            .expect("tp");
        let p1 = e.camera_pose();
        assert_eq!(
            [p1.target[0], p1.target[1]],
            [3.0, 4.0],
            "读回=落位 [CAPI-27]"
        );
        assert!((p1.dist - p0.dist).abs() < 1e-12, "保距");
    }

    // spec: CAPI-26
    #[test]
    fn navigate_click_flies_to_ground_target() {
        let mut e = Engine::new_headless(200, 100).expect("adapter");
        let (pos, _nrm, idx) = quad();
        let big: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 20.0, p[1] * 20.0, 0.0])
            .collect();
        e.add_mesh(&big, None, &idx, [0.1, 0.6, 0.2, 1.0], [0.0; 3])
            .expect("ground");
        e.set_map(Some((0.5, 0.0, 0.5, 1.0)), 40.0).expect("map");
        let t0 = e.camera_pose();
        // 小图左上 1/4 处点击 = 世界 (target + (−0.25·2·zoom_x, +0.25·2·zoom_y))
        // rect=(100,0,100,100)；local 点击 (25,75)→NDC(−0.5,+0.5)→Δworld(−40, +20·aspect…以 ortho 定义)
        e.navigate_click(125.0, 75.0, 0).expect("nav teleport");
        let t1 = e.camera_pose();
        assert!(
            (t1.target[0] - t0.target[0]).abs() > 1.0,
            "target 必移动 got {t1:?}"
        );
        assert!((t1.target[2]).abs() < 1e-9, "落位 z=0");
        assert!(
            (t1.dist - t0.dist).abs() < 1e-12 && (t1.yaw - t0.yaw).abs() < 1e-12,
            "保距保角 [裁决 e]"
        );
        // 区外点击=拒且零改
        let t2 = e.camera_pose();
        assert!(
            e.navigate_click(50.0, 50.0, 0).is_err(),
            "主视区点击走此口须拒"
        );
        let t3 = e.camera_pose();
        assert!((t3.target[0] - t2.target[0]).abs() < 1e-12);
        // 带时长的飞：done 收敛 + 位姿=目标
        e.navigate_click(175.0, 25.0, 1).expect("nav fly dur=1ms");
        assert!(!e.fly_state_done() || e.camera_pose().target != t3.target);
        let mut guard = 0;
        while !e.fly_state_done() && guard < 200 {
            let _ = e.render();
            std::thread::sleep(std::time::Duration::from_millis(5));
            guard += 1;
        }
        assert!(e.fly_state_done(), "飞行必到达");
    }

    // spec: CAPI-13
    #[test]
    fn hide_filters_pick_keeps_enumeration() {
        let mut e = eng();
        let hit = e.pick(80.0, 60.0).expect("中心命中（CAPI-04 族）");
        let h = enc_entity(hit);
        assert_eq!(e.set_visible(h, true), Ok(()), "默认可见域重复设=幂等");
        assert_eq!(e.set_visible(h, false), Ok(()));
        assert_eq!(e.set_visible(h, false), Ok(()), "重复隐藏幂等");
        assert_ne!(
            e.pick(80.0, 60.0).map(enc_entity).unwrap_or(u64::MAX),
            h,
            "隐藏后中心不得命中本体"
        );
        assert_eq!(e.entity_count(), 2, "枚举域不变");
        assert!(
            (0..2u32).any(|k| e.entity_at(k) == Some(hit)),
            "隐藏件仍可枚举"
        );
        assert_eq!(e.is_visible(h), Some(false));
    }

    // spec: CAPI-14
    #[test]
    fn visible_getter_roundtrip_and_unknown() {
        let mut e = eng();
        let h0 = enc_entity(e.entity_at(0).unwrap());
        assert_eq!(e.is_visible(h0), Some(true), "默认可见");
        assert_eq!(e.set_visible(h0, false), Ok(()));
        assert_eq!(e.is_visible(h0), Some(false));
        assert_eq!(
            e.is_visible(u64::MAX >> 1),
            None,
            "未知位形=None（FFI 层投影 -1）"
        );
    }

    // spec: CAPI-15
    #[test]
    fn add_mesh_enumeration_and_degenerate_no_partial() {
        let mut e = eng();
        let (pos, nrm, idx) = quad();
        let h = e
            .add_mesh(
                &pos,
                Some(&nrm),
                &idx,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0],
            )
            .expect("add ok");
        assert_ne!(h, 0, "0=失败哨兵，成功必给位形");
        assert_eq!(e.entity_count(), 3);
        assert!(e.render().is_ok(), "加入件即入命令流");
        // 退化三路：索引越界 / 空几何 / normals 长度失配——均无部分提交
        assert!(
            e.add_mesh(&pos, Some(&nrm), &[0, 9, 2], [1.0; 4], [0.0; 3])
                .is_err()
        );
        assert!(e.add_mesh(&[], None, &[], [1.0; 4], [0.0; 3]).is_err());
        assert!(
            e.add_mesh(&pos, Some(&[[0.0, 0.0, 1.0]; 3]), &idx, [1.0; 4], [0.0; 3])
                .is_err()
        );
        assert_eq!(e.entity_count(), 3, "退化零提交");
        assert!(e.remove_entity(h).is_ok());
        assert_eq!(e.entity_count(), 2);
    }

    // spec: CAPI-16
    #[test]
    fn remove_reentry_and_aba_isolation() {
        let mut e = eng();
        let h0 = enc_entity(e.entity_at(0).unwrap());
        assert_eq!(e.remove_entity(h0), Ok(()));
        assert_eq!(e.entity_count(), 1);
        assert!(e.remove_entity(h0).is_err(), "旧句柄再入=VE_ERR_ARG 谱");
        assert_eq!(e.is_visible(h0), None, "已删位形查询同谱拒绝");
        let (pos, nrm, idx) = quad();
        let h_new = e
            .add_mesh(&pos, Some(&nrm), &idx, [1.0; 4], [0.0; 3])
            .expect("同槽再_spawn：代际+1 新位形");
        assert_ne!(h_new, h0, "ABA：旧句柄永不撞新代行");
        assert!(e.remove_entity(h0).is_err(), "新实体在场，旧句柄仍死");
        assert_eq!(e.remove_entity(h_new), Ok(()));
    }
}
