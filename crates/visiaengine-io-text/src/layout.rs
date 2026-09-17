//! IO-09：LTR 手工排布（advance 累加+bearing→px 偏移；空串=零 quad；组合标记不拆=档①声明）。
//! 坐标约定 [条款体]：`offset_px=(右, 上)` 相对基线锚，y 向上为正（渲染端 vs_label 翻屏序）。

use crate::cache::GlyphCache;
use crate::face::FontFace;

/// 单 quad：世界锚上一个字形的屏幕恒大小贴片表源（uv 归一化于 [`GLYPH_ATLAS_PX`]）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelQuad {
    /// 字形左上角相对锚点 (dx 右正, dy 上正) px。
    pub top_left_px: [f32; 2],
    /// 像素尺寸 (w, h)。
    pub size_px: [f32; 2],
    /// atlas UV (左, 下)——y 向下行主序纹理。
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    /// 码点（诊断/测试谓词用）。
    pub ch: char,
}

/// text → quad 序列 + 总 advance 宽（居中/右对齐由调用方以宽度平移 dx）。
/// `size_px ≤ 0` → 空（域闸，不 panic）。零覆盖字形（space）出 advance 不出 quad。
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
            pen += size_px * 0.5; // 容灾：装不下的怪字给经验宽，后续字形继续排（不整串弃）
            continue;
        };
        pen += s.advance;
        let [x, y, w, h] = s.rect;
        if w == 0 || h == 0 {
            continue; // space 类：占进不产片
        }
        // top_left：x=pen_before+left；y=top（向上正）。pen_before=加 advance 前。
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
