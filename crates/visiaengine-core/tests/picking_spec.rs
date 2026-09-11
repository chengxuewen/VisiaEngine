//! CORE-14/15：拾取射线数学面（[E3D:4xx] 交互片地基，Möller–Trumbore）。
//! 设计注记：Scene 现无几何组件——相交核心以纯函数落地，
//! 实体↔几何关联（几何组件或渲染侧 BVH）属后续片，本层零耦合可独立测试。

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
    // 背向件默认剔除语义：CCW 正面朝 +z，从 +z 看命中 t=5
    let t = ray_triangle(r, a, b, c).expect("front hit");
    assert!((t - 5.0).abs() < 1e-9);
    // 反旋（CW 序）= 背向 → None（默认剔除）
    assert!(ray_triangle(r, a, c, b).is_none());
    // 强制双面口（marker 拾取用）
    assert!(ray_triangle_double(r, a, c, b).is_some());
    // 平行 / 射线背后三角形 / 出界命中（穿越但 u+v>1）
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
    // 退化三角形（零面积）不 panic → None
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
    assert!(!ray_aabb(r, [2.0, 2.0, -1.0], [3.0, 3.0, 1.0]), "旁路剪除");
    assert!(!ray_aabb(r, [-1.0, -1.0, 6.0], [1.0, 1.0, 7.0]), "背后剪除");
    assert!(
        ray_aabb(r, [-0.5, -0.5, 4.0], [1.0, 1.0, 6.0]),
        "起点在框内=命中"
    );
}
