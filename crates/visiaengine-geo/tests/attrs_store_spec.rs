//! GEO-17/18: feature attribute columnized storage + styles read through columns ([E3D:A1/A5] port).
//! Core promise: attributes are no longer discarded after parsing -- the host can query by row; the style hot path has zero Value matching.

use visiaengine_geo::{RepairPolicy, parse_geojson_with};

const MIXED: &str = r##"{
  "type": "FeatureCollection",
  "features": [
    {"type":"Feature","properties":{
        "name":"Tower A","fill":"#ff0000","fill-opacity":0.5,
        "height":12.5,"active":true,"tags":["a","b"],"meta":{"k":1},"void":null},
     "geometry":{"type":"Point","coordinates":[10.0,50.0]}},
    {"type":"Feature","properties":{"name":"Block","fill":"#00ff00"},
     "geometry":{"type":"GeometryCollection","geometries":[
        {"type":"Point","coordinates":[11.0,51.0]},
        {"type":"Point","coordinates":[12.0,52.0]}]}},
    {"type":"Feature","properties":{},
     "geometry":{"type":"Point","coordinates":[13.0,53.0]}}
  ]
}"##;

// spec: GEO-17
#[test]
fn scalar_props_land_in_columns_rows_align_features() {
    let (doc, _rep) = parse_geojson_with(MIXED.as_bytes(), RepairPolicy::Lenient).unwrap();
    // row alignment = feature count (GC flattens per the existing strategy into one row per child piece, GEO-06 semantics)
    assert_eq!(doc.features().len(), 4);
    assert_eq!(doc.attrs().len(), 4);
    // typed host query entries
    assert_eq!(doc.attr_f64(0, "height"), Some(12.5));
    assert_eq!(doc.attr_bool(0, "active"), Some(true));
    assert_eq!(doc.attr_str(0, "name"), Some("Tower A"));
    // nested/array/null values do not enter columns (skipped rather than distorted)
    assert_eq!(doc.attr_str(0, "tags"), None);
    assert_eq!(doc.attr_str(0, "meta"), None);
    assert_eq!(doc.attr_f64(0, "void"), None);
    // missing key != zero value (the GEO-17 semantic core; row 3 = the empty-props feature)
    assert_eq!(doc.attr_f64(3, "height"), None);
}

// spec: GEO-17
#[test]
fn gc_children_inherit_feature_props() {
    let (doc, _) = parse_geojson_with(MIXED.as_bytes(), RepairPolicy::Lenient).unwrap();
    // feature[1]'s props are copied to all of its GC child rows (rows 1/2 are "Block" too)
    assert_eq!(
        doc.features()[1].style.fill[1],
        1.0,
        "#00ff00 survives on child piece 0"
    );
    assert_eq!(
        doc.features()[2].style.fill[1],
        1.0,
        "same props covers child piece 1"
    );
    assert_eq!(doc.attr_str(1, "name"), Some("Block"));
    assert_eq!(doc.attr_str(2, "name"), Some("Block"));
}

// spec: GEO-18
#[test]
fn style_reads_through_columns_identical_to_legacy() {
    let (doc, _) = parse_geojson_with(MIXED.as_bytes(), RepairPolicy::Lenient).unwrap();
    let s0 = &doc.features()[0].style;
    // the six-key results match pre-migration item by item (fill hex -> r = 1.0; opacity clamp; absent keys fall back to defaults)
    assert!((s0.fill[0] - 1.0).abs() < 1e-6 && s0.fill[1] < 1e-6);
    assert!((s0.fill_opacity - 0.5).abs() < 1e-6);
    assert_eq!(
        s0.stroke,
        [1.0, 1.0, 1.0, 1.0],
        "missing stroke falls back to default, not height interference"
    );
    // beyond the style keys, the remaining scalar attributes stay queryable (style consumption no longer swallows attributes)
    assert_eq!(doc.attr_f64(0, "fill-opacity"), Some(0.5));
    // the park.geojson regression family lives in geo_spec/tess_style_spec (the existing 14 style assertions = the migration net)
}

// spec: GEO-19
#[test]
fn v8_paint_alias_keys_are_accepted_as_fallback() {
    // D9: MapLibre v8 static paint key aliases; simplestyle primary keys take precedence, aliases as fallback
    let v8 = r##"{
      "type":"FeatureCollection",
      "features":[{"type":"Feature","properties":{
          "fill-color":"#0000ff","line-color":"#ff0000","line-width":2.5,
          "circle-color":"#00ff00","circle-radius":7.0},"geometry":
        {"type":"Point","coordinates":[10.0,50.0]}}]
    }"##;
    let (doc, _) = parse_geojson_with(v8.as_bytes(), RepairPolicy::Lenient).unwrap();
    let s = &doc.features()[0].style;
    assert!((s.fill[2] - 1.0).abs() < 1e-6, "fill-color→fill");
    assert!((s.stroke[0] - 1.0).abs() < 1e-6, "line-color→stroke");
    assert!(
        (s.stroke_width_px - 2.5).abs() < 1e-6,
        "line-width→stroke-width"
    );
    assert!(
        (s.marker_color[1] - 1.0).abs() < 1e-6,
        "circle-color→marker-color"
    );
    assert!(
        (s.radius_px - 7.0).abs() < 1e-6,
        "circle-radius→marker-radius"
    );
}

// spec: GEO-19
#[test]
fn simplestyle_primary_wins_over_v8_alias() {
    // primary key + alias coexist -> the primary key wins (the priority rule nailed down, eradicating behavioral ambiguity)
    let both = r##"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"fill":"#ff0000","fill-color":"#0000ff"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}]}"##;
    let (doc, _) = parse_geojson_with(both.as_bytes(), RepairPolicy::Lenient).unwrap();
    assert!(
        (doc.features()[0].style.fill[0] - 1.0).abs() < 1e-6,
        "the fill primary key beats the fill-color alias"
    );
}

// spec: GEO-25
#[test]
fn text_style_keys_parsed_column_template_and_literal() {
    // "{name}" = column-reference template; bare string = constant; missing key = None (no label by default)
    let fc = r##"{"type":"FeatureCollection","features":[
      {"type":"Feature","properties":{"text-field":"{name}","text-size":18,
        "text-color":"#112233","name":"Gate"},"geometry":
        {"type":"Point","coordinates":[10.0,50.0]}},
      {"type":"Feature","properties":{"text-field":"Static"},"geometry":
        {"type":"Point","coordinates":[11.0,51.0]}}
    ]}"##;
    let (doc, _) = parse_geojson_with(fc.as_bytes(), RepairPolicy::Lenient).unwrap();
    let s0 = &doc.features()[0].style;
    assert_eq!(
        s0.text_field.as_deref(),
        Some("name"),
        "curly-brace unwrap = column name"
    );
    assert!((s0.text_size_px - 18.0).abs() < 1e-6);
    assert_eq!(
        s0.text_color,
        [
            0x11 as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x33 as f32 / 255.0,
            1.0
        ],
        "text-color reuses parse_color (host sRGB face, same system as GEO-12)"
    );
    assert_eq!(
        doc.features()[1].style.text_field.as_deref(),
        Some("Static"),
        "bare string = constant as-is"
    );
    assert!(
        (doc.features()[1].style.text_size_px - 14.0).abs() < 1e-6,
        "default 14px"
    );
    // no text key (all existing fixture roads) = None, zero regression
    assert!(doc.features()[1].style.text_field.is_some());
    let plain = r##"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"fill":"#ff0000"},"geometry":{"type":"Point","coordinates":[1.0,2.0]}}]}"##;
    let (d2, _) = parse_geojson_with(plain.as_bytes(), RepairPolicy::Lenient).unwrap();
    assert!(
        d2.features()[0].style.text_field.is_none(),
        "missing key = None"
    );
}
