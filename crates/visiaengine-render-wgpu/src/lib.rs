//! # visiaengine-render-wgpu
//!
//! The default backend (D4 final ruling: wgpu used directly with the self-built pipeline). wgpu types stop at this crate (invariant 2).
//!
//! Layering (docs/sdd/render-wgpu.md):
//! - L0 headless face: [`create_instance`] / [`available_adapters`] (WGPU-01/02)
//! - L1 offscreen golden: offscreen rendering (S4 slice)
//! - L2 window smoke: `examples/` (`--frames N` auto-exit)

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

pub mod headless;
pub mod mesh_core;
mod offscreen;

pub use headless::HeadlessBackend;
pub use mesh_core::MultiClearPolicy;
pub use offscreen::{
    OffscreenFrame, render_offscreen_cube, render_offscreen_cube_at, render_offscreen_triangle,
    unit_box_mesh,
};

/// Build the Instance with the PRIMARY backend family (Vulkan/Metal/DX12/GL).
#[must_use]
pub fn create_instance() -> wgpu::Instance {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = wgpu::Backends::PRIMARY;
    // explicit empty flags during the skeleton phase: under the debug profile default = from_build_config() turns on VALIDATION,
    // which tries VK_EXT_debug_utils -- lavapipe/older loaders panic on missing symbols (the same mine for CI debug tests).
    // Validation/diagnostic modes (GPU_BASED_VALIDATION etc.) belong to a specialist slice, enabled with the Studio diagnostic face.
    desc.flags = wgpu::InstanceFlags::empty();
    wgpu::Instance::new(desc)
}

/// Enumerate available adapter info. An empty vec without GPU/driver (a reportable state, not a crash reason).
#[must_use]
pub fn available_adapters() -> Vec<wgpu::AdapterInfo> {
    let instance = create_instance();
    pollster::block_on(instance.enumerate_adapters(wgpu::Backends::PRIMARY))
        .into_iter()
        .map(|adapter| adapter.get_info())
        .collect()
}

pub mod surface;
