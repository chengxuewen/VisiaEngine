//! # visiaengine-io-points
//! 点云（PLY ascii/binary_little_endian）→ CPU 侧数据。零渲染依赖（io-* 族纪律，架构②）。
//! 行为契约：`docs/sdd/io-points.md`（IO-01..06）。色=sRGB 宿主面（CORE-16）；
//! 坐标=origin-local f32 + f64 origin（D7：先 f64 求差再转 f32）。

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use thiserror::Error;
use visiaengine_core::AttrSet;

/// IO-05 容量守卫（本带唯一点数门；add_points 不受——CAPI-18 宿主自带账）。
pub const MAX_POINTS_CAP: u64 = 4_000_000;

/// IO-03 四类分型报告（重复点不成类=无 dup 字段，IO-03 注记）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PclReport {
    pub dropped_non_finite: u32,
    pub dropped_out_of_domain: u32,
    pub dropped_unsupported: u32,
    pub truncated_points: u32,
    pub kept: u64,
}

/// IO-06 产物（origin-local f32 + f64 origin；色=sRGB 面；meta=云级单行 AttrSet）。
#[derive(Clone, Debug)]
pub struct PclCloud {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
    pub radius_px: f32,
    pub origin: [f64; 3],
    pub meta: AttrSet,
    pub report: PclReport,
}

#[derive(Error, Debug)]
pub enum PclError {
    #[error("非 PLY/头非法: {0}")]
    Malformed(String),
    #[error("格式不支持（v0=ascii+bin_le；be/其余=文件级拒）: {0}")]
    UnsupportedFormat(String),
    #[error("截断且 FastFail: 声明 {declared} 完整 {complete}")]
    Truncated { declared: u64, complete: u64 },
    #[error("超容量帽: declared {declared} > cap {cap}")]
    OverCapacity { declared: u64, cap: u64 },
    #[error("脏数据且 FastFail: {0} 点被丢弃")]
    Dirty(u32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fmt {
    Ascii,
    BinLe,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pty {
    F32,
    F64,
    U8,
    I8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}

impl Pty {
    const fn width(self) -> usize {
        match self {
            Self::U8 | Self::I8 => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 | Self::F32 => 4,
            Self::I64 | Self::U64 | Self::F64 => 8,
        }
    }
    /// 消费列（IO-01 属性面）：坐标=任意数值宽度，色=U8/F32/F64。
    const fn consumable(self) -> bool {
        matches!(self, Self::F32 | Self::F64 | Self::U8)
    }
}

const CONSUMED: [&str; 6] = ["x", "y", "z", "red", "green", "blue"];

struct Prop {
    name: String,
    ty: Pty,
    /// 非消费列（未知名/未知型/list 外者）：计 dropped_unsupported，bin 仍占 stride。
    skip: bool,
}

struct Element {
    name: String,
    count: u64,
    props: Vec<Prop>,
}

/// IO-01..05 统一入口（bytes 形）。lenient=false=FastFail：**点级**丢弃/截断整文件拒；
/// 列与元素级 unsupported 只计数不触发（IO-03 落点澄清，双模式同权）。
pub fn parse_pcl(bytes: &[u8], lenient: bool) -> Result<PclCloud, PclError> {
    let (fmt, elements) = parse_head(bytes)?;
    let vertex = elements
        .iter()
        .find(|e| e.name == "vertex")
        .ok_or_else(|| PclError::Malformed("缺 element vertex".into()))?;
    for req in ["x", "y", "z"] {
        match vertex.props.iter().find(|p| p.name == req) {
            Some(p) if !p.skip => {}
            _ => return Err(PclError::Malformed(format!("vertex 缺必需消费属性 {req}"))),
        }
    }
    let declared = vertex.count;
    if declared > MAX_POINTS_CAP {
        return Err(PclError::OverCapacity {
            declared,
            cap: MAX_POINTS_CAP,
        });
    }
    let unsupported_cols = vertex.props.iter().filter(|p| p.skip).count() as u32;
    let unsupported_elems = elements.iter().filter(|e| e.name != "vertex").count() as u32;

    let body = &bytes[header_len(bytes)..];
    let mut pts: Vec<([f64; 3], Option<[f32; 3]>)> = Vec::new();
    let mut rep = PclReport::default();
    match fmt {
        Fmt::Ascii => parse_ascii(body, vertex, declared, &mut pts, &mut rep)?,
        Fmt::BinLe => parse_bin_le(body, vertex, declared, &mut pts, &mut rep)?,
    }
    rep.kept = pts.len() as u64;
    rep.dropped_unsupported = unsupported_cols + unsupported_elems;
    if !lenient {
        if rep.truncated_points > 0 {
            return Err(PclError::Truncated {
                declared,
                complete: rep.kept,
            });
        }
        let dirty = rep.dropped_non_finite + rep.dropped_out_of_domain;
        if dirty > 0 {
            return Err(PclError::Dirty(dirty));
        }
    }
    let label = match fmt {
        Fmt::Ascii => "ply/ascii",
        Fmt::BinLe => "ply/binary_le",
    };
    Ok(finalize(pts, rep, label))
}

fn header_len(bytes: &[u8]) -> usize {
    let mut pos = 0usize;
    for line in bytes.split_inclusive(|b| *b == b'\n') {
        pos += line.len();
        if line.starts_with(b"end_header") {
            break;
        }
    }
    pos
}

fn parse_head(bytes: &[u8]) -> Result<(Fmt, Vec<Element>), PclError> {
    if !bytes.starts_with(b"ply") {
        return Err(PclError::Malformed("magic≠ply".into()));
    }
    let mut fmt: Option<Fmt> = None;
    let mut elements: Vec<Element> = Vec::new();
    for raw in bytes.split(|b| *b == b'\n') {
        let Ok(line) = std::str::from_utf8(raw) else {
            continue;
        };
        let line = line.trim_end_matches('\r');
        let mut w = line.split_whitespace();
        match w.next() {
            None => {}
            Some("end_header") => break,
            Some("ply") | Some("comment") | Some("obj_info") => {}
            Some("format") => {
                let (f, v) = (w.next(), w.next());
                if v != Some("1.0") {
                    return Err(PclError::Malformed("format 版本".into()));
                }
                fmt = Some(match f {
                    Some("ascii") => Fmt::Ascii,
                    Some("binary_little_endian") => Fmt::BinLe,
                    Some("binary_big_endian") => {
                        return Err(PclError::UnsupportedFormat("binary_big_endian".into()));
                    }
                    _ => return Err(PclError::Malformed("format 词".into())),
                });
            }
            Some("element") => {
                let (Some(name), Some(n)) =
                    (w.next(), w.next().and_then(|v| v.parse::<u64>().ok()))
                else {
                    return Err(PclError::Malformed("element 头".into()));
                };
                elements.push(Element {
                    name: name.to_string(),
                    count: n,
                    props: Vec::new(),
                });
            }
            Some("property") => {
                let Some(e) = elements.last_mut() else {
                    return Err(PclError::Malformed("property 先于 element".into()));
                };
                let (Some(ty), Some(name)) = (w.next(), w.next()) else {
                    return Err(PclError::Malformed("property 字段".into()));
                };
                if ty == "list" {
                    // vertex list=不可读（IO-01）；非 vertex 元素整类计弃。
                    if e.name == "vertex" {
                        return Err(PclError::Malformed("vertex 含 list 属性".into()));
                    }
                    continue;
                }
                let pt = match ty {
                    "float" | "float32" => Pty::F32,
                    "double" | "float64" => Pty::F64,
                    "uchar" | "uint8" => Pty::U8,
                    "char" | "int8" => Pty::I8,
                    "short" | "int16" => Pty::I16,
                    "ushort" | "uint16" => Pty::U16,
                    "int" | "int32" => Pty::I32,
                    "uint" | "uint32" => Pty::U32,
                    "int64" | "uint64" => {
                        e.props.push(Prop {
                            name: name.to_string(),
                            ty: if ty == "int64" { Pty::I64 } else { Pty::U64 },
                            skip: true,
                        });
                        continue;
                    }
                    _ => {
                        return Err(PclError::Malformed(format!("未知属性类型 {ty}")));
                    }
                };
                let skip = !CONSUMED.contains(&name) || !pt.consumable();
                e.props.push(Prop {
                    name: name.to_string(),
                    ty: pt,
                    skip,
                });
            }
            _ => return Err(PclError::Malformed(format!("未知头行: {line}"))),
        }
    }
    let fmt = fmt.ok_or_else(|| PclError::Malformed("无 format 行".into()))?;
    Ok((fmt, elements))
}

/// ascii 逐行：token 序=属性序；尾行缺 token=截断（IO-04，EOF 前完整行保留）。
fn parse_ascii(
    body: &[u8],
    vertex: &Element,
    declared: u64,
    pts: &mut Vec<([f64; 3], Option<[f32; 3]>)>,
    rep: &mut PclReport,
) -> Result<(), PclError> {
    let all = String::from_utf8_lossy(body).into_owned();
    let mut lines = all.lines().filter(|l| !l.trim().is_empty());
    let pos = |k: &str| vertex.props.iter().position(|p| p.name == k);
    let ntok = vertex.props.len();
    for i in 0..declared {
        let Some(line) = lines.next() else {
            rep.truncated_points = (declared - i) as u32;
            return Ok(());
        };
        let toks: Vec<&str> = line.split_whitespace().collect();
        if toks.len() < ntok {
            rep.truncated_points = (declared - i) as u32;
            return Ok(());
        }
        let mut vals = Vec::with_capacity(ntok);
        for t in toks.iter().take(ntok) {
            vals.push(t.parse::<f64>().unwrap_or(f64::NAN));
        }
        let (ix, iy, iz) = (pos("x"), pos("y"), pos("z"));
        let p = match (ix, iy, iz) {
            (Some(a), Some(b), Some(c)) => [vals[a], vals[b], vals[c]],
            _ => return Err(PclError::Malformed("xyz 定位".into())),
        };
        if !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite() {
            rep.dropped_non_finite += 1;
            continue;
        }
        if p.iter().any(|v| v.abs() > f64::from(f32::MAX)) {
            rep.dropped_out_of_domain += 1;
            continue;
        }
        pts.push((p, ascii_rgb(&vals, vertex, pos)));
    }
    Ok(())
}

/// 色对：uchar(宽=1)/255 归一，float 直传；缺/非消费=默认灰（finalize 侧）。
fn ascii_rgb(
    vals: &[f64],
    vertex: &Element,
    pos: impl Fn(&str) -> Option<usize>,
) -> Option<[f32; 3]> {
    let val = |k: &str| pos(k).filter(|&j| !vertex.props[j].skip).map(|j| vals[j]);
    let narrow = pos("red").is_some_and(|j| vertex.props[j].ty.width() == 1);
    rgb_from(val, narrow)
}

fn rgb_from(mut val: impl FnMut(&str) -> Option<f64>, narrow: bool) -> Option<[f32; 3]> {
    let (r, g, b) = (val("red")?, val("green")?, val("blue")?);
    let f = if narrow {
        |v: f64| (v / 255.0) as f32
    } else {
        |v: f64| v as f32
    };
    Some([f(r), f(g), f(b)])
}

/// binary_le：显式端序逐列 from_le_bytes（IO-02；宽度=0 属性=头已拦）。
fn parse_bin_le(
    body: &[u8],
    vertex: &Element,
    declared: u64,
    pts: &mut Vec<([f64; 3], Option<[f32; 3]>)>,
    rep: &mut PclReport,
) -> Result<(), PclError> {
    let stride: usize = vertex.props.iter().map(|p| p.ty.width()).sum();
    let complete = (body.len() / stride).min(declared as usize);
    rep.truncated_points = declared.saturating_sub(complete as u64) as u32;
    let pos = |k: &str| vertex.props.iter().position(|p| p.name == k);
    let offs: Vec<usize> = {
        let mut acc = 0usize;
        vertex
            .props
            .iter()
            .map(|p| {
                let o = acc;
                acc += p.ty.width();
                o
            })
            .collect()
    };
    let rd = |row: &[u8], j: usize| -> Option<f64> {
        let p = vertex.props.get(j)?;
        let b = row.get(offs[j]..offs[j] + p.ty.width())?;
        Some(match p.ty {
            Pty::F32 => f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
            Pty::F64 => {
                let mut a = [0u8; 8];
                a.copy_from_slice(b);
                f64::from_le_bytes(a)
            }
            Pty::U8 => b[0] as f64,
            Pty::I8 => (b[0] as i8) as f64,
            Pty::I16 => i16::from_le_bytes([b[0], b[1]]) as f64,
            Pty::U16 => u16::from_le_bytes([b[0], b[1]]) as f64,
            Pty::I32 => i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
            Pty::U32 => u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
            Pty::I64 => {
                let mut a = [0u8; 8];
                a.copy_from_slice(b);
                i64::from_le_bytes(a) as f64
            }
            Pty::U64 => {
                let mut a = [0u8; 8];
                a.copy_from_slice(b);
                u64::from_le_bytes(a) as f64
            }
        })
    };
    for r in 0..complete {
        let row = &body[r * stride..(r + 1) * stride];
        let (ix, iy, iz) = match (pos("x"), pos("y"), pos("z")) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => return Err(PclError::Malformed("xyz 定位".into())),
        };
        let p = match (rd(row, ix), rd(row, iy), rd(row, iz)) {
            (Some(a), Some(b), Some(c)) => [a, b, c],
            _ => {
                rep.dropped_non_finite += 1;
                continue;
            }
        };
        if !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite() {
            rep.dropped_non_finite += 1;
            continue;
        }
        if p.iter().any(|v| v.abs() > f64::from(f32::MAX)) {
            rep.dropped_out_of_domain += 1;
            continue;
        }
        let rgb = {
            let narrow = pos("red").is_some_and(|j| vertex.props[j].ty.width() == 1);
            rgb_from(
                |k| {
                    pos(k)
                        .filter(|&j| !vertex.props[j].skip)
                        .and_then(|j| rd(row, j))
                },
                narrow,
            )
        };
        pts.push((p, rgb));
    }
    Ok(())
}

/// IO-06：origin=bbox 中心 f64；positions=f64 先求差再转 f32；meta 云级单行 8 列。
fn finalize(
    pts: Vec<([f64; 3], Option<[f32; 3]>)>,
    report: PclReport,
    format: &'static str,
) -> PclCloud {
    let mut mn = [f64::MAX; 3];
    let mut mx = [f64::MIN; 3];
    for (p, _) in &pts {
        for j in 0..3 {
            mn[j] = mn[j].min(p[j]);
            mx[j] = mx[j].max(p[j]);
        }
    }
    let empty = pts.is_empty();
    let origin = if empty {
        [0.0; 3]
    } else {
        std::array::from_fn(|j| (mn[j] + mx[j]) / 2.0)
    };
    let mut meta = AttrSet::new();
    let r = meta.add_row();
    meta.set_f64(r, "point_count", report.kept as f64);
    meta.set_str(r, "format", format);
    if !empty {
        for (j, k) in ["bbox_min_x", "bbox_min_y", "bbox_min_z"]
            .iter()
            .enumerate()
        {
            meta.set_f64(r, k, mn[j]);
        }
        for (j, k) in ["bbox_max_x", "bbox_max_y", "bbox_max_z"]
            .iter()
            .enumerate()
        {
            meta.set_f64(r, k, mx[j]);
        }
    }
    PclCloud {
        positions: pts
            .iter()
            .map(|(p, _)| std::array::from_fn(|j| (p[j] - origin[j]) as f32))
            .collect(),
        colors: pts
            .iter()
            .map(|(_, c)| c.unwrap_or([0.7, 0.75, 0.8]))
            .collect(),
        radius_px: 3.0,
        origin,
        meta,
        report,
    }
}
