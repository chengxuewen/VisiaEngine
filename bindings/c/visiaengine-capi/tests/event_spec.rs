//! CAPI-17：事件推送口（进度/错误/摘除/句柄门）。

use std::ffi::{CString, c_void};
use visiaengine::{
    VE_ERR_ARG, VE_EVT_LOAD_ERROR, VE_EVT_LOAD_PROGRESS, VeEventCb, visiaengine_create_headless,
    visiaengine_destroy, visiaengine_load_geojson, visiaengine_set_event_callback,
};

fn geo_path() -> CString {
    CString::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/data/park.geojson"
    ))
    .unwrap()
}
fn missing() -> CString {
    CString::new("/nonexistent/ve-park.geojson").unwrap()
}

#[derive(Default)]
struct Sink {
    ev: Vec<(u32, u64, u64)>,
}
extern "C" fn tap(user: *mut c_void, event: u32, a: u64, b: u64) {
    // 同步语义=调用线程触发（条款声明的活体证明）；栈借转裸指针仅在本调用内使用
    if let Some(s) = unsafe { user.cast::<Sink>().as_mut() } {
        s.ev.push((event, a, b));
    }
}
const TAP: VeEventCb = Some(tap);

// spec: CAPI-17
#[test]
fn progress_events_monotonic_terminal_and_removal() {
    let ve = visiaengine_create_headless(160, 120);
    let mut sink = Sink::default();
    assert_eq!(
        visiaengine_set_event_callback(ve, TAP, &mut sink as *mut Sink as *mut c_void),
        0
    );
    assert_eq!(visiaengine_load_geojson(ve, geo_path().as_ptr()), 0);
    let prog: Vec<_> = sink
        .ev
        .iter()
        .filter(|e| e.0 == VE_EVT_LOAD_PROGRESS)
        .collect();
    assert!(!prog.is_empty(), "装载无进度事件=推送半边缺席");
    let mut prev = 0u64;
    for (_, a, _b) in prog.iter() {
        assert!(*a > prev, "done 非单调: {a}≤{prev}");
        prev = *a;
    }
    let (a, b) = (prog.last().unwrap().1, prog.last().unwrap().2);
    assert_eq!(a, b, "终态 done≠total");
    // 摘除后静默（替换不叠加面同谱：NULL 摘→再载零新事件）
    assert_eq!(
        visiaengine_set_event_callback(ve, None, std::ptr::null_mut()),
        0
    );
    let n = sink.ev.len();
    assert_eq!(visiaengine_load_geojson(ve, geo_path().as_ptr()), 0);
    assert_eq!(sink.ev.len(), n, "NULL 摘除后仍触发=叠加泄漏");
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-17
#[test]
fn error_event_and_handle_gate() {
    let ve = visiaengine_create_headless(160, 120);
    let mut sink = Sink::default();
    assert_eq!(
        visiaengine_set_event_callback(ve, TAP, &mut sink as *mut Sink as *mut c_void),
        0
    );
    assert_ne!(visiaengine_load_geojson(ve, missing().as_ptr()), 0);
    assert!(
        sink.ev.iter().any(|e| e.0 == VE_EVT_LOAD_ERROR),
        "失败出口无 LOAD_ERROR 事件"
    );
    // 句柄门同族（stale/foreign 全 -1 谱，CAPI-03 分工不变）
    assert_eq!(
        visiaengine_set_event_callback(0, None, std::ptr::null_mut()),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_set_event_callback(ve << 1, None, std::ptr::null_mut()),
        VE_ERR_ARG
    );
    assert_eq!(visiaengine_destroy(ve), 0);
}
