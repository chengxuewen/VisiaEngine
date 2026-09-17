//! E402 · 交互·hover/多选窗 —— 光标射线悬停预高亮 + 左键点选多选（toggle）全链演示。
//! 用法: cargo run --example E402_pick_interactive [选项]
//! 交互：左键按住拖=轨道旋转 | 松开未移动=点选(toggle 多选) | R=清空选择 | 滚轮=远近
//! 关窗=退出。--frames N：渲染预算退出（ctest/xvfb 路，注册表 argv 专属，C15）。

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
const HOVER: [f32; 4] = [1.0, 0.6, 0.2, 1.0]; // 悬停橙（选中态之下档，同族可辨）

/// 世界平移 ×缩放 s 的刚体位形（行向量约定；pick world 与 render transform 同源单份——
/// E401 双份烘移离轴 64px 教训的构造性根除）。
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
    mat_hover: Option<MaterialId>,
    mats: Vec<MaterialId>,
    cursor: (f64, f64),
    hover: Option<usize>,
    dragging: Option<(f64, f64, bool)>, // (last_x, last_y, moved)
    frames_left: Option<u32>,
}

impl App {
    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// 屏幕坐标 → 透视射线 → pick_meshes → 命中盒索引（REND-21/23 路，f64 全程）。
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

    fn toggle_at(&mut self, x: f64, y: f64) {
        if let Some(i) = self.pick_at(x, y) {
            self.boxes[i].selected = !self.boxes[i].selected;
            println!(
                "PICK {} selected={} total={}",
                self.boxes[i].name,
                self.boxes[i].selected,
                self.boxes.iter().filter(|b| b.selected).count()
            );
            self.request_redraw();
        }
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
                            "VisiaEngine E402 · hover/多选（左键点选=黄 悬停=橙 拖=轨道 R=清空 关窗退出）",
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
            label: Some("visiaengine-window"),
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
        let palette = [
            [0.90, 0.25, 0.25, 1.0f32],
            [0.20, 0.75, 0.35, 1.0],
            [0.20, 0.45, 0.90, 1.0],
        ];
        let mats: Vec<MaterialId> = palette
            .iter()
            .map(|c| core.upload_material(*c).expect("upload material"))
            .collect();
        let mat_hl = core.upload_material(SELECTED).expect("upload hl");
        let mat_hover = core.upload_material(HOVER).expect("upload hover");

        let mut scene = Scene::new();
        let defs: [(&str, [[f64; 4]; 4]); 3] = [
            ("A", placed(-3.2, 0.0, 1.1, 2.2)),
            ("B", placed(0.0, 0.6, 1.9, 2.2)),
            ("C", placed(3.2, -0.4, 1.4, 2.2)),
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
        self.mat_hover = Some(mat_hover);
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
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => match state {
                ElementState::Pressed => {
                    self.dragging = Some((self.cursor.0, self.cursor.1, false));
                }
                ElementState::Released => {
                    let click = matches!(self.dragging, Some((_, _, false)));
                    self.dragging = None;
                    if click {
                        let (x, y) = self.cursor;
                        self.toggle_at(x, y);
                    }
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                self.cursor = (x, y);
                if let Some((lx, ly, moved)) = self.dragging {
                    if moved {
                        self.rig.orbit_delta((x - lx) * 0.006, (y - ly) * 0.006);
                    }
                    if (x - lx).abs() + (y - ly).abs() > 2.0 {
                        self.dragging = Some((x, y, true));
                    }
                    self.request_redraw();
                } else {
                    // 悬停预览：每动一次一次射线（≤3 盒场景零成本；大场景升级=节流，见注）
                    let h = self.pick_at(x, y);
                    if h != self.hover {
                        self.hover = h;
                        self.request_redraw();
                    }
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
                let (Some(mesh), Some(hl), Some(hov)) = (self.mesh, self.mat_hl, self.mat_hover)
                else {
                    event_loop.exit();
                    return;
                };
                let mut commands = vec![DrawCommand::ClearColor { rgba: CLEAR }];
                for (i, b) in self.boxes.iter().enumerate() {
                    let material = if b.selected {
                        hl
                    } else if self.hover == Some(i) {
                        hov
                    } else {
                        self.mats[i]
                    };
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
                    px_world_scale: 1.0, // 三角系不读（REND-29 消费面=扩片族）
                    shadow: None,
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
        rig: CameraRig::orbit([0.0, 0.0, 1.5], 0.35, 0.95, 11.0, 1.0, 1.0, 0.1, 100.0),
        boxes: Vec::new(),
        positions: Vec::new(),
        indices: Vec::new(),
        mesh: None,
        mat_hl: None,
        mat_hover: None,
        mats: Vec::new(),
        cursor: (f64::from(960 / 2), f64::from(600 / 2)),
        hover: None,
        dragging: None,
        frames_left: frames,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
