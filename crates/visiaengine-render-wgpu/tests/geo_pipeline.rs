//! WGPU-11：geo 全链路装配（解析→细分→D7 上传→离屏着色）——host 用法样板。

use visiaengine_geo::{load_geojson, tessellate};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 640;
const H: u32 = 480;

#[must_use]
fn px(f: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]]
}

// spec: WGPU-11
#[test]
fn golden_geo_fill_hit() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/park.geojson"
    );
    let doc = match load_geojson(path) {
        Ok(d) => d,
        Err(e) => panic!("fixture 解析失败: {e}"),
    };
    let [x0, y0, x1, y1] = doc.layer_bbox().expect("layer bbox");
    let origin = [(x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0];
    let zoom = ((x1 - x0) / 2.0).max((y1 - y0) / 2.0) * 1.25;

    let mut backend = match HeadlessBackend::new(W, H) {
        Some(b) => b,
        None => {
            eprintln!("SKIP: no adapter");
            return;
        }
    };
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    // buildingB（无六键色→默认蓝 [0,0.45,1,1]）质心
    let neg = [-origin[0], -origin[1]];
    for f in doc.features() {
        let parts = tessellate(&f.kind.shifted(neg), &f.style).unwrap();
        for p in parts {
            if p.positions.is_empty() {
                continue;
            }
            let mesh = backend
                .create_mesh(&MeshDesc {
                    positions: &p.positions,
                    normals: &vec![[0.0, 0.0, 1.0]; p.positions.len()],
                    indices: &p.indices,
                })
                .unwrap();
            let mat = backend.create_material(p.color).unwrap();
            commands.push(DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin,
                transform: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            });
            let _ = f.name.as_deref();
        }
    }
    let rig = CameraRig::look_at(
        [origin[0], origin[1], 10.0],
        [origin[0], origin[1], 0.0],
        [0.0, 1.0, 0.0],
    );
    let proj = rig
        .ortho_frame(zoom as f32, W as f32, H as f32, 0.1, 1000.0)
        .unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(zoom as f32, zoom as f32 * H as f32 / W as f32, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: [origin[0], origin[1], 10.0],
        proj,
        commands,
    };
    let img = backend.render_to_pixels(&frame).expect("render");

    // buildingB 中心世界坐标（web mercator 值由 fixture 经纬推出）
    let bc = visiaengine_geo::web_mercator(2.35035, 48.8500250).unwrap();
    let nx = (bc[0] - origin[0]) as f32 / zoom as f32;
    let ny = (bc[1] - origin[1]) as f32 / (zoom as f32 * H as f32 / W as f32);
    let pxc = (((nx + 1.0) / 2.0 * W as f32) as i64).clamp(0, (W - 1) as i64) as u32;
    let pyc = (((1.0 - ny) / 2.0 * H as f32) as i64).clamp(0, (H - 1) as i64) as u32;
    let mut blues: Vec<(u32, u32)> = Vec::new();
    for y in (0..H).step_by(2) {
        for x in (0..W).step_by(2) {
            let p = px(&img, x, y);
            // 默认蓝 [0,.45,1] 过方向光 ×0.625 → 期望 ~(0,72,160)
            if p[2] > 120 && p[2] as i16 - p[0] as i16 > 80 && p[1] < 120 {
                blues.push((x, y));
            }
        }
    }
    let bbox = (
        blues.iter().map(|v| v.0).min().unwrap_or(0),
        blues.iter().map(|v| v.0).max().unwrap_or(0),
        blues.iter().map(|v| v.1).min().unwrap_or(0),
        blues.iter().map(|v| v.1).max().unwrap_or(0),
    );
    println!(
        "BLUE bbox {bbox:?} count {} expect {pxc},{pyc}",
        blues.len()
    );
    assert!(!blues.is_empty(), "全图无默认蓝像素（geo 管线未出图）");
}

// spec: GEO-20
#[test]
fn golden_scalar_ramp_pixels() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/heights.geojson"
    );
    let doc = load_geojson(path).unwrap();
    let [x0, y0, x1, y1] = doc.layer_bbox().expect("layer bbox");
    let origin = [(x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0];
    let zoom = ((x1 - x0) / 2.0).max((y1 - y0) / 2.0) * 1.25;
    let Some(mut backend) = HeadlessBackend::new(W, H) else {
        eprintln!("SKIP: no adapter");
        return;
    };
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    let neg = [-origin[0], -origin[1]];
    for f in doc.features() {
        let parts = tessellate(&f.kind.shifted(neg), &f.style).unwrap();
        for p in parts {
            if p.positions.is_empty() {
                continue;
            }
            let mesh = backend
                .create_mesh(&MeshDesc {
                    positions: &p.positions,
                    normals: &vec![[0.0, 0.0, 1.0]; p.positions.len()],
                    indices: &p.indices,
                })
                .unwrap();
            let mat = backend.create_material(p.color).unwrap();
            commands.push(DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin,
                transform: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            });
        }
    }
    let rig = CameraRig::look_at(
        [origin[0], origin[1], 10.0],
        [origin[0], origin[1], 0.0],
        [0.0, 1.0, 0.0],
    );
    let proj = rig
        .ortho_frame(zoom as f32, W as f32, H as f32, 0.1, 1000.0)
        .unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(zoom as f32, zoom as f32 * H as f32 / W as f32, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: [origin[0], origin[1], 10.0],
        proj,
        commands,
    };
    let img = backend.render_to_pixels(&frame).expect("render");
    // 三色族像素分类计数（方向光 ×0.625 经验域，见 golden_geo_fill_hit 注释）
    let (mut greenish, mut reddish, mut midyellow) = (0u32, 0u32, 0u32);
    for y in (0..H).step_by(2) {
        for x in (0..W).step_by(2) {
            let p = px(&img, x, y);
            let (r, g) = (p[0] as i16, p[1] as i16);
            if g > 120 && r < 50 {
                greenish += 1;
            } else if r > 120 && g < 50 {
                reddish += 1;
            } else if r > 50 && g > 50 && (r - g).abs() < 25 && p[2] < 40 {
                midyellow += 1;
            }
        }
    }
    assert!(greenish > 20, "h0 纯低绿色族像素缺失: {greenish}");
    assert!(reddish > 20, "h50 纯高红色族像素缺失: {reddish}");
    assert!(midyellow > 20, "h25 中点黄族像素缺失: {midyellow}");
}
