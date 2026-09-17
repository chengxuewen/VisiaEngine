//! WGPU-17：Strokes 扩片管线——色/宽住表、px 语义、盖面 polygon-offset、退化兜底。
//! 谓词纪律 [PIT-8]：窗口+区域族统计+位置主导（right/up 基向量正确性锁），不锁单像素。

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
/// 眼 [0,-8,4]：目标面 ~0.16 世界/px（半高 5.16/64——宽度阶梯的量纲前提）
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

/// 窗口内族计数。
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
            // 红色横线（下方）
            StrokeSeg::new([-2.5, -1.6, 0.0], [2.5, -1.6, 0.0], [1.0, 0.0, 0.0], 4.0),
            // 绿色竖线（左上）
            StrokeSeg::new([-2.2, 0.2, 0.0], [-2.2, 2.6, 0.0], [0.0, 1.0, 0.0], 4.0),
            // 蓝色斜线（右上）
            StrokeSeg::new([0.5, 0.2, 0.0], [2.4, 2.6, 0.0], [0.0, 0.0, 1.0], 4.0),
        ],
    );
    let img = b.render_to_pixels(&stroke_frame(tid)).expect("render");
    let red = |p: [u8; 4]| p[0] > 200 && p[1] < 40 && p[2] < 40;
    let green = |p: [u8; 4]| p[1] > 200 && p[0] < 40 && p[2] < 40;
    let blue = |p: [u8; 4]| p[2] > 200 && p[0] < 40 && p[1] < 40;
    // 位置主导=right/up 基正确性锁（符号错→镜像→窗口断言翻）
    // 窗口界=probe_map 实测（透视下世界 y=-1.6 投影在行 35-39，非底部带）
    assert!(count(&img, 0, W, 33, 44, red) >= 30, "红横线缺席下方窗");
    assert!(count(&img, 0, 32, 0, 44, green) >= 10, "绿竖线缺席左上窗");
    assert!(count(&img, 32, W, 0, 44, blue) >= 10, "蓝斜线缺席右上窗");
    // 全画布纯族（无混色泄漏进背景）
    assert_eq!(
        count(&img, 0, 8, 0, 8, |p| { p[0] + p[1] + p[2] > 30 }),
        0,
        "背景角落被污染"
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
    assert!(c1 > 10 && c2 > 40, "族量 {c1}/{c2}");
    assert!(c2 * 10 > c1 * 25, "宽度阶梯失真 {c1}→{c2}");
}

// spec: WGPU-17
#[test]
fn stroke_over_coplanar_fill_wins_via_offset() {
    // 同 z=0：fill 先画（灰），stroke 后画压其上——无 polygon-offset 时深度相等
    // 严格 Less → 描边被吞（本条即 [E3D:B7③] 的回归锁）。
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
    assert!(g > 100, "fill 缺席（灰带 {g}）");
    assert!(d >= 20, "描边被同深度 fill 吞噬（暗像素 {d}）");
}

// spec: WGPU-17
#[test]
fn view_parallel_segment_falls_back_no_nan() {
    // R2 退化真义：段轴∥视向→cross=0→normalize(0)=NaN。该段投影本就退化为点
    // （0 像素合法），关键=它的 NaN 不得污染同批其它段。测：退化红段 + 可见绿控制段，
    // 断言绿段照常成片（若 perp 未兜底，NaN 顶点会弃掉整批→绿=0 即红）。
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
            ), // 退化（∥视向）
            StrokeSeg::new([-2.0, -1.6, 0.0], [2.0, -1.6, 0.0], [0.0, 1.0, 0.0], 4.0), // 可见控制
        ],
    );
    let img = b.render_to_pixels(&stroke_frame(tid)).expect("no panic");
    let green = |p: [u8; 4]| p[1] > 200 && p[0] < 40 && p[2] < 40;
    assert!(
        count(&img, 0, W, 0, H, green) >= 30,
        "退化段 NaN 污染全批（绿控制段消失）"
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
    // 退化段
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
/// E503 窗口路镜像（扩片族独立课随带门，REND-29 恒宽语义）：ortho 两档 zoom（4×
/// 跨度）下**屏幕厚度恒定**——白线 10px / 绿点 φ24px 两档同值（世界厚度模型会 ×4）。
/// 谓词三自证：①正例=恒宽 10/24；②世界宽假形必抓（zoom 档间比偏离 >2×）；
/// ③背景纯黑不吃色值。
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
            commands,
        };
        let img = b.render_to_pixels(&frame).unwrap();
        // 白线竖向厚度：列 x=96（两档 zoom 下世界 x∈[-1.3,-0.325] 恒落线幅 [-2,2] 内，
        // 且距绿点球径世界 ±0.24 远远的——列选择=与 zoom 无关的采样相位纪律）
        let thick: u32 = (0..256u32)
            .filter(|&y| {
                let o = ((y * 256 + 96) * 4) as usize;
                let p = &img.rgba[o..o + 3];
                p[0] > 200 && p[1] > 200 && p[2] > 200
            })
            .count() as u32;
        // 绿点横向直径：过球心的扫描行（找最大绿族连续段，行不预设——球心随 zoom 换屏位）
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
            "档{zi} zoom={zoom} 线屏厚={thick}≠恒宽 10（世界宽模型下必随 zoom 缩放）"
        );
        assert!(diam.abs_diff(24) <= 4, "档{zi} 点直径={diam}≠恒 φ24");
        if zi == 1 {
            // 两档厚度等值=恒宽的跨档锁（比 4× 世界模型分道扬镳）
            assert!(
                thick.abs_diff(thick_prev) <= 4 && thick_prev.abs_diff(10) <= 3,
                "档间漂移 prev={thick_prev} now={thick}"
            );
        }
        thick_prev = thick;
    }
}
