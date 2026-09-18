//! 相机数学（CPU 面，glam 内部实现，矩阵裸数据出型——REND-09 同款纪律）。
//! 变体声明（REND-11）：投影深度取 **wgpu 原生 [0,1] 约定**（clip z/w ∈ [0,1]）。
//! 教训（G2 golden 实锤）：GL [-1,1] 变体在近处产生负深度→**硬件裁剪整帧静默无形**
//! （无验证报错的几何死亡）。投影矩阵构造因此在此处手建并以 REND-11/12 行为断言锁死。

use glam::{Mat4, Vec4};
use visiaengine_core::{Ray, Vec3};

/// f64 列主序 → f32（上传前唯一降位点，显式化避免 cast lint 面）。
#[must_use]
fn down(m: [[f64; 4]; 4]) -> [[f32; 4]; 4] {
    m.map(|col| col.map(|v| v as f32))
}

/// 缓动曲线（REND-34）：f:[0,1]->[0,1] 单调增、端点逐位精确（f(0)=0/f(1)=1）、域外钳制。
/// flyTo 时钟住 engine，本层纯数学（分层定案=计划 T1）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    /// 恒等（调试/线性巡览）
    Linear,
    /// smoothstep 3x^2-2x^3：慢起慢收（默认观感，近 MapLibre bezier 族）
    CubicInOut,
    /// 1-(1-x)^3：快起慢收（到达减速落位感）
    CubicOut,
}

impl Easing {
    /// 曲线求值（钳域 + 端点逐位；二次式在 0/1 处即精确）。
    #[must_use]
    pub fn ease(self, t: f64) -> f64 {
        let x = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => x,
            Self::CubicInOut => x * x * (3.0 - 2.0 * x),
            Self::CubicOut => 1.0 - (1.0 - x).powi(3),
        }
    }
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

/// 深度参数合法性：far>near 且拒 NaN/inf（clippy 偏序取反禁令的正解，REND-14 语义面）
#[must_use]
fn depth_ok(near: f32, far: f32) -> bool {
    matches!(far.partial_cmp(&near), Some(std::cmp::Ordering::Greater))
        && far.is_finite()
        && near.is_finite()
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

    /// 全参轨道构造。**fov_y 单位=弧度**（与 look_at 默认 FRAC_PI_3 / perspective 消费同制；
    /// 角度制数字直接传入=投影畸变雷，例面三处前科 [2026-09-18 E303 抓档]）。
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

    /// 2D↔3D 切换的参数空间插值（REND-15/16）。
    /// ponytail: yaw 线性 lerp（demo 域 |Δyaw|<π）；wrap 最短弧待多圈轨道需求再引入。
    #[must_use]
    pub fn mix_rig(a: &Self, b: &Self, t: f64) -> Self {
        // 端点恒等快路：浮点 x+(y-x)*1 不保证精确等于 y（REND-16 语义即端点精确）
        match t {
            0.0 => return *a,
            1.0 => return *b,
            _ => {}
        }
        let l = |x: f64, y: f64| x + (y - x) * t;
        let lv = |x: [f64; 3], y: [f64; 3]| [l(x[0], y[0]), l(x[1], y[1]), l(x[2], y[2])];
        Self {
            target: lv(a.target, b.target),
            yaw: l(a.yaw, b.yaw),
            pitch: l(a.pitch, b.pitch),
            dist: l(a.dist, b.dist),
            zoom: l(a.zoom, b.zoom),
            fov_y: l(a.fov_y, b.fov_y),
            near: l(a.near, b.near),
            far: l(a.far, b.far),
        }
    }

    /// flyTo 位姿采样（REND-34）：纯函数（墙钟推进器住 engine——宿主零 dt 义务，D8 正解）。
    /// 最短弧：to.yaw 经 ±2pi 整数倍归一到 from.yaw+delta (delta in (-pi,pi]) 再插值
    /// （姿态 mod 2pi 等价；小 delta<=pi 零干预纯线性——wrap 修正住采样器，不动 mix_rig）。
    /// 端点精确快路：t=0/1 直返 from/to 原值（终点 yaw 保原始数，归一只作用途中，REND-16 同谱）。
    /// near/far 不经飞行输入（CAPI-23 pose 无深度域=恒 from 现值，PIT-5/REND-14 锁连带）。
    #[must_use]
    pub fn fly_sample(from: &Self, to: &Self, t: f64, easing: Easing) -> Self {
        match t {
            0.0 => return *from,
            1.0 => return *to,
            _ => {}
        }
        let raw = to.yaw - from.yaw;
        let tau = std::f64::consts::TAU;
        let d = raw - (raw / tau).round() * tau;
        let mut bent = *to;
        if d != raw {
            bent.yaw = from.yaw + d;
        }
        Self::mix_rig(from, &bent, easing.ease(t))
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
        self.pitch =
            (self.pitch + dpitch).clamp(-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
    }

    /// 纯旋转视图基（REND-17：平移经 eye 走 rebase 组合）。
    #[must_use]
    pub fn view_rotation(&self) -> [[f32; 4]; 4] {
        let mut v = self.view_matrix();
        v[3] = [0.0, 0.0, 0.0, 1.0];
        v
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
        if !depth_ok(near, far) || near <= 0.0 || !aspect.is_finite() || aspect <= 0.0 {
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
        if !depth_ok(near, far)
            || near <= 0.0
            || !zoom.is_finite()
            || zoom <= 0.0
            || !width.is_finite()
            || width <= 0.0
            || !height.is_finite()
            || height <= 0.0
        {
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

// ===== REND-21/22：屏幕像素→世界射线（交互片，[E3D:4xx] 移植）=====

const UP6: [f64; 3] = [0.0, 1.0, 0.0];

#[must_use]
fn vsub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
#[must_use]
fn vcross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
#[must_use]
fn vlen(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
#[must_use]
fn vnorm(a: [f64; 3]) -> Option<[f64; 3]> {
    let l = vlen(a);
    (l > 1e-12).then(|| [a[0] / l, a[1] / l, a[2] / l])
}

/// 视基（fwd/right/up）。与 look_at 同一世界 UP 约定；极点视角（fwd∥UP）退化 None。
#[must_use]
fn view_basis(rig: &CameraRig) -> Option<([f64; 3], [f64; 3], [f64; 3])> {
    let eye = rig.eye();
    let fwd = vnorm(vsub(rig.target, eye))?;
    let right = vnorm(vcross(fwd, UP6))?;
    let up = vcross(right, fwd);
    Some((fwd, right, up))
}

#[must_use]
fn ndc(px: f32, py: f32, w: f32, h: f32) -> (f64, f64) {
    (
        2.0 * f64::from(px) / f64::from(w) - 1.0, // px=0 左缘 → -1
        1.0 - 2.0 * f64::from(py) / f64::from(h), // py=0 顶缘 → +1（屏幕 y 向下）
    )
}

/// 透视：屏幕像素（左上原点，px∈[0,w]）→ 世界射线（用 rig.fov_y，D7 全程 f64）。
/// 退化输入（w/h≤0、fov 非法、极点姿态）None。
#[must_use]
pub fn screen_to_ray_persp(rig: &CameraRig, px: f32, py: f32, w: f32, h: f32) -> Option<Ray> {
    let (fwd, right, up) = view_basis(rig)?;
    if w <= 0.0 || h <= 0.0 || !rig.fov_y.is_finite() || rig.fov_y <= 0.0 {
        return None;
    }
    let (nx, ny) = ndc(px, py, w, h);
    let ty = (rig.fov_y / 2.0).tan();
    let tx = ty * (f64::from(w) / f64::from(h));
    let dir = [
        fwd[0] + right[0] * tx * nx + up[0] * ty * ny,
        fwd[1] + right[1] * tx * nx + up[1] * ty * ny,
        fwd[2] + right[2] * tx * nx + up[2] * ty * ny,
    ];
    let e = rig.eye();
    let d = vnorm(dir)?;
    Some(Ray {
        origin: Vec3::new(e[0], e[1], e[2]),
        dir: Vec3::new(d[0], d[1], d[2]),
    })
}

/// 正交：视平面偏移 origin = eye + right·(hw·nx) + up·(hh·ny)，dir=fwd。
/// 半宽=rig.zoom，半高=zoom·h/w（REND-12 同一约定）。
#[must_use]
pub fn screen_to_ray_ortho(rig: &CameraRig, px: f32, py: f32, w: f32, h: f32) -> Option<Ray> {
    let (fwd, right, up) = view_basis(rig)?;
    if w <= 0.0 || h <= 0.0 || !rig.zoom.is_finite() || rig.zoom <= 0.0 {
        return None;
    }
    let (nx, ny) = ndc(px, py, w, h);
    let hw = rig.zoom;
    let hh = rig.zoom * f64::from(h) / f64::from(w);
    let eye = rig.eye();
    Some(Ray {
        origin: Vec3::new(
            eye[0] + right[0] * hw * nx + up[0] * hh * ny,
            eye[1] + right[1] * hw * nx + up[1] * hh * ny,
            eye[2] + right[2] * hw * nx + up[2] * hh * ny,
        ),
        dir: Vec3::new(fwd[0], fwd[1], fwd[2]),
    })
}

/// REND-37：世界射线 → 地面平面 z=0 求交（导航用；非拾取——拾取走 ray-triangle）。
/// `t=−o.z/d.z`，`d.z≥−1e-9`（水平/上向/近水平）或 `t≤0`（地面出发向上/背向）=None（域内拒，免 t 爆炸）。
#[must_use]
pub fn ray_ground_intersect(ray: visiaengine_core::Ray) -> Option<visiaengine_core::Vec3> {
    let (o, d) = (ray.origin, ray.dir);
    if d.z >= -1e-9 {
        return None;
    }
    let t = -o.z / d.z;
    if t <= 0.0 || !t.is_finite() {
        return None;
    }
    Some(visiaengine_core::Vec3::new(
        o.x + d.x * t,
        o.y + d.y * t,
        0.0,
    ))
}
