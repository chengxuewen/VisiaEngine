//! E304 Projection Morph — 2D↔3D continuous projection morphTime (N2.1).
//!
//! Dual-mode (E303 same shape): MeshCore direct + render::morph pure fns —
//! the SAME interpolation Engine::set_morph_time uses (single source of
//! truth: render::morph, REND-41).
//! - No-arg = resident window: `[` / `]` slide t (0=ortho map, 1=persp 3D),
//!   title echoes t, Esc closes.
//! - `--frames N` = offscreen assert path: matrix endpoints exact; mid
//!   frames distinct + non-degenerate.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use visiaengine_render::morph::{morph_proj, morph_px_scale};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, MultiClearPolicy};

const W: u32 = 320;
const H: u32 = 240;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Upload seam (E303's Up trait shape): one scene builder serving both
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

fn scene(up: &mut impl Up) -> Vec<DrawCommand> {
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    // Ground.
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
    // Two towers (8-vert boxes, 10 tris: top + 4 sides).
    for (x, y, h, color) in [
        (-1.5f32, -1.5f32, 2.0f32, [0.25, 0.45, 0.80, 1.0]),
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

/// Frame at morph time t (0 = ortho, 1 = perspective). Both projections from
/// the SAME rig — only the projection matrices lerp (REND-41 law).
fn morph_frame(cmds: &[DrawCommand], t: f64, w: u32, h: u32, hw: f64) -> Frame {
    let rig = CameraRig::look_at([0.0, -14.0, 9.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let persp = rig
        .perspective(rig.fov_y as f32, w as f32 / h as f32, 0.1, 1000.0)
        .expect("persp");
    let ortho = rig
        .ortho_frame(
            hw as f32,
            w as f32,
            h as f32,
            0.1,
            (hw * 2.0).max(1000.0) as f32,
        )
        .expect("ortho");
    let camera = if t <= 0.5 {
        Camera::ortho(
            hw as f32,
            hw as f32 * h as f32 / w as f32,
            0.1,
            (hw * 2.0).max(1000.0) as f32,
        )
    } else {
        Camera::perspective(rig.fov_y as f32, w as f32 / h as f32, 0.1, 1000.0)
    };
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera,
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: morph_proj(&ortho, &persp, t),
        px_world_scale: morph_px_scale(2.0 * hw as f32 / w as f32, t),
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds.to_vec(),
    }
}

// ── headless assert path ─────────────────────────────────────────────────────
fn prove(frames: u32) {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let cmds = scene(&mut b);

    let ortho = b
        .render_to_pixels(&morph_frame(&cmds, 0.0, W, H, 8.0))
        .unwrap();
    let persp = b
        .render_to_pixels(&morph_frame(&cmds, 1.0, W, H, 8.0))
        .unwrap();

    // Matrix-level endpoint law (exact, scene-independent).
    let rig = CameraRig::look_at([0.0, -14.0, 9.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let p = rig.perspective(0.9, 1.0, 0.1, 1000.0).unwrap();
    let o = rig.ortho_frame(8.0, 100.0, 100.0, 0.1, 1000.0).unwrap();
    assert_eq!(morph_proj(&o, &p, 0.0), o, "morph_proj t=0 == ortho exact");
    assert_eq!(morph_proj(&o, &p, 1.0), p, "morph_proj t=1 == persp exact");

    // Mid frames: distinct from both endpoints, non-degenerate.
    let mut distinct = 0;
    for i in 0..frames.min(4) {
        let t = 0.2 + 0.2 * f64::from(i);
        let mid = b
            .render_to_pixels(&morph_frame(&cmds, t, W, H, 8.0))
            .unwrap();
        let lit: u32 = mid
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|px| u32::from(px[0]) + u32::from(px[1]) + u32::from(px[2]) > 40)
            .count() as u32;
        assert!(lit > 500, "mid t={t} renders the scene (lit={lit})");
        assert_ne!(mid.rgba, ortho.rgba, "mid differs from ortho");
        assert_ne!(mid.rgba, persp.rgba, "mid differs from persp");
        distinct += 1;
    }
    println!("E304 OK: matrix endpoints exact; {distinct} mid frames distinct + non-degenerate");
}

// ── resident window path (E303 winit shape) ──────────────────────────────────
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    commands: Vec<DrawCommand>,
    t: f64,
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
                        .with_title("E304 morph t=0.00 · [ / ] slide · Esc exit"),
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
            label: Some("visiaengine-e304"),
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
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::BracketLeft) => {
                        self.t = (self.t - 0.1).max(0.0);
                        self.redraw();
                    }
                    winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::BracketRight) => {
                        self.t = (self.t + 0.1).min(1.0);
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
        let frame = morph_frame(&self.commands, self.t, w, h, 8.0);
        let full = visiaengine_render::ViewportRect::new(0, 0, w, h);
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                let view = tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                core.render_view_rects(
                    &[(frame, full)],
                    &view,
                    w,
                    h,
                    config.format,
                    MultiClearPolicy::FirstClearRestLoad,
                );
                core.queue.present(tex);
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                surface.configure(&core.device, config);
            }
            other => eprintln!("skip {other:?}"),
        }
        window.set_title(format!("E304 morph t={:.2} · [ / ] slide · Esc exit", self.t).as_str());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut frames: Option<u32> = None;
    let mut t_init = 0.0f64;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--frames" {
            frames = args
                .next()
                .and_then(|v| v.parse().ok())
                .filter(|n: &u32| *n > 0);
        } else if a == "--t" {
            // diagnostic: force initial morphTime (0=ortho 1=persp)
            t_init = args.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
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
                t: t_init,
                ready: false,
            };
            el.run_app(&mut app)?;
        }
    }
    Ok(())
}
