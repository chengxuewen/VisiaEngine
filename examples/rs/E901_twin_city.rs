//! E901 · 垂直切片 seed·孪生城 —— park.geojson 底图 × instanced 城 × PCSS 软影 × PNG 人检图。
//! 批 4 全成果面（4ab/4c/4de/4f）一图流；PNG 落盘 target/twin_city.png（T3 人检清单）。
//! 双模（E501 同制）：无参=常驻交互窗（左键拖=轨道 滚轮=远近 关窗退出）；
//! `--frames N`=离屏单帧+断言+PNG（ctest 注册形/smoke-twin-city）。

use std::sync::Arc;

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
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

const W: u32 = 640;
const H: u32 = 480;
const SIDE: usize = 16; // 16²=256 栋（park 实尺 93×31m，城 32×32m 置中）
const ID64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// 两路共用的资源构造脸（MeshId/MaterialId/TableId/InstanceId 皆 u64 位形，直接转发）。
/// 本例内小 trait（orphan 规则合法），第四消费者出现时再议上位 render-wgpu。
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

/// 底图：park.geojson（fills/strokes/markers 全族，geo_viewer 同款转换）→ 命令表 + 层原点。
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
                let h = 1.6 + (i * 7919 % 11) as f32 * 1.15; // 1.6~13m 塔群
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

/// 地面+城资源上传 → 追加命令（geo 层之后；两路同源）。
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

/// 太阳（PCSS 软影）：西南高角，light 正交框随视口比例（同 E501 位形参数）。
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

// ── headless 路（CI 契约原样：单帧 + 亮暗族断言 + PNG 落盘 + OK 行）────────────
fn headless_run() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter（无 GPU 环境如实报错）");
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
        px_world_scale: 0.14, // 480px / ~66m 视高 @ 参考深度
        shadow: Some(shadow_setup(origin, W, H)),
        clip: None,
        commands,
    };
    let img = b.render_to_pixels(&frame).expect("render");

    // ---- PNG 人检落盘 + 行为自检（城+底图+影斑三族）----
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/twin_city.png");
    image::RgbaImage::from_raw(W, H, img.rgba.clone())
        .expect("rgba len")
        .save(path)
        .expect("png save");
    let mut dark = 0u64;
    let mut bright = 0u64; // 影调族=影地+楼暗面（前景长影带）；亮族=向阳顶/侧面
    for p in img.rgba.as_chunks::<4>().0.iter() {
        let (r, g, bl) = (p[0], p[1], p[2]);
        if r > 150 && g > 150 && bl > 150 {
            bright += 1; // 楼体浅色族（v∈[0.72,0.9]×shade）
        }
        let lum = r as u32 + g as u32 + bl as u32;
        if lum < 240 && r > 10 && bl.abs_diff(r) < 14 {
            dark += 1; // 影斑族=ambient×灰蓝地的近中性低亮（天空 clear lum=381 天然排除）
        }
    }
    assert!(bright > 120, "楼体亮色面不足 {bright}");
    assert!(dark > 30, "影斑缺席 {dark}");
    println!(
        "OK twin_city {} buildings, bright={bright} dark={dark} -> {path}",
        n
    );
    println!("T3 人检：打开上述 PNG 核 ①楼宇阴影落在光反侧 ②底图描边/圆点无 z-fight ③半影软边");
    println!(
        "T3 人检（交互窗形）：无参=常驻窗——C 开剖切（影随刀塌）[/] 刀高 ±1m L 标签开关 拖=轨道 滚轮=远近"
    );
}

// ── 交互路（人工检验面：MeshCore 直渲 surface，同场景同阴影——E501 同制）───────
struct App {
    window: Option<Arc<Window>>,
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    rig: CameraRig,
    dragging: Option<(f64, f64)>,
    origin: [f64; 3],
    commands: Vec<DrawCommand>,
    frames_left: Option<u32>,
    /// 交互收口带（D17-h）：C=剖切开关 [/]=刀高 ±1m，L=标签显隐，Esc=退出
    label_cmds: Vec<DrawCommand>,
    show_labels: bool,
    clip_h: Option<f32>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_inner_size(PhysicalSize::new(960u32, 600u32))
                        .with_title("VisiaEngine E901 孪生城 · 左键拖=轨道 滚轮=远近 关窗退出"),
                )
                .expect("create_window"),
        );
        let size = window.inner_size();
        let instance = visiaengine_render_wgpu::create_instance();
        let surface = instance
            .create_surface(Arc::clone(&window))
            .expect("create_surface");
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            }))
            .ok()
        else {
            eprintln!("no adapter — lavapipe/real GPU required");
            event_loop.exit();
            return;
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("visiaengine-e901"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("request_device");
        let mut core = MeshCore::new(device, queue, instance, adapter.clone());
        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface config");
        config.format = caps.formats[0];
        surface.configure(&core.device, &config);

        // 场景上传（与 headless 路同源函数）
        let mut commands = vec![DrawCommand::ClearColor {
            rgba: [0.30, 0.45, 0.70, 1.0],
        }];
        let (geo_cmds, origin) = geo_layer(&mut core);
        commands.extend(geo_cmds);
        ground_city(&mut core, origin, &mut commands);
        // 三塔屋顶标注（表+atlas 构造；基础命令不含标签——redraw 帧按 L 开关拼接）。
        // 字体缺失=整段静默降级（demo 容灾，非错误路径）。
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
        self.surface = Some(surface);
        self.config = Some(config);
        self.window = Some(window);
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => self.handle_key(event_loop, &event),
            WindowEvent::Resized(size) => {
                if let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &mut self.config)
                    && size.width > 0
                    && size.height > 0
                {
                    config.width = size.width;
                    config.height = size.height;
                    surface.configure(&core.device, config);
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
                self.rig.dist = (self.rig.dist * (1.0 - d * 0.0012)).clamp(20.0, 180.0);
                self.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, &self.surface, &self.config)
                else {
                    return;
                };
                let aspect = config.width as f32 / config.height.max(1) as f32;
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
                    // 世界面 z≤h 保留（n=(0,0,-1), d=h）：楼体逐层削显+影随刀塌
                    ClipSetup::new(&[[0.0, 0.0, -1.0, f64::from(h)]])
                });
                let frame = Frame {
                    viewport: Viewport::new(config.width, config.height, 1.0),
                    camera: Camera::perspective(self.rig.fov_y as f32, aspect, 1.0, 200.0),
                    view_rot: self.rig.view_rotation(),
                    eye: self.rig.eye(),
                    proj,
                    px_world_scale: 0.14, // 参考深度不变（E501 同制：窗 resize 不重标线宽）
                    shadow: Some(shadow_setup(self.origin, config.width, config.height)),
                    clip,
                    commands: cmds,
                };
                match surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(tex)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => {
                        let view = tex
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        core.render_view_format(
                            &frame,
                            &view,
                            config.width,
                            config.height.max(1),
                            config.format,
                        );
                        core.queue.present(tex);
                    }
                    wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                        surface.configure(&core.device, config);
                    }
                    other => eprintln!("skipped frame: {other:?}"),
                }
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
        if let Some(w) = &self.window {
            w.set_title(&format!(
                "E901 孪生城 · C={} [/]={:.1}m · L={} · 拖=轨道 滚轮=远近 Esc=退出",
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
        if let Some(w) = &self.window {
            w.request_redraw();
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
        headless_run(); // CI 路：单帧断言即退（N 在 smoke 语境=单帧契约）
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
        window: None,
        core: None,
        surface: None,
        config: None,
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
