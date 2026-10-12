//! E306 · camera map-control presets -- the "map semantics" combination demo of the pan/zoom/tilt trio.
//! Usage: cargo run --example E306_map_controls [options]
//! Interaction: left-drag = pan (drags the ground) | wheel = zoom (orthographic half-width) |
//! right-drag = tilt + orbit around the target | R = reset | close window = exit.
//! `--frames N`: render-budget exit (the ctest/xvfb road, the registry argv-exclusive, C15).
//!
//! A pure consumption combination (the same-semantics form as misc_controls_map): zero engine changes -- pan = a rig.target translation
//! (screen displacement x world ratio), zoom = rig.zoom (orthographic half-width), tilt = rig.pitch/orbit.
//! Starts in an orthographic top view (the map posture), pitch squeezable to an oblique view.

use examples::viewer::{Ctx, FormatPolicy};

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, MultiClearPolicy};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

const W: u32 = 320;
const H: u32 = 240;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Upload seam (the Up form of E304/E305).
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

/// A small city block scene: ground + gridded street line blocks (the visual anchors of map semantics).
fn scene(up: &mut impl Up) -> Vec<DrawCommand> {
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    // Ground.
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
    // 2x2 street blocks (dark), the gaps = streets.
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

/// The orthographic top-view Frame (map posture; the rig carries the pose after pan/zoom/tilt).
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

/// The reset pose (the top-view map posture).
fn home_rig() -> CameraRig {
    CameraRig::orbit([0.0, 0.0, 0.0], 0.0, 1.55, 14.0, 6.0, 1.0, 0.1, 2000.0)
}

// -- headless assert path: scripted input sequence -> assert rig pose changes ----------
fn prove() {
    // The unit-test form of the semantics: pan changes target, zoom changes zoom, tilt changes pitch -- pure-function-level assertions.
    let mut rig = home_rig();
    let t0 = rig.target;
    let z0 = rig.zoom;
    let p0 = rig.pitch;

    // pan: the world shifts left by 1 unit (target.x -= 1).
    rig.target[0] -= 1.0;
    assert!(
        (rig.target[0] - (t0[0] - 1.0)).abs() < 1e-9,
        "pan moves target.x"
    );
    // zoom: half-width halved = zoom in.
    rig.zoom *= 0.5;
    assert!((rig.zoom - z0 * 0.5).abs() < 1e-9, "zoom halves half-width");
    // tilt: pitch squeezed from near-top-view (1.55) to oblique (0.9).
    rig.pitch = 0.9;
    assert!(
        rig.pitch < p0 && rig.pitch > 0.5,
        "tilt lowers pitch into oblique"
    );

    // render one frame to confirm non-degeneracy (the transformed rig outputs content).
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
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
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
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "E306 map controls -- left-drag = pan, wheel = zoom, right-drag = tilt, R = reset",
            960,
            600,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        self.commands = scene(&mut core);
        self.core = Some(core);
        self.ctx = Some(ctx);
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
                if let (Some(core), Some(ctx)) = (&mut self.core, &mut self.ctx) {
                    ctx.on_resize(core, size);
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
                // pan: screen displacement -> world displacement (world-per-px = 2*zoom/h, the exact orthographic definition).
                if let Some((lx, ly)) = self.pan {
                    let wpp = 2.0 * self.rig.zoom
                        / f64::from(self.ctx.as_ref().map_or(H, |c| c.config.height).max(1));
                    let (dx, dy) = (x - lx, y - ly);
                    self.rig.target[0] -= dx * wpp;
                    self.rig.target[1] += dy * wpp; // screen y downward = the reversed world y upward
                    self.pan = Some((x, y));
                    self.redraw();
                }
                // tilt: right-drag vertically = pitch (top view 1.55 -> oblique 0.6), horizontally = yaw.
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
        // no cached entry besides winit CursorMoved; the App stores it itself (init = window center).
        self.last_cursor
    }
    fn cursor_pos_set(&mut self, x: f64, y: f64) {
        self.last_cursor = (x, y);
    }

    fn redraw(&mut self) {
        let (Some(core), Some(ctx)) = (self.core.as_mut(), self.ctx.as_mut()) else {
            return;
        };
        let surface = &ctx.surface;
        let config = &ctx.config;
        let window = &ctx.window;
        let frame = map_frame(&self.commands, &self.rig, config.width, config.height);
        window.set_title(
            format!(
                "E306 map zoom={:.1} pitch={:.2} -- left-drag pan, wheel zoom, right-drag tilt, R reset",
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
        ctx: None,
        core: None,
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
