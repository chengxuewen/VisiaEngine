//! N2.4 (CAPI-36..38): property diff-update via the C surface — typed-
//! reject + override-first reads. Fixture + enumeration shape copied from
//! attr_ffi_spec.rs (verified precedent).

use std::ffi::CString;
use std::fs;

use visiaengine::{
    VE_ERR_ARG, visiaengine_attr_bool, visiaengine_attr_f64, visiaengine_attr_str,
    visiaengine_create_headless, visiaengine_entity_at, visiaengine_load_geojson,
    visiaengine_update_entity_attr_bool, visiaengine_update_entity_attr_f64,
    visiaengine_update_entity_attr_str,
};

const GEO: &str = r#"{"type":"FeatureCollection","features":[
 {"type":"Feature","properties":{"name":"buildingA","height":12.5,"active":true},
  "geometry":{"type":"Polygon","coordinates":[[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]]}}
]}"#;

fn tmp_layer(tag: &str, body: &str) -> CString {
    let dir = std::env::temp_dir().join(format!("ve-attr-diff-{tag}-{}", std::process::id(),));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("layer.geojson");
    fs::write(&path, body).unwrap();
    CString::new(path.to_str().unwrap()).unwrap()
}

#[must_use]
fn loaded() -> (u64, u64) {
    let ve = visiaengine_create_headless(8, 8);
    assert_ne!(ve, 0);
    assert_eq!(
        visiaengine_load_geojson(ve, tmp_layer("n", GEO).as_ptr()),
        0
    );
    let ent = visiaengine_entity_at(ve, 0);
    assert_ne!(ent, u64::MAX);
    (ve, ent)
}

// spec: CAPI-36
#[test]
fn diff_update_override_first_and_typed_reject() {
    let (ve, ent) = loaded();

    let name = CString::new("name").unwrap();
    let mut buf = [0i8; 64];
    assert_eq!(
        visiaengine_attr_str(ve, ent, name.as_ptr(), buf.as_mut_ptr(), 64),
        1,
        "source column present"
    );

    // spec: CAPI-37
    // Same-type str update -> override-first read.
    let val = CString::new("overridden").unwrap();
    assert_eq!(
        visiaengine_update_entity_attr_str(ve, ent, name.as_ptr(), val.as_ptr()),
        0
    );
    let mut buf2 = [0i8; 64];
    assert_eq!(
        visiaengine_attr_str(ve, ent, name.as_ptr(), buf2.as_mut_ptr(), 64),
        1
    );
    let got: Vec<u8> = buf2
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as u8)
        .collect();
    assert_eq!(
        String::from_utf8(got).as_deref(),
        Ok("overridden"),
        "override-first read"
    );

    // New f64 column (first write types it) + same-type overwrite.
    let temp = CString::new("temperature").unwrap();
    assert_eq!(
        visiaengine_update_entity_attr_f64(ve, ent, temp.as_ptr(), 21.5),
        0
    );
    let mut out = 0.0f64;
    assert_eq!(visiaengine_attr_f64(ve, ent, temp.as_ptr(), &mut out), 1);
    assert_eq!(out, 21.5);
    assert_eq!(
        visiaengine_update_entity_attr_f64(ve, ent, temp.as_ptr(), 19.0),
        0
    );
    assert_eq!(visiaengine_attr_f64(ve, ent, temp.as_ptr(), &mut out), 1);
    assert_eq!(out, 19.0);

    // Typed-reject: f64 into a str column -> VE_ERR_ARG, zero partial write.
    assert_eq!(
        visiaengine_update_entity_attr_f64(ve, ent, name.as_ptr(), 1.0),
        VE_ERR_ARG
    );
    let mut buf3 = [0i8; 64];
    assert_eq!(
        visiaengine_attr_str(ve, ent, name.as_ptr(), buf3.as_mut_ptr(), 64),
        1
    );
    let got3: Vec<u8> = buf3
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as u8)
        .collect();
    assert_eq!(
        String::from_utf8(got3).as_deref(),
        Ok("overridden"),
        "rejected write left data intact"
    );

    // NULL name reject.
    assert_eq!(
        visiaengine_update_entity_attr_f64(ve, ent, std::ptr::null(), 1.0),
        VE_ERR_ARG
    );
}

// spec: CAPI-38
#[test]
fn bool_roundtrip_and_unknown_entity() {
    let (ve, ent) = loaded();
    let name = CString::new("selected").unwrap();
    let mut out = 0i32;
    assert_eq!(
        visiaengine_update_entity_attr_bool(ve, ent, name.as_ptr(), 1),
        0
    );
    assert_eq!(visiaengine_attr_bool(ve, ent, name.as_ptr(), &mut out), 1);
    assert_eq!(out, 1);
    assert_eq!(
        visiaengine_update_entity_attr_bool(ve, ent, name.as_ptr(), 0),
        0
    );
    assert_eq!(visiaengine_attr_bool(ve, ent, name.as_ptr(), &mut out), 1);
    assert_eq!(out, 0);

    // Unknown entity reject (huge handle, nothing loaded under it).
    let name2 = CString::new("x").unwrap();
    assert_eq!(
        visiaengine_update_entity_attr_f64(ve, 987654, name2.as_ptr(), 1.0),
        VE_ERR_ARG
    );
}
