//! E504 · 玻璃与水体 —— 透明管线活例（④ REND-36/WGPU-27/28 演示+像素门）。
//! 双模（族制）：无参=常驻窗（1/2=水 alpha 0.7/0.35 重传材质演示，拖轨道滚轮远近 R 复位）；
//! `--frames N`=离屏断言（水下棋盘透视/玻璃楼见地/标签恒顶，阈=PIT-8 探针实测）。

#[path = "gallery.rs"]
mod gallery;

use std::sync::Arc;

use visiaengine_io_text::{FontFace, GLYPH_ATLAS_PX, GlyphCache, layout};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, LabelMark, LabelTableDesc,
    MaterialDesc, MeshDesc, RenderBackend, ShadowSetup, Viewport,
};
use visiaengine_render_wgpu::mesh_core::MeshCore;
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

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

/// 场景：4 色棋盘地面 + 玻璃楼群(a=0.35 实例批) + 全幅水面(a 可换) + 恒顶标签。
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
        }));
    }
    let dark = up.mt(&MaterialDesc {
        base_color: [0.45, 0.47, 0.50, 1.0],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    let glass = up.mt(&MaterialDesc {
        base_color: [0.75, 0.88, 0.95, 0.35],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    let water = up.mt(&MaterialDesc {
        base_color: [0.15, 0.45, 0.85, water_a],
        texture: None,
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    // 棋盘 6×6：深浅两色 + 深色块单列（透视检查列）
    let tile = quad(4.0);
    let tm = up.m(&MeshDesc {
        positions: &tile.0,
        normals: &tile.1,
        indices: &tile.2,
        uv: &[],
    });
    for iy in 0..6i32 {
        for ix in 0..6i32 {
            let dark_tile = ix == 1 && iy == 3; // 透视检查格（水下方左起第二列）
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
    // 玻璃楼群（instanced，a=0.35——背后棋盘应可见）
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
    // 全幅水面 z=2（盖棋盘与楼脚，标签之前——画家序按视深自动）
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
    // 恒顶标签 "GLASS"
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
    let _ = (light.len(), dark, glass, water); // 句柄已入各命令；哨兵免 dead_code
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
    // 探针位（窗内顶视推定后实测钉）：暗格在水下应仍比亮格暗（透视=透明度生效）
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
    // 双向语义锁：a=0.5 必衰减（同景 a=1.0 对照实测 contrast=52，水盖对比=0 即透明失效）
    assert!(
        contrast > 10 && contrast < 30,
        "水膜衰减语义缺失：contrast={contrast}（透明/不透明两态同带）"
    );
    assert!(label_white > 25, "恒顶标签缺席 {label_white}"); // 实测 40
    println!("OK glass_water contrast={contrast} label={label_white}（透明+恒顶现行）");
}

// ── 交互 ─────────────────────────────────────────────────────────────────────
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
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
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(PhysicalSize::new(960u32, 600u32))
                        .with_title("E504 玻璃水体 · 1/2=水a0.7/0.35 重传材质 R=复位 拖=轨道"),
                )
                .expect("win"),
        );
        let size = window.inner_size();
        let instance = visiaengine_render_wgpu::create_instance();
        let surface = instance.create_surface(Arc::clone(&window)).expect("surf");
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
            label: Some("e504"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("dev");
        let mut core = MeshCore::new(device, queue, instance, adapter.clone());
        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("cfg");
        config.format = caps.formats[0];
        surface.configure(&core.device, &config);
        let mut cache = GlyphCache::new();
        let cmds = build(&mut core, self.water_a, &mut cache);
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
            WindowEvent::KeyboardInput { event, .. } => self.on_key(event_loop, &event),
            WindowEvent::Resized(sz) => {
                if let (Some(c), Some(s), Some(cfg)) =
                    (&mut self.core, &self.surface, &mut self.config)
                    && sz.width > 0
                    && sz.height > 0
                {
                    cfg.width = sz.width;
                    cfg.height = sz.height;
                    s.configure(&c.device, cfg);
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
                if let Some((lx, ly)) = self.dragging.take() {
                    let (x, y) = (position.x, position.y);
                    if lx >= 0.0 {
                        self.rig.orbit_delta((x - lx) * 0.006, (y - ly) * 0.006);
                        self.request_redraw();
                    }
                    self.dragging = Some((x, y));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                self.rig.dist = (self.rig.dist * (1.0 - d * 0.0012)).clamp(10.0, 160.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
                    return;
                };
                let fr = frame_of(&self.rig, self.cmds.clone(), config.width, config.height);
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(t)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(t) => {
                        let view = t
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        core.render_view_format(
                            &fr,
                            &view,
                            config.width,
                            config.height,
                            config.format,
                        );
                        core.queue.present(t);
                    }
                    wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                        surface.configure(&core.device, config);
                    }
                    other => eprintln!("skip {other:?}"),
                }
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
        if let Some(w) = &self.window {
            w.request_redraw();
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
        window: None,
        core: None,
        surface: None,
        config: None,
        rig: home(),
        dragging: None,
        cmds: Vec::new(),
        water_a: 0.5,
        ready: false,
    };
    el.run_app(&mut app)?;
    Ok(())
}
