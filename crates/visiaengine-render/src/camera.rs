//! Camera math (CPU face, glam as the internal implementation, matrices exported as bare data -- the same discipline as REND-09).
//! Variant declaration (REND-11): projection depth takes the **wgpu-native [0,1] convention** (clip z/w in [0,1]).
//! Lesson (G2 golden proof): the GL [-1,1] variant produces negative depth up close -> **the hardware clips the whole frame silently away**
//! (geometric death with no validation error). The projection matrix is therefore hand-built here and locked by the REND-11/12 behavior assertions.

use glam::{Mat4, Vec4};
use visiaengine_core::{Ray, Vec3};

/// f64 column-major -> f32 (the single precision-lowering point before upload, made explicit to dodge the cast-lint face).
#[must_use]
fn down(m: [[f64; 4]; 4]) -> [[f32; 4]; 4] {
    m.map(|col| col.map(|v| v as f32))
}

/// Easing curves (REND-34): f:[0,1]->[0,1] monotonic increasing, endpoints bit-exact (f(0)=0/f(1)=1), out-of-domain clamped.
/// The flyTo clock lives in the engine, this layer is pure math (layering settled = plan T1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    /// Identity (debug / linear tour)
    Linear,
    /// smoothstep 3x^2-2x^3: slow start slow settle (default look, near the MapLibre bezier family)
    CubicInOut,
    /// 1-(1-x)^3: fast start slow settle (arrival-deceleration landing feel)
    CubicOut,
}

impl Easing {
    /// Curve evaluation (clamped domain + bit-exact endpoints; the quadratic is exact at 0/1 by construction).
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

/// Orbit camera rig: target at the center, yaw/pitch/dist spherical (f64 steady state, f32 matrices out).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraRig {
    pub target: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub dist: f64,
    /// Orthographic half-width (zoom semantics, REND-12).
    pub zoom: f64,
    pub fov_y: f64,
    pub near: f64,
    pub far: f64,
}

/// Depth-parameter legality: far > near and NaN/inf rejected (the right answer to the clippy partial-order negation ban, the REND-14 semantics face)
#[must_use]
fn depth_ok(near: f32, far: f32) -> bool {
    matches!(far.partial_cmp(&near), Some(std::cmp::Ordering::Greater))
        && far.is_finite()
        && near.is_finite()
}

const UP: glam::DVec3 = glam::DVec3::new(0.0, 1.0, 0.0);

impl CameraRig {
    /// Free look_at construction (the eye->target direction auto-solves yaw/pitch/dist).
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

    /// Full-parameter orbit construction. **fov_y unit = radians** (same system as the look_at default FRAC_PI_3 / the perspective consumption;
    /// feeding degree numbers straight in = a projection-distortion mine, three priors on the case file [2026-09-18 E303 capture]).
    /// Orbit-parameter construction (common for orthographic top views: pitch = 90 degrees is the map view).
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

    /// Parameter-space interpolation for the 2D<->3D switch (REND-15/16).
    /// ponytail: yaw linear lerp (demo domain |dyaw| < pi); the wrap shortest arc arrives when multi-loop orbit needs demand it.
    #[must_use]
    pub fn mix_rig(a: &Self, b: &Self, t: f64) -> Self {
        // Endpoint identity fast path: floating x+(y-x)*1 does not guarantee exactly y (the REND-16 semantics are endpoint exactness)
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

    /// flyTo pose sampling (REND-34): pure function (the wall-clock thruster lives in the engine -- zero dt duty for the host, the D8 right answer).
    /// Shortest arc: to.yaw is normalized over integer +/-2pi into from.yaw + delta (delta in (-pi,pi]) before interpolating
    /// (poses are mod-2pi equivalent; small delta <= pi passes untouched pure linear -- the wrap correction lives in the sampler, not mix_rig).
    /// Endpoint-exact fast path: t = 0/1 returns the from/to original values directly (the endpoint keeps the raw yaw, normalization acts mid-flight only, same scheme as REND-16).
    /// near/far never pass through flight input (CAPI-23 pose carries no depth domain = always the from values, the PIT-5/REND-14 lock linkage).
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

    /// Incremental orbit (yaw around the Z-up horizontal circle, pitch softly clamped at the poles -- the REND-13 assertion face).
    pub fn orbit_delta(&mut self, dyaw: f64, dpitch: f64) {
        self.yaw += dyaw;
        self.pitch =
            (self.pitch + dpitch).clamp(-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
    }

    /// Pure-rotation view basis (REND-17: translation rides eye through the rebase composition).
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
    /// Perspective projection (REND-11: RH depth [0,1]; degenerates rejected -- REND-14).
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

    /// Orthographic framing (REND-12: hw = zoom, hh = zoom*h/w; depth [0,1], degenerates rejected -- REND-14).
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
                glam::DVec4::new(0.0, 0.0, 1.0 / nf, 0.0), // z' = (z+n)/(n-f): A = 1/nf
                glam::DVec4::new(0.0, 0.0, near / nf, 1.0),
            )
            .to_cols_array_2d(),
        ))
    }
}

// ===== REND-21/22: screen pixels -> world rays (the interaction slice, [E3D:4xx] port) =====

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

/// View basis (fwd/right/up). Same world UP convention as look_at; pole views (fwd parallel UP) degenerate to None.
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
        2.0 * f64::from(px) / f64::from(w) - 1.0, // px = 0 left edge -> -1
        1.0 - 2.0 * f64::from(py) / f64::from(h), // py = 0 top edge -> +1 (screen y downward)
    )
}

/// Perspective: screen pixels (top-left origin, px in [0,w]) -> world ray (uses rig.fov_y, all f64 per D7).
/// Degenerate input (w/h <= 0, illegal fov) -> None.
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

/// Orthographic: view-plane offset origin = eye + right*(hw*nx) + up*(hh*ny), dir = fwd.
/// (the same REND-12 convention).
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

/// REND-37: world ray -> ground-plane z = 0 intersection (CPU companion of navigate_click; non-finite / parallel = None).
/// `t=-o.z/d.z`, and `d.z >= -1e-9` (horizontal / upward / near-horizontal) or `t <= 0` (starting from the ground upward / pointing away) = None (domain rejection to avoid t explosion).
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
