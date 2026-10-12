//! WGPU-14/15: the Textured pipeline (sampling x base_color x shade) and fractional repeat;
//! the Flat pipeline's per-pixel zero regression is self-proven by the existing golden full set.

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, TextureDesc, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 64;
const H: u32 = 64;

/// 4x4 RGBA checkerboard: (i+j) even = red, odd = green.
fn checker() -> Vec<u8> {
    let mut px = Vec::new();
    for j in 0..4u32 {
        for i in 0..4u32 {
            px.extend_from_slice(if (i + j) % 2 == 0 {
                &[255, 0, 0, 255]
            } else {
                &[0, 255, 0, 255]
            });
        }
    }
    px
}

struct Scene {
    frame: Frame,
}

struct QuadMesh {
    positions: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

fn quad_and_frame(tex_mat: u64) -> (QuadMesh, Scene) {
    // ±2.5: tiles the 64px viewport (under z = 3 / fov60 the ±1 occupies only half the screen, the sampling window would leave the face)
    let positions = vec![
        [-2.5, -2.5, 0.],
        [2.5, -2.5, 0.],
        [2.5, 2.5, 0.],
        [-2.5, 2.5, 0.],
    ];
    let uv = vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
    let indices = vec![0u32, 1, 2, 0, 2, 3];
    let rig = CameraRig::look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0., 0., 0., 1.],
            },
            DrawCommand::DrawMesh {
                mesh: 0,
                material: tex_mat,
                origin: [0.; 3],
                transform: [
                    [1., 0., 0., 0.],
                    [0., 1., 0., 0.],
                    [0., 0., 1., 0.],
                    [0., 0., 0., 1.],
                ],
            },
        ],
    };
    (
        QuadMesh {
            positions,
            uv,
            indices,
        },
        Scene { frame },
    )
}

fn px(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [
        img.rgba[i],
        img.rgba[i + 1],
        img.rgba[i + 2],
        img.rgba[i + 3],
    ]
}

fn draw(repeat: [f32; 2]) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tid = b
        .upload_texture(&TextureDesc {
            rgba: &checker(),
            width: 4,
            height: 4,
        })
        .expect("texture");
    let mid = b
        .create_material_desc(&visiaengine_render::MaterialDesc {
            base_color: [1., 1., 1., 1.],
            texture: Some(tid),
            repeat,
            specular: 0.0,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        })
        .expect("material");
    let (q, mut sc) = quad_and_frame(mid);
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &q.positions,
            normals: &[[0., 0., 1.]; 4],
            indices: &q.indices,
            uv: &q.uv,
        })
        .unwrap();
    if let DrawCommand::DrawMesh {
        mesh: m, material, ..
    } = &mut sc.frame.commands[1]
    {
        *m = mesh;
        *material = mid;
    }
    b.render_to_pixels(&sc.frame).expect("render")
}

/// A 64x64 horizontal gradient texture scene (R = 4i, G = 4j) -- the WGPU-15 wrap oracle.
fn draw_grad(repeat: [f32; 2]) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut grad = Vec::new();
    for j in 0..64u32 {
        for i in 0..64u32 {
            grad.extend_from_slice(&[(i * 4) as u8, (j * 4) as u8, 0, 255]);
        }
    }
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tid = b
        .upload_texture(&TextureDesc {
            rgba: &grad,
            width: 64,
            height: 64,
        })
        .expect("texture");
    let mid = b
        .create_material_desc(&visiaengine_render::MaterialDesc {
            base_color: [1., 1., 1., 1.],
            texture: Some(tid),
            repeat,
            specular: 0.0,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        })
        .expect("material");
    let (q, mut sc) = quad_and_frame(mid);
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &q.positions,
            normals: &[[0., 0., 1.]; 4],
            indices: &q.indices,
            uv: &q.uv,
        })
        .unwrap();
    if let DrawCommand::DrawMesh {
        mesh: m, material, ..
    } = &mut sc.frame.commands[1]
    {
        *m = mesh;
        *material = mid;
    }
    b.render_to_pixels(&sc.frame).expect("render")
}

fn draw_flat_mat_rough(roughness: f32) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let mid = b
        .create_material_desc(&visiaengine_render::MaterialDesc {
            base_color: [1., 1., 1., 1.],
            texture: None,
            repeat: [1., 1.],
            specular: 0.0, // retired mock slot (WGPU-35)
            roughness,
            metallic: 0.0,
        })
        .expect("material");
    let (q, mut sc) = quad_and_frame(mid);
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &q.positions,
            normals: &[[0., 0., 1.]; 4],
            indices: &q.indices,
            uv: &q.uv,
        })
        .unwrap();
    if let DrawCommand::DrawMesh {
        mesh: m, material, ..
    } = &mut sc.frame.commands[1]
    {
        *m = mesh;
        *material = mid;
    }
    b.render_to_pixels(&sc.frame).expect("render")
}

// spec: WGPU-14
#[test]
fn textured_pipeline_samples_checker_x_base_x_shade() {
    let img = draw([1., 1.]);
    // center-region scan (avoiding the quad edges): the checkerboard's two polarities each form a sheet, luminance band 170..230
    // verifies the shade(~ 0.62) x base(1.0) x texel(Srgb-decoded 1.0) multiply chain
    // CORE-16 domain: encode(0.62) = 207 (rejects 255 from an unmultiplied shade -- the 230 upper bound keeps a distance of 25)
    let (mut reds, mut greens) = (0u32, 0u32);
    for y in 20..44 {
        for x in 20..44 {
            let p = px(&img, x, y);
            assert!(
                p[0] < 230 && p[1] < 230,
                "the unmultiplied-shade chain emitted 255 {p:?}"
            );
            // the hue-band predicate (the CORE-16 domain: after linear mixing the encoding lifts midtones, absolute thresholds are brittle):
            // the dominant channel strong + large channel difference = checkerboard polarity; mixed bands (r ~ g) naturally fall out
            if p[0] > 150 && p[0] > p[1] + 50 {
                reds += 1;
            } else if p[1] > 150 && p[1] > p[0] + 50 {
                greens += 1;
            }
        }
    }
    assert!(
        reds >= 15 && greens >= 15,
        "the checkerboard dual polarity missing red={reds} green={greens}"
    );
    // (threshold 15 = the conservative seat of the measured half-region; the hue-band predicate above)
}

// spec: WGPU-15
#[test]
fn fractional_repeat_shifts_pattern() {
    // the gradient texture (R = 4i horizontal ramp) is the exact repeat oracle: repeat = 1 has no wrap inside one row,
    // repeat = 2 samples w = 2u and hits exactly one wrap cliff mid-way (the checkerboard odd-even assertion does not hold under linear filtering, discarded)
    let one = draw_grad([1., 1.]);
    let two = draw_grad([2., 2.]);
    let wraps = |img: &visiaengine_render_wgpu::OffscreenFrame| -> u32 {
        (1..64u32)
            .filter(|&x| px(img, x - 1, 32)[0] as i16 - px(img, x, 32)[0] as i16 > 100)
            .count() as u32
    };
    assert_eq!(wraps(&one), 0, "the repeat=1 ramp must not wrap");
    assert_eq!(wraps(&two), 1, "repeat=2 must wrap exactly once");
}

// spec: WGPU-14
#[test]
fn flat_pipeline_zero_regression_marker() {
    // the Flat path (texture-less material) rides the existing pipeline: the twoprim fixture's center luminance band (family assertion,
    // the exact-pixel regression is carried by the offscreen_golden / geo_pipeline full sets)
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let doc = visiaengine_io_gltf::load_gltf(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/twoprim.glb"
    ))
    .unwrap();
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.],
    }];
    let mut handles = Vec::new();
    for e in doc.entities() {
        let m = b
            .create_mesh(&MeshDesc {
                positions: &e.mesh.positions,
                normals: &e.mesh.normals,
                indices: &e.mesh.indices,
                uv: &[],
            })
            .unwrap();
        let mat = b.create_material(e.mesh.base_color).unwrap();
        handles.push((m, mat));
    }
    for (m, mat) in &handles {
        commands.push(DrawCommand::DrawMesh {
            mesh: *m,
            material: *mat,
            origin: [0.; 3],
            transform: [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
        });
    }
    let rig = CameraRig::look_at([0., 0., 3.], [0., 0., 0.], [0., 1., 0.]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    };
    let img = b.render_to_pixels(&frame).expect("render flat");
    let c = px(&img, 32, 32);
    assert!(
        c[0] + c[1] + c[2] > 40,
        "the Flat image is not background {c:?}"
    );
}

// spec: REND-25
#[test]
fn meshdesc_uv_optional_contract() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let nrm = [[0.0, 0.0, 1.0]; 3];
    let idx = [0u32, 1, 2];
    // an empty piece = legal (the Flat existing form: the whole mesh's uv zero-filled)
    assert!(
        b.create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[]
        })
        .is_ok()
    );
    // non-empty and length != vertex count = rejected (no silent padding)
    assert!(
        b.create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[[0.0, 0.0]]
        })
        .is_err()
    );
}

// spec: WGPU-14
#[test]
fn material_unknown_texture_is_err() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let r = b.create_material_desc(&visiaengine_render::MaterialDesc {
        base_color: [1.0; 4],
        texture: Some(999),
        repeat: [1.0, 1.0],
        specular: 0.0,
        roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
        metallic: 0.0,
    });
    assert!(
        r.is_err(),
        "an unknown texture reference must not silently downgrade to Flat"
    );
}

// spec: WGPU-14
// N6 GGX re-probe (2026-09-29): the mock specular slot is RETIRED by WGPU-35 —
// `specular` is IR-compat only and the shader ignores it. The old brightness
// ladder (spec 0.8 > spec 0) is replaced by the GGX contract probe: a flat
// quad facing the camera shades UNIFORM (directional light + constant n), so
// the whole-window value IS the signal — roughness reaches the shader through
// both the lobe-energy term and the IBL-lite ambient (1-r*0.5).
#[test]
fn roughness_ggx_shaping_replaces_mock() {
    let rough = draw_flat_mat_rough(0.8);
    let smooth = draw_flat_mat_rough(0.2);
    let val = |img: &visiaengine_render_wgpu::OffscreenFrame| -> u32 {
        let mut acc = 0u64;
        for y in 20..44 {
            for x in 20..44 {
                acc += u64::from(px(img, x, y)[0]);
            }
        }
        (acc / (24 * 24)) as u32
    };
    let (r, s) = (val(&rough), val(&smooth));
    // Probe (lavapipe 2026-09-29): smooth 219, rough 204 — smooth is brighter
    // (tighter GGX lobe keeps more energy on-axis + higher ambient floor).
    assert!(s > r, "GGX energy law: smooth {s} must exceed rough {r}");
    assert!((210..=228).contains(&s), "smooth plateau band: {s}");
    assert!((195..=214).contains(&r), "rough plateau band: {r}");
}
