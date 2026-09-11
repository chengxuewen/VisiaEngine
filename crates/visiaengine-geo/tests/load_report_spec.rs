//! GEO-15/16：加载报告与修复策略契约（[E3D:A4] 移植）。
//! 类别计数=丢弃时按类型内部分类（typed GeoError 变体映射），非 reason 字符串反解。

use visiaengine_geo::{RepairPolicy, parse_geojson, parse_geojson_lenient, parse_geojson_with};

/// 五类脏源各一件 + 两件正常（GEO-15 主 fixture）
const DIRTY_FC: &str = r#"{
  "type": "FeatureCollection",
  "features": [
    {"type":"Feature","properties":{"name":"ok1"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}},
    {"type":"Feature","properties":{"name":"oob"},"geometry":{"type":"Point","coordinates":[10.0,91.0]}},
    {"type":"Feature","properties":{"name":"nan"},"geometry":{"type":"Point","coordinates":[1e400,0.0]}},
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
        .expect("Lenient：单件脏不得拖垮整文档");
    // 仅两件正常几何存活
    assert_eq!(doc.features().len(), 2);
    assert_eq!(
        doc.features()
            .iter()
            .filter_map(|f| f.name.as_deref())
            .collect::<Vec<_>>(),
        ["ok1", "ok2"]
    );
    // 每类恰好命中一次（分类正确性 > 总数）
    assert_eq!(rep.dropped_out_of_bounds, 1, "lat=91° 出界域");
    assert_eq!(rep.dropped_non_finite, 1, "1e400 → inf 非有限");
    assert_eq!(rep.dropped_unsupported, 1, "MultiPolygon 属 H2 拆分面未就绪");
    assert_eq!(rep.dropped_null_geometry, 1, "geometry:null 不得再静默");
    assert_eq!(rep.dropped_nested_collection, 1, "GC 嵌套 ≥2 层拒绝计数");
    assert_eq!(rep.total_dropped(), 5);
}

// spec: GEO-16
#[test]
fn fastfail_entry_keeps_first_error_semantics() {
    // 旧入口 = FastFail：遇首件脏几何即整文档 Err（GEO-08 既有契约不变）
    assert!(parse_geojson(DIRTY_FC.as_bytes()).is_err());
    assert!(parse_geojson_with(DIRTY_FC.as_bytes(), RepairPolicy::FastFail).is_err());
    // 纯干净文档双入口结果一致（报告全零）
    let clean = r#"{"type":"FeatureCollection","features":[
      {"type":"Feature","properties":{},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}]}"#;
    let (d2, r2) = parse_geojson_lenient(clean.as_bytes()).unwrap();
    assert_eq!(r2.total_dropped(), 0);
    assert_eq!(
        parse_geojson(clean.as_bytes()).unwrap().features().len(),
        d2.features().len()
    );
}
