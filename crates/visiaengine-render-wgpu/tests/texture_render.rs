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
            px.extend_from_slice(if (i + j) % 2 == 0 {
                &[255, 0, 0, 255]
            } else {
                &[0, 255, 0, 255]
            });
        }
    }
    px
}

struct Scene {
    frame: Frame,
}

struct QuadMesh {
    positions: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

fn quad_and_frame(tex_mat: u64) -> (QuadMesh, Scene) {
    // ±2.5：铺满 64px 视口（z=3/fov60 下 ±1 只占半屏，采样窗会脱面）
    let positions = vec![
        [-2.5, -2.5, 0.],
        [2.5, -2.5, 0.],
        [2.5, 2.5, 0.],
        [-2.5, 2.5, 0.],
    ];
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
        px_world_scale: 1.0,
        shadow: None,
        commands: vec![
            DrawCommand::ClearColor {
                rgba: [0., 0., 0., 1.],
            },
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
    (
        QuadMesh {
            positions,
            uv,
            indices,
        },
        Scene { frame },
    )
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

fn draw(repeat: [f32; 2]) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tid = b
        .upload_texture(&TextureDesc {
            rgba: &checker(),
            width: 4,
            height: 4,
        })
        .expect("texture");
    let mid = b
        .create_material_desc(&visiaengine_render::MaterialDesc {
            base_color: [1., 1., 1., 1.],
            texture: Some(tid),
            repeat,
            specular: 0.0,
        })
        .expect("material");
    let (q, mut sc) = quad_and_frame(mid);
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &q.positions,
            normals: &[[0., 0., 1.]; 4],
            indices: &q.indices,
            uv: &q.uv,
        })
        .unwrap();
    if let DrawCommand::DrawMesh {
        mesh: m, material, ..
    } = &mut sc.frame.commands[1]
    {
        *m = mesh;
        *material = mid;
    }
    b.render_to_pixels(&sc.frame).expect("render")
}

/// 64×64 水平渐变纹理场景（R=4i，G=4j）——WGPU-15 回绕 oracle。
fn draw_grad(repeat: [f32; 2]) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut grad = Vec::new();
    for j in 0..64u32 {
        for i in 0..64u32 {
            grad.extend_from_slice(&[(i * 4) as u8, (j * 4) as u8, 0, 255]);
        }
    }
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let tid = b
        .upload_texture(&TextureDesc {
            rgba: &grad,
            width: 64,
            height: 64,
        })
        .expect("texture");
    let mid = b
        .create_material_desc(&visiaengine_render::MaterialDesc {
            base_color: [1., 1., 1., 1.],
            texture: Some(tid),
            repeat,
            specular: 0.0,
        })
        .expect("material");
    let (q, mut sc) = quad_and_frame(mid);
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &q.positions,
            normals: &[[0., 0., 1.]; 4],
            indices: &q.indices,
            uv: &q.uv,
        })
        .unwrap();
    if let DrawCommand::DrawMesh {
        mesh: m, material, ..
    } = &mut sc.frame.commands[1]
    {
        *m = mesh;
        *material = mid;
    }
    b.render_to_pixels(&sc.frame).expect("render")
}

fn draw_flat_mat(specular: f32) -> visiaengine_render_wgpu::OffscreenFrame {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let mid = b
        .create_material_desc(&visiaengine_render::MaterialDesc {
            base_color: [1., 1., 1., 1.],
            texture: None,
            repeat: [1., 1.],
            specular,
        })
        .expect("material");
    let (q, mut sc) = quad_and_frame(mid);
    let mesh = b
        .create_mesh(&MeshDesc {
            positions: &q.positions,
            normals: &[[0., 0., 1.]; 4],
            indices: &q.indices,
            uv: &q.uv,
        })
        .unwrap();
    if let DrawCommand::DrawMesh {
        mesh: m, material, ..
    } = &mut sc.frame.commands[1]
    {
        *m = mesh;
        *material = mid;
    }
    b.render_to_pixels(&sc.frame).expect("render")
}

// spec: WGPU-14
#[test]
fn textured_pipeline_samples_checker_x_base_x_shade() {
    let img = draw([1., 1.]);
    // 中心区扫描（避开四边形边缘）：棋盘双极性各自成片，亮度带 140..200
    // 验证 shade(≈0.62)×base(1.0)×texel(255) 乘法链（=160±40，拒绝未乘 shade 的 255）
    let (mut reds, mut greens) = (0u32, 0u32);
    for y in 20..44 {
        for x in 20..44 {
            let p = px(&img, x, y);
            assert!(p[0] < 200 && p[1] < 200, "shade 未乘链出 255 {p:?}");
            if p[0] > 130 && p[1] < 70 {
                reds += 1;
            } else if p[1] > 130 && p[0] < 70 {
                greens += 1;
            }
        }
    }
    assert!(
        reds >= 15 && greens >= 15,
        "棋盘双极性缺失 red={reds} green={greens}"
    );
}

// spec: WGPU-15
#[test]
fn fractional_repeat_shifts_pattern() {
    // 渐变纹理（R=4i 水平斜坡）是 repeat 的精确 oracle：repeat=1 一行内无回绕，
    // repeat=2 采样 w=2u 中途恰好一次 wrap 断崖（棋盘奇偶断言在线性滤波下不成立，弃）
    let one = draw_grad([1., 1.]);
    let two = draw_grad([2., 2.]);
    let wraps = |img: &visiaengine_render_wgpu::OffscreenFrame| -> u32 {
        (1..64u32)
            .filter(|&x| px(img, x - 1, 32)[0] as i16 - px(img, x, 32)[0] as i16 > 100)
            .count() as u32
    };
    assert_eq!(wraps(&one), 0, "repeat=1 斜坡不应回绕");
    assert_eq!(wraps(&two), 1, "repeat=2 应恰一次回绕");
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
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.],
    }];
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
        px_world_scale: 1.0,
        shadow: None,
        commands,
    };
    let img = b.render_to_pixels(&frame).expect("render flat");
    let c = px(&img, 32, 32);
    assert!(c[0] + c[1] + c[2] > 40, "Flat 出图非背景 {c:?}");
}

// spec: REND-25
#[test]
fn meshdesc_uv_optional_contract() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let pos = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let nrm = [[0.0, 0.0, 1.0]; 3];
    let idx = [0u32, 1, 2];
    // 空片=合法（Flat 存量形态：整 mesh uv 零填充）
    assert!(
        b.create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[]
        })
        .is_ok()
    );
    // 非空且长度≠顶点数=拒绝（不静默补齐）
    assert!(
        b.create_mesh(&MeshDesc {
            positions: &pos,
            normals: &nrm,
            indices: &idx,
            uv: &[[0.0, 0.0]]
        })
        .is_err()
    );
}

// spec: WGPU-14
#[test]
fn material_unknown_texture_is_err() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let r = b.create_material_desc(&visiaengine_render::MaterialDesc {
        base_color: [1.0; 4],
        texture: Some(999),
        repeat: [1.0, 1.0],
        specular: 0.0,
    });
    assert!(r.is_err(), "未知纹理引用不得静默降 Flat");
}

// spec: WGPU-14
#[test]
fn specular_mockup_boosts_lambert() {
    // mock-up [4ab①]：specular 参与 Lambert 亮度系数（非 GGX）。
    // 白基色无纹理平面：spec=0.8 的中心区最大值应比 spec=0 亮 ≥15%。
    let bright = draw_flat_mat(0.8);
    let plain = draw_flat_mat(0.0);
    let mx = |img: &visiaengine_render_wgpu::OffscreenFrame| -> u32 {
        (20..44)
            .map(|y| (20..44).map(|x| px(img, x, y)[0] as u32).max().unwrap_or(0))
            .max()
            .unwrap_or(0)
    };
    let (b, p) = (mx(&bright), mx(&plain));
    assert!(p > 60, "base 亮度 {p}");
    assert!(b * 100 > p * 115, "mock-up 未生效 {b} vs {p}");
}
