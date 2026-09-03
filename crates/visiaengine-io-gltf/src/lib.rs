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
    /// 世界变换，列主序，平移=m[3]（GLTF-02/08）。
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

/// 解析 GLB 文件为实体列表（场景节点世界变换烘焙，prim 级拆分）。
pub fn load_gltf(path: impl AsRef<std::path::Path>) -> Result<GltfDocument, IoError> {
    let _ = path;
    todo!("G1 GREEN")
}
