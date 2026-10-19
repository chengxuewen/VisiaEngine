//! # visiaengine-io-gltf
//! glTF (GLB) -> CPU-side entity data. Zero render dependency (the io-* family discipline, architecture 2).
//! Behavior contract: docs/sdd/io-gltf.md (GLTF-01..08).

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use thiserror::Error;
use visiaengine_core::linear_to_srgb;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("file does not exist: {path}")]
    NotFound { path: String },
    #[error("GLB/JSON parse failed: {reason}")]
    Parse { reason: String },
    #[error(
        "v0 supports the GLB container only (.gltf + external resources supported in a later slice)"
    )]
    UnsupportedFormat,
    #[error("IO failure: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct GltfMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub base_color: [f32; 4],
    /// GLTF-11: TEXCOORD_0 (default = empty; equal-length needs no same-style placeholder discipline as GLTF-03 -- no padding).
    pub uv: Vec<[f32; 2]>,
    /// The **image slot** (textures() index) after resolving baseColorTexture; no texture /
    /// texCoord != 0 (v0 supports the first UV set only) / uri image = None.
    pub texture: Option<usize>,
    /// PBR factor collection (the specular composition of the render IR lives at M2; the io layer converts nothing).
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

/// GLTF-11: CPU-decoded products of embedded textures (RGBA8 fully expanded -- GPU compressed formats = the real PBR round).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureDesc {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GltfEntity {
    pub name: Option<String>,
    pub mesh: GltfMesh,
    /// World transform, column-major (translation = m[3], GLTF-02/08).
    pub world: [[f64; 4]; 4],
    /// V2.2: node-LOCAL TRS (GLTF-08 direct, un-multiplied) — the animation
    /// face writes this per-frame; parent chain composes engine-side.
    pub local: [[f64; 4]; 4],
    /// V2.2: parent entity INDEX within the same document (None = scene root);
    /// rigid hierarchy only (no skinning) — V2.4 scope split.
    pub parent: Option<usize>,
}

#[derive(Debug)]
pub struct GltfDocument {
    entities: Vec<GltfEntity>,
    textures: Vec<TextureDesc>,
}

impl GltfDocument {
    #[must_use]
    pub fn entities(&self) -> &[GltfEntity] {
        &self.entities
    }

    /// Decoded textures in images[] slot order (GLTF-11).
    #[must_use]
    pub fn textures(&self) -> &[TextureDesc] {
        &self.textures
    }
}

type GetBuf<'a, 'b> = &'b dyn Fn(gltf::buffer::Buffer<'a>) -> Option<&'a [u8]>;

/// Column-major 4x4 multiplication: (a * b).
fn mul(a: &[[f64; 4]; 4], b: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut out = [[0.0f64; 4]; 4];
    for (c, col) in out.iter_mut().enumerate() {
        for (r, cell) in col.iter_mut().enumerate() {
            *cell = (0..4).map(|k| a[k][r] * b[c][k]).sum();
        }
    }
    out
}

fn mat4_f64(m: [[f32; 4]; 4]) -> [[f64; 4]; 4] {
    // gltf's Transform::matrix() is COLUMN-major (glTF spec; crate doc: "4x4
    // transformation matrix in column-major order", translation in column 3)
    // -- same convention as the engine (REND-18); widen verbatim, no transpose.
    m.map(|col| col.map(f64::from))
}

fn read_v3<'a>(get: GetBuf<'a, '_>, acc: gltf::Accessor<'a>) -> Vec<[f32; 3]> {
    use gltf::accessor::{DataType, Dimensions};
    // pre-check the type then construct precisely (Iter::new debug_assert-panics on type mismatch in debug builds -- must not probe)
    if acc.data_type() != DataType::F32 || acc.dimensions() != Dimensions::Vec3 {
        return Vec::new();
    }
    gltf::accessor::util::Iter::<[f32; 3]>::new(acc, get).map_or(Vec::new(), |it| it.collect())
}

/// Indices read downgraded by source type u32 -> u16 -> u8 (real assets are mostly u16/u8).
fn read_v2<'a>(get: GetBuf<'a, '_>, acc: gltf::Accessor<'a>) -> Vec<[f32; 2]> {
    use gltf::accessor::{DataType, Dimensions};
    if acc.data_type() != DataType::F32 || acc.dimensions() != Dimensions::Vec2 {
        return Vec::new();
    }
    gltf::accessor::util::Iter::<[f32; 2]>::new(acc, get).map_or(Vec::new(), |it| it.collect())
}

/// Decode all embedded images into RGBA8 slots (GLTF-11). uri/data:URI images =
/// UnsupportedFormat-grade Err (the v0 contract forbids data:URI); decode failure = Parse (consistent with FastFail,
/// no silent texture loss). Zero gltf feature changes [FFI-R:Momus-B2]: goes through the json root.
fn decode_images(root: &gltf::json::Root, blob: &[u8]) -> Result<Vec<TextureDesc>, IoError> {
    root.images
        .iter()
        .enumerate()
        .map(|(i, img)| {
            let Some(bv_index) = img.buffer_view else {
                return Err(IoError::UnsupportedFormat);
            };
            let bv = &root.buffer_views[bv_index.value()];
            // USize64(pub u64): ported units unify through u64 -> usize try_from
            let start =
                usize::try_from(bv.byte_offset.map_or(0, |o| o.0)).map_err(|_| IoError::Parse {
                    reason: format!("image[{i}] byte_offset out of bounds"),
                })?;
            let len = usize::try_from(bv.byte_length.0).map_err(|_| IoError::Parse {
                reason: format!("image[{i}] byte_length out of bounds"),
            })?;
            let bytes = blob
                .get(
                    start..start.checked_add(len).ok_or_else(|| IoError::Parse {
                        reason: format!("image[{i}] range overflow"),
                    })?,
                )
                .ok_or_else(|| IoError::Parse {
                    reason: format!("image[{i}] outside the blob range"),
                })?;
            let img = image::load_from_memory(bytes).map_err(|e| IoError::Parse {
                reason: format!("image[{i}] decode failed: {e}"),
            })?;
            let (width, height) = (img.width(), img.height());
            Ok(TextureDesc {
                rgba: img.to_rgba8().into_raw(),
                width,
                height,
            })
        })
        .collect()
}

fn read_indices<'a>(get: GetBuf<'a, '_>, acc: gltf::Accessor<'a>) -> Vec<u32> {
    use gltf::accessor::{DataType, Dimensions};
    if acc.dimensions() != Dimensions::Scalar {
        return Vec::new();
    }
    // index source types u32/u16/u8 dispatched precisely (real assets are mostly u16/u8)
    match acc.data_type() {
        DataType::U32 => {
            gltf::accessor::util::Iter::<u32>::new(acc, get).map(|it| it.collect::<Vec<u32>>())
        }
        DataType::U16 => gltf::accessor::util::Iter::<u16>::new(acc, get)
            .map(|it| it.map(u32::from).collect::<Vec<u32>>()),
        DataType::U8 => gltf::accessor::util::Iter::<u8>::new(acc, get)
            .map(|it| it.map(u32::from).collect::<Vec<u32>>()),
        _ => None,
    }
    .unwrap_or_default()
}

fn read_mesh<'a>(
    get: GetBuf<'a, '_>,
    prim: &gltf::Primitive<'a>,
    tex_slot: &dyn Fn(usize) -> Option<usize>,
) -> GltfMesh {
    let positions = prim
        .get(&gltf::Semantic::Positions)
        .map_or(Vec::new(), |acc| read_v3(get, acc));
    // GLTF-03: missing normals / length mismatch -> equal-length zero placeholders (the fallback signal at the shading end)
    let normals = match prim.get(&gltf::Semantic::Normals) {
        Some(acc) => {
            let n = read_v3(get, acc);
            if n.len() == positions.len() {
                n
            } else {
                vec![[0.0; 3]; positions.len()]
            }
        }
        None => vec![[0.0; 3]; positions.len()],
    };
    let indices = prim
        .indices()
        .map_or(Vec::new(), |acc| read_indices(get, acc));
    let pbr = prim.material().pbr_metallic_roughness();
    // GLTF-11: baseColorTexture -> image slot; texCoord != 0 (v0 supports the first UV set only)
    // and missing texture = None (material kept, texture dropped -- contract note)
    let texture = pbr
        .base_color_texture()
        .filter(|info| info.tex_coord() == 0)
        .and_then(|info| tex_slot(info.texture().index()));
    let uv = prim
        .get(&gltf::Semantic::TexCoords(0))
        .map_or(Vec::new(), |acc| read_v2(get, acc));
    GltfMesh {
        positions,
        normals,
        indices,
        base_color: linear_to_srgb(pbr.base_color_factor()), // glTF factor = linear (spec); the IR face = sRGB convention, one inverse transform back home
        uv,
        texture,
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
    }
}

#[allow(clippy::too_many_arguments)]
fn walk<'a>(
    get: GetBuf<'a, '_>,
    node: &gltf::Node<'a>,
    parent_world: &[[f64; 4]; 4],
    out: &mut Vec<GltfEntity>,
    rep: &mut LoadReport,
    tex_slot: &dyn Fn(usize) -> Option<usize>,
    parent: Option<usize>,
) {
    // Transform composition is done by the crate (T*R*S or the Matrix verbatim); this side only widens to f64 + multiplies the hierarchy (GLTF-08 passthrough)
    let world = mul(parent_world, &mat4_f64(node.transform().matrix()));
    let first_entity = out.len(); // V2.2: index children will parent to
    if let Some(mesh) = node.mesh() {
        for prim in mesh.primitives() {
            // GLTF-09: mode filter (once smuggled in as fake triangles) + skip empty positions
            if prim.mode() != gltf::mesh::Mode::Triangles {
                rep.skipped_non_triangle += 1;
                continue;
            }
            let data = read_mesh(get, &prim, tex_slot);
            if data.positions.is_empty() {
                rep.skipped_unreadable_positions += 1;
                continue;
            }
            out.push(GltfEntity {
                name: node.name().map(String::from),
                mesh: data,
                world,
                local: mat4_f64(node.transform().matrix()),
                parent: if first_entity == out.len() {
                    parent
                } else {
                    Some(first_entity)
                },
            });
        }
    }
    // V2.2 rigid parent chain: children hang off this node's FIRST mesh
    // entity index captured before the prim loop (multi-prim nodes share the
    // node transform); child-of-meshless-node = keeps the incoming parent
    // (grandparent attachment — honest scope note for the loader clause).
    for child in node.children() {
        walk(get, &child, &world, out, rep, tex_slot, parent);
    }
}

/// Primitive-level skip report (GLTF-09, [E3D:A4] port).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// mode != Triangles (POINTS/LINES must not smuggle into the pipeline as fake triangles)
    pub skipped_non_triangle: u32,
    /// POSITION missing / wrong type / empty read
    pub skipped_unreadable_positions: u32,
}

impl LoadReport {
    /// Skip total.
    #[must_use]
    pub fn total_skipped(&self) -> u32 {
        self.skipped_non_triangle + self.skipped_unreadable_positions
    }
}

/// Parse a GLB file into an entity list (scene-root traversal + world transform baking, split at primitive level).
/// GLTF-09: non-TRIANGLES primitives and empty-positions primitives are always filtered (correctness);
/// the report needs the explicit entry [`load_gltf_with_report`].
pub fn load_gltf(path: impl AsRef<std::path::Path>) -> Result<GltfDocument, IoError> {
    Ok(load_gltf_with_report(path)?.0)
}

/// GLB bytes parsing (no filesystem -- the bytes-first main entry for batch 7 wasm transport [FFI-R:EP-附];
/// the GLTF-09 filter/report shares its source with the path entry).
pub fn load_gltf_bytes(data: &[u8]) -> Result<GltfDocument, IoError> {
    Ok(load_gltf_bytes_with_report(data)?.0)
}

/// The bytes-face entry with a report (the GLTF-09 main body).
pub fn load_gltf_bytes_with_report(bytes: &[u8]) -> Result<(GltfDocument, LoadReport), IoError> {
    let gltf = gltf::Gltf::from_slice(bytes).map_err(|e| IoError::Parse {
        reason: e.to_string(),
    })?;
    let doc = &gltf.document;
    // v0 boundary: external buffer uri refuses parsing (the GLB embedded blob is the only data source)
    if doc
        .buffers()
        .any(|b| !matches!(b.source(), gltf::buffer::Source::Bin))
    {
        return Err(IoError::UnsupportedFormat);
    }
    let blob = gltf.blob.unwrap_or_default();
    let get = |buf: gltf::buffer::Buffer<'_>| -> Option<&[u8]> {
        matches!(buf.source(), gltf::buffer::Source::Bin).then_some(blob.as_slice())
    };
    let jroot = doc.as_json();
    let textures = decode_images(jroot, &blob)?;
    let tex_slot = |t: usize| -> Option<usize> { jroot.textures.get(t).map(|x| x.source.value()) };
    let mut entities = Vec::new();
    let mut rep = LoadReport::default();
    let root = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    if let Some(scene) = doc.default_scene() {
        for node in scene.nodes() {
            walk(&get, &node, &root, &mut entities, &mut rep, &tex_slot, None);
        }
    }
    Ok((GltfDocument { entities, textures }, rep))
}

/// The path entry with a skip report (GLTF-09; the NotFound/IO semantics live at this layer, the GLTF-05 contract position).
pub fn load_gltf_with_report(
    path: impl AsRef<std::path::Path>,
) -> Result<(GltfDocument, LoadReport), IoError> {
    let path = path.as_ref();
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(IoError::NotFound {
                path: path.display().to_string(),
            });
        }
        Err(e) => return Err(IoError::Io { source: e }),
    };
    load_gltf_bytes_with_report(&bytes)
}
