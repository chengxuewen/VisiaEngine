//! GEO-09..14: subdivision + style (docs/sdd/geo.md).

use visiaengine_core::Vec3;
use visiaengine_geo::{GeoKind, StyleRecord, load_geojson, tessellate};

fn fill_positions(gp: &visiaengine_geo::GeoPart) -> &Vec<[f32; 3]> {
    match gp {
        visiaengine_geo::GeoPart::Fill(t) => &t.positions,
        _ => panic!("expected fill"),
    }
}

fn poly(ext: Vec<[f64; 2]>, holes: Vec<Vec<[f64; 2]>>) -> GeoKind {
    GeoKind::Poly { ext, holes }
}
fn square(half: f64) -> Vec<[f64; 2]> {
    vec![
        [-half, -half],
        [half, -half],
        [half, half],
        [-half, half],
        [-half, -half],
    ]
}

fn total_area_and_inside(parts: &[visiaengine_geo::GeoPart], p: (f64, f64)) -> (f64, bool) {
    let mut area = 0.0;
    let mut inside = false;
    for part in parts.iter().filter_map(|gp| match gp {
        visiaengine_geo::GeoPart::Fill(t) => Some(t),
        _ => None,
    }) {
        let (tri, _) = part.positions.as_chunks::<3>();
        for t in tri.iter().map(|sq| [sq[0], sq[1], sq[2]]) {
            let (a, b, c) = (t[0], t[1], t[2]);
            let ar = ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])) as f64 / 2.0;
            area += ar.abs();
            // same-side-rule inside test (all three edges share the sign, winding-insensitive, tolerant of 0)
            let pt = (p.0 as f32, p.1 as f32);
            let cr = |u: [f32; 3], v: [f32; 3]| {
                (v[0] - u[0]) * (pt.1 - u[1]) - (v[1] - u[1]) * (pt.0 - u[0])
            };
            let (d1, d2, d3) = (cr(a, b), cr(b, c), cr(c, a));
            if (d1 >= -1e-3 && d2 >= -1e-3 && d3 >= -1e-3)
                || (d1 <= 1e-3 && d2 <= 1e-3 && d3 <= 1e-3)
            {
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
    assert!(inside, "the centroid must be covered by the fill");
}

// spec: GEO-10
#[test]
fn hole_interior_uncovered() {
    let st = StyleRecord::default();
    let parts = tessellate(&poly(square(1.0), vec![square(0.2)]), &st).unwrap();
    let (_, inside) = total_area_and_inside(&parts, (0.0, 0.0));
    assert!(!inside, "a point inside the hole covered = hole lost");
}

// spec: GEO-11
#[test]
fn stroke_emits_linestrip_with_width_px() {
    // GEO-24: lines no longer expand CPU ribbons; output LineStrip (shape + width_px, expansion lives on the GPU)
    let line = GeoKind::Line(vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 0.0)]);
    let st2 = StyleRecord {
        stroke_width_px: 2.0,
        ..StyleRecord::default()
    };
    let parts = tessellate(&line, &st2).unwrap();
    let strip = match &parts[0] {
        visiaengine_geo::GeoPart::Strokes(s) => &s[0],
        _ => panic!("expected a Strokes part"),
    };
    assert_eq!(strip.width_px, 2.0, "px semantics collected");
    assert_eq!(strip.pts.len(), 2, "centerline shape order kept");
    assert!(
        (strip.pts[1][0] - 10.0).abs() < 1e-3,
        "length lives in the shape not the geometry"
    );
}

// spec: GEO-12
#[test]
fn simplestyle_six_keys_parsed() {
    let doc = load_geojson(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/park.geojson"
    ))
    .unwrap();
    let a = &doc.features()[0].style;
    assert!(
        (a.fill[0] - 1.0).abs() < 1e-6 && a.fill[1] < 1e-6 && (a.fill_opacity - 0.8).abs() < 1e-6
    );
    let road = &doc.features()[2].style;
    assert!(
        (road.stroke[0] - 0.50196).abs() < 0.01,
        "stroke {}",
        road.stroke[0]
    );
    let lamp = &doc.features()[4].style;
    assert!((lamp.marker_color[1] - 0.545).abs() < 0.01);
}

// spec: GEO-13
#[test]
fn missing_props_default_style() {
    let doc = load_geojson(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/park.geojson"
    ))
    .unwrap();
    // buildingB's "color" is not a six-key -> all defaults; grove has no props -> defaults
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
    let (pa, pb) = (fill_positions(&a[0]), fill_positions(&b[0]));
    assert_eq!(pa.len(), pb.len());
    for (x, y) in pa.iter().zip(pb.iter()) {
        for k in 0..2 {
            assert!(
                (x[k] - y[k]).abs() < 1e-3,
                "translation invariance broken {:?} vs {:?}",
                x,
                y
            );
        }
    }
}

// spec: GEO-24
#[test]
fn geopart_units_and_closure_semantics() {
    // (1) px default values (1.5/4.0 -- the body assertion of the unit-reinterpretation correction case)
    let st = StyleRecord::default();
    assert_eq!(st.stroke_width_px, 1.5);
    assert_eq!(st.radius_px, 4.0);
    // (2) ring output closed (first = last point -- the last segment of the consumer windows(2) expansion depends on it)
    let parts = tessellate(&poly(square(1.0), vec![]), &st).unwrap();
    let visiaengine_geo::GeoPart::Strokes(strips) = &parts[1] else {
        panic!("a poly default style must emit Strokes")
    };
    assert_eq!(
        strips[0].pts[0],
        *strips[0].pts.last().unwrap(),
        "ring not closed"
    );
    // (3) width=0 disables the stroke -> no Strokes part
    let st0 = StyleRecord {
        stroke_width_px: 0.0,
        ..StyleRecord::default()
    };
    let parts0 = tessellate(&poly(square(1.0), vec![]), &st0).unwrap();
    assert!(
        !parts0
            .iter()
            .any(|p| matches!(p, visiaengine_geo::GeoPart::Strokes(_))),
        "width=0 still emits strokes"
    );
    // (4) Point -> Markers (the true round-dot road: shape + px, no square triangles)
    let pm = tessellate(
        &GeoKind::Point(Vec3::new(0.5, 0.5, 0.0)),
        &StyleRecord::default(),
    )
    .unwrap();
    let visiaengine_geo::GeoPart::Markers(ms) = &pm[0] else {
        panic!("a point must emit Markers")
    };
    assert_eq!(ms[0].radius_px, 4.0);
    assert_eq!(ms[0].pos, [0.5, 0.5]);
}

// spec: GEO-26
#[test]
fn fill_opacity_flows_to_part_color_alpha() {
    // (4) head of the transparency-family decision chain: fill-opacity -> part material a (the GEO-12 domain clamp [0,1] already applies)
    let style = visiaengine_geo::StyleRecord {
        fill: [1.0, 0.0, 0.0, 1.0],
        fill_opacity: 0.8,
        ..Default::default()
    };
    let parts = visiaengine_geo::tessellate(&poly(square(4.0), vec![]), &style).expect("tess");
    for gp in &parts {
        if let visiaengine_geo::GeoPart::Fill(t) = gp {
            assert!(
                (t.color[3] - 0.8).abs() < 1e-6,
                "part alpha must be 0.8  got {:?}",
                t.color
            );
            return;
        }
    }
    panic!("no fill part");
}
