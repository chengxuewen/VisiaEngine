//! E901 · 垂直切片 seed·孪生城 —— park.geojson 底图 × instanced 城 × PCSS 软影 × PNG 人检图。
//! 批 4 全成果面（4ab/4c/4de/4f）一图流；PNG 落盘 target/twin_city.png（T3 人检清单）。
//! 用法：`cargo run -p visiaengine-render-wgpu --example E901_twin_city`（pixi: smoke-twin-city）

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MaterialDesc, MeshDesc,
    RenderBackend, ShadowSetup, Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};

const W: u32 = 640;
const H: u32 = 480;
const SIDE: usize = 16; // 16²=256 栋（park 实尺 93×31m，城 32×32m 置中）
const ID64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn main() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter（无 GPU 环境如实报错）");
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.30, 0.45, 0.70, 1.0],
    }];

    // ---- 底图：park.geojson（fills/strokes/markers 全族，geo_viewer 同款转换）----
    let doc = visiaengine_geo::load_geojson(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/park.geojson"
    ))
    .expect("park.geojson");
    let [x0, y0, x1, y1] = doc.layer_bbox().expect("bbox");
    let geo_origin = [(x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0];
    let neg = [-geo_origin[0], -geo_origin[1]];
    let mut geo_cmds = Vec::new();
    for f in doc.features() {
        for gp in visiaengine_geo::tessellate(&f.kind.shifted(neg), &f.style).expect("tess") {
            match gp {
                visiaengine_geo::GeoPart::Fill(p) => {
                    if p.positions.is_empty() || p.indices.len() < 3 {
                        continue;
                    }
                    let mesh = b
                        .create_mesh(&MeshDesc {
                            positions: &p.positions,
                            normals: &vec![[0.0, 0.0, 1.0]; p.positions.len()],
                            indices: &p.indices,
                            uv: &[],
                        })
                        .expect("geo mesh");
                    let mat = b.create_material(p.color).expect("geo mat");
                    geo_cmds.push(DrawCommand::DrawMesh {
                        mesh,
                        material: mat,
                        origin: geo_origin,
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
                    let table = b
                        .create_strokes(&visiaengine_render::StrokeTableDesc { data: &segs })
                        .expect("strokes");
                    geo_cmds.push(DrawCommand::DrawStrokes {
                        table,
                        origin: geo_origin,
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
                    let table = b
                        .create_points(&visiaengine_render::PointTableDesc { data: &marks })
                        .expect("points");
                    geo_cmds.push(DrawCommand::DrawPoints {
                        table,
                        origin: geo_origin,
                        transform: ID64,
                    });
                }
            }
        }
    }
    commands.extend(geo_cmds);

    // ---- 地面：60×60m 灰板 z=-0.05（geo fill z=0 盖于其上；城基 -0.06 穿板站立）----
    let ground = b
        .create_mesh(&MeshDesc {
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
    let gdmat = b
        .create_material([0.55, 0.55, 0.58, 1.0])
        .expect("ground mat");
    commands.push(DrawCommand::DrawMesh {
        mesh: ground,
        material: gdmat,
        origin: geo_origin,
        transform: ID64,
    });

    // ---- 城：SIDE² instanced 楼块（局部坐标，geo_origin 共用层原点）----
    let (pos, nrm, idx) = unit_box_mesh();
    let boxy = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .expect("box");
    let bmat = b
        .create_material_desc(&MaterialDesc {
            base_color: [1.0, 1.0, 1.0, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        })
        .expect("box mat");
    let step = 2.0f32;
    let span = step * SIDE as f32 / 2.0;
    let city: Vec<Instance> = (0..SIDE)
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
        .collect();
    let iid = b
        .create_instances(&InstanceDesc { data: &city })
        .expect("city table");
    commands.push(DrawCommand::DrawInstances {
        mesh: boxy,
        material: bmat,
        instances: iid,
        origin: geo_origin,
        transform: ID64,
    });

    // ---- 太阳（PCSS 软影）+ 主相机（俯角斜视）----
    let ld = [0.35f32, 0.5, 0.79]; // 太阳：西南高角
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let leye = [
        geo_origin[0] + f64::from(ld[0] / nn) * 80.0,
        geo_origin[1] + f64::from(ld[1] / nn) * 80.0,
        f64::from(ld[2] / nn) * 80.0,
    ];
    let lrig = CameraRig::look_at(leye, [geo_origin[0], geo_origin[1], 0.0], [0.0, 1.0, 0.0]);
    let shadow = ShadowSetup {
        proj: lrig
            .ortho_frame(45.0, W as f32, H as f32, 1.0, 160.0)
            .expect("light proj"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5,
        bias: ShadowSetup::DEFAULT_BIAS,
    };
    let (cx, cy) = (geo_origin[0], geo_origin[1]);
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
        shadow: Some(shadow),
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
        city.len()
    );
    println!("T3 人检：打开上述 PNG 核 ①楼宇阴影落在光反侧 ②底图描边/圆点无 z-fight ③半影软边");
}
