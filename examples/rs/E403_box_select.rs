//! E403 · interaction, rubber-band box select -- left-drag = rectangular box selection (select all entities inside the box) demo.
//! Usage: cargo run --example E403_box_select [options]
//! Interaction: left-drag = box select (a 5x5 sampling-ray grid decides the entities inside the box, a hit joins the selection) | release = commit the selection
//! right button = clear | R = clear | wheel = distance | close window = exit.
//! --frames N: render-budget exit (the ctest/xvfb road, the registry argv-exclusive, C15).
//!
//! Box-select decision = sampling-ray reuse (the REND-21/23 pick base, zero new math): a 5x5 grid of points inside the rectangle
//! each fires one pick ray, and the entities hit merge into the selection set. In sparse box scenes this is equivalent
//! to the exact projection decision; the large-scene upgrade path = a frustum-cull pure function (ticket-based).

use examples::viewer::{Ctx, FormatPolicy};

use visiaengine_core::{EntityId, Scene};
use visiaengine_render::screen_to_ray_persp;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialId, MeshCandidate, MeshDesc, MeshId, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::unit_box_mesh;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

const CLEAR: [f32; 4] = [0.05, 0.07, 0.10, 1.0];
const SELECTED: [f32; 4] = [1.0, 0.83, 0.29, 1.0]; // highlight yellow (the WGPU-12 color family)

fn placed(x: f64, y: f64, z: f64, s: f64) -> [[f64; 4]; 4] {
    [
        [s, 0.0, 0.0, 0.0],
        [0.0, s, 0.0, 0.0],
        [0.0, 0.0, s, 0.0],
        [x, y, z, 1.0],
    ]
}

struct BoxRec {
    entity: EntityId,
    name: &'static str,
    world: [[f64; 4]; 4],
    selected: bool,
}

struct App {
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
    rig: CameraRig,
    boxes: Vec<BoxRec>,
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    mesh: Option<MeshId>,
    mat_hl: Option<MaterialId>,
    mats: Vec<MaterialId>,
    cursor: (f64, f64),
    /// Drag state: start + current point (rubber-band diagonal); None = not dragging.
    band: Option<((f64, f64), (f64, f64))>,
    frames_left: Option<u32>,
}

impl App {
    fn request_redraw(&self) {
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
        }
    }

    /// Screen coordinates -> ray -> pick_meshes -> box index (the same pick road as E402).
    fn pick_at(&self, x: f64, y: f64) -> Option<usize> {
        let Some(ctx) = &self.ctx else {
            return None;
        };
        let ray = screen_to_ray_persp(
            &self.rig,
            x as f32,
            y as f32,
            ctx.config.width as f32,
            ctx.config.height as f32,
        )?;
        let cands: Vec<MeshCandidate> = self
            .boxes
            .iter()
            .map(|b| MeshCandidate {
                entity: b.entity,
                positions: &self.positions,
                indices: &self.indices,
                world: &b.world,
                origin: [0.0; 3],
            })
            .collect();
        let hit = visiaengine_render::pick_meshes(ray, &cands)?;
        self.boxes
            .iter()
            .position(|b| b.entity == hit.entity)
            .filter(|_| hit.t.is_finite())
    }

    /// Rubber-band rectangle -> 5x5 sampling rays -> the set of hit entities (the body of the box-select decision).
    fn select_band(&mut self, (x0, y0): (f64, f64), (x1, y1): (f64, f64)) {
        let (min_x, max_x) = (x0.min(x1), x0.max(x1));
        let (min_y, max_y) = (y0.min(y1), y0.max(y1));
        let mut hits: Vec<usize> = Vec::new();
        for sy in 0..5 {
            for sx in 0..5 {
                let px = min_x + (max_x - min_x) * f64::from(sx) / 4.0;
                let py = min_y + (max_y - min_y) * f64::from(sy) / 4.0;
                if let Some(i) = self.pick_at(px, py)
                    && !hits.contains(&i)
                {
                    hits.push(i);
                }
            }
        }
        let names: Vec<&str> = hits.iter().map(|&i| self.boxes[i].name).collect();
        for i in hits {
            self.boxes[i].selected = true;
        }
        println!(
            "BAND-SELECT [{}] selected_total={}",
            names.join(","),
            self.boxes.iter().filter(|b| b.selected).count()
        );
        self.request_redraw();
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: &KeyEvent) {
        if key.state != ElementState::Pressed {
            return;
        }
        match &key.logical_key {
            Key::Character(s) if s.eq_ignore_ascii_case("r") => {
                for b in &mut self.boxes {
                    b.selected = false;
                }
                println!("CLEAR selection");
                self.request_redraw();
            }
            Key::Named(NamedKey::Escape) => event_loop.exit(),
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.is_some() {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "VisiaEngine E403 · box select (left-drag = select all inside, right/R = clear, wheel = distance)",
            960,
            600,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        let (positions, normals, indices) = unit_box_mesh();
        let mesh = core
            .upload_mesh(&MeshDesc {
                positions: &positions,
                normals: &normals,
                indices: &indices,
                uv: &[],
            })
            .expect("upload mesh");
        // lay out a 6-box 2x3 grid (the density feel of the box-select demo).
        let palette = [
            [0.90, 0.25, 0.25, 1.0f32],
            [0.20, 0.75, 0.35, 1.0],
            [0.20, 0.45, 0.90, 1.0],
            [0.85, 0.55, 0.20, 1.0],
            [0.55, 0.20, 0.85, 1.0],
            [0.20, 0.80, 0.80, 1.0],
        ];
        let mats: Vec<MaterialId> = palette
            .iter()
            .map(|c| core.upload_material(*c).expect("upload material"))
            .collect();
        let mat_hl = core.upload_material(SELECTED).expect("upload hl");

        let mut scene = Scene::new();
        let defs: [(&str, [[f64; 4]; 4]); 6] = [
            ("A", placed(-3.0, -1.5, 1.0, 1.6)),
            ("B", placed(0.0, -1.5, 1.0, 1.6)),
            ("C", placed(3.0, -1.5, 1.0, 1.6)),
            ("D", placed(-3.0, 1.5, 1.0, 1.6)),
            ("E", placed(0.0, 1.5, 1.0, 1.6)),
            ("F", placed(3.0, 1.5, 1.0, 1.6)),
        ];
        self.boxes = defs
            .iter()
            .map(|(name, world)| BoxRec {
                entity: scene.spawn(),
                name,
                world: *world,
                selected: false,
            })
            .collect();
        println!("loaded {} entities", self.boxes.len());

        self.core = Some(core);
        self.ctx = Some(ctx);
        self.mesh = Some(mesh);
        self.mats = mats;
        self.mat_hl = Some(mat_hl);
        self.positions = positions;
        self.indices = indices;
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

            WindowEvent::KeyboardInput { event, .. } => self.handle_key(event_loop, &event),
            WindowEvent::MouseInput { button, state, .. } => match (button, state) {
                (MouseButton::Left, ElementState::Pressed) => {
                    self.band = Some((self.cursor, self.cursor));
                }
                (MouseButton::Left, ElementState::Released) => {
                    if let Some((start, end)) = self.band.take() {
                        self.select_band(start, end);
                    }
                }
                (MouseButton::Right, ElementState::Pressed) => {
                    for b in &mut self.boxes {
                        b.selected = false;
                    }
                    println!("CLEAR selection (right-click)");
                    self.request_redraw();
                }
                _ => {}
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
                if let Some((start, _)) = self.band {
                    self.band = Some((start, self.cursor));
                    self.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                self.rig.dist = (self.rig.dist * (1.0 - d * 0.0012)).clamp(3.0, 80.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(ctx)) = (&mut self.core, &mut self.ctx) else {
                    event_loop.exit();
                    return;
                };
                let (Some(mesh), Some(hl)) = (self.mesh, self.mat_hl) else {
                    event_loop.exit();
                    return;
                };
                let mut commands = vec![DrawCommand::ClearColor { rgba: CLEAR }];
                for (i, b) in self.boxes.iter().enumerate() {
                    let material = if b.selected { hl } else { self.mats[i] };
                    commands.push(DrawCommand::DrawMesh {
                        mesh,
                        material,
                        origin: [0.0, 0.0, 0.0],
                        transform: b.world,
                    });
                }
                let aspect = ctx.config.width as f32 / ctx.config.height.max(1) as f32;
                let (near, far) = ((self.rig.dist * 0.01) as f32, (self.rig.dist * 30.0) as f32);
                let Some(proj) = self
                    .rig
                    .perspective(self.rig.fov_y as f32, aspect, near, far)
                else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(ctx.config.width, ctx.config.height, 1.0),
                    camera: Camera::perspective(self.rig.fov_y as f32, aspect, near, far),
                    view_rot: self.rig.view_rotation(),
                    eye: self.rig.eye(),
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
        ctx: None,
        core: None,
        rig: CameraRig::orbit([0.0, 0.0, 1.0], 0.0, 1.2, 12.0, 1.0, 1.0, 0.1, 100.0),
        boxes: Vec::new(),
        positions: Vec::new(),
        indices: Vec::new(),
        mesh: None,
        mat_hl: None,
        mats: Vec::new(),
        cursor: (f64::from(960 / 2), f64::from(600 / 2)),
        band: None,
        frames_left: frames,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
