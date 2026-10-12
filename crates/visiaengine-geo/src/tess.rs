//! Subdivision (GEO-09..11/14): GeoKind -> triangle parts.
//! D7 interface discipline: input is local coordinates (projection done at the parsing boundary; origin decomposition on the upload side).
//! v0 form: positions expand into independent triangles (identity indices) -- trading simplicity for small vertex counts,
//! index reuse (vertex cache) left to the performance slice.

use lyon_path::Path;
use lyon_tessellation::geometry_builder::{BuffersBuilder, Positions, VertexBuffers};
use lyon_tessellation::math::Point;
use lyon_tessellation::{FillOptions, FillTessellator};

use crate::{GeoError, GeoKind, StyleRecord};

/// rgba -> rgb (colors for the tile-expansion family that lands on the GPU table; direct 4-element slice take, no panic surface [clippy unwrap_used discipline]).
const fn rgb(c: [f32; 4]) -> [f32; 3] {
    [c[0], c[1], c[2]]
}

/// Part-purpose discriminator (the consumer binds the material color).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartKind {
    Fill,
}

/// Polyline ribbon (GEO-24, 4de): adjacent points = one segment; **the tile-expansion math lives on the GPU (WGPU-17),
/// the geo layer only supplies shape + px semantics** (neutral types, no render IR dependency -- layering discipline).
#[derive(Clone, Debug, PartialEq)]
pub struct LineStrip {
    pub pts: Vec<[f32; 2]>,
    pub color: [f32; 3],
    pub width_px: f32,
}

/// Point marker (GEO-24): the round splat is delivered by the WGPU-18 FS mask.
#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub pos: [f32; 2],
    pub color: [f32; 3],
    pub radius_px: f32,
}

/// The three families of subdivision output (GEO-24 replaces the triangle forms of Stroke/Marker -- the CPU ribbon/square retires).
#[derive(Clone, Debug)]
pub enum GeoPart {
    Fill(TessPart),
    Strokes(Vec<LineStrip>),
    Markers(Vec<Marker>),
}

/// One mesh that can land on the GPU (positions: local x, y, z = 0; every 3 vertices = one triangle).
#[derive(Clone, Debug)]
pub struct TessPart {
    pub kind: PartKind,
    pub color: [f32; 4],
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl TessPart {
    fn new(kind: PartKind, color: [f32; 4]) -> Self {
        Self {
            kind,
            color,
            positions: Vec::new(),
            indices: Vec::new(),
        }
    }
    fn extend_tris(&mut self, verts: &[Point], idx: &[u32]) {
        for &i in idx {
            let v = &verts[i as usize];
            self.positions.push([v.x, v.y, 0.0]);
            self.indices.push(self.indices.len() as u32);
        }
    }
}

fn build_path(rings: &[&Vec<[f64; 2]>]) -> Path {
    // each element is one ring (outer ring and holes arrive at the same level)
    let mut b = Path::builder();
    for ring in rings {
        let Some(first) = ring.first() else {
            continue;
        };
        b.begin(lyon_tessellation::math::point(
            first[0] as f32,
            first[1] as f32,
        ));
        for p in &ring[1..] {
            b.line_to(lyon_tessellation::math::point(p[0] as f32, p[1] as f32));
        }
        b.end(true);
    }
    b.build()
}

fn tess_fill(path: &Path, color: [f32; 4]) -> Result<TessPart, GeoError> {
    let mut buf = VertexBuffers::<Point, u32>::new();
    {
        let mut bb = BuffersBuilder::new(&mut buf, Positions);
        FillTessellator::new()
            .tessellate_path(path, &FillOptions::default(), &mut bb)
            .map_err(|e| GeoError::Parse {
                reason: format!("fill tessellation: {e}"),
            })?;
    }
    let mut part = TessPart::new(PartKind::Fill, color);
    part.extend_tris(&buf.vertices, &buf.indices);
    Ok(part)
}

/// Geometry -> subdivision parts.
/// Line features also gain endpoint markers (a v0 visualization choice); points always emit a Marker part (GEO-11 tests with a clean two-point line, the empty-positions marker does not affect its fold assertion -- see the test construction).
pub fn tessellate(kind: &GeoKind, style: &StyleRecord) -> Result<Vec<GeoPart>, GeoError> {
    let mut parts: Vec<GeoPart> = Vec::new();
    match kind {
        GeoKind::Poly { ext, holes } => {
            let mut rings: Vec<&Vec<[f64; 2]>> = vec![ext];
            rings.extend(holes.iter());
            if ext.len() < 3 {
                return Err(GeoError::Parse {
                    reason: "polygon exterior < 3 pts".into(),
                });
            }
            let path = build_path(&rings);
            parts.push(GeoPart::Fill(tess_fill(
                &path,
                with_alpha(style.fill, style.fill_opacity),
            )?));
            if style.stroke_width_px > 0.0 {
                let strips = rings
                    .iter()
                    .filter(|r| r.len() >= 2)
                    .map(|r| {
                        let mut pts: Vec<[f32; 2]> =
                            r.iter().map(|p| [p[0] as f32, p[1] as f32]).collect();
                        pts.push(pts[0]); // ring closure (the GEO-24 assertion face)
                        LineStrip {
                            pts,
                            color: rgb(style.stroke),
                            width_px: style.stroke_width_px,
                        }
                    })
                    .collect::<Vec<_>>();
                if !strips.is_empty() {
                    parts.push(GeoPart::Strokes(strips));
                }
            }
        }
        GeoKind::Line(pts) if pts.len() >= 2 => {
            if style.stroke_width_px > 0.0 {
                parts.push(GeoPart::Strokes(vec![LineStrip {
                    pts: pts.iter().map(|p| [p.x as f32, p.y as f32]).collect(),
                    color: rgb(style.stroke),
                    width_px: style.stroke_width_px,
                }]));
            }
        }
        GeoKind::Line(_) => {}
        GeoKind::Point(p) => {
            parts.push(GeoPart::Markers(vec![Marker {
                pos: [p.x as f32, p.y as f32],
                color: rgb(style.marker_color),
                radius_px: style.radius_px,
            }]));
        }
        GeoKind::MultiPoint(ps) => {
            let ms = ps
                .iter()
                .map(|p| Marker {
                    pos: [p.x as f32, p.y as f32],
                    color: rgb(style.marker_color),
                    radius_px: style.radius_px,
                })
                .collect::<Vec<_>>();
            if !ms.is_empty() {
                parts.push(GeoPart::Markers(ms));
            }
        }
    }
    Ok(parts)
}

fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}
