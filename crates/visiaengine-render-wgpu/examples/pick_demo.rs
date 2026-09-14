//! E401 pick_demo：屏幕射线→pick_meshes→高亮覆写 全链演示（headless 形态，
//! 零窗口依赖 → 本机/CI 直跑）。场景=双箱堆叠，拾取中心=视点近侧件。
//! 用法：cargo run --example pick_demo -- [--frames N]

use visiaengine_core::Scene;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render::{MeshCandidate, pick_meshes, screen_to_ray_ortho, screen_to_ray_persp};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 256; // readback 行对齐（1024B=4×256）
const H: u32 = 256;

#[allow(clippy::type_complexity)]
fn cube() -> (Vec<[f32; 3]>, Vec<u32>) {
    let positions = vec![
        [-1., -1., -1.],
        [1., -1., -1.],
        [1., 1., -1.],
        [-1., 1., -1.],
        [-1., -1., 1.],
        [1., -1., 1.],
        [1., 1., 1.],
        [-1., 1., 1.],
    ];
    let idx: &[&[u32]] = &[
        &[0, 2, 1],
        &[0, 3, 2],
        &[4, 5, 6],
        &[4, 6, 7],
        &[0, 1, 5],
        &[0, 5, 4],
        &[2, 3, 7],
        &[2, 7, 6],
        &[1, 2, 6],
        &[1, 6, 5],
        &[3, 0, 4],
        &[3, 4, 7],
    ];
    (positions, idx.concat())
}

fn main() {
    let mut frames = 3u32;
    let mut bench = false;
    let mut args = std::env::args();
    while let Some(a) = args.next() {
        if a == "--frames"
            && let Some(n) = args.next()
        {
            frames = n.parse().unwrap_or(1);
        } else if a == "--bench" {
            bench = true;
        }
    }
    if bench {
        bench_pick_10k();
        return;
    }
    let Some(mut backend) = HeadlessBackend::new(W, H) else {
        eprintln!("ERROR: no adapter");
        std::process::exit(2);
    };
    let mut scene = Scene::new();
    let near = scene.spawn();
    let far = scene.spawn();
    let (pos, idx) = cube();
    let rig = CameraRig::look_at([4.0, 0.0, 20.0], [4.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let origin = [4.0, 0.0, 0.0];

    // 拾取：中心射线命中视点近侧件（z 遮挡序与 draw 序无关，WGPU-13）
    let ray = screen_to_ray_ortho(&rig, W as f32 / 2.0, H as f32 / 2.0, W as f32, H as f32)
        .expect("center ray");
    // 拾取候选=完整世界系（无 origin 概念）；渲染 transform=**层内局部**
    // （世界级 x=4 已由 origin=(4,0,0) 承担——双份烘移=离轴 64px，本行教训锁注释）
    let world_near = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [4.0, 0.0, 5.0, 1.0],
    ];
    let world_far = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [4.0, 0.0, 1.0, 1.0],
    ];
    let trz = |dz: f64| {
        [
            [1.0f64, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, dz, 1.0],
        ]
    };
    let cands = [
        MeshCandidate {
            entity: near,
            positions: &pos,
            indices: &idx,
            world: &world_near,
        },
        MeshCandidate {
            entity: far,
            positions: &pos,
            indices: &idx,
            world: &world_far,
        },
    ];
    let hit = pick_meshes(ray, &cands).expect("center must hit a cube");
    // 选择态：CPU 侧查色（高亮=纯色 overlay，[E3D:A5] 渲染侧最小子集）
    let highlight = [1.0, 0.83, 0.29, 1.0];
    let base = [0.0, 0.45, 1.0, 1.0];
    println!(
        "PICK t={:.1} tri={} selected_slot={}",
        hit.t,
        hit.triangle,
        hit.entity.slot()
    );

    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    for (entity, transform) in [(near, trz(5.0)), (far, trz(1.0))] {
        let m = backend
            .create_mesh(&MeshDesc {
                positions: &pos,
                normals: &vec![[0.0, 0.0, 1.0]; pos.len()],
                indices: &idx,
                uv: &[],
            })
            .expect("mesh");
        let selected = hit.entity == entity;
        let mat = backend
            .create_material(if selected { highlight } else { base })
            .expect("mat");
        commands.push(DrawCommand::DrawMesh {
            mesh: m,
            material: mat,
            origin,
            transform,
        });
    }
    let proj = rig
        .ortho_frame(8.0, W as f32, H as f32, 0.1, 100.0)
        .expect("proj");
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(8.0, 8.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [4.0, 0.0, 20.0],
        proj,
        px_world_scale: 1.0,
        commands,
    };
    for f in 0..frames {
        let img = backend.render_to_pixels(&frame).expect("render");
        let i = ((H / 2 * W + W / 2) * 4) as usize;
        println!(
            "FRAME {f} center=({},{},{}) hit_entity_slot={}",
            img.rgba[i],
            img.rgba[i + 1],
            img.rgba[i + 2],
            hit.entity.slot()
        );
    }
    println!("OK pick_demo");
}

/// [6b/3.5b 欠账结账] 10k 候选拾取压力一行：透视射线 ×100，
/// 全扫+即算 AABB 剪枝（CORE-14/15 路，scan-first 纪律的 bench 证据面）。
/// `RESULT bench_pick_10k <ms> ms`=每射线均值。
fn bench_pick_10k() {
    use std::time::Instant;
    let (pos, idx) = cube();
    let (side, step) = (100usize, 1.0f64);
    assert_eq!(side * side, 10_000, "候选数=10k 面值");
    let worlds: Vec<[[f64; 4]; 4]> = (0..side)
        .flat_map(|gy| (0..side).map(move |gx| [gx as f64 * step, gy as f64 * step, 0.0]))
        .map(|t| {
            [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [t[0], t[1], t[2], 1.0],
            ]
        })
        .collect();
    let mut scene = Scene::new();
    let ents: Vec<visiaengine_core::EntityId> = (0..10_000).map(|_| scene.spawn()).collect();
    let cands: Vec<MeshCandidate<'_>> = ents
        .iter()
        .zip(&worlds)
        .map(|(e, w)| MeshCandidate {
            entity: *e,
            positions: &pos,
            indices: &idx,
            world: w,
        })
        .collect();
    let rig = CameraRig::look_at([50.0, -80.0, 60.0], [50.0, 50.0, 0.0], [0.0, 1.0, 0.0]);
    let rays: Vec<_> = (0..100u32)
        .filter_map(|i| screen_to_ray_persp(&rig, 20.0 + i as f32 * 2.5, 128.0, W as f32, H as f32))
        .collect();
    assert!(!rays.is_empty(), "射线生成");
    let t0 = Instant::now();
    let hits: usize = rays
        .iter()
        .map(|&r| pick_meshes(r, &cands).is_some() as usize)
        .sum();
    // 命中自检：全漏=bench 只测剪枝路径，掩盖三角面成本面（诚实性锁）
    assert!(
        hits * 2 > rays.len(),
        "射线族命中过少 {hits}/{}",
        rays.len()
    );
    let ms = t0.elapsed().as_secs_f64() * 1000.0 / rays.len() as f64;
    println!("RESULT bench_pick_10k {ms:.3} ms");
    println!("ok bench_pick_10k rays={} hits={}", rays.len(), hits);
}
