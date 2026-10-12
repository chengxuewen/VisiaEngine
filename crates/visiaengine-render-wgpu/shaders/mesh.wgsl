struct Mat {
    base_color: vec4<f32>,
    repeat: vec2<f32>,
    // [8] = retired mock specular slot (WGPU-14 note; pad since N6/WGPU-35)
    roughness: f32, // GGX perceptual roughness (alpha = r*r)
    metallic: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    _pad3: f32,
};

// REND-29 delivered: View block 128B (mat 64 + right@64 up@80 px_scale@92 eye_local@96,
// WGSL alignment rules). Triangle-family entries read only view_proj's first 64B = the bitwise zero-regression construction guarantee.
struct View {
    view_proj: mat4x4<f32>,
    right: vec3<f32>,
    up: vec3<f32>,
    px_scale: f32,
    eye_local: vec3<f32>,
    _pad: f32,
    // WGPU-19: composed light matrix (same mechanism as compose_mvp, D7 full chain); when shadow=None, zero-fill +
    // enabled off + early-out at consumption -- the first 128B stay bit-for-bit in place = triangle families keep their zero-regression guarantee
    light_view_proj: mat4x4<f32>,
    // WGPU-21 section clipping (model-space plane coefficients per REND-32): dot(n,q)+w >= 0 keeps;
    // clip_count=0 (None/EMPTY always bound) = keep-all early-out = bitwise zero regression [R4 guard form].
    // Trailing 80B segment (float 44..64): the first 176B stay in place = existing construction guarantee persists.
    planes: array<vec4<f32>, 4>,
    clip_count: f32,
    _cpad0: f32,
    _cpad1: f32,
    _cpad2: f32,
    // WGPU-39 (I band): SH-9 environment irradiance tail. Uniform address
    // space requires 16-aligned array stride: 9 x vec4 (RGB + pad comp).
    // All-zero/flag-0 = legacy constant ambient (bit-identical legacy path;
    // floats 0..64 bit-order unchanged). 256 + 144 + 16 = 416 total.
    env_sh: array<vec4<f32>, 9>,
    env_on: f32,
    _epad0: f32,
    _epad1: f32,
    _epad2: f32,
};
@group(0) @binding(0) var<uniform> view: View;

// WGPU-19 shadow resource surface: params/map/sampler always bound (dummy resident) -- the dynamic enabled branch
// is not a layout branch ([E3D:B2] mega-bool rules = shader-layout domain; this note prevents misreading)
struct ShadowParams {
    a: vec4<f32>,  // (light_dir, enabled)
    b: vec4<f32>,  // (size-> penumbra ratio, texel, bias_c[stored], bias_s[stored])
};
@group(0) @binding(1) var<uniform> sp: ShadowParams;
@group(0) @binding(7) var shadow_map: texture_depth_2d;
@group(0) @binding(8) var shadow_smp: sampler_comparison;
@group(0) @binding(2) var<uniform> mat: Mat;
// Textured-layout-only slots (Flat layout omits 3/4 -- two pipelines, two layouts,
// the mega-bool branch veto stands [E3D:B2])
@group(0) @binding(3) var diffuse: texture_2d<f32>;
@group(0) @binding(4) var samp: sampler;
// Instanced-layout-only slots (WGPU-16): 32B/instance table (byte-identical to CPU Instance, REND-27 layout lock)
struct Inst {
    offset: vec3<f32>,
    height: f32,
    color: vec3<f32>,
    _pad: f32,
};
@group(0) @binding(5) var<storage, read> insts: array<Inst>;

// WGPU-17 stroke-family table (Strokes bgl exclusive binding; 48B/entry, same StrokeSeg layout lock)
struct Seg {
    a: vec4<f32>,
    b: vec4<f32>,
    color: vec3<f32>,
    width_px: f32,
};
@group(0) @binding(5) var<storage, read> segs: array<Seg>;

// WGPU-18 point table (Points bgl binding6; 32B, same PointMark layout lock)
struct Mark {
    pos: vec3<f32>,
    radius_px: f32,
    color: vec3<f32>,
    _pad: f32,
};
@group(0) @binding(6) var<storage, read> marks: array<Mark>;

// WGPU-24 label table (Labels bgl exclusive: binding11 table + 12 atlas texture + 13 atlas sampler).
// 64B = 4xvec4, byte-identical to LabelMark (REND-33 layout lock). atlas = R8 coverage, single channel.
struct Label {
    pos: vec4<f32>,       // (xyz entity-local anchor, pad)
    color: vec4<f32>,     // linear RGBA
    uv: vec4<f32>,        // (u0, v0, u1, v1)
    metrics: vec4<f32>,   // (w_px, h_px, dx right-positive, dy up-positive)
};
@group(0) @binding(11) var<storage, read> labels: array<Label>;
@group(0) @binding(12) var atlas: texture_2d<f32>;
@group(0) @binding(13) var atlas_smp: sampler;

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
    // LIGHT constants retired into the UBO [R4]: dummy params carry the default bit pattern (0.5,0.7,0.4) -> existing output unchanged bitwise
    let l = normalize(sp.a.rgb);
    // GGX PBR (WGPU-35): Cook-Torrance replaces the mock `0.35+0.65*ndl` band.
    // View vector in the same entity-local frame as eye_local (world eye − origin).
    let v = normalize(view.eye_local - in.pos);
    let intensity = length(sp.a.rgb);
    let shade = ggx_shade(mat.base_color.rgb, n, l, v, mat.roughness, mat.metallic, intensity);
    out.color = shade;
    out.uv = in.uv * mat.repeat;
    return out;
}

/// PCSS three stages [E3D:B3 port] (WGPU-20): (1) blocker search (8 rings) -> (2) similar-triangle penumbra
/// estimate -> (3) 16-tap Poisson PCF. Returns visibility in [0,1]; 1 = fully lit. map edge 1024 is
/// paired with the backend `shadow_frame_res::MAP` (any change must sync both).
/// Cook-Torrance GGX specular (WGPU-35): D/G/F with Smith-Schlick geometry and
/// Schlick fresnel; F0 = mix(vec3(0.04), base.rgb, metallic). Diffuse =
/// Lambert with energy split (1-metallic)(1-F). Returns lit linear RGB for a
/// single directional light; `ambient` = IBL-lite constant radiance term
/// (WGPU-35: vec3(0.35)*albedo*(1-roughness*0.5), probe-pinned legacy band).
fn sh_at(i: u32) -> vec3<f32> {
    // coeff i = env_sh[i].xyz (RGB; .w = pad)
    return view.env_sh[i].xyz;
}

fn env_irradiance(n: vec3<f32>) -> vec3<f32> {
    // SH-9 evaluation (L1 + L2 bands, cosine-window simplified): the classic
    // Ramamoorthi irradiance weights on the 9 coefficients. Linear-domain
    // result replaces the legacy constant when env_on == 1.0.
    // band 0 + L1 (coeffs 1..4): y/z/x directions; L2 (5..9)
    let l0 = sh_at(0u) * 0.282095;
    let l1 = sh_at(1u) * (0.488603 * n.y) + sh_at(2u) * (0.488603 * n.z)
        + sh_at(3u) * (0.488603 * n.x);
    let l2xx = sh_at(4u) * (0.315392 * (n.x * n.x - n.y * n.y));
    let l2xy = sh_at(5u) * (1.092548 * n.x * n.y);
    let l2yz = sh_at(6u) * (1.092548 * n.y * n.z);
    let l2zz = sh_at(7u) * (0.315392 * (3.0 * n.z * n.z - 1.0));
    let l2xz = sh_at(8u) * (1.092548 * n.x * n.z);
    // cosine-lobe convolution folded into the host-side projection
    // (coefficients arrive pre-convolved); demo-grade honesty.
    return max(vec3(0.0), l0 + l1 + l2xx + l2xy + l2yz + l2zz + l2xz);
}

fn ggx_shade(albedo: vec3<f32>, n: vec3<f32>, l: vec3<f32>, v: vec3<f32>,
             roughness: f32, metallic: f32, light_intensity: f32) -> vec3<f32> {
    let alpha = max(roughness * roughness, 1e-4);
    let ndv = max(dot(n, v), 1e-4);
    let ndl = max(dot(n, l), 0.0);
    let h = normalize(v + l);
    let ndh = max(dot(n, h), 0.0);
    let vdh = max(dot(v, h), 0.0);
    // D: GGX/Trowbridge-Reitz
    let a2 = alpha * alpha;
    let dnm = ndh * ndh * (a2 - 1.0) + 1.0;
    let d = a2 / (3.14159265 * dnm * dnm);
    // G: Smith-Schlick (k = alpha/2)
    let k = alpha * 0.5;
    let g1v = ndv / (ndv * (1.0 - k) + k);
    let g1l = ndl / (ndl * (1.0 - k) + k);
    let g = g1v * g1l;
    // F: Schlick fresnel
    let f0 = mix(vec3(0.04), albedo, metallic);
    let f = f0 + (1.0 - f0) * pow(clamp(1.0 - vdh, 0.0, 1.0), 5.0);
    let spec = d * g * f / (4.0 * ndv * max(ndl, 1e-4) + 1e-4);
    let diffuse = albedo * (1.0 - metallic) * (vec3(1.0) - f) * ndl;
    // WGPU-39: env SH irradiance replaces the constant when enabled; demo-
    // grade specular tint = irradiance scaled by (1-roughness) (full PMREM
    // prefilter + BRDF LUT = the separate L ticket — claim discipline).
    // WGSL: let-if expressions unsupported — var + statement form.
    var ambient = vec3(0.35) * albedo * (1.0 - roughness * 0.5);
    if view.env_on > 0.5 {
        ambient = env_irradiance(n) * albedo * (1.0 - metallic)
            + env_irradiance(reflect(-v, n)) * albedo * roughness * 0.4;
    }
    return (diffuse + spec) * light_intensity + ambient;
}

fn shadow_vis(lc: vec4<f32>) -> f32 {
    if (sp.a.w < 0.5 || lc.w <= 0.0) {
        return 1.0;
    }
    let l = lc / lc.w;
    let uv = vec2(l.x * 0.5 + 0.5, 0.5 - l.y * 0.5);
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 1.0; // outside the light cone = lit (v0 has no cascades [non-fit 4])
    }
    let z = l.z - 0.0015; // receiver-side constant bias (caster-side bias lives in the map-building pass)
    let texel = sp.b.y;
    let light_size = sp.b.x;
    // (1) blocker search: ring radius widens with light size (texel units, floor 1.5 = near-hard shadow)
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
        return 1.0; // nothing occluding = fully lit
    }
    // (2) penumbra estimate (similar-triangle ratio; uv units, cap 0.06 = blow-up guardrail [P3 boundedness])
    let blocker = bsum / bfound;
    let pen = clamp(light_size * (z - blocker) / max(blocker, 0.001) * 2.5, texel, 0.06);
    // (3) 16-tap Poisson filter
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

/// Section criterion [WGPU-21..23]: any negative side = clipped (AND combination); count=0 early-out = zero-cost always-pass.
/// Zero planes (padding slots) give dot=0 -> `<0.0` is constantly false, inherently harmless.
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


/// Instanced variant vs (WGPU-16): bottom-aligned z extrusion + instance-color multiply chain.
/// Axis-aligned box normals never flip under z scaling (sides carry no z, top stays +z) -- normals pass through with zero error [4c ruling 4].
/// Textured combination deliberately not fitted [four non-fits 2]: the material texture slot is ignored in this entry.
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
    // GGX PBR (WGPU-35), instanced variant: instance color folds into albedo
    // (multiplicative chain preserved from the mock band).
    let v = normalize(view.eye_local - p);
    let albedo = mat.base_color.rgb * inst.color;
    let intensity = length(sp.a.rgb);
    let shade = ggx_shade(albedo, n, l, v, mat.roughness, mat.metallic, intensity);
    out.color = shade;
    out.uv = in.uv;
    return out;
}

/// Stroke-expansion vs [E3D:B4 port]: side four corners -> perp = cross(view dir, axis)*halfw.
/// Degenerate (segment parallel to view) falls back to perp=right [R2] -- zero NaN tolerance (all vertices dropped = family count 0 goes red).
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

/// Color straight out (the lighting chain does not participate [4de ruling point a] -- line color = what you see (GIS convention)).
@fragment
fn fs_stroke(in: FsIn) -> @location(0) vec4<f32> {
    if (clipped(in.wpos)) {
        discard;
    }
    return vec4<f32>(in.color, 1.0);
}

/// Point splat vs: screen-basis expansion (right/up x radius_px x px_scale) -- true round dots, route A (FS distance discards).
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

/// Circle mask: outside the unit disc in local space is discarded (the square-dot correction); alpha=1, color straight out.
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

/// Label vs (WGPU-24): world-anchored quad with constant screen size (px x px_scale spread along right/up).
/// Always on top: fs writes a fixed near depth + last command order (overwrite semantics [ruling point e], not occlusion).
@vertex
fn vs_label(in: QuadIn, @builtin(instance_index) ii: u32) -> FsIn {
    var out: FsIn;
    let l = labels[ii];
    let w = l.metrics.x; // glyph width px
    let h = l.metrics.y; // glyph height px
    let dx = l.metrics.z; // top-left offset from anchor (right positive)
    let dy = l.metrics.w; // top-left offset from anchor (up positive)
    // side.x=-1 -> left(u=0,+dx), +1 -> right(u=1,+dx+w); side.y=+1 -> top(v=0), -1 -> bottom(v=1,+dy-h).
    let offx = dx + (in.side.x * 0.5 + 0.5) * w;
    let offy = dy - (0.5 - in.side.y * 0.5) * h;
    let p = l.pos.xyz + view.right * (offx * view.px_scale) + view.up * (offy * view.px_scale);
    out.pos = view.view_proj * vec4<f32>(p, 1.0);
    out.light_clip = vec4<f32>(0.0);
    out.wpos = l.pos.xyz; // WGPU-25 anchor test: the whole label lives or dies together (fs clips at the anchor, not per pixel)
    // uv: side.x=-1->u0, +1->u1; side.y=+1(top)->v0, -1(bottom)->v1
    let tu = (in.side.x + 1.0) * 0.5;
    let tv = (1.0 - in.side.y) * 0.5;
    out.uv = mix(l.uv.xy, l.uv.zw, vec2<f32>(tu, tv));
    out.color = l.color.rgb;
    return out;
}

/// Label fs: atlas coverage single channel x table color, linear straight out (no lighting chain [same family as ruling point a]).
@fragment
fn fs_label(in: FsIn) -> @location(0) vec4<f32> {
    if (clipped(in.wpos)) {
        discard; // anchor on the negative side = whole label dies [WGPU-25]
    }
    let cov = textureSample(atlas, atlas_smp, in.uv).r;
    if (cov < 0.02) {
        discard;
    }
    return vec4<f32>(in.color * cov, cov);
}

/// Flat variant (shadow off = `x mix(0.25,1,1.0) = x1.0` identity per bit = zero regression persists).
@fragment
fn fs(in: FsIn) -> @location(0) vec4<f32> {
    if (clipped(in.wpos)) {
        discard;
    }
    let vis = shadow_vis(in.light_clip);
    return vec4<f32>(in.color * mix(0.25, 1.0, vis), mat.base_color.a);
}

/// Textured variant: sample x base x shade (WGPU-14 luminance chain).
@fragment
fn fs_textured(in: FsIn) -> @location(0) vec4<f32> {
    if (clipped(in.wpos)) {
        discard;
    }
    let t = textureSample(diffuse, samp, in.uv);
    let vis = shadow_vis(in.light_clip);
    return vec4<f32>(in.color * t.rgb * mix(0.25, 1.0, vis), mat.base_color.a * t.a);
}

/// WGPU-19 caster pass: light-space position written straight out (no color target; depth is the output).
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
    // WGPU-22 same clip in the caster: section-clipped geometry must not cast (receiver clears depth = that region reads lit; behavior correct)
    if (clipped(in.wpos)) {
        discard;
    }
}
