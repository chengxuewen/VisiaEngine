//! WGPU-17: the Strokes tile-expansion pipeline -- color/width live in the table, px semantics, cover-surface polygon-offset, degenerate fallback.
//! Predicate discipline [PIT-8]: windows + region-family statistics + positional dominance (the right/up basis-vector correctness lock), no single-pixel locks.

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, StrokeSeg, StrokeTableDesc,
    TableId, Viewport,
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
/// eye [0,-8,4]: the target plane ~ 0.16 world/px (half-height 5.16/64 -- the unit premise of the width ladder)
const PX_SCALE: f32 = 0.16;

fn frame_with(commands: Vec<DrawCommand>) -> Frame {
    let rig = CameraRig::look_at([0.0, -8.0, 4.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
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

fn stroke_frame(table: TableId) -> Frame {
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.0, 0.0, 0.0, 1.0],
    }];
    commands.push(DrawCommand::DrawStrokes {
        table,
        origin: [0.0; 3],
        transform: T4,
    });
    frame_with(commands)
}

fn upload(b: &mut HeadlessBackend, segs: &[StrokeSeg]) -> TableId {
    b.create_strokes(&StrokeTableDesc { data: segs })
        .expect("strokes")
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

/// Family count inside a window.
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

// spec: WGPU-17
#[test]
fn three_segments_color_families_with_positions() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tid = upload(
        &mut b,
        &[
            // the red horizontal line (lower)
            StrokeSeg::new([-2.5, -1.6, 0.0], [2.5, -1.6, 0.0], [1.0, 0.0, 0.0], 4.0),
            // the green vertical line (upper-left)
            StrokeSeg::new([-2.2, 0.2, 0.0], [-2.2, 2.6, 0.0], [0.0, 1.0, 0.0], 4.0),
            // the blue diagonal (upper-right)
            StrokeSeg::new([0.5, 0.2, 0.0], [2.4, 2.6, 0.0], [0.0, 0.0, 1.0], 4.0),
        ],
    );
    let img = b.render_to_pixels(&stroke_frame(tid)).expect("render");
    let red = |p: [u8; 4]| p[0] > 200 && p[1] < 40 && p[2] < 40;
    let green = |p: [u8; 4]| p[1] > 200 && p[0] < 40 && p[2] < 40;
    let blue = |p: [u8; 4]| p[2] > 200 && p[0] < 40 && p[1] < 40;
    // positional dominance = the right/up basis correctness lock (a wrong sign mirrors -> the window assertion flips)
    // window bounds = probe_map measured (under perspective the world y = -1.6 projects at rows 35-39, not the bottom band)
    assert!(
        count(&img, 0, W, 33, 44, red) >= 30,
        "the red horizontal line is absent from the lower window"
    );
    assert!(
        count(&img, 0, 32, 0, 44, green) >= 10,
        "the green vertical line is absent from the upper-left window"
    );
    assert!(
        count(&img, 32, W, 0, 44, blue) >= 10,
        "the blue diagonal is absent from the upper-right window"
    );
    // canvas-pure family (no mixed-color leakage into the background)
    assert_eq!(
        count(&img, 0, 8, 0, 8, |p| { p[0] + p[1] + p[2] > 30 }),
        0,
        "the background corner was polluted"
    );
}

// spec: WGPU-17
#[test]
fn width_px_ladder_scales_with_table() {
    let thin = {
        let mut b = HeadlessBackend::new(W, H).expect("adapter");
        let tid = upload(
            &mut b,
            &[StrokeSeg::new(
                [-2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
                2.0,
            )],
        );
        b.render_to_pixels(&stroke_frame(tid)).expect("thin")
    };
    let thick = {
        let mut b = HeadlessBackend::new(W, H).expect("adapter");
        let tid = upload(
            &mut b,
            &[StrokeSeg::new(
                [-2.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
                8.0,
            )],
        );
        b.render_to_pixels(&stroke_frame(tid)).expect("thick")
    };
    let white = |p: [u8; 4]| p[0] > 200 && p[1] > 200 && p[2] > 200;
    let (c1, c2) = (
        count(&thin, 0, W, 0, H, white),
        count(&thick, 0, W, 0, H, white),
    );
    assert!(c1 > 10 && c2 > 40, "family counts {c1}/{c2}");
    assert!(
        c2 * 10 > c1 * 25,
        "the width ladder is distorted {c1}->{c2}"
    );
}

// spec: WGPU-17
#[test]
fn stroke_over_coplanar_fill_wins_via_offset() {
    // same z = 0: the fill draws first (gray), the stroke draws after and rides on top -- with no polygon-offset the depths are equal and
    // strict Less swallows the stroke (this clause is exactly the [E3D:B7-3] regression lock).
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let quad = MeshDesc {
        positions: &[
            [-3.0, -3.0, 0.0],
            [3.0, -3.0, 0.0],
            [3.0, 3.0, 0.0],
            [-3.0, 3.0, 0.0],
        ],
        normals: &[[0.0, 0.0, 1.0]; 4],
        indices: &[0u32, 1, 2, 0, 2, 3],
        uv: &[],
    };
    let mesh = b.create_mesh(&quad).unwrap();
    let mat = b.create_material([0.8, 0.8, 0.8, 1.0]).unwrap();
    let tid = upload(
        &mut b,
        &[StrokeSeg::new(
            [-2.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            5.0,
        )],
    );
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.0, 0.0, 0.0, 1.0],
    }];
    commands.push(DrawCommand::DrawMesh {
        mesh,
        material: mat,
        origin: [0.0; 3],
        transform: T4,
    });
    commands.push(DrawCommand::DrawStrokes {
        table: tid,
        origin: [0.0; 3],
        transform: T4,
    });
    let img = b.render_to_pixels(&frame_with(commands)).expect("render");
    let dark = |p: [u8; 4]| p[0] < 60 && p[1] < 60 && p[2] < 60;
    let gray = |p: [u8; 4]| p[0] > 80 && p[0] < 180;
    let (d, g) = (
        count(&img, 16, 48, 24, 40, dark),
        count(&img, 16, 48, 24, 40, gray),
    );
    assert!(g > 100, "fill absent (gray band {g})");
    assert!(
        d >= 20,
        "the stroke was swallowed by the same-depth fill (dark pixels {d})"
    );
}

// spec: WGPU-17
#[test]
fn view_parallel_segment_falls_back_no_nan() {
    // The true meaning of R2 degeneracy: the segment axis parallel to the view direction -> cross = 0 -> normalize(0) = NaN. That segment's projection
    // degenerates to a point anyway (0 pixels is legal); the key is that its NaN must not pollute the other segments in the same batch. Test: a degenerate red
    // segment + a visible green control segment, assert the green segment still forms its sheet (were perp unguarded, NaN vertices would discard the whole batch -> green = 0 means red).
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let axis = [0.0f32, -0.89, 0.45];
    let tid = upload(
        &mut b,
        &[
            StrokeSeg::new(
                [0.0, 0.0, 0.0],
                [axis[0] * 6.0, axis[1] * 6.0, axis[2] * 6.0],
                [1.0, 0.0, 0.0],
                6.0,
            ), // degenerate (parallel to the view direction)
            StrokeSeg::new([-2.0, -1.6, 0.0], [2.0, -1.6, 0.0], [0.0, 1.0, 0.0], 4.0), // visible control
        ],
    );
    let img = b.render_to_pixels(&stroke_frame(tid)).expect("no panic");
    let green = |p: [u8; 4]| p[1] > 200 && p[0] < 40 && p[2] < 40;
    assert!(
        count(&img, 0, W, 0, H, green) >= 30,
        "the degenerate segment's NaN polluted the whole batch (the green control segment vanished)"
    );
}

#[test]
#[ignore = "probe"]
fn probe_map() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tid = upload(
        &mut b,
        &[
            StrokeSeg::new([-2.5, -1.6, 0.0], [2.5, -1.6, 0.0], [1.0, 0.0, 0.0], 4.0),
            StrokeSeg::new([-2.2, 0.2, 0.0], [-2.2, 2.6, 0.0], [0.0, 1.0, 0.0], 4.0),
            StrokeSeg::new([0.5, 0.2, 0.0], [2.4, 2.6, 0.0], [0.0, 0.0, 1.0], 4.0),
        ],
    );
    let img = b.render_to_pixels(&stroke_frame(tid)).expect("render");
    for y in (0..64).step_by(2) {
        let row: String = (0..64)
            .step_by(2)
            .map(|x| {
                let p = px(&img, x, y);
                match (p[0] > 200, p[1] > 200, p[2] > 200) {
                    (true, false, false) => "R",
                    (false, true, false) => "G",
                    (false, false, true) => "B",
                    (t, g, bl) if t || g || bl => "o",
                    _ => ".",
                }
                .to_string()
            })
            .collect();
        println!("{y:2} {row}");
    }
    // degenerate segments
    let mut b2 = HeadlessBackend::new(W, H).expect("adapter");
    let t2 = upload(
        &mut b2,
        &[StrokeSeg::new(
            [0.0, 0.0, 0.0],
            [0.0, -2.67, 1.35],
            [1.0, 0.0, 0.0],
            4.0,
        )],
    );
    let img2 = b2.render_to_pixels(&stroke_frame(t2)).expect("render2");
    let red = |p: [u8; 4]| p[0] > 200 && p[1] < 40;
    let mut ys = (0..64).filter(|&y| (0..64).any(|x| red(px(&img2, x, y))));
    println!(
        "degen red rows: {:?} count={}",
        (ys.next(), ys.next()),
        (0..64)
            .filter(|&y| (0..64).any(|x| red(px(&img2, x, y))))
            .count()
    );
}

// spec: WGPU-17
/// The E503 window-path mirror (the gate carried by the tile-expansion family's standalone lesson, the REND-29 constant-width semantics): under ortho two zoom
/// gears (a 4x span) the **screen thickness is constant** -- the white line 10px / the green dot diameter 24px keep the same value in both gears (a world-thickness model would multiply by 4).
/// Predicate triple self-proof: (1) positive = constant widths 10/24; (2) a world-width fake form must catch it (the between-gear ratio deviates beyond 2x);
/// (3) the pure-black background eats no color values.
#[test]
fn golden_zoom_width_invariant() {
    let Some(mut b) = HeadlessBackend::new(256, 256) else {
        eprintln!("SKIP: no adapter");
        return;
    };
    let segs = [visiaengine_render::StrokeSeg::new(
        [-2.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [0.92, 0.94, 0.96],
        10.0,
    )];
    let pts = [visiaengine_render::PointMark::new(
        [0.0, 0.6, 0.0],
        [0.30, 0.80, 0.45],
        12.0,
    )];
    let st = b
        .create_strokes(&visiaengine_render::StrokeTableDesc { data: &segs })
        .unwrap();
    let pt = b
        .create_points(&visiaengine_render::PointTableDesc { data: &pts })
        .unwrap();
    let rig = CameraRig::look_at([0.0, 0.0, 12.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let mut thick_prev = 0u32;
    for (zi, zoom) in [1.3f64, 5.2].into_iter().enumerate() {
        let commands = vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            DrawCommand::DrawStrokes {
                table: st,
                origin: [0.0; 3],
                transform: T4,
            },
            DrawCommand::DrawPoints {
                table: pt,
                origin: [0.0; 3],
                transform: T4,
            },
        ];
        let frame = Frame {
            viewport: Viewport::new(256, 256, 1.0),
            camera: Camera::ortho(zoom as f32, zoom as f32, 0.1, 360.0),
            view_rot: rig.view_rotation(),
            eye: rig.eye(),
            proj: rig
                .ortho_frame(zoom as f32, 256.0, 256.0, 0.1, 360.0)
                .unwrap(),
            px_world_scale: 2.0 * zoom as f32 / 256.0,
            shadow: None,
            clip: None,
            edl: None,
            post: Vec::new(),
            commands,
        };
        let img = b.render_to_pixels(&frame).unwrap();
        // the white line's vertical thickness at column x = 96 (under both zoom gears the world x in [-1.3,-0.325] always falls inside the line span [-2,2],
        // and stays far from the green sphere's world radius +/- 0.24 -- the column choice = a sampling-phase discipline independent of zoom)
        let thick: u32 = (0..256u32)
            .filter(|&y| {
                let o = ((y * 256 + 96) * 4) as usize;
                let p = &img.rgba[o..o + 3];
                p[0] > 200 && p[1] > 200 && p[2] > 200
            })
            .count() as u32;
        // the green dot's transverse diameter: a scan row through the sphere center (find the longest continuous green-family run, the row not pre-assumed -- the sphere center moves with zoom)
        let mut diam = 0u32;
        for y in 0..256u32 {
            let run = (0..256u32)
                .map(|x| {
                    let o = ((y * 256 + x) * 4) as usize;
                    let p = &img.rgba[o..o + 3];
                    (p[1] > 150 && p[0] < 130 && p[2] < 170) as u32
                })
                .fold((0u32, 0u32), |(cur, best), hit| {
                    let cur = if hit == 1 { cur + 1 } else { 0 };
                    (cur, best.max(cur))
                })
                .1;
            diam = diam.max(run);
        }
        assert!(
            thick.abs_diff(10) <= 3,
            "gear {zi} zoom={zoom} line screen thickness = {thick} is not the constant width 10 (under a world-width model it must scale with zoom)"
        );
        assert!(
            diam.abs_diff(24) <= 4,
            "gear {zi} dot diameter = {diam}, not the constant diameter 24"
        );
        if zi == 1 {
            // equal thickness across the two gears = the constant-width between-gear lock (the 4x world model diverges dramatically)
            assert!(
                thick.abs_diff(thick_prev) <= 4 && thick_prev.abs_diff(10) <= 3,
                "between-gear drift prev={thick_prev} now={thick}"
            );
        }
        thick_prev = thick;
    }
}
