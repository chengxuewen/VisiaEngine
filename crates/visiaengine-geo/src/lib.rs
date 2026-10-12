//! # visiaengine-geo
//! GeoJSON -> 3857 plane f64 geometry + minimal style. Zero render dependency (the host side performs the D7 origin decomposition).
//! Behavior contract: docs/sdd/geo.md (GEO-01..08; subdivision/style GEO-09..14 ride H2).

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

mod measure;
mod style;
mod tess;

pub use measure::{planar_distance, ring_area};
pub use style::{apply_scalar_ramp, parse_color};
pub use tess::{GeoPart, LineStrip, Marker, PartKind, TessPart, tessellate};

use thiserror::Error;
use visiaengine_core::{AttrSet, Vec3};

#[derive(Error, Debug)]
pub enum GeoError {
    #[error("file does not exist: {path}")]
    NotFound { path: String },
    #[error("GeoJSON parse failed: {reason}")]
    Parse { reason: String },
    #[error("latitude {lat}° outside the Web Mercator domain ±85.0511°")]
    InvalidCoord { lat: f64 },
    #[error("coordinate component non-finite (NaN/inf)")]
    NonFinite,
    #[error("unsupported geometry variant: {what}")]
    Unsupported { what: &'static str },
    #[error("GeometryCollection nested >= 2 levels")]
    NestedCollection,
    #[error("IO failure: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
}

/// Pre-subdivision geometry (3857 meters, z always 0).
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

/// The simplestyle six-key subset (GEO-12/13, consumed by H2).
impl GeoKind {
    /// Translate the geometry (GEO-14: world->local = shifted(-origin), eliminating hand-edited vertices on the consumer side).
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

/// Default blue (GEO-13 semantics); the six-key parsing is in the style module.
impl Default for StyleRecord {
    fn default() -> Self {
        default_style()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleRecord {
    pub fill: [f32; 4],
    pub fill_opacity: f32,
    pub stroke: [f32; 4],
    pub stroke_width_px: f32,
    pub marker_color: [f32; 4],
    pub radius_px: f32,
    /// GEO-25: label text source (`"{prop}"` unwrapped = property column name; bare string = constant text; None = no label).
    pub text_field: Option<Box<str>>,
    /// Label font size px (screen-constant size, default 14).
    pub text_size_px: f32,
    /// Label color (host sRGB face = same system as GEO-12; linearization lives at the consumer's chokepoint).
    pub text_color: [f32; 4],
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
    attrs: AttrSet,
}

impl GeoDocument {
    #[must_use]
    pub fn features(&self) -> &[GeoFeature] {
        &self.features
    }

    /// Attribute column set (GEO-17: row count aligns 1:1 with features()).
    #[must_use]
    pub fn attrs(&self) -> &AttrSet {
        &self.attrs
    }

    /// Host query entry (row = feature index).
    #[must_use]
    pub fn attr_f64(&self, feature: usize, name: &str) -> Option<f64> {
        self.attrs.f64(feature, name)
    }

    #[must_use]
    pub fn attr_str(&self, feature: usize, name: &str) -> Option<&str> {
        self.attrs.str_value(feature, name)
    }

    #[must_use]
    pub fn attr_bool(&self, feature: usize, name: &str) -> Option<bool> {
        self.attrs.bool(feature, name)
    }

    /// The whole layer's union 3857 bbox (empty layer None, GEO-05).
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

/// WGS84 lon/lat -> EPSG:3857 spherical formula (GEO-02/08; |phi| > 85.0511 deg rejected as a divergent domain).
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

/// Default style (GEO-13; simplestyle six-key parsing belongs to H2).
#[must_use]
pub fn default_style() -> StyleRecord {
    StyleRecord {
        fill: [0.0, 0.45, 1.0, 1.0],
        fill_opacity: 1.0,
        stroke: [1.0, 1.0, 1.0, 1.0],
        stroke_width_px: 1.5,
        marker_color: [0.0, 0.45, 1.0, 1.0],
        radius_px: 4.0,
        text_field: None,
        text_size_px: 14.0,
        text_color: [1.0, 1.0, 1.0, 1.0],
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
            _ => {
                return Err(bad(
                    "MultiLineString len>1 belongs to the H2 split (multi-part strategy)",
                ));
            }
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
        _ => return Err(bad("MultiPolygon/GC are handled at the document layer")),
    })
}

/// Geometry -> 3857 bbox (a pure function extracted from the feature construction segment, GEO-05 semantics unchanged).
fn bbox_of(kind: &GeoKind) -> [f64; 4] {
    let mut mins = [f64::MAX; 2];
    let mut maxs = [f64::MIN; 2];
    let mut acc = |x: f64, y: f64| push_bbox(&mut mins, &mut maxs, x, y);
    match kind {
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
    [mins[0], mins[1], maxs[0], maxs[1]]
}

/// JSON scalar attributes into columns (the GEO-17 parsing boundary -- serde_json appears here and at the document layer only).
/// number/string/bool go into the matching typed column; null/object/array are skipped (absence beats distortion).
fn load_props(a: &mut AttrSet, row: usize, p: &serde_json::Map<String, serde_json::Value>) {
    use serde_json::Value;
    for (k, v) in p {
        match v {
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    a.set_f64(row, k, f);
                }
            }
            Value::String(s) => {
                a.set_str(row, k, s.clone());
            }
            Value::Bool(b) => {
                a.set_bool(row, k, *b);
            }
            _ => {}
        }
    }
}

/// Repair policy ([E3D:A4] port): FastFail = one dirty piece of geometry rejects the whole document as Err (the existing contract);
/// Lenient = per-piece discard with by-type counting (GEO-15/16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepairPolicy {
    Lenient,
    FastFail,
}

/// Lenient load report: by-type counts of discard reasons (GEO-15). Classification is fixed at the generation point
/// (a typed GeoError variant mapping), never by parsing the reason string back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// lon/lat outside the Web Mercator domain
    pub dropped_out_of_bounds: u32,
    /// coordinate component NaN/inf
    pub dropped_non_finite: u32,
    /// not-ready variants such as MultiPolygon / multi-part MLS / empty ring
    pub dropped_unsupported: u32,
    /// features with `geometry: null` (no longer silently ignored under Lenient)
    pub dropped_null_geometry: u32,
    /// GeometryCollection nested >= 2 levels
    pub dropped_nested_collection: u32,
}

impl LoadReport {
    /// Total discards (sum of the five kinds).
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
            // document-level errors (syntax/IO/NotFound) = the whole file is Err, never entering the discard count
            GeoError::Parse { .. } | GeoError::Io { .. } | GeoError::NotFound { .. } => {}
        }
    }
}

/// The feature-level propagate/discard decision converger (the row production point of attribute columnization GEO-17).
struct Collector {
    policy: RepairPolicy,
    features: Vec<GeoFeature>,
    report: LoadReport,
    attrs: AttrSet,
}

impl Collector {
    fn accept(&mut self, r: Result<(), GeoError>) -> Result<(), GeoError> {
        match r {
            Ok(()) => Ok(()),
            Err(e) if self.policy == RepairPolicy::FastFail => Err(e),
            Err(e) => {
                self.report.classify(&e);
                Ok(())
            }
        }
    }

    /// Geometry -> feature single pipeline (projection single point: proj() inside kind_of, rejected on the same road as GEO-08).
    /// kind_of failure precedes row appending -- attrs rows and features stay aligned forever (GEO-17).
    fn push_geom(
        &mut self,
        gj: &geojson::Geometry,
        name: Option<String>,
        props: Option<&serde_json::Map<String, serde_json::Value>>,
    ) -> Result<(), GeoError> {
        let kind = kind_of(gj)?;
        let bbox = bbox_of(&kind);
        let row = self.attrs.add_row();
        if let Some(p) = props {
            load_props(&mut self.attrs, row, p);
        }
        let style = style::style_from_attrs(&self.attrs, row);
        self.features.push(GeoFeature {
            name,
            kind,
            style,
            world_bbox: Some(bbox),
        });
        Ok(())
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
            return Ok(()); // null geometry: FastFail keeps the old silent skip (GEO-16 contract position)
        };
        let props = f.properties.as_ref();
        if matches!(g.value, geojson::GeometryValue::GeometryCollection { .. }) {
            for sub in flatten_gc(g) {
                // GC children share the feature-level name/props (each child an independent row, GEO-17)
                let sub = sub.and_then(|g| self.push_geom(g, name.clone(), props));
                self.accept(sub)?;
            }
            return Ok(());
        }
        // ordinary pieces must also route through the accept decision (returning directly = bypassing Lenient, caught in the GREEN regression)
        let r = self.push_geom(g, name, props);
        self.accept(r)
    }
}

/// Parse from bytes (GEO-06/07/08 no-file entry; = FastFail semantics, GEO-16 contract position).
pub fn parse_geojson(bytes: &[u8]) -> Result<GeoDocument, GeoError> {
    Ok(parse_impl(bytes, RepairPolicy::FastFail)?.0)
}

/// The Lenient entry (GEO-15): dirty pieces are discarded per piece with by-type reporting.
pub fn parse_geojson_lenient(bytes: &[u8]) -> Result<(GeoDocument, LoadReport), GeoError> {
    parse_impl(bytes, RepairPolicy::Lenient)
}

/// The policy-explicit entry (the two wrappers are equivalent).
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
        attrs: AttrSet::new(),
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
                let sub = sub.and_then(|g| col.push_geom(g, None, None));
                col.accept(sub)?;
            }
        }
    }
    Ok((
        GeoDocument {
            features: col.features,
            attrs: col.attrs,
        },
        col.report,
    ))
}

/// GC flattening one level (borrowed output; props copied into each child row by the caller).
fn flatten_gc(g: &geojson::Geometry) -> Vec<Result<&geojson::Geometry, GeoError>> {
    let geojson::GeometryValue::GeometryCollection { geometries } = &g.value else {
        return vec![Ok(g)];
    };
    geometries
        .iter()
        .map(|s| {
            if matches!(s.value, geojson::GeometryValue::GeometryCollection { .. }) {
                Err(GeoError::NestedCollection)
            } else {
                Ok(s)
            }
        })
        .collect()
}

/// Parse a style from a JSON object string (test/host convenience entry; falls back to default when invalid).
/// The GEO-17 boundary entry: JSON -> one-shot AttrSet -> columnized reads (same implementation as the main pipeline).
#[must_use]
pub fn parse_style(props_json: &str) -> StyleRecord {
    let Ok(serde_json::Value::Object(m)) = serde_json::from_str(props_json) else {
        return default_style();
    };
    let mut a = AttrSet::new();
    let r = a.add_row();
    load_props(&mut a, r, &m);
    style::style_from_attrs(&a, r)
}

/// Parse a GeoJSON file.
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
