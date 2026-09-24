//! MVT → local geometry mapping (GEO-27, super-band B3).
//! Fixture B (point + polygon layers) reused; mapping verified against the
//! tile bbox anchor + y-flip + extent scale, then the shifted() D7 view.

#![allow(clippy::float_cmp)]

use visiaengine_io_tiles::{GeoTile, TileGeom, TileId, decode_tile};

// Fixture B: poi layer (point at tile-local (2048,1024), extent 4096)
//            + land layer (polygon ring covering the full tile).
const FIXTURE_B: &[u8] = &[
    26, 64, 10, 3, 112, 111, 105, 18, 17, 8, 7, 24, 1, 18, 4, 0, 0, 1, 1, 34, 5, 9, 128, 32, 128,
    16, 26, 4, 110, 97, 109, 101, 26, 4, 114, 97, 110, 107, 26, 4, 111, 112, 101, 110, 34, 7, 10,
    5, 116, 111, 119, 101, 114, 34, 2, 32, 42, 34, 2, 56, 1, 40, 128, 32, 48, 2, 26, 51, 10, 4,
    108, 97, 110, 100, 18, 24, 8, 8, 24, 3, 18, 2, 0, 0, 34, 14, 9, 0, 0, 26, 128, 64, 0, 0, 128,
    64, 255, 63, 0, 15, 26, 4, 107, 105, 110, 100, 34, 6, 10, 4, 97, 114, 101, 97, 40, 128, 32, 48,
    2,
];

// spec: GEO-27
#[test]
fn point_maps_to_world_center_z1() {
    // z=1 tile (0,0) covers west/north quarter; point (2048,1024)/4096 = center
    // of that tile in tile px → world center of that tile.
    let tile = decode_tile(FIXTURE_B).expect("decode");
    let id = TileId::new(1, 0, 0).expect("valid");
    let gt = GeoTile::from_layers(id, &tile.layers);
    assert_eq!(gt.features.len(), 2, "point + polygon");

    let (min_x, min_y, max_x, max_y) = id.bbox();
    assert_eq!(gt.origin, [min_x, min_y, 0.0]);
    // find the point feature
    let pt = gt
        .features
        .iter()
        .find(|f| matches!(f.geom, TileGeom::Point(_)))
        .expect("point feature");
    match &pt.geom {
        TileGeom::Point([x, y]) => {
            let h = max_y - min_y;
            assert!(
                (*x - (min_x + max_x) / 2.0).abs() < 1e-6,
                "x center of tile"
            );
            // MVT y=1024/4096 (top quarter, y-down) → y-flip → 3/4 up from min_y
            assert!(
                (*y - (min_y + 0.75 * h)).abs() < 1e-6,
                "y at 3/4 after flip"
            );
        }
        other => panic!("expected point, got {other:?}"),
    }
    // attrs: resolved strings survive the mapping
    assert_eq!(pt.attrs.get("name").map(String::as_str), Some("tower"));
}

// spec: GEO-27
#[test]
fn y_flip_places_top_row_points_north() {
    // A point at tile px y=0 (top edge) must land at the tile's max_y (north).
    // Build via fixture B's polygon corner instead: first ring vertex (0,0) is
    // the TOP-LEFT corner → world (min_x, max_y).
    let tile = decode_tile(FIXTURE_B).expect("decode");
    let id = TileId::new(1, 0, 0).expect("valid");
    let gt = GeoTile::from_layers(id, &tile.layers);
    let poly = gt
        .features
        .iter()
        .find(|f| matches!(f.geom, TileGeom::Poly(_)))
        .expect("poly feature");
    match &poly.geom {
        TileGeom::Poly(ps) => {
            let (min_x, _, _, max_y) = id.bbox();
            assert!((ps[0][0] - min_x).abs() < 1e-6, "top-left x = min_x");
            assert!(
                (ps[0][1] - max_y).abs() < 1e-6,
                "top-left y = max_y (y-flip)"
            );
        }
        other => panic!("expected poly, got {other:?}"),
    }
}

// spec: GEO-27
#[test]
fn shifted_view_recenters_to_origin() {
    let tile = decode_tile(FIXTURE_B).expect("decode");
    let id = TileId::new(1, 0, 0).expect("valid");
    let gt = GeoTile::from_layers(id, &tile.layers);
    let shifted = gt.shifted();
    assert_eq!(shifted.len(), 2);
    for (g, _) in &shifted {
        match g {
            TileGeom::Point([x, y]) => {
                assert!(
                    *x >= 0.0 && *y >= 0.0,
                    "local coords non-negative (D7 rebase)"
                );
            }
            TileGeom::Poly(ps) => {
                for p in ps {
                    assert!(p[0] >= 0.0 && p[1] >= 0.0, "poly local non-negative");
                }
            }
            _ => {}
        }
    }
    // shifted point = world − origin
    if let Some((TileGeom::Point([x, y]), _)) = shifted
        .iter()
        .find(|(g, _)| matches!(g, TileGeom::Point(_)))
    {
        let (min_x, min_y, max_x, max_y) = id.bbox();
        let h = max_y - min_y;
        assert!((*x - ((max_x - min_x) / 2.0)).abs() < 1e-6);
        assert!((*y - (0.75 * h)).abs() < 1e-6);
    }
}
