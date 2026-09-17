//! E502 · 材质与光影·色彩标定 —— sRGB 全链往返「所见即所得」活证（CORE-16 带 K2）。
//! 四色板法向∥光向 → shade=1.0 → 读回字节应≈CSS 原值（后端线性化输入 + Srgb 目标
//! 编码输出，8bit 量化内互逆）。色域错装=本例首现红。零窗口依赖，headless 直跑。
//! 用法：cargo run --example E502_color_tuning [--frames N]

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 256; // 行 1024B=4×256 对齐
const H: u32 = 256;

/// 四色板：CSS 原值（sRGB 域）。纯主色/中灰/白——中灰最能抓线性域错装。
const SWATCHES: [(u8, u8, u8); 4] = [
    (255, 0, 0),     // 纯红
    (128, 128, 128), // 中灰（错装=偏亮，本案判据）
    (0, 0, 255),     // 纯蓝
    (255, 255, 255), // 白
];

/// 光向（与后端 shadow-off dummy params 旧 LIGHT 同位型 normalize(0.5,0.7,0.4)）——
/// 面元法向取此向 → ndl=1 → shade=0.35+0.65=1.0（色板满亮，色域纯判）。
fn light_dir() -> [f32; 3] {
    let (x, y, z) = (0.5f32, 0.7, 0.4);
    let l = (x * x + y * y + z * z).sqrt();
    [x / l, y / l, z / l]
}

/// 单色板矩形（局部系 z=0，法向=light_dir；四角两三角）。cx=板心 x。
fn swatch_quad(cx: f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
    let n = light_dir();
    let (x0, x1, y0, y1) = (cx - 0.5, cx + 0.5, -1.0, 1.0);
    let pos = vec![[x0, y0, 0.0], [x1, y0, 0.0], [x1, y1, 0.0], [x0, y1, 0.0]];
    let nrm = vec![n; 4];
    (pos, nrm, vec![0, 1, 2, 0, 2, 3])
}

fn main() {
    let mut frames = 1u32;
    let mut args = std::env::args();
    while let Some(a) = args.next() {
        if a == "--frames"
            && let Some(n) = args.next()
        {
            frames = n.parse().unwrap_or(1).max(1);
        }
    }
    let Some(mut b) = HeadlessBackend::new(W, H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };
    let rig = CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let mut commands = vec![DrawCommand::ClearColor { rgba: [0.0; 4] }];
    // 板心 x：-1.5 -0.5 0.5 1.5（ortho 半宽 2 覆盖）；每板独立 mesh+material
    for (i, (r, g, bl)) in SWATCHES.into_iter().enumerate() {
        let css = [r as f32 / 255.0, g as f32 / 255.0, bl as f32 / 255.0, 1.0];
        let (pos, nrm, idx) = swatch_quad(-1.5 + i as f32);
        let mesh = b
            .create_mesh(&MeshDesc {
                positions: &pos,
                normals: &nrm,
                indices: &idx,
                uv: &[],
            })
            .expect("mesh");
        let mat = b.create_material(css).expect("mat");
        commands.push(DrawCommand::DrawMesh {
            mesh,
            material: mat,
            origin: [0.0; 3],
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        });
    }
    let proj = rig
        .ortho_frame(2.0, W as f32, H as f32, 0.1, 100.0)
        .unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(2.0, 2.0 * H as f32 / W as f32, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [0.0, 0.0, 10.0],
        proj,
        px_world_scale: 1.0,
        shadow: None,
        commands,
    };
    for _ in 0..frames {
        let img = b.render_to_pixels(&frame).expect("render");
        let mut bad = 0usize;
        for (i, (cr, cg, cb)) in SWATCHES.into_iter().enumerate() {
            // 板心像素：ortho 半宽 zoom=2 → px=(0.5 + x/(2·hw))·W=(0.5+x/4)·256
            // 板心 x=-1.5..1.5 → 采样列 32/96/160/224（板界 64/128/192 避开）
            let cxw = -1.5 + i as f32;
            let px = ((0.5 + cxw / 4.0) * W as f32) as u32;
            let py = H / 2;
            let o = ((py * W + px) * 4) as usize;
            let (r, g, bl) = (img.rgba[o], img.rgba[o + 1], img.rgba[o + 2]);
            // 8bit 量化 + 采样相位容差 ±8；中灰错装会偏到 ~188（线性域直存）→ 必抓
            let ok = r.abs_diff(cr) <= 8 && g.abs_diff(cg) <= 8 && bl.abs_diff(cb) <= 8;
            if !ok {
                bad += 1;
            }
            println!(
                "SWATCH {i} css=({cr},{cg},{cb}) read=({r},{g},{bl}) {}",
                if ok { "OK" } else { "✗ 色域漂移" }
            );
        }
        if bad > 0 {
            println!("FAIL color tuning bad={bad}");
            std::process::exit(1);
        }
    }
    println!("OK color tuning（sRGB 全链往返互逆·所见即所得）");
}
