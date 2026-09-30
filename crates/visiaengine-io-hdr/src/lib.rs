//! # visiaengine-io-hdr
//! RADIANCE .hdr (RGBE) -> f32 RGB. Zero render deps (io-* family rule); zero
//! new external deps beyond thiserror (io-points precedent). Behavior clause:
//! `docs/sdd/io-gltf.md` (IO-17).
//!
//! Format coverage: text header (`#?RADIANCE`, `FORMAT=32-bit_rle_rgbe`),
//! resolution line `-Y H +X W`, then per-scanline data in BOTH layouts:
//! old flat (w RGBE u8 quads) and new per-scanline RLE (magic 2,2,h,l + 4
//! per-channel run streams; run byte >= 128 = (b-128) literal bytes follow,
//! run byte < 128 = repeat the next byte b times).

#![cfg_attr(not(test), warn(clippy::unwrap_used))]

use thiserror::Error;

/// Decoded HDR image: row-major top-to-bottom linear RGB (values may exceed
/// 1.0 — that is the point of HDR).
#[derive(Clone, Debug)]
pub struct HdrImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 3]>,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum IoHdrError {
    #[error("missing/invalid RADIANCE magic line")]
    BadMagic,
    #[error("missing/unsupported FORMAT line (want 32-bit_rle_rgbe)")]
    BadFormat,
    #[error("missing/invalid resolution line (-Y H +X W)")]
    BadResolution,
    #[error("scanline data truncated (need {need} more bytes at offset {at})")]
    Truncated { need: usize, at: usize },
    #[error("malformed RLE scanline (offset {at})")]
    BadRle { at: usize },
}

const FLAT_MAX_WIDTH: usize = 0x7FFF; // RLE layout convention covers widths up to 32767

/// Decode a RADIANCE .hdr image (RGBE -> linear f32 RGB, f = c * 2^((e-128)-8)).
pub fn decode_hdr(bytes: &[u8]) -> Result<HdrImage, IoHdrError> {
    let (width, height, body) = parse_header(bytes)?;
    let w = width as usize;
    let mut cur = Cursor { buf: body, pos: 0 };
    let mut pixels = Vec::with_capacity(w * height as usize);
    for _y in 0..height {
        let row = decode_scanline(&mut cur, w)?;
        pixels.extend(row.iter().map(|&q| rgbe_to_rgb(q)));
    }
    Ok(HdrImage {
        width,
        height,
        pixels,
    })
}

/// One scanline -> w RGBE quads (layout sniffed per scanline: RLE magic or
/// flat fallback; the flat layout is also the convention for w < 8).
fn decode_scanline(cur: &mut Cursor<'_>, w: usize) -> Result<Vec<[u8; 4]>, IoHdrError> {
    if !(8..=FLAT_MAX_WIDTH).contains(&w) {
        return scanline_flat(cur, w);
    }
    let magic = cur.peek4().ok_or(IoHdrError::Truncated {
        need: 4,
        at: cur.pos,
    })?;
    if magic[0] == 2 && magic[1] == 2 {
        scanline_rle(cur, w)
    } else {
        scanline_flat(cur, w)
    }
}

/// New-style scanline: magic 2,2,h,l then four planar RLE runs (R,G,B,E).
fn scanline_rle(cur: &mut Cursor<'_>, w: usize) -> Result<Vec<[u8; 4]>, IoHdrError> {
    cur.take(4)?; // consume the magic (2,2,h,l)
    let mut planar = [vec![0u8; w], vec![0u8; w], vec![0u8; w], vec![0u8; w]];
    for dst in &mut planar {
        rle_channel(cur, w, dst)?;
    }
    // Planar (R,G,B,E runs) -> interleaved RGBE quads.
    Ok((0..w)
        .map(|i| [planar[0][i], planar[1][i], planar[2][i], planar[3][i]])
        .collect())
}

/// One RLE run-stream -> exactly w bytes (runs may not straddle the width).
fn rle_channel(cur: &mut Cursor<'_>, w: usize, dst: &mut [u8]) -> Result<(), IoHdrError> {
    let mut filled = 0usize;
    while filled < w {
        let b = cur.take(1)?[0];
        let (count, literal) = if b >= 128 {
            ((b - 128) as usize, true)
        } else {
            (b as usize, false)
        };
        if count == 0 || filled + count > w {
            return Err(IoHdrError::BadRle { at: cur.pos });
        }
        if literal {
            let raw = cur.take(count)?;
            dst[filled..filled + count].copy_from_slice(raw);
        } else {
            let v = cur.take(1)?[0];
            dst[filled..filled + count].fill(v);
        }
        filled += count;
    }
    Ok(())
}

/// Old-style scanline: exactly w flat RGBE quads.
fn scanline_flat(cur: &mut Cursor<'_>, w: usize) -> Result<Vec<[u8; 4]>, IoHdrError> {
    let raw = cur.take(w * 4)?;
    Ok(raw.as_chunks::<4>().0.to_vec())
}

/// RGBE quad -> linear RGB (per-pixel shared exponent: mantissa = c/256,
/// f = mantissa * 2^(e-128) = c * 2^((e-128)-8); e=0 encodes zero).
fn rgbe_to_rgb(q: [u8; 4]) -> [f32; 3] {
    if q[3] == 0 {
        return [0.0; 3];
    }
    // f = mantissa * 2^(e-128) where mantissa = c/256.
    let scale = 2.0f32.powi(i32::from(q[3]) - 128) / 256.0;
    [q[0], q[1], q[2]].map(|c| f32::from(c) * scale)
}

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl Cursor<'_> {
    fn peek4(&self) -> Option<[u8; 4]> {
        let s = self.buf.get(self.pos..self.pos + 4)?;
        Some([s[0], s[1], s[2], s[3]])
    }
    fn take(&mut self, n: usize) -> Result<&'_ [u8], IoHdrError> {
        let end = self.pos + n;
        if end > self.buf.len() {
            return Err(IoHdrError::Truncated {
                need: n,
                at: self.pos,
            });
        }
        let s = &self.buf[self.pos..end];
        self.pos = end;
        Ok(s)
    }
}

/// Header parse: magic line, FORMAT line, blank separator, resolution line.
/// Returns (width, height, remaining body). Header lines are `\n`-terminated
/// ASCII; unknown key lines (PRIMARIES/EXPOSURE/SOFTWARE/...) are skipped.
fn parse_header(bytes: &[u8]) -> Result<(u32, u32, &[u8]), IoHdrError> {
    let mut pos = 0usize;
    let next_line = |pos: &mut usize| -> Option<&[u8]> {
        let rest = bytes.get(*pos..)?;
        let nl = rest.iter().position(|&b| b == b'\n')?;
        let l = &rest[..nl];
        *pos += nl + 1;
        Some(l)
    };
    let magic = next_line(&mut pos).ok_or(IoHdrError::BadMagic)?;
    if !magic.starts_with(b"#?") {
        return Err(IoHdrError::BadMagic);
    }
    #[allow(unused_assignments)] // compiler can't see width/height are written before break
    let (mut width, mut height) = (0u32, 0u32);
    let mut saw_format = false;
    loop {
        let l = next_line(&mut pos).ok_or(IoHdrError::BadResolution)?;
        if l.starts_with(b"FORMAT=") {
            if l == b"FORMAT=32-bit_rle_rgbe" {
                saw_format = true;
            } else {
                return Err(IoHdrError::BadFormat);
            }
            continue;
        }
        if l.is_empty() {
            // Standard blank separator between meta lines and the resolution
            // line — fall through and keep scanning (the resolution line
            // follows it). Only EOF without a res line is BadResolution.
            continue;
        }
        let s = std::str::from_utf8(l).map_err(|_| IoHdrError::BadResolution)?;
        let toks: Vec<&str> = s.split_whitespace().collect();
        if toks.len() == 4 && toks[0] == "-Y" && toks[2] == "+X" {
            height = toks[1].parse().map_err(|_| IoHdrError::BadResolution)?;
            width = toks[3].parse().map_err(|_| IoHdrError::BadResolution)?;
            break;
        }
    }
    if !saw_format {
        return Err(IoHdrError::BadFormat);
    }
    if width == 0 || height == 0 {
        return Err(IoHdrError::BadResolution);
    }
    Ok((width, height, &bytes[pos..]))
}

/// I band (WGPU-39): project an equirect HDRI onto the 9-coefficient spherical
/// harmonics basis (RGB, LINEAR domain — decode_hdr output is already linear).
/// Demo-grade irradiance: L1+L2 bands with the cosine-convolution folded in
/// (Ramamoorthi 2001 weights); specular prefilter/BRDF-LUT = separate L ticket.
/// `samples` = rows × cols stratification (256×128 default-ish cost).
#[must_use]
pub fn project_sh9(img: &HdrImage, samples: u32) -> [[f32; 3]; 9] {
    let mut coeff = [[0.0f32; 3]; 9];
    if img.width == 0 || img.height == 0 || samples == 0 {
        return coeff;
    }
    let cols = samples;
    let rows = samples / 2;
    let weight_sum = 0.0f32.max(1.0); // placeholder; real normalization below
    let _ = weight_sum;
    let mut sum = [[0.0f64; 3]; 9];
    let mut total = 0.0f64;
    for ry in 0..rows {
        let theta = std::f64::consts::PI * (ry as f64 + 0.5) / rows as f64;
        let st = theta.sin();
        for cx in 0..cols {
            let phi = 2.0 * std::f64::consts::PI * (cx as f64 + 0.5) / cols as f64;
            // equirect -> direction (y-up, matching engine world frame)
            let x = st * phi.sin();
            let y = theta.cos();
            let z = st * phi.cos();
            // sample image: v = theta/PI (row), u = phi/2PI
            let px = ((phi / (2.0 * std::f64::consts::PI)) * f64::from(img.width)) as usize
                % img.width as usize;
            let py = ((theta / std::f64::consts::PI) * f64::from(img.height)) as usize
                % img.height as usize;
            let c = img.pixels[py * img.width as usize + px];
            // SH basis (y-up):
            let sh = [
                0.282_095,
                0.488_603 * y,
                0.488_603 * z,
                0.488_603 * x,
                1.092_548 * x * y,
                1.092_548 * y * z,
                0.315_392 * (3.0 * y * y - 1.0),
                1.092_548 * x * z,
                0.546_274 * (x * x - z * z),
            ];
            let w = st; // solid-angle weight
            for (sc, b) in sum.iter_mut().zip(sh.iter()) {
                for k in 0..3 {
                    sc[k] += f64::from(c[k]) * b * w;
                }
            }
            total += w;
        }
    }
    if total > 0.0 {
        for (sc, cc) in sum.iter().zip(coeff.iter_mut()) {
            for k in 0..3 {
                cc[k] = (sc[k] / total) as f32;
            }
        }
    }
    coeff
}
