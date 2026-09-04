//! REND-19/20：D7 分层 origin 组合数学（docs/sdd/render.md）。

#![allow(clippy::float_cmp)]

use visiaengine_render::rebase::compose_mvp;

const I4: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn apply(m: &[[f32; 4]; 4], v: [f32; 4]) -> [f32; 4] {
    let mut o = [0.0f32; 4];
    for (r, cell) in o.iter_mut().enumerate() {
        *cell = (0..4).map(|c| m[c][r] * v[c]).sum();
    }
    o
}

// spec: REND-19
#[test]
fn rebase_identity_at_zero_origin() {
    let proj = [
        [2.0f32, 0.0, 0.0, 0.0],
        [0.0, 2.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let eye = [5.0f64, -3.0, 2.0];
    let m_local = [
        [1.5f64, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [4.0, 5.0, 6.0, 1.0],
    ];
    let mvp = compose_mvp(&proj, &I4, &eye, &[0.0; 3], &m_local);
    // 旧组合：P · V(eye) · M_world 手算（V=R·T(-e)，此处 R=I）
    let v = [-5.0f32, 3.0, -2.0, 0.0]; // 平移列（w 分量增量为 0：T·M 的 w 行不变）
    let mut old_m = [[0.0f32; 4]; 4];
    for c in 0..4 {
        for r in 0..4 {
            old_m[c][r] = [m_local[c][r] as f32][0] + if c == 3 { v[r] } else { 0.0 };
        }
    }
    // M·T 简化：直接验点对同一性
    let p = [1.0f32, 1.0, 1.0, 1.0];
    let a = apply(&mvp, p);
    let b = apply(&proj, apply(&old_m, p));
    for r in 0..4 {
        assert!(
            (a[r] - b[r]).abs() < 1e-3,
            "REND-19 漂移 r{r}: {a:?} vs {b:?}"
        );
    }
}

// spec: REND-20
#[test]
fn far_origin_precision_preserved() {
    let origin = [1.0e7f64, 0.0, 0.0];
    let eye = [1.0e7f64 + 10.0, 0.0, 0.0];
    let mvp = compose_mvp(&I4, &I4, &eye, &origin, &I4.map(|c| c.map(f64::from)));
    // 顶点 local (0.5,0,0)：新 clip = 0.5 + (origin-eye) = -9.5（f32 精确小值）
    let clip = apply(&mvp, [0.5, 0.0, 0.0, 1.0]);
    assert!((clip[0] + 9.5).abs() < 1e-4, "新路 clip={}", clip[0]);
    // 旧路：world 顶点 1e7+0.5 直 cast → f32 在 1e7 处 ulp≈1，0.5 被吞（失真≥0.25m）
    let legacy_world = 1.0e7f64 + 0.5;
    let legacy_err = (legacy_world as f32 as f64 - legacy_world).abs();
    assert!(
        legacy_err >= 0.25,
        "旧路应失真≥0.25m 以对照，实测 {legacy_err}"
    );
}
