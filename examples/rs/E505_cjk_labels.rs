//! E505 · CJK 中文标注 —— io-text 宿主注字体同口（CAPI-21）的中文全链演示。
//! 双模（E205 同制）：无参=常驻窗（Esc/关窗退）；--frames N=离屏断言快退。
//! 字体：resources/data/NotoSansSC-demo.otf（OFL 子集，仅演示字形——加字重跑
//! scripts/gen_cjk_subset.py；全文案见 E505_TEXT 常量）。

#[path = "gallery.rs"]
mod gallery;

use visiaengine_io_text::{FontFace, GLYPH_ATLAS_PX, GlyphCache, layout};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, LabelMark, LabelTableDesc, MaterialDesc, MeshDesc,
    RenderBackend, Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, mesh_core::MeshCore};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

use std::sync::Arc;

const FIX: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../resources/data/NotoSansSC-demo.otf"
);
/// 演示文案（子集字形的覆盖清单=此处的去重字符集）。
const E505_TEXT: [&str; 4] = ["维视引擎", "中文标注演示", "塔A · 主控", "塔B · 数据中心"];
const ID64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

trait Up {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64;
    fn mt(&mut self, d: &MaterialDesc) -> u64;
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64;
}
impl Up for HeadlessBackend {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.create_mesh(d).expect("m")
    }
    fn mt(&mut self, d: &MaterialDesc) -> u64 {
        self.create_material_desc(d).expect("mt")
    }
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64 {
        self.create_labels(d).expect("lb")
    }
}
impl Up for MeshCore {
    fn m(&mut self, d: &MeshDesc<'_>) -> u64 {
        self.upload_mesh(d).expect("m")
    }
    fn mt(&mut self, d: &MaterialDesc) -> u64 {
        self.upload_material_desc(d).expect("mt")
    }
    fn lb(&mut self, d: &LabelTableDesc<'_>) -> u64 {
        self.create_labels(d).expect("lb")
    }
}

fn scene(up: &mut impl Up, face: &FontFace, glyphs: &mut GlyphCache) -> Vec<DrawCommand> {
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    // 地面。
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
    let gt = up.mt(&MaterialDesc {
        base_color: [0.20, 0.28, 0.22, 1.0],
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
        transform: ID64,
    });
    // 两塔。
    for (x, y, color) in [
        (-2.0f32, -1.0f32, [0.25, 0.45, 0.80, 1.0]),
        (2.0, 1.5, [0.30, 0.65, 0.35, 1.0]),
    ] {
        let (x0, y0, x1, y1) = (x - 0.8, y - 0.8, x + 0.8, y + 0.8);
        let verts = [
            [x0, y0, 0.0],
            [x1, y0, 0.0],
            [x1, y1, 0.0],
            [x0, y1, 0.0],
            [x0, y0, 2.4],
            [x1, y0, 2.4],
            [x1, y1, 2.4],
            [x0, y1, 2.4],
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
        let bt = up.mt(&MaterialDesc {
            base_color: color,
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 1.0,
            metallic: 0.0,
        });
        commands.push(DrawCommand::DrawMesh {
            mesh: bm,
            material: bt,
            origin: [0.0; 3],
            transform: ID64,
        });
    }
    // CJK 标签（世界锚，白字）。
    let mut marks: Vec<LabelMark> = Vec::new();
    for (world, text) in [
        ([-2.0, -1.0, 2.9], E505_TEXT[2]),
        ([2.0, 1.5, 2.9], E505_TEXT[3]),
        ([0.0, 0.0, 0.4], E505_TEXT[1]),
        ([-4.5, 4.5, 0.4], E505_TEXT[0]),
    ] {
        let (quads, _pen) = layout(text, face, glyphs, 30.0);
        for q in &quads {
            marks.push(LabelMark::new(
                [world[0] as f32, world[1] as f32, world[2] as f32],
                [1.0, 1.0, 1.0, 1.0],
                [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                [
                    q.size_px[0],
                    q.size_px[1],
                    q.top_left_px[0],
                    q.top_left_px[1],
                ],
            ));
        }
    }
    println!(
        "E505: {} CJK glyphs laid out from {} labels",
        marks.len(),
        E505_TEXT.len()
    );
    let lt = up.lb(&LabelTableDesc { data: &marks });
    commands.push(DrawCommand::DrawLabels {
        table: lt,
        origin: [0.0; 3],
        transform: ID64,
    });
    commands
}

fn cam_frame(cmds: Vec<DrawCommand>, w: u32, h: u32) -> Frame {
    let rig = CameraRig::look_at([0.0, -13.0, 8.0], [0.0, 0.0, 1.2], [0.0, 1.0, 0.0]);
    let aspect = w as f32 / h.max(1) as f32;
    Frame {
        viewport: Viewport::new(w, h, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, aspect, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, aspect, 0.1, 1000.0)
            .expect("p"),
        px_world_scale: 0.12,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds,
    }
}

fn prove() {
    let bytes = std::fs::read(FIX).expect("CJK subset font");
    let face = FontFace::from_bytes(&bytes).expect("face");
    let mut glyphs = GlyphCache::new();
    let mut b = HeadlessBackend::new(480, 320).expect("adapter");
    // 图集先灌（labels 采样图集纹理——E205 同序）。
    let cmds = {
        // layout 已在 scene() 内跑；此处先跑一遍只为了 glyph cache 填充 + 图集上传。
        let _ = layout(E505_TEXT[0], &face, &mut glyphs, 30.0);
        let _ = layout(E505_TEXT[1], &face, &mut glyphs, 30.0);
        let _ = layout(E505_TEXT[2], &face, &mut glyphs, 30.0);
        let _ = layout(E505_TEXT[3], &face, &mut glyphs, 30.0);
        b.set_glyph_atlas(glyphs.pixels(), GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
            .expect("atlas");
        scene(&mut b, &face, &mut glyphs)
    };
    let img = b.render_to_pixels(&cam_frame(cmds, 480, 320)).unwrap();
    // 白字像素门（图集字形采样=白族像素，非背景非地面非塔色）。
    let white: u32 = img
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > 200 && p[1] > 200 && p[2] > 200)
        .count() as u32;
    println!("E505 probe: white_glyph_px={white}");
    assert!(
        white > 150,
        "CJK labels rendered as white glyphs (white={white})"
    );
    gallery::save_frame(&img, "E505_cjk_labels");
    println!("E505 OK: CJK labels visible ({white} white px) — subset font path proven");
}

struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    commands: Vec<DrawCommand>,
    glyphs: Option<visiaengine_io_text::GlyphCache>,
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
                    WindowAttributes::default()
                        .with_inner_size(PhysicalSize::new(960u32, 600u32))
                        .with_title("E505 CJK 标注 — 中文全链（Noto Sans SC 子集）· Esc 退出"),
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
            label: Some("visiaengine-e505"),
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

        let bytes = std::fs::read(FIX).expect("CJK subset font");
        let face = FontFace::from_bytes(&bytes).expect("face");
        let mut glyphs = GlyphCache::new();
        let _ = layout(E505_TEXT[0], &face, &mut glyphs, 30.0);
        let _ = layout(E505_TEXT[1], &face, &mut glyphs, 30.0);
        let _ = layout(E505_TEXT[2], &face, &mut glyphs, 30.0);
        let _ = layout(E505_TEXT[3], &face, &mut glyphs, 30.0);
        core.set_glyph_atlas(glyphs.pixels(), GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
            .expect("atlas");
        self.commands = scene(&mut core, &face, &mut glyphs);
        self.glyphs = Some(glyphs);
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);
        self.ready = true;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == winit::event::ElementState::Pressed
                    && event.physical_key
                        == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) =>
            {
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
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
        Some(_) => prove(), // assert path: single deterministic frame
        None => {
            let el = EventLoop::new()?;
            el.set_control_flow(ControlFlow::Wait);
            let mut app = App {
                window: None,
                core: None,
                surface: None,
                config: None,
                commands: Vec::new(),
                glyphs: None,
                ready: false,
            };
            el.run_app(&mut app)?;
        }
    }
    Ok(())
}
