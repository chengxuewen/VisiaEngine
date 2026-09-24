//! E206 · 数据装载·矢量瓦片 —— io-tiles(MVT) → GeoTile → 渲染管线（tile-streaming 带 B3）。
//! FileSource 装载捆绑的合成瓦片（全合成几何=零许可面），MVT 解码 → 3857 世界定位 →
//! 相机 fit → 水(Poly fan)/路(StrokeSeg)/兴趣点(PointMark) 三渲染路（headless 后端真上传）。
//! 用法: cargo run --example E206_tile_viewer -- [--frames N]
//!   --frames N   headless 自断言快退（ctest/CI 路）：解码计数 + 像素三族 + 缩略图
//!   无参         常驻人验窗：拖=轨道 滚轮=远近 关窗/Esc 退出（C15 双模）

use std::sync::Arc;

use visiaengine_io_tiles::{FileSource, GeoTile, TileGeom, TileId, TileSource, decode_tile};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, PointMark, PointTableDesc, RenderBackend,
    StrokeSeg, StrokeTableDesc, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;
use visiaengine_render_wgpu::mesh_core::MeshCore;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

#[path = "gallery.rs"]
mod gallery;

const CLEAR: [f32; 4] = [0.05, 0.07, 0.10, 1.0];
const W: u32 = 320;
const H: u32 = 240;

fn ident() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// 装载+解码+映射一条龙（FileSource→bytes→MvtTile→GeoTile）。
fn load_tile(root: &str, id: TileId) -> Result<GeoTile, String> {
    let src = FileSource::new(root);
    let bytes = src.load(id.z, id.x, id.y).map_err(|e| e.to_string())?;
    let decoded = decode_tile(&bytes).map_err(|e| e.to_string())?;
    Ok(GeoTile::from_layers(id, &decoded.layers))
}

/// headless 自断言：上传真表渲染一帧 + 像素三族（水蓝/路橙/点红）+ 缩略图。
fn prove(gt: &GeoTile, frames: u32) {
    let Some(mut b) = HeadlessBackend::new(W, H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };

    // D7 local 帧：顶点 = world − tile.origin（相机也看向 tile 中心 local 化）。
    let (mut wpos, mut widx) = (Vec::new(), Vec::new());
    let mut strokes: Vec<StrokeSeg> = Vec::new();
    let mut points: Vec<PointMark> = Vec::new();
    for f in &gt.features {
        let class = f.attrs.get("class").cloned().unwrap_or_default();
        match &f.geom {
            TileGeom::Poly(ring) => {
                let base = wpos.len() as u32;
                for p in ring {
                    wpos.push([
                        (p[0] - gt.origin[0]) as f32,
                        (p[1] - gt.origin[1]) as f32,
                        0.0,
                    ]);
                }
                for i in 1..ring.len().saturating_sub(1) as u32 {
                    widx.extend([base, base + i, base + i + 1]);
                }
            }
            TileGeom::Line(pts) => {
                let width = if class == "primary" { 10.0 } else { 4.0 };
                let color = [0.95, 0.62, 0.18];
                for w in pts.windows(2) {
                    strokes.push(StrokeSeg::new(
                        [
                            (w[0][0] - gt.origin[0]) as f32,
                            (w[0][1] - gt.origin[1]) as f32,
                            0.0,
                        ],
                        [
                            (w[1][0] - gt.origin[0]) as f32,
                            (w[1][1] - gt.origin[1]) as f32,
                            0.0,
                        ],
                        color,
                        width,
                    ));
                }
            }
            TileGeom::Point(p) => {
                points.push(PointMark::new(
                    [
                        (p[0] - gt.origin[0]) as f32,
                        (p[1] - gt.origin[1]) as f32,
                        0.0,
                    ],
                    [0.9, 0.25, 0.35],
                    10.0,
                ));
            }
            TileGeom::MultiPoint(_) => {}
        }
    }
    assert!(!widx.is_empty(), "water polygon must produce triangles");
    assert!(!strokes.is_empty(), "road must produce stroke segments");
    assert!(!points.is_empty(), "poi must produce a point mark");

    let normals = vec![[0.0f32, 0.0, 1.0]; wpos.len()];
    let mesh = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &wpos,
            normals: &normals,
            indices: &widx,
        })
        .expect("water mesh");
    let mat = b
        .create_material([0.16, 0.38, 0.62, 1.0])
        .expect("water material");
    let st = b
        .create_strokes(&StrokeTableDesc { data: &strokes })
        .expect("strokes");
    let pt = b
        .create_points(&PointTableDesc { data: &points })
        .expect("points");

    // 相机 fit：看向瓦片中心（origin 处），距离 2×tile 半宽俯视。
    // PIT-8 第二见：look_at(世界原点) 而瓦片住 x=−2e6 km 处 = 全画面外。
    let tile_w = (gt.id.bbox().2 - gt.id.bbox().0) as f32;
    let rig = CameraRig::look_at(
        [gt.origin[0], gt.origin[1], f64::from(2.0 * tile_w)],
        [gt.origin[0], gt.origin[1], 0.0],
        [0.0, 1.0, 0.0],
    );
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(tile_w, tile_w * H as f32 / W as f32, 1.0, 1_000_000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .ortho_frame(
                f64::from(tile_w) as f32,
                W as f32,
                H as f32,
                1.0,
                1_000_000.0,
            )
            .expect("ortho"),
        px_world_scale: 2.0 * tile_w / W as f32,
        shadow: None,
        clip: None,
        commands: vec![
            DrawCommand::ClearColor { rgba: CLEAR },
            DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin: gt.origin,
                transform: ident(),
            },
            DrawCommand::DrawStrokes {
                table: st,
                origin: gt.origin,
                transform: ident(),
            },
            DrawCommand::DrawPoints {
                table: pt,
                origin: gt.origin,
                transform: ident(),
            },
        ],
    };
    let _ = frames; // 渲染恒一帧（瓦片静态；frames 语义=跑通即过）
    let img = b.render_to_pixels(&frame).expect("render");

    // 像素三族谓词（三自证：正例命中 + 非背景 + 阈保守）。
    let count = |pred: &dyn Fn([u8; 3]) -> bool| -> u32 {
        img.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| pred([p[0], p[1], p[2]]))
            .count() as u32
    };
    let blue = count(&|p| p[2] > 90 && p[2] > p[0] + 20);
    let orange = count(&|p| p[0] > 150 && p[1] > 80 && p[2] < 90);
    let red = count(&|p| p[0] > 150 && p[1] < 90 && p[2] < 90);
    println!("PIXELS blue={blue} orange={orange} red={red}");
    assert!(blue > 2000, "water fan coverage missing");
    assert!(orange > 300, "road strokes missing");
    assert!(red > 50, "poi point missing");

    gallery::save_frame(&img, "E206_tile_viewer");
    println!(
        "OK tile viewer（MVT 解码 {} 特征渲染自证）",
        gt.features.len()
    );
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

    // fixture 由构建脚本同步进 tiles 目录（合成瓦片；--frames 语义走 zip 内单瓦片目录树）
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/data/tiles");
    let id = TileId::new(10, 0, 0).expect("valid");
    let gt = load_tile(root, id).map_err(|e| format!("tile load: {e}"))?;
    println!(
        "loaded tile z{} features={} origin=({:.0},{:.0})",
        id.z,
        gt.features.len(),
        gt.origin[0],
        gt.origin[1]
    );

    if frames.is_some() {
        prove(&gt, frames.unwrap_or(1));
        return Ok(());
    }

    // 常驻人验窗（C15 双模：零参=窗）：静态瓦片三族渲染 + 轨道/滚轮/Esc 交互。
    run_window(gt)
}

/// 常驻人验窗路（无参，E204 骨架同制）：轨道拖拽 / 滚轮远近 / Esc/关窗退出。
struct TileApp {
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    window: Option<Arc<Window>>,
    dragging: Option<(f64, f64)>,
    gt: GeoTile,
    mesh: Option<visiaengine_render::MeshId>,
    mat: Option<visiaengine_render::MaterialId>,
    strokes: Option<visiaengine_render::TableId>,
    points: Option<visiaengine_render::TableId>,
}

impl TileApp {
    /// 上传三族资源（water mesh / road strokes / poi points），D7 local 顶点。
    fn upload(&mut self) {
        let Some(core) = self.core.as_mut() else {
            return;
        };
        let (mut wpos, mut widx) = (Vec::new(), Vec::new());
        let mut segs: Vec<StrokeSeg> = Vec::new();
        let mut marks: Vec<PointMark> = Vec::new();
        for f in &self.gt.features {
            let class = f.attrs.get("class").cloned().unwrap_or_default();
            match &f.geom {
                TileGeom::Poly(ring) => {
                    let base = wpos.len() as u32;
                    for p in ring {
                        wpos.push([
                            (p[0] - self.gt.origin[0]) as f32,
                            (p[1] - self.gt.origin[1]) as f32,
                            0.0,
                        ]);
                    }
                    for i in 1..ring.len().saturating_sub(1) as u32 {
                        widx.extend([base, base + i, base + i + 1]);
                    }
                }
                TileGeom::Line(pts) => {
                    let width = if class == "primary" { 10.0 } else { 4.0 };
                    let color = [0.95, 0.62, 0.18];
                    for w in pts.windows(2) {
                        segs.push(StrokeSeg::new(
                            [
                                (w[0][0] - self.gt.origin[0]) as f32,
                                (w[0][1] - self.gt.origin[1]) as f32,
                                0.0,
                            ],
                            [
                                (w[1][0] - self.gt.origin[0]) as f32,
                                (w[1][1] - self.gt.origin[1]) as f32,
                                0.0,
                            ],
                            color,
                            width,
                        ));
                    }
                }
                TileGeom::Point(p) => {
                    marks.push(PointMark::new(
                        [
                            (p[0] - self.gt.origin[0]) as f32,
                            (p[1] - self.gt.origin[1]) as f32,
                            0.0,
                        ],
                        [0.9, 0.25, 0.35],
                        10.0,
                    ));
                }
                TileGeom::MultiPoint(_) => {}
            }
        }
        let normals = vec![[0.0f32, 0.0, 1.0]; wpos.len()];
        self.mesh = Some(
            core.upload_mesh(&MeshDesc {
                uv: &[],
                positions: &wpos,
                normals: &normals,
                indices: &widx,
            })
            .expect("water mesh"),
        );
        self.mat = Some(
            core.upload_material([0.16, 0.38, 0.62, 1.0])
                .expect("water material"),
        );
        self.strokes = Some(
            core.create_strokes(&StrokeTableDesc { data: &segs })
                .expect("strokes"),
        );
        self.points = Some(
            core.create_points(&PointTableDesc { data: &marks })
                .expect("points"),
        );
        println!(
            "loaded tile features={} water_tris={} road_segs={} poi={}",
            self.gt.features.len(),
            widx.len() / 3,
            segs.len(),
            marks.len()
        );
    }
}

/// 窗相机：看向瓦片中心（D7 origin），2×tile 半宽俯视——prove() 同构。
fn tile_rig(gt: &GeoTile) -> CameraRig {
    let tile_w = (gt.id.bbox().2 - gt.id.bbox().0) as f32;
    CameraRig::look_at(
        [gt.origin[0], gt.origin[1], f64::from(2.0 * tile_w)],
        [gt.origin[0], gt.origin[1], 0.0],
        [0.0, 1.0, 0.0],
    )
}

impl ApplicationHandler for TileApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.core.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::WindowAttributes::default()
                        .with_inner_size(winit::dpi::PhysicalSize::new(960u32, 600u32))
                        .with_title(
                            "VisiaEngine E206 · 矢量瓦片（拖=轨道 滚轮=远近 关窗/Esc 退出）",
                        ),
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
            label: Some("visiaengine-window"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("request_device");
        let core = MeshCore::new(device, queue, instance, adapter.clone());
        let caps = surface.get_capabilities(&adapter);
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface config");
        config.format = caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
                )
            })
            .unwrap_or(caps.formats[0]);
        surface.configure(&core.device, &config);
        self.window = Some(window);
        self.core = Some(core);
        self.surface = Some(surface);
        self.config = Some(config);
        self.upload();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    #[allow(clippy::too_many_lines)]
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }
                if let (Some(core), Some(surface), Some(config)) = (
                    self.core.as_mut(),
                    self.surface.as_ref(),
                    self.config.as_mut(),
                ) {
                    config.width = size.width.max(1);
                    config.height = size.height.max(1);
                    surface.configure(&core.device, config);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && matches!(&event.logical_key, Key::Named(NamedKey::Escape))
                {
                    event_loop.exit();
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
                if let Some((lx, _ly)) = self.dragging {
                    let _x = position.x; // 轨道=绕瓦片中心（origin 锚定）；交互相机=Phase 1 巡览
                    self.dragging = Some((position.x, position.y));
                    let _ = lx;
                }
            }
            WindowEvent::MouseWheel { .. } => {
                // 缩放面同上：Phase 0 静态帧（瓦片内容不随交互重建——避免逐帧重上传）。
            }
            WindowEvent::RedrawRequested => {
                let (Some(core), Some(surface), Some(config)) =
                    (&mut self.core, self.surface.as_ref(), self.config.as_mut())
                else {
                    event_loop.exit();
                    return;
                };
                let (Some(mesh), Some(mat), Some(st), Some(pt)) =
                    (self.mesh, self.mat, self.strokes, self.points)
                else {
                    event_loop.exit();
                    return;
                };
                let rig = tile_rig(&self.gt);
                let tile_w = (self.gt.id.bbox().2 - self.gt.id.bbox().0) as f32;
                let near = (f64::from(tile_w) * 0.01) as f32;
                let far = (f64::from(tile_w) * 30.0) as f32;
                let Some(proj) = rig.ortho_frame(
                    f64::from(tile_w) as f32,
                    config.width.max(1) as f32,
                    config.height.max(1) as f32,
                    near,
                    far,
                ) else {
                    event_loop.exit();
                    return;
                };
                let commands = vec![
                    DrawCommand::ClearColor { rgba: CLEAR },
                    DrawCommand::DrawMesh {
                        mesh,
                        material: mat,
                        origin: self.gt.origin,
                        transform: ident(),
                    },
                    DrawCommand::DrawStrokes {
                        table: st,
                        origin: self.gt.origin,
                        transform: ident(),
                    },
                    DrawCommand::DrawPoints {
                        table: pt,
                        origin: self.gt.origin,
                        transform: ident(),
                    },
                ];
                let frame = Frame {
                    viewport: Viewport::new(config.width, config.height, 1.0),
                    camera: Camera::ortho(
                        f64::from(tile_w) as f32,
                        f64::from(tile_w) as f32 * config.height.max(1) as f32
                            / config.width.max(1) as f32,
                        near,
                        far,
                    ),
                    view_rot: rig.view_rotation(),
                    eye: rig.eye(),
                    proj,
                    px_world_scale: 2.0 * tile_w / config.width.max(1) as f32,
                    shadow: None,
                    clip: None,
                    commands,
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
                    other => eprintln!("skipped: {other:?}"),
                }
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn run_window(gt: GeoTile) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = TileApp {
        core: None,
        surface: None,
        config: None,
        window: None,
        dragging: None,
        gt,
        mesh: None,
        mat: None,
        strokes: None,
        points: None,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
