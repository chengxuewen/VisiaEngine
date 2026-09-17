//! IO-08：glyph atlas shelf 缓存。R8 覆盖图集，key=(char,size×10)。
//! 满则整图清空重烘（fontdue 确定性=透明可重建，抖而不崩）。
//! ponytail: 全量 reclaim 非逐字形 LRU——逐字 LRU 待 profile 证实重烘疼再上（`reset()` 一处换）。

use crate::face::FontFace;
use std::collections::HashMap;

/// atlas 边长（512² R8=256KB；64² 字形均 ≈ 1024 槽）。
pub const GLYPH_ATLAS_PX: u32 = 512;

/// 单字形在 atlas 的矩形 + 布局度量。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphSlot {
    /// atlas 像素矩形 (x, y, w, h)；y 向下（行主序上传）。零 rect=无覆盖（space 类）。
    pub rect: [u32; 4],
    /// pen 前进量。
    pub advance: f32,
    /// 字形左距锚（fontdue xmin，可负）。
    pub left: f32,
    /// 字形顶距基线（ymin+height，向上为正）。
    pub top: f32,
}

type Key = (char, u32);
fn key_of(ch: char, size_px: f32) -> Key {
    (ch, (size_px * 10.0).round() as u32)
}

/// R8 字形图集缓存。
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

    /// 上传面：R8 像素（行主序，宽 [`GLYPH_ATLAS_PX`]）。
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// 取走 dirty（有新栅格=真，取后清）——渲染端据此决定是否重传纹理。
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// 最终入口：命中返槽；未中栅格装箱；图集满→全清重烘一次再试。
    /// `size_px ≤ 0/非有限` → None（IO-07/08 域闸）。
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
            // 零覆盖（space 类）：记 advance，不占 atlas、不脏图
            GlyphSlot {
                rect: [0, 0, 0, 0],
                advance: m.advance_width,
                left: m.xmin as f32,
                top,
            }
        } else {
            let (x, y) = self.alloc(w, h)?; // 满=None，由 get 触发重烘
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

    /// shelf 游标装箱（1px 缝=线性采样防邻字渗色）；放不下→换行；越界→图集满信号。
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

    /// 全清重烘（透明降级；单字形超过整图容量时 get 仍 None=调用方跳过）。
    fn reset(&mut self) {
        self.pixels.iter_mut().for_each(|p| *p = 0);
        self.entries.clear();
        self.shelf_x = 0;
        self.shelf_y = 0;
        self.shelf_h = 0;
        self.dirty = true;
    }

    fn blit(&mut self, x: u32, y: u32, w: u32, h: u32, src: &[u8]) {
        debug_assert_eq!(src.len(), (w * h) as usize, "fontdue 位图必 w×h 行主序");
        let stride = GLYPH_ATLAS_PX as usize;
        for (row, chunk) in src.chunks_exact(w as usize).enumerate() {
            let oy = (y as usize + row) * stride + x as usize;
            self.pixels[oy..oy + w as usize].copy_from_slice(chunk);
        }
    }
}
