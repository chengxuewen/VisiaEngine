//! E301 · camera and views -- stepless perspective<->orthographic switching (mix_rig interpolation, the 2D/2.5D/3D tri-state demo).
//! Usage: cargo run --example E301_switch_camera [path.glb] [--frames N] (smoke: N = 3)
//! Header correction note: this file's header comment was load_gltf copy-paste residue since G3 (discovered and fixed in batch 5 T1).

use examples::viewer::{Ctx, FormatPolicy};

use visiaengine_io_gltf::load_gltf;
use visiaengine_render::{Camera, CameraRig, DrawCommand, Frame, MeshDesc, MeshId, Viewport};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

/// Upload-time decomposition record: mesh, material, world origin, origin-local pose (D7).
type DrawRecord = (MeshId, MeshId, [f64; 3], [[f64; 4]; 4]);

const CLEAR: [f32; 4] = [0.05, 0.07, 0.10, 1.0];

struct App {
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
    dragging: Option<(f64, f64)>,
    statics: Vec<DrawRecord>, // mesh, material (reused id), world
    frames_left: Option<u32>,
    glb_path: String,
    rig_a: CameraRig,
    rig_b: CameraRig,
    t: f64,
    dir: f64,
}

impl App {
    fn glb_path(&self) -> &str {
        &self.glb_path
    }
}

impl App {
    /// Unified input routing (the E301 root disease): the render camera = mix_rig(rig_a, rig_b, t) --
    /// an old piece writing self.rig = overwritten by mix every frame, keyboard and mouse on the same dead road (the fit also rides both ends).
    fn orbit_view(&mut self, dyaw: f64, dpitch: f64) {
        for r in [&mut self.rig_a, &mut self.rig_b] {
            r.orbit_delta(dyaw, dpitch);
        }
    }
    fn zoom_view(&mut self, f: f64) {
        for r in [&mut self.rig_a, &mut self.rig_b] {
            r.dist = (r.dist * f).clamp(1.0, 600.0);
            r.zoom = (r.zoom * f).clamp(0.05, 4000.0);
        }
    }
    fn request_redraw(&self) {
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
        }
    }
    fn handle_key(&mut self, key: &KeyEvent) {
        if key.state != ElementState::Pressed {
            return;
        }
        match &key.logical_key {
            Key::Character(s) => match s.as_str() {
                "a" => self.orbit_view(0.12, 0.0),
                "d" => self.orbit_view(-0.12, 0.0),
                "w" => self.orbit_view(0.0, 0.08),
                "s" => self.orbit_view(0.0, -0.08),
                "e" => self.zoom_view(1.12),
                "q" => self.zoom_view(1.0 / 1.12),
                _ => return,
            },
            Key::Named(NamedKey::Tab) => self.dir = -self.dir,
            _ => return,
        }
        self.request_redraw();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.is_some() {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "VisiaEngine E301 · 2D<->3D (Tab switches, left-drag = orbit, wheel = zoom, close to exit)",
            960,
            600,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        // Scene upload: each glTF entity -> mesh + material
        let doc = match load_gltf(self.glb_path()) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("load failed: {e}");
                event_loop.exit();
                return;
            }
        };
        let mut bbox = examples::BBox::new();
        for e in doc.entities() {
            for pt in &e.mesh.positions {
                bbox.push(*pt, &e.world);
            }
            let Ok(mesh) = core.upload_mesh(&MeshDesc {
                positions: &e.mesh.positions,
                normals: &e.mesh.normals,
                indices: &e.mesh.indices,
                uv: &[],
            }) else {
                eprintln!("skip entity: mesh upload failed");
                continue;
            };
            let Ok(mat) = core.upload_material(e.mesh.base_color) else {
                continue;
            };
            let origin = [e.world[3][0], e.world[3][1], e.world[3][2]];
            let mut local = e.world;
            local[3] = [0.0, 0.0, 0.0, 1.0];
            self.statics.push((mesh, mat, origin, local));
        }
        println!("loaded {} entities", self.statics.len());
        // Load-time framing fit (the BBox gate): three rigs share the target; 2D orthographic zoom = radius x 1.35, 3D perspective dist = x2.6
        let r = bbox.radius();
        for rig in [&mut self.rig_a, &mut self.rig_b] {
            rig.target = bbox.center();
            rig.dist = r * 2.6;
            rig.zoom = (r * 1.35).max(0.2);
        }
        self.core = Some(core);
        self.ctx = Some(ctx);
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(core), Some(ctx)) = (self.core.as_ref(), self.ctx.as_mut()) {
                    ctx.on_resize(core, size);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => self.handle_key(&event),
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
                        self.orbit_view((x - lx) * 0.006, (y - ly) * 0.006);
                    }
                    self.dragging = Some((x, y));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                self.zoom_view(1.0 - d * 0.0012);
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(ctx)) = (&mut self.core, &mut self.ctx) else {
                    event_loop.exit();
                    return;
                };
                let st = std::mem::take(&mut self.statics);
                let mut commands = vec![DrawCommand::ClearColor { rgba: CLEAR }];
                commands.extend(
                    st.iter()
                        .map(|(m, mat, origin, local)| DrawCommand::DrawMesh {
                            mesh: *m,
                            material: *mat,
                            origin: *origin,
                            transform: *local,
                        }),
                );
                self.statics = st;
                self.t = (self.t + self.dir / 60.0).clamp(0.0, 1.0);
                let rig = CameraRig::mix_rig(&self.rig_a, &self.rig_b, self.t);
                let aspect = ctx.config.width as f32 / ctx.config.height.max(1) as f32;
                let (near, far) = ((rig.dist * 0.01) as f32, (rig.dist * 30.0) as f32);
                // the projection type switches at t = 0.5 (rig parameters interpolate continuously; matrix-level blending belongs to a later camera slice)
                let (camera, proj) = if self.t < 0.5 {
                    let hw = rig.zoom as f32;
                    (
                        Camera::ortho(hw, hw * aspect, near, far),
                        rig.ortho_frame(
                            hw,
                            ctx.config.width as f32,
                            ctx.config.height as f32,
                            near,
                            far,
                        ),
                    )
                } else {
                    (
                        Camera::perspective(rig.fov_y as f32, aspect, near, far),
                        rig.perspective(rig.fov_y as f32, aspect, near, far),
                    )
                };
                let Some(proj) = proj else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(ctx.config.width, ctx.config.height, 1.0),
                    camera,
                    view_rot: rig.view_rotation(),
                    eye: rig.eye(),
                    proj,
                    px_world_scale: 1.0,
                    shadow: None,
                    clip: None,
                    edl: None,
                    post: Vec::new(),
                    commands,
                };
                ctx.present(core, &frame);
                let again = match self.frames_left {
                    None => true,
                    Some(1) => false,
                    Some(n) => {
                        self.frames_left = Some(n - 1);
                        true
                    }
                };
                if again {
                    ctx.request_redraw();
                } else {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut path = String::from("resources/data/hierarchy.glb");
    let mut frames: Option<u32> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--frames" => {
                frames = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n: &u32| *n > 0);
            }
            p => path = p.to_string(),
        }
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        ctx: None,
        core: None,
        dragging: None,
        statics: Vec::new(),
        frames_left: frames,
        glb_path: path,
        rig_a: CameraRig::orbit([0.0; 3], 0.001, 1.45, 24.0, 2.2, 1.1, 0.01, 500.0),
        rig_b: CameraRig::orbit(
            [0.0; 3],
            0.7,
            0.35,
            9.0,
            1.0,
            std::f64::consts::FRAC_PI_3,
            0.01,
            500.0,
        ),
        t: 0.0,
        dir: 1.0,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
