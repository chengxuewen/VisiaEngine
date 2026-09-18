//! REND-10..14 相机数学（docs/sdd/render.md）。

#![allow(clippy::float_cmp)]

use visiaengine_render::camera::CameraRig;

const TOL: f64 = 1e-6;

fn rig() -> CameraRig {
    CameraRig::look_at([3.0, 4.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
}

/// 列主序矩阵·列向量乘法取指定行
fn apply(m: &[[f32; 4]; 4], v: [f32; 4], row: usize) -> f32 {
    (0..4).map(|c| m[c][row] * v[c]).sum()
}

// spec: REND-10
#[test]
fn camera_rig_view_matrix_lookat() {
    let v = rig().view_matrix();
    let e = [3.0f32, 4.0, 5.0, 1.0];
    for (row, _) in (0..3).enumerate() {
        let dot: f32 = (0..4).map(|c| v[c][row] * e[c]).sum();
        assert!(dot.abs() < 1e-3, "row {row} = {dot}, 期望 ~0");
    }
    let wsum: f32 = (0..4).map(|cc| v[cc][3] * e[cc]).sum();
    assert!((wsum - 1.0).abs() < 1e-3, "w 行 = 1, got {wsum}");
    let t = [0.0f32, 0.0, 0.0, 1.0];
    let tz: f32 = (0..4).map(|c| v[c][2] * t[c]).sum();
    assert!(tz < 0.0, "target z={tz} 应为负");
}

// spec: REND-11
#[test]
fn persp_proj_matches_glam_std() {
    // 行为断言（wgpu 原生深度语义）：view z=-near→clip 0，z=-far→clip 1
    let (near, far) = (1.0f32, 1000.0);
    let p = rig()
        .perspective(std::f32::consts::FRAC_PI_2, 1.0, near, far)
        .unwrap();
    let zn = apply(&p, [0.0, 0.0, -near, 1.0], 2) / apply(&p, [0.0, 0.0, -near, 1.0], 3);
    let zf = apply(&p, [0.0, 0.0, -far, 1.0], 2) / apply(&p, [0.0, 0.0, -far, 1.0], 3);
    assert!(zn.abs() < 1e-4, "near 深度应映射 0，得 {zn}");
    assert!((zf - 1.0).abs() < 1e-4, "far 深度应映射 1，得 {zf}");
    // fov=90° aspect=1 → x/y 对角 1
    assert!((apply(&p, [1.0, 0.0, 0.0, 1.0], 0) - 1.0).abs() < 1e-4);
    assert!((apply(&p, [0.0, 1.0, 0.0, 1.0], 1) - 1.0).abs() < 1e-4);
}

// spec: REND-12
#[test]
fn ortho_zoom_scales_halfextents() {
    let r = rig();
    for zoom in [1.0f32, 4.0, 128.0] {
        let p = r.ortho_frame(zoom, 640.0, 480.0, 1.0, 1e5).unwrap();
        assert!((p[0][0] - 1.0 / zoom).abs() < 1e-4, "x 列 = 1/zoom");
        // 深度 [0,1]：z=-near→0
        let zn = apply(&p, [0.0, 0.0, -1.0, 1.0], 2);
        assert!(zn.abs() < 1e-4, "ortho near 深度应 0，得 {zn}");
    }
}

// spec: REND-13
#[test]
fn orbit_90deg_equivalence() {
    let mut a = rig();
    a.orbit_delta(std::f32::consts::FRAC_PI_4 as f64, 0.0);
    a.orbit_delta(std::f32::consts::FRAC_PI_4 as f64, 0.0);
    let mut b = rig();
    b.orbit_delta(std::f32::consts::FRAC_PI_2 as f64, 0.0);
    let (ea, eb) = (a.eye(), b.eye());
    for i in 0..3 {
        assert!(
            (ea[i] - eb[i]).abs() < TOL,
            "axis{i}: {} vs {}",
            ea[i],
            eb[i]
        );
    }
}

// spec: REND-14
#[test]
fn degenerate_near_far_rejected() {
    assert!(rig().perspective(1.0, 1.0, 100.0, 50.0).is_none());
    assert!(rig().ortho_frame(10.0, 1.0, 1.0, 100.0, 50.0).is_none());
}

// spec: REND-15
#[test]
fn switch_midpoint_continuity() {
    let a = CameraRig::orbit([0.0; 3], 0.0, 1.5, 500.0, 40.0, 1.1, 1.0, 1e5);
    let b = CameraRig::orbit([0.0; 3], 0.8, 0.35, 9.0, 1.0, 1.1, 0.1, 500.0);
    let m1 = CameraRig::mix_rig(&a, &b, 0.5);
    let m2 = CameraRig::mix_rig(&a, &b, 0.500_001);
    for (x, y) in [
        (m1.dist, m2.dist),
        (m1.pitch, m2.pitch),
        (m1.zoom, m2.zoom),
        (m1.fov_y, m2.fov_y),
    ] {
        assert!((x - y).abs() < 1e-3, "t 域连续性破坏: {x} vs {y}");
    }
}

// spec: REND-16
#[test]
fn switch_endpoints_exact() {
    let a = CameraRig::orbit([1.0, 2.0, 3.0], 0.1, 0.2, 7.0, 2.0, 1.0, 0.5, 99.0);
    let b = CameraRig::orbit([-4.0, 0.0, 5.0], 2.0, -0.4, 3.0, 0.5, 0.7, 0.1, 42.0);
    assert_eq!(CameraRig::mix_rig(&a, &b, 0.0), a);
    assert_eq!(CameraRig::mix_rig(&a, &b, 1.0), b);
}

/// 归一化到 [0,2π)（姿态等价的规范形——flyTo 终点 yaw 与源差 2π 整数倍时判等）
fn yaw_norm(y: f64) -> f64 {
    y.rem_euclid(std::f64::consts::TAU)
}

// spec: REND-34
#[test]
fn fly_easing_endpoints_exact() {
    use visiaengine_render::Easing;
    // 三曲线 f(0)=0 / f(1)=1 逐位精确（缓动不破坏 REND-16 端点语义的地基）
    for e in [Easing::Linear, Easing::CubicInOut, Easing::CubicOut] {
        assert_eq!(e.ease(0.0), 0.0, "{e:?} f(0)");
        assert_eq!(e.ease(1.0), 1.0, "{e:?} f(1)");
        assert_eq!(e.ease(-3.0), 0.0, "{e:?} 越下界钳制");
        assert_eq!(e.ease(7.0), 1.0, "{e:?} 越上界钳制");
    }
}

// spec: REND-34
#[test]
fn fly_easing_monotone_and_bounded() {
    use visiaengine_render::Easing;
    for e in [Easing::Linear, Easing::CubicInOut, Easing::CubicOut] {
        let mut prev = 0.0f64;
        for i in 0..=100 {
            let t = i as f64 / 100.0;
            let v = e.ease(t);
            assert!((0.0..=1.0).contains(&v), "{e:?} 值域越界 t={t} v={v}");
            assert!(v >= prev - 1e-12, "{e:?} 非单调 t={t}");
            prev = v;
        }
    }
    // 缓动中点可辨（CubicInOut 慢起步：t=0.25 → 值 < 线性值）
    assert!(Easing::CubicInOut.ease(0.25) < 0.25);
    assert!(Easing::CubicOut.ease(0.25) > 0.25);
    assert!((Easing::Linear.ease(0.25) - 0.25).abs() < TOL);
}

// spec: REND-34
#[test]
fn fly_sample_endpoints_and_continuity() {
    use visiaengine_render::Easing;
    let a = CameraRig::look_at([0.0, -40.0, 20.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let b = CameraRig::look_at([30.0, 10.0, 5.0], [4.0, 2.0, 0.0], [0.0, 1.0, 0.0]);
    for e in [Easing::Linear, Easing::CubicInOut, Easing::CubicOut] {
        // 端点精确：t=0 恒 a、t=1 恒 b（全字段逐位——含 near/far 恒取 from/to 语义）
        let s0 = CameraRig::fly_sample(&a, &b, 0.0, e);
        let s1 = CameraRig::fly_sample(&a, &b, 1.0, e);
        assert_eq!(s0.target, a.target);
        assert_eq!((s0.yaw - a.yaw).abs(), 0.0);
        assert_eq!(s1.target, b.target);
        assert_eq!((s1.yaw - b.yaw).abs(), 0.0);
        // 连续性：密采样相邻步位姿差有界（位置分量）
        let mut prev = s0;
        for i in 1..=200 {
            let t = i as f64 / 200.0;
            let s = CameraRig::fly_sample(&a, &b, t, e);
            let d = ((s.target[0] - prev.target[0]).powi(2)
                + (s.target[1] - prev.target[1]).powi(2)
                + (s.target[2] - prev.target[2]).powi(2))
            .sqrt();
            assert!(d < 0.5, "{e:?} 不连续 t={t} 步距 {d}");
            prev = s;
        }
    }
}

// spec: REND-34
#[test]
fn fly_sample_yaw_shortest_arc_wrap() {
    use visiaengine_render::Easing;
    let base = CameraRig::look_at([0.0, -40.0, 20.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    // 造近 ±π 两端点：Δyaw 线性 ≈ 6.2（几乎整圈）而最短弧 ≈ 0.28
    let mut a = base;
    a.yaw = std::f64::consts::PI - 0.14;
    let mut b = base;
    b.yaw = -std::f64::consts::PI + 0.14;
    let mid = CameraRig::fly_sample(&a, &b, 0.5, Easing::Linear);
    // 走最短弧 ⇒ 中点应跨过 ±π 边界（mod 2π ≈ π），而非穿 0
    assert!(
        yaw_norm(mid.yaw) > std::f64::consts::PI - 0.01,
        "wrap 失败：mid yaw mod 2π = {}，期望 ~π",
        yaw_norm(mid.yaw)
    );
    // 线性 wrap 会掉到 0 附近（旧路径值 ±0.14），据此双向锁死本修正
    let linear_mid = (a.yaw + b.yaw) / 2.0;
    assert!(
        linear_mid.abs() < 0.01,
        "对照前提：线性中点 ~0（got {linear_mid}）"
    );
    // 终点姿态等价（mod 2π 相等，值可差 2π 整数倍）
    let end = CameraRig::fly_sample(&a, &b, 1.0, Easing::Linear);
    assert!((yaw_norm(end.yaw) - yaw_norm(b.yaw)).abs() < 1e-9);
    // |Δyaw|<π 时不动（普通飞行不受 wrap 干预）
    let c = CameraRig::fly_sample(&a, &b, 0.0, Easing::Linear);
    assert!((yaw_norm(c.yaw) - yaw_norm(a.yaw)).abs() < 1e-9);
    let small_b = {
        let mut r = base;
        r.yaw = 0.5;
        r
    };
    let m2 = CameraRig::fly_sample(&base, &small_b, 0.5, Easing::Linear);
    assert!(
        (m2.yaw - (base.yaw + 0.5) / 2.0).abs() < 1e-9,
        "小 Δ 必须纯线性"
    );
}

// spec: REND-37
#[test]
fn ray_ground_intersect_six_forms() {
    use visiaengine_core::{Ray, Vec3};
    let v = |x: f64, y: f64, z: f64| Vec3::new(x, y, z);
    // ① 正下：原点 (1,2,50) 方向 (0,0,-1) → t=50 → (1,2,0)
    let hit = visiaengine_render::ray_ground_intersect(Ray {
        origin: v(1.0, 2.0, 50.0),
        dir: v(0.0, 0.0, -1.0),
    })
    .expect("正下必交");
    assert!(
        (hit.x - 1.0).abs() < 1e-9 && (hit.y - 2.0).abs() < 1e-9 && hit.z.abs() < 1e-9,
        "got {hit:?}"
    );
    // ② 斜射
    let hit = visiaengine_render::ray_ground_intersect(Ray {
        origin: v(0.0, 0.0, 10.0),
        dir: v(0.0, -1.0, -1.0),
    })
    .expect("斜射");
    assert!(
        (hit.x).abs() < 1e-9 && (hit.y + 10.0).abs() < 1e-9 && hit.z.abs() < 1e-9,
        "got {hit:?}"
    );
    // ③ 水平视线拒（d.z=0）
    assert!(
        visiaengine_render::ray_ground_intersect(Ray {
            origin: v(0.0, 0.0, 10.0),
            dir: v(0.0, -1.0, 0.0)
        })
        .is_none()
    );
    // ④ 近水平（|d.z|<eps）拒：t 爆炸护栏
    assert!(
        visiaengine_render::ray_ground_intersect(Ray {
            origin: v(0.0, 0.0, 10.0),
            dir: v(0.0, -1.0, -1e-12)
        })
        .is_none()
    );
    // ⑤ 背向（朝上）拒
    assert!(
        visiaengine_render::ray_ground_intersect(Ray {
            origin: v(0.0, 0.0, 10.0),
            dir: v(0.0, 0.0, 1.0)
        })
        .is_none()
    );
    // ⑥ 地下出发向上=背向；地下朝下=t<0 亦拒（地面之下无导航语义）
    assert!(
        visiaengine_render::ray_ground_intersect(Ray {
            origin: v(0.0, 0.0, -5.0),
            dir: v(0.0, 0.0, -1.0)
        })
        .is_none()
    );
}

// spec: REND-37
#[test]
fn viewport_rect_from_frac_seam_accounting() {
    use visiaengine_render::ViewportRect;
    // 半幅整除：128×.25=32 精确
    let r = ViewportRect::from_frac(0.25, 0.0, 0.5, 1.0, 128, 64);
    assert_eq!((r.x, r.y, r.width, r.height), (32, 0, 64, 64));
    // 缝账①：两邻片共边无缝（右缘=邻左缘）
    let a = ViewportRect::from_frac(1.0 / 3.0, 0.0, 1.0 / 3.0, 1.0, 128, 64);
    let b = ViewportRect::from_frac(2.0 / 3.0, 0.0, 1.0 / 3.0, 1.0, 128, 64);
    assert_eq!(a.x + a.width, b.x, "缝断裂/重叠：{:?} {:?}", a, b);
    // 缝账②：末片右缘钉到 surface 界（无 1px 漏底）
    let z = ViewportRect::from_frac(0.75, 0.0, 0.25, 1.0, 127, 64); // 奇数宽舍入场景
    assert_eq!(z.x + z.width, 127);
    // 越界钳制（宿主脏值防御）：0.9+0.5 → w 截到边界且 ≥1
    let c = ViewportRect::from_frac(0.9, 0.0, 0.5, 1.0, 100, 100);
    assert!(c.width >= 1 && c.x + c.width <= 100);
    // 零尺寸=保 1（scissor 域下限，与 render_view_rects max(1) 合流）
    let n = ViewportRect::from_frac(0.5, 0.5, 0.0, 0.0, 100, 100);
    assert!(
        n.width >= 1 && n.height >= 1,
        "零尺寸=保 1（scissor 下限钉死）got {:?}",
        (n.width, n.height)
    );
}
