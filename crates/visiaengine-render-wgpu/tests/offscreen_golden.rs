//! L1 离屏 golden（真 GPU/lavapipe 渲染回读；无适配器 SKIP——验证地点如实记录）。

const W: u32 = 640;
const H: u32 = 480;

fn px(f: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [f.rgba[i], f.rgba[i + 1], f.rgba[i + 2], f.rgba[i + 3]]
}

macro_rules! skip_if_no_gpu {
    ($frame:expr) => {
        match $frame {
            Some(f) => f,
            None => {
                eprintln!("SKIP: no adapter (record verification location elsewhere)");
                return;
            }
        }
    };
}

// spec: WGPU-03
#[test]
fn golden_center_pixel() {
    let frame = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_triangle());
    let [r, g, b, a] = px(&frame, W / 2, H / 2);
    assert!(
        r >= 200 && g <= 60 && b <= 60 && a == 255,
        "center not red: {r},{g},{b},{a}"
    );
}

// spec: WGPU-04
#[test]
fn golden_frame_dimensions() {
    let frame = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_triangle());
    assert_eq!((frame.width, frame.height), (W, H));
    assert_eq!(frame.rgba.len(), (W * H * 4) as usize);
}

// spec: WGPU-05
#[test]
fn golden_corner_clear_color() {
    let frame = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_triangle());
    for (x, y) in [(0, 0), (W - 1, 0), (0, H - 1), (W - 1, H - 1)] {
        let [r, g, b, _] = px(&frame, x, y);
        assert!(
            r.abs_diff(13) <= 16 && g.abs_diff(18) <= 16 && b.abs_diff(26) <= 16,
            "corner ({x},{y}) polluted: {r},{g},{b}"
        );
    }
}

#[must_use]
fn is_dominant(px: [u8; 4], i: usize) -> bool {
    let mine = px[i] as i16;
    mine >= 40
        && px
            .iter()
            .take(3)
            .enumerate()
            .all(|(j, v)| i == j || mine - (*v as i16) >= 40)
}

// spec: WGPU-06
#[test]
fn golden_cube_center_hit() {
    let frame = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_cube([
        1.0, 0.0, 0.0, 1.0
    ]));
    let p = px(&frame, W / 2, H / 2);
    assert!(is_dominant(p, 0), "center not red-dominant: {p:?}");
}

// spec: WGPU-07
#[test]
fn golden_cube_corner_clear_color() {
    let frame = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_cube([
        1.0, 0.0, 0.0, 1.0
    ]));
    for (x, y) in [(0, 0), (W - 1, 0), (0, H - 1), (W - 1, H - 1)] {
        let [r, g, b, _] = px(&frame, x, y);
        assert!(
            r.abs_diff(13) <= 16 && g.abs_diff(18) <= 16 && b.abs_diff(26) <= 16,
            "corner ({x},{y}) polluted: {r},{g},{b}"
        );
    }
}

// spec: WGPU-08
#[test]
fn golden_cube_silhouette_row() {
    let frame = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_cube([
        1.0, 0.0, 0.0, 1.0
    ]));
    let dyed = |x: u32| {
        let p = px(&frame, x, H / 2);
        p[0] > 80 && p[3] == 255
    };
    let mut transitions = 0;
    let mut run = 0;
    let mut max_run = 0;
    let mut prev = false;
    for x in 0..W {
        let d = dyed(x);
        if d != prev {
            transitions += 1;
            prev = d;
        }
        run = if d { run + 1 } else { 0 };
        max_run = max_run.max(run);
    }
    assert_eq!(transitions, 2, "凸体中线应恰 2 次清↔染跳变");
    assert!(max_run > 100, "投影宽度不足: {max_run}");
}

// spec: WGPU-09
#[test]
fn material_uniform_path_live() {
    // 同几何同相机换绿 material：中心绿主导=uniform 值真实流经管线（非 SKIP 假绿）
    let frame = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_cube([
        0.0, 1.0, 0.0, 1.0
    ]));
    let p = px(&frame, W / 2, H / 2);
    assert!(is_dominant(p, 1), "center not green-dominant: {p:?}");
}

// spec: WGPU-10
#[test]
fn golden_far_origin_pixel_identity() {
    let base = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_cube_at([0.0; 3]));
    let far = skip_if_no_gpu!(visiaengine_render_wgpu::render_offscreen_cube_at([
        1.0e7, 0.0, 0.0
    ]));
    for (x, y) in [
        (W / 2, H / 2),
        (0, 0),
        (W - 1, H - 1),
        (200, 240),
        (440, 300),
    ] {
        assert_eq!(
            px(&base, x, y),
            px(&far, x, y),
            "({x},{y}) 像素不一致=D7 路径未闭环"
        );
    }
}
