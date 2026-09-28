// Outline post-pass (WGPU-34, band N4): depth-edge silhouette detection with
// selection-orange overlay. Same math family as EDL (edl.wgsl): 8-neighbor
// depth gradient via textureLoad; an edge pixel = center depth far from the
// neighborhood average relative to its own distance from the background wall
// (scale-invariant ratio, WGPU-33 probe-earned form). Edge pixels blend a
// fixed selection-orange [1.0, 0.83, 0.29] at `strength`; params =
// (texel_w, texel_h, strength, 0). `width` (Outline field) widens the detect
// radius in texels — v1 uses integer radius via params.w-lowered loop bound.

// Vertex: fullscreen triangle strip from vertex_index (edl.wgsl same shape).
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
@group(0) @binding(3) var<uniform> params: vec4<f32>; // (texel_w, texel_h, strength, width)

const NEIGHBORS: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0,  0.0),                       vec2<f32>(1.0,  0.0),
    vec2<f32>(-1.0,  1.0), vec2<f32>(0.0,  1.0), vec2<f32>(1.0,  1.0),
);

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(color_tex));
    let radius = max(params.w, 1.0);
    let texel = vec2<f32>(params.x, params.y) * radius;
    let depth_center = textureLoad(depth_tex, vec2<i32>(in.uv * dims), 0);
    var depth_sum = 0.0;
    for (var i = 0; i < 8; i++) {
        let off = NEIGHBORS[i] * texel;
        depth_sum += textureLoad(depth_tex, vec2<i32>((in.uv + off) * dims), 0);
    }
    let depth_avg = depth_sum / 8.0;
    let grad = abs(depth_avg - depth_center);

    // Scale-invariant edge ratio (WGPU-33 family). Content-vs-background at
    // typical near/far: interior < 0.05, silhouette 0.2-0.5. Background
    // pixels (depth >= 0.999) never outlined — no full-frame halo.
    let is_bg = depth_center > 0.999;
    let rel = grad / max(1.0 - depth_center, 1e-4);
    let edge = rel > 0.08;

    let base = textureSampleLevel(color_tex, samp, in.uv, 0.0);
    let orange = vec3<f32>(1.0, 0.83, 0.29);
    let mixed = mix(base.rgb, orange, clamp(params.z, 0.0, 1.0));
    let out_rgb = select(base.rgb, mixed, edge && !is_bg);
    return vec4<f32>(out_rgb, base.a);
}
