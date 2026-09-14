//! visiaengine-render 契约测试（仅公开 API；// spec: 标签入双向追溯门禁）。

use visiaengine_render::{
    BackendError, Camera, CameraRig, Capability, DrawCommand, Frame, Instance, InstanceDesc,
    MaterialDesc, MaterialId, MeshDesc, MeshId, PointMark, PointTableDesc, RenderBackend,
    ShadowBias, ShadowSetup, StrokeSeg, StrokeTableDesc, TextureDesc, Viewport,
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
                DrawCommand::ClearColor { .. }
                | DrawCommand::DrawMesh { .. }
                | DrawCommand::DrawInstances { .. }
                | DrawCommand::DrawStrokes { .. }
                | DrawCommand::DrawPoints { .. } => {}
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
        px_world_scale: 1.0,
        shadow: None,
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
        DrawCommand::DrawInstances {
            mesh: 1,
            material: 2,
            instances: 3,
            origin: [0.0; 3],
            transform: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        },
    ];
    let kinds: Vec<&'static str> = cmds.iter().map(DrawCommand::kind).collect();
    assert_eq!(kinds, vec!["clear-color", "draw-mesh", "draw-instances"]);
    // 穷举面自检：match 未来加变体时此函数编译失败，逼 IR 消费端显式处理
    for c in &cmds {
        match c {
            DrawCommand::ClearColor { .. }
            | DrawCommand::DrawMesh { .. }
            | DrawCommand::DrawInstances { .. }
            | DrawCommand::DrawStrokes { .. }
            | DrawCommand::DrawPoints { .. } => {}
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
        px_world_scale: 1.0,
        shadow: None,
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
        px_world_scale: 1.0,
        shadow: None,
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
        DrawCommand::ClearColor { .. }
        | DrawCommand::DrawInstances { .. }
        | DrawCommand::DrawStrokes { .. }
        | DrawCommand::DrawPoints { .. } => {
            panic!("expected mesh")
        }
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
            data: &[Instance::new([1.0, 2.0, 0.0], 3.0, [0.8, 0.8, 0.9])],
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

// spec: REND-27
#[test]
fn instance_pod_layout_32b() {
    // CPU repr(C) ↔ WGSL struct 逐字节同形锁（offset 0..12/height 12/color 16..28/pad 28）
    assert_eq!(std::mem::size_of::<Instance>(), 32);
    assert_eq!(std::mem::offset_of!(Instance, offset), 0);
    assert_eq!(std::mem::offset_of!(Instance, height), 12);
    assert_eq!(std::mem::offset_of!(Instance, color), 16);
    let inst = Instance::new([1.0, 2.0, 3.0], 4.0, [5.0, 6.0, 7.0]);
    let raw: &[u8; 32] = bytemuck::bytes_of(&inst).try_into().unwrap();
    let f = |o: usize| -> f32 { f32::from_le_bytes(raw[o..o + 4].try_into().unwrap()) };
    assert_eq!((f(0), f(12), f(16)), (1.0, 4.0, 5.0));
    assert_eq!(f(28), 0.0, "pad 归零（new 构造担保）");
}

const T4F64: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

// spec: REND-29
#[test]
fn frame_carries_px_world_scale() {
    let f = Frame {
        viewport: Viewport::new(8, 8, 1.0),
        camera: Camera::perspective(60.0, 1.0, 0.1, 100.0),
        view_rot: IDENTITY4,
        eye: [0.0; 3],
        proj: IDENTITY4,
        px_world_scale: 0.25, // 1px ≙ 0.25 世界单位（宿主给，GPU 乘子）
        shadow: None,
        commands: vec![],
    };
    assert_eq!(f.px_world_scale, 0.25);
}

// spec: REND-30
#[test]
fn expansion_tables_default_err_and_layout_locked() {
    let mut s = Stub {
        meshes: 0,
        materials: 0,
    };
    let e = s
        .create_strokes(&StrokeTableDesc {
            data: &[StrokeSeg::new(
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                2.0,
            )],
        })
        .unwrap_err();
    assert!(e.reason.contains("strokes"), "{e:?}");
    let e = s
        .create_points(&PointTableDesc {
            data: &[PointMark::new([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], 3.0)],
        })
        .unwrap_err();
    assert!(e.reason.contains("points"), "{e:?}");
    // 布局锁（WGSL 同形：vec4 同宽字段消对齐歧义 [4de R1]）
    assert_eq!(std::mem::size_of::<StrokeSeg>(), 48);
    assert_eq!(std::mem::offset_of!(StrokeSeg, a), 0);
    assert_eq!(std::mem::offset_of!(StrokeSeg, b), 16);
    assert_eq!(std::mem::offset_of!(StrokeSeg, color), 32);
    assert_eq!(std::mem::offset_of!(StrokeSeg, width_px), 44);
    assert_eq!(std::mem::size_of::<PointMark>(), 32);
    assert_eq!(std::mem::offset_of!(PointMark, pos), 0);
    assert_eq!(std::mem::offset_of!(PointMark, radius_px), 12);
    assert_eq!(std::mem::offset_of!(PointMark, color), 16);
}

// spec: REND-30
#[test]
fn draw_strokes_and_points_kinds() {
    let a = DrawCommand::DrawStrokes {
        table: 5,
        origin: [1.0e7, 0.0, 0.0],
        transform: T4F64,
    };
    let b = DrawCommand::DrawPoints {
        table: 6,
        origin: [0.0; 3],
        transform: T4F64,
    };
    assert_eq!(a.kind(), "draw-strokes");
    assert_eq!(b.kind(), "draw-points");
}

// spec: REND-31
#[test]
fn shadow_setup_is_frame_option_and_none_default_regression_key() {
    // Frame.shadow=Option：None=现行为逐位不变（4f 零回归钥匙位）；
    // 深度域构造路=CameraRig 系（REND-11/12 行为断言域）——本测试锁类型轮转，
    // 症状级死锁（非全黑非全亮）在 WGPU-19 像素面（shadows.rs）。
    let rig = CameraRig::look_at([0.0, -10.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let setup = ShadowSetup {
        proj: rig.ortho_frame(8.0, 64.0, 64.0, 1.0, 100.0).unwrap(),
        view_rot: rig.view_rotation(),
        eye: rig.eye(),
        light_dir: [0.3, 0.5, 0.7],
        size: 0.4,
        bias: ShadowBias {
            constant: -1.2,
            slope: -1.5,
        },
    };
    assert_eq!(setup, setup); // PartialEq（帧缓存对比面）
    let mut f = Frame {
        viewport: Viewport::new(8, 8, 1.0),
        camera: Camera::perspective(60.0, 1.0, 0.1, 100.0),
        view_rot: IDENTITY4,
        eye: [0.0; 3],
        proj: IDENTITY4,
        px_world_scale: 1.0,
        shadow: Some(setup),
        commands: vec![],
    };
    assert!(f.shadow.is_some());
    f.shadow = None;
    assert_eq!(f.shadow, None);
    // 默认 bias 常数在案（调参基线）
    assert_eq!(ShadowSetup::DEFAULT_BIAS.constant, -1.2);
}
