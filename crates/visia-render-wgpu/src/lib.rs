//! # visia-render-wgpu
//!
//! 默认后端（D4 终审：wgpu 直用自研管线）。实现 `visia-render::RenderBackend`。
//!
//! 分层：L0 实例/枚举（WGPU-01/02）· L1 离屏 golden（WGPU-03..05）·
//! L2 窗口 smoke（examples + `--frames N`，CI xvfb-run）。
//! 行为契约：`docs/sdd/render-wgpu.md`（S3/S4 片填充）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

/// S0 占位：S3 被后端实现取代。
#[must_use]
pub fn s0_placeholder() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::s0_placeholder;

    #[test]
    fn s0_backend_crate_compiles() {
        assert!(s0_placeholder());
    }
}
