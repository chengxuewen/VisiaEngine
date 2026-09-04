//! D7 分层 origin 组合律（REND-19/20）：clip = P · view_rot · T(origin−eye) · M_local。
//! origin、eye 为 f64 世界坐标，其差在 f64 域算完再降 f32——远原点在此相消，
//! 顶点/中间量恒为小值，规避 f32 大坐标 ulp 吞精度（旧路 world 直 cast 的病）。

/// 组合 MVP（列主序 f32 输出，供 GPU uniform）。
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
    let view_trans = t_local * m_local; // f64 域：origin−eye 平移 ∘ 模型位姿
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
