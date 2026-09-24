//! Slippy tile math invariants (IO-11, super-band B3).

#![allow(clippy::float_cmp)]

use visiaengine_io_tiles::{TileId, WORLD_EXTENT};

// spec: IO-11
#[test]
fn z0_is_whole_world() {
    let t = TileId::new(0, 0, 0).expect("z0 valid");
    let (min_x, min_y, max_x, max_y) = t.bbox();
    assert!((min_x - -WORLD_EXTENT / 2.0).abs() < 1e-6);
    assert!((max_x - WORLD_EXTENT / 2.0).abs() < 1e-6);
    assert!((min_y - -WORLD_EXTENT / 2.0).abs() < 1e-6);
    assert!((max_y - WORLD_EXTENT / 2.0).abs() < 1e-6);
}

// spec: IO-11
#[test]
fn out_of_range_xy_rejected() {
    assert!(TileId::new(0, 1, 0).is_none(), "z0 has only tile 0,0");
    assert!(TileId::new(0, 0, 1).is_none());
    assert!(TileId::new(1, 2, 0).is_none(), "z1 has 2x2");
    assert!(TileId::new(2, 3, 4).is_none(), "y must be < 4");
    assert!(TileId::new(2, 3, 3).is_some(), "corner tile valid");
}

// spec: IO-11
#[test]
fn tile_count_doubles_per_zoom() {
    // z=1: 2x2 grid covering the world; each tile half the extent.
    let t00 = TileId::new(1, 0, 0).expect("valid");
    let (min_x, min_y, max_x, max_y) = t00.bbox();
    assert!(
        (max_x - 0.0).abs() < 1e-6,
        "z1 x=0 right edge = prime meridian"
    );
    assert!((min_x - -WORLD_EXTENT / 2.0).abs() < 1e-6);
    // slippy y=0 = northmost row → max_y = +half
    assert!((max_y - WORLD_EXTENT / 2.0).abs() < 1e-6);
    assert!((min_y - 0.0).abs() < 1e-6);
}

// spec: IO-11
#[test]
fn neighbor_tiles_share_edges() {
    // Adjacent tiles must agree on shared edges (no seams in the math).
    let a = TileId::new(3, 1, 1).expect("valid");
    let right = TileId::new(3, 2, 1).expect("valid");
    let below = TileId::new(3, 1, 2).expect("valid");
    let (_, ay0, ax1, _) = a.bbox();
    let (rx0, _, _, _) = right.bbox();
    let (_, _, _, by1) = below.bbox();
    assert!((ax1 - rx0).abs() < 1e-6, "right neighbor shares east edge");
    assert!(
        (ay0 - by1).abs() < 1e-6,
        "tile below shares south edge (3857 y north-up)"
    );
}

// spec: IO-11
#[test]
fn london_lands_in_expected_tile() {
    // London ~ (-0.1276°, 51.5072°). At z=3, Europe/NW-Europe tile is (4, 2).
    // x = (lon+180)/360 * 2^z = (179.8724/360)*8 = 3.997 → 3? Compute exactly:
    let lon: f64 = -0.1276;
    let n = 8.0;
    let xf = (lon + 180.0) / 360.0 * n;
    assert!((3.0..4.0).contains(&xf), "x float = {xf}");
    let x = xf.floor() as u32;
    let t = TileId::new(3, x, 2).expect("valid");
    let (min_x, _, max_x, _) = t.bbox();
    let lon_x = lon / 180.0 * (WORLD_EXTENT / 2.0);
    assert!(
        min_x <= lon_x && lon_x < max_x,
        "bbox must contain the point"
    );
}
