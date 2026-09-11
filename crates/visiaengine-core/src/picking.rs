//! 拾取射线数学（交互片地基，CORE-14/15）。
//! 纯函数面：Ray × 三角形（Möller–Trumbore）/ AABB 剪除。
//! 实体↔几何关联（组件化/BVH）归后续片——本层零耦合。

use crate::Vec3;

/// 世界系射线（D7：全 f64，远原点无精度灾难）。`dir` 约定单位长（口构造方归一）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

const DET_EPS: f64 = 1e-12;
const T_EPS: f64 = 1e-9;

#[must_use]
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
#[must_use]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
#[must_use]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn mt(r: Ray, a: [f64; 3], b: [f64; 3], c: [f64; 3], double_sided: bool) -> Option<f64> {
    let rd = [r.dir.x, r.dir.y, r.dir.z];
    let ro = [r.origin.x, r.origin.y, r.origin.z];
    let e1 = sub(b, a);
    let e2 = sub(c, a);
    let p = cross(rd, e2);
    let det = dot(e1, p);
    if !double_sided {
        // 正面单面（CCW 绕序朝射线来向）；退化三角形同路被此分支或除界拒
        if det < DET_EPS {
            return None;
        }
    } else if det.abs() < DET_EPS {
        return None;
    }
    let inv = 1.0 / det;
    let tv = sub(ro, a);
    let u = dot(tv, p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(tv, e1);
    let v = dot(rd, q) * inv;
    if !(0.0..=1.0).contains(&v) || u + v > 1.0 {
        return None;
    }
    let t = dot(e2, q) * inv;
    // 背后剔除（t 必须正向前进）
    (t > T_EPS).then_some(t)
}

/// 射线×三角形（CCW 正面，背向剔除）。命中返回参数 t（`origin + t·dir`）。
#[must_use]
pub fn ray_triangle(r: Ray, a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<f64> {
    mt(r, a, b, c, false)
}

/// 双面口（marker 等无绕序保证件）。退化→None。
#[must_use]
pub fn ray_triangle_double(r: Ray, a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<f64> {
    mt(r, a, b, c, true)
}

/// 射线×AABB（slab 法）。起点在框内=命中；供实体级剪除（即算不常存）。
#[must_use]
pub fn ray_aabb(r: Ray, min: [f64; 3], max: [f64; 3]) -> bool {
    let mut tmin = f64::NEG_INFINITY;
    let mut tmax = f64::INFINITY;
    let o = [r.origin.x, r.origin.y, r.origin.z];
    let d = [r.dir.x, r.dir.y, r.dir.z];
    for i in 0..3 {
        if d[i].abs() < DET_EPS {
            // 平行轴：越界即无交
            if o[i] < min[i] || o[i] > max[i] {
                return false;
            }
        } else {
            let inv = 1.0 / d[i];
            let mut t0 = (min[i] - o[i]) * inv;
            let mut t1 = (max[i] - o[i]) * inv;
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
            }
            tmin = tmin.max(t0);
            tmax = tmax.min(t1);
            if tmin > tmax {
                return false;
            }
        }
    }
    tmax >= 0.0
}
