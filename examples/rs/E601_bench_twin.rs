//! E601 · scale and performance -- 100k building-block instanced single draw + the [6b] `RESULT <name> <value> <unit>` protocol.
//! Usage: `cargo run --release -p visiaengine-render-wgpu --example E601_bench_twin -- [--count N] [--frames N]`
//! headless form (the smoke-pick family, xvfb-free). **Numbers = this machine's lavapipe software-raster observations, not a CI gate** [6b].

#[path = "gallery.rs"]
mod gallery;

use std::time::Instant;

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, Instance, InstanceDesc, MeshDesc, PointMark,
    PointTableDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::{HeadlessBackend, unit_box_mesh};

const W: u32 = 512;
const H: u32 = 512;

/// xorshift32 -- deterministic pseudorandom (zero rand dependency, same seed same city [6b reproducibility discipline])
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
    let mut pcl: Option<u32> = None;
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
            "--points" => {
                pcl = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .filter(|n: &u32| *n > 0);
            }
            _ => {}
        }
    }
    let Some(mut b) = HeadlessBackend::new(W, H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };

    if let Some(n) = pcl {
        bench_pcl(&mut b, n, frames);
        return;
    }

    // ===== upload-segment timing (CPU generation + table upload, the REND-27/WGPU-16 face) =====
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
    let half = side as f32; // spacing 2 -> city radius ~ side units
    let mut rng = Rng(0x9E37_79B9 ^ count);
    let table: Vec<Instance> = (0..count)
        .map(|i| {
            let x = (i % side) as f32 * 2.0 - half;
            let y = (i / side) as f32 * 2.0 - half;
            let h = 1.0 + rng.next01() * 7.0;
            let g = 0.55 + h / 7.0 * 0.25; // height -> bright band (the gray-blue gradient of the white model)
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
        clip: None,
        edl: None,
        post: Vec::new(),
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

    // ===== frame timing (the first frame = warmup excluding the pipeline's lazy bind-group builds, the mean takes the rest [6b]) =====
    let img = b.render_to_pixels(&frame).expect("warmup");
    let t1 = Instant::now();
    let mut last = img;
    for _ in 0..frames {
        last = b.render_to_pixels(&frame).expect("frame");
        gallery::save_frame(&last, "E601_bench_twin");
    }
    let frame_ms = t1.elapsed().as_secs_f64() * 1000.0 / f64::from(frames);

    // ===== image honesty self-check (non-background ratio -- running through != drawing right, the minimum line, [PIT-8] region statistics) =====
    let lit = last
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] as u32 + p[1] as u32 + p[2] as u32 > 60)
        .count();
    let total = (W * H) as usize;
    assert!(
        lit * 10 > total,
        "the city image's non-background ratio is anomalous {}/{}",
        lit,
        total
    );

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

/// The point-cloud frame-cost tier (the G3 slice, release-only observation and not a gate -- the ruling-D input; no extrapolation, measurements talk).
fn bench_pcl(b: &mut HeadlessBackend, n: u32, frames: u32) {
    let t0 = Instant::now();
    let side = (f64::from(n).sqrt().ceil()) as u32;
    let marks: Vec<PointMark> = (0..n)
        .map(|i| {
            let gx = i % side;
            let gy = i / side;
            PointMark::new(
                [
                    f32::from(gx as i16) * 0.03 - 4.0,
                    f32::from(gy as i16) * 0.03 - 4.0,
                    0.0,
                ],
                [0.9, 0.9, 0.9],
                3.0,
            )
        })
        .collect();
    let table = b
        .create_points(&PointTableDesc { data: &marks })
        .expect("pcl table");
    let upload_ms = t0.elapsed().as_secs_f64() * 1e3;
    let rig = CameraRig::look_at([0.0, 0.0, 12.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(6.0, 6.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [0.0, 0.0, 12.0],
        proj: rig
            .ortho_frame(6.0, W as f32, H as f32, 0.1, 100.0)
            .unwrap(),
        px_world_scale: 2.0 * 6.0 / W as f32,
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0.05, 0.07, 0.10, 1.0],
            },
            DrawCommand::DrawPoints {
                table,
                origin: [0.0; 3],
                transform: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        ],
    };
    let t1 = Instant::now();
    for _ in 0..frames {
        b.render_to_pixels(&frame).expect("pcl render");
    }
    let frame_ms = t1.elapsed().as_secs_f64() * 1e3 / f64::from(frames.max(1));
    println!("RESULT bench_pcl_upload {upload_ms:.3} ms");
    println!("RESULT bench_pcl_frame {frame_ms:.3} ms");
    println!("RESULT bench_pcl_points {n} count");
}
