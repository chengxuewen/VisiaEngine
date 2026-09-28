//! E305 Entity Animation — keyframe trajectory replay (N2.2, REND-42).
//!
//! Dual-mode:
//! - No-arg = resident window: the drone replays the waypoint path on a
//!   loop (t advances by wall-clock delta, wraps at path end); trail
//!   markers are pre-baked (static tables), the drone draw is re-issued
//!   per frame with the sampled origin. Esc closes.
//! - `--frames N` = offscreen assert path: N trail markers + drone at the
//!   final waypoint; endpoint law asserted on this very table; mid-flight
//!   origin asserted distinct from landing (the animation is real).

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use visiaengine_render::anim_origin;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, MultiClearPolicy};

const W: u32 = 320;
const H: u32 = 240;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Waypoint table: take-off -> cruise -> hover -> land (ascending times).
const WAYPOINTS: &[(f64, [f64; 3])] = &[
    (0.0, [-4.0, -4.0, 0.5]),
    (1.0, [0.0, -4.0, 3.0]),
    (2.0, [4.0, 0.0, 3.0]),
    (3.0, [4.0, 4.0, 3.0]),
    (4.0, [0.0, 4.0, 3.0]),
    (5.0, [0.0, 0.0, 0.5]),
];

/// Upload seam (E304's Up shape) — one build for either backend.
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

/// Static GPU tables: ground + pre-baked marker trail + drone.
struct Tables {
    ground: (u64, u64),
    markers: Vec<((u64, u64), [f64; 3])>,
    drone: (u64, u64),
}

fn build(up: &mut impl Up, n_markers: u32) -> Tables {
    let ground = [
        [-6.0f32, -6.0, 0.0],
        [6.0, -6.0, 0.0],
        [6.0, 6.0, 0.0],
        [-6.0, 6.0, 0.0],
    ];
    let gm = up.m(&MeshDesc {
        uv: &[],
        positions: &ground,
        normals: &[[0.0, 0.0, 1.0]; 4],
        indices: &[0, 1, 2, 0, 2, 3],
    });
    let gt = up.t(&MaterialDesc {
        base_color: [0.20, 0.28, 0.22, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });

    let mut markers = Vec::new();
    let t_end = WAYPOINTS.last().expect("non-empty").0;
    for i in 0..n_markers {
        let t = t_end * f64::from(i) / f64::from(n_markers);
        let o = anim_origin(WAYPOINTS, t);
        let marker = [
            [0.0f32, -0.18, 0.0],
            [0.18, 0.0, 0.0],
            [0.0, 0.18, 0.0],
            [-0.18, 0.0, 0.0],
        ];
        let m = up.m(&MeshDesc {
            uv: &[],
            positions: &marker,
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
        });
        let t2 = up.t(&MaterialDesc {
            base_color: [0.2, 0.8, 0.85, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        });
        markers.push(((m, t2), o));
    }

    let drone = [
        [0.0f32, -0.4, 0.0],
        [0.4, 0.0, 0.0],
        [0.0, 0.4, 0.0],
        [-0.4, 0.0, 0.0],
        [0.0, 0.0, 0.35],
    ];
    let didx: Vec<u32> = vec![0, 1, 2, 0, 2, 3, 1, 2, 4, 0, 4, 3, 3, 4, 2, 1, 4, 2];
    let dm = up.m(&MeshDesc {
        uv: &[],
        positions: &drone,
        normals: &[[0.0, 0.0, 1.0]; 5],
        indices: &didx,
    });
    let dt = up.t(&MaterialDesc {
        base_color: [0.9, 0.25, 0.3, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    Tables {
        ground: (gm, gt),
        markers,
        drone: (dm, dt),
    }
}

/// Camera rig: perspective oblique over the pad (both paths share it).
fn cam_rig() -> CameraRig {
    CameraRig::look_at([0.0, -16.0, 11.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0])
}

/// Group the full draw list with the drone at `drone_origin`.
fn group(tbl: &Tables, drone_origin: [f64; 3]) -> Vec<DrawCommand> {
    let mut c = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    c.push(DrawCommand::DrawMesh {
        mesh: tbl.ground.0,
        material: tbl.ground.1,
        origin: [0.0; 3],
        transform: T4,
    });
    for ((m, t), o) in &tbl.markers {
        c.push(DrawCommand::DrawMesh {
            mesh: *m,
            material: *t,
            origin: *o,
            transform: T4,
        });
    }
    c.push(DrawCommand::DrawMesh {
        mesh: tbl.drone.0,
        material: tbl.drone.1,
        origin: drone_origin,
        transform: T4,
    });
    c
}

fn frame(cmds: Vec<DrawCommand>, w: u32, h: u32) -> Frame {
    let rig = cam_rig();
    let aspect = w as f32 / h.max(1) as f32;
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, aspect, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, aspect, 0.1, 1000.0)
            .expect("proj"),
        px_world_scale: 0.12,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds,
    }
}

// ── offscreen assert path ────────────────────────────────────────────────────
fn prove(frames: u32) {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tbl = build(&mut b, frames.clamp(4, 96));

    // Endpoint law asserted on this very table.
    let t_end = WAYPOINTS.last().expect("non-empty").0;
    assert_eq!(
        anim_origin(WAYPOINTS, 0.0),
        WAYPOINTS[0].1,
        "t=0 -> first waypoint verbatim"
    );
    let end = anim_origin(WAYPOINTS, t_end);
    assert_eq!(
        end,
        WAYPOINTS.last().expect("non-empty").1,
        "path-end -> last waypoint verbatim"
    );
    let mid = anim_origin(WAYPOINTS, 2.0);
    assert_ne!(mid, end, "mid-flight origin differs from landing");

    let img = b.render_to_pixels(&frame(group(&tbl, end), W, H)).unwrap();
    let (cyan, red): (u32, u32) =
        img.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .fold((0u32, 0u32), |(c, r), p| {
                let hit_c = p[2] > 140 && p[1] > 110 && p[0] < 90;
                let hit_r = p[0] > 150 && p[1] < 90;
                (c + u32::from(hit_c), r + u32::from(hit_r))
            });
    println!(
        "E305 probe: cyan={cyan} red={red} markers={}",
        tbl.markers.len()
    );
    assert!(cyan > 20, "trail markers visible (cyan={cyan})");
    assert!(red > 10, "drone visible at landing (red={red})");
    println!(
        "E305 OK: {} markers + drone at landing {end:?}; mid-flight {mid:?} differs",
        tbl.markers.len()
    );
}

// ── resident window (E304 winit shape; drone replays on a wall-clock loop) ──
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    tbl: Option<Tables>,
    t: f64,
    last: Option<std::time::Instant>,
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
                        .with_inner_size(winit::dpi::PhysicalSize::new(640u32, 480u32))
                        .with_title("E305 drone replay — Esc exit"),
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
            label: Some("visiaengine-e305"),
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
        self.tbl = Some(build(&mut core, 32));
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
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        // Animation clock: wall-delta advance, wrap at path end.
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last.unwrap_or(now)).as_secs_f64();
        self.last = Some(now);
        let t_end = WAYPOINTS.last().expect("non-empty").0;
        self.t = (self.t + dt) % t_end;

        let (Some(core), Some(surface), Some(config), Some(window)) = (
            self.core.as_mut(),
            self.surface.as_ref(),
            self.config.as_ref(),
            self.window.as_ref(),
        ) else {
            return;
        };
        let tbl = self.tbl.as_ref().expect("built");
        let drone_o = anim_origin(WAYPOINTS, self.t);
        window.set_title(format!("E305 drone replay t={:.2} — Esc exit", self.t).as_str());
        let cmds = group(tbl, drone_o);
        let frame = frame(cmds, config.width, config.height);
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
    match frames {
        Some(n) => prove(n),
        None => {
            let el = EventLoop::new()?;
            el.set_control_flow(ControlFlow::Poll);
            let mut app = App {
                window: None,
                core: None,
                surface: None,
                config: None,
                tbl: None,
                t: 0.0,
                last: None,
                ready: false,
            };
            el.run_app(&mut app)?;
        }
    }
    Ok(())
}
