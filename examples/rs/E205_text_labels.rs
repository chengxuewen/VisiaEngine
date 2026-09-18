//! E205 · 图注文字·标注双模 —— io-text 全链活例（CAPI 同底座：face/cache/layout + 标签管线）。
//! 双模（E901 同制）：无参=常驻窗（Esc/关窗退）；--frames N=离屏断言快退。
//! 注：DejaVu 无 CJK 字形——中文路=宿主注 CJK 字体同口（演示如实拉丁，明账）。

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
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

use std::sync::Arc;

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

/// 双后端上传脸（E901 Ctor 同制小 trait：HeadlessBackend 走 trait 正名、MeshCore 走原生）。
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

/// 场景构造（headless/窗双路同源）：彩板地面 ×3 + 三段标签。
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
        });
        cmds.push(DrawCommand::DrawMesh {
            mesh: m,
            material: mt,
            origin: [0.0; 3],
            transform: ID64,
        });
    }
    // 三段标签（大小/色语境各异）：anchor=中心点 pen/2 左移，dx/dy 原样（add_label 同式）
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
        // center 对齐=dx 侧平移（px 域）；anchor 恒世界位（units 混用陷阱：pen 是 px
        // 不可进 world 坐标——E814/hpp/capi 三处同式互证）
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
        px_world_scale: 2.0 * zoom / w as f32, // REND-29 精确路：标签恒 px
        shadow: None,
        clip: None,
        commands,
    }
}

fn load_face() -> FontFace {
    FontFace::from_bytes(&std::fs::read(FIX).expect("DejaVu fixture")).expect("face")
}

// ── headless 路（--frames N：断言+OK 行）──────────────────────────────────────
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
    assert!(white > 120, "主标签白墨缺席 {white}");
    assert!(warm > 30, "金色副标签缺席 {warm}");
    println!("OK text_labels white={white} warm={warm}（三语境标签 atlas 全链通）");
}

// ── 交互路（无参：常驻窗，Esc/关窗退）─────────────────────────────────────────
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    cmds: Vec<DrawCommand>,
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
                        .with_title(
                            "VisiaEngine E205 图注 · Esc/关窗退出（DejaVu 无 CJK=拉丁演示）",
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
            label: Some("visiaengine-e205"),
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
        let face = load_face();
        let mut cache = GlyphCache::new();
        let cmds = scene(&mut core, &face, &mut cache);
        core.set_glyph_atlas(cache.pixels(), GLYPH_ATLAS_PX, GLYPH_ATLAS_PX)
            .expect("atlas");
        self.cmds = cmds;
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);
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
                if let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &mut self.config)
                    && sz.width > 0
                    && sz.height > 0
                {
                    config.width = sz.width;
                    config.height = sz.height;
                    surface.configure(&core.device, config);
                    self.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
                    return;
                };
                let frame = ortho_frame(self.cmds.clone(), config.width, config.height, 30.0);
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                        let view = t
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        core.render_view_format(
                            &frame,
                            &view,
                            config.width,
                            config.height.max(1),
                            config.format,
                        );
                        core.queue.present(t);
                    }
                    wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                        surface.configure(&core.device, config);
                    }
                    other => eprintln!("skip {other:?}"),
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

fn main() {
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
    if let Some(n) = frames {
        headless_run(n);
        return;
    }
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App {
        window: None,
        core: None,
        surface: None,
        config: None,
        cmds: Vec::new(),
        ready: false,
    };
    event_loop.run_app(&mut app).expect("run");
}
