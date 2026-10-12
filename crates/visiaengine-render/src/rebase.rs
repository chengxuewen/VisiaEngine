//! D7 layered-origin composition law (REND-19/20): clip = P * view_rot * T(origin-eye) * M_local.
//! origin and eye are f64 world coordinates; their difference is computed in the f64 domain and only then lowered to f32 -- the far origin cancels here,
//! vertices and intermediates stay small values, dodging the f32 large-coordinate ulp-swallowed precision (the old road's disease of casting world directly).

/// Compose the MVP (column-major f32 output, for the GPU uniform).
#[must_use]
pub fn compose_mvp(
    proj: &[[f32; 4]; 4],
    view_rot: &[[f32; 4]; 4],
    eye: &[f64; 3],
    origin: &[f64; 3],
    model_local: &[[f64; 4]; 4],
) -> [[f32; 4]; 4] {
    let d = glam::DVec3::new(origin[0] - eye[0], origin[1] - eye[1], origin[2] - eye[2]);
    let t_local = glam::DMat4::from_cols(
        glam::DVec4::new(1.0, 0.0, 0.0, 0.0),
        glam::DVec4::new(0.0, 1.0, 0.0, 0.0),
        glam::DVec4::new(0.0, 0.0, 1.0, 0.0),
        glam::DVec4::new(d.x, d.y, d.z, 1.0),
    );
    let m_local = glam::DMat4::from_cols_array_2d(model_local);
    let view_trans = t_local * m_local; // f64 domain: the origin-eye translation composed with the model pose
    let view_trans = [
        view_trans.x_axis,
        view_trans.y_axis,
        view_trans.z_axis,
        view_trans.w_axis,
    ]
    .map(|v| glam::Vec4::new(v.x as f32, v.y as f32, v.z as f32, v.w as f32));
    let vt = glam::Mat4::from_cols(view_trans[0], view_trans[1], view_trans[2], view_trans[3]);
    let p = glam::Mat4::from_cols_array_2d(proj);
    let r = glam::Mat4::from_cols_array_2d(view_rot);
    (p * r * vt).to_cols_array_2d()
}
