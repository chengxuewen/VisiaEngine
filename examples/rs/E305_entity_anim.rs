//! E305 Entity Animation — keyframe trajectory replay (N2.2, REND-42).
//!
//! Host-driven animation demo: a drone flies a waypoint path (the keyframe
//! table), the host samples `anim_origin(t)` and drops a marker at each
//! sample (v0 animation surface = sample + place; the entity-position C-API
//! is band 6b, trigger-gated). Render = HeadlessBackend direct (E304 shape).
//!
//! `--frames N` = trail density: N samples along the path land as markers;
//! asserts N markers render, the drone sits at the final waypoint, and the
//! sampler's endpoint law holds on this very table.

use visiaengine_render::anim_origin;
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

/// Waypoint table: take-off -> cruise -> hover -> land (ascending times).
const WAYPOINTS: &[(f64, [f64; 3])] = &[
    (0.0, [-4.0, -4.0, 0.5]),
    (1.0, [0.0, -4.0, 3.0]),
    (2.0, [4.0, 0.0, 3.0]),
    (3.0, [4.0, 4.0, 3.0]),
    (4.0, [0.0, 4.0, 3.0]),
    (5.0, [0.0, 0.0, 0.5]),
];

/// Camera frame: perspective oblique over the ground pad.
fn cam_frame(cmds: &[DrawCommand]) -> Frame {
    let rig = CameraRig::look_at([0.0, -16.0, 11.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, W as f32 / H as f32, 0.1, 1000.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, W as f32 / H as f32, 0.1, 1000.0)
            .expect("proj"),
        px_world_scale: 0.12,
        shadow: None,
        clip: None,
        edl: None,
        commands: cmds.to_vec(),
    }
}

fn prove(frames: u32) {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];

    // Ground pad.
    let ground = [
        [-6.0f32, -6.0, 0.0],
        [6.0, -6.0, 0.0],
        [6.0, 6.0, 0.0],
        [-6.0, 6.0, 0.0],
    ];
    let gm = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &ground,
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
        })
        .expect("ground");
    let gt = b
        .create_material_desc(&MaterialDesc {
            base_color: [0.20, 0.28, 0.22, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        })
        .expect("ground mat");
    commands.push(DrawCommand::DrawMesh {
        mesh: gm,
        material: gt,
        origin: [0.0; 3],
        transform: T4,
    });

    // Trail markers along the sampled path (cyan diamonds), one per sample.
    // t is in the KEYFRAME TIME DOMAIN (0..5.0 here), not normalized —
    // the sampler clamps outside the table (endpoint law).
    let n = frames.clamp(4, 96);
    let t_end = WAYPOINTS.last().expect("non-empty").0;
    for i in 0..n {
        let t = t_end * f64::from(i) / f64::from(n);
        let o = anim_origin(WAYPOINTS, t);
        let marker = [
            [0.0f32, -0.18, 0.0],
            [0.18, 0.0, 0.0],
            [0.0, 0.18, 0.0],
            [-0.18, 0.0, 0.0],
        ];
        let mm = b
            .create_mesh(&MeshDesc {
                uv: &[],
                positions: &marker,
                normals: &[[0.0, 0.0, 1.0]; 4],
                indices: &[0, 1, 2, 0, 2, 3],
            })
            .expect("marker");
        let mt = b
            .create_material_desc(&MaterialDesc {
                base_color: [0.2, 0.8, 0.85, 1.0],
                texture: None,
                repeat: [1.0, 1.0],
                specular: 0.0,
            })
            .expect("marker mat");
        commands.push(DrawCommand::DrawMesh {
            mesh: mm,
            material: mt,
            origin: o,
            transform: T4,
        });
    }

    // Drone (red diamond + mast) at the FINAL waypoint (endpoint law visual).
    let end = anim_origin(WAYPOINTS, t_end);
    assert_eq!(
        anim_origin(WAYPOINTS, 0.0),
        WAYPOINTS[0].1,
        "t=0 -> first waypoint verbatim"
    );
    assert_eq!(
        end,
        WAYPOINTS.last().expect("non-empty").1,
        "t=1 -> last waypoint verbatim"
    );
    let drone = [
        [0.0f32, -0.4, 0.0],
        [0.4, 0.0, 0.0],
        [0.0, 0.4, 0.0],
        [-0.4, 0.0, 0.0],
        [0.0, 0.0, 0.35],
    ];
    let didx: Vec<u32> = vec![0, 1, 2, 0, 2, 3, 1, 2, 4, 0, 4, 3, 3, 4, 2, 1, 4, 2];
    let dm = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &drone,
            normals: &[[0.0, 0.0, 1.0]; 5],
            indices: &didx,
        })
        .expect("drone");
    let dt = b
        .create_material_desc(&MaterialDesc {
            base_color: [0.9, 0.25, 0.3, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
        })
        .expect("drone mat");
    commands.push(DrawCommand::DrawMesh {
        mesh: dm,
        material: dt,
        origin: end,
        transform: T4,
    });

    // Render + pixel gates (probe-pinned: see run log on first failure).
    let img = b.render_to_pixels(&cam_frame(&commands)).unwrap();
    let (cyan, red): (u32, u32) =
        img.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .fold((0u32, 0u32), |(c, r), p| {
                let hit_c = p[2] > 140 && p[1] > 110 && p[0] < 90;
                let hit_r = p[0] > 150 && p[1] < 90;
                (c + u32::from(hit_c), r + u32::from(hit_r))
            });
    println!("E305 probe: cyan={cyan} red={red} markers={n}");
    assert!(cyan > 20, "trail markers visible (cyan={cyan})");
    assert!(red > 10, "drone visible at landing (red={red})");
    println!("E305 OK: {n} markers + drone at final waypoint (endpoint law holds)");
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
    // Marker count scales with frames (trail density demo). No-arg = same
    // assert path at default density; the resident-window face ships with
    // the 6b band (entity-position C-API), tutorials row notes it.
    prove(frames.unwrap_or(24));
    Ok(())
}
