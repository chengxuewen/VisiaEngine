//! CORE-14/15: the picking ray math face ([E3D:4xx] interaction-slice foundation, Moller-Trumbore).
//! Design note: Scene currently has no geometry component -- the intersection core lands as pure functions,
//! entity-geometry association (geometry components or a render-side BVH) belongs to later slices; this layer is zero-coupled and independently testable.

use visiaengine_core::{Ray, Vec3, ray_aabb, ray_triangle, ray_triangle_double};

const Z5: Vec3 = Vec3::new(0.0, 0.0, 5.0);

// spec: CORE-14
#[test]
fn ray_triangle_hit_miss_and_backface() {
    let r = Ray {
        origin: Vec3::new(0.0, 0.0, 5.0),
        dir: Vec3::new(0.0, 0.0, -1.0),
    };
    let a = [-1.0, -1.0, 0.0];
    let b = [1.0, -1.0, 0.0];
    let c = [0.0, 1.0, 0.0];
    // backface default-culling semantics: a CCW front face toward +z, viewed from +z hits at t=5
    let t = ray_triangle(r, a, b, c).expect("front hit");
    assert!((t - 5.0).abs() < 1e-9);
    // reversed (CW order) = backface -> None (default culling)
    assert!(ray_triangle(r, a, c, b).is_none());
    // forced double-sided entry (for marker picking)
    assert!(ray_triangle_double(r, a, c, b).is_some());
    // parallel / triangle behind the ray / out-of-bounds hit (crossing but u+v>1)
    let par = Ray {
        origin: Z5,
        dir: Vec3::new(1.0, 0.0, 0.0),
    };
    assert!(ray_triangle(par, a, b, c).is_none());
    let behind = Ray {
        origin: Vec3::new(0.0, 0.0, -1.0),
        dir: Vec3::new(0.0, 0.0, -1.0),
    };
    assert!(ray_triangle(behind, a, b, c).is_none());
    let edge = Ray {
        origin: Vec3::new(3.0, 3.0, 5.0),
        dir: Vec3::new(0.0, 0.0, -1.0),
    };
    assert!(ray_triangle(edge, a, b, c).is_none());
    // degenerate triangle (zero area) does not panic -> None
    assert!(ray_triangle(r, a, b, b).is_none());
}

// spec: CORE-15
#[test]
fn ray_aabb_prune() {
    let r = Ray {
        origin: Z5,
        dir: Vec3::new(0.0, 0.0, -1.0),
    };
    assert!(ray_aabb(r, [-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]));
    assert!(
        !ray_aabb(r, [2.0, 2.0, -1.0], [3.0, 3.0, 1.0]),
        "bypass culling"
    );
    assert!(
        !ray_aabb(r, [-1.0, -1.0, 6.0], [1.0, 1.0, 7.0]),
        "behind culling"
    );
    assert!(
        ray_aabb(r, [-0.5, -0.5, 4.0], [1.0, 1.0, 6.0]),
        "start inside the box = hit"
    );
}
