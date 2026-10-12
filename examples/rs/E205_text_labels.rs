//! E205 · image-caption text, labels in dual mode -- the io-text full-chain living example (same CAPI base: face/cache/layout + the label pipeline).
//! Dual mode (same system as E901): no args = resident window (Esc/close exits); --frames N = offscreen asserting fast exit.
//! Note: DejaVu has no CJK glyphs -- the Chinese road = the host injecting a CJK font through the same entry (the demo honestly stays Latin, explicit account).

#[path = "gallery.rs"]
mod gallery;

use visiaengine_io_text::{FontFace, GLYPH_ATLAS_PX, GlyphCache, layout};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, LabelMark, LabelTableDesc, MaterialDesc, MeshDesc,
    RenderBackend, Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, mesh_core::MeshCore};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

use examples::viewer::{Ctx, FormatPolicy};

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

/// Dual-backend upload face (the small trait of the E901 Ctor same system: HeadlessBackend through the trait's proper name, MeshCore through the native one).
trait Up {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64;
    fn mt(&mut self, d: &MaterialDesc) -> u64;
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64;
}
impl Up for HeadlessBackend {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.create_mesh(d).expect("mesh")
    }
    fn mt(&mut self, d: &MaterialDesc) -> u64 {
        self.create_material_desc(d).expect("mat")
    }
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64 {
        self.create_labels(d).expect("labels")
    }
}
impl Up for MeshCore {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.upload_mesh(d).expect("mesh")
    }
    fn mt(&mut self, d: &MaterialDesc) -> u64 {
        self.upload_material_desc(d).expect("mat")
    }
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64 {
        self.create_labels(d).expect("labels")
    }
}

/// Scene construction (headless/window two roads one source): colored ground panels x3 + three label segments.
fn scene(b: &mut impl Up, face: &FontFace, cache: &mut GlyphCache) -> Vec<DrawCommand> {
    let mut cmds = vec![DrawCommand::ClearColor {
        rgba: [0.09, 0.11, 0.16, 1.0],
    }];
    for (i, (x0, y0, col)) in [
        (-24.0f32, -18.0f32, [0.24, 0.55, 0.42, 1.0]),
        (2.0, -6.0, [0.75, 0.42, 0.25, 1.0]),
        (-10.0, 8.0, [0.3, 0.42, 0.72, 1.0]),
    ]
    .iter()
    .enumerate()
    {
        let sz = 18.0 + i as f32 * 4.0;
        let m = b.m(&MeshDesc {
            positions: &[
                [*x0, *y0, 0.0],
                [*x0 + sz, *y0, 0.0],
                [*x0 + sz, *y0 + sz, 0.0],
                [*x0, *y0 + sz, 0.0],
            ],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        });
        let mt = b.mt(&MaterialDesc {
            base_color: *col,
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        });
        cmds.push(DrawCommand::DrawMesh {
            mesh: m,
            material: mt,
            origin: [0.0; 3],
            transform: ID64,
        });
    }
    // three label segments (sizes/color contexts each different): anchor = center point shifted left by pen/2, dx/dy as-is (same expression as add_label)
    let mut marks = Vec::new();
    for (text, cx, cy, size, col) in [
        ("VISIA", 0.0f32, -2.0f32, 34.0f32, [1.0f32, 1.0, 1.0, 1.0]),
        (
            "twin-city / gate-07",
            -4.0,
            14.0,
            16.0,
            [1.0, 0.85, 0.3, 1.0],
        ),
        ("12.5 m", 16.0, -12.0, 20.0, [0.4, 0.95, 1.0, 1.0]),
    ] {
        let (quads, pen) = layout(text, face, cache, size);
        let lin = visiaengine_core::srgb_to_linear(col);
        // center alignment = lateral dx shift (px domain); the anchor stays world-positioned (the units-mixing trap: pen is px
        // and must not enter world coordinates -- the same expression cross-proved at E814/hpp/capi in three places)
        marks.extend(quads.iter().map(|q| {
            LabelMark::new(
                [cx, cy, 0.4],
                lin,
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
    let table = b.lb(&LabelTableDesc { data: &marks });
    cmds.push(DrawCommand::DrawLabels {
        table,
        origin: [0.0; 3],
        transform: ID64,
    });
    cmds
}

fn ortho_frame(commands: Vec<DrawCommand>, w: u32, h: u32, zoom: f32) -> Frame {
    let rig = CameraRig::look_at([0.0, 0.0, 60.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::ortho(zoom, zoom * h as f32 / w as f32, 0.5, 500.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .ortho_frame(zoom, w as f32, h as f32, 0.5, 500.0)
            .expect("ortho"),
        px_world_scale: 2.0 * zoom / w as f32, // the REND-29 exact road: labels stay constant px
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    }
}

fn load_face() -> FontFace {
    FontFace::from_bytes(&std::fs::read(FIX).expect("DejaVu fixture")).expect("face")
}

// -- headless road (--frames N: assertions + OK line) --------------------------------
fn headless_run(_n: u32) {
    const W: u32 = 240;
    const H: u32 = 180;
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let face = load_face();
    let mut cache = GlyphCache::new();
    let cmds = scene(&mut b, &face, &mut cache);
    b.set_glyph_atlas(cache.pixels(), GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
        .expect("atlas");
    let img = b
        .render_to_pixels(&ortho_frame(cmds, W, H, 30.0))
        .expect("render");
    gallery::save_frame(&img, "E205_text_labels");
    let white = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 150 && p[1] > 150 && p[2] > 150)
        .count();
    let warm = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 150 && p[1] > 150 && p[2] < 120)
        .count();
    println!("PROBE white={white} warm={warm}");
    assert!(white > 120, "the main label's white ink absent {white}");
    assert!(warm > 30, "the gold secondary label absent {warm}");
    println!(
        "OK text_labels white={white} warm={warm} (three-context labels pass the whole atlas chain)"
    );
}

// -- interactive road (no args: resident window, Esc/close exits) ----------------------
struct App {
    core: Option<MeshCore>,
    ctx: Option<Ctx>,
    cmds: Vec<DrawCommand>,
    ready: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ready {
            return;
        }
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "VisiaEngine E205 captions · Esc/close to exit (DejaVu without CJK = Latin demo)",
            960,
            600,
            FormatPolicy::SurfaceDefault,
        ) else {
            event_loop.exit();
            return;
        };
        let face = load_face();
        let mut cache = GlyphCache::new();
        let cmds = scene(&mut core, &face, &mut cache);
        core.set_glyph_atlas(cache.pixels(), GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
            .expect("atlas");
        self.cmds = cmds;
        self.core = Some(core);
        self.ctx = Some(ctx);
        self.ready = true;
        self.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == Key::Named(NamedKey::Escape) && event.state.is_pressed() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(sz) => {
                if let (Some(core), Some(ctx)) = (self.core.as_ref(), self.ctx.as_mut()) {
                    ctx.on_resize(core, sz);
                    self.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(ctx)) = (&mut self.core, &mut self.ctx) else {
                    return;
                };
                let frame =
                    ortho_frame(self.cmds.clone(), ctx.config.width, ctx.config.height, 30.0);
                ctx.present(core, &frame);
            }
            _ => {}
        }
    }
}

impl App {
    fn request_redraw(&self) {
        if let Some(ctx) = &self.ctx {
            ctx.request_redraw();
        }
    }
}

fn main() {
    let frames = examples::viewer::argv_frames();
    if let Some(n) = frames {
        headless_run(n);
        return;
    }
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        core: None,
        ctx: None,
        cmds: Vec::new(),
        ready: false,
    };
    event_loop.run_app(&mut app).expect("run");
}
