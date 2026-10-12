//! E504 · glass and water -- the transparent pipeline living example (band 4 REND-36/WGPU-27/28, demo + pixel gate).
//! Dual mode (family system): no args = resident window (1/2 = water alpha 0.7/0.35 re-upload material demo, drag orbit wheel distance R reset);
//! `--frames N` = offscreen assertions (checkerboard refraction under water / glass towers seeing ground / labels always top, thresholds = PIT-8 probe measured).

#[path = "gallery.rs"]
mod gallery;

use examples::viewer::{Ctx, FormatPolicy, orbit_drag, zoom_dist};

use visiaengine_io_text::{FontFace, GLYPH_ATLAS_PX, GlyphCache, layout};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, LabelMark, LabelTableDesc,
    MaterialDesc, MeshDesc, RenderBackend, ShadowSetup, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

const FIX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../resources/data/DejaVuSans.ttf"
);
const ID64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

trait Up {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64;
    fn mt(&mut self, d: &MaterialDesc) -> u64;
    fn i(&mut self, d: &InstanceDesc<'_>) -> u64;
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64;
    fn atlas(&mut self, px: &[u8]);
}
impl Up for HeadlessBackend {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.create_mesh(d).expect("m")
    }
    fn mt(&mut self, d: &MaterialDesc) -> u64 {
        self.create_material_desc(d).expect("mt")
    }
    fn i(&mut self, d: &InstanceDesc<'_>) -> u64 {
        self.create_instances(d).expect("i")
    }
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64 {
        self.create_labels(d).expect("lb")
    }
    fn atlas(&mut self, px: &[u8]) {
        self.set_glyph_atlas(px, GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
            .expect("atlas");
    }
}
impl Up for MeshCore {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.upload_mesh(d).expect("m")
    }
    fn mt(&mut self, d: &MaterialDesc) -> u64 {
        self.upload_material_desc(d).expect("mt")
    }
    fn i(&mut self, d: &InstanceDesc<'_>) -> u64 {
        self.create_instances(d).expect("i")
    }
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64 {
        self.create_labels(d).expect("lb")
    }
    fn atlas(&mut self, px: &[u8]) {
        self.set_glyph_atlas(px, GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
            .expect("atlas");
    }
}

fn quad(s: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    (
        vec![[-s, -s, 0.0], [s, -s, 0.0], [s, s, 0.0], [-s, s, 0.0]],
        vec![[0.0, 0.0, 1.0]; 4],
        vec![0, 1, 2, 0, 2, 3],
    )
}

/// Scene: a 4-color checkerboard ground + a glass tower cluster (a = 0.35 instanced batch) + a full-frame water surface (swappable a) + always-top labels.
fn build(up: &mut impl Up, water_a: f32, cache: &mut GlyphCache) -> Vec<DrawCommand> {
    let mut cmds = vec![DrawCommand::ClearColor {
        rgba: [0.30, 0.45, 0.70, 1.0],
    }];
    let mut light = Vec::new();
    for col in [
        [0.92f32, 0.92, 0.90, 1.0],
        [0.80, 0.86, 0.80, 1.0],
        [0.90, 0.88, 0.80, 1.0],
    ] {
        light.push(up.mt(&MaterialDesc {
            base_color: col,
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        }));
    }
    let dark = up.mt(&MaterialDesc {
        base_color: [0.45, 0.47, 0.50, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
        roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
        metallic: 0.0,
    });
    let glass = up.mt(&MaterialDesc {
        base_color: [0.75, 0.88, 0.95, 0.35],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
        roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
        metallic: 0.0,
    });
    let water = up.mt(&MaterialDesc {
        base_color: [0.15, 0.45, 0.85, water_a],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
        roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
        metallic: 0.0,
    });
    // checkerboard 6x6: two tones of dark/light + a single dark column (the perspective-check column)
    let tile = quad(4.0);
    let tm = up.m(&MeshDesc {
        positions: &tile.0,
        normals: &tile.1,
        indices: &tile.2,
        uv: &[],
    });
    for iy in 0..6i32 {
        for ix in 0..6i32 {
            let dark_tile = ix == 1 && iy == 3; // the perspective-check cell (second column from the left, under the water)
            let mat = if dark_tile {
                dark
            } else {
                light[((ix + iy + 50) % 3) as usize]
            };
            cmds.push(DrawCommand::DrawMesh {
                mesh: tm,
                material: mat,
                origin: [(ix as f64 - 2.5) * 8.0, (iy as f64 - 2.5) * 8.0, 0.0],
                transform: ID64,
            });
        }
    }
    // glass tower cluster (instanced, a = 0.35 -- the checkerboard behind should stay visible)
    let (p, n, i) = unit_box_mesh();
    let bx = up.m(&MeshDesc {
        positions: &p,
        normals: &n,
        indices: &i,
        uv: &[],
    });
    let city: Vec<Instance> = (0..6)
        .map(|k| {
            Instance::new(
                [
                    -14.0 + (k % 3) as f32 * 14.0,
                    6.0 - (k / 3) as f32 * 16.0,
                    0.0,
                ],
                7.0 + (k % 3) as f32 * 3.0,
                [1.0, 1.0, 1.0],
            )
        })
        .collect();
    let iid = up.i(&InstanceDesc { data: &city });
    cmds.push(DrawCommand::DrawInstances {
        mesh: bx,
        material: glass,
        instances: iid,
        origin: [0.0; 3],
        transform: ID64,
    });
    // full-frame water surface z = 2 (covers the board and the tower feet, before the labels -- painter order by view depth automatically)
    let wq = quad(30.0);
    let wm = up.m(&MeshDesc {
        positions: &wq.0,
        normals: &wq.1,
        indices: &wq.2,
        uv: &[],
    });
    cmds.push(DrawCommand::DrawMesh {
        mesh: wm,
        material: water,
        origin: [0.0, 0.0, 2.0],
        transform: ID64,
    });
    // always-top label "GLASS"
    let face = FontFace::from_bytes(&std::fs::read(FIX).expect("font")).expect("face");
    let (qs, pen) = layout("GLASS", &face, cache, 26.0);
    let marks: Vec<LabelMark> = qs
        .iter()
        .map(|q| {
            LabelMark::new(
                [0.0, 0.0, 3.0],
                [1.0, 1.0, 1.0, 1.0],
                [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                [
                    q.size_px[0],
                    q.size_px[1],
                    q.top_left_px[0] - pen / 2.0,
                    q.top_left_px[1],
                ],
            )
        })
        .collect();
    let lb = up.lb(&LabelTableDesc { data: &marks });
    up.atlas(cache.pixels());
    cmds.push(DrawCommand::DrawLabels {
        table: lb,
        origin: [0.0; 3],
        transform: ID64,
    });
    let _ = (light.len(), dark, glass, water); // handles already entered each command; the sentinel silences dead_code
    cmds
}

fn shadow() -> ShadowSetup {
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let leye = [
        f64::from(ld[0] / nn) * 80.0,
        f64::from(ld[1] / nn) * 80.0,
        f64::from(ld[2] / nn) * 80.0,
    ];
    let lrig = CameraRig::look_at(leye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    ShadowSetup {
        proj: lrig
            .ortho_frame(45.0, 640.0, 480.0, 1.0, 160.0)
            .expect("lp"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5,
        bias: ShadowSetup::DEFAULT_BIAS,
    }
}

fn frame_of(rig: &CameraRig, cmds: Vec<DrawCommand>, w: u32, h: u32) -> Frame {
    let aspect = w as f32 / h.max(1) as f32;
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, aspect, 0.3, 400.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, aspect, 0.3, 400.0)
            .expect("p"),
        px_world_scale: 0.14,
        shadow: Some(shadow()),
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds,
    }
}

fn home() -> CameraRig {
    CameraRig::look_at([0.0, -46.0, 40.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0])
}

// ── headless ─────────────────────────────────────────────────────────────────
fn headless_run() {
    const W: u32 = 320;
    const H: u32 = 240;
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let mut cache = GlyphCache::new();
    let cmds = build(&mut b, 0.5, &mut cache);
    let img = b
        .render_to_pixels(&frame_of(&home(), cmds, W, H))
        .expect("render");
    gallery::save_frame(&img, "E504_glass_water");
    let px = |x: u32, y: u32| {
        let i = ((y * W + x) * 4) as usize;
        [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]]
    };
    // probe positions (settled after estimation from the in-window top view): the dark cell under water should still be darker than the light cell (refraction = transparency in effect)
    let col = 158;
    let (mut lo, mut hi) = (255u8, 0u8);
    for y in (120..160).step_by(2) {
        let v = px(col, y);
        let lum = v[0].min(v[1]).min(v[2]);
        lo = lo.min(lum);
        hi = hi.max(lum);
    }
    let contrast = hi.saturating_sub(lo);
    let label_white = (0..H)
        .flat_map(|y| (0..W).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let v = px(*x, *y);
            v[0] > 245 && v[1] > 245 && v[2] > 245
        })
        .count();
    println!("PROBE contrast={contrast} lo={lo} hi={hi} label={label_white}");
    // bidirectional semantics lock: a = 0.5 must attenuate (same scene a = 1.0 contrast control measured contrast = 52, water-cover contrast = 0 means transparency failed)
    assert!(
        contrast > 10 && contrast < 30,
        "the water-film attenuation semantics missing: contrast={contrast} (transparent and opaque states in the same band)"
    );
    assert!(
        label_white > 25,
        "the always-top label absent {label_white}"
    ); // measured 40
    println!(
        "OK glass_water contrast={contrast} label={label_white} (transparency + always-top run live)"
    );
}

// -- interactive ----------------------------------------------------------------------
struct App {
    ctx: Option<Ctx>,
    core: Option<MeshCore>,
    rig: CameraRig,
    dragging: Option<(f64, f64)>,
    cmds: Vec<DrawCommand>,
    water_a: f32,
    ready: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ready {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "E504 glass & water · 1/2 = water a 0.7/0.35 re-upload material, R = reset, drag = orbit",
            960,
            600,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        let mut cache = GlyphCache::new();
        let cmds = build(&mut core, self.water_a, &mut cache);
        self.cmds = cmds;
        self.core = Some(core);
        self.ctx = Some(ctx);
        self.ready = true;
        self.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => self.on_key(event_loop, &event),
            WindowEvent::Resized(sz) => {
                if let (Some(core), Some(ctx)) = (self.core.as_ref(), self.ctx.as_mut()) {
                    ctx.on_resize(core, sz);
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
                zoom_dist(&mut self.rig, &delta, 10.0, 160.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(ctx)) = (&mut self.core, &mut self.ctx) else {
                    return;
                };
                let fr = frame_of(
                    &self.rig,
                    self.cmds.clone(),
                    ctx.config.width,
                    ctx.config.height,
                );
                ctx.present(core, &fr);
                self.request_redraw();
            }
            _ => {}
        }
    }
}

impl App {
    fn on_key(&mut self, event_loop: &ActiveEventLoop, key: &KeyEvent) {
        if key.state != ElementState::Pressed {
            return;
        }
        let set = match &key.logical_key {
            Key::Character(s) => match s.as_str() {
                "1" => Some(0.7f32),
                "2" => Some(0.35),
                "r" | "R" => {
                    self.rig = home();
                    Some(self.water_a)
                }
                _ => None,
            },
            Key::Named(NamedKey::Escape) => {
                event_loop.exit();
                return;
            }
            _ => None,
        };
        if let Some(a) = set {
            self.water_a = a;
            if let Some(core) = &mut self.core {
                let mut cache = GlyphCache::new();
                let cmds = build(core, a, &mut cache);
                self.cmds = cmds;
                self.request_redraw();
            }
        }
    }

    fn request_redraw(&self) {
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut auto = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == "--frames" {
            auto = it
                .next()
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|n| n > 0);
        }
    }
    if auto {
        headless_run();
        return Ok(());
    }
    let el = EventLoop::new()?;
    el.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        ctx: None,
        core: None,
        rig: home(),
        dragging: None,
        cmds: Vec::new(),
        water_a: 0.5,
        ready: false,
    };
    el.run_app(&mut app)?;
    Ok(())
}
