//! CAPI-06: the attach argument-validation face (real window behavior belongs to demo_x11/smoke-x11, no windowless environment touches the swapchain).

use visiaengine::{
    VE_ERR_ARG, visiaengine_attach, visiaengine_create_headless, visiaengine_destroy,
};

// spec: CAPI-06
#[test]
fn attach_argument_surface_rejects_without_touching_gpu() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // kind out of range (3/-1) and win=0 -> -1 (entry argument family)
    assert_eq!(
        visiaengine_attach(ve, 0x1234, 0x1, 3),
        VE_ERR_ARG,
        "kind 3 illegal"
    );
    assert_eq!(visiaengine_attach(ve, 0x1234, 0x1, -1), VE_ERR_ARG);
    assert_eq!(
        visiaengine_attach(ve, 0, 0x1, 0),
        VE_ERR_ARG,
        "win=0 illegal"
    );
    // x11 must fill both slots (display non-0 -- rwh Xlib cannot build without a display; only win32's hinstance may be 0 = auto)
    assert_eq!(
        visiaengine_attach(ve, 0x1234, 0, 0),
        VE_ERR_ARG,
        "x11 display required"
    );
    // cross-platform illegal handle = a safe Err path: a win32 handle on linux has no matching backend ->
    // create_surface_unsafe must Err (the only real surface probe on the Rust test face that does not segfault;
    // real x11 behavior belongs to demo_x11/smoke-x11 -- feeding Xlib a garbage display pointer segfaults, forbidden in tests)
    assert_eq!(
        visiaengine_attach(ve, 0xDEAD_BEEF, 0, 1),
        -2,
        "win32-on-linux should fail with -2 state"
    );
    // after a failed attach the headless mode is intact (no half-way face swap)
    assert_eq!(visiaengine_attach(ve, 0x1234, 0x1, 9), VE_ERR_ARG);
    assert_eq!(visiaengine_destroy(ve), 0);
}
