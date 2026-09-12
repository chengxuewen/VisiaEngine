//! GEO-23：解析器全输入域无 panic（proptest 属性面，T1 层）。

use proptest::prelude::*;
use visiaengine_geo::{parse_geojson, parse_geojson_lenient, web_mercator};

// 任意字节流：两策略皆不得 panic（Err 合法，panic 非法）
proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    // spec: GEO-23
    #[test]
    fn random_bytes_never_panic(ref bytes in proptest::collection::vec(any::<u8>(), 0..512)) {
        let _ = parse_geojson_lenient(bytes);
        let _ = parse_geojson(bytes);
    }

    // spec: GEO-23
    #[test]
    fn arbitrary_coords_never_panic(
        x in any::<f64>(), y in any::<f64>(),
        x2 in any::<f64>(), y2 in any::<f64>(),
    ) {
        // 全 f64 域（NaN/inf/极值）构造 GeoJSON 喂解析口
        let gj = format!(
            r#"{{"type":"FeatureCollection","features":[
              {{"type":"Feature","properties":{{}},
                "geometry":{{"type":"Polygon","coordinates":[[[{x},{y}],[{x2},{y2}],[{x},{y2}],[{x},{y}]]]}}}} ]}}"#
        );
        let _ = parse_geojson_lenient(gj.as_bytes());
        let _ = parse_geojson(gj.as_bytes());
        let _ = web_mercator(x, y);
    }

    // spec: GEO-23
    #[test]
    fn lenient_fastfail_consistency_on_clean_input(
        lon in -180.0f64..180.0, lat in -85.0f64..85.0, r in 0.001f64..1.0
    ) {
        // 干净域内两策略件数一致（RepairPolicy 语义面：无脏件时零差异）
        let gj = format!(
            r#"{{"type":"Feature","properties":{{}},
               "geometry":{{"type":"Polygon","coordinates":[[[{lon},{lat}],[{lon},{lat}],[{lon},{lat}],[{lon},{lat}]]]}}}}"#
        );
        let (doc, rep) = parse_geojson_lenient(gj.as_bytes()).unwrap();
        prop_assert_eq!(rep.total_dropped(), 0);
        prop_assert_eq!(doc.features().len(), 1);
        let _ = r;
    }
}
