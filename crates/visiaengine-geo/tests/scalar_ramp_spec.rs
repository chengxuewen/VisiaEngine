//! GEO-20: the visia:color-* extension keys = scalar -> color ramp (materialized at parse time, own naming beyond the D9 boundary).
//! Mechanism revision note: plan v1.1's original "256px LUT texture + WGSL sampling" was downgraded per ponytail --
//! the real shape of GIS attribute coloring is **per-feature constant color** (building height = one value per feature),
//! so a CPU lerp is complete; GPU LUT textures await the real per-vertex-field need (DEM/point cloud) (backlog).

use visiaengine_geo::{load_geojson, parse_geojson_lenient};

const ONE: &str = r##"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"height":40,"visia:color-column":"height","visia:color-lo":0,"visia:color-hi":50,"visia:color-low":"#00ff00","visia:color-high":"#ff0000"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}]}"##;

// spec: GEO-20
#[test]
fn scalar_lands_on_color_ramp() {
    let (doc, _) = parse_geojson_lenient(ONE.as_bytes()).unwrap();
    let s = &doc.features()[0].style;
    // t=(40-0)/50=0.8 → lerp(green,red,0.8)=(0.8,0.2,0)
    assert!((s.fill[0] - 0.8).abs() < 1e-5, "red channel rises with t");
    assert!((s.fill[1] - 0.2).abs() < 1e-5, "green channel falls with t");
    assert_eq!(s.fill[3], 1.0);
    // marker synced (point-piece consistency)
    assert_eq!(s.marker_color, s.fill);
}

// spec: GEO-20
#[test]
fn out_of_range_clamps_and_missing_column_falls_back() {
    // beyond the upper bound clamps to the pure high color
    let hi = ONE.replace("\"height\":40", "\"height\":999");
    let (doc, _) = parse_geojson_lenient(hi.as_bytes()).unwrap();
    assert_eq!(doc.features()[0].style.fill, [1.0, 0.0, 0.0, 1.0]);
    // missing column (referencing a nonexistent attribute) -> fall back to six-keys/default, no panic no silence-drop
    let miss = ONE.replace("\"height\":40,", "");
    let (doc, _) = parse_geojson_lenient(miss.as_bytes()).unwrap();
    assert_eq!(doc.features()[0].style.fill, [0.0, 0.45, 1.0, 1.0]);
    // invalid interval (hi <= lo) -> silently disabled
    let bad = ONE.replace("\"visia:color-hi\":50", "\"visia:color-hi\":0");
    let (doc, _) = parse_geojson_lenient(bad.as_bytes()).unwrap();
    assert_eq!(doc.features()[0].style.fill, [0.0, 0.45, 1.0, 1.0]);
}

// spec: GEO-20
#[test]
fn fixture_file_end_to_end() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/heights.geojson"
    );
    let doc = load_geojson(path).unwrap();
    let fills: Vec<[f32; 4]> = doc.features().iter().map(|f| f.style.fill).collect();
    assert_eq!(fills[0], [0.0, 1.0, 0.0, 1.0], "h0 = pure low color");
    assert_eq!(fills[1], [0.5, 0.5, 0.0, 1.0], "h25 = midpoint");
    assert_eq!(fills[2], [1.0, 0.0, 0.0, 1.0], "h50 = pure high color");
    assert_eq!(
        fills[3],
        [0.0, 0.45, 1.0, 1.0],
        "missing-column piece falls back to default blue"
    );
}

/// M0 exposure lock: the ramp is the reuse base of the point-cloud band (io-points coloring/E204) -- pub-face endpoints/monotonicity pinned.
// spec: GEO-20
#[test]
fn ramp_helper_exposed_for_host_consumers() {
    let fill_at = |v: f64| -> [f32; 4] {
        let mut a = visiaengine_core::AttrSet::new();
        let r = a.add_row();
        a.set_f64(r, "h", v);
        a.set_str(r, "visia:color-column", "h");
        a.set_f64(r, "visia:color-lo", 0.0);
        a.set_f64(r, "visia:color-hi", 10.0);
        a.set_str(r, "visia:color-low", "#000000");
        a.set_str(r, "visia:color-high", "#ffffff");
        let mut s = visiaengine_geo::StyleRecord::default();
        visiaengine_geo::apply_scalar_ramp(&a, r, &mut s);
        s.fill
    };
    assert_eq!(
        fill_at(0.0),
        [0.0, 0.0, 0.0, 1.0],
        "lower endpoint bit-exact"
    );
    assert_eq!(
        fill_at(10.0),
        [1.0, 1.0, 1.0, 1.0],
        "upper endpoint bit-exact"
    );
    assert_eq!(fill_at(5.0), [0.5, 0.5, 0.5, 1.0], "midpoint lerp");
    assert!(fill_at(3.0)[0] < fill_at(7.0)[0], "monotonic");
    assert_eq!(
        fill_at(99.0),
        [1.0, 1.0, 1.0, 1.0],
        "out-of-range clamp face locked too"
    );
}
