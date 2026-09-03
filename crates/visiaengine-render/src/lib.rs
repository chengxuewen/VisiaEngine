//! # visiaengine-render
//!
//! 渲染抽象层：`RenderBackend` trait + 渲染指令 IR（纯数据）。
//! 契约条款：`docs/sdd/render.md`（REND-01..05，S2 实装）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

pub mod contract;

pub use contract::{Camera, Capability, DrawCommand, Frame, MeshId, RenderBackend, Viewport};
