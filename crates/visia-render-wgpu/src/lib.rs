//! # visia-render-wgpu
//!
//! 默认后端（D4 终审：wgpu 直用自研管线）。wgpu 类型止步于本 crate（不变式②）。
//!
//! 分层（docs/sdd/render-wgpu.md）：
//! - L0 无头面：[`create_instance`] / [`available_adapters`]（WGPU-01/02）
//! - L1 离屏 golden：offscreen 渲染（S4 片）
//! - L2 窗口 smoke：`examples/`（`--frames N` 自动退出）

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

mod offscreen;

pub use offscreen::{OffscreenFrame, render_offscreen_triangle};

/// 以 PRIMARY 后端族构造 Instance（Vulkan/Metal/DX12/GL）。
#[must_use]
pub fn create_instance() -> wgpu::Instance {
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = wgpu::Backends::PRIMARY;
    // 骨架期显式空标志：debug profile 下 default=from_build_config() 开 VALIDATION，
    // 将尝试 VK_EXT_debug_utils——lavapipe/旧 loader 缺符号即 panic（CI debug 测试同雷）。
    // 校验/诊断模式（GPU_BASED_VALIDATION 等）属专项切片，随 Studio 诊断面启用。
    desc.flags = wgpu::InstanceFlags::empty();
    wgpu::Instance::new(desc)
}

/// 枚举可用适配器信息。无 GPU/驱动缺失时为空 vec（可报告状态，非崩溃理由）。
#[must_use]
pub fn available_adapters() -> Vec<wgpu::AdapterInfo> {
    let instance = create_instance();
    pollster::block_on(instance.enumerate_adapters(wgpu::Backends::PRIMARY))
        .into_iter()
        .map(|adapter| adapter.get_info())
        .collect()
}
