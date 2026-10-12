//! WGPU-19: shadow pre-pass + receiver PCF. Behavior locks [PIT-8: probe the threshold first, then tighten]:
//! (1) bright and dark states coexist (not all-black/all-bright = the PIT-5 depth-domain symptom-level deadlock);
//! (2) the shadow-patch dark pixels' row centroid is on the anti-light side (+y tilted to the light -> shadow falls screen-down);
//! (3) the far ground patch's std is bounded (the self-shadow acne high-frequency large-variance lock);
//! (4) the None position has no dark patches (off-position behavior = the status quo). Bitwise zero regression = the existing golden full set self-proves.

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MeshDesc, RenderBackend,
    ShadowBias, ShadowSetup, Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};

const W: u32 = 96;
const H: u32 = 96;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
/// The world light direction (pointing to the light source): +y tilt up, +z high.
const LIGHT_DIR: [f32; 3] = [0.0, 0.42, 0.91];

fn scene(with_shadow: Option<ShadowSetup>) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let ground = b
        .create_mesh(&MeshDesc {
            positions: &[
                [-4.0, -4.0, 0.0],
                [4.0, -4.0, 0.0],
                [4.0, 4.0, 0.0],
                [-4.0, 4.0, 0.0],
            ],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        })
        .unwrap();
    let gmat = b.create_material([0.8, 0.8, 0.8, 1.0]).unwrap();
    let (pos, nrm, idx) = unit_box_mesh();
    let cube = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .unwrap();
    let wmat = b.create_material([1.0, 1.0, 1.0, 1.0]).unwrap();
    let iid = b
        .create_instances(&InstanceDesc {
            data: &[Instance::new([0.0, 0.0, 0.0], 2.0, [1.0, 1.0, 1.0])],
        })
        .unwrap();

    let rig = CameraRig::look_at([0.0, -7.0, 6.0], [0.0, 0.0, 0.6], [0.0, 1.0, 0.0]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 0.1,
        shadow: with_shadow,
        edl: None,
        post: Vec::new(),
        clip: None,
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: ground,
                material: gmat,
                origin: [0.0; 3],
                transform: T4,
            },
            DrawCommand::DrawInstances {
                mesh: cube,
                material: wmat,
                instances: iid,
                origin: [0.0; 3],
                transform: T4,
            },
        ],
    };
    b.render_to_pixels(&frame).expect("render")
}

/// The light's orthographic camera: 30 down along the light direction (the REND-31 construction contract: the rig-road triple).
fn setup(size: f32) -> ShadowSetup {
    let n =
        (LIGHT_DIR[0] * LIGHT_DIR[0] + LIGHT_DIR[1] * LIGHT_DIR[1] + LIGHT_DIR[2] * LIGHT_DIR[2])
            .sqrt();
    let eye = [
        f64::from(LIGHT_DIR[0] / n) * 30.0,
        f64::from(LIGHT_DIR[1] / n) * 30.0,
        f64::from(LIGHT_DIR[2] / n) * 30.0 + 0.6,
    ];
    let rig = CameraRig::look_at(eye, [0.0, 0.0, 0.6], [0.0, 1.0, 0.0]);
    ShadowSetup {
        proj: rig
            .ortho_frame(10.0, W as f32, H as f32, 1.0, 80.0)
            .unwrap(),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        light_dir: LIGHT_DIR,
        size,
        bias: ShadowBias {
            constant: -1.2,
            slope: -1.5,
        },
    }
}

/// Ground-gray judgment (equal channels, the 20..230 band -- excluding the pure background / direct-lit cube side faces' 255 highlights).
/// CORE-16 domain note: the encoding curve lifts dark values -- lit face 198 / shadow core 84 / penumbra 105..181 (the old 51 series).
/// N6 GGX reprobe (2026-09-29): lit face 198->223 (shadow-on light direction ndl 0.91 shifts the whole band right),
/// the shadow core 84 unmoved -- the window upper bound 205->230 adapted to the new brightness truth (the double-sided band kept).
fn gray(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> Option<u8> {
    let i = ((y * W + x) * 4) as usize;
    let (r, g, bl) = (img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]);
    (r == g && g == bl && r > 20 && r < 230).then_some(r)
}

// spec: WGPU-19
#[test]
fn shadow_exists_not_all_black_or_white() {
    let img = scene(Some(setup(0.02)));
    let (mut dark, mut lit) = (0u32, 0u32);
    for y in 0..H {
        for x in 0..W {
            if let Some(r) = gray(&img, x, y) {
                if r <= 95 {
                    dark += 1;
                } else {
                    lit += 1;
                }
            }
        }
    }
    assert!(
        dark >= 30 && lit >= 300,
        "bright/dark two-state imbalance dark={dark} lit={lit}"
    );
}

// spec: WGPU-19
#[test]
fn shadow_centroid_lies_anti_light_direction() {
    let img = scene(Some(setup(0.02)));
    let (mut n, mut sum) = (0u64, 0u64);
    for y in 0..H {
        for x in 0..W {
            // CORE-16 domain re-pinned: shadow core 84 + near-side penumbra band <= 140 (the old r < 70 domain) -- the dark centroid
            // sits on the anti-light physical fact unchanged, the capture band shifted right after quantization compression (measured cy = 49.9 > 48)
            if gray(&img, x, y).is_some_and(|r| r <= 140) {
                n += 1;
                sum += u64::from(y);
            }
        }
    }
    assert!(n > 30, "no measurable shadow patch");
    let cy = sum as f64 / n as f64;
    assert!(
        cy > f64::from(H) / 2.0,
        "the dark centroid row {cy:.1} is not on the anti-light side (below)"
    );
}

// spec: WGPU-19
#[test]
fn far_ground_patch_is_smooth() {
    let img = scene(Some(setup(0.02)));
    let mut vals: Vec<u32> = Vec::new();
    for y in 74..92 {
        for x in 6..26 {
            if let Some(r) = gray(&img, x, y) {
                vals.push(u32::from(r));
            }
        }
    }
    assert!(
        vals.len() > 150,
        "the far-ground sample is insufficient {}",
        vals.len()
    );
    let mean = vals.iter().sum::<u32>() as f64 / vals.len() as f64;
    let var = vals
        .iter()
        .map(|v| (f64::from(*v) - mean).powi(2))
        .sum::<f64>()
        / vals.len() as f64;
    assert!(
        var.sqrt() < 10.0,
        "the ground patch std {:.1} is too large (acne)",
        var.sqrt()
    );
}

// spec: WGPU-19
#[test]
fn none_bit_produces_no_dark_patch() {
    let img = scene(None);
    let dark = (0..H)
        .map(|y| {
            (0..W)
                .filter(|&x| gray(&img, x, y).is_some_and(|r| r < 70))
                .count() as u32
        })
        .sum::<u32>();
    assert!(dark < 10, "the None position produced dark patches {dark}");
}

/// The penumbra band (intermediate grays inside the ground window; the CORE-16 domain = 96..=190, measured hard = 96 / soft = 3613).
fn penumbra_count(size: f32) -> u32 {
    let img = scene(Some(setup(size)));
    let mut mid = 0u32;
    for y in 40..H {
        for x in 0..W {
            if gray(&img, x, y).is_some_and(|r| (96..=190).contains(&r)) {
                mid += 1;
            }
        }
    }
    mid
}

// spec: WGPU-20
#[test]
fn penumbra_grows_with_light_size() {
    // PCSS [E3D:B3]: the source angular size rises -> the penumbra width rises (intermediate grays monotonic increasing); and far below the solid band (bounded)
    let (hard, soft) = (penumbra_count(0.02), penumbra_count(1.5));
    assert!(
        hard < 400,
        "the near-hard shadow has too many intermediate grays {hard}"
    );
    assert!(
        soft > hard * 2,
        "the penumbra does not expand with size hard={hard} soft={soft}"
    );
    assert!(soft < 6000, "the penumbra exploded (clamp missing) {soft}");
}
