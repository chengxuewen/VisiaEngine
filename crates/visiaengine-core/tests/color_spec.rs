//! CORE-16: the sRGB<->linear conversion contract (the CPU half of the color-management band, three locks: endpoints/monotonicity/round-trip).

// spec: CORE-16
#[test]
fn endpoints_exact_and_mid_value_pinned() {
    use visiaengine_core::{linear_to_srgb, srgb_to_linear};
    // endpoints bit-exact (golden locked values depend on this form)
    assert_eq!(srgb_to_linear([0.0, 0.0, 0.0, 1.0]), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(srgb_to_linear([1.0, 1.0, 1.0, 0.5]), [1.0, 1.0, 1.0, 0.5]);
    assert_eq!(linear_to_srgb([0.0, 0.0, 0.0, 1.0]), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(linear_to_srgb([1.0, 1.0, 1.0, 0.5]), [1.0, 1.0, 1.0, 0.5]);
    // mid-gray pinned value: sRGB 128/255 -> linear 0.2158 +/- 1e-3 (the IEC piecewise form, not pow2.2's 0.2176? -- the exact form rules)
    let m = srgb_to_linear([128.0 / 255.0; 4]);
    assert!(
        (m[0] - 0.2158).abs() < 1e-3,
        "mid-gray linear value drift: {}",
        m[0]
    );
    // the alpha channel never participates in conversion
    let a = srgb_to_linear([0.5, 0.5, 0.5, 0.25]);
    assert_eq!(a[3], 0.25);
}

// spec: CORE-16
#[test]
fn monotonic_and_roundtrip_stable() {
    use visiaengine_core::{linear_to_srgb, srgb_to_linear};
    // monotonic: the 256-level ladder is strictly non-decreasing with distinct endpoints
    let mut prev = -1.0f32;
    for i in 0..=255 {
        let v = srgb_to_linear([i as f32 / 255.0, 0.0, 0.0, 1.0])[0];
        assert!(v >= prev, "non-monotonic @{i}");
        prev = v;
    }
    assert_eq!(prev, 1.0);
    // round-trip: 8 sample points <= 2e-7 (self-consistent within f32 precision)
    for i in [1, 10, 43, 128, 180, 200, 230, 254] {
        let x = i as f32 / 255.0;
        let rt = linear_to_srgb(srgb_to_linear([x, x, x, 1.0]))[0];
        assert!((rt - x).abs() < 2e-7, "round-trip {i} drift: {rt}");
    }
}
