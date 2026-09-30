//! WGPU-39: SH-9 environment irradiance — env-on shading delta + zero-vector
//! bitwise legacy restore + flag-index discipline (float 100 = byte 400).

use visiaengine_render::RenderBackend;
use visiaengine_render::contract::{DrawCommand, Frame, Viewport};
use visiaengine_render_wgpu::HeadlessBackend;

// spec: WGPU-39
#[test]
fn env_sh_on_changes_shading_zero_restores_legacy() {
    let mut b = HeadlessBackend::new(160, 120).expect("adapter");
    // minimal geometry: one lit quad facing the camera
    let positions: Vec<[f32; 3]> = vec![
        [-4.0, -3.0, 0.0],
        [4.0, -3.0, 0.0],
        [4.0, 3.0, 0.0],
        [-4.0, 3.0, 0.0],
    ];
    let normals = vec![[0.0f32, 0.0, 1.0]; 4];
    let mesh = b
        .create_mesh(&visiaengine_render::MeshDesc {
            uv: &[],
            positions: &positions,
            normals: &normals,
            indices: &[0, 1, 2, 0, 2, 3],
        })
        .expect("mesh");
    let material = b
        .create_material_desc(&visiaengine_render::MaterialDesc {
            base_color: [0.7, 0.5, 0.3, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 0.5,
            metallic: 0.0,
        })
        .expect("mat");
    let cmds = vec![
        DrawCommand::ClearColor {
            rgba: [0.05, 0.05, 0.05, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh,
            material,
            origin: [0.0; 3],
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        },
    ];
    let frame = |cmds: &Vec<DrawCommand>| Frame {
        viewport: Viewport::new(160, 120, 1.0),
        camera: visiaengine_render::Camera::perspective(1.0, 160.0 / 120.0, 0.1, 100.0),
        view_rot: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        eye: [0.0, 0.0, 10.0],
        proj: {
            let rig = visiaengine_render::CameraRig::look_at(
                [0.0, 0.0, 10.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            );
            rig.perspective(1.0, 160.0 / 120.0, 0.1, 100.0)
                .expect("proj")
        },
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds.clone(),
    };

    let off = b.render_to_pixels(&frame(&cmds)).expect("off");
    // hand-built asymmetric SH (strong +y sky, -y ground dark) — nonzero by
    // construction, no fixture dependency.
    let mut sh = [[0.0f32; 3]; 9];
    sh[0] = [0.6, 0.55, 0.5];
    sh[1] = [0.4, 0.35, 0.3];
    b.set_env_sh(sh);
    let on = b.render_to_pixels(&frame(&cmds)).expect("on");
    b.set_env_sh([[0.0; 3]; 9]);
    let restored = b.render_to_pixels(&frame(&cmds)).expect("restored");

    let diff = off
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(on.rgba.as_chunks::<4>().0)
        .filter(|(x, y)| x[..3] != y[..3])
        .count();
    // probe (lavapipe 160x120): the quad fills the center; most of its pixels
    // shift. Floor conservative (PIT-8): 200 px.
    eprintln!("WGPU-39 probe: env diff={diff}");
    assert!(diff > 200, "SH env must change geometry shading: {diff}");
    // zero-vector restore = bitwise legacy (flag float-100 discipline)
    assert_eq!(
        off.rgba, restored.rgba,
        "zero-SH restore must be bitwise legacy"
    );
}
