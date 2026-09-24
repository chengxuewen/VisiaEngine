//! E206 · 数据装载·矢量瓦片 —— io-tiles(MVT) → GeoTile → 渲染管线（tile-streaming 带 B3+多瓦片）。
//! 3×3 邻域瓦片（FileSource 装载捆绑合成 fixture=零许可面），各瓦片独立 D7 origin 锚定上传
//! （瓦片缝=边框描线可视化），滚轮=正交 zoom（half-width 语义），拖拽=球面轨道。
//! 用法: cargo run --example E206_tile_viewer -- [--frames N]
//!   --frames N   headless 自断言快退（ctest/CI 路）：3×3 解码计数 + 像素四族 + 缩略图
//!   无参         常驻人验窗：拖=轨道 滚轮=缩放（正交 zoom）关窗/Esc 退出

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
const W: u32 = 480;
const H: u32 = 360;

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

/// 3×3 邻域装载（z10，x=0..2 y=0..2——捆绑 fixture 目录树）。
fn load_neighborhood(root: &str) -> Result<Vec<GeoTile>, String> {
    let mut out = Vec::new();
    for dx in 0..3u32 {
        for dy in 0..3u32 {
            let id = TileId::new(10, dx, dy).ok_or("tile id")?;
            out.push(load_tile(root, id).map_err(|e| format!("tile {dx},{dy}: {e}"))?);
        }
    }
    Ok(out)
}

/// 场景中心（3×3 邻域的 3857 中点=中心瓦片 (2,2) 的中心）。
fn scene_center(tiles: &[GeoTile]) -> [f64; 2] {
    let c = &tiles[4];
    let (min_x, min_y, max_x, max_y) = c.id.bbox();
    [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0]
}

/// 场景装配：全部瓦片几何合并为一次上传（每瓦片顶点按各自 origin local 化=同一
/// 3857 世界帧；D7 锚使不同瓦片的 local 顶点可直接并批——origin 只进 Draw 命令，
/// 而命令统一以 [0,0,0] 锚渲染=顶点已自带世界位）。
struct SceneBatch {
    wpos: Vec<[f32; 3]>,
    widx: Vec<u32>,
    normals: Vec<[f32; 3]>,
    strokes: Vec<StrokeSeg>,
    points: Vec<PointMark>,
}

fn build_batch(tiles: &[GeoTile]) -> SceneBatch {
    let mut b = SceneBatch {
        wpos: Vec::new(),
        widx: Vec::new(),
        normals: Vec::new(),
        strokes: Vec::new(),
        points: Vec::new(),
    };
    for gt in tiles {
        // world = own_origin + local；Draw origin=[0,0,0] ⇒ 顶点直接烘世界位是
        // f32 精度雷（D7 违）——正确形=顶点存 world−batch_anchor，batch_anchor
        // =场景中心（小值域），Draw origin=center。
        for f in &gt.features {
            let class = f.attrs.get("class").cloned().unwrap_or_default();
            match &f.geom {
                TileGeom::Poly(ring) => {
                    let start = b.wpos.len() as u32;
                    for p in ring {
                        b.wpos.push([(p[0]) as f32, (p[1]) as f32, 0.0]);
                    }
                    for i in 1..ring.len().saturating_sub(1) as u32 {
                        b.widx.extend([start, start + i, start + i + 1]);
                    }
                }
                TileGeom::Line(pts) => {
                    let (color, width) = if class == "boundary" {
                        ([0.30, 0.55, 0.55], 3.0)
                    } else {
                        ([0.95, 0.62, 0.18], 9.0)
                    };
                    for w in pts.windows(2) {
                        b.strokes.push(StrokeSeg::new(
                            [w[0][0] as f32, w[0][1] as f32, 0.0],
                            [w[1][0] as f32, w[1][1] as f32, 0.0],
                            color,
                            width,
                        ));
                    }
                }
                TileGeom::Point(p) => {
                    b.points.push(PointMark::new(
                        [p[0] as f32, p[1] as f32, 0.0],
                        [0.9, 0.25, 0.35],
                        9.0,
                    ));
                }
                TileGeom::MultiPoint(_) => {}
            }
        }
    }
    b.normals = vec![[0.0f32, 0.0, 1.0]; b.wpos.len()];
    b
}

/// headless 自断言：3×3 解码计数 + 像素四族（水蓝/路橙/点红/缝青）+ 缩略图。
fn prove(tiles: &[GeoTile], frames: u32) {
    let Some(mut b) = HeadlessBackend::new(W, H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };
    let batch = build_batch(tiles);
    assert_eq!(tiles.len(), 9, "3x3 neighborhood");
    assert!(!batch.widx.is_empty() && !batch.strokes.is_empty() && !batch.points.is_empty());
    assert!(
        batch.points.len() >= 9,
        "9 center-poi expected, got {}",
        batch.points.len()
    );

    let mesh = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &batch.wpos,
            normals: &batch.normals,
            indices: &batch.widx,
        })
        .expect("meshes");
    let mat = b
        .create_material([0.16, 0.38, 0.62, 1.0])
        .expect("material");
    let st = b
        .create_strokes(&StrokeTableDesc {
            data: &batch.strokes,
        })
        .expect("strokes");
    let pt = b
        .create_points(&PointTableDesc {
            data: &batch.points,
        })
        .expect("points");

    let center = scene_center(tiles);
    let tile_w = (tiles[0].id.bbox().2 - tiles[0].id.bbox().0) as f32;
    let rig = CameraRig::look_at(
        [center[0], center[1], f64::from(tile_w) * 3.2],
        [center[0], center[1], 0.0],
        [0.0, 1.0, 0.0],
    );
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(
            tile_w * 1.6,
            tile_w * 1.6 * H as f32 / W as f32,
            1.0,
            1_000_000.0,
        ),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .ortho_frame(tile_w * 1.6, W as f32, H as f32, 1.0, 1_000_000.0)
            .expect("ortho"),
        px_world_scale: 2.0 * tile_w * 1.6 / W as f32,
        shadow: None,
        clip: None,
        commands: vec![
            DrawCommand::ClearColor { rgba: CLEAR },
            DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin: [0.0; 3],
                transform: ident(),
            },
            DrawCommand::DrawStrokes {
                table: st,
                origin: [0.0; 3],
                transform: ident(),
            },
            DrawCommand::DrawPoints {
                table: pt,
                origin: [0.0; 3],
                transform: ident(),
            },
        ],
    };
    let _ = frames;
    let img = b.render_to_pixels(&frame).expect("render");

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
    let teal = count(&|p| p[1] > 60 && p[2] > 60 && p[0] < 110 && p[1] >= p[0]);
    println!("PIXELS blue={blue} orange={orange} red={red} teal(seams)={teal}");
    assert!(blue > 2000, "water coverage");
    assert!(orange > 400, "roads");
    assert!(red >= 6, "9 poi points expected visible");
    assert!(teal > 1500, "tile seam outlines (multi-tile proof)");

    gallery::save_frame(&img, "E206_tile_viewer");
    println!("OK tile viewer（{} 瓦片解码渲染自证）", tiles.len());
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

    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/data/tiles");
    let tiles = load_neighborhood(root)?;
    println!("loaded {} tiles", tiles.len());

    if frames.is_some() {
        prove(&tiles, frames.unwrap_or(1));
        return Ok(());
    }

    // 常驻人验窗（C15 双模：零参=窗）：3×3 瓦片 + 轨道/正交缩放/Esc。
    run_window(tiles)
}

/// 常驻人验窗路（无参）：球面轨道拖拽 / 滚轮=正交 zoom（half-width）/ Esc/关窗退出。
struct TileApp {
    core: Option<MeshCore>,
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
    window: Option<Arc<Window>>,
    dragging: Option<(f64, f64)>,
    /// 正交 half-width（滚轮消费面=真缩放语义）。
    zoom: f64,
    /// 球面轨道角（拖拽消费面）。
    yaw: f64,
    pitch: f64,
    gt_center: [f64; 2],
    tile_w: f32,
    mesh: Option<visiaengine_render::MeshId>,
    mat: Option<visiaengine_render::MaterialId>,
    strokes: Option<visiaengine_render::TableId>,
    points: Option<visiaengine_render::TableId>,
    /// resumed 期一次性消费（上传后保留供 prove 语义；窗面不复用）。
    tiles: Option<Vec<GeoTile>>,
}

impl TileApp {
    fn upload(&mut self, tiles: &[GeoTile]) {
        let Some(core) = self.core.as_mut() else {
            return;
        };
        let batch = build_batch(tiles);
        self.mesh = Some(
            core.upload_mesh(&MeshDesc {
                uv: &[],
                positions: &batch.wpos,
                normals: &batch.normals,
                indices: &batch.widx,
            })
            .expect("meshes"),
        );
        self.mat = Some(
            core.upload_material([0.16, 0.38, 0.62, 1.0])
                .expect("material"),
        );
        self.strokes = Some(
            core.create_strokes(&StrokeTableDesc {
                data: &batch.strokes,
            })
            .expect("strokes"),
        );
        self.points = Some(
            core.create_points(&PointTableDesc {
                data: &batch.points,
            })
            .expect("points"),
        );
        println!(
            "uploaded 9 tiles: tris={} segs={} pois={}",
            batch.widx.len() / 3,
            batch.strokes.len(),
            batch.points.len()
        );
    }
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
                            "VisiaEngine E206 · 3×3 矢量瓦片（拖=轨道 滚轮=缩放 关窗/Esc 退出）",
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
        if let Some(tiles) = self.tiles.take() {
            self.upload(&tiles);
            self.tiles = Some(tiles);
        }
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
                if let Some((lx, ly)) = self.dragging {
                    let (x, y) = (position.x, position.y);
                    if lx >= 0.0 {
                        self.yaw -= (x - lx) * 0.006;
                        self.pitch = (self.pitch - (y - ly) * 0.006).clamp(0.15, 1.55);
                        if let Some(w) = &self.window {
                            w.request_redraw();
                        }
                    }
                    self.dragging = Some((x, y));
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // 正交缩放=half-width（zoom），滚轮指数式（E204 同款手感）。
                let d = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => -f64::from(y) * 40.0,
                    winit::event::MouseScrollDelta::PixelDelta(p) => -p.y,
                };
                let f = 1.0 - d * 0.0012;
                self.zoom = (self.zoom * f).clamp(8_000.0, 500_000.0);
                if let Some(w) = &self.window {
                    w.request_redraw();
                }
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
                // 球面轨道：yaw/pitch 绕场景中心，dist=2.2×tile_w 固定观察距。
                let c = self.gt_center;
                let dist = f64::from(self.tile_w) * 2.2;
                let eye = [
                    c[0] + dist * self.pitch.cos() * self.yaw.cos(),
                    c[1] + dist * self.pitch.cos() * self.yaw.sin(),
                    dist * self.pitch.sin(),
                ];
                let rig = CameraRig::look_at(eye, [c[0], c[1], 0.0], [0.0, 1.0, 0.0]);
                let near = (dist * 0.01) as f32;
                let far = (dist * 30.0) as f32;
                let hw = self.zoom as f32;
                let aspect = config.height.max(1) as f32 / config.width.max(1) as f32;
                let Some(proj) = rig.ortho_frame(
                    hw,
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
                        origin: [0.0; 3],
                        transform: ident(),
                    },
                    DrawCommand::DrawStrokes {
                        table: st,
                        origin: [0.0; 3],
                        transform: ident(),
                    },
                    DrawCommand::DrawPoints {
                        table: pt,
                        origin: [0.0; 3],
                        transform: ident(),
                    },
                ];
                let frame = Frame {
                    viewport: Viewport::new(config.width, config.height, 1.0),
                    camera: Camera::ortho(hw, hw * aspect, near, far),
                    view_rot: rig.view_rotation(),
                    eye: rig.eye(),
                    proj,
                    px_world_scale: 2.0 * hw / config.width.max(1) as f32,
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

fn run_window(tiles: Vec<GeoTile>) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let center = scene_center(&tiles);
    let tile_w = (tiles[0].id.bbox().2 - tiles[0].id.bbox().0) as f32;
    let mut app = TileApp {
        core: None,
        surface: None,
        config: None,
        window: None,
        dragging: None,
        zoom: f64::from(tile_w) * 1.6,
        yaw: 0.0,
        pitch: std::f64::consts::FRAC_PI_2,
        gt_center: center,
        tile_w,
        mesh: None,
        mat: None,
        strokes: None,
        points: None,
        tiles: Some(tiles),
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
