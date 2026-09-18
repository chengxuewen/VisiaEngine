//! E302 · 相机飞行 —— 预设视角 flyTo 巡览（⑤a REND-34 交互面）。
//! 双模（E501/E901 同制）：无参=常驻窗（1/2/3=预设视角 拖/滚轮=cancel+轨道 R=回家 Esc=退）；
//! `--frames N`=确定性纯函数断言路（墙钟不进 CI 契约）。

use std::sync::Arc;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Easing, Frame, Instance, InstanceDesc, MaterialDesc, MeshDesc,
    RenderBackend, ShadowSetup, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const W: u32 = 640;
const H: u32 = 480;
const SIDE: usize = 10;
const ID64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

struct Tower {
    offset: [f32; 3],
    height: f32,
}

fn city() -> Vec<Tower> {
    let step = 2.4f32;
    let span = step * SIDE as f32 / 2.0;
    (0..SIDE)
        .flat_map(|gy| {
            (0..SIDE).map(move |gx| {
                let i = gx + gy * SIDE;
                Tower {
                    offset: [
                        -span + gx as f32 * step + step * 0.4,
                        -span + gy as f32 * step + step * 0.4,
                        -0.06,
                    ],
                    height: 1.6 + (i * 7919 % 11) as f32 * 1.15,
                }
            })
        })
        .collect()
}

/// 预设三元（头注/标题文案同源）：1 航拍 / 2 街景 / 3 最高塔顶。
fn presets() -> [CameraRig; 3] {
    let c = city();
    let tall = c
        .iter()
        .max_by(|a, b| a.height.total_cmp(&b.height))
        .unwrap();
    [
        CameraRig::orbit([0.0, 0.0, 2.0], 0.8, 0.95, 95.0, 40.0, 46.0, 0.5, 400.0),
        CameraRig::orbit([0.0, -2.0, 1.2], -2.1, 0.05, 14.0, 14.0, 60.0, 0.3, 200.0),
        CameraRig::orbit(
            [
                f64::from(tall.offset[0]),
                f64::from(tall.offset[1]),
                f64::from(tall.height) * 0.6,
            ],
            2.4,
            0.28,
            12.0,
            10.0,
            55.0,
            0.3,
            200.0,
        ),
    ]
}

fn shadow_setup(w: u32, h: u32) -> ShadowSetup {
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let leye = [
        f64::from(ld[0] / nn) * 80.0,
        f64::from(ld[1] / nn) * 80.0,
        f64::from(ld[2] / nn) * 80.0,
    ];
    let lrig = CameraRig::look_at(leye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    ShadowSetup {
        proj: lrig
            .ortho_frame(45.0, w as f32, h as f32, 1.0, 160.0)
            .expect("lp"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5,
        bias: ShadowSetup::DEFAULT_BIAS,
    }
}

fn frame_of(rig: &CameraRig, commands: Vec<DrawCommand>, w: u32, h: u32) -> Frame {
    let aspect = w as f32 / h.max(1) as f32;
    let (near, far) = (0.3f32, 400.0f32);
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, aspect, near, far),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, aspect, near, far)
            .expect("p"),
        px_world_scale: 0.12,
        shadow: Some(shadow_setup(w, h)),
        clip: None,
        commands,
    }
}

/// 场景上传（双路同源脸，REND-33 Ctor 先例——本例两脸够用不升格 trait）。
macro_rules! upload_scene {
    ($b:expr) => {{
        let b = $b;
        let mut cmds = vec![DrawCommand::ClearColor {
            rgba: [0.42, 0.58, 0.80, 1.0],
        }];
        let ground = b.m(&MeshDesc {
            positions: &[
                [-30.0, -30.0, -0.1],
                [30.0, -30.0, -0.1],
                [30.0, 30.0, -0.1],
                [-30.0, 30.0, -0.1],
            ],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        });
        let gmat = b.mt(&MaterialDesc {
            base_color: [0.62, 0.63, 0.66, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        });
        cmds.push(DrawCommand::DrawMesh {
            mesh: ground,
            material: gmat,
            origin: [0.0; 3],
            transform: ID64,
        });
        let (pos, nrm, idx) = unit_box_mesh();
        let boxy = b.m(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        });
        let bmat = b.mt(&MaterialDesc {
            base_color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        });
        let insts: Vec<Instance> = city()
            .iter()
            .map(|t| {
                let v = 0.74 + (t.height * 7.0 % 3.0) as f32 * 0.02;
                Instance::new(t.offset, t.height, [v, v * 1.02, v * 1.08])
            })
            .collect();
        let iid = b.inst(&InstanceDesc { data: &insts });
        cmds.push(DrawCommand::DrawInstances {
            mesh: boxy,
            material: bmat,
            instances: iid,
            origin: [0.0; 3],
            transform: ID64,
        });
        cmds
    }};
}

// ── headless 路（确定性：纯函数断言，零墙钟）────────────────────────────────
fn headless_run() {
    let p = presets();
    let [a, b, _c] = p;
    // 端点精确（REND-16 同谱）+ 中点不重合
    for e in [Easing::Linear, Easing::CubicInOut, Easing::CubicOut] {
        assert_eq!(CameraRig::fly_sample(&a, &b, 0.0, e).target, a.target);
        assert_eq!(CameraRig::fly_sample(&a, &b, 1.0, e).target, b.target);
        let m = CameraRig::fly_sample(&a, &b, 0.5, e);
        assert!(
            (m.dist - a.dist).abs() > 1.0 && (m.dist - b.dist).abs() > 1.0,
            "中点漂浮 {e:?}"
        );
    }
    // 大角飞行（航拍→街景 yaw 跨度大）走最短弧：中点 yaw 与线性均值两侧对比可证
    let mut wide_b = b;
    wide_b.yaw = a.yaw + 5.9; // |Δ|>π ⇒ 线性中点 ~a.yaw+2.95，最短弧中点 ~a.yaw-0.24
    let mid = CameraRig::fly_sample(&a, &wide_b, 0.5, Easing::Linear);
    assert!(
        (mid.yaw - (a.yaw - 0.23)).abs() < 0.2,
        "wrap 未生效：mid yaw {0} 期望 ~{1}",
        mid.yaw,
        a.yaw - 0.23
    );
    // 渲染路通（帧环消费 fly_sample 位姿=展示形与断言路同源）
    let mut backend = HeadlessBackend::new(W, H).expect("adapter");
    struct Face<'x>(&'x mut HeadlessBackend);
    impl Face<'_> {
        fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
            self.0.create_mesh(d).expect("m")
        }
        fn mt(&mut self, d: &MaterialDesc) -> u64 {
            self.0.create_material_desc(d).expect("mt")
        }
        fn inst(&mut self, d: &InstanceDesc<'_>) -> u64 {
            self.0.create_instances(d).expect("i")
        }
    }
    let cmds = upload_scene!(&mut Face(&mut backend));
    let img = backend
        .render_to_pixels(&frame_of(&mid, cmds, W, H))
        .expect("render");
    let bright = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|q| q[0] > 150 && q[1] > 150 && q[2] > 150)
        .count();
    assert!(bright > 40, "飞行中景亮面缺席 {bright}"); // 实测 96 的 −58% 保守位（PIT-8）
    println!("OK fly_camera endpoints+wrap+mid-render bright={bright}");
}

// ── 交互路 ───────────────────────────────────────────────────────────────────
struct Flight {
    from: CameraRig,
    to: CameraRig,
    start: std::time::Instant,
    dur_s: f64,
}

struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    rig: CameraRig,
    flight: Option<Flight>,
    dragging: Option<(f64, f64)>,
    commands: Vec<DrawCommand>,
    presets: [CameraRig; 3],
}

struct FaceRef<'x>(&'x mut MeshCore);
impl FaceRef<'_> {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.0.upload_mesh(d).expect("m")
    }
    fn mt(&mut self, d: &MaterialDesc) -> u64 {
        self.0.upload_material_desc(d).expect("mt")
    }
    fn inst(&mut self, d: &InstanceDesc<'_>) -> u64 {
        self.0.create_instances(d).expect("i")
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
                        .with_title("E302 相机飞行 · 1/2/3=预设 R=回家 拖/滚轮=中断接管 Esc=退出"),
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
            eprintln!("no adapter — lavapipe/real GPU required");
            event_loop.exit();
            return;
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("visiaengine-e302"),
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
        self.commands = upload_scene!(&mut FaceRef(&mut core));
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);
        self.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => self.on_key(event_loop, &event),
            WindowEvent::Resized(sz) => {
                if let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &mut self.config)
                    && sz.width > 0
                    && sz.height > 0
                {
                    config.width = sz.width;
                    config.height = sz.height;
                    surface.configure(&core.device, config);
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                // A 派中断：拖=取消飞行后接管（CAPI-23 语义同制）
                if matches!(state, ElementState::Pressed) {
                    self.flight = None;
                }
                self.dragging = matches!(state, ElementState::Pressed).then_some((-1.0, -1.0));
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((lx, ly)) = self.dragging.take() {
                    let (x, y) = (position.x, position.y);
                    if lx >= 0.0 {
                        self.rig.orbit_delta((x - lx) * 0.006, (y - ly) * 0.006);
                        self.request_redraw();
                    }
                    self.dragging = Some((x, y));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.flight = None;
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                self.rig.dist = (self.rig.dist * (1.0 - d * 0.0012)).clamp(6.0, 160.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                // 墙钟推进（例内自配——rs 层消费者与 capi 引擎同形不同体）
                if let Some(f) = &self.flight {
                    let t = f.start.elapsed().as_secs_f64() / f.dur_s;
                    self.rig = if t >= 1.0 {
                        f.to
                    } else {
                        CameraRig::fly_sample(&f.from, &f.to, t, Easing::CubicInOut)
                    };
                    if t >= 1.0 {
                        self.flight = None;
                    }
                }
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
                    return;
                };
                let frame = frame_of(
                    &self.rig,
                    self.commands.clone(),
                    config.width,
                    config.height,
                );
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
                self.request_redraw();
            }
            _ => {}
        }
    }
}

impl App {
    fn fly(&mut self, to: CameraRig) {
        self.flight = Some(Flight {
            from: self.rig,
            to,
            start: std::time::Instant::now(),
            dur_s: 1.6,
        });
    }

    fn on_key(&mut self, event_loop: &ActiveEventLoop, key: &KeyEvent) {
        if key.state != ElementState::Pressed {
            return;
        }
        match &key.logical_key {
            Key::Character(s) => match s.as_str() {
                "1" => self.fly(self.presets[0]),
                "2" => self.fly(self.presets[1]),
                "3" => self.fly(self.presets[2]),
                "r" | "R" => self.fly(self.presets[0]),
                _ => {}
            },
            Key::Named(NamedKey::Escape) => event_loop.exit(),
            _ => {}
        }
    }

    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut auto = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--frames" {
            auto = args
                .next()
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|n| n > 0);
        }
    }
    if auto {
        headless_run();
        return Ok(());
    }
    let p = presets();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        window: None,
        core: None,
        surface: None,
        config: None,
        rig: p[0],
        flight: None,
        dragging: None,
        commands: Vec::new(),
        presets: p,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
