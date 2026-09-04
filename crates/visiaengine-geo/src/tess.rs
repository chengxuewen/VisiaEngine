//! 细分（GEO-09..11/14）：GeoKind → 三角形 parts。
//! D7 接口纪律：输入 local 坐标（解析边界已投影；origin 分解在上传方）。

#[allow(unused_imports)]
use lyon_tessellation::{
    geometry_builder::VertexBuffers, FillOptions, FillTessellator, StrokeOptions,
    StrokeTessellator,
};

use crate::{GeoError, GeoKind, StyleRecord};

/// part 用途判别（消费端绑 material 色）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartKind {
    Fill,
    Stroke,
    Marker,
}

/// 一块可上 GPU 的三角网格（positions 为 local x,y，z=0）。
#[derive(Clone, Debug)]
pub struct TessPart {
    pub kind: PartKind,
    pub color: [f32; 4],
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

/// 几何 → 细分 parts（Poly 含洞 / Line / Point·MultiPoint 圆点方化）。
pub fn tessellate(kind: &GeoKind, style: &StyleRecord) -> Result<Vec<TessPart>, GeoError> {
    let _ = (kind, style);
    todo!("H2 GREEN")
}
