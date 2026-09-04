@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(2) var<uniform> base_color: vec4<f32>;

struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct FsIn {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

const LIGHT: vec3<f32> = vec3<f32>(0.5, 0.7, 0.4);

@vertex
fn vs(in: VsIn) -> FsIn {
    var out: FsIn;
    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    let n = normalize(in.normal);
    let l = normalize(LIGHT);
    let shade = 0.35 + 0.65 * max(dot(n, l), 0.0);
    out.color = base_color.rgb * shade;
    return out;
}

@fragment
fn fs(in: FsIn) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, base_color.a);
}
