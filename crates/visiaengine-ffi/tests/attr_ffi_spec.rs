//! FFI-level locks for CAPI-10/11/12 (direct rlib calls on the extern face, true ABI shape):
//! shape key = the handle the host got passes straight through -- tri-state return codes (1/0/<0) -- attr_str zero partial write -- missing-parameter tables.

use std::ffi::{CStr, CString, c_char};
use std::fs;

use visiaengine::{
    VE_ERR_ARG, VE_ERR_SIZE, visiaengine_attr_bool, visiaengine_attr_f64, visiaengine_attr_str,
    visiaengine_create_headless, visiaengine_destroy, visiaengine_entity_at,
    visiaengine_load_geojson,
};

const GEO: &str = r#"{"type":"FeatureCollection","features":[
 {"type":"Feature","properties":{"name":"buildingA","height":12.5,"active":true},
  "geometry":{"type":"Polygon","coordinates":[[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]]}},
 {"type":"Feature","properties":{"name":"buildingB"},
  "geometry":{"type":"Polygon","coordinates":[[[2.0,2.0],[3.0,2.0],[3.0,3.0],[2.0,2.0]]]}}
]}"#;
const GEO2: &str = r#"{"type":"FeatureCollection","features":[
 {"type":"Feature","properties":{"name":"towerC","height":99.0},
  "geometry":{"type":"Polygon","coordinates":[[[5.0,5.0],[6.0,5.0],[6.0,6.0],[5.0,5.0]]]}}
]}"#;

fn tmp_layer(tag: &str, body: &str) -> CString {
    let dir = std::env::temp_dir().join(format!(
        "ve-attr-ffi-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0),
    ));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("layer.geojson");
    fs::write(&path, body).unwrap();
    CString::new(path.to_str().unwrap()).unwrap()
}

#[must_use]
fn loaded() -> u64 {
    let ve = visiaengine_create_headless(8, 8);
    assert_ne!(ve, 0);
    assert_eq!(
        visiaengine_load_geojson(ve, tmp_layer("a", GEO).as_ptr()),
        0
    );
    ve
}

#[test]
fn ffi_attr_hit_miss_out_untouched() {
    // spec: CAPI-10
    let ve = loaded();
    let a = visiaengine_entity_at(ve, 0);
    assert_ne!(a, u64::MAX);
    let k = CString::new("height").unwrap();
    let mut out = -777.0f64;
    assert_eq!(visiaengine_attr_f64(ve, a, k.as_ptr(), &mut out), 1);
    assert_eq!(out, 12.5);
    // absent: out untouched (the C projection of absent != zero value)
    let no = CString::new("nope").unwrap();
    assert_eq!(visiaengine_attr_f64(ve, a, no.as_ptr(), &mut out), 0);
    assert_eq!(out, 12.5, "the miss branch must not write out");
    // bool hit + type mismatch = miss (str column asked for bool)
    let kb = CString::new("active").unwrap();
    let mut bo = -3i32;
    assert_eq!(visiaengine_attr_bool(ve, a, kb.as_ptr(), &mut bo), 1);
    assert_eq!(bo, 1);
    let kn = CString::new("name").unwrap();
    assert_eq!(visiaengine_attr_bool(ve, a, kn.as_ptr(), &mut bo), 0);
    assert_eq!(bo, 1, "bool type-mismatch miss must not write either");
    // a MISS shape handle = pure absence
    let mut o2 = 0.0;
    assert_eq!(visiaengine_attr_f64(ve, u64::MAX, k.as_ptr(), &mut o2), 0);
    assert_eq!(visiaengine_destroy(ve), 0);
}

#[test]
fn ffi_attr_str_buffer_contract() {
    // spec: CAPI-11
    let ve = loaded();
    let a = visiaengine_entity_at(ve, 0);
    let kn = CString::new("name").unwrap();
    let mut buf = [0i8; 256];
    assert_eq!(
        visiaengine_attr_str(ve, a, kn.as_ptr(), buf.as_mut_ptr(), buf.len() as u64),
        1
    );
    assert_eq!(
        unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
        "buildingA"
    );
    // cap insufficient = -5 with zero partial write (no length-probe sub-mode, C-2 ruling)
    let mut small = [0x5Ai8; 8];
    assert_eq!(
        visiaengine_attr_str(ve, a, kn.as_ptr(), small.as_mut_ptr(), 3),
        VE_ERR_SIZE
    );
    assert!(
        small.iter().all(|&b| b == 0x5A),
        "cap insufficient must not partially write"
    );
    // absent = 0; argument table null = -1
    let no = CString::new("nope").unwrap();
    assert_eq!(
        visiaengine_attr_str(ve, a, no.as_ptr(), buf.as_mut_ptr(), 256),
        0
    );
    assert_eq!(
        visiaengine_attr_str(ve, a, std::ptr::null(), buf.as_mut_ptr(), 256),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_attr_str(ve, a, kn.as_ptr(), std::ptr::null_mut() as *mut c_char, 256),
        VE_ERR_ARG
    );
    assert_eq!(visiaengine_destroy(ve), 0);
}

#[test]
fn ffi_attr_gate_table() {
    // spec: CAPI-10
    let ve = loaded();
    let k = CString::new("height").unwrap();
    let mut out = 0.0;
    // null key / null out = -1 (even a legal handle must reject)
    assert_eq!(
        visiaengine_attr_f64(ve, 1, std::ptr::null(), &mut out),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_attr_f64(ve, 1, k.as_ptr(), std::ptr::null_mut()),
        VE_ERR_ARG
    );
    // an engine with no geo load: any entity bit = pure absence (the retention face publishes zero for gltf pieces)
    let bare = visiaengine_create_headless(8, 8);
    assert_eq!(visiaengine_attr_f64(bare, 7, k.as_ptr(), &mut out), 0);
    assert_eq!(visiaengine_destroy(bare), 0);
    assert_eq!(visiaengine_destroy(ve), 0);
}

#[test]
fn ffi_attr_multi_load_isolation() {
    // spec: CAPI-12
    let ve = loaded();
    let old = visiaengine_entity_at(ve, 0);
    assert_eq!(
        visiaengine_load_geojson(ve, tmp_layer("b", GEO2).as_ptr()),
        0
    );
    let k = CString::new("name").unwrap();
    let mut buf = [0i8; 256];
    // an old entity reads the old doc (rows do not drift)
    assert_eq!(
        visiaengine_attr_str(ve, old, k.as_ptr(), buf.as_mut_ptr(), 256),
        1
    );
    assert_eq!(
        unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
        "buildingA"
    );
    // a new entity reads the new doc
    let c = visiaengine_entity_at(ve, 2);
    let kh = CString::new("height").unwrap();
    let mut h = 0.0;
    assert_eq!(visiaengine_attr_f64(ve, c, kh.as_ptr(), &mut h), 1);
    assert_eq!(h, 99.0);
    assert_eq!(visiaengine_destroy(ve), 0);
}
