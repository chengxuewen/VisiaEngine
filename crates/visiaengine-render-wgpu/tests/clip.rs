//! WGPU-21/22/23: section-clip pipeline -- fs discard (mesh family / tile-expansion family / caster consistency).
//! Predicate discipline [PIT-8]: family counts + intervals, canary bitwise form (no-mistaken-kill lock).

use visiaengine_render::{
    Camera, CameraRig, ClipSetup, DrawCommand, Frame, MeshDesc, PointMark, PointTableDesc,
    RenderBackend, ShadowSetup, StrokeSeg, StrokeTableDesc, Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, OffscreenFrame, unit_box_mesh};

const W: u32 = 64;
const H: u32 = 64;
const T4: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

// -- top-view wall scene: z = 0 red wall 32x32, camera straight top view = the half-cut exact half-region form --

fn wall_mesh(b: &mut HeadlessBackend) -> u64 {
    b.create_mesh(&MeshDesc {
        positions: &[
            [-16.0, -16.0, 0.0],
            [16.0, -16.0, 0.0],
            [16.0, 16.0, 0.0],
            [-16.0, 16.0, 0.0],
        ],
        normals: &[[0.0, 0.0, 1.0]; 4],
        indices: &[0, 1, 2, 0, 2, 3],
        uv: &[],
    })
    .expect("wall")
}

fn red(b: &mut HeadlessBackend) -> u64 {
    b.create_material([1.0, 0.0, 0.0, 1.0]).expect("red")
}

fn top_frame(commands: Vec<DrawCommand>, clip: Option<ClipSetup>) -> Frame {
    let rig = CameraRig::look_at([0.0, 0.0, 20.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.5, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, 1.0, 0.5, 100.0)
            .expect("proj"),
        px_world_scale: 0.3,
        shadow: None,
        clip,
        edl: None,
        post: Vec::new(),
        commands,
    }
}

fn render_wall(b: &mut HeadlessBackend, clip: Option<ClipSetup>) -> OffscreenFrame {
    let cmds = wall_only(b);
    let f = top_frame(cmds, clip);
    b.render_to_pixels(&f).expect("render")
}

fn wall_only(b: &mut HeadlessBackend) -> Vec<DrawCommand> {
    let (w_, r_) = (wall_mesh(b), red(b));
    vec![
        DrawCommand::ClearColor {
            rgba: [0.0, 0.0, 0.0, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: w_,
            material: r_,
            origin: [0.0; 3],
            transform: T4,
        },
    ]
}

fn px(img: &OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [
        img.rgba[i],
        img.rgba[i + 1],
        img.rgba[i + 2],
        img.rgba[i + 3],
    ]
}
fn is_red(p: [u8; 4]) -> bool {
    p[0] > 120 && p[1] < 80 && p[2] < 80
}
fn count_red(img: &OffscreenFrame) -> u32 {
    (0..H)
        .map(|y| (0..W).filter(|&x| is_red(px(img, x, y))).count() as u32)
        .sum()
}

/// Triple self-proof (1) canary = no-mistaken-kill bitwise form + (2) half-cut sharp drop + None/EMPTY equivalence + AND corner domain.
#[test]
// spec: WGPU-21
fn wall_canary_halfcut_empty_and_corner() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let base = render_wall(&mut b, None);
    let n0 = count_red(&base);
    assert!(n0 > 3500, "the wall should tile the viewport got {n0}");
    // (1) canary: a plane keeping everything -> byte-identical (dual lock: the predicate's no-mistaken-kill + the None position's zero regression)
    let keep_all = ClipSetup::new(&[[0.0, -1.0, 0.0, 50.0]]).expect("keeps y <= 50");
    let img_k = render_wall(&mut b, Some(keep_all));
    assert_eq!(
        base.rgba, img_k.rgba,
        "must be byte-identical under the canary plane"
    );
    // None = Some(EMPTY) (byte by byte)
    let img_e = render_wall(&mut b, Some(ClipSetup::EMPTY));
    assert_eq!(
        base.rgba, img_e.rgba,
        "EMPTY must be byte-identical to no clipping"
    );
    // (2) half-cut y >= 0: the red family ~ half (threshold set by measurement = PIT-8; interval = half-region +/- 8px edge-row tolerance)
    let half = ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0]]).expect("keeps y >= 0");
    let img_h = render_wall(&mut b, Some(half));
    let nh = count_red(&img_h);
    assert!(
        (nh * 2).abs_diff(n0) <= 260,
        "half cut should ~ half n0={n0} nh={nh}"
    );
    // full cut: the plane on the other side of the wall -> red = 0 (the positive-kill form, guarding canary false-green bidirectionally)
    let cut_all = ClipSetup::new(&[[0.0, 1.0, 0.0, -50.0]]).expect("keeps y >= 50");
    let img_a = render_wall(&mut b, Some(cut_all));
    assert_eq!(
        count_red(&img_a),
        0,
        "a full cut must zero out (positive-kill control)"
    );
    // (3) AND corner domain y >= 0 and x >= 0 -> ~ one quarter
    let corner =
        ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]]).expect("corner domain");
    let img_c = render_wall(&mut b, Some(corner));
    let nc = count_red(&img_c);
    assert!(
        (nc * 4).abs_diff(n0) <= 260,
        "the AND corner domain should ~ 1/4 n0={n0} nc={nc}"
    );
}

/// Far-origin bitwise-same predicate [the pipeline delivery of the REND-32 precision discipline]: origin = 1e7 campus frame = origin frame byte by byte.
#[test]
// spec: WGPU-21
fn far_origin_clip_bitwise_identity() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (wm, rm) = (wall_mesh(&mut b), red(&mut b));
    let big = [1.0e7, -2.0e6, 0.0];
    let cmds_small = vec![
        DrawCommand::ClearColor {
            rgba: [0.0, 0.0, 0.0, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: wm,
            material: rm,
            origin: [0.0; 3],
            transform: T4,
        },
    ];
    let cmds_big = vec![
        DrawCommand::ClearColor {
            rgba: [0.0, 0.0, 0.0, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: wm,
            material: rm,
            origin: big,
            transform: T4,
        },
    ];
    // plane: origin-frame y >= 0 = large-coordinate-frame y >= -2e6 (world coefficients f64-anchored, d = +2e6)
    let clip_small = ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0]]).expect("c1");
    let clip_big = ClipSetup::new(&[[0.0, 1.0, 0.0, 2.0e6]]).expect("c2");
    let img_small = b
        .render_to_pixels(&top_frame(cmds_small, Some(clip_small)))
        .expect("r1");
    // the large-coordinate version shifts camera/light wholesale by origin (D7 eye difference = same local): eye shifts +big in sync
    let mut f_big = top_frame(cmds_big, Some(clip_big));
    f_big.eye = [big[0], big[1], 20.0];
    let rig = CameraRig::look_at(f_big.eye, [big[0], big[1], 0.0], [0.0, 1.0, 0.0]);
    f_big.view_rot = rig.view_rotation();
    let img_big = b.render_to_pixels(&f_big).expect("r2");
    assert_eq!(
        img_small.rgba, img_big.rgba,
        "the same geometry at a far origin must be byte-identical"
    );
    assert!(count_red(&img_small) > 1000, "non-empty scene self-proof");
}

/// The tile-expansion family [WGPU-23]: half-clipped yellow line = sharp drop; green disc (kept side) bitwise unmoved; blue disc (clipped side) zeroed.
#[test]
// spec: WGPU-23
fn stroke_and_point_families_clipped() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let seg = StrokeSeg::new([0.0, -10.0, 0.1], [0.0, 10.0, 0.1], [1.0, 1.0, 0.0], 3.0);
    let st = b
        .create_strokes(&StrokeTableDesc { data: &[seg] })
        .expect("st");
    let gm = PointMark::new([6.0, 6.0, 0.1], [0.0, 1.0, 0.0], 5.0);
    let bm = PointMark::new([-6.0, -6.0, 0.1], [0.0, 0.0, 1.0], 5.0);
    let pt = b
        .create_points(&PointTableDesc { data: &[gm, bm] })
        .expect("pt");
    let mk = |clip: Option<ClipSetup>| {
        let rig = CameraRig::look_at([0.0, 0.0, 20.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        Frame {
            viewport: Viewport::new(W, H, 1.0),
            camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.5, 100.0),
            view_rot: rig.view_rotation(),
            eye: rig.eye(),
            proj: rig
                .perspective(rig.fov_y as f32, 1.0, 0.5, 100.0)
                .expect("p"),
            px_world_scale: 0.3,
            shadow: None,
            clip,
            edl: None,
            post: Vec::new(),
            commands: vec![
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
            ],
        }
    };
    let is_yellow = |q: [u8; 4]| q[0] > 120 && q[1] > 120 && q[2] < 100;
    let is_green = |q: [u8; 4]| q[1] > 120 && q[0] < 100 && q[2] < 100;
    let is_blue = |q: [u8; 4]| q[2] > 120 && q[0] < 100 && q[1] < 100;
    let fam = |img: &OffscreenFrame, f: fn([u8; 4]) -> bool| -> u32 {
        (0..H)
            .map(|y| (0..W).filter(|&x| f(px(img, x, y))).count() as u32)
            .sum()
    };
    let base = b.render_to_pixels(&mk(None)).expect("r0");
    let (y0, g0, b0) = (
        fam(&base, is_yellow),
        fam(&base, is_green),
        fam(&base, is_blue),
    );
    assert!(
        y0 > 60 && g0 > 40 && b0 > 40,
        "three-family presence self-proof y={y0} g={g0} b={b0}"
    );
    let half = ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0]]).expect("y≥0");
    let cut = b.render_to_pixels(&mk(Some(half))).expect("r1");
    let (y1, g1, b1) = (
        fam(&cut, is_yellow),
        fam(&cut, is_green),
        fam(&cut, is_blue),
    );
    assert_eq!(b1, 0, "the disc outside the side must zero out got {b1}");
    assert!(
        y1 * 2 <= y0 && y1 > 10,
        "the line's half cut ~ half y0={y0} y1={y1}"
    );
    assert_eq!(
        g0, g1,
        "the kept-side disc stays bitwise unmoved (family counts) g0={g0} g1={g1}"
    );
}

/// Caster consistency [WGPU-22]: an 8x8 instance city clipped the same -> the shadow-patch family collapses (probe-measured threshold first [PIT-8]).
#[test]
// spec: WGPU-22
fn shadow_caster_respects_clip() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let ground = b
        .create_mesh(&MeshDesc {
            positions: &[
                [-20.0, -20.0, -0.05],
                [20.0, -20.0, -0.05],
                [20.0, 20.0, -0.05],
                [-20.0, 20.0, -0.05],
            ],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        })
        .expect("g");
    let gmat = b.create_material([0.75, 0.75, 0.78, 1.0]).expect("gm");
    let (pos, nrm, idx) = unit_box_mesh();
    let boxy = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .expect("bx");
    let bmat = b.create_material([1.0, 1.0, 1.0, 1.0]).expect("bm");
    let city: Vec<visiaengine_render::Instance> = (0..8)
        .flat_map(|gy| {
            (0..8).map(move |gx| {
                visiaengine_render::Instance::new(
                    [-7.0 + gx as f32 * 2.0, -7.0 + gy as f32 * 2.0, 0.0],
                    4.0 + ((gx + gy) % 3) as f32 * 2.0,
                    [0.9, 0.9, 0.92],
                )
            })
        })
        .collect();
    let iid = b
        .create_instances(&visiaengine_render::InstanceDesc { data: &city })
        .expect("ct");
    let rig = CameraRig::look_at([0.0, -30.0, 22.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let leye = [
        f64::from(ld[0] / nn) * 60.0,
        f64::from(ld[1] / nn) * 60.0,
        f64::from(ld[2] / nn) * 60.0,
    ];
    let lrig = CameraRig::look_at(leye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let shadow = ShadowSetup {
        proj: lrig
            .ortho_frame(30.0, W as f32, H as f32, 1.0, 120.0)
            .expect("lp"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5,
        bias: ShadowSetup::DEFAULT_BIAS,
    };
    let mk = |clip: Option<ClipSetup>| Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.5, 300.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .perspective(rig.fov_y as f32, 1.0, 0.5, 300.0)
            .expect("p"),
        px_world_scale: 0.12,
        shadow: Some(shadow),
        clip,
        edl: None,
        post: Vec::new(),
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.35, 0.55, 0.85, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: ground,
                material: gmat,
                origin: [0.0; 3],
                transform: T4,
            },
            DrawCommand::DrawInstances {
                mesh: boxy,
                material: bmat,
                instances: iid,
                origin: [0.0; 3],
                transform: T4,
            },
        ],
    };
    let is_neutral = |q: [u8; 4]| {
        q[0].abs_diff(q[1]) < 18 && q[0].abs_diff(q[2]) < 26 && (40..200).contains(&q[0])
    };
    let dark = |img: &OffscreenFrame| -> u32 {
        (0..H)
            .map(|y| {
                (0..W)
                    .filter(|&x| is_neutral(px(img, x, y)) && px(img, x, y)[0] < 129)
                    .count() as u32
            })
            .sum()
    };
    let full = b.render_to_pixels(&mk(None)).expect("r0");
    let d_full = dark(&full);
    // instances z in [0,4..8]; the clip plane z <= 0.5 -> tower bodies dead, 0.5m bases kept (thin shadow residue)
    let kill = ClipSetup::new(&[[0.0, 0.0, -1.0, 0.5]]).expect("z≤0.5");
    let cut = b.render_to_pixels(&mk(Some(kill))).expect("r1");
    let d_cut = dark(&cut);
    let ground_left = (0..H)
        .map(|y| (0..W).filter(|&x| is_neutral(px(&cut, x, y))).count() as u32)
        .sum::<u32>();
    println!("PROBE d_full={d_full} d_cut={d_cut} ground={ground_left}");
    assert!(d_full > 100, "city-shadow baseline d_full={d_full}");
    assert!(
        d_cut * 3 < d_full,
        "the caster clipped the same, shadow patches collapse full={d_full} cut={d_cut}"
    );
    assert!(
        ground_left > 200,
        "ground survival control got {ground_left}"
    );
}
