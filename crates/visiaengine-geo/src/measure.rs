//! 地图平面量测（GEO-21/22）：3857 单位纯函数。
//! 语义冻结：墨卡托平面距离/面积随纬度畸变（cos φ 因子）——大地测量学
//! 语义属"坐标系完善"（README Alpha），本层不充胖子。

use visiaengine_core::Vec3;

/// 折线平面长度（3857 米）。非有限点跳过不成段（NaN 传播防御）。
#[must_use]
pub fn planar_distance(pts: &[Vec3]) -> f64 {
    let mut sum = 0.0;
    let mut prev: Option<(f64, f64)> = None;
    for q in pts.iter().filter_map(finite2) {
        if let Some(pv) = prev {
            sum += ((q.0 - pv.0) * (q.0 - pv.0) + (q.1 - pv.1) * (q.1 - pv.1)).sqrt();
        }
        prev = Some(q);
    }
    sum
}

/// 环面积（shoelace 绝对值，米²）。方向无关、闭合与否同值（GeoJSON 首尾
/// 重复惯例自动成立）。任一非有限点 → 整环 0（不猜测残缺多边形）。
#[must_use]
pub fn ring_area(ring: &[Vec3]) -> f64 {
    let Some(pts): Option<Vec<(f64, f64)>> = ring.iter().map(finite2).collect() else {
        return 0.0;
    };
    if pts.len() < 3 {
        return 0.0;
    }
    let mut acc = 0.0;
    for i in 0..pts.len() {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % pts.len()];
        acc += x1 * y2 - x2 * y1;
    }
    (acc * 0.5).abs()
}

#[must_use]
fn finite2(v: &Vec3) -> Option<(f64, f64)> {
    (v.x.is_finite() && v.y.is_finite()).then_some((v.x, v.y))
}
