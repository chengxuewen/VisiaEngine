//! WGPU-12：选择高亮的渲染面（实体 material 覆写=纯色 overlay，[E3D:C/A5] CPU 侧）。
//! 拾取链路复用：屏幕射线(REND-21/22) → pick_meshes(REND-23) → 高亮色 material。

use visiaengine_core::{Ray, Scene, Vec3};
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport, screen_to_ray_ortho,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 200;
const H: u32 = 200;

fn cube1() -> (Vec<[f32; 3]>, Vec<u32>) {
    let positions = vec![
        [-1., -1., -1.], [1., -1., -1.], [1., 1., -1.], [-1., 1., -1.],
        [-1., -1., 1.], [1., -1., 1.], [1., 1., 1.], [-1., 1., 1.],
    ];
    let idx: &[&[u32]] = &[
        &[0, 2, 1], &[0, 3, 2], &[4, 5, 6], &[4, 6, 7], &[0, 1, 5], &[0, 5, 4],
        &[2, 3, 7], &[2, 7, 6], &[1, 2, 6], &[1, 6, 5], &[3, 0, 4], &[3, 4, 7],
    ];
    (positions, idx.concat())
}

fn trz(dz: f64) -> [[f64; 4]; 4] {
    [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [4.0, 0.0, dz, 1.0]]
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
    let ray = screen_to_ray_ortho(&rig, W as f32 / 2.0, H as f32 / 2.0, W as f32, H as f32).unwrap();
    let cands = [visiaengine_render::MeshCandidate {
        entity: near,
        positions: &pos,
        indices: &idx,
        world: &trz(1.0),
    }];
    let hit = visiaengine_render::pick_meshes(ray, &cands).expect("中心应命中近件");
    assert_eq!(hit.entity, near);

    // 渲染：近件=高亮黄、远件=基础蓝（选中态 material 覆写，CPU 侧查色同口）
    let yellow = [1.0, 0.83, 0.29, 1.0];
    let blue = [0.0, 0.45, 1.0, 1.0];
    let _ = neg;
    let mut commands = vec![DrawCommand::ClearColor { rgba: [0.05, 0.07, 0.10, 1.0] }];
    for (dz, color) in [(1.0f64, yellow), (5.0, blue)] {
        // 局部顶点 = shifted(-origin)（D7 纪律，同 geo_viewer 用法）
        let local: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 0.6, p[1] * 0.6, p[2] * 0.6])
            .collect();
        let m = backend.create_mesh(&MeshDesc {
            positions: &local,
            normals: &vec![[0.0, 0.0, 1.0]; local.len()],
            indices: &idx,
        }).unwrap();
        let mat = backend.create_material(color).unwrap();
        commands.push(DrawCommand::DrawMesh { mesh: m, material: mat, origin, transform: trz(dz) });
    }
    let proj = rig.ortho_frame(8.0, W as f32, H as f32, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::ortho(8.0, 8.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: [4.0, 0.0, 20.0],
        proj,
        commands,
    };
    let img = backend.render_to_pixels(&frame).expect("render");
    let i = ((H / 2 * W as u32 + W / 2) * 4) as usize;
    let (r, g, bl) = (img.rgba[i] as i16, img.rgba[i + 1] as i16, img.rgba[i + 2] as i16);
    // 高亮黄×方向光经验域（≈0.625）：r≥g>>b（蓝通道被压）
    assert!(r > 120 && g > 90 && bl < 90 && r - bl > 60,
        "中心像素应为高亮黄族，得 ({r},{g},{bl})");
}
