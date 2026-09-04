//! GEO-09..14：细分 + 样式（docs/sdd/geo.md）。

use visiaengine_core::Vec3;
use visiaengine_geo::{GeoKind, StyleRecord, load_geojson, tessellate};

fn poly(ext: Vec<[f64; 2]>, holes: Vec<Vec<[f64; 2]>>) -> GeoKind {
    GeoKind::Poly { ext, holes }
}
fn square(half: f64) -> Vec<[f64; 2]> {
    vec![
        [-half, -half],
        [half, -half],
        [half, half],
        [-half, -half],
    ]
}

fn total_area_and_inside(parts: &[visiaengine_geo::TessPart], p: (f64, f64)) -> (f64, bool) {
    let mut area = 0.0;
    let mut inside = false;
    for part in parts {
        let tri = part.positions.chunks_exact(3);
        for t in tri {
            let (a, b, c) = (t[0], t[1], t[2]);
            let ar = ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])) as f64 / 2.0;
            area += ar.abs();
            // 同侧法点内测试（三边符号一致，含 0 容差）
            let pt = (p.0 as f32, p.1 as f32);
            let s = |u: [f32; 3], v: [f32; 3]| {
                ((v[0] - u[0]) * (pt.1 - u[1]) - (v[1] - u[1]) * (pt.0 - u[0])) >= -1e-3
            };
            if s(a, b) && s(b, c) && s(c, a) {
                inside = true;
            }
        }
    }
    (area, inside)
}

// spec: GEO-09
#[test]
fn fill_area_matches_polygon() {
    let st = StyleRecord::default();
    let parts = tessellate(&poly(square(1.0), vec![]), &st).unwrap();
    let (area, inside) = total_area_and_inside(&parts, (0.0, 0.0));
    assert!((area - 4.0).abs() / 4.0 < 0.05, "area={area}");
    assert!(inside, "重心必须被填充覆盖");
}

// spec: GEO-10
#[test]
fn hole_interior_uncovered() {
    let st = StyleRecord::default();
    let parts = tessellate(&poly(square(1.0), vec![square(0.2)]), &st).unwrap();
    let (_, inside) = total_area_and_inside(&parts, (0.0, 0.0));
    assert!(!inside, "洞内点被覆盖=洞丢失");
}

// spec: GEO-11
#[test]
fn stroke_expands_to_width() {
    let st = StyleRecord::default();
    let line = GeoKind::Line(vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)]);
    let st2 = StyleRecord {
        stroke_width_m: 2.0,
        ..StyleRecord::default()
    };
    let parts = tessellate(&line, &st2).unwrap();
    let stroke = parts
        .iter()
        .find(|p| p.kind == visiaengine_geo::PartKind::Stroke)
        .expect("stroke part");
    let ys: Vec<f32> = stroke.positions.iter().map(|v| v[1]).collect();
    let xs: Vec<f32> = stroke.positions.iter().map(|v| v[0]).collect();
    let w = ys.iter().fold(f32::MIN, |a, &b| a.max(b)) - ys.iter().fold(f32::MAX, |a, &b| a.min(b));
    let l = xs.iter().fold(f32::MIN, |a, &b| a.max(b)) - xs.iter().fold(f32::MAX, |a, &b| a.min(b));
    assert!((w - 2.0).abs() < 0.1, "stroke 宽 {w}");
    assert!((l - 10.0).abs() < 0.1, "stroke 长 {l}");
}

// spec: GEO-12
#[test]
fn simplestyle_six_keys_parsed() {
    let doc = load_geojson(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/park.geojson"
    ))
    .unwrap();
    let a = &doc.features()[0].style;
    assert!((a.fill[0] - 1.0).abs() < 1e-6 && a.fill[1] < 1e-6 && (a.fill_opacity - 0.8).abs() < 1e-6);
    let road = &doc.features()[2].style;
    assert!((road.stroke[0] - 0.50196).abs() < 0.01, "stroke {}", road.stroke[0]);
    let lamp = &doc.features()[4].style;
    assert!((lamp.marker_color[1] - 0.545).abs() < 0.01);
}

// spec: GEO-13
#[test]
fn missing_props_default_style() {
    let doc = load_geojson(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../testdata/park.geojson"
    ))
    .unwrap();
    // buildingB 的 "color" 非六键 → 全默认；grove 无 props → 默认
    assert_eq!(doc.features()[1].style, StyleRecord::default());
    assert_eq!(doc.features()[3].style, StyleRecord::default());
}

// spec: GEO-14
#[test]
fn tessellate_input_is_origin_local() {
    let st = StyleRecord::default();
    let base = square(1.0);
    let far: Vec<[f64; 2]> = base.iter().map(|p| [p[0] + 1e7, p[1]]).collect();
    let local: Vec<[f64; 2]> = far.iter().map(|p| [p[0] - 1e7, p[1]]).collect();
    let a = tessellate(&poly(base, vec![]), &st).unwrap();
    let b = tessellate(&poly(local, vec![]), &st).unwrap();
    let (pa, pb) = (&a[0].positions, &b[0].positions);
    assert_eq!(pa.len(), pb.len());
    for (x, y) in pa.iter().zip(pb.iter()) {
        for k in 0..2 {
            assert!((x[k] - y[k]).abs() < 1e-3, "平移不变性破坏 {:?} vs {:?}", x, y);
        }
    }
}
