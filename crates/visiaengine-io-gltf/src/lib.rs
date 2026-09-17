//! # visiaengine-io-gltf
//! glTF(GLB) → CPU 侧实体数据。零渲染依赖（io-* 族纪律，架构②）。
//! 行为契约：docs/sdd/io-gltf.md（GLTF-01..08）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use thiserror::Error;
use visiaengine_core::linear_to_srgb;

#[derive(Error, Debug)]
pub enum IoError {
    #[error("文件不存在: {path}")]
    NotFound { path: String },
    #[error("GLB/JSON 解析失败: {reason}")]
    Parse { reason: String },
    #[error("v0 仅支持 GLB 容器（.gltf+外链资源后续片支持）")]
    UnsupportedFormat,
    #[error("IO 失败: {source}")]
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
    /// GLTF-11：TEXCOORD_0（缺省=空，等长由 GLTF-03 同款占位纪律免——不填充）。
    pub uv: Vec<[f32; 2]>,
    /// baseColorTexture 解析后的 **image 槽位**（textures() 下标）；无纹理/
    /// texCoord≠0（v0 只支持第一套 UV）/uri 图= None。
    pub texture: Option<usize>,
    /// PBR 因子收纳（render IR 的 specular 组合住 M2；io 层零换算）。
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

/// GLTF-11：embedded 纹理 CPU 解码产物（RGBA8 全展开——GPU 压缩格式=真 PBR 轮）。
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
    /// 世界变换，列主序（平移=m[3]，GLTF-02/08）。
    pub world: [[f64; 4]; 4],
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

    /// images[] 槽位序的解码纹理（GLTF-11）。
    #[must_use]
    pub fn textures(&self) -> &[TextureDesc] {
        &self.textures
    }
}

type GetBuf<'a, 'b> = &'b dyn Fn(gltf::buffer::Buffer<'a>) -> Option<&'a [u8]>;

/// 列主序 4x4 乘法：(a·b)。
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
    m.map(|col| col.map(f64::from))
}

fn read_v3<'a>(get: GetBuf<'a, '_>, acc: gltf::Accessor<'a>) -> Vec<[f32; 3]> {
    use gltf::accessor::{DataType, Dimensions};
    // 预检类型再精确构造（Iter::new 对类型不合者在 debug 构建 debug_assert panic——不得探测）
    if acc.data_type() != DataType::F32 || acc.dimensions() != Dimensions::Vec3 {
        return Vec::new();
    }
    gltf::accessor::util::Iter::<[f32; 3]>::new(acc, get).map_or(Vec::new(), |it| it.collect())
}

/// 索引按 u32→u16→u8 源类型降级读取（真实资产多为 u16/u8）。
fn read_v2<'a>(get: GetBuf<'a, '_>, acc: gltf::Accessor<'a>) -> Vec<[f32; 2]> {
    use gltf::accessor::{DataType, Dimensions};
    if acc.data_type() != DataType::F32 || acc.dimensions() != Dimensions::Vec2 {
        return Vec::new();
    }
    gltf::accessor::util::Iter::<[f32; 2]>::new(acc, get).map_or(Vec::new(), |it| it.collect())
}

/// 解码全部 embedded images 为 RGBA8 槽位（GLTF-11）。uri/data:URI 图=
/// UnsupportedFormat 级 Err（v0 禁 data:URI 契约）；解码失败=Parse（FastFail
/// 一致，不静默丢纹理）。零 gltf feature 改动[FFI-R:Momus-B2]：走 json 根。
fn decode_images(root: &gltf::json::Root, blob: &[u8]) -> Result<Vec<TextureDesc>, IoError> {
    root.images
        .iter()
        .enumerate()
        .map(|(i, img)| {
            let Some(bv_index) = img.buffer_view else {
                return Err(IoError::UnsupportedFormat);
            };
            let bv = &root.buffer_views[bv_index.value()];
            // USize64(pub u64)：端口径统一走 u64→usize try_from
            let start =
                usize::try_from(bv.byte_offset.map_or(0, |o| o.0)).map_err(|_| IoError::Parse {
                    reason: format!("image[{i}] byte_offset 越界"),
                })?;
            let len = usize::try_from(bv.byte_length.0).map_err(|_| IoError::Parse {
                reason: format!("image[{i}] byte_length 越界"),
            })?;
            let bytes = blob
                .get(
                    start..start.checked_add(len).ok_or_else(|| IoError::Parse {
                        reason: format!("image[{i}] 区间溢出"),
                    })?,
                )
                .ok_or_else(|| IoError::Parse {
                    reason: format!("image[{i}] 出 blob 范围"),
                })?;
            let img = image::load_from_memory(bytes).map_err(|e| IoError::Parse {
                reason: format!("image[{i}] 解码失败: {e}"),
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
    // 索引源类型 u32/u16/u8 精确分派（真实资产多为 u16/u8）
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
    // GLTF-03：缺法线/长度不齐 → 等长零占位（着色端兜底信号）
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
    // GLTF-11：baseColorTexture→image 槽位；texCoord≠0（v0 只支持第一套 UV）
    // 与缺纹理= None（材质保留，纹理丢弃——契约注记）
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
        base_color: linear_to_srgb(pbr.base_color_factor()), // glTF factor=线性(规范)；IR 面=sRGB 约定，反变换一次归位
        uv,
        texture,
        metallic_factor: pbr.metallic_factor(),
        roughness_factor: pbr.roughness_factor(),
    }
}

fn walk<'a>(
    get: GetBuf<'a, '_>,
    node: &gltf::Node<'a>,
    parent_world: &[[f64; 4]; 4],
    out: &mut Vec<GltfEntity>,
    rep: &mut LoadReport,
    tex_slot: &dyn Fn(usize) -> Option<usize>,
) {
    // Transform 合成由 crate 完成（T·R·S 或原样 Matrix），本侧仅 f64 升位+层级乘（GLTF-08 直通）
    let world = mul(parent_world, &mat4_f64(node.transform().matrix()));
    if let Some(mesh) = node.mesh() {
        for prim in mesh.primitives() {
            // GLTF-09：模式过滤（曾以假三角混入）+ 空 positions 跳过
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
            });
        }
    }
    for child in node.children() {
        walk(get, &child, &world, out, rep, tex_slot);
    }
}

/// primitive 级跳过报告（GLTF-09，[E3D:A4] 移植）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// mode ≠ Triangles（POINTS/LINES 不得以假三角混入管线）
    pub skipped_non_triangle: u32,
    /// POSITION 缺失/类型不合/空读取
    pub skipped_unreadable_positions: u32,
}

impl LoadReport {
    /// 跳过总数。
    #[must_use]
    pub fn total_skipped(&self) -> u32 {
        self.skipped_non_triangle + self.skipped_unreadable_positions
    }
}

/// 解析 GLB 文件为实体列表（场景根遍历+世界变换烘焙，primitive 级拆分）。
/// GLTF-09：非 TRIANGLES 原语与空 positions 原语始终过滤（正确性），
/// 报告需显式入口 [`load_gltf_with_report`]。
pub fn load_gltf(path: impl AsRef<std::path::Path>) -> Result<GltfDocument, IoError> {
    Ok(load_gltf_with_report(path)?.0)
}

/// GLB 字节解析（无文件系统——批 7 wasm 传输的 bytes-first 主体口 [FFI-R:EP-附]；
/// GLTF-09 过滤/报告与 path 口同源）。
pub fn load_gltf_bytes(data: &[u8]) -> Result<GltfDocument, IoError> {
    Ok(load_gltf_bytes_with_report(data)?.0)
}

/// 字节面的带报告入口（GLTF-09 主体）。
pub fn load_gltf_bytes_with_report(bytes: &[u8]) -> Result<(GltfDocument, LoadReport), IoError> {
    let gltf = gltf::Gltf::from_slice(bytes).map_err(|e| IoError::Parse {
        reason: e.to_string(),
    })?;
    let doc = &gltf.document;
    // v0 边界：外链 buffer uri 拒解析（GLB 内嵌 blob 是唯一数据源）
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
            walk(&get, &node, &root, &mut entities, &mut rep, &tex_slot);
        }
    }
    Ok((GltfDocument { entities, textures }, rep))
}

/// 带跳过报告的 path 入口（GLTF-09；NotFound/IO 语义在此层，GLTF-05 契约位）。
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
