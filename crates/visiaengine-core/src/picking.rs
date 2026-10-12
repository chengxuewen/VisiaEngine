//! Picking ray math (the foundation of the interaction slice, CORE-14/15).
//! Pure-function face: Ray x triangle (Moller-Trumbore) / AABB culling.
//! Entity-geometry association (componentization/BVH) belongs to later slices -- this layer is zero-coupled.

use crate::Vec3;

/// World-space ray (D7: all f64, no precision disaster at far origins). `dir` is conventionally unit-length (entry constructors normalize).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: Vec3,
    pub dir: Vec3,
}

const DET_EPS: f64 = 1e-12;
const T_EPS: f64 = 1e-9;
/// u/v boundary tolerance: the picking ray can land exactly on quad diagonals/shared edges (screen pixel centers have
/// no alignment relation to world geometry), and without slack the hit flips on 1ulp noise across the winding side = flaky.
const UV_EPS: f64 = 1e-12;

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
        // front single-sided (CCW winding facing the ray's origin); degenerate triangles are rejected by this branch or the divisor bound on the same road
        if det < DET_EPS {
            return None;
        }
    } else if det.abs() < DET_EPS {
        return None;
    }
    let inv = 1.0 / det;
    let tv = sub(ro, a);
    let u = dot(tv, p) * inv;
    if !(-UV_EPS..=1.0 + UV_EPS).contains(&u) {
        return None;
    }
    let q = cross(tv, e1);
    let v = dot(rd, q) * inv;
    if !(-UV_EPS..=1.0 + UV_EPS).contains(&v) || u + v > 1.0 + UV_EPS {
        return None;
    }
    let t = dot(e2, q) * inv;
    // backface culling (t must advance forward-positive)
    (t > T_EPS).then_some(t)
}

/// Ray x triangle (CCW front face, backface culled). On hit returns the parameter t (`origin + t*dir`).
#[must_use]
pub fn ray_triangle(r: Ray, a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<f64> {
    mt(r, a, b, c, false)
}

/// Double-sided entry (pieces without winding guarantees, e.g. markers). Degenerate -> None.
#[must_use]
pub fn ray_triangle_double(r: Ray, a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> Option<f64> {
    mt(r, a, b, c, true)
}

/// Ray x AABB (slab method). Start inside the box = hit; for entity-level culling (computed on demand, not stored).
#[must_use]
pub fn ray_aabb(r: Ray, min: [f64; 3], max: [f64; 3]) -> bool {
    let mut tmin = f64::NEG_INFINITY;
    let mut tmax = f64::INFINITY;
    let o = [r.origin.x, r.origin.y, r.origin.z];
    let d = [r.dir.x, r.dir.y, r.dir.z];
    for i in 0..3 {
        if d[i].abs() < DET_EPS {
            // axis-parallel: out of bounds means no intersection
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
