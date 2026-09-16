//! E601 · 规模与性能 —— 10 万楼块实例化单 draw + [6b] `RESULT <name> <value> <unit>` 协议。
//! 用法：`cargo run --release -p visiaengine-render-wgpu --example E601_bench_twin -- [--count N] [--frames N]`
//! headless 形态（smoke-pick 同族免 xvfb）。**数字=本机 lavapipe 软光栅观测，非 CI 门禁** [6b]。

use std::time::Instant;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MeshDesc, RenderBackend,
    Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};

const W: u32 = 512;
const H: u32 = 512;

/// xorshift32——确定性伪随机（零 rand 依赖，同 seed 同城 [6b 可复现纪律]）
struct Rng(u32);

impl Rng {
    fn next01(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }
}

const IDENTITY: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

fn main() {
    let (mut count, mut frames) = (100_000u32, 3u32);
    let mut args = std::env::args();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--count" => {
                count = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n: &u32| *n > 0)
                    .unwrap_or(count);
            }
            "--frames" => {
                frames = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n: &u32| *n > 0)
                    .unwrap_or(frames);
            }
            _ => {}
        }
    }
    let Some(mut b) = HeadlessBackend::new(W, H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };

    // ===== 上传段计时（CPU 生成 + 表上传，REND-27/WGPU-16 面）=====
    let t0 = Instant::now();
    let (pos, nrm, idx) = unit_box_mesh();
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .expect("box mesh");
    let mat = b.create_material([1.0, 1.0, 1.0, 1.0]).expect("white");
    let side = (f64::from(count).sqrt().ceil()) as u32;
    let half = side as f32; // 间距 2 → 城市半径 ≈ side 单位
    let mut rng = Rng(0x9E37_79B9 ^ count);
    let table: Vec<Instance> = (0..count)
        .map(|i| {
            let x = (i % side) as f32 * 2.0 - half;
            let y = (i / side) as f32 * 2.0 - half;
            let h = 1.0 + rng.next01() * 7.0;
            let g = 0.55 + h / 7.0 * 0.25; // 高度→亮带（白模的灰蓝渐变）
            Instance::new([x, y, 0.0], h, [g * 0.9, g, g * 1.05])
        })
        .collect();
    let iid = b
        .create_instances(&InstanceDesc { data: &table })
        .expect("100k table");
    let upload_ms = t0.elapsed().as_secs_f64() * 1000.0;

    let dist = f64::from(half) * 2.7;
    let rig = CameraRig::look_at(
        [0.0, -dist * 0.8, dist * 0.6],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );
    let proj = rig
        .perspective(rig.fov_y as f32, 1.0, 1.0, (dist * 6.0) as f32)
        .expect("proj");
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 1.0, (dist * 6.0) as f32),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 1.0,
        shadow: None,
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.02, 0.02, 0.03, 1.0],
            },
            DrawCommand::DrawInstances {
                mesh,
                material: mat,
                instances: iid,
                origin: [0.0; 3],
                transform: IDENTITY,
            },
        ],
    };

    // ===== 帧计时（首帧=预热除管线惰性建组，均值取后续 [6b]）=====
    let img = b.render_to_pixels(&frame).expect("warmup");
    let t1 = Instant::now();
    let mut last = img;
    for _ in 0..frames {
        last = b.render_to_pixels(&frame).expect("frame");
    }
    let frame_ms = t1.elapsed().as_secs_f64() * 1000.0 / f64::from(frames);

    // ===== 出图诚实自检（非背景占比——跑通≠画对最低线，[PIT-8] 区域统计）=====
    let lit = last
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 60)
        .count();
    let total = (W * H) as usize;
    assert!(lit * 10 > total, "城市出图非背景占比异常 {}/{}", lit, total);

    println!("RESULT bench_twin_upload {upload_ms:.3} ms");
    println!("RESULT bench_twin_frame {frame_ms:.3} ms");
    println!("RESULT bench_twin_instances {count} count");
    println!(
        "ok bench_twin lit={}/{} ({:.1}%)",
        lit,
        total,
        lit as f64 * 100.0 / total as f64
    );
}
