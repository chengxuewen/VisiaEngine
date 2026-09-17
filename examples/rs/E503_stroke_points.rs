//! E503 · 材质与光影·扩片族独立课 —— 屏幕空间线宽/真圆点的缩放恒宽秀（REND-29/30 卖点）。
//! 三笔白色描边（横/纵/斜，1.5/4/10px 族）+ 四点半径梯度（4..12px）；滚轮/z/x 缩放
//! 时**世界长宽剧变、屏幕粗细恒等**=px_world_scale 单字段驱动的重建零成本证明。
//! 用法: cargo run --example E503_stroke_points [选项]（--frames N=ctest/xvfb 路）
//! 交互：左键拖=转轨道 | 滚轮=远近 | z/x=地图缩放 | 关窗退出。

use std::sync::Arc;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, PointMark, PointTableDesc, StrokeSeg, StrokeTableDesc,
    Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const CLEAR: [f32; 4] = [0.05, 0.07, 0.10, 1.0];
const IDENT: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// 三笔线族（世界坐标，粗→细各一）与四点点族（半径梯度）。
fn segs() -> Vec<StrokeSeg> {
    let white = [0.92, 0.94, 0.96];
    let warm = [1.0, 0.75, 0.35];
    vec![
        StrokeSeg::new([-2.4, -1.4, 0.0], [2.4, -1.4, 0.0], white, 10.0),
        StrokeSeg::new([-1.2, -0.6, 0.0], [1.6, 1.6, 0.0], warm, 4.0),
        StrokeSeg::new([2.0, -1.6, 0.0], [2.0, 1.8, 0.0], white, 1.5),
    ]
}
fn marks() -> Vec<PointMark> {
    let green = [0.30, 0.80, 0.45];
    [(-2.2, 1.6), (-0.6, 0.4), (0.8, -0.4), (2.2, 0.9)]
        .iter()
        .enumerate()
        .map(|(i, (x, y))| PointMark::new([*x, *y, 0.0], green, 4.0 + 2.7 * i as f32))
        .collect()
}

struct App {
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    window: Option<Arc<Window>>,
    dragging: Option<(f64, f64)>,
    rig: CameraRig,
    strokes: Vec<DrawCommand>,
    points: Vec<DrawCommand>,
    frames_left: Option<u32>,
}

impl App {
    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
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
                            "VisiaEngine E503 · 扩片族（缩放恒粗！拖=轨道 滚轮=远近 z/x=缩放 关窗退出）",
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
        config.format = caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
                )
            })
            .unwrap_or(caps.formats[0]);
        surface.configure(&core.device, &config);

        let s = segs();
        let m = marks();
        let table_s = core
            .create_strokes(&StrokeTableDesc { data: &s })
            .expect("stroke table");
        let table_p = core
            .create_points(&PointTableDesc { data: &m })
            .expect("point table");
        self.strokes = vec![DrawCommand::DrawStrokes {
            table: table_s,
            origin: [0.0; 3],
            transform: IDENT,
        }];
        self.points = vec![DrawCommand::DrawPoints {
            table: table_p,
            origin: [0.0; 3],
            transform: IDENT,
        }];
        println!("loaded {} strokes + {} points", s.len(), m.len());

        self.window = Some(window);
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
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
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match &event.logical_key {
                        Key::Character(s) if s.as_str() == "z" => self.rig.zoom *= 1.15,
                        Key::Character(s) if s.as_str() == "x" => self.rig.zoom /= 1.15,
                        Key::Named(NamedKey::Escape) => event_loop.exit(),
                        _ => {}
                    }
                    self.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.dragging = matches!(state, ElementState::Pressed).then_some((-1.0, -1.0));
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((lx, ly)) = self.dragging {
                    let (x, y) = (position.x, position.y);
                    if lx >= 0.0 {
                        self.rig.orbit_delta((x - lx) * 0.006, (y - ly) * 0.006);
                        self.request_redraw();
                    }
                    self.dragging = Some((x, y));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                let f = 1.0 - d * 0.0012;
                self.rig.zoom = (self.rig.zoom * f).clamp(0.5, 2000.0);
                self.rig.dist = (self.rig.dist * f).clamp(5.0, 800.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
                    event_loop.exit();
                    return;
                };
                let mut commands = vec![DrawCommand::ClearColor { rgba: CLEAR }];
                commands.extend(self.strokes.clone());
                commands.extend(self.points.clone());
                let aspect = config.width as f32 / config.height.max(1) as f32;
                let (near, far) = ((self.rig.dist * 0.01) as f32, (self.rig.dist * 30.0) as f32);
                let Some(proj) = self.rig.ortho_frame(
                    self.rig.zoom as f32,
                    config.width as f32,
                    config.height as f32,
                    near,
                    far,
                ) else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(config.width, config.height, 1.0),
                    camera: Camera::ortho(
                        self.rig.zoom as f32,
                        self.rig.zoom as f32 * aspect,
                        near,
                        far,
                    ),
                    view_rot: self.rig.view_rotation(),
                    eye: self.rig.eye(),
                    proj,
                    // 顶视 ortho 精确路（REND-29）：半宽=zoom → 2·zoom 世界铺 width 像素
                    px_world_scale: 2.0 * self.rig.zoom as f32 / config.width.max(1) as f32,
                    shadow: None,
                    clip: None,
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
        dragging: None,
        // 顶视基（pitch=1.5≈正俯）：扩片族屏幕恒粗语义在透视/正交两态同真
        rig: CameraRig::orbit([0.0, 0.0, 0.0], 0.0, 1.5, 12.0, 2.6, 1.0, 0.1, 100.0),
        strokes: Vec::new(),
        points: Vec::new(),
        frames_left: frames,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
