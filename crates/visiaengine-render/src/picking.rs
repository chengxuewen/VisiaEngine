//! 场景级拾取编排（REND-23/24）：候选 mesh 集 + 射线 → 最近命中。
//! bbox **即算不常存**（[E3D:A5] 懒算语义；升级位=BVH，触发=bench 证据）。
//! 几何查找闭包注入模式弃用——数据参数化（调用方持 scene↔mesh 关联，
//! 引擎零新状态；Scene 现无几何组件，关联归宿主/几何组件片）。
//! B2 补充：点云屏幕空间拾取（REND-38/39）——world→clip 投影最近点谓词。
//! bbox **即算不常存**（[E3D:A5] 懒算语义；升级位=BVH，触发=bench 证据）。
//! 几何查找闭包注入模式弃用——数据参数化（调用方持 scene↔mesh 关联，
//! 引擎零新状态；Scene 现无几何组件，关联归宿主/几何组件片）。

use visiaengine_core::{EntityId, Ray, Vec3, ray_aabb, ray_triangle};

/// 拾取候选：局部顶点 + 世界矩阵（`DrawMesh.transform` 同型列主序）。
#[derive(Debug)]
pub struct MeshCandidate<'a> {
    pub entity: EntityId,
    pub positions: &'a [[f32; 3]],
    pub indices: &'a [u32],
    pub world: &'a [[f64; 4]; 4],
}

/// 命中结果（三角索引 + 距离参数 + 世界命中点，D7 全程 f64）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PickHit {
    pub entity: EntityId,
    pub triangle: usize,
    pub t: f64,
    pub point: Vec3,
}

#[must_use]
fn xform(m: &[[f64; 4]; 4], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0],
        m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1],
        m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2],
    ]
}

/// 局部 AABB → 8 角变换 → 保守世界 AABB（标准做法，误差可接受于剪除）。
#[must_use]
fn world_bbox(c: &MeshCandidate) -> Option<([f64; 3], [f64; 3])> {
    if c.positions.is_empty() || c.indices.len() < 3 {
        return None;
    }
    let (mut lmin, mut lmax) = ([f64::MAX; 3], [f64::MIN; 3]);
    for p in c.positions {
        for i in 0..3 {
            let v = f64::from(p[i]);
            lmin[i] = lmin[i].min(v);
            lmax[i] = lmax[i].max(v);
        }
    }
    let mut wmin = [f64::MAX; 3];
    let mut wmax = [f64::MIN; 3];
    for sx in [0, 1] {
        for sy in [0, 1] {
            for sz in [0, 1] {
                let corner = [
                    if sx == 0 { lmin[0] } else { lmax[0] },
                    if sy == 0 { lmin[1] } else { lmax[1] },
                    if sz == 0 { lmin[2] } else { lmax[2] },
                ];
                let w = xform(c.world, corner);
                for i in 0..3 {
                    wmin[i] = wmin[i].min(w[i]);
                    wmax[i] = wmax[i].max(w[i]);
                }
            }
        }
    }
    Some((wmin, wmax))
}

/// 集合最近命中（bbox 剪除 → 逐三角 MT 正面）。全旁路/空输入 None。
#[must_use]
pub fn pick_meshes(ray: Ray, cands: &[MeshCandidate]) -> Option<PickHit> {
    let ro = [ray.origin.x, ray.origin.y, ray.origin.z];
    let mut best: Option<PickHit> = None;
    for c in cands {
        let Some((wmin, wmax)) = world_bbox(c) else {
            continue;
        };
        if !ray_aabb(ray, wmin, wmax) {
            continue;
        }
        for ti in 0..c.indices.len() / 3 {
            let mut tri = [[0.0; 3]; 3];
            for (k, dst) in tri.iter_mut().enumerate() {
                let li = c.indices[ti * 3 + k] as usize;
                if li >= c.positions.len() {
                    continue;
                }
                let p = c.positions[li];
                *dst = xform(c.world, [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]);
            }
            // 索引越界件的退化三角（零面积）由 MT 拒绝
            if let Some(t) = ray_triangle(ray, tri[0], tri[1], tri[2])
                && (best.is_none() || t < best.map(|b| b.t).unwrap_or(f64::INFINITY))
            {
                let p = [
                    ro[0] + ray.dir.x * t,
                    ro[1] + ray.dir.y * t,
                    ro[2] + ray.dir.z * t,
                ];
                best = Some(PickHit {
                    entity: c.entity,
                    triangle: ti,
                    t,
                    point: Vec3::new(p[0], p[1], p[2]),
                });
            }
        }
    }
    best
}

/// 点云拾取候选（REND-38）：世界位形（origin+transform，DrawPoints 同谱）+ 留存位置。
pub struct PointCloudCandidate<'a> {
    pub entity: EntityId,
    pub origin: [f64; 3],
    pub transform: &'a [[f64; 4]; 4],
    pub positions: &'a [[f32; 3]],
}

/// 点云命中（REND-38）：屏幕距离 + 视深 + 命中点下标（调用方 clip 逐点过滤用）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointHit {
    pub entity: EntityId,
    pub screen_d2: f64,
    pub view_z: f64,
    pub point_index: usize,
}

/// REND-38: screen-space nearest-point predicate.
///
/// Projects each retained point through the same MVP composition used by render
/// (`compose_mvp` D7 chain: proj · view_rot · T(origin−eye) · transform), converts
/// clip → screen px, and returns the point cloud whose nearest in-radius point has
/// the smallest view depth. Radius `r_px` is in the same screen-pixel domain as
/// REND-30 `radius_px` / REND-29 stroke width. Clip filtering (keeps-negative-side,
/// mirrored from the mesh pick retry loop) is the caller's policy — this function
/// is clip-agnostic by design (WGPU-21 owns the world-frame clip surface).
///
/// ponytail: linear scan (no BVH); hover-throttle = rung 1, BVH = rung 2.
#[must_use]
pub fn pick_points(
    mvp: &[[f32; 4]; 4],
    w: f32,
    h: f32,
    px: f32,
    py: f32,
    r_px: f32,
    cands: &[PointCloudCandidate<'_>],
) -> Option<PointHit> {
    let (hw, hh) = (f64::from(w) * 0.5, f64::from(h) * 0.5);
    let (sx, sy) = (f64::from(px), f64::from(py));
    let r2 = f64::from(r_px) * f64::from(r_px);
    let mut best: Option<PointHit> = None;
    for c in cands {
        let mut cloud_best: Option<(f64, f64, usize)> = None; // (screen_d2, view_z, point_index)
        for (pi, p) in c.positions.iter().enumerate() {
            // MVP = P·R·T(origin−eye)·M_local already bakes the D7 origin−eye
            // translation (compose_mvp) — feed the LOCAL point directly; adding
            // origin here would double-count it (test: origin_offset_respected_d7).
            let l = [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])];
            let cx = f64::from(mvp[0][0]) * l[0]
                + f64::from(mvp[1][0]) * l[1]
                + f64::from(mvp[2][0]) * l[2]
                + f64::from(mvp[3][0]);
            let cy = f64::from(mvp[0][1]) * l[0]
                + f64::from(mvp[1][1]) * l[1]
                + f64::from(mvp[2][1]) * l[2]
                + f64::from(mvp[3][1]);
            let cz = f64::from(mvp[0][2]) * l[0]
                + f64::from(mvp[1][2]) * l[1]
                + f64::from(mvp[2][2]) * l[2]
                + f64::from(mvp[3][2]);
            let cw = f64::from(mvp[0][3]) * l[0]
                + f64::from(mvp[1][3]) * l[1]
                + f64::from(mvp[2][3]) * l[2]
                + f64::from(mvp[3][3]);
            if cw <= 0.0 || !cw.is_finite() {
                continue;
            }
            let depth = cz / cw;
            // RH depth domain [0,1] (PIT-5): outside = behind camera / beyond far
            if !(0.0..=1.0).contains(&depth) {
                continue;
            }
            let ndc_x = cx / cw;
            let ndc_y = cy / cw;
            // NDC → screen px (y flipped: NDC +1 = top)
            let s_x = (ndc_x + 1.0) * hw;
            let s_y = (1.0 - ndc_y) * hh;
            let dx = s_x - sx;
            let dy = s_y - sy;
            let d2 = dx * dx + dy * dy;
            if d2 > r2 {
                continue;
            }
            if cloud_best.is_none_or(|(bd2, _, _)| d2 < bd2) {
                cloud_best = Some((d2, depth, pi));
            }
        }
        if let Some((d2, depth, idx)) = cloud_best
            && best.is_none_or(|b| depth < b.view_z)
        {
            best = Some(PointHit {
                entity: c.entity,
                screen_d2: d2,
                view_z: depth,
                point_index: idx,
            });
        }
    }
    best
}
