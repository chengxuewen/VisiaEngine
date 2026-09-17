//! CORE-16：sRGB↔线性换算口契约（色彩管理带的 CPU 半边，端点/单调/往返三锁）。

// spec: CORE-16
#[test]
fn endpoints_exact_and_mid_value_pinned() {
    use visiaengine_core::{linear_to_srgb, srgb_to_linear};
    // 端点逐位精确（golden 锁值依赖此形）
    assert_eq!(srgb_to_linear([0.0, 0.0, 0.0, 1.0]), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(srgb_to_linear([1.0, 1.0, 1.0, 0.5]), [1.0, 1.0, 1.0, 0.5]);
    assert_eq!(linear_to_srgb([0.0, 0.0, 0.0, 1.0]), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(linear_to_srgb([1.0, 1.0, 1.0, 0.5]), [1.0, 1.0, 1.0, 0.5]);
    // 中灰钉值：sRGB 128/255 → 线性 0.2158±1e-3（IEC 分段式，非 pow2.2 的 0.2176?——以精确式为准）
    let m = srgb_to_linear([128.0 / 255.0; 4]);
    assert!((m[0] - 0.2158).abs() < 1e-3, "中灰线性值漂移: {}", m[0]);
    // alpha 通道永不参与换算
    let a = srgb_to_linear([0.5, 0.5, 0.5, 0.25]);
    assert_eq!(a[3], 0.25);
}

// spec: CORE-16
#[test]
fn monotonic_and_roundtrip_stable() {
    use visiaengine_core::{linear_to_srgb, srgb_to_linear};
    // 单调：256 级阶梯严格不减且端点异值
    let mut prev = -1.0f32;
    for i in 0..=255 {
        let v = srgb_to_linear([i as f32 / 255.0, 0.0, 0.0, 1.0])[0];
        assert!(v >= prev, "非单调 @{i}");
        prev = v;
    }
    assert_eq!(prev, 1.0);
    // 往返：8 采样点 ≤2e-7（f32 精度内自洽）
    for i in [1, 10, 43, 128, 180, 200, 230, 254] {
        let x = i as f32 / 255.0;
        let rt = linear_to_srgb(srgb_to_linear([x, x, x, 1.0]))[0];
        assert!((rt - x).abs() < 2e-7, "往返 {i} 漂: {rt}");
    }
}
