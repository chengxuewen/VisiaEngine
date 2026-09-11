//! GEO-17/18：feature 属性列化存储 + 样式经列读取（[E3D:A1/A5] 移植）。
//! 核心承诺：解析后属性不再丢弃——宿主可按行查；样式热路径零 Value 匹配。

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
    // 行对齐=feature 数（GC 按既有策略展平为每子件一行，GEO-06 语义）
    assert_eq!(doc.features().len(), 4);
    assert_eq!(doc.attrs().len(), 4);
    // 类型化宿主查询口
    assert_eq!(doc.attr_f64(0, "height"), Some(12.5));
    assert_eq!(doc.attr_bool(0, "active"), Some(true));
    assert_eq!(doc.attr_str(0, "name"), Some("Tower A"));
    // 嵌套/数组/null 值不入列（跳过而非失真）
    assert_eq!(doc.attr_str(0, "tags"), None);
    assert_eq!(doc.attr_str(0, "meta"), None);
    assert_eq!(doc.attr_f64(0, "void"), None);
    // 缺失键 ≠ 零值（GEO-17 语义核心；行 3=空 props feature）
    assert_eq!(doc.attr_f64(3, "height"), None);
}

// spec: GEO-17
#[test]
fn gc_children_inherit_feature_props() {
    let (doc, _) = parse_geojson_with(MIXED.as_bytes(), RepairPolicy::Lenient).unwrap();
    // feature[1] 的 props 对其 GC 全部子件行复制（行 1/2 同为 "Block"）
    assert_eq!(doc.features()[1].style.fill[1], 1.0, "#00ff00 存活于子件 0");
    assert_eq!(doc.features()[2].style.fill[1], 1.0, "同 props 覆盖子件 1");
    assert_eq!(doc.attr_str(1, "name"), Some("Block"));
    assert_eq!(doc.attr_str(2, "name"), Some("Block"));
}

// spec: GEO-18
#[test]
fn style_reads_through_columns_identical_to_legacy() {
    let (doc, _) = parse_geojson_with(MIXED.as_bytes(), RepairPolicy::Lenient).unwrap();
    let s0 = &doc.features()[0].style;
    // 六键结果与迁移前逐项一致（fill hex→r=1.0；opacity clamp；未给键回默认）
    assert!((s0.fill[0] - 1.0).abs() < 1e-6 && s0.fill[1] < 1e-6);
    assert!((s0.fill_opacity - 0.5).abs() < 1e-6);
    assert_eq!(
        s0.stroke,
        [1.0, 1.0, 1.0, 1.0],
        "缺 stroke 回默认，非 height 干扰"
    );
    // 样式键之外，其余标量属性保持可查（样式消费不再吞属性）
    assert_eq!(doc.attr_f64(0, "fill-opacity"), Some(0.5));
    // park.geojson 回归族在 geo_spec/tess_style_spec（既有 14 条样式断言=迁移网）
}
