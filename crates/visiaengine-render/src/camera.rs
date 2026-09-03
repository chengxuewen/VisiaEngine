//! 相机数学（CPU 面，glam 内部实现，矩阵裸数据出型——REND-09 同款纪律）。
//! 变体声明（REND-11）：投影深度取 **wgpu 原生 [0,1] 约定**（clip z/w ∈ [0,1]）。
//! 教训（G2 golden 实锤）：GL [-1,1] 变体在近处产生负深度→**硬件裁剪整帧静默无形**
//! （无验证报错的几何死亡）。投影矩阵构造因此在此处手建并以 REND-11/12 行为断言锁死。

use glam::{Mat4, Vec4};

/// f64 列主序 → f32（上传前唯一降位点，显式化避免 cast lint 面）。
#[must_use]
fn down(m: [[f64; 4]; 4]) -> [[f32; 4]; 4] {
    m.map(|col| col.map(|v| v as f32))
}

/// 轨道相机 rig：target 为中心，yaw/pitch/dist 球面（f64 稳态，输出 f32 矩阵）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraRig {
    pub target: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub dist: f64,
    /// 正交半宽（zoom 语义，REND-12）。
    pub zoom: f64,
    pub fov_y: f64,
    pub near: f64,
    pub far: f64,
}

const UP: glam::DVec3 = glam::DVec3::new(0.0, 1.0, 0.0);

impl CameraRig {
    /// 自由 look_at 构造（eye→target 指向自动解算 yaw/pitch/dist）。
    #[must_use]
    pub fn look_at(eye: [f64; 3], target: [f64; 3], _up: [f64; 3]) -> Self {
        let e = glam::DVec3::from(eye);
        let t = glam::DVec3::from(target);
        let d = e - t;
        let dist = d.length();
        let (yaw, pitch) = if dist == 0.0 {
            (0.0, 0.0)
        } else {
            (d.y.atan2(d.x), (d.z / dist).asin())
        };
        Self {
            target,
            yaw,
            pitch,
            dist,
            zoom: 1.0,
            fov_y: std::f64::consts::FRAC_PI_3,
            near: 0.1,
            far: 1000.0,
        }
    }

    /// 轨道参数构造（正交俯拍常用：pitch=90° 即地图视角）。
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn orbit(
        target: [f64; 3],
        yaw: f64,
        pitch: f64,
        dist: f64,
        zoom: f64,
        fov_y: f64,
        near: f64,
        far: f64,
    ) -> Self {
        Self {
            target,
            yaw,
            pitch,
            dist,
            zoom,
            fov_y,
            near,
            far,
        }
    }

    #[must_use]
    pub fn eye(&self) -> [f64; 3] {
        let t = glam::DVec3::from(self.target);
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        (t + self.dist * glam::DVec3::new(cp * cy, cp * sy, sp)).to_array()
    }

    /// 增量轨道（yaw 绕 Z-up 水平圆，pitch 极点软钳制——REND-13 断言面）。
    pub fn orbit_delta(&mut self, dyaw: f64, dpitch: f64) {
        self.yaw += dyaw;
        self.pitch = (self.pitch + dpitch).clamp(-1.570_796, 1.570_796);
    }

    #[must_use]
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        down(
            glam::dcamera::rh::view::look_at_mat4(
                glam::DVec3::from(self.eye()),
                glam::DVec3::from(self.target),
                UP,
            )
            .to_cols_array_2d(),
        )
    }

    /// 透视投影（REND-11：RH 深度 [0,1]；退化拒——REND-14）。
    #[must_use]
    pub fn perspective(
        &self,
        fov_y: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Option<[[f32; 4]; 4]> {
        if !(far > near) || near <= 0.0 || aspect <= 0.0 {
            return None;
        }
        let f = 1.0 / (fov_y * 0.5).tan();
        let nf = near - far;
        Some(
            Mat4::from_cols(
                Vec4::new(f / aspect, 0.0, 0.0, 0.0),
                Vec4::new(0.0, f, 0.0, 0.0),
                Vec4::new(0.0, 0.0, far / nf, -1.0),
                Vec4::new(0.0, 0.0, far * near / nf, 0.0),
            )
            .to_cols_array_2d(),
        )
    }

    /// 正交取景（REND-12：hw=zoom，hh=zoom*h/w；深度 [0,1]，退化拒——REND-14）。
    #[must_use]
    pub fn ortho_frame(
        &self,
        zoom: f32,
        width: f32,
        height: f32,
        near: f32,
        far: f32,
    ) -> Option<[[f32; 4]; 4]> {
        if !(far > near) || near <= 0.0 || zoom <= 0.0 || width <= 0.0 || height <= 0.0 {
            return None;
        }
        let hw = f64::from(zoom);
        let hh = f64::from(zoom) * (f64::from(height) / f64::from(width));
        let (near, far) = (f64::from(near), f64::from(far));
        let nf = near - far;
        Some(down(
            glam::DMat4::from_cols(
                glam::DVec4::new(1.0 / hw, 0.0, 0.0, 0.0),
                glam::DVec4::new(0.0, 1.0 / hh, 0.0, 0.0),
                glam::DVec4::new(0.0, 0.0, 1.0 / nf, 0.0), // z'=(z+n)/(n-f)：A=1/nf
                glam::DVec4::new(0.0, 0.0, near / nf, 1.0),
            )
            .to_cols_array_2d(),
        ))
    }
}
