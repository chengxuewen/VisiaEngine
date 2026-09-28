//! E507 HDR + Tonemap — first N5 consumer: PostEffect::Tonemap live demo
//! (REND-44) on top of the N4 chain framework (WGPU-34). The companion .hdr
//! loader (visiaengine-io-hdr, IO-17) is exercised by its own unit tests +
//! the committed fixture `resources/data/demo_sky.hdr`; this example proves
//! the TONEMAP effect on a bright scene.
//!
//! Dual-mode (E506 same shape): MeshCore direct + HeadlessBackend assert.
//! - No-arg = resident window: `1` off, `2` Reinhard, `3` ACES, `0` bloom
//!   toggle (stacks after the curve — compose proof); title echoes, Esc exits.
//! - `--frames N` = offscreen assert path: off vs Reinhard vs ACES all
//!   pairwise differ; tonemap+bloom composes; off deterministic (canary).
//!
//! Honesty (REND-44 clause): the pipeline renders LDR — tonemap on LDR input
//! is contrast reshape (mood), not true HDR display. True HDR background
//! needs the float-texture path (ticket).

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

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

/// Upload seam (E506's Up trait shape): one scene builder serving both
/// HeadlessBackend (assert path) and MeshCore (window path). Ground + bright
/// "sun" quad (luma above the bloom threshold — tonemap fodder) + one tower.
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
    });
    commands.push(DrawCommand::DrawMesh {
        mesh: gm,
        material: gt,
        origin: [0.0; 3],
        transform: T4,
    });
    // Bright standing "sun" quad (standing quad like E506's bloom fodder —
    // luma above the 0.25 threshold after sRGB encode).
    let bright = [
        [-1.0f32, -1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 3.0],
        [-1.0, 1.0, 3.0],
    ];
    let bm = up.m(&MeshDesc {
        uv: &[],
        positions: &bright,
        normals: &[[0.0, -1.0, 0.0]; 4],
        indices: &[0, 1, 2, 0, 2, 3],
    });
    let bt = up.t(&MaterialDesc {
        base_color: [0.95, 0.90, 0.75, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    commands.push(DrawCommand::DrawMesh {
        mesh: bm,
        material: bt,
        origin: [0.0; 3],
        transform: T4,
    });
    // One dim green tower (context, and a low-luma region the ACES toe moves
    // but Reinhard barely touches — the probe asymmetry live).
    let tower = [
        [2.0f32, 0.5, 0.0],
        [3.0, 0.5, 0.0],
        [3.0, 2.5, 0.0],
        [2.0, 2.5, 0.0],
        [2.0, 0.5, 1.8],
        [3.0, 0.5, 1.8],
        [3.0, 2.5, 1.8],
        [2.0, 2.5, 1.8],
    ];
    let idx: Vec<u32> = vec![
        4, 5, 6, 4, 6, 7, 0, 1, 5, 0, 5, 4, 1, 2, 6, 1, 6, 5, 2, 3, 7, 2, 7, 6, 3, 0, 4, 3, 4, 7,
    ];
    let tm = up.m(&MeshDesc {
        uv: &[],
        positions: &tower,
        normals: &[[0.0, 0.0, 1.0]; 8],
        indices: &idx,
    });
    let tt = up.t(&MaterialDesc {
        base_color: [0.30, 0.65, 0.35, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    commands.push(DrawCommand::DrawMesh {
        mesh: tm,
        material: tt,
        origin: [0.0; 3],
        transform: T4,
    });
    commands
}

fn frame(cmds: &[DrawCommand], post: Vec<PostEffect>, w: u32, h: u32) -> Frame {
    let rig = CameraRig::look_at([0.0, -14.0, 9.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = rig
        .perspective(rig.fov_y as f32, w as f32 / h as f32, 0.1, 1000.0)
        .expect("persp");
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, w as f32 / h as f32, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
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

    // Fixture sanity: the committed .hdr decodes (IO-17 chain alive in-tree;
    // deep coverage lives in the io-hdr crate tests).
    let hdr_bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/demo_sky.hdr"
    ))
    .expect("demo_sky.hdr fixture (scripts/gen_hdr.py regenerates)");
    let sky = visiaengine_io_hdr::decode_hdr(&hdr_bytes).expect("decode demo_sky.hdr");
    let over1 = sky
        .pixels
        .iter()
        .filter(|p| p[0] > 1.0 || p[1] > 1.0 || p[2] > 1.0)
        .count();
    println!(
        "E507 fixture: {} px, {over1} px over 1.0 (HDR proof)",
        sky.pixels.len()
    );
    assert_eq!((sky.width, sky.height), (64, 32));
    assert!(over1 > 400, "fixture must contain >1.0 samples: {over1}");

    let off_a = b.render_to_pixels(&frame(&cmds, Vec::new(), W, H)).unwrap();
    let off_b = b.render_to_pixels(&frame(&cmds, Vec::new(), W, H)).unwrap();
    assert_eq!(off_a.rgba, off_b.rgba, "off state must be deterministic");

    let reinhard = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::tonemap(0).expect("tonemap")],
            W,
            H,
        ))
        .unwrap();
    let aces = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::tonemap(1).expect("tonemap")],
            W,
            H,
        ))
        .unwrap();
    let aces_bloom = b
        .render_to_pixels(&frame(
            &cmds,
            vec![
                PostEffect::tonemap(1).expect("tonemap"),
                PostEffect::bloom(0.8).expect("bloom"),
            ],
            W,
            H,
        ))
        .unwrap();

    let d_rein = diff_px(&off_a, &reinhard);
    let d_aces = diff_px(&off_a, &aces);
    let d_modes = diff_px(&reinhard, &aces);
    let d_compose = diff_px(&aces, &aces_bloom);
    // Probe (lavapipe, 320×240 city scene, 76800 px): off-vs-reinhard=14957,
    // off-vs-aces=76800 (filmic toe moves EVERY pixel), reinhard-vs-aces
    // =76800. Floors (PIT-8): −40%.
    println!(
        "E507 probe: off-vs-reinhard={d_rein} off-vs-aces={d_aces} rein-vs-aces={d_modes} compose={d_compose}"
    );
    assert!(d_rein > 9000, "reinhard must reshape: {d_rein}");
    assert!(d_aces > 45000, "aces must move the whole frame: {d_aces}");
    assert!(d_modes > 45000, "curves must differ: {d_modes}");
    assert!(
        d_compose > 700,
        "bloom after tonemap must compose: {d_compose}"
    );
    println!("E507 OK: off deterministic; reinhard/aces/bloom-compose pixel-verified");
    gallery::save_frame(&aces, "E507_hdr_tonemap");
}

// ── resident window path (E506 winit shape) ──────────────────────────────────
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    commands: Vec<DrawCommand>,
    /// None = off; Some(mode) = tonemap mode (0 Reinhard, 1 ACES).
    mode: Option<u32>,
    bloom: bool,
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
                        .with_title(
                            "E507 tonemap off · 1=off 2=reinhard 3=aces 0=bloom · Esc exit",
                        ),
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
            label: Some("visiaengine-e507"),
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
        self.commands = scene(&mut core);
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);
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
                        self.mode = None;
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2) => {
                        self.mode = Some(0);
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit3) => {
                        self.mode = Some(1);
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit0) => {
                        self.bloom = !self.bloom;
                        self.redraw();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}

impl App {
    fn redraw(&mut self) {
        let (Some(core), Some(surface), Some(config), Some(window)) = (
            self.core.as_mut(),
            self.surface.as_ref(),
            self.config.as_ref(),
            self.window.as_ref(),
        ) else {
            return;
        };
        let size = window.inner_size();
        let (w, h) = (size.width.max(1), size.height.max(1));
        let mut post = Vec::new();
        if let Some(mode) = self.mode {
            post.push(PostEffect::tonemap(mode).expect("tonemap"));
        }
        if self.bloom {
            post.push(PostEffect::bloom(0.8).expect("bloom"));
        }
        let frame = frame(&self.commands, post, w, h);
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                core.render_view_format(&frame, &view, w, h, config.format);
                core.queue.present(tex);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                surface.configure(&core.device, config);
            }
            other => eprintln!("skip {other:?}"),
        }
        let curve = match self.mode {
            None => "off",
            Some(0) => "reinhard",
            _ => "aces",
        };
        let bloom = if self.bloom { "+bloom" } else { "" };
        window.set_title(
            format!("E507 tonemap {curve}{bloom} · 1=off 2=reinhard 3=aces 0=bloom · Esc exit")
                .as_str(),
        );
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
            el.set_control_flow(ControlFlow::Wait);
            let mut app = App {
                window: None,
                core: None,
                surface: None,
                config: None,
                commands: Vec::new(),
                mode: None,
                bloom: false,
                ready: false,
            };
            el.run_app(&mut app)?;
        }
    }
    Ok(())
}
