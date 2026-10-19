//! WGPU-12: the render face of selection highlight (entity material override = solid-color overlay, [E3D:C/A5] on the CPU side).
//! The pick chain is reused: screen ray (REND-21/22) -> pick_meshes (REND-23) -> highlight-color material.

use visiaengine_core::Scene;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport, screen_to_ray_ortho,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 256; // the readback row is 1024B = 4x256 aligned (COPY_BYTES_PER_ROW_ALIGNMENT)
const H: u32 = 256;

fn cube1() -> (Vec<[f32; 3]>, Vec<u32>) {
    let positions = vec![
        [-1., -1., -1.],
        [1., -1., -1.],
        [1., 1., -1.],
        [-1., 1., -1.],
        [-1., -1., 1.],
        [1., -1., 1.],
        [1., 1., 1.],
        [-1., 1., 1.],
    ];
    let idx: &[&[u32]] = &[
        &[0, 2, 1],
        &[0, 3, 2],
        &[4, 5, 6],
        &[4, 6, 7],
        &[0, 1, 5],
        &[0, 5, 4],
        &[2, 3, 7],
        &[2, 7, 6],
        &[1, 2, 6],
        &[1, 6, 5],
        &[3, 0, 4],
        &[3, 4, 7],
    ];
    (positions, idx.concat())
}

/// Pick candidates use the **full world matrix** (pick has no origin concept, the ray is also in world space).
fn world4(dz: f64) -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [4.0, 0.0, dz, 1.0],
    ]
}

/// Rendering uses a **pure local transform** (the z offset within the layer) -- world x = 4 belongs to origin (D7:
/// the transform must not bake in world-level translation, otherwise rebase fails to cancel -- this file stepped on it and locked the semantics into the matrix).
fn trz(dz: f64) -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, dz, 1.0],
    ]
}

// spec: WGPU-12
#[test]
fn selected_entity_renders_highlight_overlay() {
    let Some(mut backend) = HeadlessBackend::new(W, H) else {
        eprintln!("SKIP: no adapter");
        return;
    };
    let (pos, idx) = cube1();
    let mut scene = Scene::new();
    let near = scene.spawn();
    let _far = scene.spawn();
    let origin = [4.0, 0.0, 0.0];
    let neg = [-origin[0], -origin[1]];

    // the pick chain: center ray -> hit the near piece (the x = 4 column)
    let rig = CameraRig::look_at([4.0, 0.0, 20.0], [4.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let ray =
        screen_to_ray_ortho(&rig, W as f32 / 2.0, H as f32 / 2.0, W as f32, H as f32).unwrap();
    let cands = [visiaengine_render::MeshCandidate {
        entity: near,
        positions: &pos,
        indices: &idx,
        world: &world4(1.0),
        origin: [0.0; 3],
    }];
    let hit =
        visiaengine_render::pick_meshes(ray, &cands).expect("the center should hit the near piece");
    assert_eq!(hit.entity, near);

    // render: the piece on the viewpoint's near side (z = 5, first seen in top view) = highlight yellow; the lower layer = base blue. The selected state overrides the material.
    let yellow = [1.0, 0.83, 0.29, 1.0];
    let blue = [0.0, 0.45, 1.0, 1.0];
    let _ = neg;
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    for (dz, color) in [(5.0f64, yellow), (1.0, blue)] {
        // local vertices = shifted(-origin) (the D7 discipline, the same usage as geo_viewer)
        let local: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 0.6, p[1] * 0.6, p[2] * 0.6])
            .collect();
        let m = backend
            .create_mesh(&MeshDesc {
                positions: &local,
                normals: &vec![[0.0, 0.0, 1.0]; local.len()],
                indices: &idx,
                uv: &[],
            })
            .unwrap();
        let mat = backend.create_material(color).unwrap();
        commands.push(DrawCommand::DrawMesh {
            mesh: m,
            material: mat,
            origin,
            transform: trz(dz),
        });
    }
    let proj = rig
        .ortho_frame(8.0, W as f32, H as f32, 0.1, 100.0)
        .unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(8.0, 8.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [4.0, 0.0, 20.0],
        proj,
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    };
    let img = backend.render_to_pixels(&frame).expect("render");
    let i = ((H / 2 * W + W / 2) * 4) as usize;
    let (r, g, bl) = (
        img.rgba[i] as i16,
        img.rgba[i + 1] as i16,
        img.rgba[i + 2] as i16,
    );
    // highlight yellow x the directional-light empirical domain (~ 0.625): r >= g >> b (the blue channel suppressed)
    assert!(
        r > 120 && g > 90 && bl < 90 && r - bl > 60,
        "the center pixel should be in the highlight-yellow family, got ({r},{g},{bl})"
    );
}

// spec: WGPU-13
#[test]
fn occlusion_independent_of_draw_order() {
    // the same stacked scene with the draw order reversed (far blue first / near yellow second, and the converse) -> the center pixel always = the near-side yellow
    // (depth occlusion is order-irrelevant -- if the pipeline lacked depth, whoever drew last always wins = this assertion must go red)
    let yellow = [1.0, 0.83, 0.29, 1.0];
    let blue = [0.0, 0.45, 1.0, 1.0];
    let mut centers = Vec::new();
    for order in [0u8, 1] {
        let Some(mut backend) = HeadlessBackend::new(W, H) else {
            eprintln!("SKIP: no adapter");
            return;
        };
        let (pos, idx) = cube1();
        let origin = [4.0, 0.0, 0.0];
        let mut commands = vec![DrawCommand::ClearColor {
            rgba: [0.05, 0.07, 0.10, 1.0],
        }];
        let slots = match order {
            0 => [(1.0f64, blue), (5.0, yellow)],
            _ => [(5.0f64, yellow), (1.0, blue)],
        };
        for (dz, color) in slots {
            let m = backend
                .create_mesh(&MeshDesc {
                    positions: &pos,
                    normals: &vec![[0.0, 0.0, 1.0]; pos.len()],
                    indices: &idx,
                    uv: &[],
                })
                .unwrap();
            let mat = backend.create_material(color).unwrap();
            commands.push(DrawCommand::DrawMesh {
                mesh: m,
                material: mat,
                origin,
                transform: trz(dz),
            });
        }
        let rig = CameraRig::look_at([4.0, 0.0, 20.0], [4.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let proj = rig
            .ortho_frame(8.0, W as f32, H as f32, 0.1, 100.0)
            .unwrap();
        let frame = Frame {
            viewport: Viewport::new(W, H, 1.0),
            camera: Camera::ortho(8.0, 8.0, 0.1, 100.0),
            view_rot: rig.view_rotation(),
            eye: [4.0, 0.0, 20.0],
            proj,
            px_world_scale: 1.0,
            shadow: None,
            clip: None,
            edl: None,
            post: Vec::new(),
            commands,
        };
        let img = backend.render_to_pixels(&frame).expect("render");
        let i = ((H / 2 * W + W / 2) * 4) as usize;
        centers.push((img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]));
    }
    for (idx, c) in centers.iter().enumerate() {
        assert!(
            c.0 > 120 && c.1 > 90 && c.2 < 90,
            "draw order {idx}: the center should = the near-side yellow, got {c:?}"
        );
    }
}

// spec: WGPU-12
/// The E402 window-path mirror pixel gate (carried by the 2026-09-17 hover/multi-select window; testing.md visual-case double insurance clause 1).
/// rig / placement / palette mirrored verbatim from examples/rs/E402_pick_interactive.rs -- the window and the gate share math.
/// Predicate triple self-proof: (1) unselected center = base color family; (2) selected center = highlight-yellow family (the WGPU-12 domain); (3) an outer-corner ray must not hit
/// (canary -- when the ray/placement solves wrong, the accidental greens of "hits everything" or "hits nothing" are nailed dead by this assertion).
#[test]
fn golden_pick_window_mirror() {
    const PW: u32 = 512; // the 16:10 window-form mirror (E402's 960x600 same ratio; 512 = a row-alignment multiple)
    const PH: u32 = 320;
    let mut scene = Scene::new();
    let ents: Vec<_> = (0..3).map(|_| scene.spawn()).collect();
    let (pos, nrm, idx) = visiaengine_render_wgpu::unit_box_mesh();
    let placed = |x: f64, y: f64, z: f64, s: f64| {
        [
            [s, 0.0, 0.0, 0.0],
            [0.0, s, 0.0, 0.0],
            [0.0, 0.0, s, 0.0],
            [x, y, z, 1.0],
        ]
    };
    let worlds = [
        placed(-3.2, 0.0, 1.1, 2.2),
        placed(0.0, 0.6, 1.9, 2.2),
        placed(3.2, -0.4, 1.4, 2.2),
    ];
    let palette = [
        [0.90f32, 0.25, 0.25, 1.0],
        [0.20, 0.75, 0.35, 1.0],
        [0.20, 0.45, 0.90, 1.0],
    ];
    let yellow = [1.0f32, 0.83, 0.29, 1.0];
    // E402 main()'s camera position verbatim (dist = 11, fov_y = 1.0 rad)
    let rig = CameraRig::orbit([0.0, 0.0, 1.5], 0.35, 0.95, 11.0, 1.0, 1.0, 0.1, 100.0);
    let Some(mut backend) = HeadlessBackend::new(PW, PH) else {
        eprintln!("SKIP: no adapter");
        return;
    };
    let mut mats = Vec::new();
    for c in palette.iter().chain(std::iter::once(&yellow)) {
        mats.push(backend.create_material(*c).unwrap());
    }
    let mesh = backend
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .unwrap();
    let render = |be: &mut HeadlessBackend, sel: Option<usize>| {
        let mut commands = vec![DrawCommand::ClearColor {
            rgba: [0.05, 0.07, 0.10, 1.0],
        }];
        for i in 0..3 {
            let material = if sel == Some(i) { mats[3] } else { mats[i] };
            commands.push(DrawCommand::DrawMesh {
                mesh,
                material,
                origin: [0.0, 0.0, 0.0],
                transform: worlds[i],
            });
        }
        let near = (rig.dist * 0.01) as f32;
        let far = (rig.dist * 30.0) as f32;
        let frame = Frame {
            viewport: Viewport::new(PW, PH, 1.0),
            camera: Camera::perspective(rig.fov_y as f32, PW as f32 / PH as f32, near, far),
            view_rot: rig.view_rotation(),
            eye: rig.eye(),
            proj: rig
                .perspective(rig.fov_y as f32, PW as f32 / PH as f32, near, far)
                .unwrap(),
            px_world_scale: 1.0,
            shadow: None,
            clip: None,
            edl: None,
            post: Vec::new(),
            commands,
        };
        be.render_to_pixels(&frame).expect("render")
    };
    let center = |img: &visiaengine_render_wgpu::OffscreenFrame| {
        let i = ((PH / 2 * PW + PW / 2) as usize) * 4;
        [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]]
    };

    // (1) the box the center ray hits (E402's point-select chain same shape)
    let ray = visiaengine_render::screen_to_ray_persp(
        &rig,
        PW as f32 / 2.0,
        PH as f32 / 2.0,
        PW as f32,
        PH as f32,
    )
    .expect("center ray");
    let cands: Vec<_> = ents
        .iter()
        .zip(&worlds)
        .map(|(e, w)| visiaengine_render::MeshCandidate {
            entity: *e,
            positions: &pos,
            indices: &idx,
            world: w,
            origin: [0.0; 3],
        })
        .collect();
    let hit = visiaengine_render::pick_meshes(ray, &cands).expect("the center should hit one box");
    let hit_i = ents.iter().position(|e| *e == hit.entity).unwrap();
    println!("PROBE center hits box #{hit_i}");

    // (2) unselected center = that box's base-color family (green family g dominant for #1, red family r dominant for the #0/#2 side boxes)
    let img0 = render(&mut backend, None);
    let [r0, g0, b0] = center(&img0);
    println!("PROBE unselected center=({r0},{g0},{b0})");
    let base = palette[hit_i];
    let dom = if base[0] > base[1] && base[0] > base[2] {
        0
    } else if base[1] > base[2] {
        1
    } else {
        2
    };
    let ch = [r0, g0, b0];
    assert!(
        ch[dom] > 60 && ch[dom] > ch[(dom + 1) % 3] && ch[dom] > ch[(dom + 2) % 3],
        "the unselected center should be box #{hit_i}'s base-color family, got ({r0},{g0},{b0})"
    );

    // (3) selected center = the highlight-yellow family (the predicate inherits the WGPU-12 proven domain)
    let img1 = render(&mut backend, Some(hit_i));
    let [r1, g1, b1] = center(&img1);
    println!("PROBE selected center=({r1},{g1},{b1})");
    assert!(
        r1 > 120 && g1 > 90 && b1 < 90 && r1 - b1 > 60,
        "the selected center should be in the highlight-yellow family, got ({r1},{g1},{b1})"
    );

    // (4) canary: the outer top-right corner ray must miss (nails dead the accidental "hits everything")
    let ray_corner =
        visiaengine_render::screen_to_ray_persp(&rig, PW as f32 - 2.0, 2.0, PW as f32, PH as f32)
            .expect("corner ray");
    assert!(
        visiaengine_render::pick_meshes(ray_corner, &cands).is_none(),
        "the outer corner ray unexpectedly hit -- the ray domain is out of control"
    );
}
