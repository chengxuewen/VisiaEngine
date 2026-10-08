//! E511 Gate — rigid node transform live demo (CAPI-42/43).
//!
//! Two gate leaves share one geometry and differ only by `DrawMesh.transform`
//! (column-major f64 pose) plus `DrawMesh.origin` (the pivot) — the same
//! pipeline segment CAPI-42 writes into for glTF node animation.
//!
//! Dual mode (C15): zero-arg = resident human window (leaves swing on a clock,
//! drag = orbit, wheel = zoom, Esc exits); `--frames N` = headless assertion
//! lane run by ctest (closed-vs-open pixel delta + composed-offset leaf).
//!
//! Claim: rigid node transforms only (CAPI-42/43) — skeletal/morph is V2.4.

use std::sync::Arc;
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, MultiClearPolicy};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[path = "gallery.rs"]
mod gallery;
use gallery::save_frame;

use visiaengine_render::contract::{Camera, DrawCommand, Frame, MaterialDesc, MeshDesc, Viewport};
use visiaengine_render::{CameraRig, MaterialId, MeshId, ViewportRect};

const W: u32 = 480;
const H: u32 = 320;

/// One leaf: 2 units wide, 4 tall, pivoting at its own local origin (x=0),
/// extending to +x. Both lanes build their scene from this.
fn leaf_geometry() -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    (
        vec![
            [-1.0, -2.0, 0.0],
            [1.0, -2.0, 0.0],
            [1.0, 2.0, 0.0],
            [-1.0, 2.0, 0.0],
        ],
        vec![[0.0f32, 0.0, 1.0]; 4],
    )
}

/// GPU products for the scene, uploaded once per backend (both lanes share it).
fn gate_products(core: &mut MeshCore) -> (MeshId, MeshId, MaterialId) {
    let (leaf, normals) = leaf_geometry();
    let desc = || MeshDesc {
        uv: &[],
        positions: &leaf,
        normals: &normals,
        indices: &[0, 1, 2, 0, 2, 3],
    };
    let m0 = core.upload_mesh(&desc()).expect("leaf0");
    let m1 = core.upload_mesh(&desc()).expect("leaf1");
    let mat = core
        .upload_material_desc(&MaterialDesc {
            base_color: [0.8, 0.55, 0.2, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 0.7,
            metallic: 0.0,
        })
        .expect("mat");
    (m0, m1, mat)
}

/// Leaf 2 sits 3 units to the right *inside its own transform* (composed),
/// which is what DrawMesh.origin alone would not express.
fn shifted_x(x: f64) -> [[f64; 4]; 4] {
    let mut m = identity();
    m[0][3] = x; // column-major translation column
    m
}

/// The two leaves at open angle `theta` (radians about +Z-up Y axis).
fn gate_cmds(m0: MeshId, m1: MeshId, mat: MaterialId, theta: f64) -> Vec<DrawCommand> {
    let rot = rot_y_f64(theta);
    vec![
        DrawCommand::ClearColor {
            rgba: [0.06, 0.08, 0.11, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: m0,
            material: mat,
            origin: [0.0, 0.0, 0.0],
            transform: rot,
        },
        DrawCommand::DrawMesh {
            mesh: m1,
            material: mat,
            origin: [3.0, 0.0, 0.0],
            transform: mul_mat(shifted_x(3.0), rot),
        },
    ]
}

fn headless_frame(commands: Vec<DrawCommand>, w: u32, h: u32) -> Frame {
    let rig = CameraRig::look_at([0.0, 0.0, 12.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::perspective(1.0, w as f32 / h as f32, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(1.0, w as f32 / h as f32, 0.1, 100.0)
            .expect("proj"),
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    }
}

fn prove(_frames: u32) {
    // Assertion lane: rotating the pose must move pixels; the composed-offset
    // second leaf must follow the same pose (canary against dropping transform).
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (m0, m1, mat) = gate_products(b.core_mut());

    let closed = b
        .render_to_pixels(&headless_frame(gate_cmds(m0, m1, mat, 0.0), W, H))
        .expect("closed");
    let opened = b
        .render_to_pixels(&headless_frame(
            gate_cmds(m0, m1, mat, std::f64::consts::FRAC_PI_4),
            W,
            H,
        ))
        .expect("opened");
    let diff = closed
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(opened.rgba.as_chunks::<4>().0)
        .filter(|(a, b2)| a[..3] != b2[..3])
        .count();
    println!("E511 probe: gate-open diff={diff}");
    assert!(diff > 500, "rotated leaves must move pixels: {diff}");
    save_frame(&opened, "E511_gate");
    println!("E511 OK: transform face drives rigid rotation; static offset leaf composed");
}

/// Window lane state (C15 dual-mode: zero-arg = resident human window).
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    ids: Option<(MeshId, MeshId, MaterialId)>,
    // Gate clock: theta sweeps 0..60 degrees on a cosine so both ends dwell.
    t: f64,
    last: Option<std::time::Instant>,
    theta: f64,
    rig: CameraRig,
    drag: bool,
    last_pos: (f64, f64),
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
                    winit::window::WindowAttributes::default()
                        .with_inner_size(winit::dpi::PhysicalSize::new(760u32, 520u32))
                        .with_title("E511 gate swing · drag=orbit wheel=zoom · Esc exit"),
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
            label: Some("visiaengine-e511"),
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
        self.ids = Some(gate_products(&mut core));
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);
        self.ready = true;
        self.last = Some(std::time::Instant::now());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == winit::event::ElementState::Pressed
                    && event.physical_key
                        == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =>
            {
                event_loop.exit();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    self.drag = state == winit::event::ElementState::Pressed;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.drag {
                    let dx = position.x - self.last_pos.0;
                    let dy = position.y - self.last_pos.1;
                    self.orbit(dx * 0.005, -dy * 0.005);
                }
                self.last_pos = (position.x, position.y);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => f64::from(y),
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y * 0.05,
                };
                self.zoom(d);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last.unwrap_or(now)).as_secs_f64();
        self.last = Some(now);
        self.t += dt;
        // 0 .. 60 degrees, dwelling at both ends (cosine easing, no snap).
        self.theta = (1.0 - (self.t * 1.1).cos()) * 0.5 * std::f64::consts::FRAC_PI_3;

        let (Some(core), Some(surface), Some(config), Some(window), Some((m0, m1, mat))) = (
            self.core.as_mut(),
            self.surface.as_ref(),
            self.config.as_ref(),
            self.window.as_ref(),
            self.ids,
        ) else {
            return;
        };
        window.set_title(
            format!(
                "E511 gate swing · theta={:.1}deg · drag=orbit wheel=zoom · Esc exit",
                self.theta.to_degrees()
            )
            .as_str(),
        );
        let rig = &self.rig;
        let frame = Frame {
            viewport: Viewport::new(config.width, config.height, 1.0),
            camera: Camera::perspective(
                rig.fov_y as f32,
                config.width as f32 / config.height.max(1) as f32,
                0.1,
                200.0,
            ),
            view_rot: rig.view_rotation(),
            eye: rig.eye(),
            proj: rig
                .perspective(
                    rig.fov_y as f32,
                    config.width as f32 / config.height.max(1) as f32,
                    0.1,
                    200.0,
                )
                .expect("proj"),
            px_world_scale: 1.0,
            shadow: None,
            clip: None,
            edl: None,
            post: Vec::new(),
            commands: gate_cmds(m0, m1, mat, self.theta),
        };
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                core.render_view_rects(
                    &[(frame, ViewportRect::new(0, 0, config.width, config.height))],
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

impl App {
    /// Orbit around the gate centre by azimuth/elevation radians (E510 shape).
    fn orbit(&mut self, dtheta: f64, dphi: f64) {
        let target = [1.5, 0.0, 0.0];
        let eye = self.rig.eye();
        let off = [eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]];
        let r = (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt();
        let theta = off[1].atan2(off[0]) + dtheta;
        let phi = (off[2] / r).clamp(-1.4, 1.4) + dphi;
        self.rig = CameraRig::look_at(
            [
                target[0] + r * phi.cos() * theta.cos(),
                target[1] + r * phi.cos() * theta.sin(),
                target[2] + r * phi.sin().max(0.3),
            ],
            target,
            [0.0, 0.0, 1.0],
        );
    }

    fn zoom(&mut self, d: f64) {
        let target = [1.5, 0.0, 0.0];
        let eye = self.rig.eye();
        let off = [eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]];
        let len = (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt();
        let k = (len * (1.0 - d * 0.1)).clamp(8.0, 90.0) / len;
        self.rig = CameraRig::look_at(
            [
                target[0] + off[0] * k,
                target[1] + off[1] * k,
                target[2] + off[2] * k,
            ],
            target,
            [0.0, 0.0, 1.0],
        );
    }
}

fn window_form() -> Result<(), Box<dyn std::error::Error>> {
    let el = EventLoop::new()?;
    el.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        window: None,
        core: None,
        surface: None,
        config: None,
        ids: None,
        t: 0.0,
        last: None,
        theta: 0.0,
        rig: CameraRig::look_at([1.5, -24.0, 14.0], [1.5, 0.0, 0.0], [0.0, 0.0, 1.0]),
        drag: false,
        last_pos: (0.0, 0.0),
        ready: false,
    };
    el.run_app(&mut app)?;
    Ok(())
}

fn rot_y_f64(a: f64) -> [[f64; 4]; 4] {
    let (s, c) = a.sin_cos();
    [
        [c, 0.0, -s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn mul_mat(a: [[f64; 4]; 4], b: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut o = [[0.0f64; 4]; 4];
    for (r, or) in o.iter_mut().enumerate() {
        for c in 0..4 {
            or[c] = (0..4).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    o
}

fn identity() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
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
    match frames {
        Some(n) => prove(n),
        None => window_form()?,
    }
    Ok(())
}
