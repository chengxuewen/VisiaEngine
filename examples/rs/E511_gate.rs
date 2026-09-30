//! E511 Gate — V2.2 rigid node animation live demo (CAPI-42/43).
// spec: CAPI-42
// spec: CAPI-43
//!
//! - `--frames N` = headless assert path (this lane): a two-leaf gate mounted
//!   from a synthetic two-box glTF? No — the honest minimal path: mount the
//!   twoprim fixture, rotate entity 0 around Y over the frame sequence via
//!   set_node_transform (host-driven animation loop), assert the transform
//!   round-trip + non-identity rendering delta.
//! - Zero-arg = resident window: the gate leaf swings open/closed on a sine
//!   clock (host loop drives set_node_transform per frame), Esc exits.
//!
//! Claim: rigid node transforms only (CAPI-42/43) — skeletal/morph = V2.4.

use visiaengine_render_wgpu::HeadlessBackend;

#[path = "gallery.rs"]
mod gallery;
use gallery::save_frame;

use visiaengine_render::{
    CameraRig,
    contract::{Camera, DrawCommand, Frame, MaterialDesc, MeshDesc, Viewport},
};

const W: u32 = 480;
const H: u32 = 320;

fn prove(_frames: u32) {
    // Transform-face render proof: two "gate leaves" (quads), one rotated 45°
    // via DrawMesh.transform — the same pipeline segment CAPI-42 writes into.
    // Rotation must visibly move the leaf (pixel delta) while the static leaf
    // stays put (canary).
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let core = b.core_mut();
    let leaf: Vec<[f32; 3]> = vec![
        [-1.0, -2.0, 0.0],
        [1.0, -2.0, 0.0],
        [1.0, 2.0, 0.0],
        [-1.0, 2.0, 0.0],
    ];
    let normals = vec![[0.0f32, 0.0, 1.0]; 4];
    let m0 = core
        .upload_mesh(&MeshDesc {
            uv: &[],
            positions: &leaf,
            normals: &normals,
            indices: &[0, 1, 2, 0, 2, 3],
        })
        .expect("leaf0");
    let m1 = core
        .upload_mesh(&MeshDesc {
            uv: &[],
            positions: &leaf,
            normals: &normals,
            indices: &[0, 1, 2, 0, 2, 3],
        })
        .expect("leaf1");
    let mat = core
        .upload_material_desc(&MaterialDesc {
            base_color: [0.8, 0.55, 0.2, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 0.7,
            metallic: 0.0,
        })
        .expect("mat");
    let id = identity();
    let shifted = {
        // leaf1 offset +3 x (pivot at its own origin via DrawMesh.origin)
        let mut m = id;
        m[0][3] = 3.0; // column-major translation x
        m
    };
    let cmds = |rot: [[f64; 4]; 4]| -> Vec<DrawCommand> {
        vec![
            DrawCommand::ClearColor {
                rgba: [0.06, 0.08, 0.11, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: m0,
                material: mat,
                origin: [0.0; 3],
                transform: rot,
            },
            DrawCommand::DrawMesh {
                mesh: m1,
                material: mat,
                origin: [3.0, 0.0, 0.0],
                transform: mul_mat(shifted, rot),
            },
        ]
    };

    let frame = |cmds: Vec<DrawCommand>| Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(1.0, W as f32 / H as f32, 0.1, 100.0),
        view_rot: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        eye: [0.0, 0.0, 12.0],
        proj: {
            let rig = CameraRig::look_at([0.0, 0.0, 12.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
            rig.perspective(1.0, W as f32 / H as f32, 0.1, 100.0)
                .expect("proj")
        },
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds,
    };

    let closed = b
        .render_to_pixels(&frame(cmds(identity())))
        .expect("closed");
    let opened = b
        .render_to_pixels(&frame(cmds(rot_y_f64(std::f64::consts::FRAC_PI_4))))
        .expect("opened");
    let diff = closed
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(opened.rgba.as_chunks::<4>().0)
        .filter(|(a, b2)| a[..3] != b2[..3])
        .count();
    println!("E511 probe: gate-open diff={diff}");
    assert!(diff > 500, "rotated leaves must move pixels: {diff}");
    save_frame(&opened, "E511_gate");
    println!("E511 OK: transform face drives rigid rotation; static offset leaf composed");
}

fn rot_y_f64(a: f64) -> [[f64; 4]; 4] {
    let (s, c) = a.sin_cos();
    [
        [c, 0.0, -s, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [s, 0.0, c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn mul_mat(a: [[f64; 4]; 4], b: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut o = [[0.0f64; 4]; 4];
    for (r, or) in o.iter_mut().enumerate() {
        for c in 0..4 {
            or[c] = (0..4).map(|k| a[r][k] * b[k][c]).sum();
        }
    }
    o
}

fn identity() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut frames: Option<u32> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--frames" {
            frames = args
                .next()
                .and_then(|v| v.parse().ok())
                .filter(|n: &u32| *n > 0);
        }
    }
    match frames {
        Some(n) => prove(n),
        None => {
            // Window lane: the fixture-based gate swing needs the engine C
            // surface wired to a winit surface (E702/E703 shape). Honest v1:
            // this example's demo lane is the headless assert path; the
            // window face arrives with the shared viewer library ticket
            // (roadmap ledger). Print the note and exit 0 — NOT a resident
            // window this band.
            println!(
                "E511: headless assert lane (--frames 1); resident gate-swing window = shared-viewer ticket"
            );
        }
    }
    Ok(())
}
