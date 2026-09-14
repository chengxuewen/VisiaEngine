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
    assert!(count(&img, 0, W, 40, H, red) >= 12, "红横线缺席下方窗");
    assert!(count(&img, 0, 32, 0, 44, green) >= 12, "绿竖线缺席左上窗");
    assert!(count(&img, 32, W, 0, 44, blue) >= 12, "蓝斜线缺席右上窗");
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
    // R2 退化：段轴∥视向 → cross→0，兜底 perp=right。NaN 顶点=像素全弃（族=0 即红）。
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let axis = [0.0f32, -0.89, 0.45];
    let tid = upload(
        &mut b,
        &[StrokeSeg::new(
            [0.0, 0.0, 0.0],
            [axis[0] * 3.0, axis[1] * 3.0, axis[2] * 3.0],
            [1.0, 0.0, 0.0],
            4.0,
        )],
    );
    let img = b.render_to_pixels(&stroke_frame(tid)).expect("no panic");
    let red = |p: [u8; 4]| p[0] > 200 && p[1] < 40;
    assert!(
        count(&img, 0, W, 0, H, red) >= 5,
        "退化兜底缺席（NaN 吞噬）"
    );
}
