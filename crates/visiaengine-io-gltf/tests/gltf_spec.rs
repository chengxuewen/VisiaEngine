//! GLTF-01..08 契约测试（fixture=程序化 glb，testdata/ 入 git）。

use visiaengine_io_gltf::{IoError, load_gltf};

// fixture 以 crate 相对定位（cargo 测试 CWD=crate root，非仓根）
fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata")
        .join(name)
}
const TRI: &str = "tri-blue.glb";
const HIER: &str = "hierarchy.glb";
const TWO: &str = "twoprim.glb";

// spec: GLTF-01
#[test]
fn parses_glb_fixture() {
    let doc = load_gltf(fixture(TRI)).unwrap();
    assert_eq!(doc.entities().len(), 1);
    let e = &doc.entities()[0];
    assert_eq!(e.name.as_deref(), Some("Quad"));
    assert_eq!(e.mesh.positions.len(), 4);
    assert_eq!(e.mesh.indices.len(), 6);
}

// spec: GLTF-02
#[test]
fn node_hierarchy_transform_baked() {
    let doc = load_gltf(fixture(HIER)).unwrap();
    let w = doc.entities()[0].world;
    assert_eq!(w[3][0], 11.0);
    assert_eq!(w[3][1], 2.0);
    assert_eq!(w[3][2], 3.0);
}

// spec: GLTF-03
#[test]
fn missing_normals_default_or_generated() {
    let doc = load_gltf(fixture(TWO)).unwrap();
    let second = &doc.entities()[1];
    assert_eq!(second.mesh.normals.len(), second.mesh.positions.len());
    assert!(second.mesh.normals.iter().all(|n| *n == [0.0, 0.0, 0.0]));
}

// spec: GLTF-04
#[test]
fn base_color_from_material() {
    let doc = load_gltf(fixture(TRI)).unwrap();
    assert_eq!(doc.entities()[0].mesh.base_color, [0.1, 0.2, 0.9, 1.0]);
    let two = load_gltf(fixture(TWO)).unwrap();
    assert_eq!(two.entities()[0].mesh.base_color, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(two.entities()[1].mesh.base_color, [0.0, 1.0, 0.0, 1.0]);
}

// spec: GLTF-05
#[test]
fn multi_primitive_counts_consistent() {
    let doc = load_gltf(fixture(TWO)).unwrap();
    assert_eq!(doc.entities().len(), 2);
    for e in doc.entities() {
        assert_eq!(e.mesh.positions.len(), 3);
        assert_eq!(e.mesh.indices.len(), 6);
    }
}

// spec: GLTF-06
#[test]
fn corrupt_file_err_not_panic() {
    let bytes = std::fs::read(fixture(TRI)).unwrap();
    let truncated = &bytes[..bytes.len() / 2];
    std::fs::write("/tmp/visia-corrupt.glb", truncated).unwrap();
    let err = load_gltf("/tmp/visia-corrupt.glb").unwrap_err();
    assert!(matches!(err, IoError::Parse { .. }), "got {err:?}");
}

// spec: GLTF-07
#[test]
fn missing_file_io_err() {
    let err = load_gltf(fixture("definitely-not-here.glb")).unwrap_err();
    assert!(matches!(err, IoError::NotFound { .. }), "got {err:?}");
}

// spec: GLTF-08
#[test]
fn y_up_orientation_preserved() {
    let doc = load_gltf(fixture(HIER)).unwrap();
    let w = doc.entities()[0].world;
    assert_eq!(w[3][2], 3.0, "z 原样保留（无 Z-up 翻号）");
    assert_eq!(w[0][0], 1.0, "无旋转烘焙时单位对角");
}

// ===== GLTF-09：primitive 模式过滤 + 加载报告（[E3D:A4] 移植，顺带正确性修复）=====

/// 手工组 GLB：mesh0 = [POINTS primitive(脏混入), TRIANGLES primitive(正常)]
fn glb_bytes(json: &str, bin: &[u8]) -> Vec<u8> {
    let mut jp = json.as_bytes().to_vec();
    while jp.len() % 4 != 0 {
        jp.push(b' ');
    }
    let mut bp = bin.to_vec();
    while bp.len() % 4 != 0 {
        bp.push(0);
    }
    let total = 12 + 8 + jp.len() + 8 + bp.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&[0x67, 0x6C, 0x54, 0x46]); // glTF magic LE
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(jp.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // JSON
    out.extend_from_slice(&jp);
    out.extend_from_slice(&(bp.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x004E4942u32.to_le_bytes()); // BIN
    out.extend_from_slice(&bp);
    out
}

fn mixed_mode_glb() -> Vec<u8> {
    // BIN: acc0 POINTS pos 3×vec3(36B) | acc1 TRI pos 4×vec3(48B) | acc2 idx u32×6(24B)
    let mut bin = Vec::new();
    for i in 0..9 {
        bin.extend_from_slice(&(i as f32).to_le_bytes());
    }
    for i in 0..12 {
        bin.extend_from_slice(&(i as f32).to_le_bytes());
    }
    for i in [0u32, 1, 2, 2, 3, 0] {
        bin.extend_from_slice(&i.to_le_bytes());
    }
    let json = r#"{"asset":{"version":"2.0"},"scene":0,"scenes":[{"nodes":[0]}],
      "nodes":[{"mesh":0}],
      "meshes":[{"primitives":[
        {"attributes":{"POSITION":0},"mode":0},
        {"attributes":{"POSITION":1,"indices":2}}]}],
      "buffers":[{"byteLength":108}],
      "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":36},
        {"buffer":0,"byteOffset":36,"byteLength":48},
        {"buffer":0,"byteOffset":84,"byteLength":24,"target":34963}],
      "accessors":[
        {"bufferView":0,"componentType":5126,"count":3,"type":"VEC3"},
        {"bufferView":1,"componentType":5126,"count":4,"type":"VEC3"},
        {"bufferView":2,"componentType":5125,"count":6,"type":"SCALAR"}]}"#;
    glb_bytes(json, &bin)
}

#[test]
fn non_triangle_primitives_are_skipped_and_reported() {
    // spec: GLTF-09
    let dir = std::env::temp_dir().join(format!("visiaengine_gltf09_{}.glb", std::process::id()));
    std::fs::write(&dir, mixed_mode_glb()).unwrap();
    let (doc, rep) = visiaengine_io_gltf::load_gltf_with_report(&dir).unwrap();
    let _ = std::fs::remove_file(&dir);
    // POINTS primitive 曾以"假三角"混进实体列表 —— 现正确跳过并计数
    assert_eq!(doc.entities().len(), 1, "仅 TRIANGLES primitive 存活");
    assert_eq!(doc.entities()[0].mesh.positions.len(), 4);
    assert_eq!(rep.skipped_non_triangle, 1);
    assert_eq!(rep.total_skipped(), 1);
}
