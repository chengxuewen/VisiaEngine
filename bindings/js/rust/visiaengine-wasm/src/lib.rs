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

    // ── B1 数据带镜像（CAPI-13..16 纯编组薄叶 [FFI-R:BS-5]）──
    #[wasm_bindgen(js_name = setEntityVisible)]
    pub fn set_entity_visible(&mut self, entity: u64, visible: bool) -> i32 {
        match self.inner.set_visible(entity, visible) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    #[must_use]
    #[wasm_bindgen(js_name = entityVisible)]
    pub fn entity_visible(&self, entity: u64) -> i32 {
        match self.inner.is_visible(entity) {
            Some(true) => 1,
            Some(false) => 0,
            None => -1,
        }
    }

    // ── B2 剖面带镜像（CAPI-20 纯编组薄叶 [FFI-R:BS-5]）──
    /// 扁平 4n f64 系数面（[]=唯一清空形）；界/角不对齐/退化=-1=C 面同谱。
    #[wasm_bindgen(js_name = setClips)]
    pub fn set_clips(&mut self, planes: &[f64]) -> i32 {
        if planes.len() % 4 != 0 {
            return -1;
        }
        let arr: Vec<[f64; 4]> = planes
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        match self.inner.set_clips(&arr) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// 读回扁平 4n（归一化后单位形=C 面同谱）。
    #[must_use]
    #[wasm_bindgen(js_name = getClips)]
    pub fn get_clips(&self) -> Vec<f64> {
        self.inner.clips().into_iter().flatten().collect()
    }

    // ── ⑤a 飞行带镜像（CAPI-23/24 薄叶）──
    /// 起飞/改道：pose 扁平 8=[target3, yaw, pitch, dist, zoom, fov]；durMs=0 瞬移形。
    /// 0=起飞，-1=域拒（同 C 面）。
    #[wasm_bindgen(js_name = flyTo)]
    pub fn fly_to(&mut self, pose: &[f64], dur_ms: u32) -> i32 {
        if pose.len() != 8 {
            return -1;
        }
        match self.inner.fly_to(
            [pose[0], pose[1], pose[2]],
            pose[3],
            pose[4],
            pose[5],
            pose[6],
            pose[7],
            u64::from(dur_ms),
        ) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// 状态：1=done(含 idle) / 0=飞行中（C 面同谱；无句柄错误域）。
    #[must_use]
    #[wasm_bindgen(js_name = flyState)]
    pub fn fly_state(&self) -> i32 {
        if self.inner.fly_state_done() { 1 } else { 0 }
    }

    /// 进度 [0,1]（done/idle=1.0 单主值域，与 C 面「done 不写 out」合并语义同谱）。
    #[must_use]
    #[wasm_bindgen(js_name = flyProgress)]
    pub fn fly_progress(&self) -> f64 {
        self.inner.fly_progress().unwrap_or(1.0)
    }

    // ── S2 文字带镜像（CAPI-21/22 薄叶）──
    /// 注字体（bytes=TTF/OTF 全式；替换式）。0=成功，负=错误码（同 C 面）。
    #[wasm_bindgen(js_name = loadFont)]
    pub fn load_font(&mut self, data: &[u8]) -> i32 {
        match self.inner.set_font(data) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// 世界锚标签：pos3+srgb 色 4+utf8 文本；成功=位形 bigint（0 合法=CAPI-01 web 投影），
    /// 拒/败=MISS(0xFFFF...)（addMesh 同谱）。
    #[must_use]
    #[wasm_bindgen(js_name = addLabel)]
    pub fn add_label(&mut self, pos: &[f64], color: &[f32], text: &str, size_px: f32) -> u64 {
        if pos.len() != 3 || color.len() != 4 {
            return crate::MISS;
        }
        let c = [color[0], color[1], color[2], color[3]];
        match self
            .inner
            .add_label([pos[0], pos[1], pos[2]], text, c, size_px)
        {
            Ok(bits) => bits,
            Err(_) => crate::MISS,
        }
    }
    // ── ⑤b 二波小地图镜像（CAPI-25..27 薄叶）──
    /// CAPI-25 开小地图：比例表 fx/fy/fw/fh∈[0,1]（fw/fh>0）+zoom>0；0=开，-1=域拒。关闭用 clearMap()。
    #[wasm_bindgen(js_name = setMap)]
    pub fn set_map(&mut self, fx: f32, fy: f32, fw: f32, fh: f32, zoom: f64) -> i32 {
        match self.inner.set_map(Some((fx, fy, fw, fh)), zoom) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-25 关小地图（回旧单帧全幅路）。0=成功。
    #[wasm_bindgen(js_name = clearMap)]
    pub fn clear_map(&mut self) -> i32 {
        match self.inner.set_map(None, 0.0) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-26 点击导航：顶视拾实体优先→地面兜底；主 rig 保角保距换 target；dur_ms=0 瞬移。
    #[wasm_bindgen(js_name = navigateClick)]
    pub fn navigate_click(&mut self, px: f32, py: f32, dur_ms: u32) -> i32 {
        match self.inner.navigate_click(px, py, u64::from(dur_ms)) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-27 主相机位姿读回：扁平 8=[target3, yaw, pitch, dist, zoom, fov]（flyTo 同形）。
    #[must_use]
    #[wasm_bindgen(js_name = getCameraPose)]
    pub fn get_camera_pose(&self) -> Vec<f64> {
        let r = self.inner.camera_pose();
        vec![
            r.target[0],
            r.target[1],
            r.target[2],
            r.yaw,
            r.pitch,
            r.dist,
            r.zoom,
            r.fov_y,
        ]
    }

    /// 扁平三元组面（js 数组=调用期拷贝，bindgen 天然）；退化=0 哨兵与 C 面同谱。
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    #[wasm_bindgen(js_name = addMesh)]
    pub fn add_mesh(
        &mut self,
        positions: &[f32],
        normals: &[f32],
        indices: &[u32],
        base_color: &[f32],
        origin: &[f64],
    ) -> u64 {
        if positions.len() % 3 != 0 || base_color.len() != 4 || origin.len() != 3 {
            return 0;
        }
        let pos: Vec<[f32; 3]> = positions
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        let nrm: Option<Vec<[f32; 3]>> = if normals.is_empty() {
            None
        } else if normals.len() != positions.len() {
            return 0;
        } else {
            Some(
                normals
                    .chunks_exact(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect(),
            )
        };
        let col = [base_color[0], base_color[1], base_color[2], base_color[3]];
        let org = [origin[0], origin[1], origin[2]];
        // js 面哨兵=MISS(MAX)：0 是合法位形不可占用（CAPI-01 分工的 web 投影）
        match self.inner.add_mesh(&pos, nrm.as_deref(), indices, col, org) {
            Ok(h) => h,
            Err(_) => u64::MAX,
        }
    }

    /// CAPI-18 镜像：扁平点云阵（8 float/点=pos×3+radius_px+color×3，sRGB 宿主面）；
    /// 退化/失败=MISS(MAX) 同 addMesh 谱（位形 0 合法禁占用——CAPI-01 web 投影）。
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    #[wasm_bindgen(js_name = addPoints)]
    pub fn add_points(&mut self, marks: &[f32]) -> u64 {
        if marks.len() % 8 != 0 || marks.is_empty() {
            return u64::MAX;
        }
        let raw: Vec<visiaengine::VePointMark> = marks
            .chunks_exact(8)
            .map(|m| visiaengine::VePointMark {
                pos: [m[0], m[1], m[2]],
                radius_px: m[3],
                color: [m[4], m[5], m[6]],
            })
            .collect();
        match self.inner.add_points(&raw) {
            Ok(h) => h,
            Err(_) => u64::MAX,
        }
    }

    /// CAPI-17 镜像：JS 回调 `(event, a, b) => void`；null=摘除。u64 计数面在
    /// progress 域 <2^53（要素数），f64 直传无损——bigint 纪律仅指句柄位形面。
    #[wasm_bindgen(js_name = setEventCallback)]
    pub fn set_event_callback(&mut self, cb: Option<js_sys::Function>) {
        match cb {
            Some(f) => self.inner.set_event_fn(Some(Box::new(move |ev, a, b| {
                let _ = f.call3(
                    &JsValue::NULL,
                    &JsValue::from(ev),
                    &JsValue::from_f64(f64::from(u32::try_from(a).unwrap_or(u32::MAX))),
                    &JsValue::from_f64(f64::from(u32::try_from(b).unwrap_or(u32::MAX))),
                );
            }))),
            None => self.inner.set_event_fn(None),
        }
    }

    /// CAPI-19 镜像：bytes 形（path 穿不过 wasm——load_geojson_bytes 同款宿主侧读）。
    /// report 暂不过界（v0 宿主以 kept==0 哨兵 MISS 判败因；d.ts 注记随镜像测）。
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    #[wasm_bindgen(js_name = loadPclBytes)]
    pub fn load_pcl_bytes(&mut self, bytes: &[u8], lenient: bool) -> u64 {
        match self.inner.load_pcl_bytes(bytes, lenient) {
            Ok((h, _)) => h,
            Err(_) => u64::MAX,
        }
    }

    #[wasm_bindgen(js_name = removeEntity)]
    pub fn remove_entity(&mut self, entity: u64) -> i32 {
        match self.inner.remove_entity(entity) {
            Ok(()) => 0,
            Err(_) => -1,
        }
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
