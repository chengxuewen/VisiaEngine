// Bloom post-pass (WGPU-34, band N4): bright-pass + 9-tap blur + additive
// composite in ONE fullscreen pass (ponytail: single-pass approximation —
// no separable two-pass chain until profiled; upgrade path = two half-res
// blur passes feeding this composite).
//
// fs samples a 3x3 color neighborhood, keeps pixels whose luma exceeds a
// fixed threshold, averages the kept contributions, and ADDS strength-scaled
// bloom on top of the base color. params = (texel_w, texel_h, strength, 0);
// threshold is a compile-time constant (THRESHOLD below) so the constructor
// domain stays single-knob (strength).

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
@group(0) @binding(1) var depth_tex: texture_depth_2d;  // bound for bgl-shape sharing; unused by bloom
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> params: vec4<f32>; // (texel_w, texel_h, strength, 0)

// Luma threshold on SAMPLED (linear-decoded, sRGB-view) color. Probe-pinned
// (lavapipe, city test scene): bright quad face decodes to luma ~0.25-0.4
// (sum 413/3 = 138 sRGB), ground ~0.04, clear ~0.005. 0.25 keeps the quad +
// its blur taps, rejects scene content.
const THRESHOLD: f32 = 0.25;

const NEIGHBORS: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0,  0.0),                       vec2<f32>(1.0,  0.0),
    vec2<f32>(-1.0,  1.0), vec2<f32>(0.0,  1.0), vec2<f32>(1.0,  1.0),
);

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let texel = vec2<f32>(params.x, params.y);
    let base = textureSampleLevel(color_tex, samp, in.uv, 0.0);

    var bright_sum = vec3<f32>(0.0, 0.0, 0.0);
    var bright_n = 0.0;
    let center_l = dot(base.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if (center_l > THRESHOLD) {
        bright_sum += base.rgb;
        bright_n += 1.0;
    }
    for (var i = 0; i < 8; i++) {
        let c = textureSampleLevel(color_tex, samp, in.uv + NEIGHBORS[i] * texel, 0.0);
        let l = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        if (l > THRESHOLD) {
            bright_sum += c.rgb;
            bright_n += 1.0;
        }
    }
    // 9-tap neighborhood blur of the kept bright pixels (>=1: center passed
    // when any accumulation happened). strength scales the additive term.
    let bloom = select(vec3<f32>(0.0), bright_sum / max(bright_n, 1.0), bright_n > 0.0);
    return vec4<f32>(base.rgb + bloom * params.z, base.a);
}
