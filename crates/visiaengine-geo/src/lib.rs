//! # visiaengine-geo
//! GeoJSON → 3857 平面 f64 几何 + 最小样式。零渲染依赖（host 侧做 D7 origin 分解）。
//! 行为契约：docs/sdd/geo.md（GEO-01..08；细分/样式 GEO-09..14 随 H2）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use thiserror::Error;
use visiaengine_core::Vec3;

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("文件不存在: {path}")]
    NotFound { path: String },
    #[error("GeoJSON 解析失败: {reason}")]
    Parse { reason: String },
    #[error("纬度 {lat}° 超出 Web Mercator 域 ±85.0511°")]
    InvalidCoord { lat: f64 },
    #[error("IO 失败: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

/// 细分前几何（3857 米，z 恒 0）。
#[derive(Clone, Debug, PartialEq)]
pub enum GeoKind {
    Point(Vec3),
    MultiPoint(Vec<Vec3>),
    Line(Vec<Vec3>),
    Poly {
        ext: Vec<[f64; 2]>,
        holes: Vec<Vec<[f64; 2]>>,
    },
}

/// simplestyle 六键子集（GEO-12/13，H2 消费）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StyleRecord {
    pub fill: [f32; 4],
    pub fill_opacity: f32,
    pub stroke: [f32; 4],
    pub stroke_width_m: f32,
    pub marker_color: [f32; 4],
    pub radius_m: f32,
}

#[derive(Clone, Debug)]
pub struct GeoFeature {
    pub name: Option<String>,
    pub kind: GeoKind,
    pub style: StyleRecord,
    pub world_bbox: Option<[f64; 4]>,
}

#[derive(Debug)]
pub struct GeoDocument {
    features: Vec<GeoFeature>,
}

impl GeoDocument {
    #[must_use]
    pub fn features(&self) -> &[GeoFeature] {
        &self.features
    }

    /// 全层 3857 bbox 并集（空层 None，GEO-05）。
    #[must_use]
    pub fn layer_bbox(&self) -> Option<[f64; 4]> {
        todo!("H1 GREEN")
    }
}

/// WGS84 经纬 → EPSG:3857 球面式（GEO-02/08）。
#[must_use]
pub fn web_mercator(lon: f64, lat: f64) -> Option<[f64; 2]> {
    let _ = (lon, lat);
    todo!("H1 GREEN")
}

/// 解析 GeoJSON 文件（仅 FeatureCollection/Feature/单几何；GC 展平）。
pub fn load_geojson(path: impl AsRef<std::path::Path>) -> Result<GeoDocument, GeoError> {
    let _ = path;
    todo!("H1 GREEN")
}

/// 从字节解析（GEO-06/07/08 的无文件入口；load_geojson = read + 本函数）。
pub fn parse_geojson(bytes: &[u8]) -> Result<GeoDocument, GeoError> {
    let _ = bytes;
    todo!("H1 GREEN")
}
