//! wasm32 独占 crate（native `cargo test --workspace` 编译空壳）。
//! JS/wasm 面（批 7 路线 A）：Engine 之上的**纯编组薄叶**——任何策略逻辑
//! 出现即违规（[FFI-R:BS-5] 双面口单源纪律）；常量/错误码 getter 皆为
//! Rust 单源的 JS 镜像（CAPI-09 机器对账）。

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

use visiaengine::{Engine, enc_entity};
use visiaengine::{
    KIND_KEY, KIND_NO_SUCH, KIND_PTR_DOWN, KIND_PTR_MOVE, KIND_PTR_UP, KIND_WHEEL, MISS,
    VE_ERR_ARG, VE_ERR_IO, VE_ERR_PANIC, VE_ERR_SIZE, VE_ERR_STATE, visiaengine_abi_version,
};

/// 引擎 JS 句柄（GC 回收=Drop=析构；无 free 函数纪律 wasm 侧由生命周期天然担保）。
#[wasm_bindgen]
pub struct VisiaEngine {
    inner: Engine,
}

#[wasm_bindgen]
impl VisiaEngine {
    /// 异步工厂：canvas 尺寸自带 attach（web GPU 构造无同步路径——Promise 原生
    /// [FFI-R:v13-FEAS-2]）。失败（无 adapter）=null。
    #[wasm_bindgen(js_name = fromCanvas)]
    pub async fn from_canvas(canvas: &web_sys::HtmlCanvasElement) -> Option<VisiaEngine> {
        Engine::new_canvas(canvas)
            .await
            .map(|inner| VisiaEngine { inner })
    }

    #[wasm_bindgen(js_name = loadGltfBytes)]
    pub fn load_gltf_bytes(&mut self, data: &[u8]) -> i32 {
        match self.inner.load_gltf_bytes(data) {
            Ok(_) => 0,
            Err(_) => VE_ERR_IO,
        }
    }

    #[wasm_bindgen(js_name = loadGeoJsonBytes)]
    pub fn load_geojson_bytes(&mut self, data: &[u8]) -> i32 {
        match self.inner.load_geojson_bytes(data) {
            Ok(_) => 0,
            Err(_) => VE_ERR_IO,
        }
    }

    /// 拉模型一帧（宿主 rAF 内调）。
    pub fn render(&mut self) -> i32 {
        match self.inner.render() {
            Ok(()) => 0,
            Err(_) => VE_ERR_STATE,
        }
    }

    /// 唯一输入口（扁平参数——VeInput 不过 wasm 界，struct_size 议题天然消解）。
    pub fn input(&mut self, kind: u32, px: f32, py: f32, wheel: f32) -> i32 {
        match self.inner.apply_input(kind, px, py, wheel) {
            Some(consumed) => i32::from(consumed),
            None => 0,
        }
    }

    pub fn pick(&mut self, px: f32, py: f32) -> u64 {
        self.inner.pick(px, py).map_or(MISS, enc_entity)
    }

    #[wasm_bindgen(js_name = entityAt)]
    pub fn entity_at(&mut self, index: u32) -> u64 {
        self.inner.entity_at(index).map_or(MISS, enc_entity)
    }

    #[wasm_bindgen(js_name = entityCount)]
    pub fn entity_count(&self) -> i32 {
        self.inner.entity_count() as i32
    }

    #[wasm_bindgen(js_name = viewport)]
    pub fn viewport(&mut self, w: u32, h: u32) -> i32 {
        if w == 0 || h == 0 {
            return VE_ERR_SIZE;
        }
        self.inner.resize(w, h);
        0
    }

    #[wasm_bindgen(js_name = abiVersion)]
    pub fn abi_version() -> u32 {
        visiaengine_abi_version()
    }
}

// ===== 常量镜像 getters（Rust 单源；CAPI-09 值对账锚点）=====
// 普通 fn 导出（wasm-bindgen 不支持 const fn 属性宏——实测拒收，值仍为编译期常量引用，单源不变）
#[wasm_bindgen]
#[must_use]
pub fn kind_ptr_move() -> u32 {
    KIND_PTR_MOVE
}
#[wasm_bindgen]
#[must_use]
pub fn kind_ptr_down() -> u32 {
    KIND_PTR_DOWN
}
#[wasm_bindgen]
#[must_use]
pub fn kind_ptr_up() -> u32 {
    KIND_PTR_UP
}
#[wasm_bindgen]
#[must_use]
pub fn kind_wheel() -> u32 {
    KIND_WHEEL
}
#[wasm_bindgen]
#[must_use]
pub fn kind_key() -> u32 {
    KIND_KEY
}
#[wasm_bindgen]
#[must_use]
pub fn kind_no_such() -> u32 {
    KIND_NO_SUCH
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_arg() -> i32 {
    VE_ERR_ARG
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_state() -> i32 {
    VE_ERR_STATE
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_io() -> i32 {
    VE_ERR_IO
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_panic() -> i32 {
    VE_ERR_PANIC
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_size() -> i32 {
    VE_ERR_SIZE
}
