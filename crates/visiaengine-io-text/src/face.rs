//! IO-07：font face 装载契约。宿主注入字节（无默认字体）；非 TTF/空 拒。
//! 栅格域 `size_px > 0` 由 cache 入口闸（本层透传 fontdue）。

/// 宿主注入字体字节的引擎侧包装（无默认字体；装载成功才激活文字管线）。
pub struct FontFace {
    inner: fontdue::Font,
}

// fontdue::Font 无 Debug——手写形态（错误消息路径需 Result::unwrap_err 约束）
impl core::fmt::Debug for FontFace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("FontFace(<fontdue::Font>)")
    }
}

/// 装载失败分型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceError {
    /// 空字节
    Empty,
    /// 非 TTF/解析失败
    Parse,
}

impl FontFace {
    /// 解析字体字节（fontdue `default-features=false`+hashbrown 面：无 SIMD/std）。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FaceError> {
        if bytes.is_empty() {
            return Err(FaceError::Empty);
        }
        fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
            .map(|inner| Self { inner })
            .map_err(|_| FaceError::Parse)
    }

    /// 栅格化单字形 @字号 px → (度量, 位图 w×h 行主序自上而下)。
    #[must_use]
    pub fn rasterize(&self, ch: char, size_px: f32) -> (fontdue::Metrics, Vec<u8>) {
        self.inner.rasterize(ch, size_px)
    }
}
