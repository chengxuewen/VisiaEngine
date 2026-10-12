//! GLTF-10: the GLB decoder panics on no input (structured header tampering + random stream, two roads).

use proptest::prelude::*;
use visiaengine_io_gltf::load_gltf_bytes;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    // spec: GLTF-10
    #[test]
    fn random_bytes_never_panic(ref bytes in proptest::collection::vec(any::<u8>(), 0..1024)) {
        let _ = load_gltf_bytes(bytes);
    }

    // spec: GLTF-10
    #[test]
    fn tampered_glb_headers_never_panic(
        magic in prop::array::uniform4(any::<u8>()),
        ver in any::<u32>(),
        total in any::<u32>(),
        body in proptest::collection::vec(any::<u8>(), 0..256),
    ) {
        let mut buf = Vec::new();
        buf.extend_from_slice(&magic);
        buf.extend_from_slice(&ver.to_le_bytes());
        buf.extend_from_slice(&total.to_le_bytes());
        buf.extend_from_slice(&body);
        let _ = load_gltf_bytes(&buf);
    }
}

/// Real legal GLB prefix + tail truncation/tampering (the fixture-guided boundary band)
#[test]
fn truncated_valid_glb_never_panic() {
    let full = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/twoprim.glb"
    ))
    .unwrap();
    for cut in [
        0usize,
        1,
        12,
        13,
        full.len() / 2,
        full.len().saturating_sub(1),
    ] {
        let _ = load_gltf_bytes(&full[..cut.min(full.len())]);
    }
}
