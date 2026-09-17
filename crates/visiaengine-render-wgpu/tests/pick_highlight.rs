//! WGPU-12：选择高亮的渲染面（实体 material 覆写=纯色 overlay，[E3D:C/A5] CPU 侧）。
//! 拾取链路复用：屏幕射线(REND-21/22) → pick_meshes(REND-23) → 高亮色 material。

use visiaengine_core::Scene;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport, screen_to_ray_ortho,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 256; // readback 行 1024B=4×256 对齐（COPY_BYTES_PER_ROW_ALIGNMENT）
const H: u32 = 256;

fn cube1() -> (Vec<[f32; 3]>, Vec<u32>) {
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

/// 拾取候选用**完整世界矩阵**（pick 无 origin 概念，射线亦世界系）。
fn world4(dz: f64) -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [4.0, 0.0, dz, 1.0],
    ]
}

/// 渲染用**纯局部 transform**（层内 z 偏移）——世界 x=4 归 origin（D7：
/// transform 禁烘世界级平移，否则 rebase 不消——本文件踩过后锁进矩阵语义）。
fn trz(dz: f64) -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, dz, 1.0],
    ]
}

// spec: WGPU-12
#[test]
fn selected_entity_renders_highlight_overlay() {
    let Some(mut backend) = HeadlessBackend::new(W, H) else {
        eprintln!("SKIP: no adapter");
        return;
    };
    let (pos, idx) = cube1();
    let mut scene = Scene::new();
    let near = scene.spawn();
    let _far = scene.spawn();
    let origin = [4.0, 0.0, 0.0];
    let neg = [-origin[0], -origin[1]];

    // 拾取链：中心射线 → 命中近件（x=4 列）
    let rig = CameraRig::look_at([4.0, 0.0, 20.0], [4.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let ray =
        screen_to_ray_ortho(&rig, W as f32 / 2.0, H as f32 / 2.0, W as f32, H as f32).unwrap();
    let cands = [visiaengine_render::MeshCandidate {
        entity: near,
        positions: &pos,
        indices: &idx,
        world: &world4(1.0),
    }];
    let hit = visiaengine_render::pick_meshes(ray, &cands).expect("中心应命中近件");
    assert_eq!(hit.entity, near);

    // 渲染：视点近侧件（z=5，俯视先见）=高亮黄；下层=基础蓝。选中态 material 覆写。
    let yellow = [1.0, 0.83, 0.29, 1.0];
    let blue = [0.0, 0.45, 1.0, 1.0];
    let _ = neg;
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    for (dz, color) in [(5.0f64, yellow), (1.0, blue)] {
        // 局部顶点 = shifted(-origin)（D7 纪律，同 geo_viewer 用法）
        let local: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 0.6, p[1] * 0.6, p[2] * 0.6])
            .collect();
        let m = backend
            .create_mesh(&MeshDesc {
                positions: &local,
                normals: &vec![[0.0, 0.0, 1.0]; local.len()],
                indices: &idx,
                uv: &[],
            })
            .unwrap();
        let mat = backend.create_material(color).unwrap();
        commands.push(DrawCommand::DrawMesh {
            mesh: m,
            material: mat,
            origin,
            transform: trz(dz),
        });
    }
    let proj = rig
        .ortho_frame(8.0, W as f32, H as f32, 0.1, 100.0)
        .unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(8.0, 8.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [4.0, 0.0, 20.0],
        proj,
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        commands,
    };
    let img = backend.render_to_pixels(&frame).expect("render");
    let i = ((H / 2 * W + W / 2) * 4) as usize;
    let (r, g, bl) = (
        img.rgba[i] as i16,
        img.rgba[i + 1] as i16,
        img.rgba[i + 2] as i16,
    );
    // 高亮黄×方向光经验域（≈0.625）：r≥g>>b（蓝通道被压）
    assert!(
        r > 120 && g > 90 && bl < 90 && r - bl > 60,
        "中心像素应为高亮黄族，得 ({r},{g},{bl})"
    );
}

// spec: WGPU-13
#[test]
fn occlusion_independent_of_draw_order() {
    // 同堆叠场景、draw 序反转（远侧蓝先/近侧黄后 与 反之）→ 中心像素恒=近侧黄
    // （深度遮挡与顺序无关——若管线缺深度，后 draw 者恒赢=本断言必红）
    let yellow = [1.0, 0.83, 0.29, 1.0];
    let blue = [0.0, 0.45, 1.0, 1.0];
    let mut centers = Vec::new();
    for order in [0u8, 1] {
        let Some(mut backend) = HeadlessBackend::new(W, H) else {
            eprintln!("SKIP: no adapter");
            return;
        };
        let (pos, idx) = cube1();
        let origin = [4.0, 0.0, 0.0];
        let mut commands = vec![DrawCommand::ClearColor {
            rgba: [0.05, 0.07, 0.10, 1.0],
        }];
        let slots = match order {
            0 => [(1.0f64, blue), (5.0, yellow)],
            _ => [(5.0f64, yellow), (1.0, blue)],
        };
        for (dz, color) in slots {
            let m = backend
                .create_mesh(&MeshDesc {
                    positions: &pos,
                    normals: &vec![[0.0, 0.0, 1.0]; pos.len()],
                    indices: &idx,
                    uv: &[],
                })
                .unwrap();
            let mat = backend.create_material(color).unwrap();
            commands.push(DrawCommand::DrawMesh {
                mesh: m,
                material: mat,
                origin,
                transform: trz(dz),
            });
        }
        let rig = CameraRig::look_at([4.0, 0.0, 20.0], [4.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let proj = rig
            .ortho_frame(8.0, W as f32, H as f32, 0.1, 100.0)
            .unwrap();
        let frame = Frame {
            viewport: Viewport::new(W, H, 1.0),
            camera: Camera::ortho(8.0, 8.0, 0.1, 100.0),
            view_rot: rig.view_rotation(),
            eye: [4.0, 0.0, 20.0],
            proj,
            px_world_scale: 1.0,
            shadow: None,
            clip: None,
            commands,
        };
        let img = backend.render_to_pixels(&frame).expect("render");
        let i = ((H / 2 * W + W / 2) * 4) as usize;
        centers.push((img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]));
    }
    for (idx, c) in centers.iter().enumerate() {
        assert!(
            c.0 > 120 && c.1 > 90 && c.2 < 90,
            "draw 序 {idx} 中心应=近侧黄，得 {c:?}"
        );
    }
}

// spec: WGPU-12
/// E402 窗口路镜像像素门（2026-09-17 hover/多选窗随带；testing.md 视觉例双保险第 1 条）。
/// rig/placement/调色逐字镜像 examples/rs/E402_pick_interactive.rs——窗口与门共用数学。
/// 谓词三自证：①未选中心=基色族；②选中中心=高亮黄族（WGPU-12 域）；③角部射线必不命中
/// （canary——射线/位形解错时"全命中"或"全不命中"的巧合绿被此断钉死）。
#[test]
fn golden_pick_window_mirror() {
    const PW: u32 = 512; // 16:10 窗口形镜像（E402 960×600 同比例；512=行对齐倍数）
    const PH: u32 = 320;
    let mut scene = Scene::new();
    let ents: Vec<_> = (0..3).map(|_| scene.spawn()).collect();
    let (pos, nrm, idx) = visiaengine_render_wgpu::unit_box_mesh();
    let placed = |x: f64, y: f64, z: f64, s: f64| {
        [
            [s, 0.0, 0.0, 0.0],
            [0.0, s, 0.0, 0.0],
            [0.0, 0.0, s, 0.0],
            [x, y, z, 1.0],
        ]
    };
    let worlds = [
        placed(-3.2, 0.0, 1.1, 2.2),
        placed(0.0, 0.6, 1.9, 2.2),
        placed(3.2, -0.4, 1.4, 2.2),
    ];
    let palette = [
        [0.90f32, 0.25, 0.25, 1.0],
        [0.20, 0.75, 0.35, 1.0],
        [0.20, 0.45, 0.90, 1.0],
    ];
    let yellow = [1.0f32, 0.83, 0.29, 1.0];
    // E402 main() 机位逐字（dist=11，fov_y=1.0rad）
    let rig = CameraRig::orbit([0.0, 0.0, 1.5], 0.35, 0.95, 11.0, 1.0, 1.0, 0.1, 100.0);
    let Some(mut backend) = HeadlessBackend::new(PW, PH) else {
        eprintln!("SKIP: no adapter");
        return;
    };
    let mut mats = Vec::new();
    for c in palette.iter().chain(std::iter::once(&yellow)) {
        mats.push(backend.create_material(*c).unwrap());
    }
    let mesh = backend
        .create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[],
        })
        .unwrap();
    let render = |be: &mut HeadlessBackend, sel: Option<usize>| {
        let mut commands = vec![DrawCommand::ClearColor {
            rgba: [0.05, 0.07, 0.10, 1.0],
        }];
        for i in 0..3 {
            let material = if sel == Some(i) { mats[3] } else { mats[i] };
            commands.push(DrawCommand::DrawMesh {
                mesh,
                material,
                origin: [0.0, 0.0, 0.0],
                transform: worlds[i],
            });
        }
        let near = (rig.dist * 0.01) as f32;
        let far = (rig.dist * 30.0) as f32;
        let frame = Frame {
            viewport: Viewport::new(PW, PH, 1.0),
            camera: Camera::perspective(rig.fov_y as f32, PW as f32 / PH as f32, near, far),
            view_rot: rig.view_rotation(),
            eye: rig.eye(),
            proj: rig
                .perspective(rig.fov_y as f32, PW as f32 / PH as f32, near, far)
                .unwrap(),
            px_world_scale: 1.0,
            shadow: None,
            clip: None,
            commands,
        };
        be.render_to_pixels(&frame).expect("render")
    };
    let center = |img: &visiaengine_render_wgpu::OffscreenFrame| {
        let i = ((PH / 2 * PW + PW / 2) as usize) * 4;
        [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]]
    };

    // ① 中心射线命中的盒子（E402 点选同款链）
    let ray = visiaengine_render::screen_to_ray_persp(
        &rig,
        PW as f32 / 2.0,
        PH as f32 / 2.0,
        PW as f32,
        PH as f32,
    )
    .expect("center ray");
    let cands: Vec<_> = ents
        .iter()
        .zip(&worlds)
        .map(|(e, w)| visiaengine_render::MeshCandidate {
            entity: *e,
            positions: &pos,
            indices: &idx,
            world: w,
        })
        .collect();
    let hit = visiaengine_render::pick_meshes(ray, &cands).expect("中心应命中一盒");
    let hit_i = ents.iter().position(|e| *e == hit.entity).unwrap();
    println!("PROBE center hits box #{hit_i}");

    // ② 未选态中心 = 该盒基色族（绿族 g 占优 for #1，红族 r 占优 for #0/#2 侧箱）
    let img0 = render(&mut backend, None);
    let [r0, g0, b0] = center(&img0);
    println!("PROBE unselected center=({r0},{g0},{b0})");
    let base = palette[hit_i];
    let dom = if base[0] > base[1] && base[0] > base[2] {
        0
    } else if base[1] > base[2] {
        1
    } else {
        2
    };
    let ch = [r0, g0, b0];
    assert!(
        ch[dom] > 60 && ch[dom] > ch[(dom + 1) % 3] && ch[dom] > ch[(dom + 2) % 3],
        "未选中心应为盒 #{hit_i} 基色族，得 ({r0},{g0},{b0})"
    );

    // ③ 选中态中心 = 高亮黄族（谓词承 WGPU-12 实证域）
    let img1 = render(&mut backend, Some(hit_i));
    let [r1, g1, b1] = center(&img1);
    println!("PROBE selected center=({r1},{g1},{b1})");
    assert!(
        r1 > 120 && g1 > 90 && b1 < 90 && r1 - b1 > 60,
        "选中中心应为高亮黄族，得 ({r1},{g1},{b1})"
    );

    // ④ canary：右上外角射线必不命中（钉死"巧合命中一切"）
    let ray_corner =
        visiaengine_render::screen_to_ray_persp(&rig, PW as f32 - 2.0, 2.0, PW as f32, PH as f32)
            .expect("corner ray");
    assert!(
        visiaengine_render::pick_meshes(ray_corner, &cands).is_none(),
        "外角射线意外命中——射线域失控"
    );
}
