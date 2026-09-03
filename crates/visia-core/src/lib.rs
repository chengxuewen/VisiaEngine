//! # visia-core
//!
//! VisiaEngine 数据模型核心：场景图（slab handle + 脏标记）、坐标类型。
//!
//! **不变式①**：本 crate 永不依赖渲染 crate——可无头运行/测试/出图。
//! 行为契约：`docs/sdd/core.md`（CORE-01..10 实装；RTC 重基留 P1）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

mod scene;

pub use scene::{Component, CoreError, EntityId, Scene, Transform, Vec3};
