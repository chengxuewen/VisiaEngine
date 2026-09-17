//! WGPU-21/22/23：剖面裁切管线——fs discard（mesh 族/扩片族/caster 一致性）。
//! 谓词纪律 [PIT-8]：族计数+区间，canary 逐位形（不误杀锁）。

use visiaengine_render::{
    Camera, CameraRig, ClipSetup, DrawCommand, Frame, MeshDesc, RenderBackend, ShadowSetup,
    StrokeSeg, StrokeTableDesc, PointMark, PointTableDesc, Viewport,
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

// ── 顶视墙场景：z=0 红墙 32×32，相机正俯视=半切精确半区形 ──────────────────

fn wall_mesh(b: &mut HeadlessBackend) -> u64 {
    b.create_mesh(&MeshDesc {
        positions: &[[-16.0, -16.0, 0.0], [16.0, -16.0, 0.0], [16.0, 16.0, 0.0], [-16.0, 16.0, 0.0]],
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
        proj: rig.perspective(rig.fov_y as f32, 1.0, 0.5, 100.0).expect("proj"),
        px_world_scale: 0.3,
        shadow: None,
        clip,
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
        DrawCommand::ClearColor { rgba: [0.0, 0.0, 0.0, 1.0] },
        DrawCommand::DrawMesh { mesh: w_, material: r_, origin: [0.0; 3], transform: T4 },
    ]
}

fn px(img: &OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2], img.rgba[i + 3]]
}
fn is_red(p: [u8; 4]) -> bool {
    p[0] > 120 && p[1] < 80 && p[2] < 80
}
fn count_red(img: &OffscreenFrame) -> u32 {
    (0..H).map(|y| (0..W).filter(|&x| is_red(px(img, x, y))).count() as u32).sum()
}

/// 三自证①canary=不误杀逐位形 + ②半切骤降 + None/EMPTY 等价 + AND 角域。
#[test]
// spec: WGPU-21
fn wall_canary_halfcut_empty_and_corner() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let base = render_wall(&mut b, None);
    let n0 = count_red(&base);
    assert!(n0 > 3500, "墙应铺满视口 got {n0}");
    // ① canary：保留全部的面 → 逐字节等（谓词不误杀 + None 位零回归双重锁）
    let keep_all = ClipSetup::new(&[[0.0, -1.0, 0.0, 50.0]]).expect("y≤50 保留");
    let img_k = render_wall(&mut b, Some(keep_all));
    assert_eq!(base.rgba, img_k.rgba, "canary 面下逐字节必等");
    // None ≡ Some(EMPTY)（逐字节）
    let img_e = render_wall(&mut b, Some(ClipSetup::EMPTY));
    assert_eq!(base.rgba, img_e.rgba, "EMPTY 必须逐字节=无裁切");
    // ② 半切 y≥0：红族≈半（实测定阈=PIT-8；区间=半区±8px 边行容差）
    let half = ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0]]).expect("y≥0 保留");
    let img_h = render_wall(&mut b, Some(half));
    let nh = count_red(&img_h);
    assert!((nh * 2).abs_diff(n0) <= 260, "半切应≈对半 n0={n0} nh={nh}");
    // 全切：面在墙外另一侧 → 红=0（正杀形，防 canary 假绿双向）
    let cut_all = ClipSetup::new(&[[0.0, 1.0, 0.0, -50.0]]).expect("y≥50 保留");
    let img_a = render_wall(&mut b, Some(cut_all));
    assert_eq!(count_red(&img_a), 0, "全切必须清零（正杀对照）");
    // ③ AND 角域 y≥0 ∧ x≥0 → ≈四分之一
    let corner =
        ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0]]).expect("角域");
    let img_c = render_wall(&mut b, Some(corner));
    let nc = count_red(&img_c);
    assert!((nc * 4).abs_diff(n0) <= 260, "AND 角域应≈¼ n0={n0} nc={nc}");
}

/// 远原点逐位同谓词 [REND-32 精度纪律的管线兑现]：origin=1e7 园区系 ≡ 原点系逐字节。
#[test]
// spec: WGPU-21
fn far_origin_clip_bitwise_identity() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let (wm, rm) = (wall_mesh(&mut b), red(&mut b));
    let big = [1.0e7, -2.0e6, 0.0];
    let cmds_small = vec![
        DrawCommand::ClearColor { rgba: [0.0, 0.0, 0.0, 1.0] },
        DrawCommand::DrawMesh { mesh: wm, material: rm, origin: [0.0; 3], transform: T4 },
    ];
    let cmds_big = vec![
        DrawCommand::ClearColor { rgba: [0.0, 0.0, 0.0, 1.0] },
        DrawCommand::DrawMesh { mesh: wm, material: rm, origin: big, transform: T4 },
    ];
    // 面：原点系 y≥0 ≡ 大坐标系 y≥-2e6（世界系数 f64 锚定，d=+2e6）
    let clip_small = ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0]]).expect("c1");
    let clip_big = ClipSetup::new(&[[0.0, 1.0, 0.0, 2.0e6]]).expect("c2");
    let img_small = b.render_to_pixels(&top_frame(cmds_small, Some(clip_small))).expect("r1");
    // 大坐标版相机/光源整体平移 origin（D7 眼位差=同 local）：eye 同步 +big
    let mut f_big = top_frame(cmds_big, Some(clip_big));
    f_big.eye = [big[0], big[1], 20.0];
    let rig = CameraRig::look_at(f_big.eye, [big[0], big[1], 0.0], [0.0, 1.0, 0.0]);
    f_big.view_rot = rig.view_rotation();
    let img_big = b.render_to_pixels(&f_big).expect("r2");
    assert_eq!(img_small.rgba, img_big.rgba, "远原点同几何必须逐字节等");
    assert!(count_red(&img_small) > 1000, "非空场景自证");
}

/// 扩片族 [WGPU-23]：线段半切=族骤降；点盘分侧=一侧清零一侧在场。
#[test]
// spec: WGPU-23
fn stroke_and_point_families_clipped() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    // 跨半轴绿竖线（x=0 线上 y∈[-10,10]）+ 两侧蓝/橙点盘
    let seg = StrokeSeg::new([0.0, -10.0, 0.1], [0.0, 10.0, 0.1], [0.0, 1.0, 0.0], 3.0);
    let st = b.create_strokes(&StrokeTableDesc { data: &[seg] }).expect("st");
    let green = PointMark::new([6.0, 6.0, 0.1], [0.0, 1.0, 0.0], 5.0);
    let blue = PointMark::new([-6.0, -6.0, 0.1], [0.0, 0.0, 1.0], 5.0);
    let pt = b.create_points(&PointTableDesc { data: &[green, blue] }).expect("pt");
    let mk = |clip: Option<ClipSetup>| {
        let rig = CameraRig::look_at([0.0, 0.0, 20.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        Frame {
            viewport: Viewport::new(W, H, 1.0),
            camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.5, 100.0),
            view_rot: rig.view_rotation(),
            eye: rig.eye(),
            proj: rig.perspective(rig.fov_y as f32, 1.0, 0.5, 100.0).expect("p"),
            px_world_scale: 0.3,
            shadow: None,
            clip,
            commands: vec![
                DrawCommand::ClearColor { rgba: [0.0, 0.0, 0.0, 1.0] },
                DrawCommand::DrawStrokes { table: st, origin: [0.0; 3], transform: T4 },
                DrawCommand::DrawPoints { table: pt, origin: [0.0; 3], transform: T4 },
            ],
        }
    };
    let is_green = |p: [u8; 4]| p[1] > 120 && p[0] < 100 && p[2] < 100;
    let is_blue = |p: [u8; 4]| p[2] > 120 && p[0] < 100 && p[1] < 100;
    let fam = |img: &OffscreenFrame, f: fn([u8; 4]) -> bool| -> u32 {
        (0..H).map(|y| (0..W).filter(|&x| f(px(img, x, y))).count() as u32).sum()
    };
    let base = b.render_to_pixels(&mk(None)).expect("r0");
    let (g0, b0) = (fam(&base, is_green), fam(&base, is_blue));
    assert!(g0 > 60 && b0 > 40, "族在场自证 g={g0} b={b0}");
    let half = ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0]]).expect("y≥0");
    let cut = b.render_to_pixels(&mk(Some(half))).expect("r1");
    let (g1, b1) = (fam(&cut, is_green), fam(&cut, is_blue));
    // 蓝盘全灭（y=-6 全侧外）；绿：竖线半长+同色盘保留 → 蓝盘灭后绿族占比骤升
    assert_eq!(b1, 0, "侧外点盘必须清零 got {b1}");
    assert!(g1 > 40 && g1 * 2 < g0 + b0, "线段逐片裁=残留骤降 g0={g0}+b0→g1={g1}");
}

/// caster 一致性 [WGPU-22]：杀投形 → 影斑族崩塌（E501 谓词直用=PIT-8 实测先例）。
#[test]
// spec: WGPU-22
fn shadow_caster_respects_clip() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let ground = b
        .create_mesh(&MeshDesc {
            positions: &[[-20.0, -20.0, -0.05], [20.0, -20.0, -0.05], [20.0, 20.0, -0.05], [-20.0, 20.0, -0.05]],
            normals: &[[0.0, 0.0, 1.0]; 4],
            indices: &[0, 1, 2, 0, 2, 3],
            uv: &[],
        })
        .expect("g");
    let gmat = b.create_material([0.75, 0.75, 0.78, 1.0]).expect("gm");
    let (pos, nrm, idx) = unit_box_mesh();
    let boxy = b
        .create_mesh(&MeshDesc { positions: &pos, normals: &nrm, indices: &idx, uv: &[] })
        .expect("bx");
    let bmat = b.create_material([1.0, 1.0, 1.0, 1.0]).expect("bm");
    let rig = CameraRig::look_at([0.0, -34.0, 26.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let ld = [0.35f32, 0.5, 0.79];
    let nn = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
    let leye = [f64::from(ld[0] / nn) * 80.0, f64::from(ld[1] / nn) * 80.0, f64::from(ld[2] / nn) * 80.0];
    let lrig = CameraRig::look_at(leye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let shadow = ShadowSetup {
        proj: lrig.ortho_frame(45.0, W as f32, H as f32, 1.0, 160.0).expect("lp"),
        view_rot: lrig.view_rotation(),
        eye: lrig.eye(),
        light_dir: ld,
        size: 0.5,
        bias: ShadowSetup::DEFAULT_BIAS,
    };
    // 塔 z∈[0,6]（scale 矩阵），地 z=-0.05；裁面 z≤0.5：地留、塔近乎全灭
    let st4 = [[3.0, 0.0, 0.0, 0.0], [0.0, 3.0, 0.0, 0.0], [0.0, 0.0, 6.0, 0.0], [0.0, 0.0, 3.0, 1.0]];
    let mk = |clip: Option<ClipSetup>| Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.5, 300.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj: rig.perspective(rig.fov_y as f32, 1.0, 0.5, 300.0).expect("p"),
        px_world_scale: 0.12,
        shadow: Some(shadow),
        clip,
        commands: vec![
            DrawCommand::ClearColor { rgba: [0.35, 0.55, 0.85, 1.0] },
            DrawCommand::DrawMesh { mesh: ground, material: gmat, origin: [0.0; 3], transform: T4 },
            DrawCommand::DrawMesh { mesh: boxy, material: bmat, origin: [0.0; 3], transform: st4 },
        ],
    };
    let is_neutral = |p: [u8; 4]| p[0].abs_diff(p[1]) < 18 && p[0].abs_diff(p[2]) < 26 && (40..200).contains(&p[0]);
    let dark = |img: &OffscreenFrame| -> u32 {
        (0..H).map(|y| (0..W).filter(|&x| is_neutral(px(img, x, y)) && px(img, x, y)[0] < 129).count() as u32).sum()
    };
    let full = b.render_to_pixels(&mk(None)).expect("r0");
    let d_full = dark(&full);
    assert!(d_full > 60, "E501 同景影斑基线自证 got {d_full}");
    // 世界 z≤0.5 保留：n=(0,0,-1) d=0.5 → -z+0.5≥0
    let kill = ClipSetup::new(&[[0.0, 0.0, -1.0, 0.5]]).expect("z≤0.5");
    let cut = b.render_to_pixels(&mk(Some(kill))).expect("r1");
    let d_cut = dark(&cut);
    assert!(d_cut * 2 < d_full, "caster 同裁：影斑必须崩塌 full={d_full} cut={d_cut}");
    // 地面存活对照（接收面没被全灭=谓词不是空场假绿）
    let ground_left = (0..H)
        .map(|y| (0..W).filter(|&x| is_neutral(px(&cut, x, y))).count() as u32)
        .sum::<u32>();
    assert!(ground_left > 400, "地面中性族须在场 got {ground_left}");
}
