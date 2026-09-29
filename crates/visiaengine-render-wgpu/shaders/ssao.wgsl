// SSAO post-pass (WGPU-36, band N7): depth-only screen-space ambient
// occlusion. 16 taps in a screen-space disc of `radius` texels; a tap
// occludes when its geometry sits IN FRONT of the center surface within a
// small depth window (0.02..3.0 view-z meters — near-contact / crease band;
// far jumps like silhouette-vs-sky don't count). Occluded fraction darkens
// the pixel: ao = 1 - min(occ, 16)/16 * intensity, per-tap distance falloff
// (center taps weigh full, edge taps half). Background (view-z > 500 m)
// never darkens — no halo (EDL/outline family law). out = vec4(rgb*ao, a).
//
// Depth linearization: view_z() assumes the post-chain demo/test projection
// range near=0.1 / far=1000 (same constants as every example Frame helper).
// Honesty note (clause WGPU-36): depth-only AO — no normal buffer, no blur
// pass — v1 quality tier; HBAO / normal-buffer / separable-blur upgrade =
// ticket.
//
// params = (texel_w, texel_h, radius, intensity). Explicit returns (WGSL is
// not Rust — no trailing expressions; N5 archive lesson).

// Vertex: fullscreen triangle strip from vertex_index (bloom/outline/tonemap
// same shape).
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
@group(0) @binding(3) var<uniform> params: vec4<f32>; // (texel_w, texel_h, radius, intensity)

// 16-tap golden-angle spiral (~2.4 rad step): even angular coverage of the
// radius disc without directional banding.
const TAPS: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    vec2<f32>(1.000, 0.000), vec2<f32>(-0.415, 0.910), vec2<f32>(-0.654, -0.756), vec2<f32>(0.997, -0.074),
    vec2<f32>(-0.980, 0.199), vec2<f32>(0.242, 0.970), vec2<f32>(0.596, -0.803), vec2<f32>(-0.973, -0.231),
    vec2<f32>(0.525, 0.851), vec2<f32>(-0.025, -0.999), vec2<f32>(-0.566, 0.824), vec2<f32>(0.986, 0.168),
    vec2<f32>(-0.590, -0.807), vec2<f32>(0.069, 0.997), vec2<f32>(0.637, -0.770), vec2<f32>(-0.998, -0.061),
);

// Demo/test projection range (matches the Frame helpers in examples/tests).
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 1000.0;
// Occluder window in view-z meters: nearer than this counts as touching;
// farther than this = distant geometry, not ambient occlusion.
const Z_WINDOW_NEAR: f32 = 0.02;
const Z_WINDOW_FAR: f32 = 3.0;
// Background depth gate: beyond this view-z nothing darkens (no halo).
const Z_BACKGROUND: f32 = 500.0;

fn view_z(d: f32) -> f32 {
    return (Z_NEAR * Z_FAR) / (Z_FAR - d * (Z_FAR - Z_NEAR));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let dims = vec2<f32>(textureDimensions(color_tex));
    let center = vec2<i32>(in.uv * dims);
    let center_z = view_z(textureLoad(depth_tex, center, 0));
    let step = vec2<f32>(params.x, params.y) * params.z;
    var occluded = 0.0;
    for (var i = 0; i < 16; i++) {
        let off = TAPS[i] * step;
        let tap = vec2<i32>((in.uv + off) * dims);
        let z = view_z(textureLoad(depth_tex, tap, 0));
        let falloff = 1.0 - 0.5 * length(TAPS[i]);
        let in_front = center_z - z;
        if (in_front > Z_WINDOW_NEAR && in_front < Z_WINDOW_FAR) {
            occluded = occluded + falloff;
        }
    }
    let ao = 1.0 - min(occluded, 16.0) / 16.0 * params.w;
    let base = textureSampleLevel(color_tex, samp, in.uv, 0.0);
    let is_bg = center_z > Z_BACKGROUND;
    let rgb = select(base.rgb * ao, base.rgb, is_bg);
    return vec4<f32>(rgb, base.a);
}
