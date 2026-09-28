//! REND-42 (N2.2): anim_origin keyframe sampler — endpoint-exact linear
//! interpolation contract tests (T1 pure unit).

use visiaengine_render::anim_origin;

const KF: &[(f64, [f64; 3])] = &[
    (0.0, [0.0, 0.0, 0.0]),
    (1.0, [10.0, 0.0, 5.0]),
    (2.5, [10.0, 8.0, 5.0]),
    (4.0, [0.0, 8.0, 0.0]),
];

// spec: REND-42
#[test]
fn endpoints_return_boundary_origins_verbatim() {
    // Before first / after last → boundary origin bitwise.
    assert_eq!(anim_origin(KF, -1.0), KF[0].1);
    assert_eq!(anim_origin(KF, 0.0), KF[0].1);
    assert_eq!(anim_origin(KF, 4.0), KF[3].1);
    assert_eq!(anim_origin(KF, 99.0), KF[3].1);
}

// spec: REND-42
#[test]
fn segments_lerp_linearly() {
    // t=0.5 in segment [0,1]: k=0.5.
    assert_eq!(anim_origin(KF, 0.5), [5.0, 0.0, 2.5]);
    // t=1.75 in segment [1,2.5]: k=0.5 → [10, 4, 5].
    assert_eq!(anim_origin(KF, 1.75), [10.0, 4.0, 5.0]);
    // Exactly on an interior key → that key verbatim.
    assert_eq!(anim_origin(KF, 1.0), KF[1].1);
    assert_eq!(anim_origin(KF, 2.5), KF[2].1);
}

// spec: REND-42
#[test]
fn single_key_table_is_constant() {
    let one = [(2.0, [3.0, 4.0, 5.0])];
    assert_eq!(anim_origin(&one, 0.0), [3.0, 4.0, 5.0]);
    assert_eq!(anim_origin(&one, 2.0), [3.0, 4.0, 5.0]);
    assert_eq!(anim_origin(&one, 99.0), [3.0, 4.0, 5.0]);
}

// spec: REND-42
#[test]
fn f64_precision_survives_far_origin() {
    // D7 domain: far-origin world coordinates must not wobble in f32.
    let base = 2.0e7;
    let kf = [(0.0, [base, 0.0, 0.0]), (1.0, [base + 1.0, 0.0, 0.0])];
    let mid = anim_origin(&kf, 0.5);
    assert!(
        (mid[0] - (base + 0.5)).abs() < 1e-9,
        "f64 lerp keeps far-origin precision (got {})",
        mid[0]
    );
}
