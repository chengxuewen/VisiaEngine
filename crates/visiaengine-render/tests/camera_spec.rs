//! REND-10..14 相机数学（docs/sdd/render.md）。

#![allow(clippy::float_cmp)]

use visiaengine_render::camera::CameraRig;

const TOL: f64 = 1e-6;

fn rig() -> CameraRig {
    CameraRig::look_at([3.0, 4.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
}

// spec: REND-10
#[test]
fn camera_rig_view_matrix_lookat() {
    let v = rig().view_matrix();
    // view·eye(齐次) = 原点：取 eye 列向量乘 view 各行
    let e = [3.0f32, 4.0, 5.0, 1.0];
    for row in 0..4 {
        let dot: f32 = (0..4).map(|c| v[c][row] * e[c]).sum();
        assert!(dot.abs() < 1e-3, "row {row} = {dot}, 期望 ~0");
    }
    // target 在 view 空间 -z（右手系看向 -z）
    let t = [0.0f32, 0.0, 0.0, 1.0];
    let tz: f32 = (0..4).map(|c| v[c][2] * t[c]).sum();
    assert!(tz < 0.0, "target z={tz} 应为负");
}

// spec: REND-11
#[test]
fn persp_proj_matches_glam_std() {
    let (near, far) = (1.0f32, 1000.0);
    let p = rig().perspective(std::f32::consts::FRAC_PI_2, 1.0, near, far).unwrap();
    // 标准 RH-GL(深度0..1) fov=90° aspect=1: f=1/tan45=1
    assert!((p[0][0] - 1.0).abs() < 1e-4);
    assert!((p[1][1] - 1.0).abs() < 1e-4);
    assert!((p[2][2] - (-(far / (far - far * 0.0 + near * 0.0 + far)) * 0.0 - far / (far - near))).abs() < 1e-3,
        "z 对角 = -far/(far-near)={}", -far / (far - near));
    assert!((p[3][2] + 1.0).abs() < 1e-4, "GL 风格 w 分量 -1");
}

// spec: REND-12
#[test]
fn ortho_zoom_scales_halfextents() {
    let r = rig();
    for zoom in [1.0f32, 4.0, 128.0] {
        let p = r.ortho_frame(zoom, 640.0, 480.0, 1.0, 1e5).unwrap();
        let expect = 2.0 / (640.0_f32.powf(0.0) as f32) * (1.0 / zoom) * zoom; // 半宽=zoom 语义→列0=1/hw
        let _ = expect;
        assert!((p[0][0] * zoom - 1.0).abs() < 1e-3, "半宽=zoom 语义下 x 列=1/zoom");
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
        assert!((ea[i] - eb[i]).abs() < TOL, "axis{i}: {} vs {}", ea[i], eb[i]);
    }
}

// spec: REND-14
#[test]
fn degenerate_near_far_rejected() {
    assert!(rig().perspective(1.0, 1.0, 100.0, 50.0).is_none());
    assert!(rig().ortho_frame(10.0, 1.0, 1.0, 100.0, 50.0).is_none());
}
