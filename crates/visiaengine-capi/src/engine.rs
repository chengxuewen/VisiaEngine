//! VeEngine 实体（计划 v1.4 §3）：I1=策略态（rig/输入状态机），
//! I2 起接 device/MeshCore/scene（bytes-first load 口，批 7 wasm 复用面）。

use visiaengine_render::CameraRig;

/// 输入→相机引擎策略（非事件透传）：按下后移动=orbit，滚轮=共享 zoom。
const ORBIT_RATE: f64 = 0.005;
const ZOOM_RATE: f64 = 0.1;

pub struct Engine {
    pub w: u32,
    pub h: u32,
    pub rig: CameraRig,
    pub down: bool,
    pub last: (f32, f32),
    /// I2: 装载管线实装后由实体表提供
    pub entity_total: u32,
}

impl Engine {
    #[must_use]
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            rig: CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            down: false,
            last: (0.0, 0.0),
            entity_total: 0,
        }
    }

    /// CAPI-05 映射：返回 Some(消费性)/None=未登记 kind（no-op）。
    pub fn apply_input(&mut self, kind: u32, px: f32, py: f32, wheel: f32) -> Option<bool> {
        match kind {
            ffi_kind::PTR_DOWN => {
                self.down = true;
                self.last = (px, py);
                Some(true)
            }
            ffi_kind::PTR_UP => {
                self.down = false;
                Some(true)
            }
            ffi_kind::PTR_MOVE => {
                if !self.down {
                    return Some(false); // 未按下的移动非本引擎事件
                }
                let (dx, dy) = (f64::from(px - self.last.0), f64::from(py - self.last.1));
                self.last = (px, py);
                self.rig.orbit_delta(-dx * ORBIT_RATE, dy * ORBIT_RATE);
                Some(true)
            }
            ffi_kind::WHEEL => {
                self.rig.zoom *= (f64::from(wheel) * ZOOM_RATE).exp();
                Some(true)
            }
            ffi_kind::KEY => Some(false),
            _ => None,
        }
    }
}

mod ffi_kind {
    pub const PTR_MOVE: u32 = 1;
    pub const PTR_DOWN: u32 = 2;
    pub const PTR_UP: u32 = 3;
    pub const WHEEL: u32 = 4;
    pub const KEY: u32 = 5;
}
