//! E203 · attributes and measurement —— the GeoJSON plane-measurement CLI (3857 meters, the GEO-21/22 consumption template).
//! Numbering note: 3.5e pre-allocated E403 -> batch 5 assigned band E203 (the interaction segment yields to E4xx).
//! Usage: cargo run -p visiaengine-geo --example E203_measure_cli [path]
//! A non-rendering CLI -- the output lines MEASURE can be asserted by grep.

use visiaengine_core::Vec3;
use visiaengine_geo::{GeoKind, load_geojson, planar_distance, ring_area};

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "resources/data/park.geojson".to_string());
    let doc = match load_geojson(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("ERROR {e}");
            std::process::exit(2);
        }
    };
    println!("MEASURE file={path} features={}", doc.features().len());
    for f in doc.features() {
        let name = f.name.as_deref().unwrap_or("-");
        match &f.kind {
            GeoKind::Line(pts) => {
                println!(
                    "MEASURE name={name} kind=line dist_m={:.2}",
                    planar_distance(pts)
                );
            }
            GeoKind::Poly { ext, .. } => {
                let ring: Vec<_> = ext.iter().map(|p| Vec3::new(p[0], p[1], 0.0)).collect();
                println!(
                    "MEASURE name={name} kind=poly area_m2={:.1}",
                    ring_area(&ring)
                );
            }
            _ => {}
        }
    }
}
