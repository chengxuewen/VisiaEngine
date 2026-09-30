//! V2.1 curve spec: Bezier/CatmullRom sampling + tube/ribbon mesh invariants.

use visiaengine_render::curve::{ribbon_xz, sample_bezier, sample_catmull_rom, tube};

// spec: REND-47
#[test]
fn bezier_endpoints_exact() {
    let p = [[0.0; 3], [1.0, 0.0, 0.0], [1.0, 0.0, 1.0], [2.0, 0.0, 1.0]];
    let pts = sample_bezier(&p, 32);
    assert_eq!(pts.len(), 32);
    assert_eq!(pts[0], [0.0; 3], "t=0 endpoint exact");
    assert_eq!(pts[31], p[3], "t=1 endpoint exact");
}

// spec: REND-47
#[test]
fn catmull_rom_passes_through_knots() {
    let knots = [
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [2.0, 0.0, 0.0],
        [3.0, 1.0, 0.0],
    ];
    // t=0 of segment i must equal knot i
    for (i, knot) in knots.iter().enumerate().take(3) {
        let p = visiaengine_render::curve::catmull_rom(&knots, i, 0.0);
        assert!(
            (p[0] - knot[0]).abs() < 1e-6
                && (p[1] - knot[1]).abs() < 1e-6
                && (p[2] - knot[2]).abs() < 1e-6,
            "seg {i} t=0 != knot {i}"
        );
    }
    let pts = sample_catmull_rom(&knots, 16);
    // 3 segs × (16-1) + 1 final = 46
    assert_eq!(pts.len(), 3 * 15 + 1);
}

// spec: REND-47
#[test]
fn tube_welded_ring_topology_and_radius() {
    // straight line along X: every ring must be radius away from the axis
    let center: Vec<[f32; 3]> = (0..16).map(|i| [i as f32, 0.0, 0.0]).collect();
    let (pos, idx) = tube(&center, 0.5, 8);
    assert_eq!(pos.len(), 16 * 8);
    assert_eq!(idx.len(), 15 * 8 * 6);
    for (i, p) in pos.iter().enumerate() {
        let ring = i / 8;
        let axis_x = center[ring][0];
        let dx = p[0] - axis_x;
        let r2 = dx * dx + p[1] * p[1] + p[2] * p[2];
        assert!(
            (r2 - 0.25).abs() < 1e-4,
            "ring {ring} vertex off radius: {r2}"
        );
    }
    // index bounds
    assert!(idx.iter().max().copied().unwrap_or(0) < pos.len() as u32);
}

// spec: REND-47
#[test]
fn ribbon_flat_and_wide() {
    let center: Vec<[f32; 3]> = (0..8).map(|i| [i as f32, 2.0, 0.0]).collect();
    let (pos, idx) = ribbon_xz(&center, 2.0);
    assert_eq!(pos.len(), 16);
    assert_eq!(idx.len(), 7 * 6);
    for p in &pos {
        assert!((p[1] - 2.0).abs() < 1e-6, "ribbon must keep center y");
    }
    // width: pair-wise z spread = 2.0 (±1 from center z=0)
    for pair in pos.chunks(2) {
        assert!((pair[0][2] - 1.0).abs() < 1e-6 && (pair[1][2] + 1.0).abs() < 1e-6);
    }
}

// spec: REND-47
#[test]
fn degenerate_inputs_empty() {
    assert!(tube(&[[0.0; 3]], 0.5, 8).0.is_empty());
    assert!(tube(&[[0.0; 3], [1.0, 0.0, 0.0]], -1.0, 8).0.is_empty());
    assert!(ribbon_xz(&[[0.0; 3]], 1.0).0.is_empty());
    assert!(sample_bezier(&[[0.0; 3]; 4], 1).is_empty());
}
