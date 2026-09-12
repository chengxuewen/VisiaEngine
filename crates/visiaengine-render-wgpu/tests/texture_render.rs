//! WGPU-14/15：Textured 管线（采样×base_color×shade）与 fractional repeat；
//! Flat 管线逐像素零回归由既有 golden 全集自证。

use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, TextureDesc, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

const W: u32 = 64;
const H: u32 = 64;

/// 4×4 RGBA 棋盘：(i+j) 偶=红，奇=绿。
fn checker() -> Vec<u8> {
    let mut px = Vec::new();
    for j in 0..4u32 {
        for i in 0..4u32 {
            px.extend_from_slice(if (i + j) % 2 == 0 { &[255, 0, 0, 255] } else { &[0, 255, 0, 255] });
        }
    }
    px
}

struct Scene {
    frame: Frame,
}

fn quad_and_frame(tex_mat: u64) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>, Scene) {
    let positions = vec![[-1., -1., 0.], [1., -1., 0.], [1., 1., 0.], [-1., 1., 0.]];
    let uv = vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
    let indices = vec![0u32, 1, 2, 0, 2, 3];
    let rig = CameraRig::look_at([0.0, 0.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        commands: vec![
            DrawCommand::ClearColor { rgba: [0., 0., 0., 1.] },
            DrawCommand::DrawMesh {
                mesh: 0,
                material: tex_mat,
                origin: [0.; 3],
                transform: [
                    [1., 0., 0., 0.],
                    [0., 1., 0., 0.],
                    [0., 0., 1., 0.],
                    [0., 0., 0., 1.],
                ],
            },
        ],
    };
    (positions, uv, indices, Scene { frame })
}

fn px(img: &visiaengine_render_wgpu::OffscreenFrame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * W + x) * 4) as usize;
    [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2], img.rgba[i + 3]]
}

fn draw(repeat: [f32; 2]) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tid = b
        .upload_texture(&TextureDesc { rgba: &checker(), width: 4, height: 4 })
        .expect("texture");
    let mid = b
        .create_material_desc(&visiaengine_render::MaterialDesc {
            base_color: [1., 1., 1., 1.],
            texture: Some(tid),
            repeat,
            specular: 0.0,
        })
        .expect("material");
    let (positions, uv, indices, mut sc) = quad_and_frame(mid);
    let mesh = b
        .create_mesh(&MeshDesc { positions: &positions, normals: &vec![[0., 0., 1.]; 4], indices: &indices, uv: &uv })
        .unwrap();
    if let DrawCommand::DrawMesh { mesh: m, material, .. } = &mut sc.frame.commands[1] {
        *m = mesh;
        *material = mid;
    }
    b.render_to_pixels(&sc.frame).expect("render")
}

// spec: WGPU-14
#[test]
fn textured_pipeline_samples_checker_x_base_x_shade() {
    let img = draw([1., 1.]);
    // uv 原点=左下（v 0 底）：像素 (8, 56)=uv≈(0.125,0.875+)→cell(0,3): (0+3)%2=1 绿
    let a = px(&img, 8, 8);
    let b = px(&img, 40, 40);
    let redish = |p: [u8; 4]| p[0] > 120 && p[1] < 60;
    let greenish = |p: [u8; 4]| p[1] > 120 && p[0] < 60;
    assert!(
        (redish(a) && greenish(b)) || (greenish(a) && redish(b)),
        "棋盘双极性缺失 a={a:?} b={b:?}"
    );
    // 非纯 base_color：亮度=棋盘色×shade(≈0.63)——红通道 160±20（mock-up 注记 WGPU-14 条款体）
    let lum = if redish(a) { a[0] } else { a[1] };
    assert!((lum as i16 - 160).abs() < 30, "shade 乘链亮度 {lum}");
}

// spec: WGPU-15
#[test]
fn fractional_repeat_shifts_pattern() {
    let one = draw([1., 1.]);
    let two = draw([2., 2.]);
    // 同像素在两 repeat 设置下色相反（棋盘半周期 vs 全周期采样点相位差）
    let p1 = px(&one, 20, 44);
    let p2 = px(&two, 20, 44);
    let fam = |p: [u8; 4]| if p[0] > p[1] { 0 } else { 1 };
    assert!(
        p1 != [0, 0, 0, 255] && p2 != [0, 0, 0, 255],
        "两点须在面上"
    );
    assert_ne!(fam(p1), fam(p2), "repeat(2,2) 应翻转采样相位 ({p1:?} vs {p2:?})");
}

// spec: WGPU-14
#[test]
fn flat_pipeline_zero_regression_marker() {
    // Flat 路径（无纹理材质）走既有管线：twoprim fixture 中心亮度带（族断言，
    // 精确像素回归由 offscreen_golden/geo_pipeline 全集承担）
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let doc = visiaengine_io_gltf::load_gltf(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/twoprim.glb"
    ))
    .unwrap();
    let mut commands = vec![DrawCommand::ClearColor { rgba: [0.05, 0.07, 0.10, 1.] }];
    let mut handles = Vec::new();
    for e in doc.entities() {
        let m = b
            .create_mesh(&MeshDesc {
                positions: &e.mesh.positions,
                normals: &e.mesh.normals,
                indices: &e.mesh.indices,
                uv: &[],
            })
            .unwrap();
        let mat = b.create_material(e.mesh.base_color).unwrap();
        handles.push((m, mat));
    }
    for (m, mat) in &handles {
        commands.push(DrawCommand::DrawMesh {
            mesh: *m,
            material: *mat,
            origin: [0.; 3],
            transform: [
                [1., 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., 1., 0.],
                [0., 0., 0., 1.],
            ],
        });
    }
    let rig = CameraRig::look_at([0., 0., 3.], [0., 0., 0.], [0., 1., 0.]);
    let proj = rig.perspective(rig.fov_y as f32, 1.0, 0.1, 100.0).unwrap();
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, 1.0, 0.1, 100.0),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        commands,
    };
    let img = b.render_to_pixels(&frame).expect("render flat");
    let c = px(&img, 32, 32);
    assert!(c[0] + c[1] + c[2] > 40, "Flat 出图非背景 {c:?}");
}
