//! E508 PBR materials — N6 consumer: Cook-Torrance GGX (WGPU-35/REND-45) live
//! on a roughness × metallic grid of boxes over a city ground, one directional
//! light (dummy params 0.5,0.7,0.4 — engine default light configuration).
//!
//! Dual-mode (E507 shape): MeshCore direct + HeadlessBackend assert.
//! - No-arg = resident window: orbit with mouse drag, wheel zooms, title
//!   echoes mode; Esc exits.
//! - `--frames N` = offscreen assert path: (1) metallic column vs dielectric
//!   column pixel families differ; (2) roughness ladder — highlight pixel
//!   counts shrink monotonically across the 4 roughness steps (probe-pinned).

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

/// Upload seam (E506/E507 Up trait shape): one scene builder serving both
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

/// Low-poly UV sphere (radius 1, center z=0, +z up): the material-ball
/// presentation — flat boxes never catch a directional-light mirror lobe
/// (ndh is constant per face: all-or-nothing highlight), curved surfaces
/// sweep ndh across the lobe so roughness has a visible gradient.
fn sphere_mesh(rings: u32, sectors: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut pos = Vec::new();
    let mut idx = Vec::new();
    for i in 0..=rings {
        let phi = std::f32::consts::PI * i as f32 / rings as f32;
        for j in 0..=sectors {
            let th = std::f32::consts::TAU * j as f32 / sectors as f32;
            pos.push([phi.sin() * th.cos(), phi.sin() * th.sin(), phi.cos()]);
        }
    }
    for i in 0..rings {
        for j in 0..sectors {
            let a = i * (sectors + 1) + j;
            let b = a + sectors + 1;
            idx.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    (pos, idx)
}

/// Scene: ground + 4×2 grid of boxes (4 roughness columns × 2 metallic rows).
/// Returns the draw commands; `slots` receives (mesh_cmd_index, material) per
/// box so the assert path can render single-box variants.
fn scene(up: &mut impl Up) -> (Vec<DrawCommand>, Vec<(u64, u64)>) {
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    let ground = [
        [-7.0f32, -4.0, 0.0],
        [7.0, -4.0, 0.0],
        [7.0, 4.0, 0.0],
        [-7.0, 4.0, 0.0],
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
    // Grid: columns = roughness [0.05, 0.3, 0.6, 0.95]; rows = metallic [0.0, 1.0].
    let rough = [0.05f32, 0.3, 0.6, 0.95];
    let metal = [0.0f32, 1.0];
    let (pos, idx) = sphere_mesh(12, 20);
    let mesh = up.m(&MeshDesc {
        uv: &[],
        positions: &pos,
        normals: &pos, // UV sphere: normal == normalized position
        indices: &idx,
    });
    let mut slots = Vec::new();
    for (row, m) in metal.iter().enumerate() {
        for (col, r) in rough.iter().enumerate() {
            let mat = up.t(&MaterialDesc {
                base_color: [0.75, 0.73, 0.70, 1.0],
                texture: None,
                repeat: [1.0, 1.0],
                specular: 0.0,
                roughness: *r,
                metallic: *m,
            });
            let x = -4.5 + col as f32 * 3.0;
            let y = -1.75 + row as f32 * 3.5;
            commands.push(DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin: [x as f64, y as f64, 1.0],
                transform: T4,
            });
            slots.push((commands.len() as u64 - 1, mat));
        }
    }
    (commands, slots)
}

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

fn count_family(
    img: &visiaengine_render_wgpu::OffscreenFrame,
    fam: impl Fn([u8; 4]) -> bool,
) -> u32 {
    img.rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| fam([p[0], p[1], p[2], p[3]]))
        .count() as u32
}

// ── headless assert path ─────────────────────────────────────────────────────
fn prove(frames: u32) {
    let _ = frames; // single pass per state; N kept for argv symmetry
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (cmds, _slots) = scene(&mut b);

    let rig = CameraRig::look_at([0.0, -13.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let full = b
        .render_to_pixels(&frame(&cmds, W, H, Vec::new(), &rig))
        .unwrap();
    // Determinism canary.
    let full2 = b
        .render_to_pixels(&frame(&cmds, W, H, Vec::new(), &rig))
        .unwrap();
    assert_eq!(full.rgba, full2.rgba, "grid render must be deterministic");

    // Metallic vs dielectric: render single-box scenes (metallic 1.0 vs 0.0,
    // same roughness 0.3) — the two pixel families must differ materially.
    let single = |b: &mut HeadlessBackend, mat: u64, mesh: u64, origin: [f64; 3]| {
        let cmds = vec![
            DrawCommand::ClearColor {
                rgba: [0.05, 0.07, 0.10, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin,
                transform: T4,
            },
        ];
        b.render_to_pixels(&frame(&cmds, W, H, Vec::new(), &rig))
            .unwrap()
    };
    let (pos, idx) = sphere_mesh(12, 20);
    let mesh_id = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &pos,
            normals: &pos,
            indices: &idx,
        })
        .expect("mesh");
    let mk_mat = |b: &mut HeadlessBackend, r: f32, m: f32| {
        b.create_material_desc(&MaterialDesc {
            base_color: [0.75, 0.73, 0.70, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: r,
            metallic: m,
        })
        .expect("mat")
    };
    let m_die = mk_mat(&mut b, 0.3, 0.0);
    let m_met = mk_mat(&mut b, 0.3, 1.0);
    let dielectric = single(&mut b, m_die, mesh_id, [0.0; 3]);
    let metal_box = single(&mut b, m_met, mesh_id, [0.0; 3]);
    let d_met = dielectric
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(metal_box.rgba.as_chunks::<4>().0)
        .filter(|(x, y)| x[..3] != y[..3])
        .count();
    // Probe (lavapipe 2026-09-29, 320×240, r=0.3 single box): 168 px differ —
    // the Fresnel term lives on the highlight only (diffuse split (1-metallic)
    // does not fire: base albedo identical), floor 100 (−40%, PIT-8).
    println!("E508 probe: metallic-vs-dielectric diff={d_met}");
    assert!(d_met > 100, "metallic column must differ: {d_met}");

    // Roughness ladder (metallic 1.0): roughness widens the GGX lobe, which
    // (a) can only keep or dilute the PEAK energy and (b) eventually drops the
    // >150 core entirely. Probe (lavapipe 2026-09-29, 12×20 sphere, 320×240):
    // peaks [255, 255, 247, 125], >150 spread [11, 28, 58, 0].
    // Two-sided pins: peaks non-increasing with saturation canary at the top
    // step, rough end desaturating ≥60 below it, and the mid-rough >150 core
    // present while the near-flat-dull step loses it.
    let mut peaks = [0u8; 4];
    let mut spread = [0u32; 4];
    for (i, r) in [0.05f32, 0.3, 0.6, 0.95].iter().enumerate() {
        let m = mk_mat(&mut b, *r, 1.0);
        let img = single(&mut b, m, mesh_id, [0.0; 3]);
        peaks[i] = img
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| p[0])
            .max()
            .unwrap_or(0);
        spread[i] = count_family(&img, |p| p[0] > 150 && p[1] > 150 && p[2] > 150);
    }
    println!(
        "E508 probe: roughness ladder peaks={peaks:?} spread>150={spread:?} (r 0.05→0.95, metallic 1.0)"
    );
    assert!(
        peaks[0] >= peaks[1] && peaks[1] >= peaks[2] && peaks[2] >= peaks[3],
        "GGX peak energy must not grow with roughness: {peaks:?}"
    );
    assert_eq!(peaks[0], 255, "smooth step must saturate: {peaks:?}");
    assert!(
        u16::from(peaks[0]) - u16::from(peaks[3]) >= 60,
        "rough end must desaturate ≥60 below smooth: {peaks:?}"
    );
    assert!(
        spread[2] > 0 && spread[3] == 0,
        ">150 core must die off at the rough end: {spread:?}"
    );
    println!("E508 OK: metallic/dielectric differ; roughness ladder monotonic");
    gallery::save_frame(&full, "E508_pbr_materials");
}

// ── resident window path (E506/E507 winit shape) ──────────────────────────────
struct App {
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
    commands: Vec<DrawCommand>,
    rig: CameraRig,
    drag: bool,
    last: (f64, f64),
    bloom_on: bool,
    outline_on: bool,
    ready: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ready {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "E508 PBR grid · drag=orbit wheel=zoom · Esc exit",
            800,
            600,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        let (cmds, _) = scene(&mut core);
        self.commands = cmds;
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
                        self.bloom_on = !self.bloom_on;
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2) => {
                        self.outline_on = !self.outline_on;
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit3) => {
                        self.bloom_on = false;
                        self.outline_on = false;
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
        let post = {
            let mut p = Vec::new();
            if self.bloom_on {
                p.push(PostEffect::Bloom { strength: 0.8 });
            }
            if self.outline_on {
                p.push(PostEffect::Outline { width: 2.0 });
            }
            p
        };
        let frame = frame(&self.commands, w, h, post, &self.rig);
        ctx.present(core, &frame);
        // Pre-existing quirk, kept as-is (band V changes no semantics): this example
        // used to echo `bloom=/outline=` into the title INSIDE the Success arm, and that
        // call is immediately overwritten by the unconditional one below -- so the state
        // echo never reaches the user. Removing the dead call is behaviour-preserving;
        // if the echo is ever wanted, it belongs after present() and this line must go.
        ctx.window
            .set_title("E508 PBR grid · drag=orbit wheel=zoom · Esc exit");
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
                bloom_on: false,
                outline_on: false,
                last: (0.0, 0.0),
                ready: false,
            };
            el.run_app(&mut app)?;
        }
    }
    Ok(())
}
