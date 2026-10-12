//! sRGB<->linear conversion entries (CORE-16). The CPU half of the color contract: host/CSS convention values = sRGB encoded,
//! the engine lighting domain = linear; the GPU half is carried by the Srgb format in hardware (render-wgpu), this module handles scalars only.

/// sRGB-encoded color -> linear light (per channel, alpha as-is). The piecewise-exact IEC 61966-2-1 form
/// (threshold 0.04045; linear segment /12.92, power segment ((c+0.055)/1.055)^2.4) -- no pow2.2 approximation,
/// the endpoints must be bit-exact (the golden locked-value face).
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

/// Linear light -> sRGB encoding (per channel, alpha as-is; the CORE-16 inverse entry, for readback/debug and conversion self-check).
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
