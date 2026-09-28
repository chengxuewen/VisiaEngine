// Tone-mapping post-pass (REND-44, band N5): FIRST consumer of the N4
// framework (WGPU-34) beyond bloom/outline. Fullscreen pass, samples the
// chain color, applies a global curve by params.x mode selector:
//   mode 0 = Reinhard c/(1+c)
//   mode 1 = ACES (Narkowicz filmic approx, "ACES Applied" blog)
// params = (mode_f32, 0, 0, 0); depth bound for bgl-shape sharing, unused.
//
// Honesty note (clause REND-44): the pipeline renders LDR (0..1 post-sRGB
// encode); tonemap on LDR input = contrast reshape (mood), not true HDR
// display — true HDR needs the float-texture path (ticket).

// Vertex: fullscreen triangle strip from vertex_index (bloom/outline same shape).
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
@group(0) @binding(1) var depth_tex: texture_depth_2d; // bound for bgl-shape sharing; unused
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> params: vec4<f32>; // (mode, 0, 0, 0)

fn reinhard(c: vec3<f32>) -> vec3<f32> {
    let one = vec3<f32>(1.0, 1.0, 1.0);
    return c / (c + one);
}

fn aces_narkowicz(x: vec3<f32>) -> vec3<f32> {
    // Narkowicz 2015: a=x*(2.51x+0.03) / (x*(2.43x+0.59)+0.14), componentwise.
    // naga (wgpu 30) parses `2.51 * x` fine but rejects scalar-vector mixed
    // paren chains in some shapes (N4 archive lesson) — keep everything
    // vec3-typed explicitly.
    let two = vec3<f32>(2.51, 2.51, 2.51);
    let k1 = vec3<f32>(0.03, 0.03, 0.03);
    let k2 = vec3<f32>(2.43, 2.43, 2.43);
    let k3 = vec3<f32>(0.59, 0.59, 0.59);
    let k4 = vec3<f32>(0.14, 0.14, 0.14);
    let a = x * (two * x + k1);
    let b = x * (k2 * x + k3) + k4;
    return a / b;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let c = textureSampleLevel(color_tex, samp, in.uv, 0.0).rgb;
    let mapped = select(reinhard(c), aces_narkowicz(c), params.x > 0.5);
    return vec4<f32>(mapped, 1.0);
}
