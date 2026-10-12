//! IO-08: glyph atlas shelf cache. R8 coverage atlas, key = (char, size x 10).
//! When full, the whole texture is cleared and re-baked (fontdue determinism = transparent rebuild, jitter but no crash).
//! ponytail: wholesale reclaim, not a per-glyph LRU -- per-glyph LRU waits until a profile to prove the re-bake hurts (`reset()` swaps in one place).

use crate::face::FontFace;
use std::collections::HashMap;

/// Atlas side length (512^2 R8 = 256KB; 64^2 average glyph ~ 1024 slots).
pub const GLYPH_ATLAS_PX: u32 = 512;

/// One glyph's rectangle in the atlas + layout metrics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphSlot {
    /// Atlas pixel rectangle (x, y, w, h); y downward (row-major upload). Zero rect = no coverage (space class).
    pub rect: [u32; 4],
    /// Pen advance.
    pub advance: f32,
    /// Glyph left bearing anchor (fontdue xmin, may be negative).
    pub left: f32,
    /// Glyph top above baseline (ymin + height, upward positive).
    pub top: f32,
}

type Key = (char, u32);
fn key_of(ch: char, size_px: f32) -> Key {
    (ch, (size_px * 10.0).round() as u32)
}

/// R8 glyph atlas cache.
pub struct GlyphCache {
    pixels: Vec<u8>,
    entries: HashMap<Key, GlyphSlot>,
    shelf_x: u32,
    shelf_y: u32,
    shelf_h: u32,
    dirty: bool,
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new()
    }
}

impl GlyphCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pixels: vec![0u8; (GLYPH_ATLAS_PX * GLYPH_ATLAS_PX) as usize],
            entries: HashMap::new(),
            shelf_x: 0,
            shelf_y: 0,
            shelf_h: 0,
            dirty: false,
        }
    }

    /// Upload face: R8 pixels (row-major, width [`GLYPH_ATLAS_PX`]).
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Take the dirty flag (new rasterization = true, cleared after take) -- the render side decides texture re-upload by it.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Final entry: a hit returns the slot; a miss rasterizes and packs; atlas full -> clear everything, re-bake once, retry.
    /// `size_px <= 0 / non-finite` -> None (the IO-07/08 domain gate).
    pub fn get(&mut self, face: &FontFace, ch: char, size_px: f32) -> Option<GlyphSlot> {
        self.raster(face, ch, size_px).or_else(|| {
            self.reset();
            self.raster(face, ch, size_px)
        })
    }

    fn raster(&mut self, face: &FontFace, ch: char, size_px: f32) -> Option<GlyphSlot> {
        if size_px <= 0.0 || !size_px.is_finite() {
            return None;
        }
        let k = key_of(ch, size_px);
        if let Some(s) = self.entries.get(&k) {
            return Some(*s);
        }
        let (m, bmp) = face.rasterize(ch, size_px);
        let (w, h) = (m.width as u32, m.height as u32);
        let top = m.ymin as f32 + m.height as f32;
        let slot = if w == 0 || h == 0 {
            // zero coverage (space class): record the advance, take no atlas space, do not dirty the texture
            GlyphSlot {
                rect: [0, 0, 0, 0],
                advance: m.advance_width,
                left: m.xmin as f32,
                top,
            }
        } else {
            let (x, y) = self.alloc(w, h)?; // full = None, the re-bake triggered by get
            self.blit(x, y, w, h, &bmp);
            self.dirty = true;
            GlyphSlot {
                rect: [x, y, w, h],
                advance: m.advance_width,
                left: m.xmin as f32,
                top,
            }
        };
        self.entries.insert(k, slot);
        Some(slot)
    }

    /// Shelf-cursor packing (1px seam = linear sampling guards against neighbor bleed); does not fit -> next row; overrun -> atlas-full signal.
    fn alloc(&mut self, w: u32, h: u32) -> Option<(u32, u32)> {
        const PAD: u32 = 1;
        if self.shelf_x + w + PAD > GLYPH_ATLAS_PX {
            self.shelf_y += self.shelf_h + PAD;
            self.shelf_x = 0;
            self.shelf_h = 0;
        }
        if self.shelf_y + h + PAD > GLYPH_ATLAS_PX {
            return None;
        }
        let at = (self.shelf_x, self.shelf_y);
        self.shelf_x += w + PAD;
        self.shelf_h = self.shelf_h.max(h);
        Some(at)
    }

    /// Clear-all re-bake (transparent downgrade; when one glyph exceeds the whole texture capacity, get still returns None = the caller skips).
    fn reset(&mut self) {
        self.pixels.iter_mut().for_each(|p| *p = 0);
        self.entries.clear();
        self.shelf_x = 0;
        self.shelf_y = 0;
        self.shelf_h = 0;
        self.dirty = true;
    }

    fn blit(&mut self, x: u32, y: u32, w: u32, h: u32, src: &[u8]) {
        debug_assert_eq!(
            src.len(),
            (w * h) as usize,
            "the fontdue bitmap must be w x h row-major"
        );
        let stride = GLYPH_ATLAS_PX as usize;
        for (row, chunk) in src.chunks_exact(w as usize).enumerate() {
            let oy = (y as usize + row) * stride + x as usize;
            self.pixels[oy..oy + w as usize].copy_from_slice(chunk);
        }
    }
}
