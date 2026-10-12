//! Map-plane measurement (GEO-21/22): pure functions in 3857 units.
//! Semantics frozen: Mercator plane distance/area distorts with latitude (the cos phi factor) -- geodesic
//! semantics belong to "coordinate-system completeness" (README Alpha), this layer does not stretch beyond its coat.

use visiaengine_core::Vec3;

/// Polyline plane length (3857 meters). Non-finite points are skipped, not segmented (NaN propagation defense).
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

/// Ring area (shoelace absolute value, sq meters). Orientation-agnostic, same value whether closed or not (GeoJSON's first-last
/// duplication convention holds automatically). Any non-finite point -> whole ring 0 (no guessing at partial polygons).
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
