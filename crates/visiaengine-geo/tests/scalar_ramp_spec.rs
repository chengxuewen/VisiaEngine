//! GEO-20：visia:color-* 扩展键 = 标量→色带（解析期物化，D9 边界外自有命名）。
//! 机制修订注记：计划 v1.1 原文"256px LUT 纹理+WGSL 采样"按 ponytail 降档——
//! GIS 属性着色的真实形态是 **per-feature 常量色**（楼高=每 feature 单值），
//! CPU lerp 即完备；GPU LUT 纹理留待逐顶点场（DEM/点云）真需求（backlog）。

use visiaengine_geo::{RepairPolicy, load_geojson, parse_geojson_lenient};

const ONE: &str = r##"{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"height":40,"visia:color-column":"height","visia:color-lo":0,"visia:color-hi":50,"visia:color-low":"#00ff00","visia:color-high":"#ff0000"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}]}"##;

// spec: GEO-20
#[test]
fn scalar_lands_on_color_ramp() {
    let (doc, _) = parse_geojson_lenient(ONE.as_bytes()).unwrap();
    let s = &doc.features()[0].style;
    // t=(40-0)/50=0.8 → lerp(green,red,0.8)=(0.8,0.2,0)
    assert!((s.fill[0] - 0.8).abs() < 1e-5, "红通道随 t 升");
    assert!((s.fill[1] - 0.2).abs() < 1e-5, "绿通道随 t 降");
    assert_eq!(s.fill[3], 1.0);
    // marker 同步（点件一致性）
    assert_eq!(s.marker_color, s.fill);
}

// spec: GEO-20
#[test]
fn out_of_range_clamps_and_missing_column_falls_back() {
    // 超上界 clamp 到纯高色
    let hi = ONE.replace("\"height\":40", "\"height\":999");
    let (doc, _) = parse_geojson_lenient(hi.as_bytes()).unwrap();
    assert_eq!(doc.features()[0].style.fill, [1.0, 0.0, 0.0, 1.0]);
    // 缺列（引用不存在属性）→ 回退六键/默认，无 panic 无声降
    let miss = ONE.replace("\"height\":40,", "");
    let (doc, _) = parse_geojson_lenient(miss.as_bytes()).unwrap();
    assert_eq!(doc.features()[0].style.fill, [0.0, 0.45, 1.0, 1.0]);
    // 无效区间（hi<=lo）→ 静默禁用
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
    assert_eq!(fills[0], [0.0, 1.0, 0.0, 1.0], "h0=纯低色");
    assert_eq!(fills[1], [0.5, 0.5, 0.0, 1.0], "h25=中点");
    assert_eq!(fills[2], [1.0, 0.0, 0.0, 1.0], "h50=纯高色");
    assert_eq!(fills[3], [0.0, 0.45, 1.0, 1.0], "缺列件回默认蓝");
}
