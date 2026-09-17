//! E201 装载取景像素门（补窗口例无像素断言的历史盲区；BBox 拟合参数的回归锁）。
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

/// BBox 逻辑镜像（正源 examples/rs/src/lib.rs；同包异引用即循环依赖，此处独立复算 + 断言同值）
#[derive(Default)]
struct LocalBox {
    min: [f64; 3],
    max: [f64; 3],
    seen: bool,
}
impl LocalBox {
    fn new() -> Self {
        Self::default()
    }
    fn push(&mut self, p: [f32; 3], w: &[[f64; 4]; 4]) {
        let mut q = [0f64; 3];
        for (j, qj) in q.iter_mut().enumerate() {
            *qj = (p[0] as f64) * w[0][j]
                + (p[1] as f64) * w[1][j]
                + (p[2] as f64) * w[2][j]
                + w[3][j];
        }
        if self.seen {
            for (mn, v) in self.min.iter_mut().zip(q) {
                *mn = (*mn).min(v);
            }
            for (mx, v) in self.max.iter_mut().zip(q) {
                *mx = (*mx).max(v);
            }
        } else {
            self.min = q;
            self.max = q;
            self.seen = true;
        }
    }
    fn center(&self) -> [f64; 3] {
        let c: [f64; 3] = std::array::from_fn(|j| (self.min[j] + self.max[j]) / 2.0);
        c
    }
    fn radius(&self) -> f64 {
        let half: [f64; 3] = std::array::from_fn(|j| (self.max[j] - self.min[j]) / 2.0);
        half.into_iter().fold(0.0f64, f64::max).max(0.5)
    }
}

const W: u32 = 320;
const H: u32 = 240;

#[test]
fn e201_framed_scene_renders_red_and_green() {
    let mut b = HeadlessBackend::new(W, H).expect("adapter");
    let doc = visiaengine_io_gltf::load_gltf(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/twoprim.glb"
    ))
    .unwrap();
    let mut commands = vec![DrawCommand::ClearColor {
        rgba: [0.05, 0.07, 0.10, 1.0],
    }];
    let mut bbox = LocalBox::new(); // 本地副本（examples lib 依赖本 crate，不可反向引用）
    for e in doc.entities() {
        for pt in &e.mesh.positions {
            bbox.push(*pt, &e.world);
        }
        let m = b
            .create_mesh(&MeshDesc {
                positions: &e.mesh.positions,
                normals: &e.mesh.normals,
                indices: &e.mesh.indices,
                uv: &e.mesh.uv,
            })
            .expect("mesh");
        let mat = b.create_material(e.mesh.base_color).expect("mat");
        let origin = [e.world[3][0], e.world[3][1], e.world[3][2]];
        let local = {
            let mut m4 = e.world;
            m4[3] = [0.0, 0.0, 0.0, 1.0];
            m4
        };
        commands.push(DrawCommand::DrawMesh {
            mesh: m,
            material: mat,
            origin,
            transform: local,
        });
    }
    // 镜像 E201 装载期拟合：target=center、dist=r*2.8、pitch≥0.85
    assert_eq!(bbox.center(), [1.5, 0.5, 0.0]);
    assert!((bbox.radius() - 1.5).abs() < 1e-9);
    let rig = CameraRig::orbit(
        bbox.center(),
        0.7,
        0.95f64.max(0.35),
        bbox.radius() * 2.2,
        1.8,
        1.1,
        0.01,
        500.0,
    );
    let aspect = W as f32 / H as f32;
    let (near, far) = ((rig.dist * 0.01) as f32, (rig.dist * 30.0) as f32);
    let proj = rig
        .perspective(rig.fov_y as f32, aspect, near, far)
        .expect("proj");
    let frame = Frame {
        viewport: Viewport::new(W, H, 1.0),
        camera: Camera::perspective(rig.fov_y as f32, aspect, near, far),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        proj,
        px_world_scale: 1.0,
        shadow: None,
        clip: None,
        commands,
    };
    let img = b.render_to_pixels(&frame).expect("render");
    let (mut red, mut green) = (0u32, 0u32);
    for p in img.rgba.as_chunks::<4>().0 {
        let (r, g, bl) = (p[0] as i32, p[1] as i32, p[2] as i32);
        if r - g.max(bl) > 60 {
            red += 1;
        } else if g - r.max(bl) > 60 {
            green += 1;
        }
    }
    println!("red={red} green={green} / {}", W * H);
    // 基线实测 red=884 green=1931（斜视+保守饱和分类器）；阈=基线−40%，专捕回归事故（旧病 red=123）
    assert!(red > 500, "红面应成景 red={red}");
    assert!(green > 1200, "绿面应成景 green={green}");
}
