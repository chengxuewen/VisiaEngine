//! GEO-21/22：地图平面量测（3857 单位，交互片 [E3D:4xx]；geodesic 属 D9 后
//! "坐标系完善"域，本条冻结平面语义——3857 距离=墨卡托畸变值，文档化）。

use visiaengine_core::Vec3;
use visiaengine_geo::{planar_distance, ring_area};

#[must_use]
fn p(x: f64, y: f64) -> Vec3 {
    Vec3::new(x, y, 0.0)
}

// spec: GEO-21
#[test]
fn distance_is_planar_polyline_sum() {
    // 3-4-5 直角（3857 米域单位）
    assert!((planar_distance(&[p(0.0, 0.0), p(300.0, 400.0)]) - 500.0).abs() < 1e-9);
    // 折线和
    let line = [p(0.0, 0.0), p(300.0, 400.0), p(300.0, 400.0)];
    assert!((planar_distance(&line) - 500.0).abs() < 1e-9);
    // 退化：空/单点=0
    assert_eq!(planar_distance(&[]), 0.0);
    assert_eq!(planar_distance(&[p(1.0, 1.0)]), 0.0);
    // 非有限点跳过不成段（3857 大坐标下 NaN 传播防御）
    let bad = [p(0.0, 0.0), p(f64::NAN, 0.0), p(300.0, 400.0)];
    assert!((planar_distance(&bad) - 500.0).abs() < 1e-9);
}

// spec: GEO-22
#[test]
fn area_is_shoelace_absolute_and_orientation_free() {
    let square = [p(0.0, 0.0), p(2.0, 0.0), p(2.0, 2.0), p(0.0, 2.0)];
    assert!((ring_area(&square) - 4.0).abs() < 1e-9);
    // 反向环同值（abs 语义，GeoJSON 外环顺时针/逆时针双俗）
    let rev: Vec<Vec3> = square.iter().rev().copied().collect();
    assert!((ring_area(&rev) - 4.0).abs() < 1e-9);
    // 闭合重复点输入与不闭合输入同面积（GeoJSON 惯例首尾重复）
    let mut c2 = square.to_vec();
    c2.push(square[0]);
    assert!((ring_area(&c2) - ring_area(&square)).abs() < 1e-9);
    // 退化：共线<3 顶点=0；含非有限点整环=0（无意义不猜测）
    assert_eq!(ring_area(&[p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)]), 0.0);
    assert_eq!(ring_area(&[p(0.0, 0.0), p(1.0, 0.0)]), 0.0);
    assert_eq!(ring_area(&[]), 0.0);
    let bad = [p(0.0, 0.0), p(f64::INFINITY, 2.0), p(2.0, 2.0), p(0.0, 2.0)];
    assert_eq!(ring_area(&bad), 0.0);
}
