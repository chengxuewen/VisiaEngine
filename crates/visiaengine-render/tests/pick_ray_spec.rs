//! REND-21/22：屏幕像素→世界射线解析构造（交互片，[E3D:4xx]）。
//! 双口签名（persp 用 rig.fov_y / ortho 用 rig.zoom）——投影模式在 Frame 装配层，
//! ray 口显式分模式，无魔法数值。

use visiaengine_render::{CameraRig, screen_to_ray_ortho, screen_to_ray_persp};

const W: f32 = 800.0;
const H: f32 = 600.0;

// spec: REND-21
#[test]
fn persp_center_and_corner_rays() {
    let rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    // 中心 → 沿 -z 直击原点（origin=eye, dir 归一）
    let r = screen_to_ray_persp(&rig, W / 2.0, H / 2.0, W, H).unwrap();
    assert!((r.origin.x).abs() < 1e-9 && (r.origin.y).abs() < 1e-9);
    assert!((r.dir.z + 1.0).abs() < 1e-9);
    let ty = (rig.fov_y / 2.0).tan();
    let aspect = f64::from(W) / f64::from(H);
    let tx = ty * aspect;
    // 左上角 (0,0)：NDC(-1,+1) → x 负向、y 正向
    let c = screen_to_ray_persp(&rig, 0.0, 0.0, W, H).unwrap();
    assert!((c.dir.x / -c.dir.z + tx).abs() < 1e-12 && (c.dir.y / -c.dir.z - ty).abs() < 1e-12);
    // 右下角：镜像
    let d = screen_to_ray_persp(&rig, W, H, W, H).unwrap();
    assert!((d.dir.x / -d.dir.z - tx).abs() < 1e-12 && (d.dir.y / -d.dir.z + ty).abs() < 1e-12);
    // 归一化
    let n = (d.dir.x * d.dir.x + d.dir.y * d.dir.y + d.dir.z * d.dir.z).sqrt();
    assert!((n - 1.0).abs() < 1e-12);
}

// spec: REND-22
#[test]
fn ortho_and_d7_far_field() {
    let mut rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    rig.zoom = 2.0; // look_at 默认 1.0——ortho 半宽语义显式设
    // 中心射线：dir=-z，origin=eye
    let r = screen_to_ray_ortho(&rig, W / 2.0, H / 2.0, W, H).unwrap();
    assert!(r.dir.x.abs() < 1e-9 && r.dir.y.abs() < 1e-9 && (r.dir.z + 1.0).abs() < 1e-9);
    // 左上角：右/上基偏移 (-hw, +hh)，hw=zoom=2 → hh=2·h/w=1.5
    let c = screen_to_ray_ortho(&rig, 0.0, 0.0, W, H).unwrap();
    assert!((c.origin.x + 2.0).abs() < 1e-9 && (c.origin.y - 1.5).abs() < 1e-9);
    // D7 远场：eye=(1e7,2,10) 正俯视 → 起点同移、方向不变（f64 全程无精度灾难）
    let mut far = CameraRig::look_at([1e7, 2.0, 10.0], [1e7, 2.0, 0.0], [0.0, 1.0, 0.0]);
    far.zoom = 2.0;
    let f = screen_to_ray_ortho(&far, W / 2.0, H / 2.0, W, H).unwrap();
    assert!((f.origin.x - 1e7).abs() < 1e-6 && (f.origin.y - 2.0).abs() < 1e-9);
    assert!((f.dir.z + 1.0).abs() < 1e-12);
}
