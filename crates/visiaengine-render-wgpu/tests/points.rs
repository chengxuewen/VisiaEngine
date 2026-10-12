//! WGPU-18: the Points splat pipeline -- screen-space expansion + FS circle mask (true round dots, correcting the geo square pre-existing state).
//! The camera takes a 76-degree depression (sin ~ 0.97) so the circle evidence is not squashed by perspective [PIT-8 calibration].

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, PointMark, PointTableDesc, RenderBackend, TableId,
    Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 64;
const H: u32 = 64;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
/// eye [0,-2,8]: the target plane ~ 0.15 world/px (half-height ~ 4.8/64 -- the premise of the ladder's unit system)
const PX_SCALE: f32 = 0.15;

fn frame_with(commands: Vec<DrawCommand>) -> Frame {
    let rig = CameraRig::look_at([0.0, -2.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: PX_SCALE,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    }
}

fn upload(b: &mut HeadlessBackend, marks: &[PointMark]) -> TableId {
    b.create_points(&PointTableDesc { data: marks })
        .expect("points")
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

fn count(
    img: &visiaengine_render_wgpu::OffscreenFrame,
    x0: u32,
    x1: u32,
    y0: u32,
    y1: u32,
    fam: impl Fn([u8; 4]) -> bool,
) -> u32 {
    let mut n = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            if fam(px(img, x, y)) {
                n += 1;
            }
        }
    }
    n
}

fn mark_frame(table: TableId) -> Frame {
    frame_with(vec![
        DrawCommand::ClearColor {
            rgba: [0.0, 0.0, 0.0, 1.0],
        },
        DrawCommand::DrawPoints {
            table,
            origin: [0.0; 3],
            transform: T4,
        },
    ])
}

// spec: WGPU-18
#[test]
fn single_mark_is_round_not_square() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let t = upload(
        &mut b,
        &[PointMark::new([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 8.0)],
    );
    let img = b.render_to_pixels(&mark_frame(t)).expect("render");
    let g = |p: [u8; 4]| p[1] > 200 && p[0] < 40 && p[2] < 40;
    let (cx, cy) = (32i32, 32i32);
    let at = |dx: i32, dy: i32| g(px(&img, (cx + dx) as u32, (cy + dy) as u32));
    // the probe calibrates the center first (tightened after probing): center solid
    assert!(at(0, 0), "center missing color");
    // circle evidence: axial near edges (±6,0) carry color; diagonals (±6.5,±6.5) world distance > r -> discarded patch
    assert!(
        at(6, 0) || at(0, 6),
        "axial near edges missing color (radius not in effect)"
    );
    assert!(
        !at(7, 7) && !at(-7, 7) && !at(7, -7) && !at(-7, -7),
        "diagonal covered = square patch, not circle (mask absent)"
    );
}

// spec: WGPU-18
#[test]
fn radius_ladder_scales_area() {
    let small = {
        let mut b = HeadlessBackend::new(W, H).expect("adapter");
        let t = upload(
            &mut b,
            &[PointMark::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 2.0)],
        );
        b.render_to_pixels(&mark_frame(t)).expect("small")
    };
    let big = {
        let mut b = HeadlessBackend::new(W, H).expect("adapter");
        let t = upload(
            &mut b,
            &[PointMark::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], 10.0)],
        );
        b.render_to_pixels(&mark_frame(t)).expect("big")
    };
    let w = |p: [u8; 4]| p[0] > 200 && p[1] > 200 && p[2] > 200;
    let (s, l) = (count(&small, 0, W, 0, H, w), count(&big, 0, W, 0, H, w));
    assert!(s >= 5, "small dot absent {s}");
    // area ratio (10/2)^2 = 25, perspective/ellipse micro-loss -> >= 8 conservative
    assert!(l > s * 8, "radius ladder distorted {s}->{l}");
}

// spec: WGPU-18
#[test]
fn three_marks_families_with_positions() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let t = upload(
        &mut b,
        &[
            PointMark::new([-2.2, 0.0, 0.0], [1.0, 0.0, 0.0], 6.0),
            PointMark::new([2.2, 0.0, 0.0], [0.0, 0.0, 1.0], 6.0),
            PointMark::new([0.0, 2.2, 0.0], [0.0, 1.0, 0.0], 6.0),
        ],
    );
    let img = b.render_to_pixels(&mark_frame(t)).expect("render");
    let r = |p: [u8; 4]| p[0] > 200 && p[1] < 40 && p[2] < 40;
    let bl = |p: [u8; 4]| p[2] > 200 && p[0] < 40 && p[1] < 40;
    let g = |p: [u8; 4]| p[1] > 200 && p[0] < 40 && p[2] < 40;
    // right/up project to world +y moving upward (rows decrease) -- a wrong sign mirrors vertically
    assert!(
        count(&img, 0, 24, 20, 46, r) >= 12,
        "the red left window is absent"
    );
    assert!(
        count(&img, 40, W, 20, 46, bl) >= 12,
        "the blue right window is absent"
    );
    assert!(
        count(&img, 20, 44, 0, 30, g) >= 12,
        "the green top window is absent"
    );
}

// spec: WGPU-18
#[test]
fn empty_and_missing_tables_semantics() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    assert!(b.create_points(&PointTableDesc { data: &[] }).is_err());
    // missing table = skip, not panic
    let f = frame_with(vec![
        DrawCommand::ClearColor {
            rgba: [0.02, 0.02, 0.02, 1.0],
        },
        DrawCommand::DrawPoints {
            table: 777,
            origin: [0.0; 3],
            transform: T4,
        },
    ]);
    let img = b
        .render_to_pixels(&f)
        .expect("a missing table must not panic");
    assert!(px(&img, 32, 32)[0] < 10, "the missing table drew something");
}

/// The G1 large-batch integrity gate (the point-cloud band E/G slices, WGPU-18 same-clause multiple instances): a 100k-point table ->
/// splat coverage lower bound + each of the four edge bands carries pixels (sealing batch truncation / stride mistakes -- the same family as WGPU-16's silent skip).
// spec: WGPU-18
#[test]
fn golden_batch_integrity_100k() {
    const S: u32 = 512;
    let Some(mut b) = HeadlessBackend::new(S, S) else {
        eprintln!("SKIP: no adapter");
        return;
    };
    let side = 317u32; // 317^2 ~ 100489 >= the 100k table magnitude
    let marks: Vec<visiaengine_render::PointMark> = (0..side)
        .flat_map(|gy| (0..side).map(move |gx| (gx, gy)))
        .map(|(gx, gy)| {
            visiaengine_render::PointMark::new(
                [
                    gx as f32 / side as f32 * 8.84 - 4.42,
                    gy as f32 / side as f32 * 8.84 - 4.42,
                    0.0,
                ],
                [0.95, 0.95, 0.95],
                4.0,
            )
        })
        .collect();
    assert!(
        marks.len() >= 100_000,
        "the table-magnitude threshold {}",
        marks.len()
    );
    let table = b
        .create_points(&visiaengine_render::PointTableDesc { data: &marks })
        .unwrap();
    let rig = visiaengine_render::CameraRig::look_at([0., 0., 12.], [0., 0., 0.], [0., 1., 0.]);
    let frame = visiaengine_render::Frame {
        viewport: visiaengine_render::Viewport::new(S, S, 1.0),
        camera: visiaengine_render::Camera::ortho(4.5, 4.5, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [0., 0., 12.],
        proj: rig
            .ortho_frame(4.5, S as f32, S as f32, 0.1, 100.0)
            .unwrap(),
        px_world_scale: 2.0 * 4.5 / S as f32,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: vec![
            visiaengine_render::DrawCommand::ClearColor {
                rgba: [0.05, 0.07, 0.10, 1.0],
            },
            visiaengine_render::DrawCommand::DrawPoints {
                table,
                origin: [0.; 3],
                transform: T4,
            },
        ],
    };
    let img = b.render_to_pixels(&frame).unwrap();
    let nb = |x0: u32, x1: u32, y0: u32, y1: u32| -> u32 {
        (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (y, x)))
            .filter(|&(y, x)| {
                let o = ((y * S + x) * 4) as usize;
                let p = &img.rgba[o..o + 3];
                p[0] > 90 || p[1] > 90 || p[2] > 90
            })
            .count() as u32
    };
    let total = nb(0, S, 0, S);
    // domain values = measured minus 40% discipline (PIT-8: run first, pin after): each of the four bands > 500 with total coverage > 60_000
    let bands = [
        nb(0, S, 0, 16),
        nb(0, S, S - 16, S),
        nb(0, 16, 0, S),
        nb(S - 16, S, 0, S),
    ];
    assert!(
        total > 60_000,
        "the 100k batch total coverage collapsed: {total}"
    );
    assert!(
        bands.iter().all(|x| *x > 500),
        "edge band missing (batch-truncation form): {bands:?}"
    );
}

/// G2 far-origin pixel reciprocity, point edition (the WGPU-10 form, same-clause multiple instances): the same local table origin = (1e7,0,0)
/// <-> origin = 0 agrees byte by byte -- the specialized catcher for the point-edition recurrence bed of the E202 gray-screen case.
// spec: WGPU-10
#[test]
fn golden_far_origin_pixels_roundtrip_points() {
    const S: u32 = 128;
    let Some(mut b) = HeadlessBackend::new(S, S) else {
        eprintln!("SKIP: no adapter");
        return;
    };
    let marks: Vec<visiaengine_render::PointMark> = (0..40u32)
        .map(|i| {
            visiaengine_render::PointMark::new(
                [i as f32 * 0.08 - 1.5, ((i % 7) as f32) * 0.3 - 1.0, 0.0],
                [1.0, 0.4, 0.2],
                6.0,
            )
        })
        .collect();
    let table = b
        .create_points(&visiaengine_render::PointTableDesc { data: &marks })
        .unwrap();
    let shoot = |origin: [f64; 3], b: &mut HeadlessBackend| -> Vec<u8> {
        let eye = [origin[0], origin[1], origin[2] + 12.0];
        let rig = visiaengine_render::CameraRig::look_at(eye, origin, [0., 1., 0.]);
        let frame = visiaengine_render::Frame {
            viewport: visiaengine_render::Viewport::new(S, S, 1.0),
            camera: visiaengine_render::Camera::ortho(3.0, 3.0, 0.1, 100.0),
            view_rot: rig.view_rotation(),
            eye,
            proj: rig
                .ortho_frame(3.0, S as f32, S as f32, 0.1, 100.0)
                .unwrap(),
            px_world_scale: 2.0 * 3.0 / S as f32,
            shadow: None,
            clip: None,
            edl: None,
            post: Vec::new(),
            commands: vec![
                visiaengine_render::DrawCommand::ClearColor {
                    rgba: [0.05, 0.07, 0.10, 1.0],
                },
                visiaengine_render::DrawCommand::DrawPoints {
                    table,
                    origin,
                    transform: T4,
                },
            ],
        };
        b.render_to_pixels(&frame).unwrap().rgba
    };
    let a = shoot([0.0; 3], &mut b);
    let f = shoot([1e7, 0.0, 0.0], &mut b);
    assert_eq!(
        a, f,
        "far-origin reciprocity pixel fork (the D7 point-face regression mine)"
    );
}
