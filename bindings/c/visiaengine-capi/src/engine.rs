//! VeEngine 实体（计划 v1.4 §3）：headless 出图真身（I2）。
//! 装载=CPU 解析 → MeshCore 上传；拾取=REND-21..24 复用；策略=输入→相机。
//! bytes-first load 口为批 7 预留（[FFI-R:EP-附]/P2）：I2 走 path，J1 前升 bytes。

use visiaengine_core::{EntityId, Scene};

use crate::ffi::enc_entity; // CAPI-10 反标键=宿主可见位形（CAPI-08 编码单源）
use visiaengine_io_text::GLYPH_ATLAS_PX;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MaterialId, MeshCandidate, MeshDesc,
    MeshId, PointMark, PointTableDesc, RenderBackend, StrokeSeg, StrokeTableDesc, Viewport,
    pick_meshes, screen_to_ray_ortho, screen_to_ray_persp,
};
use visiaengine_render_wgpu::HeadlessBackend;
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
        let ray = match self.mode {
            Proj::Persp => screen_to_ray_persp(&self.rig, px, py, self.w as f32, self.h as f32)?,
            Proj::Ortho => screen_to_ray_ortho(&self.rig, px, py, self.w as f32, self.h as f32)?,
        };
        // CAPI-20 重试环：命中点被裁=该件排除，继续向后找（剖开可见者必可拾）。
        // 最坏收敛=候选数轮（每轮至少排除一件），命中图深度量级，ponytail 可接受。
        let mut skip: Vec<EntityId> = Vec::new();
        loop {
            let cands: Vec<MeshCandidate> = self
                .items
                .iter()
                .filter(|it| {
                    !self.hidden.contains(&enc_entity(it.entity)) // CAPI-13 pick 域
                        && !skip.contains(&it.entity)
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
            return Some(hit.entity);
        }
    }

    /// CAPI-13: 显隐切换（严格存在域校验；render/pick 过滤、枚举域不变）。
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
        let pose = |t: [f64; 3], d: f64| -> Result<u64, String> {
            e.fly_to(t, 0.7, 0.5, d, 30.0, 1.1, 200)
        };
        // dur=0 = 瞬移形（立即落位，非错误）
        assert!(e.fly_to([1.0, 2.0, 3.0], 0.3, 0.2, 12.0, 25.0, 1.0, 0).is_ok());
        assert_eq!(e.fly_state_done(), true, "瞬移后即 done");
        assert!((e.rig.target[0] - 1.0).abs() < 1e-9, "瞬移落位");
        assert!((e.rig.near - 0.1).abs() < 1e-12, "near/far 恒 from 现值 [A1]");
        // 域拒：dist<=0 / 非有限
        assert!(e.fly_to([0.0; 3], 0.0, 0.0, -1.0, 25.0, 1.0, 500).is_err());
        assert!(e.fly_to([0.0; 3], 0.0, 0.0, 10.0, 25.0, f64::NAN, 500).is_err());
        // 起飞 200ms：进度单调 → 到达位姿精确
        pose([5.0, 5.0, 0.0], 40.0).expect("fly");
        assert!(!e.fly_state_done());
        let p1 = e.fly_progress().expect("t1");
        std::thread::sleep(std::time::Duration::from_millis(120));
        let _ = e.render();
        let p2 = e.fly_progress().unwrap_or(1.0);
        assert!(p2 >= p1, "进度单调 {p1}->{p2}");
        while !e.fly_state_done() {
            let _ = e.render();
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!((e.rig.target[0] - 5.0).abs() < 1e-9, "到达落位精确（t>=1 直返 to）");
        // 输入打断（MapLibre A 派）：飞行中 pointer-down = cancel → done 且可再飞
        pose([0.0, 0.0, 0.0], 50.0).expect("refly");
        assert!(e.apply_input(2, 10.0, 10.0, 0.0).is_some(), "输入仍消费");
        assert!(e.fly_state_done(), "cancel 即不在飞（idle/done 合并）");
        assert!(e.fly_progress().is_none(), "cancel 后无进度可写");
        // 飞行中改道：from=当前采样位姿（连续性：新飞 t=0 == 改道瞬间 rig，不跳变）
        pose([9.0, 9.0, 0.0], 44.0).expect("leg1");
        std::thread::sleep(std::time::Duration::from_millis(60));
        let _ = e.render();
        let mid = e.rig.target;
        pose([-9.0, -9.0, 0.0], 44.0).expect("leg2 改道");
        let _ = e.render();
        assert!(
            (e.rig.target[0] - mid[0]).abs() < 1.0,
            "改道首帧必连续（跳变={:?}→{:?}）",
            mid,
            e.rig.target
        );
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
