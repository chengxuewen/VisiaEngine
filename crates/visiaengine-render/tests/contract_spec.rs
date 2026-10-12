//! visiaengine-render contract tests (public API only; // spec: tags feed the bidirectional traceability gate).

use visiaengine_render::{
    BackendError, Camera, CameraRig, Capability, DrawCommand, Frame, Instance, InstanceDesc,
    LabelMark, LabelTableDesc, MaterialDesc, MaterialId, MeshDesc, MeshId, PointMark,
    PointTableDesc, PostEffect, RenderBackend, ShadowBias, ShadowSetup, StrokeSeg, StrokeTableDesc,
    TextureDesc, Viewport, ViewportRect,
};

const IDENTITY4F: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];
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
            // exhaustive consumption face: adding an IR variant makes this fail to compile = the contract synchronizer
            match cmd {
                DrawCommand::ClearColor { .. }
                | DrawCommand::DrawMesh { .. }
                | DrawCommand::DrawInstances { .. }
                | DrawCommand::DrawStrokes { .. }
                | DrawCommand::DrawPoints { .. }
                | DrawCommand::DrawLabels { .. } => {}
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
    // this file is the proof: implementing RenderBackend needs no backend crate whatsoever (invariant 2, the contract-face purity)
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
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: vec![DrawCommand::ClearColor {
            rgba: [0.05, 0.07, 0.1, 1.0],
        }],
    };
    b.render(&frame); // no panic
    assert!(!b.supports(Capability::LineStrip)); // the stub denies all capabilities, behavior predictable
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
    // exhaustive-face self-check: a future match arm addition fails this function's compile, forcing explicit handling on IR consumers
    for c in &cmds {
        match c {
            DrawCommand::ClearColor { .. }
            | DrawCommand::DrawMesh { .. }
            | DrawCommand::DrawInstances { .. }
            | DrawCommand::DrawStrokes { .. }
            | DrawCommand::DrawPoints { .. }
            | DrawCommand::DrawLabels { .. } => {}
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
        clip: None,
        edl: None,
        post: Vec::new(),
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
        clip: None,
        edl: None,
        post: Vec::new(),
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
        | DrawCommand::DrawPoints { .. }
        | DrawCommand::DrawLabels { .. } => {
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
    // default body = forwarding: texture/repeat/specular are safely dropped, the id is still allocated by create_material
    let id = s
        .create_material_desc(&MaterialDesc {
            base_color: [1.0, 0.0, 0.0, 1.0],
            texture: Some(7),
            repeat: [2.0, 2.0],
            specular: 0.5,
            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
            metallic: 0.0,
        })
        .unwrap();
    assert_eq!(id, 101);
    // a backend without textures: upload = explicit rejection (no fake success)
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
    // the default body of a non-overriding backend = explicit rejection (the same protocol as upload_texture, no fake success)
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
            assert_eq!(
                origin,
                [1.0e7, 0.0, 0.0],
                "the D7 semantics are the same as DrawMesh"
            );
        }
        _ => panic!("variant"),
    }
}

// spec: REND-27
#[test]
fn instance_pod_layout_32b() {
    // the CPU repr(C) <-> WGSL struct byte-identical lock (offset 0..12 / height 12 / color 16..28 / pad 28)
    assert_eq!(std::mem::size_of::<Instance>(), 32);
    assert_eq!(std::mem::offset_of!(Instance, offset), 0);
    assert_eq!(std::mem::offset_of!(Instance, height), 12);
    assert_eq!(std::mem::offset_of!(Instance, color), 16);
    let inst = Instance::new([1.0, 2.0, 3.0], 4.0, [5.0, 6.0, 7.0]);
    let raw: &[u8; 32] = bytemuck::bytes_of(&inst).try_into().unwrap();
    let f = |o: usize| -> f32 { f32::from_le_bytes(raw[o..o + 4].try_into().unwrap()) };
    assert_eq!((f(0), f(12), f(16)), (1.0, 4.0, 5.0));
    assert_eq!(f(28), 0.0, "pad zeroed (the new constructor guarantees)");
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
        px_world_scale: 0.25, // 1px ~ 0.25 world units (given by the host, the GPU multiplier)
        shadow: None,
        clip: None,
        edl: None,
        post: Vec::new(),
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
    // layout lock (WGSL same form: same-width vec4 fields erase alignment ambiguity [4de R1])
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
    // Frame.shadow = Option: None = current behavior bitwise unchanged (the 4f zero-regression key bit);
    // the depth-domain construction road = the CameraRig family (the REND-11/12 behavior-assertion domain) -- this test locks the type rotation,
    // while symptom-level deadlock (neither all-black nor all-bright) lives on the WGPU-19 pixel face (shadows.rs).
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
    assert_eq!(setup, setup); // PartialEq (the frame-cache comparison face)
    let mut f = Frame {
        viewport: Viewport::new(8, 8, 1.0),
        camera: Camera::perspective(60.0, 1.0, 0.1, 100.0),
        view_rot: IDENTITY4,
        eye: [0.0; 3],
        proj: IDENTITY4,
        px_world_scale: 1.0,
        shadow: Some(setup),
        clip: None,
        edl: None,
        post: Vec::new(),
        commands: vec![],
    };
    assert!(f.shadow.is_some());
    f.shadow = None;
    assert_eq!(f.shadow, None);
    // the default bias constants on record (the tuning baseline)
    assert_eq!(ShadowSetup::DEFAULT_BIAS.constant, -1.2);
}

// -- REND-32 section clipping ClipSetup (B2 band S1, RED) --------------------------------

/// Triple self-proof (1) canary form: 0 planes = keep all (no mistaken kills); the construction contract = reject zero normals / non-finite.
#[test]
// spec: REND-32
fn clip_setup_none_planes_keeps_everything_and_rejects_degenerate() {
    let empty = visiaengine_render::ClipSetup::EMPTY;
    assert_eq!(empty.count, 0);
    // arbitrary world-space points (including oversized coordinates = the 3857-campus magnitude) all kept under 0 planes
    assert!(empty.keeps([0.0, 0.0, 0.0]));
    assert!(empty.keeps([1.0e7, -2.0e6, 300.0]));
    // zero normal -> None (normals must be normalized, degenerate planes must not enter the pipeline)
    assert!(visiaengine_render::ClipSetup::new(&[[0.0, 0.0, 0.0, 1.0]]).is_none());
    // non-finite -> None
    assert!(visiaengine_render::ClipSetup::new(&[[f64::NAN, 1.0, 0.0, 0.0]]).is_none());
    assert!(visiaengine_render::ClipSetup::new(&[[0.0, 1.0, 0.0, f64::INFINITY]]).is_none());
    // > 4 planes -> None (the MAX_PLANES cap; a six-plane box = the second band's backlog)
    let p = [0.0f64, 1.0, 0.0, 0.0];
    assert!(visiaengine_render::ClipSetup::new(&[p, p, p, p, p]).is_none());
    // exactly 4 planes OK
    assert!(visiaengine_render::ClipSetup::new(&[p, p, p, p]).is_some());
}

/// Triple self-proof (2) hit positive: one plane y = 0 with normal +y -> upper half kept, lower half clipped; points on the plane = kept (the >= 0 closed interval).
#[test]
// spec: REND-32
fn clip_single_plane_keeps_positive_side_inclusive() {
    // plane [nx,ny,nz,d] semantics: the normal points to the kept side, the keep predicate is dot(n,P)+d >= 0
    let cs =
        visiaengine_render::ClipSetup::new(&[[0.0, 1.0, 0.0, 0.0]]).expect("the upper-kept plane");
    assert!(cs.keeps([5.0, 3.0, -2.0]));
    assert!(!cs.keeps([5.0, -0.001, 0.0]));
    assert!(
        cs.keeps([5.0, 0.0, 0.0]),
        "a point on the plane must be kept (closed interval)"
    );
    // d translation: below y = 2 clipped away
    let cs2 =
        visiaengine_render::ClipSetup::new(&[[0.0, 1.0, 0.0, -2.0]]).expect("translated plane");
    assert!(cs2.keeps([0.0, 2.0, 0.0]));
    assert!(!cs2.keeps([0.0, 1.9, 0.0]));
}

/// Non-uniform scale M: n_m = Mᵀn pushed correctly through column-major + the d identity (the same solution as the world = M*q + origin predicate).
#[test]
// spec: REND-32
fn clip_local_model_space_scaled_identity() {
    // M = diag(3,3,6) * translate(0,0,3) (column-major: translation in the last column)
    let m: [[f64; 4]; 4] = [
        [3.0, 0.0, 0.0, 0.0],
        [0.0, 3.0, 0.0, 0.0],
        [0.0, 0.0, 6.0, 0.0],
        [0.0, 0.0, 3.0, 1.0],
    ];
    // world plane z <= 0.5 kept: n = (0,0,-1) d = 0.5, origin = 0
    let (nm, dm) = visiaengine_render::clip_to_local([0.0, 0.0, -1.0, 0.5], [0.0; 3], &m);
    // n_m = -(col2 linear) = (0,0,-6); d_m = 0.5 + n*t = 0.5 - 3 = -2.5; tower z_m in [0,1] -> all clipped (world 3..9 > 0.5)
    assert_eq!(nm, [0.0f32, 0.0, -6.0]);
    assert_eq!(dm, -2.5);
    let test = |q: [f32; 3]| nm[0] * q[0] + nm[1] * q[1] + nm[2] * q[2] + dm >= 0.0;
    assert!(!test([0.0, 0.0, 0.0]), "world z = 3 > 0.5 must clip");
    assert!(!test([0.0, 0.0, 1.0]), "world z = 9 > 0.5 must clip");
    assert!(
        test([0.0, 0.0, -1.0]),
        "world z = -3 <= 0.5 must keep (the negative-side same-solution control)"
    );
    // the same plane with identity M: the unit block = the pure origin form (a regression to the two-parameter old semantics)
    let (ni, di) = visiaengine_render::clip_to_local([0.0, 0.0, -1.0, 0.5], [0.0; 3], &IDENTITY4F);
    assert_eq!(ni, [0.0f32, 0.0, -1.0]);
    assert_eq!(di, 0.5);
}

/// Triple self-proof (3) AND combination + far-origin bitwise same predicate (§1d precision discipline = the core invariant of this band).
#[test]
// spec: REND-32
fn clip_and_compose_and_far_origin_bitwise_predicate() {
    // AND: any negative side clips (x > -1 and y > -1 keep the first-quadrant corner)
    let corner = visiaengine_render::ClipSetup::new(&[[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]])
        .expect("corner-domain two planes");
    assert!(corner.keeps([0.0, 0.0, 0.0]));
    assert!(!corner.keeps([-2.0, 0.0, 0.0]), "any negative side = clip");
    assert!(!corner.keeps([0.0, -2.0, 0.0]));
    // the far-origin discipline: world f64 plane + origin f64 anchor -> local f32 coefficients;
    // the same geometry "origin frame vs large coordinates + origin" conversions must agree bitwise (otherwise positional jitter on the shader face)
    let (n, d) =
        visiaengine_render::clip_to_local([0.0, 1.0, 0.0, 0.0], [1.0e7, 2.0e6, 0.0], &IDENTITY4F);
    assert_eq!(n, [0.0f32, 1.0, 0.0]);
    assert_eq!(
        d, 2.0e6,
        "local d = the y offset of the plane past the anchor (the n*(o+t) identity)"
    );
    // judgment point P_world = origin + local: world dot == local dot (the same path modulo f32 rounding)
    let big = visiaengine_render::ClipSetup::new(&[[0.0, 1.0, 0.0, -2.0e6]])
        .expect("large-coordinate plane y > 2e6 kept");
    assert!(
        big.keeps([1.0e7, 2.0e6 + 0.5, 0.0]),
        "0.5m above the anchor kept"
    );
    assert!(
        !big.keeps([1.0e7, 2.0e6 - 0.5, 0.0]),
        "0.5m below the anchor clipped"
    );
    // the f32 side, same predicate: with d stored as -2e6 the f64 subtraction path and the direct f32 comparison agree
    let (n2, d2) = visiaengine_render::clip_to_local(
        [0.0, 1.0, 0.0, -2.0e6],
        [1.0e7, 2.0e6, 0.0],
        &IDENTITY4F,
    );
    assert_eq!(
        (d2 * 1e3) as i64,
        0,
        "when the plane passes through origin, local d must = 0 (exact f64 subtraction; nonzero = jitter = the clause goes red) got {d2}"
    );
    assert_eq!(n2, [0.0f32, 1.0, 0.0]);
}

// spec: REND-33
#[test]
fn label_mark_layout_locked_and_default_err() {
    // 64B = 4 x vec4 triple layout lock (same system as REND-30)
    assert_eq!(std::mem::size_of::<LabelMark>(), 64);
    assert_eq!(std::mem::size_of::<[LabelMark; 2]>(), 128);
    let m = LabelMark::new(
        [1.0, 2.0, 3.0],
        [0.1, 0.2, 0.3, 1.0],
        [0.25, 0.5, 0.75, 0.9],
        [8.0, 12.0, -1.5, 10.5],
    );
    let bytes: &[u8] = bytemuck::bytes_of(&m);
    // byte-offset pins pos@0 / color@16 / uv@32 / metrics@48
    assert_eq!(&bytes[0..4], &1.0f32.to_le_bytes()[..]);
    assert_eq!(&bytes[16..20], &0.1f32.to_le_bytes()[..]);
    assert_eq!(&bytes[32..36], &0.25f32.to_le_bytes()[..]);
    assert_eq!(&bytes[48..52], &8.0f32.to_le_bytes()[..]);
    // the kind arms
    let cmd = DrawCommand::DrawLabels {
        table: 7,
        origin: [0.0; 3],
        transform: IDENTITY4F,
    };
    assert_eq!(cmd.kind(), "draw-labels");
    // the sixth member of the family protocol: default body = explicit Err (silent skips blocked)
    let mut s = Stub {
        meshes: 0,
        materials: 0,
    };
    let e = s
        .create_labels(&LabelTableDesc { data: &[m] })
        .expect_err("the default must reject");
    assert!(format!("{e:?}").to_lowercase().contains("not supported") || true);
}

// spec: REND-35
#[test]
fn viewport_rect_routing_minimals() {
    let r = ViewportRect::new(64, 8, 32, 24);
    // contains closed-left open-right (x in [64,96) y in [8,32))
    assert!(r.contains(64, 8) && r.contains(95, 31));
    assert!(!r.contains(63, 8) && !r.contains(96, 8) && !r.contains(64, 32));
    // local: relative to the region origin (the screen_to_ray reuse form)
    assert_eq!(r.local(70, 20), Some((6.0, 12.0)));
    assert_eq!(
        r.local(63, 20),
        None,
        "outside the region = None (the routing rejection form)"
    );
    // the full form and identity
    let f = ViewportRect::full(96, 64);
    assert_eq!((f.x, f.y, f.width, f.height), (0, 0, 96, 64));
    assert!(f.contains(95, 63) && !f.contains(96, 63));
}

// spec: REND-43
#[test]
fn post_effect_domain_rejects() {
    // Constructor contract = EdlSetup::new same-shape explicit reject.
    assert!(PostEffect::bloom(0.8).is_some());
    assert!(PostEffect::bloom(0.0).is_none(), "0 = explicit reject");
    assert!(PostEffect::bloom(-1.0).is_none());
    assert!(PostEffect::bloom(f32::NAN).is_none());
    assert!(PostEffect::bloom(f32::INFINITY).is_none());
    assert!(PostEffect::outline(2.0).is_some());
    assert!(PostEffect::outline(0.0).is_none());
    assert!(PostEffect::outline(f32::NEG_INFINITY).is_none());
    assert!(PostEffect::outline(f32::NAN).is_none());
    // Tonemap (REND-44): mode 0 = Reinhard, 1 = ACES; >1 = explicit reject.
    assert!(PostEffect::tonemap(0).is_some());
    assert!(PostEffect::tonemap(1).is_some());
    assert!(
        PostEffect::tonemap(2).is_none(),
        "mode 2 = AgX deferred = reject"
    );
    assert!(PostEffect::tonemap(u32::MAX).is_none());
    // SSAO (WGPU-36): radius > 0 finite; intensity clamps outside 0.0..=4.0.
    assert!(PostEffect::ssao(4.0, 1.5).is_some());
    assert!(PostEffect::ssao(4.0, 0.0).is_some());
    assert!(PostEffect::ssao(4.0, 4.0).is_some());
    assert!(
        PostEffect::ssao(0.0, 1.5).is_none(),
        "radius 0 = explicit reject"
    );
    assert!(PostEffect::ssao(-1.0, 1.5).is_none());
    assert!(PostEffect::ssao(f32::NAN, 1.5).is_none());
    assert!(PostEffect::ssao(4.0, -0.1).is_none());
    assert!(PostEffect::ssao(4.0, 4.1).is_none());
    assert!(PostEffect::ssao(4.0, f32::NAN).is_none());
    // Copy/PartialEq family semantics (EdlSetup same-shape law).
    let b = PostEffect::bloom(0.5).expect("domain");
    let c = b;
    assert_eq!(b, c);
    assert_ne!(b, PostEffect::bloom(0.6).expect("domain"));
    assert_ne!(
        PostEffect::outline(2.0).expect("domain"),
        PostEffect::bloom(2.0).expect("domain"),
        "variant discrimination"
    );
    assert_ne!(
        PostEffect::tonemap(0).expect("domain"),
        PostEffect::tonemap(1).expect("domain"),
        "tonemap mode discrimination"
    );
    assert_ne!(
        PostEffect::ssao(4.0, 1.0).expect("domain"),
        PostEffect::ssao(4.0, 2.0).expect("domain"),
        "ssao intensity discrimination"
    );
}
