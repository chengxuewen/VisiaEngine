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
    #[error("坐标分量非有限（NaN/inf）")]
    NonFinite,
    #[error("不支持几何变体: {what}")]
    Unsupported { what: &'static str },
    #[error("GeometryCollection 嵌套 ≥2 层")]
    NestedCollection,
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
impl GeoKind {
    /// 平移几何（GEO-14：world→local 化 = shifted(-origin)，杜绝消费方手改顶点）。
    #[must_use]
    pub fn shifted(&self, d: [f64; 2]) -> Self {
        let s = |v: &Vec3| Vec3::new(v.x + d[0], v.y + d[1], v.z);
        match self {
            Self::Point(p) => Self::Point(s(p)),
            Self::MultiPoint(ps) => Self::MultiPoint(ps.iter().map(s).collect()),
            Self::Line(ps) => Self::Line(ps.iter().map(s).collect()),
            Self::Poly { ext, holes } => Self::Poly {
                ext: ext.iter().map(|p| [p[0] + d[0], p[1] + d[1]]).collect(),
                holes: holes
                    .iter()
                    .map(|r| r.iter().map(|p| [p[0] + d[0], p[1] + d[1]]).collect())
                    .collect(),
            },
        }
    }
}

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
    if !s[0].is_finite() || !s[1].is_finite() {
        return Err(GeoError::NonFinite);
    }
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
    let bad = |why: &'static str| GeoError::Unsupported { what: why };
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

/// 修复策略（[E3D:A4] 移植）：FastFail=单件脏几何整文档 Err（既有契约）；
/// Lenient=逐件丢弃+分型计数（GEO-15/16）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairPolicy {
    Lenient,
    FastFail,
}

/// 宽松加载报告：丢弃原因的分型计数（GEO-15）。分类在产生点定死
/// （typed GeoError 变体映射），不反解 reason 字符串。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// lon/lat 出 Web Mercator 域
    pub dropped_out_of_bounds: u32,
    /// 坐标分量 NaN/inf
    pub dropped_non_finite: u32,
    /// MultiPolygon/多部件 MLS/空环等未就绪变体
    pub dropped_unsupported: u32,
    /// `geometry: null` 的 feature（Lenient 下不再静默）
    pub dropped_null_geometry: u32,
    /// GeometryCollection 嵌套 ≥2 层
    pub dropped_nested_collection: u32,
}

impl LoadReport {
    /// 丢弃总数（五类和）。
    #[must_use]
    pub fn total_dropped(&self) -> u32 {
        self.dropped_out_of_bounds
            + self.dropped_non_finite
            + self.dropped_unsupported
            + self.dropped_null_geometry
            + self.dropped_nested_collection
    }

    fn classify(&mut self, e: &GeoError) {
        match e {
            GeoError::InvalidCoord { .. } => self.dropped_out_of_bounds += 1,
            GeoError::NonFinite => self.dropped_non_finite += 1,
            GeoError::Unsupported { .. } => self.dropped_unsupported += 1,
            GeoError::NestedCollection => self.dropped_nested_collection += 1,
            // 文档级错误（语法/IO/NotFound）=整文件 Err，永不进丢弃计数
            GeoError::Parse { .. } | GeoError::Io { .. } | GeoError::NotFound { .. } => {}
        }
    }
}

/// feature 级传播/丢弃决策收敛器。
struct Collector {
    policy: RepairPolicy,
    features: Vec<GeoFeature>,
    report: LoadReport,
}

impl Collector {
    fn accept(&mut self, r: Result<GeoFeature, GeoError>) -> Result<(), GeoError> {
        match r {
            Ok(f) => {
                self.features.push(f);
                Ok(())
            }
            Err(e) if self.policy == RepairPolicy::FastFail => Err(e),
            Err(e) => {
                self.report.classify(&e);
                Ok(())
            }
        }
    }

    fn feed(&mut self, f: &geojson::Feature) -> Result<(), GeoError> {
        let name = f
            .properties
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(geojson::JsonValue::as_str)
            .map(String::from);
        let Some(g) = &f.geometry else {
            if self.policy == RepairPolicy::Lenient {
                self.report.dropped_null_geometry += 1;
            }
            return Ok(()); // null 几何：FastFail 维持旧静默跳过（GEO-16 契约位）
        };
        if matches!(g.value, geojson::GeometryValue::GeometryCollection { .. }) {
            for sub in flatten_gc(g) {
                self.accept(sub)?;
            }
            return Ok(());
        }
        self.accept(feature_from(g, name, f.properties.as_ref()))
    }
}

/// 从字节解析（GEO-06/07/08 无文件入口；=FastFail 语义，GEO-16 契约位）。
pub fn parse_geojson(bytes: &[u8]) -> Result<GeoDocument, GeoError> {
    Ok(parse_impl(bytes, RepairPolicy::FastFail)?.0)
}

/// 宽松入口（GEO-15）：脏几何逐件丢弃+分型报告。
pub fn parse_geojson_lenient(bytes: &[u8]) -> Result<(GeoDocument, LoadReport), GeoError> {
    parse_impl(bytes, RepairPolicy::Lenient)
}

/// 策略显式入口（双口封装等价）。
pub fn parse_geojson_with(
    bytes: &[u8],
    policy: RepairPolicy,
) -> Result<(GeoDocument, LoadReport), GeoError> {
    parse_impl(bytes, policy)
}

fn parse_impl(bytes: &[u8], policy: RepairPolicy) -> Result<(GeoDocument, LoadReport), GeoError> {
    let root = geojson::GeoJson::from_reader(std::io::Cursor::new(bytes)).map_err(|e| {
        GeoError::Parse {
            reason: e.to_string(),
        }
    })?;
    let mut col = Collector {
        policy,
        features: Vec::new(),
        report: LoadReport::default(),
    };
    match root {
        geojson::GeoJson::FeatureCollection(fc) => {
            for f in &fc.features {
                col.feed(f)?;
            }
        }
        geojson::GeoJson::Feature(f) => col.feed(&f)?,
        geojson::GeoJson::Geometry(g) => {
            for sub in flatten_gc(&g) {
                col.accept(sub)?;
            }
        }
    }
    Ok((
        GeoDocument {
            features: col.features,
        },
        col.report,
    ))
}

fn flatten_gc(g: &geojson::Geometry) -> Vec<Result<GeoFeature, GeoError>> {
    let geojson::GeometryValue::GeometryCollection { geometries } = &g.value else {
        return vec![feature_from(g, None, None)];
    };
    geometries
        .iter()
        .map(|s| {
            if matches!(s.value, geojson::GeometryValue::GeometryCollection { .. }) {
                Err(GeoError::NestedCollection)
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
