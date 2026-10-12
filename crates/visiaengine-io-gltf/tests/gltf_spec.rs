//! GLTF-01..08 contract tests (fixture = programmatic glb, resources/data/ enters git).

use visiaengine_io_gltf::{IoError, load_gltf};

// fixture located crate-relatively (cargo test CWD = crate root, not the repo root)
fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../resources/data")
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
    // the IR face = sRGB convention (CORE-16): the normative linear factor lands once via linear_to_srgb,
    // the backend upload chokepoint converts back to linear -- round-trip <= 2e-7 (lossless under 8-bit quantization)
    let bc = doc.entities()[0].mesh.base_color;
    let want = [0.3491902f32, 0.4845292, 0.9546872, 1.0];
    assert!(
        bc.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-4),
        "base_color should = linear_to_srgb(0.1,0.2,0.9), got {bc:?}"
    );
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
    assert_eq!(w[3][2], 3.0, "z kept as-is (no Z-up sign flip)");
    assert_eq!(w[0][0], 1.0, "unit diagonal with no rotation baked");
}

// ===== GLTF-09: primitive mode filter + load report ([E3D:A4] port, a correctness fix along the way) =====

/// Hand-assembled GLB: mesh0 = [POINTS primitive (dirty smuggle-in), TRIANGLES primitive (normal)]
fn glb_bytes(json: &str, bin: &[u8]) -> Vec<u8> {
    let mut jp = json.as_bytes().to_vec();
    while !jp.len().is_multiple_of(4) {
        jp.push(b' ');
    }
    let mut bp = bin.to_vec();
    while !bp.len().is_multiple_of(4) {
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
        {"attributes":{"POSITION":1},"indices":2}]}],
      "buffers":[{"byteLength":108}],
      "bufferViews":[{"buffer":0,"byteOffset":0,"byteLength":36},
        {"buffer":0,"byteOffset":36,"byteLength":48},
        {"buffer":0,"byteOffset":84,"byteLength":24,"target":34963}],
      "accessors":[
        {"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[8,8,8]},
        {"bufferView":1,"componentType":5126,"count":4,"type":"VEC3","min":[0,0,0],"max":[11,11,11]},
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
    // the POINTS primitive once smuggled into the entity list as a "fake triangle" -- now correctly skipped and counted
    assert_eq!(
        doc.entities().len(),
        1,
        "only the TRIANGLES primitive survives"
    );
    assert_eq!(doc.entities()[0].mesh.positions.len(), 4);
    assert_eq!(rep.skipped_non_triangle, 1);
    assert_eq!(rep.total_skipped(), 1);
}
