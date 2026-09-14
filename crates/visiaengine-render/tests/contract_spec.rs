//! visiaengine-render 契约测试（仅公开 API；// spec: 标签入双向追溯门禁）。

use visiaengine_render::{
    BackendError, Camera, Capability, DrawCommand, Frame, MaterialDesc, MaterialId, MeshDesc,
    MeshId, RenderBackend, TextureDesc, Viewport,
};

const IDENTITY4: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

struct Stub {
    meshes: MeshId,
    materials: MaterialId,
}

impl RenderBackend for Stub {
    fn name(&self) -> &'static str {
        "stub"
    }
    fn supports(&self, _capability: Capability) -> bool {
        false
    }
    fn resize(&mut self, _viewport: Viewport) {}
    fn render(&mut self, frame: &Frame) {
        for cmd in &frame.commands {
            // 穷举消费面：IR 加变体时此处编译失败=契约同步器
            match cmd {
                DrawCommand::ClearColor { .. } | DrawCommand::DrawMesh { .. } => {}
            }
        }
    }
    fn create_mesh(&mut self, _desc: &MeshDesc) -> Result<MeshId, BackendError> {
        self.meshes += 1;
        Ok(self.meshes)
    }
    fn create_material(&mut self, _base_color: [f32; 4]) -> Result<MaterialId, BackendError> {
        self.materials += 1;
        Ok(self.materials)
    }
}

// spec: REND-01
#[test]
fn backend_trait_object_safe() {
    let b: Box<dyn RenderBackend> = Box::new(Stub {
        meshes: 0,
        materials: 0,
    });
    assert!(!b.name().is_empty());
}

// spec: REND-02
#[test]
fn stub_impl_without_wgpu() {
    // 本文件即证明：实现 RenderBackend 无需任何后端 crate（不变式②契约面纯度）
    let mut b = Stub {
        meshes: 0,
        materials: 0,
    };
    let vp = Viewport::new(64, 64, 1.0);
    b.resize(vp);
    let frame = Frame {
        viewport: vp,
        camera: Camera::perspective(1.0, 1.0, 0.1, 1000.0),
        view_rot: IDENTITY4,
        eye: [0.0; 3],
        proj: IDENTITY4,
        commands: vec![DrawCommand::ClearColor {
            rgba: [0.05, 0.07, 0.1, 1.0],
        }],
    };
    b.render(&frame); // 不 panic
    assert!(!b.supports(Capability::LineStrip)); // stub 能力全否，行为可预期
}

// spec: REND-03
#[test]
fn ir_variants_exhaustive_construct() {
    let mesh: MeshId = 7;
    let cmds = vec![
        DrawCommand::ClearColor { rgba: [0.0; 4] },
        DrawCommand::DrawMesh {
            mesh,
            material: 3,
            origin: [0.0; 3],
            transform: [
                [1.0f64, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        },
    ];
    let kinds: Vec<&'static str> = cmds.iter().map(DrawCommand::kind).collect();
    assert_eq!(kinds, vec!["clear-color", "draw-mesh"]);
    // 穷举面自检：match 未来加变体时此函数编译失败，逼 IR 消费端显式处理
    for c in &cmds {
        match c {
            DrawCommand::ClearColor { .. } | DrawCommand::DrawMesh { .. } => {}
        }
    }
}

// spec: REND-04
#[test]
fn viewport_roundtrip() {
    let vp = Viewport::new(800, 600, 1.5);
    assert_eq!(
        (vp.width(), vp.height(), vp.scale_factor()),
        (800, 600, 1.5)
    );
    let (lw, lh) = vp.logical_size();
    assert!((lw - 800.0 / 1.5).abs() < 1e-3 && (lh - 400.0).abs() < 1e-3);
}

// spec: REND-05
#[test]
fn camera_projection_variants() {
    let ortho = Camera::ortho(100.0, 100.0, -1000.0, 1000.0);
    let persp = Camera::perspective(std::f32::consts::FRAC_PI_3, 1.78, 0.1, 1e5);
    assert!(ortho.is_orthographic());
    assert!(!persp.is_orthographic());
}

// spec: REND-06
#[test]
fn mesh_desc_constructible() {
    let pos = [[0.0f32; 3]; 3];
    let nrm = [[0.0f32; 3]; 3];
    let idx = [0u32, 1, 2];
    let desc = MeshDesc {
        positions: &pos,
        normals: &nrm,
        indices: &idx,
        uv: &[],
    };
    assert_eq!(desc.positions.len(), 3);
}

// spec: REND-07
#[test]
fn create_mesh_returns_distinct_ids() {
    let mut b = Stub {
        meshes: 0,
        materials: 0,
    };
    let pos = [[0.0f32; 3]; 3];
    let desc = MeshDesc {
        positions: &pos,
        normals: &pos,
        indices: &[0, 1, 2],
        uv: &[],
    };
    let m1 = b.create_mesh(&desc).unwrap();
    let m2 = b.create_mesh(&desc).unwrap();
    assert_ne!(m1, m2);
    assert_eq!((m1, m2), (1, 2));
}

// spec: REND-08
#[test]
fn create_material_returns_distinct_ids() {
    let mut b = Stub {
        meshes: 0,
        materials: 0,
    };
    assert_eq!(
        (
            b.create_material([1.0, 0.0, 0.0, 1.0]).unwrap(),
            b.create_material([0.0, 1.0, 0.0, 1.0]).unwrap()
        ),
        (1, 2)
    );
}

// spec: REND-09
#[test]
fn frame_view_proj_fields_roundtrip() {
    let f = Frame {
        viewport: Viewport::new(1, 1, 1.0),
        camera: Camera::ortho(1.0, 1.0, -1.0, 1.0),
        view_rot: IDENTITY4,
        eye: [3.0, 4.0, 5.0],
        proj: IDENTITY4,
        commands: vec![],
    };
    assert_eq!(f.view_rot, IDENTITY4);
    assert_eq!(f.eye, [3.0, 4.0, 5.0]);
    assert_eq!(f.proj, IDENTITY4);
}

// spec: REND-17
#[test]
fn frame_camera_split_roundtrip() {
    let f = Frame {
        viewport: Viewport::new(2, 1, 1.0),
        camera: Camera::perspective(1.0, 2.0, 0.1, 10.0),
        view_rot: IDENTITY4,
        eye: [1.5e7, -2.5, 3.25],
        proj: IDENTITY4,
        commands: vec![],
    };
    assert_eq!(f.eye, [1.5e7, -2.5, 3.25]);
    assert_eq!(f.view_rot, IDENTITY4);
}

// spec: REND-18
#[test]
fn drawmesh_carries_origin() {
    let cmd = DrawCommand::DrawMesh {
        mesh: 7,
        material: 2,
        origin: [1.0e7, 0.0, 0.0],
        transform: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
    match cmd {
        DrawCommand::DrawMesh { origin, .. } => assert_eq!(origin, [1.0e7, 0.0, 0.0]),
        DrawCommand::ClearColor { .. } => panic!("expected mesh"),
    }
}

// spec: REND-26
#[test]
fn trait_defaults_texture_extension() {
    let mut s = Stub {
        meshes: 0,
        materials: 100,
    };
    // 默认体=转发：texture/repeat/specular 被安全丢弃，id 仍由 create_material 分配
    let id = s
        .create_material_desc(&MaterialDesc {
            base_color: [1.0, 0.0, 0.0, 1.0],
            texture: Some(7),
            repeat: [2.0, 2.0],
            specular: 0.5,
        })
        .unwrap();
    assert_eq!(id, 101);
    // 无纹理后端 upload=显式拒绝（不假成功）
    let e = s
        .upload_texture(&TextureDesc {
            rgba: &[0u8; 4],
            width: 1,
            height: 1,
        })
        .unwrap_err();
    assert!(e.reason.contains("unsupported"), "{e:?}");
}

// spec: REND-27
#[test]
fn create_instances_default_err() {
    let mut s = Stub {
        meshes: 0,
        materials: 0,
    };
    // 未覆写后端的默认体=显式拒绝（upload_texture 同款协议，不假成功）
    let e = s
        .create_instances(&InstanceDesc {
            data: &[Instance {
                offset: [1.0, 2.0, 0.0],
                height: 3.0,
                color: [0.8, 0.8, 0.9],
            }],
        })
        .unwrap_err();
    assert!(e.reason.contains("instancing"), "{e:?}");
}

// spec: REND-28
#[test]
fn draw_instances_carries_d7_origin_and_instance_table() {
    let cmd = DrawCommand::DrawInstances {
        mesh: 11,
        material: 22,
        instances: 33,
        origin: [1.0e7, 0.0, 0.0],
        transform: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
    assert_eq!(cmd.kind(), "draw-instances");
    match cmd {
        DrawCommand::DrawInstances {
            mesh,
            material,
            instances,
            origin,
            ..
        } => {
            assert_eq!((mesh, material, instances), (11, 22, 33));
            assert_eq!(origin, [1.0e7, 0.0, 0.0], "D7 语义与 DrawMesh 同款");
        }
        _ => panic!("variant"),
    }
}
