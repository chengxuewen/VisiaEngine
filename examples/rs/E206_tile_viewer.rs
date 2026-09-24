//! E206 · 数据装载·矢量瓦片 —— io-tiles(MVT) → GeoTile → 渲染管线（tile-streaming 带 B3）。
//! FileSource 装载捆绑的合成瓦片（全合成几何=零许可面），MVT 解码 → 3857 世界定位 →
//! 相机 fit → 水(Poly fan)/路(StrokeSeg)/兴趣点(PointMark) 三渲染路（headless 后端真上传）。
//! 用法: cargo run --example E206_tile_viewer -- [--frames N]
//!   --frames N   headless 自断言快退（ctest/CI 路）：解码计数 + 像素三族 + 缩略图
//!   无参         静态窗桩（Phase 0 交付解码链路；交互巡览=Phase 1，白皮书路线在册）

use visiaengine_io_tiles::{FileSource, GeoTile, TileGeom, TileId, TileSource, decode_tile};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, PointMark, PointTableDesc, RenderBackend,
    StrokeSeg, StrokeTableDesc, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

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

    // Phase 0 窗桩：解码链路验证完即静退（交互巡览=Phase 1，白皮书路线在册）。
    println!("OK tile viewer（Phase 0 静态；交互巡览=Phase 1）");
    Ok(())
}
