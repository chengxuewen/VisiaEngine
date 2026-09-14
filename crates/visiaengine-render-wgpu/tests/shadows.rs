//! WGPU-19：shadow pre-pass + 接收端 PCF。行为锁 [PIT-8：先 probe 定阈再收紧]：
//! ① 亮暗两态并存（非全黑全亮=PIT-5 深度域症状级死锁）；
//! ② 影斑暗像素行质心在光反侧（+y 仰光→影向屏幕下）；
//! ③ 远端地面小块 std 有界（自影 acne 高频大方差锁）；
//! ④ None 位无暗斑（关闭位行为=现状）。逐位零回归=既有 golden 全集自证。

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MeshDesc, RenderBackend,
    ShadowBias, ShadowSetup, Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};

const W: u32 = 96;
const H: u32 = 96;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
/// 世界光向（指向光源）：+y 仰、+z 高。
const LIGHT_DIR: [f32; 3] = [0.0, 0.42, 0.91];

fn scene(with_shadow: Option<ShadowSetup>) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let ground = b
        .create_mesh(&MeshDesc {
            positions: &[
                [-4.0, -4.0, 0.0],
                [4.0, -4.0, 0.0],
                [4.0, 4.0, 0.0],
                [-4.0, 4.0, 0.0],
            ],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        })
        .unwrap();
    let gmat = b.create_material([0.8, 0.8, 0.8, 1.0]).unwrap();
    let (pos, nrm, idx) = unit_box_mesh();
    let cube = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .unwrap();
    let wmat = b.create_material([1.0, 1.0, 1.0, 1.0]).unwrap();
    let iid = b
        .create_instances(&InstanceDesc {
            data: &[Instance::new([0.0, 0.0, 0.0], 2.0, [1.0, 1.0, 1.0])],
        })
        .unwrap();

    let rig = CameraRig::look_at([0.0, -7.0, 6.0], [0.0, 0.0, 0.6], [0.0, 1.0, 0.0]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 0.1,
        shadow: with_shadow,
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: ground,
                material: gmat,
                origin: [0.0; 3],
                transform: T4,
            },
            DrawCommand::DrawInstances {
                mesh: cube,
                material: wmat,
                instances: iid,
                origin: [0.0; 3],
                transform: T4,
            },
        ],
    };
    b.render_to_pixels(&frame).expect("render")
}

/// 光源正交相机：沿光向 30 俯瞰（REND-31 构造契约：rig 路三元组）。
fn setup(size: f32) -> ShadowSetup {
    let n =
        (LIGHT_DIR[0] * LIGHT_DIR[0] + LIGHT_DIR[1] * LIGHT_DIR[1] + LIGHT_DIR[2] * LIGHT_DIR[2])
            .sqrt();
    let eye = [
        f64::from(LIGHT_DIR[0] / n) * 30.0,
        f64::from(LIGHT_DIR[1] / n) * 30.0,
        f64::from(LIGHT_DIR[2] / n) * 30.0 + 0.6,
    ];
    let rig = CameraRig::look_at(eye, [0.0, 0.0, 0.6], [0.0, 1.0, 0.0]);
    ShadowSetup {
        proj: rig
            .ortho_frame(10.0, W as f32, H as f32, 1.0, 80.0)
            .unwrap(),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        light_dir: LIGHT_DIR,
        size,
        bias: ShadowBias {
            constant: -1.2,
            slope: -1.5,
        },
    }
}

/// 地面灰判定（等通道、20..205 带——排除纯背景/立方面高光）。
fn gray(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> Option<u8> {
    let i = ((y * W + x) * 4) as usize;
    let (r, g, bl) = (img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]);
    (r == g && g == bl && r > 20 && r < 205).then_some(r)
}

// spec: WGPU-19
#[test]
fn shadow_exists_not_all_black_or_white() {
    let img = scene(Some(setup(0.02)));
    let (mut dark, mut lit) = (0u32, 0u32);
    for y in 0..H {
        for x in 0..W {
            if let Some(r) = gray(&img, x, y) {
                if r < 70 {
                    dark += 1;
                } else {
                    lit += 1;
                }
            }
        }
    }
    assert!(
        dark >= 30 && lit >= 300,
        "亮暗两态失衡 dark={dark} lit={lit}"
    );
}

// spec: WGPU-19
#[test]
fn shadow_centroid_lies_anti_light_direction() {
    let img = scene(Some(setup(0.02)));
    let (mut n, mut sum) = (0u64, 0u64);
    for y in 0..H {
        for x in 0..W {
            if gray(&img, x, y).is_some_and(|r| r < 70) {
                n += 1;
                sum += u64::from(y);
            }
        }
    }
    assert!(n > 30, "无影斑可测");
    let cy = sum as f64 / n as f64;
    assert!(cy > f64::from(H) / 2.0, "暗质心行 {cy:.1} 不在光反侧（下）");
}

// spec: WGPU-19
#[test]
fn far_ground_patch_is_smooth() {
    let img = scene(Some(setup(0.02)));
    let mut vals: Vec<u32> = Vec::new();
    for y in 74..92 {
        for x in 6..26 {
            if let Some(r) = gray(&img, x, y) {
                vals.push(u32::from(r));
            }
        }
    }
    assert!(vals.len() > 150, "远端地面样本不足 {}", vals.len());
    let mean = vals.iter().sum::<u32>() as f64 / vals.len() as f64;
    let var = vals
        .iter()
        .map(|v| (f64::from(*v) - mean).powi(2))
        .sum::<f64>()
        / vals.len() as f64;
    assert!(
        var.sqrt() < 10.0,
        "地面 patch std {:.1} 过大（acne）",
        var.sqrt()
    );
}

// spec: WGPU-19
#[test]
fn none_bit_produces_no_dark_patch() {
    let img = scene(None);
    let dark = (0..H)
        .map(|y| {
            (0..W)
                .filter(|&x| gray(&img, x, y).is_some_and(|r| r < 70))
                .count() as u32
        })
        .sum::<u32>();
    assert!(dark < 10, "None 位产生暗斑 {dark}");
}

/// 半影带（地面窗内中间灰 70<=r<=150）计数。
fn penumbra_count(size: f32) -> u32 {
    let img = scene(Some(setup(size)));
    let mut mid = 0u32;
    for y in 40..H {
        for x in 0..W {
            if gray(&img, x, y).is_some_and(|r| (70..=150).contains(&r)) {
                mid += 1;
            }
        }
    }
    mid
}

// spec: WGPU-20
#[test]
fn penumbra_grows_with_light_size() {
    // PCSS [E3D:B3]：光源角尺寸↑ → 半影宽度↑（中间灰单调增）；且远小于实心带（有界）
    let (hard, soft) = (penumbra_count(0.02), penumbra_count(1.5));
    assert!(hard < 400, "近硬影中间灰过多 {hard}");
    assert!(
        soft > hard * 2,
        "半影不随 size 扩展 hard={hard} soft={soft}"
    );
    assert!(soft < 6000, "半影爆炸（clamp 缺位）{soft}");
}
