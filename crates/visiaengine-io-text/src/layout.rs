//! IO-09: manual LTR layout (advance accumulation + bearing -> px offsets; empty string = zero quads; combining marks not split = tier-1 declaration).
//! Coordinate convention [clause body]: `offset_px = (right, up)` relative to the baseline anchor, y upward positive (the render side flips to screen order in vs_label).

use crate::cache::GlyphCache;
use crate::face::FontFace;

/// One quad: the strip-table source of one glyph at a world anchor (uv normalized over [`GLYPH_ATLAS_PX`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelQuad {
    /// The glyph's top-left corner relative to the anchor (dx right positive, dy up positive) px.
    pub top_left_px: [f32; 2],
    /// Pixel size (w, h).
    pub size_px: [f32; 2],
    /// Atlas UV (left, bottom) -- y-downward row-major texture.
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    /// Codepoint (for diagnostics/test predicates).
    pub ch: char,
}

/// text -> quad sequence + total advance width (center/right alignment is done by the caller shifting dx by the width).
/// `size_px <= 0` -> empty (domain gate, no panic). Zero-coverage glyphs (space) emit advance but no quad.
#[must_use]
pub fn layout(
    text: &str,
    face: &FontFace,
    cache: &mut GlyphCache,
    size_px: f32,
) -> (Vec<LabelQuad>, f32) {
    let mut quads = Vec::new();
    let mut pen = 0.0f32;
    if size_px <= 0.0 || !size_px.is_finite() {
        return (quads, 0.0);
    }
    let a = crate::cache::GLYPH_ATLAS_PX as f32;
    for ch in text.chars() {
        let Some(s) = cache.get(face, ch, size_px) else {
            pen += size_px * 0.5; // disaster tolerance: an unplaceable odd glyph gets an empirical width, later glyphs keep laying out (the string is not abandoned)
            continue;
        };
        pen += s.advance;
        let [x, y, w, h] = s.rect;
        if w == 0 || h == 0 {
            continue; // space class: occupies advance, emits no patch
        }
        // top_left: x = pen_before + left; y = top (upward positive). pen_before = before adding the advance.
        let dx = pen - s.advance + s.left;
        quads.push(LabelQuad {
            top_left_px: [dx, s.top],
            size_px: [w as f32, h as f32],
            uv0: [x as f32 / a, y as f32 / a],
            uv1: [(x + w) as f32 / a, (y + h) as f32 / a],
            ch,
        });
    }
    (quads, pen)
}
