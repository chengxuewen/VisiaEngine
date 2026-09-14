//! WGPU-16：Instanced 管线——storage 实例表、instance_index 寻址、
//! `pos.z·h + offset` 底对齐挤出、base·inst.color·shade 乘法链。
//! 谓词纪律 [PIT-8]：区域族统计+列高对比，不锁单像素；断链（let-else 静默跳过）
//! 则三族计数恒 0——判别力保留。

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MeshDesc, RenderBackend,
    Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 64;
const H: u32 = 64;

/// 单位盒：xy∈[-0.5,0.5]，**z∈[0,1] 底对齐**（Instance 挤出语义的几何前提），
/// 24 顶点每面独立法线，36 索引。
fn unit_box() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let mut pos = Vec::new();
    let mut nrm = Vec::new();
    let faces: [([f32; 3], [[f32; 3]; 4]); 5] = [
        (
            [0.0, 0.0, -1.0],
            [
                [-0.5, -0.5, 0.],
                [0.5, -0.5, 0.],
                [0.5, 0.5, 0.],
                [-0.5, 0.5, 0.],
            ],
        ), // 底（不可见亦入册保法线穷举）
        (
            [0.0, -1.0, 0.0],
            [
                [-0.5, -0.5, 1.],
                [0.5, -0.5, 1.],
                [0.5, -0.5, 0.],
                [-0.5, -0.5, 0.],
            ],
        ), // 前（相机侧）
        (
            [1.0, 0.0, 0.0],
            [
                [0.5, -0.5, 1.],
                [0.5, 0.5, 1.],
                [0.5, 0.5, 0.],
                [0.5, -0.5, 0.],
            ],
        ),
        (
            [0.0, 1.0, 0.0],
            [
                [0.5, 0.5, 1.],
                [-0.5, 0.5, 1.],
                [-0.5, 0.5, 0.],
                [0.5, 0.5, 0.],
            ],
        ),
        (
            [0.0, 0.0, 1.0],
            [
                [-0.5, 0.5, 1.],
                [0.5, 0.5, 1.],
                [0.5, -0.5, 1.],
                [-0.5, -0.5, 1.],
            ],
        ), // 顶
    ];
    let mut idx = Vec::new();
    for (n, quad) in faces {
        for v in quad {
            pos.push(v);
            nrm.push(n);
        }
        let b = idx.len() as u32 / 4 * 4;
        idx.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
    (pos, nrm, idx)
}

fn px(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [
        img.rgba[i],
        img.rgba[i + 1],
        img.rgba[i + 2],
        img.rgba[i + 3],
    ]
}

/// 三栋楼：x=-2/0/+2，高 1/2/3，色 红/绿/蓝。返回渲染帧。
fn city() -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (pos, nrm, idx) = unit_box();
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .unwrap();
    let mat = b.create_material([1.0, 1.0, 1.0, 1.0]).unwrap();
    let table = vec![
        Instance::new([-2.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0]),
        Instance::new([0.0, 0.0, 0.0], 2.0, [0.0, 1.0, 0.0]),
        Instance::new([2.0, 0.0, 0.0], 3.0, [0.0, 0.0, 1.0]),
    ];
    let iid = b
        .create_instances(&InstanceDesc { data: &table })
        .expect("instances");
    let rig = CameraRig::look_at([0.0, -8.0, 4.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.02, 0.02, 0.02, 1.0],
            },
            DrawCommand::DrawInstances {
                mesh,
                material: mat,
                instances: iid,
                origin: [0.0; 3],
                transform: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        ],
    };
    b.render_to_pixels(&frame).expect("render")
}

// spec: WGPU-16
#[test]
fn three_instances_render_three_color_families() {
    let img = city();
    let (mut r, mut g, mut bl) = (0u32, 0u32, 0u32);
    for y in 0..H {
        for x in 0..W {
            let p = px(&img, x, y);
            if p[0] > 60 && p[1] < 20 && p[2] < 20 {
                r += 1;
            } else if p[1] > 60 && p[0] < 20 && p[2] < 20 {
                g += 1;
            } else if p[2] > 60 && p[0] < 20 && p[1] < 20 {
                bl += 1;
            }
        }
    }
    assert!(
        r > 20 && g > 20 && bl > 20,
        "三色族缺失 r={r} g={g} b={bl}（静默跳过=全 0）"
    );
}

// spec: WGPU-16
#[test]
fn instance_height_extrudes_column_extent() {
    // 底对齐挤出语义：同宽度三列的彩色列高 b > g > r（高度 3/2/1 的直接投影证据）。
    let img = city();
    let col_h = |fam: u8, x0: u32, x1: u32| -> u32 {
        let mut mx = 0u32;
        for x in x0..x1 {
            let c = (0..H)
                .filter(|&y| {
                    let p = px(&img, x, y);
                    match fam {
                        0 => p[0] > 60 && p[1] < 20 && p[2] < 20,
                        1 => p[1] > 60 && p[0] < 20 && p[2] < 20,
                        _ => p[2] > 60 && p[0] < 20 && p[1] < 20,
                    }
                })
                .count() as u32;
            mx = mx.max(c);
        }
        mx
    };
    let (hr, hg, hb) = (col_h(0, 8, 26), col_h(1, 24, 40), col_h(2, 38, 56));
    assert!(
        hr > 4 && hg > hr && hb > hg,
        "列高序破坏 r={hr} g={hg} b={hb}"
    );
}

// spec: WGPU-16
#[test]
fn empty_instance_table_rejected_on_create() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    assert!(
        b.create_instances(&InstanceDesc { data: &[] }).is_err(),
        "空表建期即拒（REND-28 闭合裁决）"
    );
}

// spec: WGPU-16
#[test]
fn missing_instance_table_skips_not_panics() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (pos, nrm, idx) = unit_box();
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .unwrap();
    let mat = b.create_material([1.0, 1.0, 1.0, 1.0]).unwrap();
    let rig = CameraRig::look_at([0.0, -8.0, 4.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.02, 0.02, 0.02, 1.0],
            },
            DrawCommand::DrawInstances {
                mesh,
                material: mat,
                instances: 9999, // 缺表
                origin: [0.0; 3],
                transform: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        ],
    };
    let img = b.render_to_pixels(&frame).expect("缺表不得 panic");
    let p = px(&img, 32, 32);
    assert!(
        p[0] < 10 && p[1] < 10 && p[2] < 10,
        "缺表=skip 非未定义 {p:?}"
    );
}
