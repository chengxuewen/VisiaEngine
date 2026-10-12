//! WGPU-16: the Instanced pipeline -- storage instance table, instance_index addressing,
//! `pos.z*h + offset` bottom-aligned extrusion, the base*inst.color*shade multiply chain.
//! Predicate discipline [PIT-8]: region-family statistics + column-height contrast, no single-pixel locks; a broken chain (let-else silent skip)
//! would make the three-family counts always 0 -- the discriminative power is retained.

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MeshDesc, RenderBackend,
    Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 64;
const H: u32 = 64;

use visiaengine_render_wgpu::unit_box_mesh;

fn px(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [
        img.rgba[i],
        img.rgba[i + 1],
        img.rgba[i + 2],
        img.rgba[i + 3],
    ]
}

/// Three buildings: x = -2/0/+2, heights 1/2/3, colors red/green/blue. Returns the rendered frame.
fn city() -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (pos, nrm, idx) = unit_box_mesh();
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .unwrap();
    let mat = b.create_material([1.0, 1.0, 1.0, 1.0]).unwrap();
    let table = vec![
        Instance::new([-2.0, 0.0, 0.0], 1.0, [1.0, 0.0, 0.0]),
        Instance::new([0.0, 0.0, 0.0], 2.0, [0.0, 1.0, 0.0]),
        Instance::new([2.0, 0.0, 0.0], 3.0, [0.0, 0.0, 1.0]),
    ];
    let iid = b
        .create_instances(&InstanceDesc { data: &table })
        .expect("instances");
    let rig = CameraRig::look_at([0.0, -8.0, 4.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
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
                rgba: [0.02, 0.02, 0.02, 1.0],
            },
            DrawCommand::DrawInstances {
                mesh,
                material: mat,
                instances: iid,
                origin: [0.0; 3],
                transform: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        ],
    };
    b.render_to_pixels(&frame).expect("render")
}

// spec: WGPU-16
#[test]
fn three_instances_render_three_color_families() {
    let img = city();
    let (mut r, mut g, mut bl) = (0u32, 0u32, 0u32);
    for y in 0..H {
        for x in 0..W {
            let p = px(&img, x, y);
            if p[0] > 60 && p[1] < 20 && p[2] < 20 {
                r += 1;
            } else if p[1] > 60 && p[0] < 20 && p[2] < 20 {
                g += 1;
            } else if p[2] > 60 && p[0] < 20 && p[1] < 20 {
                bl += 1;
            }
        }
    }
    assert!(
        r > 20 && g > 20 && bl > 20,
        "the three color families missing r={r} g={g} b={bl} (silent skip = all 0)"
    );
}

// spec: WGPU-16
#[test]
fn instance_height_extrudes_column_extent() {
    // bottom-aligned extrusion semantics: three columns of the same width, the colored column heights b > g > r (direct projection evidence of heights 3/2/1).
    let img = city();
    let col_h = |fam: u8, x0: u32, x1: u32| -> u32 {
        let mut mx = 0u32;
        for x in x0..x1 {
            let c = (0..H)
                .filter(|&y| {
                    let p = px(&img, x, y);
                    match fam {
                        0 => p[0] > 60 && p[1] < 20 && p[2] < 20,
                        1 => p[1] > 60 && p[0] < 20 && p[2] < 20,
                        _ => p[2] > 60 && p[0] < 20 && p[1] < 20,
                    }
                })
                .count() as u32;
            mx = mx.max(c);
        }
        mx
    };
    let (hr, hg, hb) = (col_h(0, 8, 26), col_h(1, 24, 40), col_h(2, 38, 56));
    assert!(
        hr > 4 && hg > hr && hb > hg,
        "column-height order broken r={hr} g={hg} b={hb}"
    );
}

// spec: WGPU-16
#[test]
fn empty_instance_table_rejected_on_create() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    assert!(
        b.create_instances(&InstanceDesc { data: &[] }).is_err(),
        "empty table rejected at build (the REND-28 closed ruling)"
    );
}

// spec: WGPU-16
#[test]
fn missing_instance_table_skips_not_panics() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (pos, nrm, idx) = unit_box_mesh();
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .unwrap();
    let mat = b.create_material([1.0, 1.0, 1.0, 1.0]).unwrap();
    let rig = CameraRig::look_at([0.0, -8.0, 4.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
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
                rgba: [0.02, 0.02, 0.02, 1.0],
            },
            DrawCommand::DrawInstances {
                mesh,
                material: mat,
                instances: 9999, // missing table
                origin: [0.0; 3],
                transform: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        ],
    };
    let img = b
        .render_to_pixels(&frame)
        .expect("a missing table must not panic");
    let p = px(&img, 32, 32);
    assert!(
        p[0] < 10 && p[1] < 10 && p[2] < 10,
        "missing table = skip, not undefined {p:?}"
    );
}
