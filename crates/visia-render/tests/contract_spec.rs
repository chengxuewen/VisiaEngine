//! visia-render 契约测试（仅公开 API；// spec: 标签入双向追溯门禁）。

use visia_core::{Component, Transform, Vec3};
use visia_render::{Camera, Capability, DrawCommand, Frame, MeshId, RenderBackend, Viewport};

struct Stub;

// spec: REND-01
#[test]
fn backend_trait_object_safe() {
    let b: Box<dyn RenderBackend> = Box::new(Stub);
    assert!(!b.name().is_empty());
}

// spec: REND-02
#[test]
fn stub_impl_without_wgpu() {
    // 本文件即证明：实现 RenderBackend 无需任何后端 crate（不变式②契约面纯度）
    let mut b = Stub;
    let vp = Viewport::new(64, 64, 1.0);
    b.resize(vp);
    let frame = Frame {
        viewport: vp,
        camera: Camera::perspective(1.0, 1.0, 0.1, 1000.0),
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
            transform: Transform::identity(),
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
    assert_eq!((vp.width(), vp.height(), vp.scale_factor()), (800, 600, 1.5));
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

// REND-02 附带面：core 数据型经 IR 流转的编译性
#[allow(dead_code)]
fn component_flows_into_frame() {
    let _c = Component::Transform(Transform {
        position: Vec3::new(1.0, 2.0, 3.0),
        scale: 1.0,
    });
}
