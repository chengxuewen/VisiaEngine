//! REND-10..14 相机数学（docs/sdd/render.md）。

#![allow(clippy::float_cmp)]

use visiaengine_render::camera::CameraRig;

const TOL: f64 = 1e-6;

fn rig() -> CameraRig {
    CameraRig::look_at([3.0, 4.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
}

/// 列主序矩阵·列向量乘法取指定行
fn apply(m: &[[f32; 4]; 4], v: [f32; 4], row: usize) -> f32 {
    (0..4).map(|c| m[c][row] * v[c]).sum()
}

// spec: REND-10
#[test]
fn camera_rig_view_matrix_lookat() {
    let v = rig().view_matrix();
    let e = [3.0f32, 4.0, 5.0, 1.0];
    for (row, _) in (0..3).enumerate() {
        let dot: f32 = (0..4).map(|c| v[c][row] * e[c]).sum();
        assert!(dot.abs() < 1e-3, "row {row} = {dot}, 期望 ~0");
    }
    let wsum: f32 = (0..4).map(|cc| v[cc][3] * e[cc]).sum();
    assert!((wsum - 1.0).abs() < 1e-3, "w 行 = 1, got {wsum}");
    let t = [0.0f32, 0.0, 0.0, 1.0];
    let tz: f32 = (0..4).map(|c| v[c][2] * t[c]).sum();
    assert!(tz < 0.0, "target z={tz} 应为负");
}

// spec: REND-11
#[test]
fn persp_proj_matches_glam_std() {
    // 行为断言（wgpu 原生深度语义）：view z=-near→clip 0，z=-far→clip 1
    let (near, far) = (1.0f32, 1000.0);
    let p = rig()
        .perspective(std::f32::consts::FRAC_PI_2, 1.0, near, far)
        .unwrap();
    let zn = apply(&p, [0.0, 0.0, -near, 1.0], 2) / apply(&p, [0.0, 0.0, -near, 1.0], 3);
    let zf = apply(&p, [0.0, 0.0, -far, 1.0], 2) / apply(&p, [0.0, 0.0, -far, 1.0], 3);
    assert!(zn.abs() < 1e-4, "near 深度应映射 0，得 {zn}");
    assert!((zf - 1.0).abs() < 1e-4, "far 深度应映射 1，得 {zf}");
    // fov=90° aspect=1 → x/y 对角 1
    assert!((apply(&p, [1.0, 0.0, 0.0, 1.0], 0) - 1.0).abs() < 1e-4);
    assert!((apply(&p, [0.0, 1.0, 0.0, 1.0], 1) - 1.0).abs() < 1e-4);
}

// spec: REND-12
#[test]
fn ortho_zoom_scales_halfextents() {
    let r = rig();
    for zoom in [1.0f32, 4.0, 128.0] {
        let p = r.ortho_frame(zoom, 640.0, 480.0, 1.0, 1e5).unwrap();
        assert!((p[0][0] - 1.0 / zoom).abs() < 1e-4, "x 列 = 1/zoom");
        // 深度 [0,1]：z=-near→0
        let zn = apply(&p, [0.0, 0.0, -1.0, 1.0], 2);
        assert!(zn.abs() < 1e-4, "ortho near 深度应 0，得 {zn}");
    }
}

// spec: REND-13
#[test]
fn orbit_90deg_equivalence() {
    let mut a = rig();
    a.orbit_delta(std::f32::consts::FRAC_PI_4 as f64, 0.0);
    a.orbit_delta(std::f32::consts::FRAC_PI_4 as f64, 0.0);
    let mut b = rig();
    b.orbit_delta(std::f32::consts::FRAC_PI_2 as f64, 0.0);
    let (ea, eb) = (a.eye(), b.eye());
    for i in 0..3 {
        assert!(
            (ea[i] - eb[i]).abs() < TOL,
            "axis{i}: {} vs {}",
            ea[i],
            eb[i]
        );
    }
}

// spec: REND-14
#[test]
fn degenerate_near_far_rejected() {
    assert!(rig().perspective(1.0, 1.0, 100.0, 50.0).is_none());
    assert!(rig().ortho_frame(10.0, 1.0, 1.0, 100.0, 50.0).is_none());
}

// spec: REND-15
#[test]
fn switch_midpoint_continuity() {
    let a = CameraRig::orbit([0.0; 3], 0.0, 1.5, 500.0, 40.0, 1.1, 1.0, 1e5);
    let b = CameraRig::orbit([0.0; 3], 0.8, 0.35, 9.0, 1.0, 1.1, 0.1, 500.0);
    let m1 = CameraRig::mix_rig(&a, &b, 0.5);
    let m2 = CameraRig::mix_rig(&a, &b, 0.500_001);
    for (x, y) in [
        (m1.dist, m2.dist),
        (m1.pitch, m2.pitch),
        (m1.zoom, m2.zoom),
        (m1.fov_y, m2.fov_y),
    ] {
        assert!((x - y).abs() < 1e-3, "t 域连续性破坏: {x} vs {y}");
    }
}

// spec: REND-16
#[test]
fn switch_endpoints_exact() {
    let a = CameraRig::orbit([1.0, 2.0, 3.0], 0.1, 0.2, 7.0, 2.0, 1.0, 0.5, 99.0);
    let b = CameraRig::orbit([-4.0, 0.0, 5.0], 2.0, -0.4, 3.0, 0.5, 0.7, 0.1, 42.0);
    assert_eq!(CameraRig::mix_rig(&a, &b, 0.0), a);
    assert_eq!(CameraRig::mix_rig(&a, &b, 1.0), b);
}
