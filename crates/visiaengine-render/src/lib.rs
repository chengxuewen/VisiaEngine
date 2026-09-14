//! # visiaengine-render
//!
//! 渲染抽象层：`RenderBackend` trait + 渲染指令 IR（纯数据）。
//! 契约条款：`docs/sdd/render.md`（REND-01..05，S2 实装）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

pub mod camera;
pub mod contract;
pub mod rebase;

mod picking;

pub use camera::{CameraRig, screen_to_ray_ortho, screen_to_ray_persp};
pub use contract::{
    BackendError, Camera, Capability, DrawCommand, Frame, Instance, InstanceDesc, InstanceId,
    MaterialDesc, MaterialId, MeshDesc, MeshId, PointMark, PointTableDesc, RenderBackend,
    StrokeSeg, StrokeTableDesc, TableId, TextureDesc, TextureId, Viewport,
};
pub use picking::{MeshCandidate, PickHit, pick_meshes};
