// EDL post-pass (WGPU-33, band N1.5): eye-dome lighting for point clouds.
//
// Fullscreen pass over the main-pass color+depth attachments. Edge pixels
// (depth gradient between neighbors) get darkened — classic eye-dome
// lightening inverted to darkening keeps the sRGB-blend pipeline untouched
// (we darken the sampled color instead of lightening, so no additive pass
// and no ordering hazard; visual goal = legibility of point silhouettes).

// Vertex: fullscreen triangle strip from vertex_index.
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i32(vi) & 1) * 2.0;      // 0, 2, 0
    let y = f32(i32(vi) >> 1) * 2.0;     // 0, 0, 2
    out.pos = vec4<f32>(vec2<f32>(-1.0 + x, -1.0 + y), 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5, 1.0 - y * 0.5);
    return out;
}

@group(0) @binding(0) var color_tex: texture_2d<f32>;
@group(0) @binding(1) var depth_tex: texture_depth_2d;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> params: vec4<f32>; // (texel_w, texel_h, strength, pad)

const NEIGHBORS: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0,  0.0),                       vec2<f32>(1.0,  0.0),
    vec2<f32>(-1.0,  1.0), vec2<f32>(0.0,  1.0), vec2<f32>(1.0,  1.0),
);

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(color_tex));
    let texel = vec2<f32>(1.0) / dims;
    let depth_center = textureLoad(depth_tex, vec2<i32>(in.uv * dims), 0);
    var depth_sum = 0.0;
    for (var i = 0; i < 8; i++) {
        let off = NEIGHBORS[i] * texel;
        depth_sum += textureLoad(depth_tex, vec2<i32>((in.uv + off) * dims), 0);
    }
    let depth_avg = depth_sum / 8.0;

    // Point pixels sit near the far plane (points don't write near depths);
    // background = 1.0. An edge pixel = center-on-content vs background
    // neighbors (or vice versa): |avg - center| large.
    let grad = abs(depth_avg - depth_center);

    let base = textureSampleLevel(color_tex, samp, in.uv, 0.0);
    // WGPU-33P1 (lavapipe probe; probe outputs were sRGB-encoded — decoded
    // first): interior grad≈3e-4 (smooth splat face), silhouette-edge
    // grad≈2e-3..5e-3 (splat depth ≈0.99 vs background 1.0 at this
    // near/far). Absolute thresholds are scene-brittle, so normalize by the
    // pixel's own distance from the background wall: rel=grad/(1-depth)
    // ≈0.03 interior vs ≈0.2-0.5 edge (scale-invariant ratio). Background
    // pixels (depth≥0.999) are never darkened — no halo.
    let is_bg = depth_center > 0.999;
    let rel = grad / max(1.0 - depth_center, 1e-4);
    // Darken edges: strength scales the dip; clamp to avoid over-darkening.
    let k = clamp(rel * params.z * 3.0, 0.0, 0.7);
    let shaded = select(mix(base.rgb, vec3<f32>(0.0, 0.0, 0.0), k), base.rgb, is_bg);
    return vec4<f32>(shaded, base.a);
}
