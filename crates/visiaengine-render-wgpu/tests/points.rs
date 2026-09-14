//! WGPU-18：Points splat 管线——屏幕系展开 + FS 圆 mask（真圆点，纠 geo 方块存量）。
//! 相机取俯角 76°（sin≈0.97）保圆证据不被透视压扁 [PIT-8 校准]。

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
/// 眼 [0,-2,8]：目标面 ≈0.15 世界/px（半高 ~4.8/64——阶梯量纲前提）
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
    // 探针先校准中心（probe 后收紧）：中心实心
    assert!(at(0, 0), "中心缺色");
    // 圆证据：轴向近缘 (±6,0) 有色；对角 (±6.5,±6.5) 世界距 >r → 弃片
    assert!(at(6, 0) || at(0, 6), "轴向近缘缺色（半径未生效）");
    assert!(
        !at(7, 7) && !at(-7, 7) && !at(7, -7) && !at(-7, -7),
        "对角被覆盖=方片非圆（mask 缺席）"
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
    assert!(s >= 5, "小点缺席 {s}");
    // 面积比 (10/2)²=25，透视/椭圆微损 → ≥8 保守
    assert!(l > s * 8, "半径阶梯失真 {s}→{l}");
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
    // 右/up 世界 +y 投影上移（行减小）——符号错→上下镜像
    assert!(count(&img, 0, 24, 20, 46, r) >= 12, "红左窗缺席");
    assert!(count(&img, 40, W, 20, 46, bl) >= 12, "蓝右窗缺席");
    assert!(count(&img, 20, 44, 0, 30, g) >= 12, "绿上窗缺席");
}

// spec: WGPU-18
#[test]
fn empty_and_missing_tables_semantics() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    assert!(b.create_points(&PointTableDesc { data: &[] }).is_err());
    // 缺表=skip 非 panic
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
    let img = b.render_to_pixels(&f).expect("缺表不得 panic");
    assert!(px(&img, 32, 32)[0] < 10, "缺表画出了东西");
}
