//! L0 headless-face tests (// spec: tags feed the traceability gate).

// spec: WGPU-01
#[test]
fn instance_create_headless() {
    // no panic = pass; the returned Instance has no Debug face, construction success is guaranteed by the type system
    let _inst = visiaengine_render_wgpu::create_instance();
}

// spec: WGPU-02
#[test]
fn adapter_enumeration_typed() {
    let adapters: Vec<wgpu::AdapterInfo> = visiaengine_render_wgpu::available_adapters();
    // this machine / lavapipe / no GPU are all legal: assert only no-panic + the type face; no lower bound on len
    let _ = adapters.len();
}
