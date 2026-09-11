//! 场景级拾取编排（REND-23/24）：候选 mesh 集 + 射线 → 最近命中。
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
