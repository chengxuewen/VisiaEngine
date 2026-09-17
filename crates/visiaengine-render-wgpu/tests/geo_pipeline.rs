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
        for gp in parts {
            let p = match gp {
                visiaengine_geo::GeoPart::Fill(t) => t,
                // WGPU-11 断言面=fill 命中；扩片族像素证据住 strokes.rs/points.rs
                _ => continue,
            };
            if p.positions.is_empty() {
                continue;
            }
            let mesh = backend
                .create_mesh(&MeshDesc {
                    positions: &p.positions,
                    normals: &vec![[0.0, 0.0, 1.0]; p.positions.len()],
                    indices: &p.indices,
                    uv: &[],
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
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
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
        for gp in parts {
            let p = match gp {
                visiaengine_geo::GeoPart::Fill(t) => t,
                // WGPU-11 断言面=fill 命中；扩片族像素证据住 strokes.rs/points.rs
                _ => continue,
            };
            if p.positions.is_empty() {
                continue;
            }
            let mesh = backend
                .create_mesh(&MeshDesc {
                    positions: &p.positions,
                    normals: &vec![[0.0, 0.0, 1.0]; p.positions.len()],
                    indices: &p.indices,
                    uv: &[],
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
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
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

// spec: WGPU-11
/// E202 窗口路镜像像素门（2026-09-17 灰屏案根修随带，testing.md 视觉例双保险第 1 条）。
/// 谓词三自证：①正例=拟合式机位必出楼；②canary=案发机位（target 世界原点外的 [0,0,0]）
/// 必空帧——断死"画面来自机位正确"而非背景巧合；③阈=实测 23222 的 −48%。
#[test]
fn golden_geo_window_fit() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/park.geojson"
    );
    let doc = load_geojson(path).expect("fixture");
    let [x0, y0, x1, y1] = doc.layer_bbox().expect("bbox");
    let origin = [(x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0];
    let neg = [-origin[0], -origin[1]];
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
    for f in doc.features() {
        for gp in tessellate(&f.kind.shifted(neg), &f.style).unwrap() {
            let visiaengine_geo::GeoPart::Fill(p) = gp else {
                continue;
            };
            if p.positions.is_empty() {
                continue;
            }
            let mesh = backend
                .create_mesh(&MeshDesc {
                    positions: &p.positions,
                    normals: &vec![[0.0, 0.0, 1.0]; p.positions.len()],
                    indices: &p.indices,
                    uv: &[],
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
    let render = |be: &mut HeadlessBackend, rig: &CameraRig| {
        let aspect = W as f32 / H as f32;
        let (near, far) = ((rig.dist * 0.01) as f32, (rig.dist * 30.0) as f32);
        let frame = Frame {
            viewport: Viewport::new(W, H, 1.0),
            camera: Camera::perspective(rig.fov_y as f32, aspect, near, far),
            view_rot: rig.view_rotation(),
            eye: rig.eye(),
            proj: rig
                .perspective(rig.fov_y as f32, aspect, near, far)
                .unwrap(),
            px_world_scale: 2.0 * rig.zoom as f32 / W as f32,
            shadow: None,
            clip: None,
            commands: commands.clone(),
        };
        be.render_to_pixels(&frame).expect("render")
    };
    // canary：案发机位（E202 出生形，target=[0,0,0] 对世界 3857 系数据）必须全空帧。
    // 参考底=空帧左上角像素（自标定：CORE-16 后 clear 字节受硬件舍入 ±1，钉死字面值脆）。
    let rig_broken = CameraRig::orbit([0.0; 3], 0.0, 1.5, 50.0, 60.0, 1.0, 0.1, 1000.0);
    let empty = render(&mut backend, &rig_broken);
    let bg = [empty.rgba[0], empty.rgba[1], empty.rgba[2]];
    let off_bg = |img: &visiaengine_render_wgpu::OffscreenFrame| -> u32 {
        img.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| {
                p[0].abs_diff(bg[0]) > 2 || p[1].abs_diff(bg[1]) > 2 || p[2].abs_diff(bg[2]) > 2
            })
            .count() as u32
    };
    assert_eq!(off_bg(&empty), 0, "案发机位意外出图——谓词没锁住灰屏案");
    // 修复机位：E202 resumed() 装载期拟合式逐字镜像
    let mut rig = rig_broken;
    let (hx, hy) = ((x1 - x0) / 2.0, (y1 - y0) / 2.0);
    let load_aspect = W as f64 / H.max(1) as f64;
    let tan_h = (rig.fov_y * 0.5).tan();
    rig.target = origin;
    rig.dist = (hx / (tan_h * load_aspect)).max(hy / tan_h) * 1.3;
    rig.zoom = rig.dist * tan_h * load_aspect;
    let img = render(&mut backend, &rig);
    let nc = off_bg(&img);
    assert!(nc > 12_000, "窗口拟合路 park 覆盖不足: {nc} px");
}
