//! REND-19/20: the D7 layered-origin composition math (docs/sdd/render.md).

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
    // the old composition: P * V(eye) * M_world by hand (V = R*T(-e), R = I here)
    let v = [-5.0f32, 3.0, -2.0, 0.0]; // translation column (the w component's increment is 0: the w row of T*M is unchanged)
    let mut old_m = [[0.0f32; 4]; 4];
    for c in 0..4 {
        for r in 0..4 {
            old_m[c][r] = [m_local[c][r] as f32][0] + if c == 3 { v[r] } else { 0.0 };
        }
    }
    // M*T simplification: verify point correspondence directly
    let p = [1.0f32, 1.0, 1.0, 1.0];
    let a = apply(&mvp, p);
    let b = apply(&proj, apply(&old_m, p));
    for r in 0..4 {
        assert!(
            (a[r] - b[r]).abs() < 1e-3,
            "REND-19 drift r{r}: {a:?} vs {b:?}"
        );
    }
}

// spec: REND-20
#[test]
fn far_origin_precision_preserved() {
    let origin = [1.0e7f64, 0.0, 0.0];
    let eye = [1.0e7f64 + 10.0, 0.0, 0.0];
    let mvp = compose_mvp(&I4, &I4, &eye, &origin, &I4.map(|c| c.map(f64::from)));
    // vertex local (0.5,0,0): new clip = 0.5 + (origin-eye) = -9.5 (f32 exact small value)
    let clip = apply(&mvp, [0.5, 0.0, 0.0, 1.0]);
    assert!(
        (clip[0] + 9.5).abs() < 1e-4,
        "the new road clip={}",
        clip[0]
    );
    // the old road: world vertex 1e7+0.5 cast directly -> f32 ulp ~ 1 at 1e7, the 0.5 is swallowed (distortion >= 0.25m)
    let legacy_world = 1.0e7f64 + 0.5;
    let legacy_err = (legacy_world as f32 as f64 - legacy_world).abs();
    assert!(
        legacy_err >= 0.25,
        "the old road should distort >= 0.25m for contrast, measured {legacy_err}"
    );
}
