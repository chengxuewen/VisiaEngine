//! E303 · 分屏驾驶舱 —— 主透视 3D + 右栏正射顶视小地图（⑤b WGPU-26 活例）。
//! 双模（E901/E302 同制）：无参=常驻窗（1/2/3=飞行 拖/滚轮=主视接管 R=回家 Esc=退）；
//! `--frames N`=离屏断言快退（分区族计数+缝色+顶视覆盖，PIT-8 探针定阈）。
//! 小地图 rig=target 跟随主相机（同批命令双投两相机=WGPU-26 安全形）。

use std::sync::Arc;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Easing, Frame, Instance, InstanceDesc, MaterialDesc, MeshDesc,
    RenderBackend, ShadowSetup, Viewport, ViewportRect,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, MultiClearPolicy, unit_box_mesh};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const SIDE: usize = 12;
const ID64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

trait Up {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64;
    fn t(&mut self, d: &MaterialDesc) -> u64;
    fn i(&mut self, d: &InstanceDesc<'_>) -> u64;
}

impl Up for HeadlessBackend {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.create_mesh(d).expect("m")
    }
    fn t(&mut self, d: &MaterialDesc) -> u64 {
        self.create_material_desc(d).expect("t")
    }
    fn i(&mut self, d: &InstanceDesc<'_>) -> u64 {
        self.create_instances(d).expect("i")
    }
}

impl Up for MeshCore {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.upload_mesh(d).expect("m")
    }
    fn t(&mut self, d: &MaterialDesc) -> u64 {
        self.upload_material_desc(d).expect("t")
    }
    fn i(&mut self, d: &InstanceDesc<'_>) -> u64 {
        self.create_instances(d).expect("i")
    }
}

fn scene(up: &mut impl Up) -> Vec<DrawCommand> {
    let mut cmds = vec![DrawCommand::ClearColor {
        rgba: [0.42, 0.58, 0.80, 1.0],
    }];
    let ground = up.m(&MeshDesc {
        positions: &[
            [-30.0, -30.0, -0.05],
            [30.0, -30.0, -0.05],
            [30.0, 30.0, -0.05],
            [-30.0, 30.0, -0.05],
        ],
        normals: &[[0.0, 0.0, 1.0]; 4],
        indices: &[0, 1, 2, 0, 2, 3],
        uv: &[],
    });
    let gm = up.t(&MaterialDesc {
        base_color: [0.62, 0.63, 0.66, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    cmds.push(DrawCommand::DrawMesh {
        mesh: ground,
        material: gm,
        origin: [0.0; 3],
        transform: ID64,
    });
    let (pos, nrm, idx) = unit_box_mesh();
    let boxy = up.m(&MeshDesc {
        positions: &pos,
        normals: &nrm,
        indices: &idx,
        uv: &[],
    });
    let bm = up.t(&MaterialDesc {
        base_color: [1.0, 1.0, 1.0, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    let step = 2.2f32;
    let span = step * SIDE as f32 / 2.0;
    let insts: Vec<Instance> = (0..SIDE)
        .flat_map(|gy| {
            (0..SIDE).map(move |gx| {
                let i = gx + gy * SIDE;
                let v = 0.78 + (i * 104729 % 7) as f32 * 0.02;
                Instance::new(
                    [
                        -span + gx as f32 * step + step * 0.4,
                        -span + gy as f32 * step + step * 0.4,
                        0.0,
                    ],
                    1.6 + (i * 7919 % 11) as f32 * 1.05,
                    [v, v * 1.02, v * 1.08],
                )
            })
        })
        .collect();
    let iid = up.i(&InstanceDesc { data: &insts });
    cmds.push(DrawCommand::DrawInstances {
        mesh: boxy,
        material: bm,
        instances: iid,
        origin: [0.0; 3],
        transform: ID64,
    });
    cmds
}

fn shadow_setup(w: u32, h: u32) -> ShadowSetup {
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let leye = [
        f64::from(ld[0] / nn) * 70.0,
        f64::from(ld[1] / nn) * 70.0,
        f64::from(ld[2] / nn) * 70.0,
    ];
    let lrig = CameraRig::look_at(leye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    ShadowSetup {
        proj: lrig
            .ortho_frame(34.0, w.max(1) as f32, h.max(1) as f32, 1.0, 140.0)
            .expect("lp"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5,
        bias: ShadowSetup::DEFAULT_BIAS,
    }
}

fn rects(sw: u32, sh: u32) -> (ViewportRect, ViewportRect) {
    let side = (sh / 2).max(16); // 右上角正方形小窗（长条=构图错）
    (
        ViewportRect::new(0, 0, sw, sh),
        ViewportRect::new(sw - side, 0, side, side),
    )
}

fn main_frames(
    cmds: &[DrawCommand],
    main: &CameraRig,
    sw: u32,
    sh: u32,
) -> Vec<(Frame, ViewportRect)> {
    let (mrect, map) = rects(sw, sh);
    let [mx, my, _] = main.target;
    let map_rig = CameraRig::look_at([mx, my, 45.0], [mx, my, 0.0], [0.0, 1.0, 0.0]);
    let aspect = mrect.width as f32 / mrect.height.max(1) as f32;
    let f_main = Frame {
        viewport: Viewport::new(mrect.width, mrect.height, 1.0),
        camera: Camera::perspective(main.fov_y as f32, aspect, 0.3, 300.0),
        view_rot: main.view_rotation(),
        eye: main.eye(),
        proj: main
            .perspective(main.fov_y as f32, aspect, 0.3, 300.0)
            .expect("p"),
        px_world_scale: 0.12,
        shadow: Some(shadow_setup(mrect.width, mrect.height)),
        clip: None,
        commands: cmds.to_vec(),
    };
    let f_map = Frame {
        viewport: Viewport::new(map.width, map.height, 1.0),
        camera: Camera::ortho(
            26.0,
            26.0 * map.height as f32 / map.width as f32,
            0.5,
            200.0,
        ),
        view_rot: map_rig.view_rotation(),
        eye: map_rig.eye(),
        proj: map_rig
            .ortho_frame(26.0, map.width as f32, map.height as f32, 0.5, 200.0)
            .expect("mp"),
        px_world_scale: 52.0 / map.width as f32,
        shadow: f_main.shadow,
        clip: None,
        commands: cmds.to_vec(),
    };
    vec![(f_main, mrect), (f_map, map)]
}

fn presets() -> [CameraRig; 3] {
    [
        CameraRig::orbit([0.0, 0.0, 4.0], 0.7, 0.9, 80.0, 40.0, 0.802_851, 0.3, 400.0),
        CameraRig::orbit(
            [0.0, -6.0, 1.2],
            -1.8,
            0.06,
            10.0,
            12.0,
            std::f64::consts::FRAC_PI_3,
            0.3,
            200.0,
        ),
        CameraRig::orbit([8.0, 8.0, 5.0], 2.2, 0.3, 12.0, 12.0, 0.959_931, 0.3, 200.0),
    ]
}

// ── headless 路 ───────────────────────────────────────────────────────────────
fn headless_run() {
    const W: u32 = 96;
    const H: u32 = 64;
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let rig = presets()[0];
    let passes = main_frames(&cmds, &rig, W, H);
    let img = b
        .render_to_pixels_rects(&passes, MultiClearPolicy::FirstClearRestLoad)
        .expect("render split");
    let (mr, map) = rects(W, H);
    let fam = |r: &ViewportRect, f: fn([u8; 4]) -> bool| -> u32 {
        (r.y..r.y + r.height)
            .flat_map(|y| (r.x..r.x + r.width).map(move |x| (x, y)))
            .filter(|(x, y)| {
                let i = ((y * W + x) * 4) as usize;
                f([
                    img.rgba[i],
                    img.rgba[i + 1],
                    img.rgba[i + 2],
                    img.rgba[i + 3],
                ])
            })
            .count() as u32
    };
    let area_m = mr.width * mr.height;
    let is_sky = |p: [u8; 4]| p[2] as i32 - p[0] as i32 > 40 && p[2] > 180;
    let is_roof = |p: [u8; 4]| p[0] > 148 && p[2] > 148; // 楼顶亮族（v≥0.78×shade 高段）
    let non_sky = |r: &ViewportRect| -> u32 { r.width * r.height - fam(r, is_sky) };
    let (ns_m, ns_g) = (non_sky(&mr), non_sky(&map));
    let roofs = fam(&map, is_roof);
    assert!(ns_m > area_m * 35 / 100, "主视地面主导不足 {ns_m}/{area_m}"); // 斜视天顶带合理（实测 51%）
    assert!(
        ns_g > map.width * map.height * 9 / 10,
        "小地图应全幅地面（顶视构图）got {ns_g}"
    );
    assert!(roofs > 40, "小地图楼顶族缺席 {roofs}（城投达？）");
    println!("OK split_screen nonsky={ns_m}+{ns_g} roofs={roofs}（双投分区现行）");
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
                        .with_title(
                            "E303 分屏驾驶舱 · 主透视+右栏顶视 · 1/2/3 飞行 拖滚=接管 R=回家",
                        ),
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
            label: Some("visiaengine-e303"),
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
                    self.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
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
                let passes = main_frames(&self.commands, &self.rig, config.width, config.height);
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(tex)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                        let view = tex
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        core.render_view_rects(
                            &passes,
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
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        window: None,
        core: None,
        surface: None,
        config: None,
        rig: presets()[0],
        flight: None,
        dragging: None,
        commands: Vec::new(),
        presets: presets(),
        ready: false,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
