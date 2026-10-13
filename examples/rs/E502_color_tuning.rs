//! E502 · materials and light, color calibration -- the sRGB full-chain round-trip "what you see is what you get" dual-mode example (CORE-16 band K2).
//! Four color panels with normals parallel to the light -> shade = 1.0 -> readback bytes should ~ equal
//! the original CSS values (the backend linearizes input and the Srgb target encodes output, reciprocal
//! within 8-bit quantization). A mis-installed color domain = the first red this example produces.
//! N6 (WGPU-35 GGX): the non-primary channels carry a ~13-14 Fresnel highlight floor (dielectric
//! F0 = 0.04 x rough-diffuse divergence, not quantization noise) -- tolerance 8->20 absorbs that floor,
//! the mid-gray panel remains the linear-domain mis-install criterion (a domain-mis-install shift is
//! ~60 units, keeping a 3x margin).
//! Dual mode (same system as E501/E801):
//!   no arguments      = resident human-verified window (the IDE visiaengine-example-E502_color_tuning-run road: four panels eyeball-accept the color chain, close/Esc exits)
//!   --frames N        = headless self-asserting fast exit (the ctest road; per-panel byte comparison, FAIL exits nonzero)

#[path = "gallery.rs"]
mod gallery;

use examples::viewer::{Ctx, FormatPolicy};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

/// Four color panels: original CSS values (sRGB domain). Pure primaries / mid-gray / white -- mid-gray best catches a linear-domain mis-install.
const SWATCHES: [(u8, u8, u8); 4] = [
    (255, 0, 0),
    (128, 128, 128), // mis-installed = brightens to ~188, this case's criterion
    (0, 0, 255),
    (255, 255, 255),
];
const HEADLESS_W: u32 = 256; // the row is 1024B = 4x256 aligned (the headless readback road)
const HEADLESS_H: u32 = 256;

/// Light direction (the same position as the backend shadow-off dummy params' old LIGHT: normalize(0.5,0.7,0.4)) --
/// the panel normals take this direction -> ndl = 1 -> shade = 0.35 + 0.65 = 1.0 (panels at full brightness, a pure color-domain judgment).
fn light_dir() -> [f32; 3] {
    let (x, y, z) = (0.5f32, 0.7, 0.4);
    let l = (x * x + y * y + z * z).sqrt();
    [x / l, y / l, z / l]
}

/// Single-panel geometry data (local frame z = 0, normals = light_dir; four corners two triangles). cx = panel center x.
fn swatch_quad(cx: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let n = light_dir();
    let (x0, x1, y0, y1) = (cx - 0.5, cx + 0.5, -1.0, 1.0);
    (
        vec![[x0, y0, 0.0], [x1, y0, 0.0], [x1, y1, 0.0], [x0, y1, 0.0]],
        vec![n; 4],
        vec![0, 1, 2, 0, 2, 3],
    )
}

const IDENT: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// The headless self-asserting road (--frames N): per-panel center pixel vs the original CSS value, tolerance ± 8.
fn prove_headless(frames: u32) {
    let Some(mut b) = HeadlessBackend::new(HEADLESS_W, HEADLESS_H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };
    let mut commands = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
    for (i, (r, g, bl)) in SWATCHES.into_iter().enumerate() {
        let css = [r as f32 / 255.0, g as f32 / 255.0, bl as f32 / 255.0, 1.0];
        let (pos, nrm, idx) = swatch_quad(-1.5 + i as f32);
        let mesh = b
            .create_mesh(&MeshDesc {
                positions: &pos,
                normals: &nrm,
                indices: &idx,
                uv: &[],
            })
            .expect("mesh");
        let mat = b.create_material(css).expect("mat");
        commands.push(DrawCommand::DrawMesh {
            mesh,
            material: mat,
            origin: [0.0; 3],
            transform: IDENT,
        });
    }
    let rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let frame = Frame {
        viewport: Viewport::new(HEADLESS_W, HEADLESS_H, 1.0),
        camera: Camera::ortho(2.0, 2.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [0.0, 0.0, 10.0],
        proj: rig
            .ortho_frame(2.0, HEADLESS_W as f32, HEADLESS_H as f32, 0.1, 100.0)
            .unwrap(),
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    };
    for _ in 0..frames.max(1) {
        let img = b.render_to_pixels(&frame).expect("render");
        gallery::save_frame(&img, "E502_color_tuning");
        let mut bad = 0usize;
        for (i, (cr, cg, cb)) in SWATCHES.into_iter().enumerate() {
            // panel-center pixels: ortho half-width 2 -> px = (0.5 + x/4) * 256, panel centers -1.5..1.5 -> 32/96/160/224
            let px = ((0.5 + (-1.5 + i as f32) / 4.0) * HEADLESS_W as f32) as u32;
            let o = (((HEADLESS_H / 2) * HEADLESS_W + px) * 4) as usize;
            let (r, g, bl) = (img.rgba[o], img.rgba[o + 1], img.rgba[o + 2]);
            // tolerance 20: absorbs the GGX Fresnel floor (probe: red panel g/b = 13, blue panel r/g = 14, not quantization noise);
            // the domain-mis-install shift is ~60 (mid-gray 128 -> ~188) -> a 3x detection margin remains.
            let ok = r.abs_diff(cr) <= 20 && g.abs_diff(cg) <= 20 && bl.abs_diff(cb) <= 20;
            if !ok {
                bad += 1;
            }
            println!(
                "SWATCH {i} css=({cr},{cg},{cb}) read=({r},{g},{bl}) {}",
                if ok { "OK" } else { "✗ color-gamut drift" }
            );
        }
        if bad > 0 {
            println!("FAIL color tuning bad={bad}");
            std::process::exit(1);
        }
    }
    println!(
        "OK color tuning (the sRGB full-chain round trip is reciprocal, what you see is what you get)"
    );
}

// --------------------------- the resident human-verified window road (no arguments) -----------

struct App {
    core: Option<MeshCore>,
    ctx: Option<Ctx>,
    commands: Vec<DrawCommand>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.ctx.is_some() {
            return;
        }
        // Band V canary: window + device + surface + the Srgb format preference come
        // from the shared bootstrap now. PreferSrgb is this example's own claim: the
        // byte-exact color round-trip needs an Srgb target format.
        let Some((ctx, mut core)) = Ctx::new(
            event_loop,
            "VisiaEngine E502 · color-calibration human window: four panels = pure red | mid-gray #808080 (must not brighten) | pure blue | white (no pink cast)",
            960,
            600,
            FormatPolicy::PreferSrgb,
        ) else {
            event_loop.exit();
            return;
        };

        let mut commands = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
        for (i, (r, g, bl)) in SWATCHES.into_iter().enumerate() {
            let css = [r as f32 / 255.0, g as f32 / 255.0, bl as f32 / 255.0, 1.0];
            let (pos, nrm, idx) = swatch_quad(-1.5 + i as f32);
            let mesh = core
                .upload_mesh(&MeshDesc {
                    positions: &pos,
                    normals: &nrm,
                    indices: &idx,
                    uv: &[],
                })
                .expect("mesh");
            let mat = core.upload_material(css).expect("mat");
            commands.push(DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin: [0.0; 3],
                transform: IDENT,
            });
        }
        println!("loaded {} swatches", SWATCHES.len());
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
            WindowEvent::Resized(size) => {
                if let (Some(core), Some(ctx)) = (self.core.as_ref(), self.ctx.as_mut()) {
                    ctx.on_resize(core, size);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && matches!(&event.logical_key, Key::Named(NamedKey::Escape))
                {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(ctx)) = (&mut self.core, &mut self.ctx) else {
                    event_loop.exit();
                    return;
                };
                let rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
                let Some(proj) = rig.ortho_frame(
                    2.0,
                    ctx.config.width as f32,
                    ctx.config.height.max(1) as f32,
                    0.1,
                    100.0,
                ) else {
                    event_loop.exit();
                    return;
                };
                let frame = Frame {
                    viewport: Viewport::new(ctx.config.width, ctx.config.height, 1.0),
                    camera: Camera::ortho(2.0, 2.0, 0.1, 100.0),
                    view_rot: rig.view_rotation(),
                    eye: [0.0, 0.0, 10.0],
                    proj,
                    px_world_scale: 1.0,
                    shadow: None,
                    clip: None,
                    edl: None,
                    post: Vec::new(),
                    commands: self.commands.clone(),
                };
                ctx.present(core, &frame);
                ctx.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let frames = examples::viewer::argv_frames();
    if let Some(n) = frames {
        prove_headless(n);
        return Ok(());
    }
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        core: None,
        ctx: None,
        commands: Vec::new(),
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
