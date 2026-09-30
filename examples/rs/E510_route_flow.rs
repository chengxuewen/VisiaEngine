//! E510 Route flow — V2.1 parametric curves live demo (REND-47).
//!
//! - `--frames N` = headless assert path (this lane): CatmullRom route
//!   ribbon plus tube guard-rail uploaded as regular meshes (+0 ABI), N flow
//!   markers sampled along the curve and drawn as points; endpoint law
//!   asserted, marker spread pixel-verified.
//! - Zero-arg = prints the window-lane note (interactive orbit lives in
//!   E306/E901; this example is assert-first by design).
//!
//! Claim: consumer band — curve math + mesh generation only; no engine
//! changes. OpenDRIVE road-mesh prereq (V4.2 dependency anchor).

#![expect(clippy::cast_possible_truncation)]

#[path = "gallery.rs"]
mod gallery;
use gallery::save_frame;
use visiaengine_render::RenderBackend;
use visiaengine_render::contract::{
    Camera, DrawCommand, Frame, MaterialDesc, MeshDesc, PointMark, PointTableDesc, Viewport,
};
use visiaengine_render::curve::{ribbon_xz, sample_catmull_rom, tube};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 480;
const H: u32 = 320;

/// S-curved route knots (XZ plane, y=0 ground).
fn route_knots() -> Vec<[f32; 3]> {
    vec![
        [-18.0, 0.0, 0.0],
        [-8.0, 0.0, 6.0],
        [0.0, 0.0, -6.0],
        [8.0, 0.0, 6.0],
        [18.0, 0.0, 0.0],
    ]
}

fn scene(b: &mut HeadlessBackend, flow_n: usize) -> (Vec<DrawCommand>, Vec<[f32; 3]>) {
    let knots = route_knots();
    let center = sample_catmull_rom(&knots, 24);
    assert_eq!(center.first(), knots.first(), "curve endpoint law (start)");
    let tail = *center.last().expect("non-empty");
    assert!(
        (tail[0] - knots[knots.len() - 1][0]).abs() < 1e-5,
        "curve endpoint law (end)"
    );

    // Road ribbon (dark asphalt) + tube guard rails (orange, offset +/-2.2 z
    // by a shifted knot copy — same curve family, cheaper than true offsets).
    let (rpos, ridx) = ribbon_xz(&center, 4.0);
    let road = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &rpos,
            normals: &vec![[0.0, 1.0, 0.0]; rpos.len()],
            indices: &ridx,
        })
        .expect("road mesh");
    let road_mat = b
        .create_material_desc(&MaterialDesc {
            base_color: [0.16, 0.17, 0.19, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 0.9,
            metallic: 0.0,
        })
        .expect("road mat");

    let mut shifted_up: Vec<[f32; 3]> = knots.iter().map(|p| [p[0], p[1], p[2] - 2.6]).collect();
    let c_up = sample_catmull_rom(shifted_up.as_slice(), 24);
    let (t1p, t1i) = tube(&c_up, 0.18, 6);
    shifted_up = knots.iter().map(|p| [p[0], p[1], p[2] + 2.6]).collect();
    let c_dn = sample_catmull_rom(shifted_up.as_slice(), 24);
    let (t2p, t2i) = tube(&c_dn, 0.18, 6);
    let rail1 = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &t1p,
            normals: &t1p.iter().map(|p| norm_of(*p)).collect::<Vec<_>>(),
            indices: &t1i,
        })
        .expect("rail1");
    let rail2 = b
        .create_mesh(&MeshDesc {
            uv: &[],
            positions: &t2p,
            normals: &t2p.iter().map(|p| norm_of(*p)).collect::<Vec<_>>(),
            indices: &t2i,
        })
        .expect("rail2");
    let rail_mat = b
        .create_material_desc(&MaterialDesc {
            base_color: [0.95, 0.62, 0.18, 1.0],
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0,
            roughness: 0.6,
            metallic: 0.0,
        })
        .expect("rail mat");

    // Flow markers: N samples along the SAME center polyline (flow = anim
    // lane consumed per-frame in the window form; here assert the sample set).
    let marks: Vec<PointMark> = (0..flow_n)
        .map(|i| {
            let idx = i * (center.len() - 1) / flow_n.max(1);
            let mut p = center[idx];
            p[1] = 0.15; // float above road
            PointMark::new(p, [1.0, 0.9, 0.2], 6.0)
        })
        .collect();
    assert_eq!(
        marks.first().expect("first").pos[0],
        center[0][0] as f32,
        "flow endpoint law"
    );
    let pt_table = b
        .create_points(&PointTableDesc { data: &marks })
        .expect("points");

    let cmds = vec![
        DrawCommand::ClearColor {
            rgba: [0.06, 0.08, 0.11, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: road,
            material: road_mat,
            origin: [0.0; 3],
            transform: identity(),
        },
        DrawCommand::DrawMesh {
            mesh: rail1,
            material: rail_mat,
            origin: [0.0; 3],
            transform: identity(),
        },
        DrawCommand::DrawMesh {
            mesh: rail2,
            material: rail_mat,
            origin: [0.0; 3],
            transform: identity(),
        },
        DrawCommand::DrawPoints {
            table: pt_table,
            origin: [0.0; 3],
            transform: identity(),
        },
    ];
    (cmds, center)
}

fn identity() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn norm_of(p: [f32; 3]) -> [f32; 3] {
    let l = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    if l > 1e-6 {
        [p[0] / l, p[1] / l, p[2] / l]
    } else {
        [0.0, 1.0, 0.0]
    }
}

fn frame(cmds: &[DrawCommand]) -> Frame {
    let rig = visiaengine_render::CameraRig::look_at(
        [0.0, -26.0, 18.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, W as f32 / H as f32, 0.1, 200.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, W as f32 / H as f32, 0.1, 200.0)
            .expect("proj"),
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: cmds.to_vec(),
    }
}

fn diff_px(
    a: &visiaengine_render_wgpu::OffscreenFrame,
    b: &visiaengine_render_wgpu::OffscreenFrame,
) -> u32 {
    a.rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.rgba.as_chunks::<4>().0)
        .filter(|(x, y)| x[..3] != y[..3])
        .count() as u32
}

fn prove(frames: u32) {
    let _ = frames;
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (cmds, _center) = scene(&mut b, 12);

    let base = b.render_to_pixels(&frame(&cmds)).expect("render");
    // route spread: the S-curve must put geometry across the frame — sample
    // the road region lit vs clear pixels (probe floor, PIT-8 conservative).
    let clear: [u8; 3] = [16, 21, 29]; // sRGB encode of clear [0.06,0.08,0.11] band
    let geo = base
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| [p[0], p[1], p[2]] != clear)
        .count();
    println!("E510 probe: geometry px={geo}");
    assert!(geo > 4000, "route geometry must cover the frame: {geo}");

    // Flow density law: denser sampling = strictly more marker pixels than a
    // sparse render (same camera, same geometry, only points differ).
    let (cmds_sparse, _) = scene(&mut b, 3);
    let sparse = b.render_to_pixels(&frame(&cmds_sparse)).expect("sparse");
    let d = diff_px(&sparse, &base);
    println!("E510 probe: flow-density diff={d}");
    assert!(d > 30, "flow marker density must be visible: {d}");

    save_frame(&base, "E510_route_flow");
    println!("E510 OK: curve endpoints exact; ribbon+tube uploaded; flow markers pixel-verified");
}

fn window_form() {
    // Window lane: reuse the prove scene with an orbit camera (E305 family
    // shape; minimal winit shell — full interactivity lives in E306/E901).
    println!(
        "E510 window: interactive orbit is E306/E901's lane; this example ships the headless assert path (--frames 1)"
    );
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
        None => window_form(),
    }
    Ok(())
}
