struct Mat {
    base_color: vec4<f32>,
    repeat: vec2<f32>,
    specular: f32, // mock-up [4ab①/WGPU-14]：参与 Lambert 亮度系数（非 GGX，真 PBR=独立轮）
    _pad: f32,
};

// REND-29 兑现：View 块 128B（mat 64 + right@64 up@80 px_scale@92 eye_local@96，
// WGSL 对齐规约）。三角系入口只读 view_proj 前 64B=逐位零回归构造保证。
struct View {
    view_proj: mat4x4<f32>,
    right: vec3<f32>,
    up: vec3<f32>,
    px_scale: f32,
    eye_local: vec3<f32>,
    _pad: f32,
    // WGPU-19：光源合成阵（compose_mvp 同机件，D7 全链）；shadow=None 时零填+
    // enabled 关闭+消费点早退——前 128B 位序不变=三角系零回归构造保证续存
    light_view_proj: mat4x4<f32>,
    // WGPU-21 剖面裁切（REND-32 换算的 model-space 面系数）：dot(n,q)+w ≥ 0 保留；
    // clip_count=0（None/EMPTY 全零恒绑）=全保留早退=逐位零回归 [R4 护栏形]。
    // 尾缀 80B 段（float 44..64）：前 176B 位序不变=存量构造保证续存。
    planes: array<vec4<f32>, 4>,
    clip_count: f32,
    _cpad0: f32,
    _cpad1: f32,
    _cpad2: f32,
};
@group(0) @binding(0) var<uniform> view: View;

// WGPU-19 shadow 资源面：params/map/sampler 恒绑（dummy 常驻）——动态 enabled 分支
// 非布局分支（[E3D:B2] mega-bool 令辖=着色布局域，注记防误读）
struct ShadowParams {
    a: vec4<f32>,  // (light_dir, enabled)
    b: vec4<f32>,  // (size→半影比例, texel, bias_c[收纳], bias_s[收纳])
};
@group(0) @binding(1) var<uniform> sp: ShadowParams;
@group(0) @binding(7) var shadow_map: texture_depth_2d;
@group(0) @binding(8) var shadow_smp: sampler_comparison;
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

// WGPU-17 扩片族表（Strokes bgl 专属 binding；48B/条同 StrokeSeg 布局锁）
struct Seg {
    a: vec4<f32>,
    b: vec4<f32>,
    color: vec3<f32>,
    width_px: f32,
};
@group(0) @binding(5) var<storage, read> segs: array<Seg>;

// WGPU-18 点表（Points bgl binding6；32B 同 PointMark 布局锁）
struct Mark {
    pos: vec3<f32>,
    radius_px: f32,
    color: vec3<f32>,
    _pad: f32,
};
@group(0) @binding(6) var<storage, read> marks: array<Mark>;

struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct FsIn {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) light_clip: vec4<f32>,
    @location(3) wpos: vec3<f32>,
};

@vertex
fn vs(in: VsIn) -> FsIn {
    var out: FsIn;
    out.pos = view.view_proj * vec4<f32>(in.pos, 1.0);
    out.light_clip = view.light_view_proj * vec4<f32>(in.pos, 1.0);
    out.wpos = in.pos;
    let n = normalize(in.normal);
    // LIGHT 常数退役入 UBO [R4]：dummy params 载默认位型 (0.5,0.7,0.4) → 存量逐位不变
    let l = normalize(sp.a.rgb);
    // mock-up [4ab①/WGPU-14]：specular 参与 Lambert 亮度系数（非 GGX）。
    // 存量材质 specular=0 → `s + 0.0` 逐位恒等=Flat 零回归不受累。
    let ndl = max(dot(n, l), 0.0);
    let shade = 0.35 + 0.65 * ndl + mat.specular * ndl;
    out.color = mat.base_color.rgb * shade;
    out.uv = in.uv * mat.repeat;
    return out;
}

/// PCSS 三阶段 [E3D:B3 移植]（WGPU-20）：① blocker 搜索(8 环)→② 相似三角形半影
/// 估计→③ 16-tap 泊松 PCF。返回可见度 ∈[0,1]；1=全亮。map 边长 1024 与后端
/// `shadow_frame_res::MAP` 成对耦合（改动必同步两处）。
fn shadow_vis(lc: vec4<f32>) -> f32 {
    if (sp.a.w < 0.5 || lc.w <= 0.0) {
        return 1.0;
    }
    let l = lc / lc.w;
    let uv = vec2(l.x * 0.5 + 0.5, 0.5 - l.y * 0.5);
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 1.0; // 光锥外=判亮（v0 无级联 [不装④]）
    }
    let z = l.z - 0.0015; // 接收端常数偏置（caster 侧 bias 在建图 pass）
    let texel = sp.b.y;
    let light_size = sp.b.x;
    // ① blocker 搜索：环半径随光源尺寸放大（texel 单位，下限 1.5=近硬影）
    let r1 = max(1.5, light_size * 220.0 * (1.0 - z));
    let off8 = array<vec2<f32>, 8>(
        vec2(1.0, 0.0), vec2(0.71, 0.71), vec2(0.0, 1.0), vec2(-0.71, 0.71),
        vec2(-1.0, 0.0), vec2(-0.71, -0.71), vec2(0.0, -1.0), vec2(0.71, -0.71),
    );
    var bsum = 0.0;
    var bfound = 0.0;
    for (var i = 0; i < 8; i = i + 1) {
        let o = off8[u32(i)] * r1;
        let c = vec2<i32>(
            i32(round((uv.x + o.x * texel) * 1024.0)),
            i32(round((uv.y + o.y * texel) * 1024.0)),
        );
        let d = textureLoad(shadow_map, c, 0);
        if (d < z) {
            bsum = bsum + d;
            bfound = bfound + 1.0;
        }
    }
    if (bfound < 1.0) {
        return 1.0; // 无遮挡者=全亮
    }
    // ② 半影估计（相似三角形比例；uv 单位，上限 0.06=爆炸护栏 [P3 有界性]）
    let blocker = bsum / bfound;
    let pen = clamp(light_size * (z - blocker) / max(blocker, 0.001) * 2.5, texel, 0.06);
    // ③ 16-tap 泊松过滤
    let off16 = array<vec2<f32>, 16>(
        vec2(-0.94201624, -0.39906216), vec2(0.94558609, -0.76890725),
        vec2(-0.09418410, -0.92938870), vec2(0.34495938, 0.29387760),
        vec2(-0.91588032, 0.45771432), vec2(-0.81544000, -0.87912430),
        vec2(-0.38277543, 0.27676845), vec2(0.97484398, 0.75966610),
        vec2(0.44344293, -0.97560616), vec2(0.93747580, 0.61648620),
        vec2(-0.14020140, 0.91047400), vec2(-0.79791000, -0.31438850),
        vec2(-0.65675550, 0.74012000), vec2(0.78506600, -0.44712450),
        vec2(-0.25946690, -0.09117550), vec2(0.40687080, -0.45018580),
    );
    var acc = 0.0;
    for (var i = 0; i < 16; i = i + 1) {
        let o = off16[u32(i)] * pen;
        acc = acc + textureSampleCompareLevel(shadow_map, shadow_smp, uv + o, z);
    }
    return acc / 16.0;
}

/// 剖面判据 [WGPU-21..23]：任一负侧=裁（AND 组合）；count=0 早退=零成本恒通。
/// 零平面（padding 位）dot=0 → `<0.0` 恒假天然无害。
fn clipped(p: vec3<f32>) -> bool {
    if (view.clip_count < 0.5) {
        return false;
    }
    for (var i = 0; i < 4; i = i + 1) {
        let pl = view.planes[u32(i)];
        if (dot(pl.xyz, p) + pl.w < 0.0) {
            return true;
        }
    }
    return false;
}


/// Instanced 变体 vs（WGPU-16）：底对齐 z 挤出 + 实例色乘法链。
/// 轴对齐盒法向在 z 缩放下不变向（侧面无 z 分量、顶面恒 +z）——法线直传零误差 [4c 裁决④]。
/// 与 Textured 组合不装 [四不装②]：材质 texture 位在本入口忽略。
@vertex
fn vs_inst(in: VsIn, @builtin(instance_index) ii: u32) -> FsIn {
    var out: FsIn;
    let inst = insts[ii];
    let p = vec3<f32>(in.pos.x, in.pos.y, in.pos.z * inst.height) + inst.offset;
    out.pos = view.view_proj * vec4<f32>(p, 1.0);
    out.light_clip = view.light_view_proj * vec4<f32>(p, 1.0);
    out.wpos = p;
    let n = normalize(in.normal);
    let l = normalize(sp.a.rgb);
    let ndl = max(dot(n, l), 0.0);
    let shade = 0.35 + 0.65 * ndl + mat.specular * ndl;
    out.color = mat.base_color.rgb * inst.color * shade;
    out.uv = in.uv;
    return out;
}

/// 线段扩片 vs [E3D:B4 移植]：side 四角 → perp = cross(视向, 轴)·halfw。
/// 退化（段∥视向）兜底 perp=right [R2]——NaN 零容忍（顶点全弃=族 0 即红）。
struct QuadIn {
    @location(0) side: vec2<f32>,
};

@vertex
fn vs_stroke(in: QuadIn, @builtin(instance_index) ii: u32) -> FsIn {
    var out: FsIn;
    let s = segs[ii];
    let axis = normalize(s.b.xyz - s.a.xyz);
    let mid = (s.a.xyz + s.b.xyz) * 0.5;
    let v = normalize(view.eye_local - mid);
    var perp = cross(v, axis);
    if (abs(dot(v, axis)) > 0.999) {
        perp = view.right;
    } else {
        perp = normalize(perp);
    }
    let halfw = 0.5 * s.width_px * view.px_scale;
    let along = mid + axis * (in.side.x * 0.5 * distance(s.b.xyz, s.a.xyz));
    let p = along + perp * (in.side.y * halfw);
    out.pos = view.view_proj * vec4<f32>(p, 1.0);
    out.light_clip = vec4<f32>(0.0);
    out.wpos = p;
    out.color = s.color;
    out.uv = in.side;
    return out;
}

/// 色直出（光照链不参与 [4de 裁决点 a]——线色=所见（GIS 约定））。
@fragment
fn fs_stroke(in: FsIn) -> @location(0) vec4<f32> {
    if (clipped(in.wpos)) {
        discard;
    }
    return vec4<f32>(in.color, 1.0);
}

/// 点 splat vs：屏幕基展开（right/up×radius_px×px_scale）——真圆点的路 A（FS 距离弃片）。
@vertex
fn vs_point(in: QuadIn, @builtin(instance_index) ii: u32) -> FsIn {
    var out: FsIn;
    let m = marks[ii];
    let r = m.radius_px * view.px_scale;
    let p = m.pos + view.right * (in.side.x * r) + view.up * (in.side.y * r);
    out.pos = view.view_proj * vec4<f32>(p, 1.0);
    out.light_clip = vec4<f32>(0.0);
    out.wpos = p;
    out.color = m.color;
    out.uv = in.side;
    return out;
}

/// 圆 mask：local 单位盘外弃片（方块纠案）；alpha=1 色直出。
@fragment
fn fs_point(in: FsIn) -> @location(0) vec4<f32> {
    if (clipped(in.wpos)) {
        discard;
    }
    if (length(in.uv) > 1.0) {
        discard;
    }
    return vec4<f32>(in.color, 1.0);
}

/// Flat 变体（shadow 关闭=`× mix(0.25,1,1.0)=×1.0` 逐位恒等=零回归续存）。
@fragment
fn fs(in: FsIn) -> @location(0) vec4<f32> {
    if (clipped(in.wpos)) {
        discard;
    }
    let vis = shadow_vis(in.light_clip);
    return vec4<f32>(in.color * mix(0.25, 1.0, vis), mat.base_color.a);
}

/// Textured 变体：采样×base×shade（WGPU-14 亮度链）。
@fragment
fn fs_textured(in: FsIn) -> @location(0) vec4<f32> {
    if (clipped(in.wpos)) {
        discard;
    }
    let t = textureSample(diffuse, samp, in.uv);
    let vis = shadow_vis(in.light_clip);
    return vec4<f32>(in.color * t.rgb * mix(0.25, 1.0, vis), mat.base_color.a * t.a);
}

/// WGPU-19 caster pass：光空间位置直写（无色彩目标，深度即输出）。
@vertex
fn vs_shadow(in: VsIn) -> FsIn {
    var out: FsIn;
    out.pos = view.light_view_proj * vec4<f32>(in.pos, 1.0);
    out.wpos = in.pos;
    out.color = vec3<f32>(0.0);
    out.uv = vec2<f32>(0.0);
    out.light_clip = vec4<f32>(0.0);
    return out;
}

@vertex
fn vs_shadow_inst(in: VsIn, @builtin(instance_index) ii: u32) -> FsIn {
    var out: FsIn;
    let inst = insts[ii];
    let p = vec3<f32>(in.pos.x, in.pos.y, in.pos.z * inst.height) + inst.offset;
    out.pos = view.light_view_proj * vec4<f32>(p, 1.0);
    out.wpos = p;
    out.color = vec3<f32>(0.0);
    out.uv = vec2<f32>(0.0);
    out.light_clip = vec4<f32>(0.0);
    return out;
}

@fragment
fn fs_shadow(in: FsIn) {
    // WGPU-22 caster 同裁：剖掉的投影物不得留影（接收端清深度=该区判亮，行为正确）
    if (clipped(in.wpos)) {
        discard;
    }
}
