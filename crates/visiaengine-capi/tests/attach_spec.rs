//! CAPI-06：attach 参数校验面（真窗口行为归 demo_x11/smoke-x11，无窗口环境不触 swapchain）。

use visiaengine::{
    VE_ERR_ARG, visiaengine_attach, visiaengine_create_headless, visiaengine_destroy,
};

// spec: CAPI-06
#[test]
fn attach_argument_surface_rejects_without_touching_gpu() {
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0);
    // kind 越界（3/−1）与 win=0 → -1（入口参数族）
    assert_eq!(
        visiaengine_attach(ve, 0x1234, 0x1, 3),
        VE_ERR_ARG,
        "kind 3 非法"
    );
    assert_eq!(visiaengine_attach(ve, 0x1234, 0x1, -1), VE_ERR_ARG);
    assert_eq!(visiaengine_attach(ve, 0, 0x1, 0), VE_ERR_ARG, "win=0 非法");
    // x11 必须双槽（display 非 0——rwh Xlib 无 display 不可构；win32 hinstance 才可 0=自动）
    assert_eq!(
        visiaengine_attach(ve, 0x1234, 0, 0),
        VE_ERR_ARG,
        "x11 display 必填"
    );
    // 跨平台非法句柄=安全 Err 路径：win32 句柄在 linux 无匹配后端 →
    // create_surface_unsafe 必 Err（Rust 测试面唯一不 segfault 的真实 surface 探针；
    // 真 x11 行为归 demo_x11/smoke-x11——垃圾 display 指针传 Xlib 会段错误，禁入测试）
    assert_eq!(
        visiaengine_attach(ve, 0xDEAD_BEEF, 0, 1),
        -2,
        "win32-on-linux 应 -2 状态失败"
    );
    // attach 失败后 headless 模式完好（不半途换面）
    assert_eq!(visiaengine_attach(ve, 0x1234, 0x1, 9), VE_ERR_ARG);
    assert_eq!(visiaengine_destroy(ve), 0);
}
