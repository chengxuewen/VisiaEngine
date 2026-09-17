//! sRGB↔线性换算口（CORE-16）。色彩契约的 CPU 半边：宿主/CSS 惯例值=sRGB 编码，
//! 引擎光照域=线性；GPU 半边由 Srgb 格式硬件承担（render-wgpu），本模块只管标量。

/// sRGB 编码色 → 线性光（逐通道，alpha 原样）。IEC 61966-2-1 分段精确式
/// （阈值 0.04045；线性段 /12.92，幂段 ((c+0.055)/1.055)^2.4）——pow2.2 近似不做，
/// 端点必须逐位精确（golden 锁值面）。
#[must_use]
pub fn srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
    let f = |v: f32| {
        let d = f64::from(v);
        let r = if d <= 0.04045 {
            d / 12.92
        } else {
            ((d + 0.055) / 1.055).powf(2.4)
        };
        r as f32
    };
    [f(c[0]), f(c[1]), f(c[2]), c[3]]
}

/// 线性光 → sRGB 编码（逐通道，alpha 原样；CORE-16 逆口，供读回/调试与换算自检）。
#[must_use]
pub fn linear_to_srgb(c: [f32; 4]) -> [f32; 4] {
    let f = |v: f32| {
        let d = f64::from(v);
        let r = if d <= 0.0031308 {
            d * 12.92
        } else {
            1.055 * d.powf(1.0 / 2.4) - 0.055
        };
        r as f32
    };
    [f(c[0]), f(c[1]), f(c[2]), c[3]]
}
