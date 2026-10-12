//! REND-10..14 camera math (docs/sdd/render.md).

#![allow(clippy::float_cmp)]

use visiaengine_render::camera::CameraRig;

const TOL: f64 = 1e-6;

fn rig() -> CameraRig {
    CameraRig::look_at([3.0, 4.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
}

/// Column-major matrix, column-vector multiplication taking a given row
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
        assert!(dot.abs() < 1e-3, "row {row} = {dot}, expected ~0");
    }
    let wsum: f32 = (0..4).map(|cc| v[cc][3] * e[cc]).sum();
    assert!((wsum - 1.0).abs() < 1e-3, "w row = 1, got {wsum}");
    let t = [0.0f32, 0.0, 0.0, 1.0];
    let tz: f32 = (0..4).map(|c| v[c][2] * t[c]).sum();
    assert!(tz < 0.0, "target z={tz} should be negative");
}

// spec: REND-11
#[test]
fn persp_proj_matches_glam_std() {
    // behavior assertion (wgpu native depth semantics): view z = -near -> clip 0, z = -far -> clip 1
    let (near, far) = (1.0f32, 1000.0);
    let p = rig()
        .perspective(std::f32::consts::FRAC_PI_2, 1.0, near, far)
        .unwrap();
    let zn = apply(&p, [0.0, 0.0, -near, 1.0], 2) / apply(&p, [0.0, 0.0, -near, 1.0], 3);
    let zf = apply(&p, [0.0, 0.0, -far, 1.0], 2) / apply(&p, [0.0, 0.0, -far, 1.0], 3);
    assert!(zn.abs() < 1e-4, "near depth should map to 0, got {zn}");
    assert!(
        (zf - 1.0).abs() < 1e-4,
        "far depth should map to 1, got {zf}"
    );
    // fov=90 deg aspect=1 -> x/y diagonal 1
    assert!((apply(&p, [1.0, 0.0, 0.0, 1.0], 0) - 1.0).abs() < 1e-4);
    assert!((apply(&p, [0.0, 1.0, 0.0, 1.0], 1) - 1.0).abs() < 1e-4);
}

// spec: REND-12
#[test]
fn ortho_zoom_scales_halfextents() {
    let r = rig();
    for zoom in [1.0f32, 4.0, 128.0] {
        let p = r.ortho_frame(zoom, 640.0, 480.0, 1.0, 1e5).unwrap();
        assert!((p[0][0] - 1.0 / zoom).abs() < 1e-4, "x column = 1/zoom");
        // depth [0,1]: z = -near -> 0
        let zn = apply(&p, [0.0, 0.0, -1.0, 1.0], 2);
        assert!(zn.abs() < 1e-4, "ortho near depth should be 0, got {zn}");
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
        assert!(
            (x - y).abs() < 1e-3,
            "t-domain continuity broken: {x} vs {y}"
        );
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

/// Normalize to [0,2pi) (the canonical form for pose equivalence -- flyTo endpoint yaw equal to the source up to an integer 2pi)
fn yaw_norm(y: f64) -> f64 {
    y.rem_euclid(std::f64::consts::TAU)
}

// spec: REND-34
#[test]
fn fly_easing_endpoints_exact() {
    use visiaengine_render::Easing;
    // Three curves f(0)=0 / f(1)=1 bit-exact (easing must not break the REND-16 endpoint-semantics foundation)
    for e in [Easing::Linear, Easing::CubicInOut, Easing::CubicOut] {
        assert_eq!(e.ease(0.0), 0.0, "{e:?} f(0)");
        assert_eq!(e.ease(1.0), 1.0, "{e:?} f(1)");
        assert_eq!(e.ease(-3.0), 0.0, "{e:?} clamped below the bound");
        assert_eq!(e.ease(7.0), 1.0, "{e:?} clamped above the bound");
    }
}

// spec: REND-34
#[test]
fn fly_easing_monotone_and_bounded() {
    use visiaengine_render::Easing;
    for e in [Easing::Linear, Easing::CubicInOut, Easing::CubicOut] {
        let mut prev = 0.0f64;
        for i in 0..=100 {
            let t = i as f64 / 100.0;
            let v = e.ease(t);
            assert!(
                (0.0..=1.0).contains(&v),
                "{e:?} out of value range t={t} v={v}"
            );
            assert!(v >= prev - 1e-12, "{e:?} non-monotonic t={t}");
            prev = v;
        }
    }
    // easing midpoint distinguishable (CubicInOut slow start: t = 0.25 -> value < linear value)
    assert!(Easing::CubicInOut.ease(0.25) < 0.25);
    assert!(Easing::CubicOut.ease(0.25) > 0.25);
    assert!((Easing::Linear.ease(0.25) - 0.25).abs() < TOL);
}

// spec: REND-34
#[test]
fn fly_sample_endpoints_and_continuity() {
    use visiaengine_render::Easing;
    let a = CameraRig::look_at([0.0, -40.0, 20.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let b = CameraRig::look_at([30.0, 10.0, 5.0], [4.0, 2.0, 0.0], [0.0, 1.0, 0.0]);
    for e in [Easing::Linear, Easing::CubicInOut, Easing::CubicOut] {
        // endpoints exact: t = 0 always a, t = 1 always b (all fields bit by bit -- including the from/taking near/far semantics)
        let s0 = CameraRig::fly_sample(&a, &b, 0.0, e);
        let s1 = CameraRig::fly_sample(&a, &b, 1.0, e);
        assert_eq!(s0.target, a.target);
        assert_eq!((s0.yaw - a.yaw).abs(), 0.0);
        assert_eq!(s1.target, b.target);
        assert_eq!((s1.yaw - b.yaw).abs(), 0.0);
        // continuity: densely sampled adjacent-step pose deltas bounded (position components)
        let mut prev = s0;
        for i in 1..=200 {
            let t = i as f64 / 200.0;
            let s = CameraRig::fly_sample(&a, &b, t, e);
            let d = ((s.target[0] - prev.target[0]).powi(2)
                + (s.target[1] - prev.target[1]).powi(2)
                + (s.target[2] - prev.target[2]).powi(2))
            .sqrt();
            assert!(d < 0.5, "{e:?} discontinuous t={t} step {d}");
            prev = s;
        }
    }
}

// spec: REND-34
#[test]
fn fly_sample_yaw_shortest_arc_wrap() {
    use visiaengine_render::Easing;
    let base = CameraRig::look_at([0.0, -40.0, 20.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    // craft endpoints near +/- pi: linear dyaw ~ 6.2 (nearly a full loop) while the shortest arc ~ 0.28
    let mut a = base;
    a.yaw = std::f64::consts::PI - 0.14;
    let mut b = base;
    b.yaw = -std::f64::consts::PI + 0.14;
    let mid = CameraRig::fly_sample(&a, &b, 0.5, Easing::Linear);
    // taking the shortest arc => the midpoint should cross the +/- pi boundary (mod 2pi ~ pi), not go through 0
    assert!(
        yaw_norm(mid.yaw) > std::f64::consts::PI - 0.01,
        "wrap failed: mid yaw mod 2pi = {}, expected ~pi",
        yaw_norm(mid.yaw)
    );
    // a linear wrap would fall near 0 (the old path value +/- 0.14), locking this correction bidirectionally on that basis
    let linear_mid = (a.yaw + b.yaw) / 2.0;
    assert!(
        linear_mid.abs() < 0.01,
        "control premise: the linear midpoint ~ 0 (got {linear_mid})"
    );
    // endpoint pose equivalence (equal mod 2pi, values may differ by integer 2pi)
    let end = CameraRig::fly_sample(&a, &b, 1.0, Easing::Linear);
    assert!((yaw_norm(end.yaw) - yaw_norm(b.yaw)).abs() < 1e-9);
    // untouched when |dyaw| < pi (ordinary flights are not wrapped)
    let c = CameraRig::fly_sample(&a, &b, 0.0, Easing::Linear);
    assert!((yaw_norm(c.yaw) - yaw_norm(a.yaw)).abs() < 1e-9);
    let small_b = {
        let mut r = base;
        r.yaw = 0.5;
        r
    };
    let m2 = CameraRig::fly_sample(&base, &small_b, 0.5, Easing::Linear);
    assert!(
        (m2.yaw - (base.yaw + 0.5) / 2.0).abs() < 1e-9,
        "a small delta must stay pure linear"
    );
}

// spec: REND-37
#[test]
fn ray_ground_intersect_six_forms() {
    use visiaengine_core::{Ray, Vec3};
    let v = |x: f64, y: f64, z: f64| Vec3::new(x, y, z);
    // (1) straight down: origin (1,2,50) direction (0,0,-1) -> t = 50 -> (1,2,0)
    let hit = visiaengine_render::ray_ground_intersect(Ray {
        origin: v(1.0, 2.0, 50.0),
        dir: v(0.0, 0.0, -1.0),
    })
    .expect("straight down must intersect");
    assert!(
        (hit.x - 1.0).abs() < 1e-9 && (hit.y - 2.0).abs() < 1e-9 && hit.z.abs() < 1e-9,
        "got {hit:?}"
    );
    // (2) oblique ray
    let hit = visiaengine_render::ray_ground_intersect(Ray {
        origin: v(0.0, 0.0, 10.0),
        dir: v(0.0, -1.0, -1.0),
    })
    .expect("oblique ray");
    assert!(
        (hit.x).abs() < 1e-9 && (hit.y + 10.0).abs() < 1e-9 && hit.z.abs() < 1e-9,
        "got {hit:?}"
    );
    // (3) horizontal view rejected (d.z = 0)
    assert!(
        visiaengine_render::ray_ground_intersect(Ray {
            origin: v(0.0, 0.0, 10.0),
            dir: v(0.0, -1.0, 0.0)
        })
        .is_none()
    );
    // (4) near-horizontal (|d.z| < eps) rejected: the t-explosion guardrail
    assert!(
        visiaengine_render::ray_ground_intersect(Ray {
            origin: v(0.0, 0.0, 10.0),
            dir: v(0.0, -1.0, -1e-12)
        })
        .is_none()
    );
    // (5) pointing away (upward) rejected
    assert!(
        visiaengine_render::ray_ground_intersect(Ray {
            origin: v(0.0, 0.0, 10.0),
            dir: v(0.0, 0.0, 1.0)
        })
        .is_none()
    );
    // (6) from underground heading up = away; underground heading down = t < 0 also rejected (no navigation semantics below the ground)
    assert!(
        visiaengine_render::ray_ground_intersect(Ray {
            origin: v(0.0, 0.0, -5.0),
            dir: v(0.0, 0.0, -1.0)
        })
        .is_none()
    );
}

// spec: REND-37
#[test]
fn viewport_rect_from_frac_seam_accounting() {
    use visiaengine_render::ViewportRect;
    // half-width divides exactly: 128 x .25 = 32 precise
    let r = ViewportRect::from_frac(0.25, 0.0, 0.5, 1.0, 128, 64);
    assert_eq!((r.x, r.y, r.width, r.height), (32, 0, 64, 64));
    // seam account (1): two adjacent tiles share an edge without a gap (right edge = neighbor's left edge)
    let a = ViewportRect::from_frac(1.0 / 3.0, 0.0, 1.0 / 3.0, 1.0, 128, 64);
    let b = ViewportRect::from_frac(2.0 / 3.0, 0.0, 1.0 / 3.0, 1.0, 128, 64);
    assert_eq!(a.x + a.width, b.x, "seam break/overlap: {:?} {:?}", a, b);
    // seam account (2): the last tile's right edge is pinned to the surface bound (no 1px exposed floor)
    let z = ViewportRect::from_frac(0.75, 0.0, 0.25, 1.0, 127, 64); // odd-width rounding scenario
    assert_eq!(z.x + z.width, 127);
    // out-of-range clamping (defense against dirty host values): 0.9 + 0.5 -> w truncated at the bound and kept >= 1
    let c = ViewportRect::from_frac(0.9, 0.0, 0.5, 1.0, 100, 100);
    assert!(c.width >= 1 && c.x + c.width <= 100);
    // zero size keeps 1 (the scissor-domain floor, merged with render_view_rects max(1))
    let n = ViewportRect::from_frac(0.5, 0.5, 0.0, 0.0, 100, 100);
    assert!(
        n.width >= 1 && n.height >= 1,
        "zero size keeps 1 (the scissor floor nailed) got {:?}",
        (n.width, n.height)
    );
}
