//! E506 Postprocessing — bloom/outline/haze post chain live demo (WGPU-34/36/38, REND-43).
//!
//! Dual-mode (E304 same shape): MeshCore direct + HeadlessBackend assert.
//! - No-arg = resident window: `1` bloom toggle, `2` outline toggle, `3` both,
//!   `4` haze toggle (WGPU-38 screen-space depth haze)
//!   off; title echoes state, Esc closes.
//! - `--frames N` = offscreen assert path: off vs bloom and off vs outline
//!   pixel diffs exceed probe-pinned K; off state deterministic (canary).

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

const W: u32 = 320;
const H: u32 = 240;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Upload seam (E304's Up trait shape): one scene builder serving both
/// HeadlessBackend (assert path) and MeshCore (window path). Ground + two
/// towers; one bright-lit tower face feeds bloom, silhouettes feed outline.
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
        roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
        metallic: 0.0,
    });
    commands.push(DrawCommand::DrawMesh {
        mesh: gm,
        material: gt,
        origin: [0.0; 3],
        transform: T4,
    });
    // Two towers; the first is BRIGHT (bloom fodder: luma above threshold).
    for (x, y, h, color) in [
        (-1.5f32, -1.5f32, 2.0f32, [0.95, 0.90, 0.75, 1.0]),
        (2.0, 1.5, 3.2, [0.30, 0.65, 0.35, 1.0]),
    ] {
        let (x0, y0, x1, y1) = (x - 1.0, y - 1.0, x + 1.0, y + 1.0);
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

    let off_a = b.render_to_pixels(&frame(&cmds, Vec::new(), W, H)).unwrap();
    let off_b = b.render_to_pixels(&frame(&cmds, Vec::new(), W, H)).unwrap();
    assert_eq!(off_a.rgba, off_b.rgba, "off state must be deterministic");

    let bloom = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::bloom(0.8).expect("bloom")],
            W,
            H,
        ))
        .unwrap();
    let outline = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::outline(2.0).expect("outline")],
            W,
            H,
        ))
        .unwrap();

    // Probe (lavapipe, 320×240 city scene): off-vs-bloom=1289 px,
    // off-vs-outline=1509 px. K floors conservative (PIT-8): −50%.
    let haze = b
        .render_to_pixels(&frame(
            &cmds,
            vec![PostEffect::haze(40.0, 200.0).expect("haze")],
            W,
            H,
        ))
        .unwrap();

    // Probe (lavapipe, 320x240 city scene): off-vs-bloom=1289 px,
    // off-vs-outline=1509 px. K floors conservative (PIT-8): -50%.
    // Haze is TWO-SIDED (WGPU-38): far region must shift, NEAR region must
    // stay bitwise identical to off (depth-gating proof, not a global tint).
    let db = diff_px(&off_a, &bloom);
    let dc = diff_px(&off_a, &outline);
    let dh = diff_px(&off_a, &haze);
    println!("E506 probe: bloom diff={db} outline diff={dc} haze diff={dh}");
    assert!(db > 600, "bloom must differ from off: {db}");
    assert!(dc > 700, "outline must differ from off: {dc}");
    assert!(dh > 400, "haze must differ from off in far region: {dh}");
    // WGPU-38 two-sided law, geometry-aware (probe-taught: rows are NOT a
    // depth proxy — background pixels live in every row band). Partition by
    // clear-color: background px (off == clear) MUST take full haze; the
    // scene's near geometry (<40 m band) must stay untouched — on this city
    // frame every geometry pixel is inside far_start, so NO geometry pixel
    // may change at all.
    // Self-calibrated background reference (PIT-45) — same rule as post_chain.rs: read the
    // clear colour out of the frame, and assert it is actually widespread.
    let offpx = off_a.rgba.as_chunks::<4>().0;
    let c0 = &offpx[0];
    let clear = [c0[0], c0[1], c0[2]];
    let clear_hits = offpx.iter().filter(|p| [p[0], p[1], p[2]] == clear).count();
    assert!(
        clear_hits * 50 > offpx.len(),
        "clear reference covers <=2% of the frame -> corner pixel is not background: {clear_hits}/{}",
        offpx.len()
    );
    println!("E506 probe: clear={clear:?} bg_px={clear_hits}");
    let mut bg = (0usize, 0usize); // (hazed, total)
    let mut geo_touched = 0usize;
    for (o, x) in off_a
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(haze.rgba.as_chunks::<4>().0)
    {
        if [o[0], o[1], o[2]] == clear {
            bg.1 += 1;
            if x[..3] != o[..3] {
                bg.0 += 1;
            }
        } else if x[..3] != o[..3] {
            geo_touched += 1;
        }
    }
    eprintln!(
        "E506 haze probe: bg {}/{} hazed; geo touched {geo_touched}",
        bg.0, bg.1
    );
    assert!(
        bg.1 > 1000 && bg.0 == bg.1,
        "background must take full haze"
    );
    assert_eq!(
        geo_touched, 0,
        "all scene geometry is <40 m — near-unchanged law"
    );
    println!(
        "E506 OK: off deterministic; bloom/outline/haze pixel-verified (bg hazed, geo untouched)"
    );
}

// ── resident window path (E304 winit shape) ──────────────────────────────────
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    commands: Vec<DrawCommand>,
    bloom: bool,
    outline: bool,
    haze: bool,
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
                        .with_title("E506 post off · 1=bloom 2=outline 4=haze 3=off · Esc exit"),
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
            label: Some("visiaengine-e506"),
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
                        self.bloom = !self.bloom;
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit2) => {
                        self.outline = !self.outline;
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit4) => {
                        self.haze = !self.haze;
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Digit3) => {
                        self.bloom = false;
                        self.outline = false;
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
        if self.bloom {
            post.push(PostEffect::bloom(0.8).expect("bloom"));
        }
        if self.outline {
            post.push(PostEffect::outline(2.0).expect("outline"));
        }
        if self.haze {
            post.push(PostEffect::haze(40.0, 200.0).expect("haze"));
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
        let state = match (self.bloom, self.outline, self.haze) {
            (false, false, false) => "off".to_string(),
            (true, false, false) => "bloom".to_string(),
            (false, true, false) => "outline".to_string(),
            (true, true, false) => "bloom+outline".to_string(),
            (b, o, true) => {
                let base = match (b, o) {
                    (false, false) => "off",
                    (true, false) => "bloom",
                    (false, true) => "outline",
                    _ => "bloom+outline",
                };
                format!("{base}+haze")
            }
        };
        window.set_title(
            format!("E506 post {state} · 1=bloom 2=outline 4=haze 3=off · Esc exit").as_str(),
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
                bloom: false,
                outline: false,
                haze: false,
                ready: false,
            };
            el.run_app(&mut app)?;
        }
    }
    Ok(())
}
