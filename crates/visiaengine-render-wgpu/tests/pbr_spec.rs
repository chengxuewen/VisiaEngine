//! N6 GGX material-block v2 gates (WGPU-35/REND-45).
//!
//! The E508 example carries the behavioral ladder probes; these tests pin the
//! backend CONTRACT side: the 48B block layout reaches the shader (a roughness
//! change must alter pixels — the WGSL-alignment bug class) and the retired
//! mock `specular` slot must not resurrect Lambert brightness.

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 320;
const H: u32 = 240;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// 12×20 UV sphere (radius 1, origin-centered) — the material-ball shape:
/// flat quads never sweep ndh across the GGX lobe (all-or-nothing highlight).
fn sphere_mesh(rings: u32, sectors: u32) -> (Vec<[f32; 3]>, Vec<u32>) {
    let mut pos = Vec::new();
    let mut idx = Vec::new();
    for i in 0..=rings {
        let phi = std::f32::consts::PI * i as f32 / rings as f32;
        for j in 0..=sectors {
            let th = std::f32::consts::TAU * j as f32 / sectors as f32;
            pos.push([phi.sin() * th.cos(), phi.sin() * th.sin(), phi.cos()]);
        }
    }
    for i in 0..rings {
        for j in 0..sectors {
            let a = i * (sectors + 1) + j;
            let b = a + sectors + 1;
            idx.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    (pos, idx)
}

fn render_sphere(
    roughness: f32,
    metallic: f32,
    specular: f32,
) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (pos, idx) = sphere_mesh(12, 20);
    let mesh = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &pos,
            normals: &pos,
            indices: &idx,
        })
        .expect("mesh");
    let mat = b
        .create_material_desc(&MaterialDesc {
            base_color: [0.75, 0.73, 0.70, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular,
            roughness,
            metallic,
        })
        .expect("mat");
    let rig = CameraRig::look_at([0.0, -13.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap(),
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.05, 0.07, 0.10, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh,
                material: mat,
                origin: [0.0; 3],
                transform: T4,
            },
        ],
    };
    b.render_to_pixels(&frame).expect("render")
}

fn peak(img: &visiaengine_render_wgpu::OffscreenFrame) -> u8 {
    img.rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| p[0])
        .max()
        .unwrap_or(0)
}

// spec: WGPU-35
// spec: REND-45
#[test]
fn ggx_roughness_reaches_shader_via_48b_block() {
    // WGSL Mat alignment pin: roughness@24 (NOT @32). If the CPU block drifts,
    // the shader reads 0.0 for both r/m and every render goes ambient-flat
    // (first-cut bug caught by exactly this class of probe — layout law).
    let smooth = render_sphere(0.05, 1.0, 0.0);
    let dull = render_sphere(0.95, 1.0, 0.0);
    let (p_s, p_d) = (peak(&smooth), peak(&dull));
    // Probe (lavapipe 2026-09-29, this camera): 255 vs 129 — the smooth lobe
    // still catches the mirror direction (255 saturation canary) and the dull
    // end desaturates hard. Two-sided: delta ≥60 keeps −40% floor (PIT-8).
    // Probe (lavapipe 2026-09-29, E508-camera shape): 255 vs 125.
    assert_eq!(p_s, 255, "smooth metallic sphere must saturate: {p_s}");
    assert_eq!(p_d, 127, "dull peak probe pin: {p_d}");
    assert!(
        u16::from(p_s) - u16::from(p_d) >= 60,
        "rough end must desaturate ≥60: {p_s} vs {p_d}"
    );
}

// spec: WGPU-35
#[test]
fn mock_specular_slot_is_retired() {
    // The retired `specular` IR field must NOT feed the shader (WGPU-14 note):
    // spec=0.0 and spec=0.9 at identical roughness/metallic render identical.
    let a = render_sphere(0.3, 0.0, 0.0);
    let b = render_sphere(0.3, 0.0, 0.9);
    assert_eq!(
        a.rgba, b.rgba,
        "specular slot must be ignored by the shader"
    );
}
