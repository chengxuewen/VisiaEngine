//! E501 · 材质与光影 —— instanced 楼块群 + 方向光 PCSS 软影（批 4 光影成果面）。
//! 双模式：`--frames N` = headless 单帧 + dark 断言（CI/ctest 契约，pixi smoke-shadow-demo）；
//! 无参数 = 交互窗口（左键拖=轨道、滚轮=远近、关窗退出）——cargo-run_E501 人工检验光影效果面。
//! 用法：`cargo run --example E501_shadow_demo [-- --frames 1]`

use std::sync::Arc;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MaterialId, MeshDesc, MeshId,
    RenderBackend, ShadowBias, ShadowSetup, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

const W: u32 = 256;
const H: u32 = 256;
const SIDE: usize = 16; // 16×16=256 楼块
const IDENTITY: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// 确定性城市：单 draw instanced，高差=素数哈希（headless/window 两路同源）
fn city() -> Vec<Instance> {
    (0..SIDE)
        .flat_map(|gy| {
            (0..SIDE).map(move |gx| {
                let i = gx + gy * SIDE;
                let h = 1.5 + (i * 7919 % 13) as f32 * 0.6; // 确定性高差
                Instance::new(
                    [
                        (gx as f32 - SIDE as f32 / 2.0) * 2.2,
                        (gy as f32 - SIDE as f32 / 2.0) * 2.2,
                        0.0,
                    ],
                    h,
                    [0.82, 0.86, 0.92],
                )
            })
        })
        .collect()
}

fn ground_desc() -> MeshDesc<'static> {
    MeshDesc {
        positions: &[
            [-20.0, -20.0, 0.0],
            [20.0, -20.0, 0.0],
            [20.0, 20.0, 0.0],
            [-20.0, 20.0, 0.0],
        ],
        normals: &[[0.0, 0.0, 1.0]; 4],
        indices: &[0, 1, 2, 0, 2, 3],
        uv: &[],
    }
}

/// 光源相机（REND-31 构造契约：rig 路）：西南高角度太阳，参数与批 4 逐位一致
fn shadow_setup() -> ShadowSetup {
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let eye = [
        f64::from(ld[0] / nn) * 80.0,
        f64::from(ld[1] / nn) * 80.0,
        f64::from(ld[2] / nn) * 80.0,
    ];
    let lrig = CameraRig::look_at(eye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    ShadowSetup {
        proj: lrig
            .ortho_frame(40.0, W as f32, H as f32, 1.0, 160.0)
            .expect("light proj"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5, // 软半影
        bias: ShadowBias {
            constant: -1.2,
            slope: -1.5,
        },
    }
}

fn scene_commands(
    ground: MeshId,
    gmat: MaterialId,
    boxy: MeshId,
    bmat: MaterialId,
    iid: u64,
) -> Vec<DrawCommand> {
    vec![
        DrawCommand::ClearColor {
            rgba: [0.35, 0.55, 0.85, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: ground,
            material: gmat,
            origin: [0.0; 3],
            transform: IDENTITY,
        },
        DrawCommand::DrawInstances {
            mesh: boxy,
            material: bmat,
            instances: iid,
            origin: [0.0; 3],
            transform: IDENTITY,
        },
    ]
}

// ── headless 路（CI 契约原样：单帧 + dark>800 断言 + OK 行）──────────────────
fn headless_run() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter（无 GPU 环境如实报错）");
    let ground = b.create_mesh(&ground_desc()).expect("ground");
    let gmat = b
        .create_material([0.75, 0.75, 0.78, 1.0])
        .expect("ground mat");
    let (pos, nrm, idx) = unit_box_mesh();
    let boxy = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .expect("box");
    let bmat = b.create_material([1.0, 1.0, 1.0, 1.0]).expect("box mat");
    let city = city();
    let iid = b
        .create_instances(&InstanceDesc { data: &city })
        .expect("city table");

    let rig = CameraRig::look_at([0.0, -34.0, 26.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let proj = rig
        .perspective(rig.fov_y as f32, 1.0, 0.5, 300.0)
        .expect("proj");
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.5, 300.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 0.12,
        shadow: Some(shadow_setup()),
        commands: scene_commands(ground, gmat, boxy, bmat, iid),
    };
    let img = b.render_to_pixels(&frame).expect("render");
    let mut dark = 0u32;
    let mut groundish = 0u32;
    for p in img.rgba.as_chunks::<4>().0.iter() {
        let (r, g, bl) = (p[0], p[1], p[2]);
        let lum = r as u32 + g as u32 + bl as u32;
        if r.abs_diff(g) < 18 && r.abs_diff(bl) < 26 && (40..200).contains(&r) {
            groundish += 1;
            if lum < 260 {
                dark += 1;
            }
        }
    }
    assert!(dark > 800, "影斑不足（城市应投影成片）dark={dark}");
    println!(
        "OK shadow demo dark={}/{} buildings={}",
        dark,
        groundish,
        city.len()
    );
}

// ── 交互路（人工检验面：MeshCore 直渲 surface，同场景同阴影）─────────────────
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    rig: CameraRig,
    dragging: Option<(f64, f64)>,
    scene: Option<(MeshId, MaterialId, MeshId, MaterialId, u64)>,
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
                        .with_title("VisiaEngine E501 shadow · 左键拖=轨道 滚轮=远近 关窗退出"),
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
            label: Some("visiaengine-e501"),
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

        // 场景上传（与 headless 路同源参数）
        let ground = core.upload_mesh(&ground_desc()).expect("ground");
        let gmat = core
            .upload_material([0.75, 0.75, 0.78, 1.0])
            .expect("ground mat");
        let (pos, nrm, idx) = unit_box_mesh();
        let boxy = core
            .upload_mesh(&MeshDesc {
                positions: &pos,
                normals: &nrm,
                indices: &idx,
                uv: &[],
            })
            .expect("box");
        let bmat = core.upload_material([1.0, 1.0, 1.0, 1.0]).expect("box mat");
        let city = city();
        let iid = core
            .create_instances(&InstanceDesc { data: &city })
            .expect("city table");
        self.scene = Some((ground, gmat, boxy, bmat, iid));
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &mut self.config)
                    && size.width > 0
                    && size.height > 0
                {
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&core.device, config);
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
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                self.rig.dist = (self.rig.dist * (1.0 - d * 0.0012)).clamp(6.0, 200.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
                    return;
                };
                let Some((ground, gmat, boxy, bmat, iid)) = self.scene else {
                    return;
                };
                let aspect = config.width as f32 / config.height.max(1) as f32;
                let (near, far) = (
                    (self.rig.dist * 0.01) as f32,
                    (self.rig.dist * 3.0 + 120.0) as f32,
                );
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
                    px_world_scale: 0.12,
                    shadow: Some(shadow_setup()),
                    commands: scene_commands(ground, gmat, boxy, bmat, iid),
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
                    other => eprintln!("skipped frame: {other:?}"),
                }
                self.request_redraw();
            }
            _ => {}
        }
    }
}

impl App {
    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
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
        headless_run(); // CI 路：单帧断言即退（N 在 smoke 语境=单帧契约）
        return Ok(());
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        window: None,
        core: None,
        surface: None,
        config: None,
        rig: CameraRig::orbit([0.0, 0.0, 2.0], 0.0, 0.55, 46.0, 40.0, 55.0, 0.1, 1000.0),
        dragging: None,
        scene: None,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
