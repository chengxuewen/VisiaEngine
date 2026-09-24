//! TileSet scheduler (IO-13/14, tile-streaming Phase 1).
//!
//! visible() = pure slippy enumeration over a 3857 bbox; ensure() = sync load
//! of missing tiles into the LRU; decoded() = decode-through-cache.
//! Uses FileSource over the bundled 3×3 fixture tree (z10, x/y ∈ 0..2).

#![allow(clippy::float_cmp)]

use visiaengine_io_tiles::{TileId, TileSet};

fn fixture_root() -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/data/tiles").to_string()
}

/// bbox covering the whole z10 (0..2, 0..2) 3×3 fixture neighborhood:
/// world extent / 1024 tiles × 3 = the 3×3 block starting at tile (0,0).
fn bbox_3x3() -> (f64, f64, f64, f64) {
    // Union of tiles (0..2, 0..2): top-left tile (0,0) max corner → bottom-right
    // tile (2,2) min corner. Anchored on REAL tile bboxes (y flip-aware).
    let nw = TileId::new(10, 0, 0).expect("valid");
    let se = TileId::new(10, 2, 2).expect("valid");
    let (min_x, _, _, max_y) = nw.bbox();
    let (_, min_y, max_x, _) = se.bbox();
    (min_x, min_y, max_x, max_y)
}

// spec: IO-13
#[test]
fn visible_enumerates_exactly_the_covered_tiles() {
    let ids = TileSet::visible(bbox_3x3(), 10);
    assert_eq!(ids.len(), 9, "3×3 bbox at z10 → 9 tiles");
    assert!(ids.contains(&TileId::new(10, 0, 0).unwrap()));
    assert!(ids.contains(&TileId::new(10, 2, 2).unwrap()));
    assert!(
        !ids.contains(&TileId::new(10, 3, 1).unwrap()),
        "outside bbox"
    );
}

// spec: IO-13
#[test]
fn visible_single_tile_bbox() {
    // bbox exactly one tile: TileId::new(10,1,1).bbox()
    let t = TileId::new(10, 1, 1).expect("valid");
    let ids = TileSet::visible(t.bbox(), 10);
    assert_eq!(ids, vec![t], "bbox == one tile → exactly that tile");
}

// spec: IO-13
#[test]
fn visible_is_row_major_deterministic() {
    let a = TileSet::visible(bbox_3x3(), 10);
    let b = TileSet::visible(bbox_3x3(), 10);
    assert_eq!(a, b, "pure function: same input → same order");
    // row-major: y outer, x inner (stable paint order for callers)
    let first = &a[0];
    let second = &a[1];
    assert_eq!(first.y, second.y, "same row");
    assert_eq!(first.x + 1, second.x, "x increments within row");
}

// spec: IO-14
#[test]
fn ensure_loads_and_is_idempotent() {
    let mut ts = TileSet::new(Box::new(visiaengine_io_tiles::FileSource::new(
        fixture_root(),
    )))
    .expect("set");
    let ids = TileSet::visible(bbox_3x3(), 10);
    let stats1 = ts.ensure(&ids).expect("ensure");
    assert_eq!(stats1.loaded, 9, "first ensure loads all 9");
    assert_eq!(stats1.cached, 0);
    let stats2 = ts.ensure(&ids).expect("ensure 2nd");
    assert_eq!(stats2.loaded, 0, "second ensure = all cache hits");
    assert_eq!(stats2.cached, 9);
}

// spec: IO-14
#[test]
fn ensure_partial_missing_loads_only_missing() {
    let mut ts = TileSet::new(Box::new(visiaengine_io_tiles::FileSource::new(
        fixture_root(),
    )))
    .expect("set");
    let ids = TileSet::visible(bbox_3x3(), 10);
    ts.ensure(&ids[..3]).expect("partial");
    let stats = ts.ensure(&ids).expect("full after partial");
    assert_eq!(stats.loaded, 6, "only the 6 missing load");
    assert_eq!(stats.cached, 3);
}

// spec: IO-14
#[test]
fn decoded_via_cache_and_source_error_typed() {
    let mut ts = TileSet::new(Box::new(visiaengine_io_tiles::FileSource::new(
        fixture_root(),
    )))
    .expect("set");
    let id = TileId::new(10, 1, 1).expect("valid");
    ts.ensure(&[id]).expect("ensure");
    let decoded = ts.decoded(&id).expect("decoded through cache");
    assert!(!decoded.layers.is_empty(), "fixture tile has layers");
    // missing tile = typed source error, not panic
    let bad = TileId::new(10, 500, 500).expect("valid id but no file");
    let err = ts.ensure(&[bad]).unwrap_err();
    assert!(matches!(err, visiaengine_io_tiles::TilesError::Source(_)));
}
