//! GLTF-01..08 契约测试（fixture=程序化 glb，testdata/ 入 git）。

use visiaengine_io_gltf::{IoError, load_gltf};

const TRI: &str = "testdata/tri-blue.glb";
const HIER: &str = "testdata/hierarchy.glb";
const TWO: &str = "testdata/twoprim.glb";

// spec: GLTF-01
#[test]
fn parses_glb_fixture() {
    let doc = load_gltf(TRI).unwrap();
    assert_eq!(doc.entities().len(), 1);
    let e = &doc.entities()[0];
    assert_eq!(e.name.as_deref(), Some("Quad"));
    assert_eq!(e.mesh.positions.len(), 4);
    assert_eq!(e.mesh.indices.len(), 6);
}

// spec: GLTF-02
#[test]
fn node_hierarchy_transform_baked() {
    let doc = load_gltf(HIER).unwrap();
    let w = doc.entities()[0].world;
    assert_eq!(w[3][0], 11.0);
    assert_eq!(w[3][1], 2.0);
    assert_eq!(w[3][2], 3.0);
}

// spec: GLTF-03
#[test]
fn missing_normals_default_or_generated() {
    let doc = load_gltf(TWO).unwrap();
    let second = &doc.entities()[1];
    assert_eq!(second.mesh.normals.len(), second.mesh.positions.len());
    assert!(second.mesh.normals.iter().all(|n| *n == [0.0, 0.0, 0.0]));
}

// spec: GLTF-04
#[test]
fn base_color_from_material() {
    let doc = load_gltf(TRI).unwrap();
    assert_eq!(doc.entities()[0].mesh.base_color, [0.1, 0.2, 0.9, 1.0]);
    let two = load_gltf(TWO).unwrap();
    assert_eq!(two.entities()[0].mesh.base_color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(two.entities()[1].mesh.base_color, [0.0, 1.0, 0.0, 1.0]);
}

// spec: GLTF-05
#[test]
fn multi_primitive_counts_consistent() {
    let doc = load_gltf(TWO).unwrap();
    assert_eq!(doc.entities().len(), 2);
    for e in doc.entities() {
        assert_eq!(e.mesh.positions.len(), 3);
        assert_eq!(e.mesh.indices.len(), 6);
    }
}

// spec: GLTF-06
#[test]
fn corrupt_file_err_not_panic() {
    let bytes = std::fs::read(TRI).unwrap();
    let truncated = &bytes[..bytes.len() / 2];
    std::fs::write("/tmp/visia-corrupt.glb", truncated).unwrap();
    let err = load_gltf("/tmp/visia-corrupt.glb").unwrap_err();
    assert!(matches!(err, IoError::Parse { .. }), "got {err:?}");
}

// spec: GLTF-07
#[test]
fn missing_file_io_err() {
    let err = load_gltf("testdata/definitely-not-here.glb").unwrap_err();
    assert!(matches!(err, IoError::NotFound { .. }), "got {err:?}");
}

// spec: GLTF-08
#[test]
fn y_up_orientation_preserved() {
    let doc = load_gltf(HIER).unwrap();
    let w = doc.entities()[0].world;
    assert_eq!(w[3][2], 3.0, "z 原样保留（无 Z-up 翻号）");
    assert_eq!(w[0][0], 1.0, "无旋转烘焙时单位对角");
}
