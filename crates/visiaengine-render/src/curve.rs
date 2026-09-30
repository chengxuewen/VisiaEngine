//! Parametric curves (V2.1, roadmap band): Bezier/CatmullRom sampling +
//! tube/ribbon mesh generation. Pure math, +0 ABI — output feeds the regular
//! MeshDesc path (consumers upload like any mesh). Y-up world frame, f32
//! (mesh-vertex domain).

/// Cubic Bezier point at parameter t (4 control points).
#[must_use]
pub fn bezier3(p: &[[f32; 3]; 4], t: f32) -> [f32; 3] {
    let u = 1.0 - t;
    let w0 = u * u * u;
    let w1 = 3.0 * u * u * t;
    let w2 = 3.0 * u * t * t;
    let w3 = t * t * t;
    [
        w0 * p[0][0] + w1 * p[1][0] + w2 * p[2][0] + w3 * p[3][0],
        w0 * p[0][1] + w1 * p[1][1] + w2 * p[2][1] + w3 * p[3][1],
        w0 * p[0][2] + w1 * p[1][2] + w2 * p[2][2] + w3 * p[3][2],
    ]
}

/// Catmull-Rom spline point at segment `seg` (needs pts.len() >= 2; endpoints
/// duplicated), parameter t in [0,1] within the segment. Centripetal-free v1
/// (uniform CR — kinks possible on uneven spacing; chordal = upgrade slot).
#[must_use]
pub fn catmull_rom(pts: &[[f32; 3]], seg: usize, t: f32) -> [f32; 3] {
    assert!(pts.len() >= 2, "catmull_rom needs >= 2 points");
    let n_seg = pts.len() - 1;
    assert!(seg < n_seg, "segment out of range");
    let p0 = if seg == 0 { pts[0] } else { pts[seg - 1] };
    let p1 = pts[seg];
    let p2 = pts[seg + 1];
    let p3 = if seg + 2 == pts.len() {
        pts[seg + 1]
    } else {
        pts[seg + 2]
    };
    let t2 = t * t;
    let t3 = t2 * t;
    [
        0.5 * ((2.0 * p1[0])
            + (-p0[0] + p2[0]) * t
            + (2.0 * p0[0] - 5.0 * p1[0] + 4.0 * p2[0] - p3[0]) * t2
            + (-p0[0] + 3.0 * p1[0] - 3.0 * p2[0] + p3[0]) * t3),
        0.5 * ((2.0 * p1[1])
            + (-p0[1] + p2[1]) * t
            + (2.0 * p0[1] - 5.0 * p1[1] + 4.0 * p2[1] - p3[1]) * t2
            + (-p0[1] + 3.0 * p1[1] - 3.0 * p2[1] + p3[1]) * t3),
        0.5 * ((2.0 * p1[2])
            + (-p0[2] + p2[2]) * t
            + (2.0 * p0[2] - 5.0 * p1[2] + 4.0 * p2[2] - p3[2]) * t2
            + (-p0[2] + 3.0 * p1[2] - 3.0 * p2[2] + p3[2]) * t3),
    ]
}

/// Sample a curve into `n` polyline points. Bezier (4 controls) or CatmullRom
/// (knot vector). n >= 2 enforced by return-empty contract.
#[must_use]
pub fn sample_bezier(p: &[[f32; 3]; 4], n: usize) -> Vec<[f32; 3]> {
    if n < 2 {
        return Vec::new();
    }
    (0..n)
        .map(|i| bezier3(p, i as f32 / (n - 1) as f32))
        .collect()
}

#[must_use]
pub fn sample_catmull_rom(pts: &[[f32; 3]], samples_per_seg: usize) -> Vec<[f32; 3]> {
    if pts.len() < 2 || samples_per_seg == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity((pts.len() - 1) * samples_per_seg);
    for seg in 0..pts.len() - 1 {
        let last = seg + 2 == pts.len();
        let end = if last {
            samples_per_seg
        } else {
            samples_per_seg - 1
        };
        for k in 0..end {
            // t=1 lands exactly on the segment's END knot only in the final
            // segment (endpoint law); interior segments stop at t<1 (the next
            // segment's t=0 covers the shared knot — no double-push).
            let t = if last && k == samples_per_seg - 1 {
                1.0
            } else {
                k as f32 / samples_per_seg as f32
            };
            out.push(catmull_rom(pts, seg, t));
        }
    }
    out
}

/// Tube mesh (V2.1): sweep `radius` along `center` with `sides` radial
/// segments. Parallel-transport frame (rotation-minimizing); world-up fallback
/// when the tangent is vertical. Returns (positions, indices) in MeshDesc
/// shape. Degenerate input (fewer than 2 centers) -> empty.
#[must_use]
pub fn tube(center: &[[f32; 3]], radius: f32, sides: usize) -> (Vec<[f32; 3]>, Vec<u32>) {
    if center.len() < 2 || sides < 3 || radius <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let mut positions = Vec::with_capacity(center.len() * sides);
    let mut prev_normal = [0.0f32, 1.0, 0.0];
    for (i, &c) in center.iter().enumerate() {
        // tangent (central difference in the interior)
        let tangent = if i == 0 {
            sub(center[1], center[0])
        } else if i + 1 == center.len() {
            sub(center[i], center[i - 1])
        } else {
            sub(center[i + 1], center[i - 1])
        };
        let t = norm(tangent);
        if norm2(t) < 1e-12 {
            // degenerate tangent: reuse previous ring offsets
            for k in 0..sides {
                let a = std::f32::consts::TAU * k as f32 / sides as f32;
                let (s, cth) = a.sin_cos();
                positions.push([
                    c[0] + radius * (prev_normal[0] * cth + cross_up(t)[0] * s),
                    c[1] + radius * (prev_normal[1] * cth + cross_up(t)[1] * s),
                    c[2] + radius * (prev_normal[2] * cth + cross_up(t)[2] * s),
                ]);
            }
            continue;
        }
        // rotation-minimizing normal: project previous, fallback world-up
        let mut n = sub(prev_normal, scale(t, dot(prev_normal, t)));
        if norm2(n) < 1e-8 {
            n = [0.0, 0.0, 1.0];
            n = sub(n, scale(t, dot(n, t)));
        }
        n = norm(n);
        prev_normal = n;
        let b = cross(t, n);
        for k in 0..sides {
            let a = std::f32::consts::TAU * k as f32 / sides as f32;
            let (s, cth) = a.sin_cos();
            positions.push([
                c[0] + radius * (n[0] * cth + b[0] * s),
                c[1] + radius * (n[1] * cth + b[1] * s),
                c[2] + radius * (n[2] * cth + b[2] * s),
            ]);
        }
    }
    // indices: ring-to-ring quads
    let mut indices = Vec::with_capacity((center.len() - 1) * sides * 6);
    for r in 0..center.len() - 1 {
        for k in 0..sides {
            let k1 = (k + 1) % sides;
            let a = (r * sides + k) as u32;
            let b = (r * sides + k1) as u32;
            let c = ((r + 1) * sides + k1) as u32;
            let d = ((r + 1) * sides + k) as u32;
            indices.extend([a, b, c, a, c, d]);
        }
    }
    (positions, indices)
}

/// Flat ribbon in the XZ plane (road/path base): width `w` along the curve,
/// y=0 constant. UVs omitted (MeshDesc uv empty = untextured; textured lane
/// upgrade = when flow-anim lands).
#[must_use]
pub fn ribbon_xz(center: &[[f32; 3]], width: f32) -> (Vec<[f32; 3]>, Vec<u32>) {
    if center.len() < 2 || width <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let hw = width * 0.5;
    let mut positions = Vec::with_capacity(center.len() * 2);
    for (i, &c) in center.iter().enumerate() {
        let tangent = if i == 0 {
            sub(center[1], center[0])
        } else if i + 1 == center.len() {
            sub(center[i], center[i - 1])
        } else {
            sub(center[i + 1], center[i - 1])
        };
        let t = norm([tangent[0], 0.0, tangent[2]]);
        if norm2(t) < 1e-12 {
            positions.push([c[0], c[1], c[2] - hw]);
            positions.push([c[0], c[1], c[2] + hw]);
            continue;
        }
        // XZ normal = perpendicular in-plane
        let n = [-t[2], 0.0, t[0]];
        positions.push([c[0] + n[0] * hw, c[1], c[2] + n[2] * hw]);
        positions.push([c[0] - n[0] * hw, c[1], c[2] - n[2] * hw]);
    }
    let mut indices = Vec::with_capacity((center.len() - 1) * 6);
    for r in 0..center.len() - 1 {
        let a = (r * 2) as u32;
        let b = (r * 2 + 1) as u32;
        let c = (r * 2 + 3) as u32;
        let d = (r * 2 + 2) as u32;
        indices.extend([a, b, c, a, c, d]);
    }
    (positions, indices)
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm2(a: [f32; 3]) -> f32 {
    dot(a, a)
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = norm2(a).sqrt();
    if l > 1e-12 { scale(a, 1.0 / l) } else { a }
}
fn cross_up(_t: [f32; 3]) -> [f32; 3] {
    [1.0, 0.0, 0.0] // arbitrary in-plane axis for degenerate fallback ring
}
