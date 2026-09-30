//! # visiaengine-render
//!
//! 渲染抽象层：`RenderBackend` trait + 渲染指令 IR（纯数据）。
//! 契约条款：`docs/sdd/render.md`（REND-01..05，S2 实装）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

pub mod anim;
pub mod camera;
pub mod contract;
pub mod curve;
pub mod morph;
pub mod rebase;

mod picking;

pub use anim::anim_origin;
pub use camera::{
    CameraRig, Easing, ray_ground_intersect, screen_to_ray_ortho, screen_to_ray_persp,
};
pub use contract::{
    BackendError, Camera, Capability, ClipSetup, DrawCommand, EdlSetup, Frame, Instance,
    InstanceDesc, InstanceId, LabelMark, LabelTableDesc, MaterialDesc, MaterialId, MeshDesc,
    MeshId, PointMark, PointTableDesc, PostEffect, RenderBackend, ShadowBias, ShadowSetup,
    StrokeSeg, StrokeTableDesc, TableId, TextureDesc, TextureId, Viewport, ViewportRect,
    clip_to_local,
};
pub use morph::{morph_ease, morph_proj, morph_px_scale};
pub use picking::{
    MeshCandidate, PickHit, PointCloudCandidate, PointHit, StrokeCandidate, StrokeHit, pick_meshes,
    pick_points, pick_strokes,
};
