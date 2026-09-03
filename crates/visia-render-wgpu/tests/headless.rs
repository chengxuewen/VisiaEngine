//! L0 无头面测试（// spec: 标签入追溯门禁）。

// spec: WGPU-01
#[test]
fn instance_create_headless() {
    // 不 panic 即通过；返回 Instance 无 Debug 面，构造成功性由类型系统担保
    let _inst = visia_render_wgpu::create_instance();
}

// spec: WGPU-02
#[test]
fn adapter_enumeration_typed() {
    let adapters: Vec<wgpu::AdapterInfo> = visia_render_wgpu::available_adapters();
    // 本机/lavapipe/无 GPU 均合法：仅断言不 panic + 类型面；len 不设下限
    let _ = adapters.len();
}
