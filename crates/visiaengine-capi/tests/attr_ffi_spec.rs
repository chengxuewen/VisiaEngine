//! CAPI-10/11/12 的 FFI 级锁（rlib 直调 extern 面，真 ABI 形状）：
//! 位形键=宿主所得句柄直通 · 返回码三态（1/0/<0）· attr_str 零部分写 · 缺参表。

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
    // 缺失：out 不动（缺失≠零值的 C 投影）
    let no = CString::new("nope").unwrap();
    assert_eq!(visiaengine_attr_f64(ve, a, no.as_ptr(), &mut out), 0);
    assert_eq!(out, 12.5, "miss 分支禁写 out");
    // bool 命中 + 异型=miss（str 列问 bool）
    let kb = CString::new("active").unwrap();
    let mut bo = -3i32;
    assert_eq!(visiaengine_attr_bool(ve, a, kb.as_ptr(), &mut bo), 1);
    assert_eq!(bo, 1);
    let kn = CString::new("name").unwrap();
    assert_eq!(visiaengine_attr_bool(ve, a, kn.as_ptr(), &mut bo), 0);
    assert_eq!(bo, 1, "bool 异型 miss 亦禁写");
    // MISS 位形句柄=纯缺失
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
    // cap 不足 = -5 且零部分写（无探长子模式，C-2 裁决）
    let mut small = [0x5Ai8; 8];
    assert_eq!(
        visiaengine_attr_str(ve, a, kn.as_ptr(), small.as_mut_ptr(), 3),
        VE_ERR_SIZE
    );
    assert!(small.iter().all(|&b| b == 0x5A), "cap 不足禁部分写");
    // 缺失=0；参数表 null=-1
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
    // null key / null out = -1（合法句柄也要拒）
    assert_eq!(
        visiaengine_attr_f64(ve, 1, std::ptr::null(), &mut out),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_attr_f64(ve, 1, k.as_ptr(), std::ptr::null_mut()),
        VE_ERR_ARG
    );
    // 无 geo 装载的引擎：任何 entity 位=纯缺失（保留面对 gltf 件零发布）
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
    // 旧 entity 读旧 doc（行不漂）
    assert_eq!(
        visiaengine_attr_str(ve, old, k.as_ptr(), buf.as_mut_ptr(), 256),
        1
    );
    assert_eq!(
        unsafe { CStr::from_ptr(buf.as_ptr()) }.to_str().unwrap(),
        "buildingA"
    );
    // 新 entity 读新 doc
    let c = visiaengine_entity_at(ve, 2);
    let kh = CString::new("height").unwrap();
    let mut h = 0.0;
    assert_eq!(visiaengine_attr_f64(ve, c, kh.as_ptr(), &mut h), 1);
    assert_eq!(h, 99.0);
    assert_eq!(visiaengine_destroy(ve), 0);
}
