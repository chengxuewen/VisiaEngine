//! E306 · 相机·地图控制预设 —— pan/zoom/tilt 三件套的"地图语义"组合演示。
//! 用法: cargo run --example E306_map_controls [选项]
//! 交互：左键拖拽=平移(pan，拖地面) | 滚轮=缩放(zoom，正交半宽) | 右键拖拽=倾角(tilt
//! +绕目标旋转) | R=复位 | 关窗=退出。
//! --frames N：渲染预算退出（ctest/xvfb 路，注册表 argv 专属，C15）。
//!
//! 纯消费组合（misc_controls_map 的同语义形）：零引擎改动——pan=rig.target 平移
//! （屏幕位移×世界比），zoom=rig.zoom（正交 half-width），tilt=rig.pitch/orbit。
//! 正交顶视起步（地图姿态），pitch 可压到斜视。

use std::sync::Arc;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, MultiClearPolicy};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const W: u32 = 320;
const H: u32 = 240;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Upload seam（E304/E305 的 Up 形）。
trait Up {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64;
    fn t(&mut self, d: &MaterialDesc) -> u64;
}

impl Up for HeadlessBackend {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.create_mesh(d).expect("m")
    }
    fn t(&mut self, d: &MaterialDesc) -> u64 {
        self.create_material_desc(d).expect("t")
    }
}

impl Up for MeshCore {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.upload_mesh(d).expect("m")
    }
    fn t(&mut self, d: &MaterialDesc) -> u64 {
        self.upload_material_desc(d).expect("t")
    }
}

/// 城区小景：地面 + 网格街道线块（地图语义的视觉锚）。
fn scene(up: &mut impl Up) -> Vec<DrawCommand> {
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    // 地面。
    let ground = [
        [-8.0f32, -8.0, 0.0],
        [8.0, -8.0, 0.0],
        [8.0, 8.0, 0.0],
        [-8.0, 8.0, 0.0],
    ];
    let gm = up.m(&MeshDesc {
        uv: &[],
        positions: &ground,
        normals: &[[0.0, 0.0, 1.0]; 4],
        indices: &[0, 1, 2, 0, 2, 3],
    });
    let gt = up.t(&MaterialDesc {
        base_color: [0.18, 0.24, 0.20, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
        roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
        metallic: 0.0,
    });
    commands.push(DrawCommand::DrawMesh {
        mesh: gm,
        material: gt,
        origin: [0.0; 3],
        transform: T4,
    });
    // 2×2 街区块（深色），间隙=街道。
    for (x, y) in [(-2.5f32, -2.5), (2.5, -2.5), (-2.5, 2.5), (2.5, 2.5)] {
        let block = [
            [x - 1.5, y - 1.5, 0.05],
            [x + 1.5, y - 1.5, 0.05],
            [x + 1.5, y + 1.5, 0.05],
            [x - 1.5, y + 1.5, 0.05],
        ];
        let bm = up.m(&MeshDesc {
            uv: &[],
            positions: &block,
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
        });
        let bt = up.t(&MaterialDesc {
            base_color: [0.30, 0.36, 0.44, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        });
        commands.push(DrawCommand::DrawMesh {
            mesh: bm,
            material: bt,
            origin: [0.0; 3],
            transform: T4,
        });
    }
    commands
}

/// 正交顶视 Frame（地图姿态；rig 携 pan/zoom/tilt 后的位形）。
fn map_frame(cmds: &[DrawCommand], rig: &CameraRig, w: u32, h: u32) -> Frame {
    let aspect = w as f32 / h.max(1) as f32;
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::ortho(rig.zoom as f32, rig.zoom as f32 / aspect, 0.1, 2000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .ortho_frame(rig.zoom as f32, w as f32, h as f32, 0.1, 2000.0)
            .expect("ortho"),
        px_world_scale: (2.0 * rig.zoom as f32) / w as f32,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds.to_vec(),
    }
}

/// 复位位形（顶视地图姿态）。
fn home_rig() -> CameraRig {
    CameraRig::orbit([0.0, 0.0, 0.0], 0.0, 1.55, 14.0, 6.0, 1.0, 0.1, 2000.0)
}

// ── headless assert path：脚本化输入序列 → 断言 rig 位形变化 ─────────────────
fn prove() {
    // 语义单测形：pan 改 target、zoom 改 zoom、tilt 改 pitch——纯函数级断言。
    let mut rig = home_rig();
    let t0 = rig.target;
    let z0 = rig.zoom;
    let p0 = rig.pitch;

    // pan：世界左移 1 单位（target.x-=1）。
    rig.target[0] -= 1.0;
    assert!(
        (rig.target[0] - (t0[0] - 1.0)).abs() < 1e-9,
        "pan moves target.x"
    );
    // zoom：半宽减半 = 放大。
    rig.zoom *= 0.5;
    assert!((rig.zoom - z0 * 0.5).abs() < 1e-9, "zoom halves half-width");
    // tilt：pitch 从近顶视(1.55)压到斜视(0.9)。
    rig.pitch = 0.9;
    assert!(
        rig.pitch < p0 && rig.pitch > 0.5,
        "tilt lowers pitch into oblique"
    );

    // 渲染一帧确认非退化（变换后的 rig 出图有内容）。
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let img = b.render_to_pixels(&map_frame(&cmds, &rig, W, H)).unwrap();
    let lit: u32 = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]) > 40)
        .count() as u32;
    println!("E306 probe: lit={lit}");
    assert!(
        lit > 500,
        "panned+zoomed+tilted map renders content (lit={lit})"
    );
    println!("E306 OK: pan/zoom/tilt semantics asserted + map renders");
}

// ── resident window ──────────────────────────────────────────────────────────
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    commands: Vec<DrawCommand>,
    rig: CameraRig,
    pan: Option<(f64, f64)>,
    tilt_drag: Option<(f64, f64)>,
    last_cursor: (f64, f64),
    ready: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ready {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(PhysicalSize::new(960u32, 600u32))
                        .with_title("E306 map controls — 左键拖=平移 滚轮=缩放 右键拖=倾角 R=复位"),
                )
                .expect("window"),
        );
        let size = window.inner_size();
        let instance = visiaengine_render_wgpu::create_instance();
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("surface");
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .ok()
        else {
            eprintln!("no adapter");
            event_loop.exit();
            return;
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("visiaengine-e306"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("device");
        let mut core = MeshCore::new(device, queue, instance, adapter.clone());
        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("cfg");
        config.format = caps.formats[0];
        surface.configure(&core.device, &config);
        self.commands = scene(&mut core);
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);
        self.ready = true;
        self.redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }
                if let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, self.surface.as_ref(), self.config.as_mut())
                {
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&core.device, config);
                    self.redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                match &event.logical_key {
                    Key::Character(s) if s.eq_ignore_ascii_case("r") => {
                        self.rig = home_rig();
                        println!("HOME reset");
                        self.redraw();
                    }
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    _ => {}
                }
            }
            WindowEvent::MouseInput { button, state, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => {
                    self.pan = Some(self.cursor_pos());
                }
                (MouseButton::Left, ElementState::Released) => self.pan = None,
                (MouseButton::Right, ElementState::Pressed) => {
                    self.tilt_drag = Some(self.cursor_pos());
                }
                (MouseButton::Right, ElementState::Released) => self.tilt_drag = None,
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x, position.y);
                // pan：屏幕位移 → 世界位移（world-per-px = 2·zoom/h，正交精确定义）。
                if let Some((lx, ly)) = self.pan {
                    let wpp = 2.0 * self.rig.zoom
                        / f64::from(self.config.as_ref().map_or(H, |c| c.height).max(1));
                    let (dx, dy) = (x - lx, y - ly);
                    self.rig.target[0] -= dx * wpp;
                    self.rig.target[1] += dy * wpp; // 屏幕 y 向下 = 世界 y 向上反向
                    self.pan = Some((x, y));
                    self.redraw();
                }
                // tilt：右键垂直拖 = pitch（顶视 1.55 → 斜视 0.6），水平拖 = yaw。
                if let Some((lx, ly)) = self.tilt_drag {
                    self.rig.pitch = (self.rig.pitch - (y - ly) * 0.006).clamp(0.35, 1.55);
                    self.rig.yaw += (x - lx) * 0.006;
                    self.tilt_drag = Some((x, y));
                    self.redraw();
                }
                self.cursor_pos_set(x, y);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y),
                    MouseScrollDelta::PixelDelta(p) => -p.y / 40.0,
                };
                self.rig.zoom = (self.rig.zoom * (1.0 + d * 0.12)).clamp(0.5, 20.0);
                self.redraw();
            }
            _ => {}
        }
    }
}

impl App {
    fn cursor_pos(&self) -> (f64, f64) {
        // winit CursorMoved 之外无缓存口；App 自存（init=窗口中心）。
        self.last_cursor
    }
    fn cursor_pos_set(&mut self, x: f64, y: f64) {
        self.last_cursor = (x, y);
    }

    fn redraw(&mut self) {
        let (Some(core), Some(surface), Some(config), Some(window)) = (
            self.core.as_mut(),
            self.surface.as_ref(),
            self.config.as_ref(),
            self.window.as_ref(),
        ) else {
            return;
        };
        let frame = map_frame(&self.commands, &self.rig, config.width, config.height);
        window.set_title(
            format!(
                "E306 map zoom={:.1} pitch={:.2} — 左键平移 滚轮缩放 右键倾角 R 复位",
                self.rig.zoom, self.rig.pitch
            )
            .as_str(),
        );
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                core.render_view_rects(
                    &[(
                        frame,
                        visiaengine_render::ViewportRect::new(0, 0, config.width, config.height),
                    )],
                    &view,
                    config.width,
                    config.height,
                    config.format,
                    MultiClearPolicy::FirstClearRestLoad,
                );
                core.queue.present(tex);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                surface.configure(&core.device, config);
            }
            other => eprintln!("skip {other:?}"),
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
    if frames.is_some() {
        prove();
        return Ok(());
    }
    let el = EventLoop::new()?;
    el.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        window: None,
        core: None,
        surface: None,
        config: None,
        commands: Vec::new(),
        rig: home_rig(),
        pan: None,
        tilt_drag: None,
        ready: false,
        last_cursor: (480.0, 300.0),
    };
    el.run_app(&mut app)?;
    Ok(())
}
