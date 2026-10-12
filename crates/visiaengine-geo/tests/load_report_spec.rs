//! GEO-15/16: load-report and repair-policy contract ([E3D:A4] port).
//! Category counts = typed internal classification at discard time (typed GeoError variant mapping), not reason-string parsing back.

use visiaengine_geo::{RepairPolicy, parse_geojson, parse_geojson_lenient, parse_geojson_with};

/// One piece per each of the five dirty sources + two normal (the GEO-15 main fixture)
const DIRTY_FC: &str = r#"{
  "type": "FeatureCollection",
  "features": [
    {"type":"Feature","properties":{"name":"ok1"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}},
    {"type":"Feature","properties":{"name":"oob"},"geometry":{"type":"Point","coordinates":[10.0,91.0]}},
    {"type":"Feature","properties":{"name":"mpoly"},"geometry":{"type":"MultiPolygon","coordinates":[[[[0,0],[1,1],[2,0],[0,0]]]]}},
    {"type":"Feature","properties":{"name":"nullg"},"geometry":null},
    {"type":"Feature","properties":{"name":"nest"},"geometry":{"type":"GeometryCollection","geometries":[{"type":"GeometryCollection","geometries":[{"type":"Point","coordinates":[1.0,1.0]}]}]}},
    {"type":"Feature","properties":{"name":"ok2"},"geometry":{"type":"LineString","coordinates":[[0.0,0.0],[1.0,1.0]]}}
  ]
}"#;

// spec: GEO-15
#[test]
fn lenient_drops_are_categorized_and_counted() {
    let (doc, rep) = parse_geojson_lenient(DIRTY_FC.as_bytes())
        .expect("Lenient: one dirty piece must not drag down the whole document");
    // only the two normal geometries survive
    assert_eq!(doc.features().len(), 2);
    assert_eq!(
        doc.features()
            .iter()
            .filter_map(|f| f.name.as_deref())
            .collect::<Vec<_>>(),
        ["ok1", "ok2"]
    );
    // each class hits exactly once (classification correctness > totals)
    assert_eq!(rep.dropped_out_of_bounds, 1, "lat=91° out of domain");
    // the JSON number-domain pre-check lives at the geojson crate document layer (see the boundary assertion below): this class is unreachable at the parse entry,
    // the field is reserved for the host path calling web_mercator directly
    assert_eq!(rep.dropped_non_finite, 0);
    assert_eq!(
        rep.dropped_unsupported, 1,
        "MultiPolygon belongs to the not-ready H2 split surface"
    );
    assert_eq!(
        rep.dropped_null_geometry, 1,
        "geometry:null must not go silent again"
    );
    assert_eq!(
        rep.dropped_nested_collection, 1,
        "GC nested >= 2 levels reject count"
    );
    assert_eq!(rep.total_dropped(), 4);
    // GEO-15 boundary: document-level errors (a JSON number beyond the f64 domain = syntax layer) yield a whole-file Err under both policies
    let doc_level = r#"{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[1e400,0.0]}}]}"#;
    assert!(parse_geojson_lenient(doc_level.as_bytes()).is_err());
}

// spec: GEO-16
#[test]
fn fastfail_entry_keeps_first_error_semantics() {
    // the old entry = FastFail: the first dirty geometry rejects the whole document as Err (the existing GEO-08 contract unchanged)
    assert!(parse_geojson(DIRTY_FC.as_bytes()).is_err());
    assert!(parse_geojson_with(DIRTY_FC.as_bytes(), RepairPolicy::FastFail).is_err());
    // a purely clean document gives equal results through both entries (all-zero report)
    let clean = r#"{"type":"FeatureCollection","features":[
      {"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}]}"#;
    let (d2, r2) = parse_geojson_lenient(clean.as_bytes()).unwrap();
    assert_eq!(r2.total_dropped(), 0);
    assert_eq!(
        parse_geojson(clean.as_bytes()).unwrap().features().len(),
        d2.features().len()
    );
}
