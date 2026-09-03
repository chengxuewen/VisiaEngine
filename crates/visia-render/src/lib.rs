//! # visia-render
//!
//! 渲染抽象层：`RenderBackend` trait + 渲染指令 IR（纯数据）。
//!
//! **不变式②**：本 crate 不出现任何后端类型——wgpu 类型止步于 `visia-render-wgpu`。
//! 本 crate 的测试内 stub 实现（REND-02）即为该不变式的构造级证明。
//! 行为契约：`docs/sdd/render.md`（条款 REND-01 起，S2 片填充）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

/// S0 占位：S2 被 trait/IR 契约取代。
#[must_use]
pub fn s0_placeholder() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::s0_placeholder;

    #[test]
    fn s0_render_contract_compiles() {
        assert!(s0_placeholder());
    }
}
