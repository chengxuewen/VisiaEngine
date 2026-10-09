//! E509 SSAO — depth-only ambient occlusion post effect demo (WGPU-36).
//!
//! Dual-mode (E508 shape): MeshCore direct + HeadlessBackend assert.
//! - No-arg = resident window: `1` SSAO toggle, `2` intensity +0.5 (cap 4.0),
//!   `3` intensity −0.5 (floor 0.5 while on), `0` all off; title echoes state,
//!   Esc closes. Creases/tower bases darken more than open ground.
//! - `--frames N` = offscreen assert path: off vs ssao-on(1.5) pixel diff
//!   exceeds probe-pinned K; off state deterministic (canary).

use examples::viewer::{Ctx, FormatPolicy};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MeshDesc, PostEffect, RenderBackend,
    Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;
use visiaengine_render_wgpu::mesh_core::MeshCore;

#[path = "gallery.rs"]
mod gallery;

const W: u32 = 320;
const H: u32 = 240;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
const DEFAULT_INTENSITY: f32 = 1.5;

/// Upload seam (E508's Up trait shape): one scene builder serving both
/// HeadlessBackend (assert path) and MeshCore (window path).
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

/// E508 city grid: ground + tower rows with a few taller boxes — the creases
/// between towers and the tower/ground junction are the AO showcase.
fn scene(up: &mut impl Up) -> Vec<DrawCommand> {
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
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
        base_color: [0.22, 0.30, 0.24, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
        roughness: 1.0,
        metallic: 0.0,
    });
    commands.push(DrawCommand::DrawMesh {
        mesh: gm,
        material: gt,
        origin: [0.0; 3],
        transform: T4,
    });
    // City grid: 3×3 towers, heights varied — a few taller boxes in the middle
    // row deepen the creases the AO kernel lives on.
    for (x, y, h, color) in [
        (-3.0f32, -3.0f32, 1.6f32, [0.42, 0.48, 0.56, 1.0]),
        (0.0, -3.0, 2.4, [0.52, 0.58, 0.66, 1.0]),
        (3.0, -3.0, 1.2, [0.42, 0.48, 0.56, 1.0]),
        (-3.0, 0.0, 2.0, [0.52, 0.58, 0.66, 1.0]),
        (0.0, 0.0, 4.5, [0.62, 0.68, 0.76, 1.0]), // tallest — deep creases
        (3.0, 0.0, 3.5, [0.62, 0.68, 0.76, 1.0]), // tall
        (-3.0, 3.0, 1.4, [0.42, 0.48, 0.56, 1.0]),
        (0.0, 3.0, 2.8, [0.52, 0.58, 0.66, 1.0]),
        (3.0, 3.0, 1.8, [0.42, 0.48, 0.56, 1.0]),
    ] {
        let (x0, y0, x1, y1) = (x - 0.8, y - 0.8, x + 0.8, y + 0.8);
        let verts = [
            [x0, y0, 0.0],
            [x1, y0, 0.0],
            [x1, y1, 0.0],
            [x0, y1, 0.0],
            [x0, y0, h],
            [x1, y0, h],
            [x1, y1, h],
            [x0, y1, h],
        ];
        let idx: Vec<u32> = vec![
            4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7, 6, 3, 0, 4, 3, 4,
            7,
        ];
        let bm = up.m(&MeshDesc {
            uv: &[],
            positions: &verts,
            normals: &[[0.0, 0.0, 1.0]; 8],
            indices: &idx,
        });
        let bt = up.t(&MaterialDesc {
            base_color: color,
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 1.0,
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

/// E508 rig-plumbed frame(): the camera lives in the App state (orbit/zoom)
/// and every frame rebuild takes the CURRENT rig.
fn frame(cmds: &[DrawCommand], w: u32, h: u32, post: Vec<PostEffect>, rig: &CameraRig) -> Frame {
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, w as f32 / h as f32, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, w as f32 / h as f32, 0.1, 1000.0)
            .expect("persp"),
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post,
        commands: cmds.to_vec(),
    }
}

fn diff_px(
    a: &visiaengine_render_wgpu::OffscreenFrame,
    b: &visiaengine_render_wgpu::OffscreenFrame,
) -> u32 {
    a.rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.rgba.as_chunks::<4>().0)
        .filter(|(x, y)| x[..3] != y[..3])
        .count() as u32
}

// ── headless assert path ─────────────────────────────────────────────────────
fn prove(frames: u32) {
    let _ = frames; // single pass per state; N kept for argv symmetry
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);
    let rig = CameraRig::look_at([0.0, -13.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);

    let off_a = b
        .render_to_pixels(&frame(&cmds, W, H, Vec::new(), &rig))
        .unwrap();
    let off_b = b
        .render_to_pixels(&frame(&cmds, W, H, Vec::new(), &rig))
        .unwrap();
    assert_eq!(off_a.rgba, off_b.rgba, "off state must be deterministic");

    let on = b
        .render_to_pixels(&frame(
            &cmds,
            W,
            H,
            vec![PostEffect::ssao(4.0, DEFAULT_INTENSITY).expect("ssao")],
            &rig,
        ))
        .unwrap();

    let d = diff_px(&off_a, &on);
    // Probe (lavapipe, 2026-09-29, 320×240, 9-tower city, radius 4,
    // intensity 1.5): diff=18678 px — tower/ground crease bands + tower-face
    // silhouette bands across the 9-tower grid (62% of frame). Floor 5000
    // (−73%, PIT-8).
    println!("E509 probe: off-vs-ssao diff={d}");
    assert!(d > 5000, "ssao must differ from off: {d}");
    println!("E509 OK: off deterministic; ssao pixel-verified");
    gallery::save_frame(&on, "E509_ssao");
}

// ── resident window path (E508 winit shape) ──────────────────────────────────
struct App {
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
    commands: Vec<DrawCommand>,
    rig: CameraRig,
    drag: bool,
    last: (f64, f64),
    ssao_on: bool,
    intensity: f32,
    ready: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ready {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "E509 SSAO · 1=toggle 2=int+ 3=int- 0=off · Esc exit",
            800,
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
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != winit::event::ElementState::Pressed {
                    return;
                }
                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => {
                        event_loop.exit();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit1) => {
                        self.ssao_on = !self.ssao_on;
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2) => {
                        self.intensity = (self.intensity + 0.5).min(4.0);
                        self.ssao_on = true;
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit3) => {
                        self.intensity = (self.intensity - 0.5).max(0.5);
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit0) => {
                        self.ssao_on = false;
                        self.intensity = DEFAULT_INTENSITY;
                        self.redraw();
                    }
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    self.drag = state == winit::event::ElementState::Pressed;
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if self.drag {
                    let dx = position.x - self.last.0;
                    let dy = position.y - self.last.1;
                    self.last = (position.x, position.y);
                    let (eye, target) = (self.rig.eye(), [0.0, 0.0, 0.0]);
                    let off = [eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]];
                    let r = (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt();
                    let theta = off[1].atan2(off[0]) + dx * 0.005;
                    let phi = (off[2] / r).clamp(-1.4, 1.4) - dy * 0.005;
                    let (eye_x, eye_y, eye_z) = (
                        target[0] + r * phi.cos() * theta.cos(),
                        target[1] + r * phi.cos() * theta.sin(),
                        target[2] + r * phi.sin(),
                    );
                    self.rig =
                        CameraRig::look_at([eye_x, eye_y, eye_z.max(0.5)], target, [0.0, 0.0, 1.0]);
                    self.redraw();
                } else {
                    self.last = (position.x, position.y);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => f64::from(y) * 1.0,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y * 0.05,
                };
                let (eye, target) = (self.rig.eye(), [0.0, 0.0, 0.0]);
                let off = [eye[0] - target[0], eye[1] - target[1], eye[2] - target[2]];
                let r = ((off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt()
                    * (1.0 - d * 0.1))
                    .clamp(4.0, 40.0);
                let len = off[0] * off[0] + off[1] * off[1] + off[2] * off[2];
                let k = r / len.sqrt();
                self.rig = CameraRig::look_at(
                    [
                        target[0] + off[0] * k,
                        target[1] + off[1] * k,
                        target[2] + off[2] * k,
                    ],
                    target,
                    [0.0, 0.0, 1.0],
                );
                self.redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.redraw();
    }
}

impl App {
    fn redraw(&mut self) {
        let (Some(core), Some(ctx)) = (self.core.as_mut(), self.ctx.as_mut()) else {
            return;
        };
        let size = ctx.window.inner_size();
        let (w, h) = (size.width.max(1), size.height.max(1));
        let post = if self.ssao_on {
            vec![PostEffect::Ssao {
                radius: 4.0,
                intensity: self.intensity,
            }]
        } else {
            Vec::new()
        };
        let frame = frame(&self.commands, w, h, post, &self.rig);
        ctx.present(core, &frame);
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
                ctx: None,
                core: None,
                commands: Vec::new(),
                rig: CameraRig::look_at([0.0, -13.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
                drag: false,
                last: (0.0, 0.0),
                ssao_on: true,
                intensity: DEFAULT_INTENSITY,
                ready: false,
            };
            el.run_app(&mut app)?;
        }
    }
    Ok(())
}
