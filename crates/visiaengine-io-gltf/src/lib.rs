//! # visiaengine-io-gltf
//! glTF(GLB) → CPU 侧实体数据。零渲染依赖（io-* 族纪律，架构②）。
//! 行为契约：docs/sdd/io-gltf.md（GLTF-01..08）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use thiserror::Error;

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
}

impl GltfDocument {
    #[must_use]
    pub fn entities(&self) -> &[GltfEntity] {
        &self.entities
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

fn read_mesh<'a>(get: GetBuf<'a, '_>, prim: &gltf::Primitive<'a>) -> GltfMesh {
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
    GltfMesh {
        positions,
        normals,
        indices,
        base_color: prim.material().pbr_metallic_roughness().base_color_factor(),
    }
}

fn walk<'a>(
    get: GetBuf<'a, '_>,
    node: &gltf::Node<'a>,
    parent_world: &[[f64; 4]; 4],
    out: &mut Vec<GltfEntity>,
) {
    // Transform 合成由 crate 完成（T·R·S 或原样 Matrix），本侧仅 f64 升位+层级乘（GLTF-08 直通）
    let world = mul(parent_world, &mat4_f64(node.transform().matrix()));
    if let Some(mesh) = node.mesh() {
        for prim in mesh.primitives() {
            out.push(GltfEntity {
                name: node.name().map(String::from),
                mesh: read_mesh(get, &prim),
                world,
            });
        }
    }
    for child in node.children() {
        walk(get, &child, &world, out);
    }
}

/// 解析 GLB 文件为实体列表（场景根遍历+世界变换烘焙，primitive 级拆分）。
pub fn load_gltf(path: impl AsRef<std::path::Path>) -> Result<GltfDocument, IoError> {
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
    let gltf = gltf::Gltf::from_slice(&bytes).map_err(|e| IoError::Parse {
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
    let mut entities = Vec::new();
    let root = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    if let Some(scene) = doc.default_scene() {
        for node in scene.nodes() {
            walk(&get, &node, &root, &mut entities);
        }
    }
    Ok(GltfDocument { entities })
}
