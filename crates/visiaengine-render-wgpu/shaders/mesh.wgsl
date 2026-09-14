struct Mat {
    base_color: vec4<f32>,
    repeat: vec2<f32>,
    specular: f32, // mock-up [4ab①/WGPU-14]：参与 Lambert 亮度系数（非 GGX，真 PBR=独立轮）
    _pad: f32,
};

@group(0) @binding(0) var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(2) var<uniform> mat: Mat;
// Textured layout 专属槽位（Flat layout 不含 3/4——两 pipeline 两 layout，
// mega-bool 分支否决 [E3D:B2]）
@group(0) @binding(3) var diffuse: texture_2d<f32>;
@group(0) @binding(4) var samp: sampler;
// Instanced layout 专属槽位（WGPU-16）：32B/条实例表（CPU Instance 逐字节同形，REND-27 布局锁）
struct Inst {
    offset: vec3<f32>,
    height: f32,
    color: vec3<f32>,
    _pad: f32,
};
@group(0) @binding(5) var<storage, read> insts: array<Inst>;

struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct FsIn {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

const LIGHT: vec3<f32> = vec3<f32>(0.5, 0.7, 0.4);

@vertex
fn vs(in: VsIn) -> FsIn {
    var out: FsIn;
    out.pos = view_proj * vec4<f32>(in.pos, 1.0);
    let n = normalize(in.normal);
    let l = normalize(LIGHT);
    // mock-up [4ab①/WGPU-14]：specular 参与 Lambert 亮度系数（非 GGX）。
    // 存量材质 specular=0 → `s + 0.0` 逐位恒等=Flat 零回归不受累。
    let ndl = max(dot(n, l), 0.0);
    let shade = 0.35 + 0.65 * ndl + mat.specular * ndl;
    out.color = mat.base_color.rgb * shade;
    out.uv = in.uv * mat.repeat;
    return out;
}

/// Instanced 变体 vs（WGPU-16）：底对齐 z 挤出 + 实例色乘法链。
/// 轴对齐盒法向在 z 缩放下不变向（侧面无 z 分量、顶面恒 +z）——法线直传零误差 [4c 裁决④]。
/// 与 Textured 组合不装 [四不装②]：材质 texture 位在本入口忽略。
@vertex
fn vs_inst(in: VsIn, @builtin(instance_index) ii: u32) -> FsIn {
    var out: FsIn;
    let inst = insts[ii];
    let p = vec3<f32>(in.pos.x, in.pos.y, in.pos.z * inst.height) + inst.offset;
    out.pos = view_proj * vec4<f32>(p, 1.0);
    let n = normalize(in.normal);
    let l = normalize(LIGHT);
    let ndl = max(dot(n, l), 0.0);
    let shade = 0.35 + 0.65 * ndl + mat.specular * ndl;
    out.color = mat.base_color.rgb * inst.color * shade;
    out.uv = in.uv;
    return out;
}

/// Flat 变体（现状逐像素一致：alpha 式、光照、乘法序全保留）。
@fragment
fn fs(in: FsIn) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, mat.base_color.a);
}

/// Textured 变体：采样×base×shade（WGPU-14 亮度链）。
@fragment
fn fs_textured(in: FsIn) -> @location(0) vec4<f32> {
    let t = textureSample(diffuse, samp, in.uv);
    return vec4<f32>(in.color * t.rgb, mat.base_color.a * t.a);
}
