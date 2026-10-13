//! # visiaengine-ffi
//!
//! C ABI face (batch 2): handle table / thread affinity / panic fence / single input entry.
//! Behavior contract: `docs/sdd/capi.md` (CAPI-01..). Artifact name `libvisiaengine` (D6).
//! Dual-face entry discipline ([FFI-R:BS-5]): both the js face and the ffi face are thin leaves above engine.rs.

mod engine;
mod ffi;

pub use engine::{Engine, Proj}; // wasm bridge + N2.1 rs test face (the morph band dropped cfg on 2026-09-28: native-side tests/examples need it too)
#[cfg(target_arch = "wasm32")]
pub use ffi::enc_entity;
pub use ffi::{
    KIND_KEY, KIND_NO_SUCH, KIND_PTR_DOWN, KIND_PTR_MOVE, KIND_PTR_UP, KIND_WHEEL, MISS,
    VE_ERR_ARG, VE_ERR_IO, VE_ERR_PANIC, VE_ERR_SIZE, VE_ERR_STATE, VE_EVT_LOAD_ERROR,
    VE_EVT_LOAD_PROGRESS, VE_OK, VeEventCb, VeInput, VeMeshDesc, VePointMark, VePointsDesc,
    visiaengine_abi_version, visiaengine_add_mesh, visiaengine_attach, visiaengine_attr_bool,
    visiaengine_attr_f64, visiaengine_attr_str, visiaengine_create_headless, visiaengine_destroy,
    visiaengine_entity_at, visiaengine_entity_count, visiaengine_entity_set_visible,
    visiaengine_entity_visible, visiaengine_last_error, visiaengine_load_geojson,
    visiaengine_load_gltf, visiaengine_on_input, visiaengine_pick, visiaengine_readback,
    visiaengine_remove_entity, visiaengine_render, visiaengine_viewport,
};

/// CAPI-18/19 native-only (the wasm face goes through the bytes bridge in the wasm crate).
#[cfg(not(target_arch = "wasm32"))]
pub use ffi::{
    VE_PCL_FASTFAIL, VE_PCL_LENIENT, VePclReport, visiaengine_add_points, visiaengine_load_pcl,
};

/// CAPI-17 native-only (wasm32 has no C ABI event face -- JS closures go through the visiaengine-wasm bridge).
#[cfg(not(target_arch = "wasm32"))]
pub use ffi::visiaengine_set_event_callback;
pub use ffi::{VeCameraPose, visiaengine_fly_state, visiaengine_fly_to};
pub use ffi::{VeClipPlane, visiaengine_get_clips, visiaengine_set_clips}; // CAPI-20
pub use ffi::{VeLabelSpec, visiaengine_add_label, visiaengine_load_font}; // CAPI-21/22
pub use ffi::{VeMapView, visiaengine_get_camera, visiaengine_navigate_click, visiaengine_set_map}; // CAPI-25..27
pub use ffi::{
    visiaengine_create_group, visiaengine_get_group_offset, visiaengine_get_parent,
    visiaengine_set_group_offset, visiaengine_set_parent,
};
pub use ffi::{
    visiaengine_get_node_transform, visiaengine_load_env_hdr, visiaengine_load_mvt_dir,
    visiaengine_load_raster_dir, visiaengine_set_node_transform, visiaengine_set_raster_view,
    visiaengine_set_tile_source_http, visiaengine_set_tile_view,
};
pub use ffi::{
    visiaengine_update_entity_attr_bool, visiaengine_update_entity_attr_f64,
    visiaengine_update_entity_attr_str,
}; // CAPI-36..38 // CAPI-28/29/35 // CAPI-30..34 scene tree

#[doc(hidden)]
pub mod test_util {
    pub use crate::ffi::test_util::guard_panic_probe;
}
