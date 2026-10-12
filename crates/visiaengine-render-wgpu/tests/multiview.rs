//! WGPU-26 multi-viewport splitting: the clear-semantics ruling gate + mutual non-aggression + single-projection canary bitwise equality + depth/shadow sharing.
//! Predicate discipline [PIT-8]: thresholds = probe-measured numbers; the ruling gate asserts the divergence point of the two forms' predictions (this machine's wgpu30 truth).

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, ShadowSetup, Viewport,
    ViewportRect,
};
use visiaengine_render_wgpu::{HeadlessBackend, MultiClearPolicy};

const W: u32 = 128;
const H: u32 = 64;
const ID64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// The big palette (world +/- 60, any top-view rect with width <= 120 must cover it fully)
fn slab(b: &mut HeadlessBackend) -> u64 {
    b.create_mesh(&MeshDesc {
        positions: &[
            [-60.0, -60.0, 0.0],
            [60.0, -60.0, 0.0],
            [60.0, 60.0, 0.0],
            [-60.0, 60.0, 0.0],
        ],
        normals: &[[0.0, 0.0, 1.0]; 4],
        indices: &[0, 1, 2, 0, 2, 3],
        uv: &[],
    })
    .expect("slab")
}

fn mat(b: &mut HeadlessBackend, col: [f32; 4]) -> u64 {
    b.create_material(col).expect("mat")
}

/// The top-view ortho frame (zoom = half-width world; the px exact road)
fn ortho_frame(commands: Vec<DrawCommand>, rect: &ViewportRect, z: f64) -> Frame {
    let rig = CameraRig::look_at([0.0, 0.0, 50.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let zw = z * (rect.height as f64 / rect.width as f64); // vertical half-height (square rect = same z)
    Frame {
        viewport: Viewport::new(rect.width, rect.height, 1.0),
        camera: Camera::ortho(z as f32, zw as f32, 0.5, 500.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig
            .ortho_frame(z as f32, rect.width as f32, rect.height as f32, 0.5, 500.0)
            .expect("p"),
        px_world_scale: 2.0 * z as f32 / rect.width as f32,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands,
    }
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
    r: &ViewportRect,
    f: fn([u8; 4]) -> bool,
) -> u32 {
    (r.y..r.y + r.height)
        .flat_map(|y| (r.x..r.x + r.width).map(move |x| (x, y)))
        .filter(|(x, y)| f(px(img, *x, *y)))
        .count() as u32
}
fn is_red(p: [u8; 4]) -> bool {
    p[0] > 120 && p[1] < 60 && p[2] < 60
}
fn is_blue(p: [u8; 4]) -> bool {
    p[2] > 120 && p[0] < 60 && p[1] < 90
}
fn is_black(p: [u8; 4]) -> bool {
    p[0] < 8 && p[1] < 8 && p[2] < 8
}

fn ra() -> ViewportRect {
    ViewportRect::new(0, 0, 50, 64)
}
fn rb() -> ViewportRect {
    ViewportRect::new(54, 0, 50, 64)
} // seams x in [50,54) and [104,128)

/// Ruling gate (1) + non-aggression (2): under the safe form red and blue each hold their own region, the seam = the full-frame clear color;
/// under the AllClear form (pass2 also Clears) the red region must be dead -- the divergence point is independently falsifiable in both forms.
#[test]
// spec: WGPU-26
fn clear_semantics_verdict_and_non_erasure() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (red, blue) = (
        mat(&mut b, [1.0, 0.0, 0.0, 1.0]),
        mat(&mut b, [0.0, 0.0, 1.0, 1.0]),
    );
    let (sr, sb) = (slab(&mut b), slab(&mut b));
    let mk = |mesh: u64, m: u64| {
        vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh,
                material: m,
                origin: [0.0; 3],
                transform: ID64,
            },
        ]
    };
    let fa = ortho_frame(mk(sr, red), &ra(), 20.0);
    let fb = ortho_frame(mk(sb, blue), &rb(), 20.0);
    // the safe form (first Clear, then Load)
    let img = b
        .render_to_pixels_rects(
            &[(fa.clone(), ra()), (fb.clone(), rb())],
            MultiClearPolicy::FirstClearRestLoad,
        )
        .expect("safe multi");
    let (red_a, blue_b) = (count(&img, &ra(), is_red), count(&img, &rb(), is_blue));
    let (red_b, blue_a) = (count(&img, &rb(), is_red), count(&img, &ra(), is_blue));
    println!("PROBE safe: redA={red_a} blueB={blue_b} bleed redB={red_b} blueA={blue_a}");
    assert!(
        red_a > 1500 && blue_b > 1500,
        "both regions' own colors must be present red={red_a} blue={blue_b}"
    );
    assert_eq!(red_b, 0, "red must not cross into the blue region");
    assert_eq!(blue_a, 0, "blue must not cross into the red region");
    let gutter = ViewportRect::new(104, 0, 24, 64);
    assert!(
        count(&img, &gutter, is_black) >= gutter.width * gutter.height - 4,
        "the right seam must = the full-frame clear background color"
    );
    // the ruling form (pass2 also Clears: if Clear respects area red survives; if Clear spans the whole attachment red dies)
    let img2 = b
        .render_to_pixels_rects(&[(fa, ra()), (fb, rb())], MultiClearPolicy::AllClear)
        .expect("allclear multi");
    let red_a2 = count(&img2, &ra(), is_red);
    println!(
        "PROBE allclear: redA={red_a2} (0 = wgpu Clear acts on the whole attachment => probe A's verdict holds, first-Clear-then-Load is the only safe form)"
    );
    assert_eq!(
        red_a2, 0,
        "ruling divergence: wgpu30 on this machine measures Clear as not-the-whole-region form -- contrary to the plan's foundation verdict, the band halts for re-presentation"
    );
}

/// Canary (2): a single full-rect multi-viewport road == the old full-screen road byte by byte (the zero-regression lock on the existing form).
#[test]
// spec: WGPU-26
fn single_rect_equals_legacy_bitwise() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let red = mat(&mut b, [1.0, 0.05, 0.05, 1.0]);
    let sr = slab(&mut b);
    let cmds = vec![
        DrawCommand::ClearColor {
            rgba: [0.0, 0.0, 0.0, 1.0],
        },
        DrawCommand::DrawMesh {
            mesh: sr,
            material: red,
            origin: [0.0; 3],
            transform: ID64,
        },
    ];
    let full = ViewportRect::full(W, H);
    let legacy = b
        .render_to_pixels(&ortho_frame(cmds.clone(), &full, 20.0))
        .expect("legacy");
    let multi = b
        .render_to_pixels_rects(
            &[(ortho_frame(cmds, &full, 20.0), full)],
            MultiClearPolicy::FirstClearRestLoad,
        )
        .expect("multi");
    assert_eq!(
        legacy.rgba, multi.rgba,
        "a single projection must be byte-identical to the old road"
    );
}

/// Depth sharing (3): two stacked plates in the blue region (near covers far) with the correct order + no interference with the red region's residual depth (scissor-locked raster).
#[test]
// spec: WGPU-26
fn shared_depth_scissor_fenced_order() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (red, blue, green) = (
        mat(&mut b, [1.0, 0.0, 0.0, 1.0]),
        mat(&mut b, [0.0, 0.0, 1.0, 1.0]),
        mat(&mut b, [0.0, 1.0, 0.0, 1.0]),
    );
    let s_red = slab(&mut b);
    let s_blue = b
        .create_mesh(&MeshDesc {
            positions: &[
                [-6.0, -6.0, 4.0],
                [6.0, -6.0, 4.0],
                [6.0, 6.0, 4.0],
                [-6.0, 6.0, 4.0],
            ],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        })
        .expect("blue");
    let s_green = slab(&mut b);
    let fa = ortho_frame(
        vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: s_red,
                material: red,
                origin: [0.0; 3],
                transform: ID64,
            },
        ],
        &ra(),
        20.0,
    );
    // blue region top view: the big green plate below, the small blue plate above (near->far order = reversed draw order, depth decides the bout)
    let fb = ortho_frame(
        vec![
            DrawCommand::ClearColor {
                rgba: [0.0, 0.0, 0.0, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: s_green,
                material: green,
                origin: [0.0; 3],
                transform: ID64,
            },
            DrawCommand::DrawMesh {
                mesh: s_blue,
                material: blue,
                origin: [0.0; 3],
                transform: ID64,
            },
        ],
        &rb(),
        20.0,
    );
    let img = b
        .render_to_pixels_rects(
            &[(fa, ra()), (fb, rb())],
            MultiClearPolicy::FirstClearRestLoad,
        )
        .expect("multi depth");
    for y in (0..H).step_by(4) {
        let row: String = (0..W)
            .step_by(4)
            .map(|x| {
                let q = px(&img, x, y);
                match (q[0] > 120, q[1] > 120, q[2] > 120) {
                    (true, false, false) => "R",
                    (false, false, true) => "B",
                    (false, true, false) => "G",
                    (true, true, true) => "W",
                    _ => ".",
                }
            })
            .collect();
        println!("ROW{y:02} {row}");
    }
    let blue_c = count(&img, &rb(), is_blue); // the blue region in full (dump-fixed form: the small plate ~64px magnitude)
    assert!(
        blue_c > 100,
        "the near plate must cover the far one (depth sharing + scissor-locked raster failing would bleed red/green) got {blue_c}"
    );
    assert!(
        count(&img, &ra(), is_red) > 1500,
        "the red region holds its own"
    );
}

/// Shadow sharing (4): one caster run serves both viewports on the same map -- each region sees its own shadow patches (duplicate cost only if each projection ran its own; sharing saves the tax).
#[test]
// spec: WGPU-26
fn caster_once_both_views_have_shadow() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let ground_m = mat(&mut b, [0.75, 0.75, 0.78, 1.0]);
    let wall_m = mat(&mut b, [1.0, 1.0, 1.0, 1.0]);
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
    let (pos, nrm, idx) = visiaengine_render_wgpu::unit_box_mesh();
    let boxy = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .expect("bx");
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
    let cmds = || {
        vec![
            DrawCommand::ClearColor {
                rgba: [0.35, 0.55, 0.85, 1.0],
            },
            DrawCommand::DrawMesh {
                mesh: ground,
                material: ground_m,
                origin: [0.0; 3],
                transform: ID64,
            },
            DrawCommand::DrawInstances {
                mesh: boxy,
                material: wall_m,
                instances: iid,
                origin: [0.0; 3],
                transform: ID64,
            },
        ]
    };
    let rig = CameraRig::look_at([0.0, -30.0, 22.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let leye = [
        f64::from(ld[0] / nn) * 60.0,
        f64::from(ld[1] / nn) * 60.0,
        f64::from(ld[2] / nn) * 60.0,
    ];
    let lrig = CameraRig::look_at(leye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let sh = ShadowSetup {
        proj: lrig
            .ortho_frame(30.0, W as f32, H as f32, 1.0, 120.0)
            .expect("lp"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5,
        bias: ShadowSetup::DEFAULT_BIAS,
    };
    let mkf = |rect: &ViewportRect| {
        let ar = rect.width as f32 / rect.height as f32;
        Frame {
            viewport: Viewport::new(rect.width, rect.height, 1.0),
            camera: Camera::perspective(rig.fov_y as f32, ar, 0.5, 300.0),
            view_rot: rig.view_rotation(),
            eye: rig.eye(),
            proj: rig
                .perspective(rig.fov_y as f32, ar, 0.5, 300.0)
                .expect("p"),
            px_world_scale: 0.12,
            shadow: Some(sh),
            clip: None,
            edl: None,
            post: Vec::new(),
            commands: cmds(),
        }
    };
    let (fa, fb) = (mkf(&ra()), mkf(&rb()));
    let img = b
        .render_to_pixels_rects(
            &[(fa, ra()), (fb, rb())],
            MultiClearPolicy::FirstClearRestLoad,
        )
        .expect("shadow multi");
    let dark = |r: &ViewportRect| -> u32 {
        count(&img, r, |q| {
            q[0].abs_diff(q[1]) < 18 && q[0].abs_diff(q[2]) < 26 && (40..129).contains(&q[0])
        })
    };
    let (d_a, d_b) = (dark(&ra()), dark(&rb()));
    println!("PROBE shadow: darkA={d_a} darkB={d_b}");
    assert!(
        d_a > 30 && d_b > 30,
        "both regions each see shadow (the shared caster map serves both projections) A={d_a} B={d_b}"
    );
}
