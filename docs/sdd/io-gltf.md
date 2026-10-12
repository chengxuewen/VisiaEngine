# visiaengine-io-gltf behavioral contract (SDD)

> Clause header lines `## GLTF-NN:` are the traceability anchors; tests carry `// spec: GLTF-NN`.
> Matrix convention: `world: [[f64;4];4]` **column-major** (isomorphic to glTF/glam), translation is `m[3][0..3]`.
> Asset discipline: fixtures are procedurally generated glb (resources/data/*.glb, self-produced in this repo, zero third-party licensing surface).
> v0 format surface: GLB only; `.gltf` + external resources -> `UnsupportedFormat` (multi-file support is a later slice).

## GLTF-01: parses_glb_fixture
`load_gltf("resources/data/tri-blue.glb")` -> 1 entity, positions.len()==4, indices.len()==6, name=Some("Quad").

## GLTF-02: node_hierarchy_transform_baked
hierarchy.glb two nodes (root translate(1,2,3) x leaf translate(10,0,0)) -> the entity world matrix translation column == (11,2,3) (f64 cumulative multiply baked in; scene expansion does not preserve the node tree).

## GLTF-03: missing_normals_default_or_generated
prim1 of twoprim.glb (no NORMAL attribute) -> normals.len()==positions.len() and all zeros (a fallback signal for the shading side, not a missing field).

## GLTF-04: base_color_from_material
baseColorFactor (glTF spec = linear) is normalized into the IR plane via `linear_to_srgb` (CORE-16 host plane = sRGB convention; the backend upload throat converts back to linear, round-trip ≤2e-7 lossless under 8-bit quantization): tri-blue=[0.1,0.2,0.9] -> sRGB [0.3492,0.4845,0.9547]; twoprim's two entities are pure primary-color endpoints [1,0,0,1]/[0,1,0,1], bit-invariant.

## GLTF-05: multi_primitive_counts_consistent
twoprim.glb -> 2 entities (primitives split independently), positions/indices counts equal the fixture construction item-by-item (3/6 and 3/6).

## GLTF-06: corrupt_file_err_not_panic
GLB truncated byte stream -> `IoError::Parse`, zero panic, zero unwrap.

## GLTF-07: missing_file_io_err
Nonexistent path -> `IoError::NotFound{}`.

## GLTF-08: y_up_orientation_preserved
The loader does zero coordinate-system rewriting: the hierarchy world translation z==3.0 comes out unchanged (no implicit Z-up sign flip — industry conversions belong to the geo slice).

## GLTF-09: primitive mode filtering + skip report
`load_gltf_with_report(path)` -> `(doc, LoadReport)`: primitives with mode≠Triangles are skipped and counted in `skipped_non_triangle` (they no longer sneak into the entity table as fake triangles — the correctness fix takes effect on the old entry as well), missing/unreadable/empty POSITION is counted in `skipped_unreadable_positions`; the `load_gltf` signature is unchanged = a wrapper that drops the report.

Coverage extension (GLTF-11 era): the no-panic contract over the fuzz surface includes **the image decoding segment** — a deterministic mutation band (PNG signature location + truncation + IDAT-region flip) alongside random-stream/header mutation as the test surface.

## GLTF-10: GLB decoding panics on no input (property surface)
`load_gltf_bytes` over arbitrary byte streams, arbitrary GLB header combinations (magic/version/total/chunk mutation), and truncation bands of the legal fixture: zero panic — Err is legal, panic is a contract violation. proptest 512 cases × 2 + one deterministic truncation-band test.

## GLTF-11: uv/texture/PBR factor reading
`GltfMesh` gains `uv` (TEXCOORD_0, default = empty, no placeholder), `texture` (baseColorTexture resolved through textures[].source into an **images slot index**; `texCoord≠0` is treated as no texture = material kept, texture dropped), `metallic_factor`/`roughness_factor` (stored as-is, zero conversion — the specular combination lives in the render IR). `GltfDocument.textures()` outputs RGBA8 decoded slots in images[] order (embedded-bufferView images; **data:URI / external images = UnsupportedFormat**, the v0 boundary); decode failure = Parse (FastFail consistency, silent texture dropping forbidden). The fixture is an in-test procedural builder (chunk 4-byte padding + POSITION min/max + complete accessors, self-check `from_slice ok` before any assertion).

## IO-17: RADIANCE .hdr (RGBE) decode (N5, visiaengine-io-hdr crate)
`decode_hdr(bytes) -> HdrImage{width,height,pixels:Vec<[f32;3]>}` (row-major top-down, linear-domain values may exceed 1.0 = the essence of HDR). Header contract: the magic line carries the `#?` prefix (`#?RADIANCE` is the typical form); `FORMAT=32-bit_rle_rgbe` is required (any other FORMAT value = BadFormat; FORMAT absent = BadFormat); unknown key lines (EXPOSURE/PRIMARIES/SOFTWARE…) are skipped; an empty line = the standard separator between meta and the resolution line (keep scanning); the resolution line is the fixed token `-Y H +X W` (sign/order variants = BadResolution; H/W = 0 = BadResolution). Scanline dual-layout per-line sniffing: the new RLE form (magic `2,2,h,l` — h/l = high/low bytes of the scanline count; then R/G/B/E four channel-plane run streams: a count byte ≥128 = (b−128) literals follow, <128 = the next byte repeats b times) + the old flat form (w RGBE quads; w<8 or w>32767 follows the flat convention per spec). Decode formula: f = (c/256)*2^(E−128) (one shared exponent per pixel; E=0 encodes zero). Five error classes: `BadMagic`/`BadFormat`/`BadResolution`/`Truncated{need,at}` (data-segment truncation reports the exact deficit)/`BadRle{at}` (count 0, a run crossing the line boundary). Fixture source of truth = `resources/data/demo_sky.hdr` (64×32 RLE gradient sky light 0.2..3.0, 960 pixels >1.0; generator = `scripts/gen_hdr.py` stdlib-only, self-produced, zero licensing surface); tests = hand-built byte fixtures (same discipline as io-tiles) 8 gates + 1 fixture round-trip gate.
