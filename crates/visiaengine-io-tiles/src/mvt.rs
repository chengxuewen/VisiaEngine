//! Hand-rolled MVT (Mapbox Vector Tile) decoder — spec v2.1 subset (IO-10).
//!
//! Zero protobuf dependencies: a minimal varint/wire reader over the stable
//! MVT schema (layers → features → keys/values → geometry commands).
//! Error taxonomy follows the io-points precedent: point-level corruption
//! fails fast (Truncated / BadWire), structural mismatches are typed
//! (UnsupportedVersion / BadGeometry).

use thiserror::Error;

/// Decode error taxonomy (IO-10; io-points four-way precedent).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum MvtError {
    #[error("buffer truncated mid-field")]
    Truncated,
    #[error("unsupported layer version {0} (spec v2.1 = version 2)")]
    UnsupportedVersion(u32),
    #[error("malformed wire record at layer/feature level")]
    BadWire,
    #[error("invalid geometry command sequence")]
    BadGeometry,
}

/// Decoded attribute value (subset used by MVT; float/double unify to Double).
#[derive(Debug, Clone, PartialEq)]
pub enum MvtValue {
    Str(String),
    Double(f64),
    Int(i64),
    UInt(u64),
    SInt(i64),
    Bool(bool),
}

/// Decoded MVT feature: identity + tags + raw geometry commands + points.
#[derive(Debug, Clone)]
pub struct MvtFeature {
    pub id: u64,
    /// key-index → value-index pairs (resolved via the layer's keys/values).
    pub tag_pairs: Vec<(u32, u32)>,
    /// 1=POINT 2=LINESTRING 3=POLYGON (raw spec enum).
    pub geom_type: u32,
    /// Decoded command stream metadata: (cmd_id, count) pairs in order.
    pub commands: Vec<(u32, u32)>,
    /// Absolute tile-local points (post zigzag/delta application), extent units.
    pub points: Vec<(i32, i32)>,
    /// Polygon rings closed with ClosePath.
    pub closed: bool,
}

impl MvtFeature {
    pub fn geom_type(&self) -> u32 {
        self.geom_type
    }

    /// Resolve tag pairs against the layer's keys/values into a name→value map.
    pub fn tags(&self, layer: &MvtLayer) -> std::collections::HashMap<String, MvtValue> {
        let mut out = std::collections::HashMap::new();
        for (k, v) in &self.tag_pairs {
            if let (Some(key), Some(val)) =
                (layer.keys.get(*k as usize), layer.values.get(*v as usize))
            {
                out.insert(key.clone(), val.clone());
            }
        }
        out
    }
}

/// Decoded MVT layer.
#[derive(Debug, Clone)]
pub struct MvtLayer {
    pub name: String,
    pub features: Vec<MvtFeature>,
    pub keys: Vec<String>,
    pub values: Vec<MvtValue>,
    pub extent: u32,
    pub version: u32,
}

/// Decoded tile: a bag of layers.
#[derive(Debug, Clone)]
pub struct MvtTile {
    pub layers: Vec<MvtLayer>,
}

// ===== minimal protobuf wire reader =====

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn eof(&self) -> bool {
        self.pos >= self.buf.len()
    }
    fn varint(&mut self) -> Result<u64, MvtError> {
        let mut out: u64 = 0;
        let mut shift = 0;
        loop {
            if self.pos >= self.buf.len() {
                return Err(MvtError::Truncated);
            }
            let b = self.buf[self.pos];
            self.pos += 1;
            out |= u64::from(b & 0x7F) << shift;
            if b & 0x80 == 0 {
                return Ok(out);
            }
            shift += 7;
            if shift > 63 {
                return Err(MvtError::BadWire);
            }
        }
    }
    fn field_key(&mut self) -> Result<(u32, u8), MvtError> {
        let key = self.varint()?;
        Ok(((key >> 3) as u32, (key & 7) as u8))
    }
    fn len_delim(&mut self) -> Result<&'a [u8], MvtError> {
        let n = self.varint()? as usize;
        if self.pos + n > self.buf.len() {
            return Err(MvtError::Truncated);
        }
        let s = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn fixed<const N: usize>(&mut self) -> Result<&'a [u8; N], MvtError> {
        if self.pos + N > self.buf.len() {
            return Err(MvtError::Truncated);
        }
        let (head, _tail) = self.buf[self.pos..self.pos + N]
            .split_first_chunk()
            .ok_or(MvtError::Truncated)?;
        self.pos += N;
        Ok(head)
    }
    fn skip(&mut self, wire: u8) -> Result<(), MvtError> {
        match wire {
            0 => {
                self.varint()?;
            }
            1 => {
                if self.pos + 8 > self.buf.len() {
                    return Err(MvtError::Truncated);
                }
                self.pos += 8;
            }
            2 => {
                self.len_delim()?;
            }
            5 => {
                if self.pos + 4 > self.buf.len() {
                    return Err(MvtError::Truncated);
                }
                self.pos += 4;
            }
            _ => return Err(MvtError::BadWire),
        }
        Ok(())
    }
}

fn zigzag(v: u64) -> i64 {
    ((v >> 1) as i64) ^ -((v & 1) as i64)
}

fn parse_value(buf: &[u8]) -> Result<MvtValue, MvtError> {
    let mut r = Reader::new(buf);
    while !r.eof() {
        let (field, wire) = r.field_key()?;
        match (field, wire) {
            (1, 2) => {
                let s = r.len_delim()?;
                return Ok(MvtValue::Str(String::from_utf8_lossy(s).into_owned()));
            }
            (2, 5) => {
                let s = r.fixed::<4>()?;
                return Ok(MvtValue::Double(f64::from(f32::from_le_bytes(*s))));
            }
            (3, 1) => {
                let s = r.fixed::<8>()?;
                return Ok(MvtValue::Double(f64::from_le_bytes(*s)));
            }
            (4, 0) => return Ok(MvtValue::Int(r.varint()? as i64)),
            (5, 0) => return Ok(MvtValue::UInt(r.varint()?)),
            (6, 0) => return Ok(MvtValue::SInt(zigzag(r.varint()?))),
            (7, 0) => return Ok(MvtValue::Bool(r.varint()? != 0)),
            (_, w) => r.skip(w)?,
        }
    }
    Err(MvtError::BadWire)
}

fn parse_feature(buf: &[u8]) -> Result<MvtFeature, MvtError> {
    let mut id = 0u64;
    let mut geom_type = 0u32;
    let mut raw_tags: Vec<u64> = Vec::new();
    let mut geometry: Option<Vec<u8>> = None;
    let mut r = Reader::new(buf);
    while !r.eof() {
        let (field, wire) = r.field_key()?;
        match (field, wire) {
            (1, 0) => id = r.varint()?,
            (2, 2) => {
                let packed = r.len_delim()?;
                let mut pr = Reader::new(packed);
                while !pr.eof() {
                    raw_tags.push(pr.varint()?);
                }
            }
            (3, 0) => geom_type = r.varint()? as u32,
            (4, 2) => geometry = Some(r.len_delim()?.to_vec()),
            (_, w) => r.skip(w)?,
        }
    }
    let tag_pairs: Vec<(u32, u32)> = raw_tags
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&[k, v]| (k as u32, v as u32))
        .collect();

    let (commands, points, closed) = decode_geometry(geometry.as_deref().unwrap_or(&[]))?;
    Ok(MvtFeature {
        id,
        tag_pairs,
        geom_type,
        commands,
        points,
        closed,
    })
}

/// Geometry command state machine: MoveTo=1 LineTo=2 ClosePath=7.
/// Command word = (id & 7) | (count << 3); parameters are zigzag DELTAS from
/// the running cursor (MoveTo included — spec: parameters are per-point deltas,
/// cursor starts at 0).
type GeometryDecoded = (Vec<(u32, u32)>, Vec<(i32, i32)>, bool);

#[allow(clippy::type_complexity)]
fn decode_geometry(buf: &[u8]) -> Result<GeometryDecoded, MvtError> {
    let mut r = Reader::new(buf);
    let mut commands = Vec::new();
    let mut points = Vec::new();
    let mut closed = false;
    let mut cx: i64 = 0;
    let mut cy: i64 = 0;
    while !r.eof() {
        let word = r.varint()?;
        let id = (word & 7) as u32;
        let count = (word >> 3) as u32;
        if count == 0 {
            return Err(MvtError::BadGeometry);
        }
        commands.push((id, count));
        match id {
            1 | 2 => {
                for _ in 0..count {
                    let dx = zigzag(r.varint()?);
                    let dy = zigzag(r.varint()?);
                    cx += dx;
                    cy += dy;
                    let px = i32::try_from(cx).map_err(|_| MvtError::BadGeometry)?;
                    let py = i32::try_from(cy).map_err(|_| MvtError::BadGeometry)?;
                    points.push((px, py));
                }
            }
            7 => {
                closed = true;
            }
            _ => return Err(MvtError::BadGeometry),
        }
    }
    Ok((commands, points, closed))
}

/// Decode a full tile buffer into layers.
pub fn decode_tile(buf: &[u8]) -> Result<MvtTile, MvtError> {
    if buf.is_empty() {
        return Err(MvtError::Truncated);
    }
    let mut layers = Vec::new();
    let mut r = Reader::new(buf);
    while !r.eof() {
        let (field, wire) = r.field_key()?;
        if (field, wire) == (3, 2) {
            let lb = r.len_delim()?;
            layers.push(parse_layer(lb)?);
        } else {
            r.skip(wire)?;
        }
    }
    Ok(MvtTile { layers })
}

fn parse_layer(buf: &[u8]) -> Result<MvtLayer, MvtError> {
    let mut name = String::new();
    let mut features = Vec::new();
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut extent = 4096u32;
    let mut version = 0u32;
    let mut r = Reader::new(buf);
    while !r.eof() {
        let (field, wire) = r.field_key()?;
        match (field, wire) {
            (1, 2) => name = String::from_utf8_lossy(r.len_delim()?).into_owned(),
            (2, 2) => features.push(parse_feature(r.len_delim()?)?),
            (3, 2) => keys.push(String::from_utf8_lossy(r.len_delim()?).into_owned()),
            (4, 2) => values.push(parse_value(r.len_delim()?)?),
            (5, 0) => extent = r.varint()? as u32,
            (6, 0) => version = r.varint()? as u32,
            (_, w) => r.skip(w)?,
        }
    }
    if version != 2 {
        return Err(MvtError::UnsupportedVersion(version));
    }
    Ok(MvtLayer {
        name,
        features,
        keys,
        values,
        extent,
        version,
    })
}

#[cfg(test)]
mod varint_tests {
    use super::*;

    #[test]
    fn varint_known_vectors() {
        let mut r = Reader::new(&[0x00]);
        assert_eq!(r.varint().unwrap(), 0);
        let mut r = Reader::new(&[0x96, 0x01]);
        assert_eq!(r.varint().unwrap(), 150);
        let mut r = Reader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert_eq!(r.varint().unwrap(), 0xFFFF_FFFF);
    }

    #[test]
    fn zigzag_known_vectors() {
        assert_eq!(zigzag(0), 0);
        assert_eq!(zigzag(1), -1);
        assert_eq!(zigzag(2), 1);
        assert_eq!(zigzag(59), -30);
        assert_eq!(zigzag(79), -40);
    }
}
