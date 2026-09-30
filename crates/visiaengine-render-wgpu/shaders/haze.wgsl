// WGPU-38: screen-space depth haze (W band). Fullscreen pass over the
// post-chain stage color, blending toward a constant haze color by LINEAR
// view distance (meters). Raw depth in the [0,1] RH domain is exponentially
// squeezed under perspective (16 m ≈ 0.994 at near=0.1/far=1000) — a ramp
// on raw depth would swallow the whole scene (probe-verified, PIT-8); the
// SSAO family's view_z() linearization (near=0.1/far=1000 constants, same
// demo/test projection range) converts to meters so far_start/far_full are
// author-meaningful. Background (no geometry) = depth 1.0 = view_z 1000 m =
// always fully hazed (sky takes the tint, matching the E506 probe intent).
// Screen-space ONLY: near pixels pass through unchanged (two-sided clause
// test); no world-space fog, no per-material uniforms — scene-space
// Frame.fog is a separate ticket (roadmap ledger). Explicit returns (WGSL
// is not Rust — N5 archive lesson).
//
// params = (texel_w, texel_h, far_start_m, far_full_m).

const HAZE_COLOR: vec3<f32> = vec3<f32>(0.72, 0.78, 0.85); // sky-lit haze tint (linear domain)
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 1000.0;

fn view_z(d: f32) -> f32 {
    return (Z_NEAR * Z_FAR) / (Z_FAR - d * (Z_FAR - Z_NEAR));
}

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    var out: VsOut;
    let x = f32(i32(vi) & 1) * 2.0;
    let y = f32(i32(vi) >> 1) * 2.0;
    out.pos = vec4<f32>(vec2<f32>(-1.0 + x, -1.0 + y), 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5, 1.0 - y * 0.5);
    return out;
}

@group(0) @binding(0) var color_tex: texture_2d<f32>;
@group(0) @binding(1) var depth_tex: texture_depth_2d;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> params: vec4<f32>; // (texel_w, texel_h, far_start, far_full)

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let scene = textureSample(color_tex, samp, in.uv).rgb;
    let d = textureSample(depth_tex, samp, in.uv); // depth2D sample = scalar
    // view-z meters (SSAO-family linearization); background = 1000 = full haze.
    let z = view_z(clamp(d, 0.0, 1.0));
    let t = smoothstep(params.z, params.w, z);
    return vec4<f32>(mix(scene, HAZE_COLOR, vec3<f32>(t)), 1.0);
}
