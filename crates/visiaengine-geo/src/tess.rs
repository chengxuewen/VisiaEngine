//! 细分（GEO-09..11/14）：GeoKind → 三角形 parts。
//! D7 接口纪律：输入 local 坐标（解析边界已投影；origin 分解在上传方）。
//! v0 形态：positions 展开为独立三角形（identity indices）——小顶点量下换取简单性，
//! 索引复用（顶点缓存）留性能片。

use lyon_path::Path;
use lyon_tessellation::geometry_builder::{BuffersBuilder, Positions, VertexBuffers};
use lyon_tessellation::math::Point;
use lyon_tessellation::{FillOptions, FillTessellator};

use crate::{GeoError, GeoKind, StyleRecord};

/// rgba → rgb（扩片族色住表用；4 元素切片直取，无 panic 面 [clippy unwrap_used 纪律]）。
const fn rgb(c: [f32; 4]) -> [f32; 3] {
    [c[0], c[1], c[2]]
}

/// part 用途判别（消费端绑 material 色）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartKind {
    Fill,
}

/// 折线条带（GEO-24，4de）：相邻点=一段；**扩片数学住 GPU（WGPU-17），
/// geo 层只给位形+px 语义**（中立类型，不引 render IR 依赖——分层纪律）。
#[derive(Clone, Debug, PartialEq)]
pub struct LineStrip {
    pub pts: Vec<[f32; 2]>,
    pub color: [f32; 3],
    pub width_px: f32,
}

/// 点标记（GEO-24）：圆 splat 由 WGPU-18 FS mask 兑现。
#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub pos: [f32; 2],
    pub color: [f32; 3],
    pub radius_px: f32,
}

/// 细分输出三族（GEO-24 取代 Stroke/Marker 三角形态——CPU 条带/方块退役）。
#[derive(Clone, Debug)]
pub enum GeoPart {
    Fill(TessPart),
    Strokes(Vec<LineStrip>),
    Markers(Vec<Marker>),
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

/// 几何 → 细分 parts。
/// 线要素同时补端点 marker（v0 可视化选择）；points 恒出 Marker part（GEO-11 用干净两点线测试，marker 空 positions 不影响其 fold 断言——见测试构造）。
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
                        pts.push(pts[0]); // 环闭合（GEO-24 断言面）
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
