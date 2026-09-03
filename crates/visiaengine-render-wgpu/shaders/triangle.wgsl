struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct FsIn {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs(in: VsIn) -> FsIn {
    var out: FsIn;
    out.pos = vec4<f32>(in.pos, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs(in: FsIn) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
