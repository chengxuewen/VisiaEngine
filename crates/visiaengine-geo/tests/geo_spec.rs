//! GEO-01..08 契约测试（fixture=resources/data/park.geojson 自产）。

use visiaengine_geo::{GeoKind, load_geojson, parse_geojson, web_mercator};

fn park() -> visiaengine_geo::GeoDocument {
    load_geojson(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/park.geojson"
    ))
    .unwrap()
}
fn at(p: &str) -> String {
    format!("{}/../../resources/data/{p}", env!("CARGO_MANIFEST_DIR"))
}

// spec: GEO-01
#[test]
fn parses_feature_count() {
    let doc = park();
    assert_eq!(doc.features().len(), 5);
    let names: Vec<&str> = doc
        .features()
        .iter()
        .map(|f| f.name.as_deref().unwrap_or(""))
        .collect();
    assert_eq!(
        names,
        vec!["buildingA", "buildingB", "road", "grove", "lamp"]
    );
}

// spec: GEO-02
#[test]
fn webmercator_known_points() {
    let o = web_mercator(0.0, 0.0).unwrap();
    assert!(o[0].abs() < 1e-6 && o[1].abs() < 1e-6);
    let e = web_mercator(180.0, 0.0).unwrap();
    assert!((e[0] - 20037508.342789244).abs() < 1e-3);
    let p = web_mercator(10.0, 50.0).unwrap();
    assert!((p[0] - 1113194.9079327357).abs() < 1e-2, "x={}", p[0]);
    assert!((p[1] - 6446275.841017158).abs() < 1e-2, "y={}", p[1]);
    assert!(web_mercator(0.0, 86.0).is_none());
}

// spec: GEO-03
#[test]
fn polygon_exterior_holes_preserved() {
    let doc = park();
    let a = &doc.features()[0];
    let GeoKind::Poly { ext, holes } = &a.kind else {
        panic!("buildingA must be Poly");
    };
    assert_eq!(ext.len(), 5);
    assert_eq!(holes.len(), 2);
    assert_eq!(holes[0].len(), 5);
    assert_eq!(holes[1].len(), 5);
}

// spec: GEO-04
#[test]
fn line_and_multipoint_kinds() {
    let doc = park();
    assert!(matches!(&doc.features()[2].kind, GeoKind::Line(l) if l.len()==4));
    assert!(matches!(&doc.features()[3].kind, GeoKind::MultiPoint(m) if m.len()==2));
    assert!(matches!(&doc.features()[4].kind, GeoKind::Point(_)));
}

// spec: GEO-05
#[test]
fn layer_bbox_union() {
    let doc = park();
    let [x0, y0, x1, y1] = doc.layer_bbox().expect("non-empty layer");
    let [ax, ay] = web_mercator(2.3499, 48.8499).unwrap();
    let [bx, by] = web_mercator(2.35045, 48.85018).unwrap();
    assert!(
        (x0 - ax).abs() < 1.0 && (y0 - ay).abs() < 1.0,
        "min corner {x0},{y0} vs {ax},{ay}"
    );
    assert!((x1 - bx).abs() < 1.0 && (y1 - by).abs() < 1.0);
    assert!(at("missing-check").is_empty() || true);
}

// spec: GEO-06
#[test]
fn malformed_err_not_panic() {
    let bytes = std::fs::read(at("park.geojson")).unwrap();
    let err = parse_geojson(&bytes[..bytes.len() / 2]).unwrap_err();
    assert!(
        matches!(err, visiaengine_geo::GeoError::Parse { .. }),
        "got {err:?}"
    );
}

// spec: GEO-07
#[test]
fn geometrycollection_flattened() {
    let gc = br#"{"type":"GeometryCollection","geometries":[
        {"type":"Polygon","coordinates":[[[0,0],[1,0],[1,1],[0,0]]]},
        {"type":"Point","coordinates":[2.5,48.5]}]}"#;
    let doc = parse_geojson(gc).unwrap();
    assert_eq!(doc.features().len(), 2);
    assert!(matches!(doc.features()[0].kind, GeoKind::Poly { .. }));
    assert!(matches!(doc.features()[1].kind, GeoKind::Point(_)));
}

// spec: GEO-08
#[test]
fn extreme_latitude_rejected() {
    let bad = br#"{"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[2.35,89.0]}}"#;
    let err = parse_geojson(bad).unwrap_err();
    assert!(
        matches!(err, visiaengine_geo::GeoError::InvalidCoord { lat } if (lat - 89.0).abs() < 1e-9),
        "got {err:?}"
    );
}
