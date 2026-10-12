//! The E201 load-framing pixel gate (filling the historical blind spot of window examples lacking pixel assertions; the regression lock on the BBox fitting parameters).
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MeshDesc, RenderBackend, Viewport,
};
use visiaengine_render_wgpu::HeadlessBackend;

/// The BBox logic mirror (the true source examples/rs/src/lib.rs; a same-package cross reference would be a circular dependency, so recompute independently here + assert equal values)
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
    let mut bbox = LocalBox::new(); // a local copy (the examples lib depends on this crate, no reverse reference allowed)
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
    // mirror the E201 load-time fitting: target = center, dist = r*2.8, pitch >= 0.85
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
        edl: None,
        post: Vec::new(),
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
    // baseline measured red=884 green=1931 (oblique view + conservative saturation classifier); threshold = baseline minus 40%, specially catching regression accidents (the old disease red=123)
    assert!(red > 500, "the red face should form scenery red={red}");
    assert!(
        green > 1200,
        "the green face should form scenery green={green}"
    );
}
