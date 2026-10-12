//! GLTF-11: the uv/texture/factor read contract (builder fixture generated programmatically, zero external assets).
//! GLTF-10 coverage extension: deterministic tampering band over the image section (flip/truncate must not panic).

use image::ImageEncoder as _;
use visiaengine_io_gltf::load_gltf_bytes;

/// 4x4 RGBA checkerboard (even = red, odd = white).
fn checker_png() -> Vec<u8> {
    let mut px = Vec::new();
    for j in 0..4u32 {
        for i in 0..4u32 {
            px.extend_from_slice(if (i + j) % 2 == 0 {
                &[255, 0, 0, 255]
            } else {
                &[255, 255, 255, 255]
            });
        }
    }
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&px, 4, 4, image::ExtendedColorType::Rgba8)
        .expect("png encode");
    png
}

fn push_f32(v: &[f32], out: &mut Vec<u8>) {
    for x in v {
        out.extend_from_slice(&x.to_le_bytes());
    }
}

/// Programmatic textured quad GLB (the R6/GLTF-11 teaching piece: chunk 4-byte padding + byteOffset alignment,
/// self-check assertions precede all consumer assertions).
fn build_textured_glb() -> Vec<u8> {
    let png = checker_png();
    let positions: Vec<f32> = vec![-1., -1., 0., 1., -1., 0., 1., 1., 0., -1., 1., 0.];
    let normals: Vec<f32> = vec![0., 0., 1., 0., 0., 1., 0., 0., 1., 0., 0., 1.];
    let uvs: Vec<f32> = vec![0., 0., 1., 0., 1., 1., 0., 1.];
    let indices: Vec<u8> = [0u16, 1, 2, 0, 2, 3]
        .iter()
        .flat_map(|i| i.to_le_bytes())
        .collect();

    let mut bin: Vec<u8> = Vec::new();
    let off_pos = bin.len();
    push_f32(&positions, &mut bin);
    let off_nrm = bin.len();
    push_f32(&normals, &mut bin);
    let off_uv = bin.len();
    push_f32(&uvs, &mut bin);
    let off_idx = bin.len();
    bin.extend_from_slice(&indices);
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let off_tex = bin.len();
    bin.extend_from_slice(&png);
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let json = format!(
        r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[0]}}],
        "nodes":[{{"mesh":0}}],
        "meshes":[{{"primitives":[{{"attributes":{{"POSITION":0,"NORMAL":1,"TEXCOORD_0":2}},"indices":3,"material":0}}]}}],
        "materials":[{{"pbrMetallicRoughness":{{"baseColorFactor":[0.5,0.25,0.5,1.0],"metallicFactor":0.2,"roughnessFactor":0.6,"baseColorTexture":{{"index":0,"texCoord":0}}}}}}],
        "textures":[{{"sampler":0,"source":0}}],
        "samplers":[{{"magFilter":9729,"minFilter":9987,"wrapS":10497,"wrapT":10497}}],
        "images":[{{"bufferView":4,"mimeType":"image/png"}}],
        "accessors":[
          {{"bufferView":0,"componentType":5126,"count":4,"type":"VEC3","min":[-1.0,-1.0,0.0],"max":[1.0,1.0,0.0]}},
          {{"bufferView":1,"componentType":5126,"count":4,"type":"VEC3"}},
          {{"bufferView":2,"componentType":5126,"count":4,"type":"VEC2"}},
          {{"bufferView":3,"componentType":5123,"count":6,"type":"SCALAR"}}],
        "buffers":[{{"byteLength":{}}}],
        "bufferViews":[
          {{"buffer":0,"byteOffset":{off_pos},"byteLength":48,"target":34962}},
          {{"buffer":0,"byteOffset":{off_nrm},"byteLength":48,"target":34962}},
          {{"buffer":0,"byteOffset":{off_uv},"byteLength":32,"target":34962}},
          {{"buffer":0,"byteOffset":{off_idx},"byteLength":12,"target":34963}},
          {{"buffer":0,"byteOffset":{off_tex},"byteLength":{}}}]
        }}"#,
        bin.len(),
        png.len(),
    );
    let mut jp = json.into_bytes();
    let jpad = (4 - jp.len() % 4) % 4;
    jp.extend(std::iter::repeat_n(b' ', jpad));
    let mut out = Vec::new();
    out.extend_from_slice(&0x4654_6C67u32.to_le_bytes());
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&((12 + 8 + jp.len() + 8 + bin.len()) as u32).to_le_bytes());
    out.extend_from_slice(&(jp.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4E4F_534Au32.to_le_bytes());
    out.extend_from_slice(&jp);
    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x004E_4942u32.to_le_bytes());
    out.extend_from_slice(&bin);
    out
}

// spec: GLTF-11
#[test]
fn textured_glb_reads_uv_texture_factors() {
    let glb = build_textured_glb();
    // R6 self-check: the builder itself first passes gltf validation (malformed = RED confusing the debug, assert ahead)
    let doc =
        load_gltf_bytes(&glb).unwrap_or_else(|e| panic!("the builder GLB must be legal: {e}"));
    assert_eq!(doc.entities().len(), 1);
    let m = &doc.entities()[0].mesh;
    assert_eq!(m.positions.len(), 4);
    assert_eq!(m.uv.len(), 4, "GLTF-11: TEXCOORD_0 read");
    assert_eq!(m.uv[3], [0.0, 1.0]);
    assert_eq!(m.texture, Some(0), "baseColorTexture -> image slot");
    assert!(
        (m.base_color[0] - 0.7354).abs() < 1e-3,
        "the IR face = sRGB convention (CORE-16)"
    ); // linear_to_srgb(0.5)
    assert!(
        (m.metallic_factor - 0.2).abs() < 1e-6,
        "factor collection (the mock-up face M2)"
    );
    assert!((m.roughness_factor - 0.6).abs() < 1e-6);
    let t = &doc.textures()[0];
    assert_eq!((t.width, t.height), (4, 4));
    assert_eq!(
        &t.rgba[0..4],
        &[255, 0, 0, 255],
        "checkerboard first pixel = red"
    );
    assert_eq!(t.rgba.len(), 64);
}

// spec: GLTF-11
#[test]
fn no_texture_primitive_is_none_and_no_uv_empty() {
    // reuse the GLTF-01 fixture (no uv/texture): the new zero-value fields do not break the existing contract
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/tri-blue.glb"
    ))
    .unwrap();
    let doc = load_gltf_bytes(&bytes).unwrap();
    assert!(doc.entities()[0].mesh.uv.is_empty());
    assert_eq!(doc.entities()[0].mesh.texture, None);
    assert!(doc.textures().is_empty());
}

// spec: GLTF-10
#[test]
fn image_segment_tampering_never_panics() {
    let glb = build_textured_glb();
    let sig = b"\x89PNG\r\n\x1a\n";
    let pos = glb
        .windows(sig.len())
        .position(|w| w == sig)
        .expect("the builder embedded PNG signature");
    // truncation band: PNG data chopped tail-ward segment by segment
    for cut in [pos + 1, pos + 8, pos + 20, glb.len() - 1] {
        let _ = load_gltf_bytes(&glb[..cut.min(glb.len())]);
    }
    // flip band: bytes toggled every 8 steps across the IDAT region (checksum/block-structure corruption face)
    for i in (pos + 12..glb.len()).step_by(8) {
        let mut v = glb.clone();
        v[i] ^= 0xA5;
        let _ = load_gltf_bytes(&v);
    }
}

/// True-source note: `resources/data/texquad.glb` = this builder's output (the capi/render_spec texture-chain
/// fixture consumption face). Regenerate: `cargo test -p visiaengine-io-gltf emit_textured_glb_fixture -- --ignored`
#[test]
#[ignore = "fixture generation (not validation)"]
fn emit_textured_glb_fixture() {
    let glb = build_textured_glb();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/texquad.glb"
    );
    std::fs::write(path, &glb).expect("write fixture");
    println!("wrote {path} ({} bytes)", glb.len());
}
