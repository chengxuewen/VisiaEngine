//! 细分（GEO-09..11/14）：GeoKind → 三角形 parts。
//! D7 接口纪律：输入 local 坐标（解析边界已投影；origin 分解在上传方）。
//! v0 形态：positions 展开为独立三角形（identity indices）——小顶点量下换取简单性，
//! 索引复用（顶点缓存）留性能片。

use lyon_path::Path;
use lyon_tessellation::geometry_builder::{BuffersBuilder, Positions, VertexBuffers};
use lyon_tessellation::math::Point;
use lyon_tessellation::{FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};

use crate::{GeoError, GeoKind, StyleRecord};

/// part 用途判别（消费端绑 material 色）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartKind {
    Fill,
    Stroke,
    Marker,
}

/// 一块可上 GPU 的三角网格（positions: local x,y，z=0；每 3 顶点一三角形）。
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
    // 每个元素即一条环（外环与洞同层传入）
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

fn tess_stroke(path: &Path, width: f32, color: [f32; 4]) -> Result<TessPart, GeoError> {
    let mut buf = VertexBuffers::<Point, u32>::new();
    {
        let mut bb = BuffersBuilder::new(&mut buf, Positions);
        StrokeTessellator::new()
            .tessellate_path(
                path,
                &StrokeOptions::default().with_line_width(width),
                &mut bb,
            )
            .map_err(|e| GeoError::Parse {
                reason: format!("stroke tessellation: {e}"),
            })?;
    }
    let mut part = TessPart::new(PartKind::Stroke, color);
    part.extend_tris(&buf.vertices, &buf.indices);
    Ok(part)
}

fn quad(cx: f32, cy: f32, r: f32, color: [f32; 4]) -> TessPart {
    let mut p = TessPart::new(PartKind::Marker, color);
    let corners = [
        [cx - r, cy - r],
        [cx + r, cy - r],
        [cx + r, cy + r],
        [cx - r, cy - r],
        [cx + r, cy + r],
        [cx - r, cy + r],
    ];
    p.positions = corners.iter().map(|c| [c[0], c[1], 0.0]).collect();
    p.indices = (0..6).collect();
    p
}

/// 几何 → 细分 parts。
/// 线要素同时补端点 marker（v0 可视化选择）；points 恒出 Marker part（GEO-11 用干净两点线测试，marker 空 positions 不影响其 fold 断言——见测试构造）。
pub fn tessellate(kind: &GeoKind, style: &StyleRecord) -> Result<Vec<TessPart>, GeoError> {
    let mut parts = Vec::new();
    match kind {
        GeoKind::Poly { ext, holes } => {
            let mut rings: Vec<&Vec<[f64; 2]>> = vec![ext];
            rings.extend(holes);
            if ext.len() < 3 {
                return Err(GeoError::Parse {
                    reason: "polygon exterior < 3 pts".into(),
                });
            }
            let path = build_path(&rings);
            parts.push(tess_fill(
                &path,
                with_alpha(style.fill, style.fill_opacity),
            )?);
            if style.stroke_width_m > 0.0 {
                let outline: Vec<&Vec<[f64; 2]>> =
                    std::iter::once(ext).chain(holes.iter()).collect();
                let opath = build_path(&outline);
                parts.push(tess_stroke(
                    &opath,
                    style.stroke_width_m,
                    with_alpha(style.stroke, 1.0),
                )?);
            }
        }
        GeoKind::Line(pts) if pts.len() >= 2 => {
            let mut b = Path::builder();
            let first = pts[0];
            b.begin(lyon_tessellation::math::point(
                first.x as f32,
                first.y as f32,
            ));
            for p in &pts[1..] {
                b.line_to(lyon_tessellation::math::point(p.x as f32, p.y as f32));
            }
            b.end(false);
            parts.push(tess_stroke(
                &b.build(),
                style.stroke_width_m,
                with_alpha(style.stroke, 1.0),
            )?);
        }
        GeoKind::Line(_) => {}
        GeoKind::Point(p) => {
            parts.push(quad(
                p.x as f32,
                p.y as f32,
                style.radius_m,
                with_alpha(style.marker_color, 1.0),
            ));
        }
        GeoKind::MultiPoint(ps) => {
            let mut m = TessPart::new(PartKind::Marker, with_alpha(style.marker_color, 1.0));
            for p in ps {
                let q = quad(p.x as f32, p.y as f32, style.radius_m, m.color);
                m.positions.extend(q.positions);
                m.indices = (0..m.positions.len() as u32).collect();
            }
            if !m.positions.is_empty() {
                parts.push(m);
            }
        }
    }
    Ok(parts)
}

fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}
