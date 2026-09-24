//! MVT feature → local geometry mapping (GEO-27, super-band B3).
//!
//! Decoded MVT features (tile-local integer coords, y-down) map to a small
//! geometry enum in a LOCAL frame; the tile bbox anchors it in 3857 world
//! space via `GeoTile::shifted` (same D7 origin discipline as the geo crate:
//! consumers get small local coordinates plus an f64 origin, never baked
//! world-space vertices).
//!
//! Ring semantics: MVT polygons are exterior-ring-first (winding is NOT
//! reliable across producers); holes detection is out of Phase-0 scope —
//! each ring becomes its own Poly (documented ceiling, ponytail note).

use crate::tiles::TileId;

/// Local-frame geometry extracted from one MVT feature.
#[derive(Debug, Clone, PartialEq)]
pub enum TileGeom {
    Point([f64; 2]),
    MultiPoint(Vec<[f64; 2]>),
    Line(Vec<[f64; 2]>),
    /// Single ring as a polygon (Phase 0: rings are NOT hole-classified).
    Poly(Vec<[f64; 2]>),
}

/// One decoded feature in tile-local frame (extent units → [0,1] normalized
/// × tile size, y flipped: MVT y-down → 3857 y-up).
#[derive(Debug, Clone, PartialEq)]
pub struct TileFeature {
    pub geom: TileGeom,
    /// Resolved string attribute (the only type E206 styles on this phase).
    pub attrs: std::collections::HashMap<String, String>,
}

/// A decoded tile anchored at its 3857 bbox origin (D7: local + origin, never
/// world-baked vertices).
#[derive(Debug, Clone)]
pub struct GeoTile {
    pub id: TileId,
    /// World anchor = bbox min corner (min_x, min_y), f64.
    pub origin: [f64; 3],
    pub features: Vec<TileFeature>,
}

impl GeoTile {
    /// Map all features of a decoded layer set into local geometry.
    /// `extent` comes from the layer (4096 standard).
    #[must_use]
    pub fn from_layers(id: TileId, layers: &[crate::mvt::MvtLayer]) -> Self {
        let (min_x, min_y, _, _) = id.bbox();
        let tile_size = crate::tiles::WORLD_EXTENT / f64::from(1u32 << id.z);
        let mut features = Vec::new();
        for layer in layers {
            let scale = tile_size / f64::from(layer.extent);
            for f in &layer.features {
                let attrs: std::collections::HashMap<String, String> = f
                    .tags(layer)
                    .into_iter()
                    .map(|(k, v)| {
                        let s = match v {
                            crate::mvt::MvtValue::Str(s) => s,
                            crate::mvt::MvtValue::Int(i) => i.to_string(),
                            crate::mvt::MvtValue::UInt(u) => u.to_string(),
                            crate::mvt::MvtValue::SInt(i) => i.to_string(),
                            crate::mvt::MvtValue::Bool(b) => b.to_string(),
                            crate::mvt::MvtValue::Double(d) => d.to_string(),
                        };
                        (k, s)
                    })
                    .collect();
                // tile-local px → world meters, y flipped (MVT y-down)
                let to_xy = |p: &(i32, i32)| {
                    [
                        min_x + f64::from(p.0) * scale,
                        min_y + (f64::from(layer.extent) - f64::from(p.1)) * scale,
                    ]
                };
                let geom = match f.geom_type {
                    1 => match f.points.len() {
                        0 => continue,
                        1 => TileGeom::Point(to_xy(&f.points[0])),
                        _ => TileGeom::MultiPoint(f.points.iter().map(to_xy).collect()),
                    },
                    2 => {
                        if f.points.len() < 2 {
                            continue;
                        }
                        TileGeom::Line(f.points.iter().map(to_xy).collect())
                    }
                    3 => {
                        if f.points.len() < 3 {
                            continue;
                        }
                        TileGeom::Poly(f.points.iter().map(to_xy).collect())
                    }
                    _ => continue,
                };
                features.push(TileFeature { geom, attrs });
            }
        }
        Self {
            id,
            origin: [min_x, min_y, 0.0],
            features,
        }
    }

    /// D7 shifted view: features re-based to `origin` = [0,0] local frame
    /// (mirrors geo GeoKind::shifted semantics; consumers tessellate locals).
    #[must_use]
    pub fn shifted(&self) -> Vec<(TileGeom, std::collections::HashMap<String, String>)> {
        self.features
            .iter()
            .map(|f| {
                let g = match &f.geom {
                    TileGeom::Point(p) => {
                        TileGeom::Point([p[0] - self.origin[0], p[1] - self.origin[1]])
                    }
                    TileGeom::MultiPoint(ps) => TileGeom::MultiPoint(
                        ps.iter()
                            .map(|p| [p[0] - self.origin[0], p[1] - self.origin[1]])
                            .collect(),
                    ),
                    TileGeom::Line(ps) => TileGeom::Line(
                        ps.iter()
                            .map(|p| [p[0] - self.origin[0], p[1] - self.origin[1]])
                            .collect(),
                    ),
                    TileGeom::Poly(ps) => TileGeom::Poly(
                        ps.iter()
                            .map(|p| [p[0] - self.origin[0], p[1] - self.origin[1]])
                            .collect(),
                    ),
                };
                (g, f.attrs.clone())
            })
            .collect()
    }
}
