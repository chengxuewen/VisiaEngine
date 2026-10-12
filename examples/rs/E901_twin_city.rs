//! E901 · vertical-slice seed, twin city -- park.geojson basemap x instanced city x PCSS soft shadows x PNG human-check image.
//! Batch 4's full outcome face (4ab/4c/4de/4f) in one picture; the PNG lands at target/twin_city.png (the T3 human-check list).
//! Dual mode (same system as E501): no args = resident interactive window (left-drag = orbit, wheel = distance, close to exit);
//! `--frames N` = offscreen single frame + assertions + PNG (the ctest-registered form/smoke-twin-city).

#[path = "gallery.rs"]
mod gallery;

use examples::viewer::{Ctx, FormatPolicy, orbit_drag, zoom_dist};

use visiaengine_io_text::{FontFace, GLYPH_ATLAS_PX, GlyphCache, layout};
use visiaengine_render::BackendError;
use visiaengine_render::{
    Camera, CameraRig, ClipSetup, DrawCommand, Frame, Instance, InstanceDesc, LabelMark,
    LabelTableDesc, MaterialDesc, MeshDesc, PointTableDesc, RenderBackend, ShadowSetup,
    StrokeTableDesc, TableId, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

const W: u32 = 640;
const H: u32 = 480;
const SIDE: usize = 16; // 16^2 = 256 blocks (park real scale 93x31m, the city 32x32m centered)
const ID64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// The resource-construction face shared by both roads (MeshId/MaterialId/TableId/InstanceId are all u64 shapes, forwarded directly).
/// A small trait inside this example (legal under the orphan rule); revisit promotion into render-wgpu when a fourth consumer appears.
trait Ctor {
    fn mesh(&mut self, d: &MeshDesc<'_>) -> Result<u64, BackendError>;
    fn material(&mut self, rgba: [f32; 4]) -> Result<u64, BackendError>;
    fn material_desc(&mut self, d: &MaterialDesc) -> Result<u64, BackendError>;
    fn strokes(&mut self, d: &StrokeTableDesc<'_>) -> Result<TableId, BackendError>;
    fn points(&mut self, d: &PointTableDesc<'_>) -> Result<TableId, BackendError>;
    fn instances(&mut self, d: &InstanceDesc<'_>) -> Result<u64, BackendError>;
}
impl Ctor for HeadlessBackend {
    fn mesh(&mut self, d: &MeshDesc<'_>) -> Result<u64, BackendError> {
        self.create_mesh(d)
    }
    fn material(&mut self, rgba: [f32; 4]) -> Result<u64, BackendError> {
        self.create_material(rgba)
    }
    fn material_desc(&mut self, d: &MaterialDesc) -> Result<u64, BackendError> {
        self.create_material_desc(d)
    }
    fn strokes(&mut self, d: &StrokeTableDesc<'_>) -> Result<TableId, BackendError> {
        self.create_strokes(d)
    }
    fn points(&mut self, d: &PointTableDesc<'_>) -> Result<TableId, BackendError> {
        self.create_points(d)
    }
    fn instances(&mut self, d: &InstanceDesc<'_>) -> Result<u64, BackendError> {
        self.create_instances(d)
    }
}
impl Ctor for MeshCore {
    fn mesh(&mut self, d: &MeshDesc<'_>) -> Result<u64, BackendError> {
        self.upload_mesh(d)
    }
    fn material(&mut self, rgba: [f32; 4]) -> Result<u64, BackendError> {
        self.upload_material(rgba)
    }
    fn material_desc(&mut self, d: &MaterialDesc) -> Result<u64, BackendError> {
        self.upload_material_desc(d)
    }
    fn strokes(&mut self, d: &StrokeTableDesc<'_>) -> Result<TableId, BackendError> {
        self.create_strokes(d)
    }
    fn points(&mut self, d: &PointTableDesc<'_>) -> Result<TableId, BackendError> {
        self.create_points(d)
    }
    fn instances(&mut self, d: &InstanceDesc<'_>) -> Result<u64, BackendError> {
        self.create_instances(d)
    }
}

/// The basemap: park.geojson (the fills/strokes/markers full families, the geo_viewer-style conversion) -> command table + layer origin.
fn geo_layer<C: Ctor>(c: &mut C) -> (Vec<DrawCommand>, [f64; 3]) {
    let doc = visiaengine_geo::load_geojson(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/park.geojson"
    ))
    .expect("park.geojson");
    let [x0, y0, x1, y1] = doc.layer_bbox().expect("bbox");
    let origin = [(x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0];
    let neg = [-origin[0], -origin[1]];
    let mut cmds = Vec::new();
    for f in doc.features() {
        for gp in visiaengine_geo::tessellate(&f.kind.shifted(neg), &f.style).expect("tess") {
            match gp {
                visiaengine_geo::GeoPart::Fill(p) => {
                    if p.positions.is_empty() || p.indices.len() < 3 {
                        continue;
                    }
                    let mesh = c
                        .mesh(&MeshDesc {
                            positions: &p.positions,
                            normals: &vec![[0.0, 0.0, 1.0]; p.positions.len()],
                            indices: &p.indices,
                            uv: &[],
                        })
                        .expect("geo mesh");
                    let mat = c.material(p.color).expect("geo mat");
                    cmds.push(DrawCommand::DrawMesh {
                        mesh,
                        material: mat,
                        origin,
                        transform: ID64,
                    });
                }
                visiaengine_geo::GeoPart::Strokes(strips) => {
                    let segs: Vec<visiaengine_render::StrokeSeg> = strips
                        .iter()
                        .flat_map(|s| {
                            s.pts.windows(2).map(move |w| {
                                visiaengine_render::StrokeSeg::new(
                                    [w[0][0], w[0][1], 0.0],
                                    [w[1][0], w[1][1], 0.0],
                                    s.color,
                                    s.width_px,
                                )
                            })
                        })
                        .collect();
                    if segs.is_empty() {
                        continue;
                    }
                    let table = c
                        .strokes(&StrokeTableDesc { data: &segs })
                        .expect("strokes");
                    cmds.push(DrawCommand::DrawStrokes {
                        table,
                        origin,
                        transform: ID64,
                    });
                }
                visiaengine_geo::GeoPart::Markers(ms) => {
                    let marks: Vec<visiaengine_render::PointMark> = ms
                        .iter()
                        .map(|m| {
                            visiaengine_render::PointMark::new(
                                [m.pos[0], m.pos[1], 0.0],
                                m.color,
                                m.radius_px,
                            )
                        })
                        .collect();
                    if marks.is_empty() {
                        continue;
                    }
                    let table = c.points(&PointTableDesc { data: &marks }).expect("points");
                    cmds.push(DrawCommand::DrawPoints {
                        table,
                        origin,
                        transform: ID64,
                    });
                }
            }
        }
    }
    (cmds, origin)
}

fn city() -> Vec<Instance> {
    let step = 2.0f32;
    let span = step * SIDE as f32 / 2.0;
    (0..SIDE)
        .flat_map(|gy| {
            (0..SIDE).map(move |gx| {
                let i = gx + gy * SIDE;
                let h = 1.6 + (i * 7919 % 11) as f32 * 1.15; // a 1.6~13m tower cluster
                let v = 0.72 + (i * 104729 % 7) as f32 * 0.03;
                Instance::new(
                    [
                        14.0 - span + gx as f32 * step + step * 0.4,
                        4.0 - span + gy as f32 * step + step * 0.4,
                        -0.06,
                    ],
                    h,
                    [v, v * 1.02, v * 1.08],
                )
            })
        })
        .collect()
}

/// Ground + city resource upload -> appended commands (after the geo layer; both roads one source).
fn ground_city<C: Ctor>(c: &mut C, origin: [f64; 3], cmds: &mut Vec<DrawCommand>) -> usize {
    let ground = c
        .mesh(&MeshDesc {
            positions: &[
                [-30.0, -30.0, -0.2],
                [30.0, -30.0, -0.2],
                [30.0, 30.0, -0.2],
                [-30.0, 30.0, -0.2],
            ],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        })
        .expect("ground");
    let gdmat = c.material([0.55, 0.55, 0.58, 1.0]).expect("ground mat");
    cmds.push(DrawCommand::DrawMesh {
        mesh: ground,
        material: gdmat,
        origin,
        transform: ID64,
    });
    let (pos, nrm, idx) = unit_box_mesh();
    let boxy = c
        .mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .expect("box");
    let bmat = c
        .material_desc(&MaterialDesc {
            base_color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        })
        .expect("box mat");
    let city = city();
    let iid = c
        .instances(&InstanceDesc { data: &city })
        .expect("city table");
    cmds.push(DrawCommand::DrawInstances {
        mesh: boxy,
        material: bmat,
        instances: iid,
        origin,
        transform: ID64,
    });
    city.len()
}

/// The sun (PCSS soft shadows): south-west high angle, the light's orthographic box scales with the viewport (same position parameters as E501).
fn shadow_setup(origin: [f64; 3], w: u32, h: u32) -> ShadowSetup {
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let leye = [
        origin[0] + f64::from(ld[0] / nn) * 80.0,
        origin[1] + f64::from(ld[1] / nn) * 80.0,
        f64::from(ld[2] / nn) * 80.0,
    ];
    let lrig = CameraRig::look_at(leye, [origin[0], origin[1], 0.0], [0.0, 1.0, 0.0]);
    ShadowSetup {
        proj: lrig
            .ortho_frame(45.0, w as f32, h as f32, 1.0, 160.0)
            .expect("light proj"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5,
        bias: ShadowSetup::DEFAULT_BIAS,
    }
}

// -- headless road (CI contract as-is: single frame + bright/dark family assertions + PNG save + OK line) --
fn headless_run() {
    let mut b = HeadlessBackend::new(W, H)
        .expect("adapter (report honestly when there is no GPU environment)");
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.30, 0.45, 0.70, 1.0],
    }];
    let (geo_cmds, origin) = geo_layer(&mut b);
    commands.extend(geo_cmds);
    let n = ground_city(&mut b, origin, &mut commands);

    let (cx, cy) = (origin[0], origin[1]);
    let rig = CameraRig::look_at([cx, cy - 52.0, 36.0], [cx, cy, 1.0], [0.0, 1.0, 0.0]);
    let proj = rig
        .perspective(rig.fov_y as f32, W as f32 / H as f32, 1.0, 200.0)
        .expect("proj");
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, W as f32 / H as f32, 1.0, 200.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 0.14, // 480px / ~66m view height @ reference depth
        shadow: Some(shadow_setup(origin, W, H)),
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    };
    let img = b.render_to_pixels(&frame).expect("render");
    gallery::save_frame(&img, "E901_twin_city");

    // ---- PNG human-check save + behavior self-check (the city + basemap + shadow-patch three families) ----
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/twin_city.png");
    image::RgbaImage::from_raw(W, H, img.rgba.clone())
        .expect("rgba len")
        .save(path)
        .expect("png save");
    let mut dark = 0u64;
    let mut bright = 0u64; // tone family = shadowed ground + tower dark faces (the long foreground shadow band); bright = sunlit tops/side faces
    for p in img.rgba.as_chunks::<4>().0.iter() {
        let (r, g, bl) = (p[0], p[1], p[2]);
        if r > 150 && g > 150 && bl > 150 {
            bright += 1; // the tower light-color family (v in [0.72,0.9] x shade)
        }
        let lum = r as u32 + g as u32 + bl as u32;
        if lum < 240 && r > 10 && bl.abs_diff(r) < 14 {
            dark += 1; // the shadow-patch family = ambient x gray-blue ground, near-neutral low brightness (the sky clear lum = 381 is naturally excluded)
        }
    }
    assert!(bright > 120, "tower bright faces insufficient {bright}");
    assert!(dark > 30, "shadow patches absent {dark}");
    println!(
        "OK twin_city {} buildings, bright={bright} dark={dark} -> {path}",
        n
    );
    println!(
        "T3 human check: open the PNG above and verify (1) tower shadows fall on the anti-light side (2) basemap strokes/dots without z-fight (3) soft penumbra edges"
    );
    println!(
        "T3 human check (window form): no args = resident window -- C toggles the clip (shadows collapse with the knife) [/] knife height +/- 1m, L toggles labels, drag = orbit, wheel = distance"
    );
}

// -- interactive road (human verification face: MeshCore renders straight to the surface, same scene same shadows -- same system as E501) --
struct App {
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
    rig: CameraRig,
    dragging: Option<(f64, f64)>,
    origin: [f64; 3],
    commands: Vec<DrawCommand>,
    frames_left: Option<u32>,
    /// The interactive control band (D17-h): C = clip toggle, [/] = knife height +/- 1m, L = label visibility, Esc = exit
    label_cmds: Vec<DrawCommand>,
    show_labels: bool,
    clip_h: Option<f32>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.is_some() {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "VisiaEngine E901 twin city · left-drag = orbit, wheel = distance, close to exit",
            960,
            600,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        // scene upload (the same-source function as the headless road)
        let mut commands = vec![DrawCommand::ClearColor {
            rgba: [0.30, 0.45, 0.70, 1.0],
        }];
        let (geo_cmds, origin) = geo_layer(&mut core);
        commands.extend(geo_cmds);
        ground_city(&mut core, origin, &mut commands);
        // three tower-rooftop labels (table + atlas construction; the base commands carry no labels -- redraw frames splice per the L toggle).
        // missing font = the whole section silently downgrades (a demo disaster tolerance, not an error path).
        let label_cmds = FontFace::from_bytes(
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../resources/data/DejaVuSans.ttf"
            ))
            .expect("DejaVu fixture"),
        )
        .ok()
        .and_then(|face| {
            let mut cache = GlyphCache::new();
            let city = city();
            let mut marks = Vec::new();
            for (tag, idx) in [("Tower-A", 3usize), ("Tower-B", 66), ("Tower-C", 189)] {
                let ins = &city[idx];
                let (quads, pen) = layout(tag, &face, &mut cache, 18.0);
                marks.extend(quads.iter().map(|q| {
                    LabelMark::new(
                        [ins.offset[0], ins.offset[1], ins.height + 1.2],
                        [1.0, 1.0, 1.0, 1.0],
                        [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                        [
                            q.size_px[0],
                            q.size_px[1],
                            q.top_left_px[0] - pen / 2.0,
                            q.top_left_px[1],
                        ],
                    )
                }));
            }
            let table = core.create_labels(&LabelTableDesc { data: &marks }).ok()?;
            core.set_glyph_atlas(cache.pixels(), GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
                .ok()?;
            Some(vec![DrawCommand::DrawLabels {
                table,
                origin,
                transform: ID64,
            }])
        });
        self.show_labels = label_cmds.is_some();
        self.label_cmds = label_cmds.unwrap_or_default();
        self.origin = origin;
        self.commands = commands;
        self.core = Some(core);
        self.ctx = Some(ctx);
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => self.handle_key(event_loop, &event),
            WindowEvent::Resized(size) => {
                if let (Some(core), Some(ctx)) = (self.core.as_ref(), self.ctx.as_mut()) {
                    ctx.on_resize(core, size);
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
                if orbit_drag(&mut self.rig, &mut self.dragging, position.x, position.y) {
                    self.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                zoom_dist(&mut self.rig, &delta, 20.0, 180.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(ctx)) = (&mut self.core, &mut self.ctx) else {
                    return;
                };
                let aspect = ctx.config.width as f32 / ctx.config.height.max(1) as f32;
                let Some(proj) = self
                    .rig
                    .perspective(self.rig.fov_y as f32, aspect, 1.0, 200.0)
                else {
                    event_loop.exit();
                    return;
                };
                let mut cmds = self.commands.clone();
                if self.show_labels {
                    cmds.extend(self.label_cmds.iter().cloned());
                }
                let clip = self.clip_h.and_then(|h| {
                    // the world plane z <= h keeps (n = (0,0,-1), d = h): towers shave layer by layer and shadows collapse with the knife
                    ClipSetup::new(&[[0.0, 0.0, -1.0, f64::from(h)]])
                });
                let frame = Frame {
                    viewport: Viewport::new(ctx.config.width, ctx.config.height, 1.0),
                    camera: Camera::perspective(self.rig.fov_y as f32, aspect, 1.0, 200.0),
                    view_rot: self.rig.view_rotation(),
                    eye: self.rig.eye(),
                    proj,
                    px_world_scale: 0.14, // the reference depth stays (same system as E501: window resize does not re-scale line widths)
                    shadow: Some(shadow_setup(
                        self.origin,
                        ctx.config.width,
                        ctx.config.height,
                    )),
                    clip,
                    edl: None,
                    post: Vec::new(),
                    commands: cmds,
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
                    self.request_redraw();
                } else {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn set_title(&self) {
        if let Some(ctx) = &self.ctx {
            ctx.window.set_title(&format!(
                "E901 twin city · C={} [/]={:.1}m · L={} · drag = orbit, wheel = distance, Esc = exit",
                if self.clip_h.is_some() { "ON" } else { "OFF" },
                self.clip_h.unwrap_or(0.0),
                if self.show_labels { "ON" } else { "OFF" },
            ));
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, key: &KeyEvent) {
        if key.state != ElementState::Pressed {
            return;
        }
        match &key.logical_key {
            Key::Character(s) if s.eq_ignore_ascii_case("c") => {
                self.clip_h = if self.clip_h.is_some() {
                    None
                } else {
                    Some(6.0)
                };
                self.set_title();
                self.request_redraw();
            }
            Key::Character(s) if s.eq_ignore_ascii_case("l") => {
                self.show_labels = !self.show_labels;
                self.set_title();
                self.request_redraw();
            }
            Key::Character(s) if s == "[" || s == "-" => {
                if let Some(h) = self.clip_h {
                    self.clip_h = Some((h - 1.0).max(0.5));
                    self.set_title();
                    self.request_redraw();
                }
            }
            Key::Character(s) if s == "]" || s == "+" => {
                if let Some(h) = self.clip_h {
                    self.clip_h = Some((h + 1.0).min(15.0));
                    self.set_title();
                    self.request_redraw();
                }
            }
            Key::Named(NamedKey::Escape) => event_loop.exit(),
            _ => {}
        }
    }

    fn request_redraw(&self) {
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
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
        headless_run(); // the CI road: a single-frame assertion then exit (N in the smoke context = the single-frame contract)
        return Ok(());
    }
    let (cx, cy) = {
        let doc = visiaengine_geo::load_geojson(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resources/data/park.geojson"
        ))
        .expect("park.geojson");
        let [x0, y0, x1, y1] = doc.layer_bbox().expect("bbox");
        ((x0 + x1) / 2.0, (y0 + y1) / 2.0)
    };
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        ctx: None,
        core: None,
        rig: CameraRig::look_at([cx, cy - 52.0, 36.0], [cx, cy, 1.0], [0.0, 1.0, 0.0]),
        dragging: None,
        origin: [0.0; 3],
        commands: Vec::new(),
        frames_left: None,
        label_cmds: Vec::new(),
        show_labels: true,
        clip_h: None,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
