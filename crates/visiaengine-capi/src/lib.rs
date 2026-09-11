//! # visiaengine-capi
//!
//! C ABI 面（批 2）：句柄表/线程亲和/panic 栅栏/输入单入口。
//! 行为契约：`docs/sdd/capi.md`（CAPI-01..）。产物名 `libvisiaengine`（D6）。
//! 双面口纪律（[FFI-R:BS-5]）：js 面与 ffi 面皆为 engine.rs 之上的薄叶。

mod engine;
mod ffi;

pub use ffi::{
    KIND_KEY, KIND_NO_SUCH, KIND_PTR_DOWN, KIND_PTR_MOVE, KIND_PTR_UP, KIND_WHEEL, MISS,
    VE_ERR_ARG, VE_ERR_IO, VE_ERR_PANIC, VE_ERR_SIZE, VE_ERR_STATE, VeInput,
    visiaengine_abi_version, visiaengine_attach, visiaengine_create_headless, visiaengine_destroy,
    visiaengine_entity_at, visiaengine_entity_count, visiaengine_last_error,
    visiaengine_load_geojson, visiaengine_load_gltf, visiaengine_on_input, visiaengine_pick,
    visiaengine_readback, visiaengine_render, visiaengine_viewport,
};

#[doc(hidden)]
pub mod test_util {
    pub use crate::ffi::test_util::guard_panic_probe;
}
