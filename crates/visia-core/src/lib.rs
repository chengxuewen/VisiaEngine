//! # visia-core
//!
//! VisiaEngine 数据模型核心：场景图、坐标类型、资源标识。
//!
//! **不变式①**：本 crate 永不依赖任何渲染 crate（可无头运行/测试）。
//! 行为契约：`docs/sdd/core.md`（条款 CORE-01 起，S1 片填充）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

/// S0 占位：证明 core 可独立编译链接（无渲染依赖）。S1 被场景图 API 取代。
#[must_use]
pub fn s0_headless_placeholder() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::s0_headless_placeholder;

    #[test]
    fn s0_core_links_without_render() {
        assert!(s0_headless_placeholder());
    }
}
