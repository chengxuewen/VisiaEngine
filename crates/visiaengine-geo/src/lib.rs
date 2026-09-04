//! # visiaengine-geo
//! GeoJSON → 3857 平面 f64 几何 + 最小样式。零渲染依赖（host 侧做 D7 origin 分解）。
//! 行为契约：docs/sdd/geo.md（GEO-01..08；细分/样式 GEO-09..14 随 H2）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

mod style;
mod tess;

pub use style::parse_color;
pub use tess::{PartKind, TessPart, tessellate};

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
/// 默认蓝（GEO-13 语义）；六键解析见 style 模块。
impl Default for StyleRecord {
    fn default() -> Self {
        default_style()
    }
}

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
        let mut it = self.features.iter().filter_map(|f| f.world_bbox);
        let first = it.next()?;
        Some(it.fold(first, |a, b| {
            [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        }))
    }
}

/// WGS84 经纬 → EPSG:3857 球面式（GEO-02/08；|φ|>85.0511° 发散域拒）。
#[must_use]
pub fn web_mercator(lon: f64, lat: f64) -> Option<[f64; 2]> {
    const R: f64 = 6378137.0;
    const MAX_LAT: f64 = 85.051_128_779_806_6;
    if !(-180.0..=180.0).contains(&lon) || !(-MAX_LAT..=MAX_LAT).contains(&lat) {
        return None;
    }
    Some([
        R * lon.to_radians(),
        R * ((std::f64::consts::FRAC_PI_4 + lat.to_radians() / 2.0).tan()).ln(),
    ])
}

/// 默认样式（GEO-13；simplestyle 六键解析属 H2）。
#[must_use]
pub const fn default_style() -> StyleRecord {
    StyleRecord {
        fill: [0.0, 0.45, 1.0, 1.0],
        fill_opacity: 1.0,
        stroke: [1.0, 1.0, 1.0, 1.0],
        stroke_width_m: 3.0,
        marker_color: [0.0, 0.45, 1.0, 1.0],
        radius_m: 4.0,
    }
}

fn proj(p: &geojson::Position) -> Result<[f64; 2], GeoError> {
    let s = p.as_slice();
    web_mercator(s[0], s[1]).ok_or(GeoError::InvalidCoord { lat: s[1] })
}

fn pt(p: &geojson::Position) -> Result<Vec3, GeoError> {
    let [x, y] = proj(p)?;
    Ok(Vec3::new(x, y, 0.0))
}

fn ring(v: &[geojson::Position]) -> Result<Vec<[f64; 2]>, GeoError> {
    v.iter().map(proj).collect()
}

fn push_bbox(mins: &mut [f64; 2], maxs: &mut [f64; 2], x: f64, y: f64) {
    mins[0] = mins[0].min(x);
    mins[1] = mins[1].min(y);
    maxs[0] = maxs[0].max(x);
    maxs[1] = maxs[1].max(y);
}

fn kind_of(gj: &geojson::Geometry) -> Result<GeoKind, GeoError> {
    let bad = |why: &str| GeoError::Parse {
        reason: why.to_string(),
    };
    Ok(match &gj.value {
        geojson::GeometryValue::Point { coordinates } => GeoKind::Point(pt(coordinates)?),
        geojson::GeometryValue::MultiPoint { coordinates } => {
            GeoKind::MultiPoint(coordinates.iter().map(pt).collect::<Result<_, _>>()?)
        }
        geojson::GeometryValue::LineString { coordinates } => {
            GeoKind::Line(coordinates.iter().map(pt).collect::<Result<_, _>>()?)
        }
        geojson::GeometryValue::MultiLineString { coordinates } => match coordinates.as_slice() {
            [single] => GeoKind::Line(single.iter().map(pt).collect::<Result<_, _>>()?),
            _ => return Err(bad("MultiLineString len>1 属 H2 拆分（多部件策略）")),
        },
        geojson::GeometryValue::Polygon { coordinates } => {
            let Some((ext, holes)) = coordinates.split_first() else {
                return Err(bad("empty polygon rings"));
            };
            GeoKind::Poly {
                ext: ring(ext)?,
                holes: holes
                    .iter()
                    .map(|r| ring(r.as_slice()))
                    .collect::<Result<_, _>>()?,
            }
        }
        _ => return Err(bad("MultiPolygon/GC 在文档层处理")),
    })
}

fn feature_from(
    gj: &geojson::Geometry,
    name: Option<String>,
    props: Option<&serde_json::Map<String, serde_json::Value>>,
) -> Result<GeoFeature, GeoError> {
    // 投影单点：kind_of 内 pt/ring 经 proj()（解析边界 4326→3857，GEO-08 同路拒绝）
    let kind = kind_of(gj)?;
    let mut mins = [f64::MAX; 2];
    let mut maxs = [f64::MIN; 2];
    let mut acc = |x: f64, y: f64| push_bbox(&mut mins, &mut maxs, x, y);
    match &kind {
        GeoKind::Point(p) => acc(p.x, p.y),
        GeoKind::MultiPoint(ps) | GeoKind::Line(ps) => {
            for p in ps {
                acc(p.x, p.y);
            }
        }
        GeoKind::Poly { ext, holes } => {
            for p in ext.iter().chain(holes.iter().flatten()) {
                acc(p[0], p[1]);
            }
        }
    }
    let style = props.map_or_else(default_style, |p| {
        style::style_from_props(&serde_json::Value::Object(p.clone()))
    });
    Ok(GeoFeature {
        name,
        kind,
        style,
        world_bbox: Some([mins[0], mins[1], maxs[0], maxs[1]]),
    })
}

/// 从字节解析（GEO-06/07/08 无文件入口）。
pub fn parse_geojson(bytes: &[u8]) -> Result<GeoDocument, GeoError> {
    let root = geojson::GeoJson::from_reader(std::io::Cursor::new(bytes)).map_err(|e| {
        GeoError::Parse {
            reason: e.to_string(),
        }
    })?;
    let mut features = Vec::new();
    let mut feed = |f: &geojson::Feature| -> Result<(), GeoError> {
        let name = f
            .properties
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(geojson::JsonValue::as_str)
            .map(String::from);
        let Some(g) = &f.geometry else { return Ok(()) };
        if matches!(g.value, geojson::GeometryValue::GeometryCollection { .. }) {
            for sub in flatten_gc(g) {
                features.push(sub?);
            }
            return Ok(());
        }
        features.push(feature_from(g, name, f.properties.as_ref())?);
        Ok(())
    };
    match root {
        geojson::GeoJson::FeatureCollection(fc) => {
            for f in &fc.features {
                feed(f)?;
            }
        }
        geojson::GeoJson::Feature(f) => feed(&f)?,
        geojson::GeoJson::Geometry(g) => {
            for sub in flatten_gc(&g) {
                features.push(sub?);
            }
        }
    }
    Ok(GeoDocument { features })
}

fn flatten_gc(g: &geojson::Geometry) -> Vec<Result<GeoFeature, GeoError>> {
    let geojson::GeometryValue::GeometryCollection { geometries } = &g.value else {
        return vec![feature_from(g, None, None)];
    };
    geometries
        .iter()
        .map(|s| {
            if matches!(s.value, geojson::GeometryValue::GeometryCollection { .. }) {
                Err(GeoError::Parse {
                    reason: "nested GeometryCollection 属 H2 展平策略".into(),
                })
            } else {
                feature_from(s, None, None)
            }
        })
        .collect()
}

/// 从 JSON 对象字符串解析样式（测试/宿主便利口；非法回默认）。
#[must_use]
pub fn parse_style(props_json: &str) -> StyleRecord {
    serde_json::from_str(props_json)
        .map(|v| style::style_from_props(&v))
        .unwrap_or_default()
}

/// 解析 GeoJSON 文件。
pub fn load_geojson(path: impl AsRef<std::path::Path>) -> Result<GeoDocument, GeoError> {
    let path = path.as_ref();
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(GeoError::NotFound {
                path: path.display().to_string(),
            });
        }
        Err(e) => return Err(GeoError::Io { source: e }),
    };
    parse_geojson(&bytes)
}
