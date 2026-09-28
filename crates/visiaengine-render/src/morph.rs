//! REND-41 (N2.1): projection morphTime — pure interpolation functions.
//!
//! Endpoint law: t=0 returns the ortho matrices EXACTLY (bitwise via f32
//! copy), t=1 the perspective ones. Mid values lerp element-wise with
//! smoothstep easing (k = t·t·(3−2t), REND-34 family). View rotation, eye,
//! and commands pass through untouched — projection-only law.

/// Smoothstep easing shared with the fly-to family (REND-34).
#[must_use]
pub fn morph_ease(t: f64) -> f64 {
    let x = t.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Element-wise matrix lerp with morph easing. k(0)=0 → `ortho` exactly,
/// k(1)=1 → `persp` exactly (endpoint canaries depend on exactness).
#[must_use]
pub fn morph_proj(ortho: &[[f32; 4]; 4], persp: &[[f32; 4]; 4], t: f64) -> [[f32; 4]; 4] {
    let k = morph_ease(t) as f32;
    let mut o = [[0.0f32; 4]; 4];
    for (i, row) in ortho.iter().enumerate() {
        for j in 0..4 {
            o[i][j] = row[j] + (persp[i][j] - row[j]) * k;
        }
    }
    o
}

/// px_world_scale lerp: ortho exact = 2·hw/W, persp = 1.0 (REND-29/41).
#[must_use]
pub fn morph_px_scale(ortho_scale: f32, t: f64) -> f32 {
    let k = morph_ease(t) as f32;
    ortho_scale + (1.0 - ortho_scale) * k
}
