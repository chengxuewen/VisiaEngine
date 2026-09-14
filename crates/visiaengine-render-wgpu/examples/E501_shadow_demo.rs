//! E501 · 材质与光影 —— instanced 楼块群 + 方向光 PCSS 软影（批 4 光影成果面）。
//! 白模光影演示兼 ShadowSetup 公共 API 示例门禁消费面。headless 单帧出图+行为打印。
//! 用法：`cargo run --example E501_shadow_demo`（pixi task: smoke-shadow-demo）

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MeshDesc, RenderBackend,
    ShadowBias, ShadowSetup, Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};

const W: u32 = 256;
const H: u32 = 256;
const SIDE: usize = 16; // 16×16=256 楼块
const IDENTITY: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn main() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter（无 GPU 环境如实报错）");
    // 地面
    let ground = b
        .create_mesh(&MeshDesc {
            positions: &[
                [-20.0, -20.0, 0.0],
                [20.0, -20.0, 0.0],
                [20.0, 20.0, 0.0],
                [-20.0, 20.0, 0.0],
            ],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        })
        .expect("ground");
    let gmat = b
        .create_material([0.75, 0.75, 0.78, 1.0])
        .expect("ground mat");
    // 楼块群（instanced，单 draw）
    let (pos, nrm, idx) = unit_box_mesh();
    let boxy = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .expect("box");
    let bmat = b.create_material([1.0, 1.0, 1.0, 1.0]).expect("box mat");
    let city: Vec<Instance> = (0..SIDE)
        .flat_map(|gy| {
            (0..SIDE).map(move |gx| {
                let i = gx + gy * SIDE;
                let h = 1.5 + (i * 7919 % 13) as f32 * 0.6; // 确定性高差
                Instance::new(
                    [
                        (gx as f32 - SIDE as f32 / 2.0) * 2.2,
                        (gy as f32 - SIDE as f32 / 2.0) * 2.2,
                        0.0,
                    ],
                    h,
                    [0.82, 0.86, 0.92],
                )
            })
        })
        .collect();
    let iid = b
        .create_instances(&InstanceDesc { data: &city })
        .expect("city table");

    // 光源相机（REND-31 构造契约：rig 路）：西南高角度太阳
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let eye = [
        f64::from(ld[0] / nn) * 80.0,
        f64::from(ld[1] / nn) * 80.0,
        f64::from(ld[2] / nn) * 80.0,
    ];
    let lrig = CameraRig::look_at(eye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let shadow = ShadowSetup {
        proj: lrig
            .ortho_frame(40.0, W as f32, H as f32, 1.0, 160.0)
            .expect("light proj"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5, // 软半影
        bias: ShadowBias {
            constant: -1.2,
            slope: -1.5,
        },
    };

    let rig = CameraRig::look_at([0.0, -34.0, 26.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let proj = rig
        .perspective(rig.fov_y as f32, 1.0, 0.5, 300.0)
        .expect("proj");
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.5, 300.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 0.12,
        shadow: Some(shadow),
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.35, 0.55, 0.85, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: ground,
                material: gmat,
                origin: [0.0; 3],
                transform: IDENTITY,
            },
            DrawCommand::DrawInstances {
                mesh: boxy,
                material: bmat,
                instances: iid,
                origin: [0.0; 3],
                transform: IDENTITY,
            },
        ],
    };
    let img = b.render_to_pixels(&frame).expect("render");
    let mut dark = 0u32;
    let mut groundish = 0u32;
    for p in img.rgba.as_chunks::<4>().0.iter() {
        let (r, g, bl) = (p[0], p[1], p[2]);
        let lum = r as u32 + g as u32 + bl as u32;
        if r.abs_diff(g) < 18 && r.abs_diff(bl) < 26 && (40..200).contains(&r) {
            groundish += 1;
            if lum < 260 {
                dark += 1;
            }
        }
    }
    assert!(dark > 800, "影斑不足（城市应投影成片）dark={dark}");
    println!(
        "OK shadow demo dark={}/{} buildings={}",
        dark,
        groundish,
        city.len()
    );
}
