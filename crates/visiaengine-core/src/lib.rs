//! # visiaengine-core
//!
//! VisiaEngine data-model core: scene graph (slab handle + dirty marks), coordinate types.
//!
//! **Invariant 1**: this crate never depends on render crates -- it can run/test/produce images headlessly.
//! Behavior contract: `docs/sdd/core.md` (CORE-01..10 implemented; RTC rebasing deferred to P1).

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

mod attrs;
mod color;
mod picking;
mod scene;

pub use attrs::AttrSet;
pub use color::{linear_to_srgb, srgb_to_linear};
pub use picking::{Ray, ray_aabb, ray_triangle, ray_triangle_double};
pub use scene::{Component, CoreError, EntityId, Scene, Transform, Vec3};
