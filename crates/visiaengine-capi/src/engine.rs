//! VeEngine 实体（计划 v1.4 §3）：headless 出图真身（I2）。
//! 装载=CPU 解析 → MeshCore 上传；拾取=REND-21..24 复用；策略=输入→相机。
//! bytes-first load 口为批 7 预留（[FFI-R:EP-附]/P2）：I2 走 path，J1 前升 bytes。

use visiaengine_core::{EntityId, Scene};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialId, MeshCandidate, MeshDesc, MeshId,
    RenderBackend, Viewport, pick_meshes, screen_to_ray_ortho, screen_to_ray_persp,
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
    frame_cache: Option<Vec<u8>>,
}

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
            frame_cache: None,
        })
    }

    fn upload(
        &mut self,
        entity: EntityId,
        positions: &[[f32; 3]],
        indices: &[u32],
        color: [f32; 4],
        origin: [f64; 3],
    ) -> Result<(), String> {
        let mesh = self
            .backend
            .create_mesh(&MeshDesc {
                positions,
                normals: &vec![[0.0, 0.0, 1.0]; positions.len()],
                indices,
                uv: &[],
            })
            .map_err(|e| format!("create_mesh: {e:?}"))?;
        let material = self
            .backend
            .create_material(color)
            .map_err(|e| format!("create_material: {e:?}"))?;
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
        for e in doc.entities() {
            let id = self.scene.spawn();
            // 局部顶点不变；headless persp 相机 ±10 域，twoprim/hierarchy 同尺度合法
            self.upload(
                id,
                &e.mesh.positions,
                &e.mesh.indices,
                e.mesh.base_color,
                [0.0; 3],
            )?;
        }
        Ok(n)
    }

    /// GeoJSON path 口（CAPI-04）。
    pub fn load_geojson(&mut self, path: &str) -> Result<usize, String> {
        let doc = visiaengine_geo::load_geojson(path).map_err(|e| format!("{path}: {e}"))?;
        self.mount_geo(&doc)
    }

    /// GeoJSON bytes 口（js 面；Lenient 策略——脏件丢弃不因数据拖死整层，
    /// 报告导出=M2）。
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    pub fn load_geojson_bytes(&mut self, data: &[u8]) -> Result<usize, String> {
        let (doc, _rep) =
            visiaengine_geo::parse_geojson_lenient(data).map_err(|e| format!("bytes: {e}"))?;
        self.mount_geo(&doc)
    }

    /// feature 粒度=实体（一件可多 part 共享 id）；layer bbox 中心
    /// =origin（D7 shifted 纪律，geo_viewer 同款）；正交 fit 取景。
    fn mount_geo(&mut self, doc: &visiaengine_geo::GeoDocument) -> Result<usize, String> {
        let [x0, y0, x1, y1] = doc.layer_bbox().ok_or("empty layer")?;
        let origin = [(x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0];
        let radius = ((x1 - x0) / 2.0).max((y1 - y0) / 2.0) * FIT_PAD;
        self.zoom = radius;
        self.rig = CameraRig::look_at(
            [origin[0], origin[1], radius.max(10.0)],
            origin,
            [0.0, 1.0, 0.0],
        );
        self.mode = Proj::Ortho;
        let neg = [-origin[0], -origin[1]];
        let mut n = 0usize;
        for f in doc.features() {
            let id = self.scene.spawn();
            n += 1;
            let parts = visiaengine_geo::tessellate(&f.kind.shifted(neg), &f.style)
                .map_err(|e| format!("tessellate: {e}"))?;
            for p in parts {
                if p.positions.is_empty() || p.indices.len() < 3 {
                    continue;
                }
                self.upload(id, &p.positions, &p.indices, p.color, origin)?;
            }
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
            commands.push(DrawCommand::DrawMesh {
                mesh: it.mesh,
                material: it.material,
                origin: it.origin,
                transform: IDENTITY,
            });
        }
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
        let cands: Vec<MeshCandidate> = self
            .items
            .iter()
            .map(|it| MeshCandidate {
                entity: it.entity,
                positions: &it.positions,
                indices: &it.indices,
                world: &IDENTITY,
            })
            .collect();
        pick_meshes(ray, &cands).map(|hit| hit.entity)
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
            frame_cache: None,
        })
    }
}

const IDENTITY: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
