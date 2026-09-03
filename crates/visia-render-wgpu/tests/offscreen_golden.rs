//! L1 离屏 golden（真 GPU/lavapipe 渲染回读；无适配器 SKIP——验证地点如实记录）。

const W: u32 = 640;
const H: u32 = 480;

fn px(f: &visia_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
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
    let frame = skip_if_no_gpu!(visia_render_wgpu::render_offscreen_triangle());
    let [r, g, b, a] = px(&frame, W / 2, H / 2);
    assert!(
        r >= 200 && g <= 60 && b <= 60 && a == 255,
        "center not red: {r},{g},{b},{a}"
    );
}

// spec: WGPU-04
#[test]
fn golden_frame_dimensions() {
    let frame = skip_if_no_gpu!(visia_render_wgpu::render_offscreen_triangle());
    assert_eq!((frame.width, frame.height), (W, H));
    assert_eq!(frame.rgba.len(), (W * H * 4) as usize);
}

// spec: WGPU-05
#[test]
fn golden_corner_clear_color() {
    let frame = skip_if_no_gpu!(visia_render_wgpu::render_offscreen_triangle());
    for (x, y) in [(0, 0), (W - 1, 0), (0, H - 1), (W - 1, H - 1)] {
        let [r, g, b, _] = px(&frame, x, y);
        assert!(
            r.abs_diff(13) <= 16 && g.abs_diff(18) <= 16 && b.abs_diff(26) <= 16,
            "corner ({x},{y}) polluted: {r},{g},{b}"
        );
    }
}
