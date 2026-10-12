//! GEO-21/22: map-plane measurement (3857 units, the interaction slice [E3D:4xx]; geodesic belongs to the post-D9
//! "coordinate-system completeness" domain; this clause freezes the plane semantics -- 3857 distance = the Mercator distorted value, documented).

use visiaengine_core::Vec3;
use visiaengine_geo::{planar_distance, ring_area};

#[must_use]
fn p(x: f64, y: f64) -> Vec3 {
    Vec3::new(x, y, 0.0)
}

// spec: GEO-21
#[test]
fn distance_is_planar_polyline_sum() {
    // 3-4-5 right triangle (unit on the 3857-meter domain)
    assert!((planar_distance(&[p(0.0, 0.0), p(300.0, 400.0)]) - 500.0).abs() < 1e-9);
    // polyline sum
    let line = [p(0.0, 0.0), p(300.0, 400.0), p(300.0, 400.0)];
    assert!((planar_distance(&line) - 500.0).abs() < 1e-9);
    // degenerate: empty / single point = 0
    assert_eq!(planar_distance(&[]), 0.0);
    assert_eq!(planar_distance(&[p(1.0, 1.0)]), 0.0);
    // non-finite points skipped, not segmented (NaN propagation defense on 3857 large coordinates)
    let bad = [p(0.0, 0.0), p(f64::NAN, 0.0), p(300.0, 400.0)];
    assert!((planar_distance(&bad) - 500.0).abs() < 1e-9);
}

// spec: GEO-22
#[test]
fn area_is_shoelace_absolute_and_orientation_free() {
    let square = [p(0.0, 0.0), p(2.0, 0.0), p(2.0, 2.0), p(0.0, 2.0)];
    assert!((ring_area(&square) - 4.0).abs() < 1e-9);
    // reversed ring, same value
    let rev: Vec<Vec3> = square.iter().rev().copied().collect();
    assert!((ring_area(&rev) - 4.0).abs() < 1e-9);
    // closed duplicated-point input and unclosed input share the area (the GeoJSON first-last duplication convention)
    let mut c2 = square.to_vec();
    c2.push(square[0]);
    assert!((ring_area(&c2) - ring_area(&square)).abs() < 1e-9);
    // degenerate: collinear < 3 vertices = 0; a ring with any non-finite point = 0 (no guessing at the meaningless)
    assert_eq!(ring_area(&[p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)]), 0.0);
    assert_eq!(ring_area(&[p(0.0, 0.0), p(1.0, 0.0)]), 0.0);
    assert_eq!(ring_area(&[]), 0.0);
    let bad = [p(0.0, 0.0), p(f64::INFINITY, 2.0), p(2.0, 2.0), p(0.0, 2.0)];
    assert_eq!(ring_area(&bad), 0.0);
}
