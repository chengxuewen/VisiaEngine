//! WGPU-26 多视口分屏：clear 语义裁决门 + 互不侵犯 + 单投 canary 逐位等 + depth/shadow 共享。
//! 谓词纪律 [PIT-8]：阈=探针实测定数；裁决门断言两形预言的分歧点（本机 wgpu30 真值）。

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

/// 大色板（world ±60，任意 ≤120 视野 rect 顶视必全覆盖）
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

/// 顶视 ortho 帧（zoom=半宽 world；px 精确路）
fn ortho_frame(commands: Vec<DrawCommand>, rect: &ViewportRect, z: f64) -> Frame {
    let rig = CameraRig::look_at([0.0, 0.0, 50.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let zw = z * (rect.height as f64 / rect.width as f64); // 竖半高（方 rect=同 z）
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
} // 缝 x∈[50,54)∪[104,128)

/// 裁决门 ① + 互不侵犯 ②：safe 形下红蓝各区自守、缝=全幅 clear 色；
/// AllClear 形（pass2 也 Clear）红区必被灭 —— 分歧点两形各自可失败。
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
    // safe 形（首 Clear 后 Load）
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
        "两区自家色必在场 red={red_a} blue={blue_b}"
    );
    assert_eq!(red_b, 0, "红不得越界蓝区");
    assert_eq!(blue_a, 0, "蓝不得越界红区");
    let gutter = ViewportRect::new(104, 0, 24, 64);
    assert!(
        count(&img, &gutter, is_black) >= gutter.width * gutter.height - 4,
        "右缝必=全幅 clear 底色"
    );
    // 裁决形（pass2 也 Clear=若 clear respects area 则红存；若 clear 全幅则红灭）
    let img2 = b
        .render_to_pixels_rects(&[(fa, ra()), (fb, rb())], MultiClearPolicy::AllClear)
        .expect("allclear multi");
    let red_a2 = count(&img2, &ra(), is_red);
    println!(
        "PROBE allclear: redA={red_a2}（0=wgpu Clear 作用全 attachment ⇒ 探针 A 定论成立，首清后 Load 为唯一安全形）"
    );
    assert_eq!(
        red_a2, 0,
        "裁决分歧：wgpu30 本机实测 Clear 非全区形——与计划地基定论相反，停带重呈"
    );
}

/// canary ②：单 full-rect 多视口路 == 旧全屏路逐字节等（存量零回归锁）。
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
    assert_eq!(legacy.rgba, multi.rgba, "单投必逐字节=旧路");
}

/// 深度共享 ③：蓝区两叠板（近覆远）序正确 + 与红区残留深度互不干涉（scissor 圈栅格）。
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
    // 蓝区顶视：绿大板在下、蓝小板在上（近→远序=画序反，深度定胜负）
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
    let blue_c = count(&img, &rb(), is_blue); // 蓝区全域（dump 定形：小板 ~64px 量级）
    assert!(
        blue_c > 100,
        "近板必覆远板（深度共享+scissor 圈栅格失效则红/绿渗底）got {blue_c}"
    );
    assert!(count(&img, &ra(), is_red) > 1500, "红区自守");
}

/// shadow 共享 ④：caster 一趟双视口同 map——两区各自见影斑（若双投各跑一遍才可能重复，共享省税）。
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
        "双区各自见影（共享 caster map 双投成立）A={d_a} B={d_b}"
    );
}
