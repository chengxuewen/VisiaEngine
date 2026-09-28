//! E403 · 交互·橡皮筋框选 —— 左键拖拽=矩形框选（框内实体全选）演示。
//! 用法: cargo run --example E403_box_select [选项]
//! 交互：左键拖拽=框选（5×5 采样射线判定框内实体，命中即选入）| 松开=提交选择
//! 右键=清空 | R=清空 | 滚轮=远近 | 关窗=退出。
//! --frames N：渲染预算退出（ctest/xvfb 路，注册表 argv 专属，C15）。
//!
//! 框选判定=采样射线复用（REND-21/23 pick 基建，零新数学）：矩形内 5×5 网格
//! 点各发一条 pick 射线，命中的实体并入选择集。稀疏盒场景下与精确投影判定
//! 等价；大场景升级路径=视锥体裁剪纯函数（票据制）。

use std::sync::Arc;

use visiaengine_core::{EntityId, Scene};
use visiaengine_render::screen_to_ray_persp;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialId, MeshCandidate, MeshDesc, MeshId, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::unit_box_mesh;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const CLEAR: [f32; 4] = [0.05, 0.07, 0.10, 1.0];
const SELECTED: [f32; 4] = [1.0, 0.83, 0.29, 1.0]; // 高亮黄（WGPU-12 色族）

fn placed(x: f64, y: f64, z: f64, s: f64) -> [[f64; 4]; 4] {
    [
        [s, 0.0, 0.0, 0.0],
        [0.0, s, 0.0, 0.0],
        [0.0, 0.0, s, 0.0],
        [x, y, z, 1.0],
    ]
}

struct BoxRec {
    entity: EntityId,
    name: &'static str,
    world: [[f64; 4]; 4],
    selected: bool,
}

struct App {
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    window: Option<Arc<Window>>,
    rig: CameraRig,
    boxes: Vec<BoxRec>,
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    mesh: Option<MeshId>,
    mat_hl: Option<MaterialId>,
    mats: Vec<MaterialId>,
    cursor: (f64, f64),
    /// 拖拽态：起点 + 当前点（橡皮筋对角）；None=未拖拽。
    band: Option<((f64, f64), (f64, f64))>,
    frames_left: Option<u32>,
}

impl App {
    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// 屏幕坐标 → 射线 → pick_meshes → 盒索引（E402 同款 pick 路）。
    fn pick_at(&self, x: f64, y: f64) -> Option<usize> {
        let Some(config) = &self.config else {
            return None;
        };
        let ray = screen_to_ray_persp(
            &self.rig,
            x as f32,
            y as f32,
            config.width as f32,
            config.height as f32,
        )?;
        let cands: Vec<MeshCandidate> = self
            .boxes
            .iter()
            .map(|b| MeshCandidate {
                entity: b.entity,
                positions: &self.positions,
                indices: &self.indices,
                world: &b.world,
            })
            .collect();
        let hit = visiaengine_render::pick_meshes(ray, &cands)?;
        self.boxes
            .iter()
            .position(|b| b.entity == hit.entity)
            .filter(|_| hit.t.is_finite())
    }

    /// 橡皮筋矩形 → 5×5 采样射线 → 命中实体集（框选判定本体）。
    fn select_band(&mut self, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
        let (min_x, max_x) = (x0.min(x1), x0.max(x1));
        let (min_y, max_y) = (y0.min(y1), y0.max(y1));
        let mut hits: Vec<usize> = Vec::new();
        for sy in 0..5 {
            for sx in 0..5 {
                let px = min_x + (max_x - min_x) * f64::from(sx) / 4.0;
                let py = min_y + (max_y - min_y) * f64::from(sy) / 4.0;
                if let Some(i) = self.pick_at(px, py)
                    && !hits.contains(&i)
                {
                    hits.push(i);
                }
            }
        }
        let names: Vec<&str> = hits.iter().map(|&i| self.boxes[i].name).collect();
        for i in hits {
            self.boxes[i].selected = true;
        }
        println!(
            "BAND-SELECT [{}] selected_total={}",
            names.join(","),
            self.boxes.iter().filter(|b| b.selected).count()
        );
        self.request_redraw();
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: &KeyEvent) {
        if key.state != ElementState::Pressed {
            return;
        }
        match &key.logical_key {
            Key::Character(s) if s.eq_ignore_ascii_case("r") => {
                for b in &mut self.boxes {
                    b.selected = false;
                }
                println!("CLEAR selection");
                self.request_redraw();
            }
            Key::Named(NamedKey::Escape) => event_loop.exit(),
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(PhysicalSize::new(960u32, 600u32))
                        .with_title(
                            "VisiaEngine E403 · 框选（左键拖拽=框内全选 右键/R=清空 滚轮=远近）",
                        ),
                )
                .expect("create_window"),
        );
        let size = window.inner_size();
        let instance = visiaengine_render_wgpu::create_instance();
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("create_surface");
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .ok()
        else {
            eprintln!("no adapter — lavapipe/real GPU required");
            event_loop.exit();
            return;
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("visiaengine-e403"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("request_device");
        let mut core = MeshCore::new(device, queue, instance, adapter.clone());
        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface config");
        config.format = caps.formats[0];
        surface.configure(&core.device, &config);

        let (positions, normals, indices) = unit_box_mesh();
        let mesh = core
            .upload_mesh(&MeshDesc {
                positions: &positions,
                normals: &normals,
                indices: &indices,
                uv: &[],
            })
            .expect("upload mesh");
        // 6 盒 2×3 网格铺开（框选演示的密度感）。
        let palette = [
            [0.90, 0.25, 0.25, 1.0f32],
            [0.20, 0.75, 0.35, 1.0],
            [0.20, 0.45, 0.90, 1.0],
            [0.85, 0.55, 0.20, 1.0],
            [0.55, 0.20, 0.85, 1.0],
            [0.20, 0.80, 0.80, 1.0],
        ];
        let mats: Vec<MaterialId> = palette
            .iter()
            .map(|c| core.upload_material(*c).expect("upload material"))
            .collect();
        let mat_hl = core.upload_material(SELECTED).expect("upload hl");

        let mut scene = Scene::new();
        let defs: [(&str, [[f64; 4]; 4]); 6] = [
            ("A", placed(-3.0, -1.5, 1.0, 1.6)),
            ("B", placed(0.0, -1.5, 1.0, 1.6)),
            ("C", placed(3.0, -1.5, 1.0, 1.6)),
            ("D", placed(-3.0, 1.5, 1.0, 1.6)),
            ("E", placed(0.0, 1.5, 1.0, 1.6)),
            ("F", placed(3.0, 1.5, 1.0, 1.6)),
        ];
        self.boxes = defs
            .iter()
            .map(|(name, world)| BoxRec {
                entity: scene.spawn(),
                name,
                world: *world,
                selected: false,
            })
            .collect();
        println!("loaded {} entities", self.boxes.len());

        self.window = Some(window);
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.mesh = Some(mesh);
        self.mats = mats;
        self.mat_hl = Some(mat_hl);
        self.positions = positions;
        self.indices = indices;
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }
                if let (Some(core), Some(surface), Some(config)) = (
                    self.core.as_mut(),
                    self.surface.as_ref(),
                    self.config.as_mut(),
                ) {
                    config.width = size.width.max(1);
                    config.height = size.height.max(1);
                    surface.configure(&core.device, config);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.handle_key(event_loop, &event),
            WindowEvent::MouseInput { button, state, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => {
                    self.band = Some((self.cursor, self.cursor));
                }
                (MouseButton::Left, ElementState::Released) => {
                    if let Some((start, end)) = self.band.take() {
                        self.select_band(start, end);
                    }
                }
                (MouseButton::Right, ElementState::Pressed) => {
                    for b in &mut self.boxes {
                        b.selected = false;
                    }
                    println!("CLEAR selection (right-click)");
                    self.request_redraw();
                }
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                if let Some((start, _)) = self.band {
                    self.band = Some((start, self.cursor));
                    self.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                self.rig.dist = (self.rig.dist * (1.0 - d * 0.0012)).clamp(3.0, 80.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
                    event_loop.exit();
                    return;
                };
                let (Some(mesh), Some(hl)) = (self.mesh, self.mat_hl) else {
                    event_loop.exit();
                    return;
                };
                let mut commands = vec![DrawCommand::ClearColor { rgba: CLEAR }];
                for (i, b) in self.boxes.iter().enumerate() {
                    let material = if b.selected { hl } else { self.mats[i] };
                    commands.push(DrawCommand::DrawMesh {
                        mesh,
                        material,
                        origin: [0.0, 0.0, 0.0],
                        transform: b.world,
                    });
                }
                let aspect = config.width as f32 / config.height.max(1) as f32;
                let (near, far) = ((self.rig.dist * 0.01) as f32, (self.rig.dist * 30.0) as f32);
                let Some(proj) = self
                    .rig
                    .perspective(self.rig.fov_y as f32, aspect, near, far)
                else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(config.width, config.height, 1.0),
                    camera: Camera::perspective(self.rig.fov_y as f32, aspect, near, far),
                    view_rot: self.rig.view_rotation(),
                    eye: self.rig.eye(),
                    proj,
                    px_world_scale: 1.0,
                    shadow: None,
                    clip: None,
                    edl: None,
                    commands,
                };
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(tex)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                        let view = tex
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        core.render_view_format(
                            &frame,
                            &view,
                            config.width,
                            config.height.max(1),
                            config.format,
                        );
                        core.queue.present(tex);
                    }
                    wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                        surface.configure(&core.device, config);
                    }
                    other => eprintln!("skipped: {other:?}"),
                }
                let again = match self.frames_left {
                    None => true,
                    Some(1) => false,
                    Some(n) => {
                        self.frames_left = Some(n - 1);
                        true
                    }
                };
                if again {
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                } else {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut frames: Option<u32> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--frames" {
            frames = args
                .next()
                .and_then(|v| v.parse().ok())
                .filter(|n: &u32| *n > 0);
        }
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        core: None,
        surface: None,
        config: None,
        window: None,
        rig: CameraRig::orbit([0.0, 0.0, 1.0], 0.0, 1.2, 12.0, 1.0, 1.0, 0.1, 100.0),
        boxes: Vec::new(),
        positions: Vec::new(),
        indices: Vec::new(),
        mesh: None,
        mat_hl: None,
        mats: Vec::new(),
        cursor: (f64::from(960 / 2), f64::from(600 / 2)),
        band: None,
        frames_left: frames,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
