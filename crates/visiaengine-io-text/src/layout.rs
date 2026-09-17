//! IO-09：LTR 手工排布（advance 累加 + bearing → dx/dy px；空串=零 quad）。

/// 单 quad：世界锚点上的一个字形（vs_label 屏幕恒大小展开的表源）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabelQuad {
    /// 字形左上角相对锚点的屏幕像素偏移 (dx, dy)。
    pub offset_px: [f32; 2],
    /// 字形像素尺寸 (w, h)。
    pub size_px: [f32; 2],
    /// atlas UV 左下/右上（N1 填）。
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
}

/// 骨架占位（N1 填实体）：文本 → quad 序列 + 总宽。
#[must_use]
pub fn layout(_text: &str, _size_px: f32) -> (Vec<LabelQuad>, f32) {
    (Vec::new(), 0.0)
}
