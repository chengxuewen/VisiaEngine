//! CAPI-17: the event-push entry (progress / error / unregister / handle gate).

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
    // synchronous semantics = fired on the calling thread (living proof of the clause declaration); borrowed-on-stack raw pointers are used only within this call
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
    assert!(
        !prog.is_empty(),
        "load with no progress event = half of the push missing"
    );
    let mut prev = 0u64;
    for (_, a, _b) in prog.iter() {
        assert!(*a > prev, "done not monotonic: {a} <= {prev}");
        prev = *a;
    }
    let (a, b) = (prog.last().unwrap().1, prog.last().unwrap().2);
    assert_eq!(a, b, "terminal done != total");
    // silence after unregister (same scheme as the replace-does-not-stack face: NULL unregister -> reload emits zero new events)
    assert_eq!(
        visiaengine_set_event_callback(ve, None, std::ptr::null_mut()),
        0
    );
    let n = sink.ev.len();
    assert_eq!(visiaengine_load_geojson(ve, geo_path().as_ptr()), 0);
    assert_eq!(
        sink.ev.len(),
        n,
        "still firing after NULL unregister = stacking leak"
    );
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
        "failure exit has no LOAD_ERROR event"
    );
    // handle gate same family (stale/foreign all on the -1 scheme, the CAPI-03 division unchanged)
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
