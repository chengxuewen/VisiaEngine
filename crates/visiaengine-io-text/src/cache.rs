//! IO-08：glyph atlas LRU cache（key=(glyph,size)，shelf 装箱，容量上限=丢最旧降级）。

/// atlas 边长（纹理 512²；dirty 全量上传，行粒度升级=后带 [ponytail]）。
pub const GLYPH_ATLAS_PX: u32 = 512;

/// 字形位图图集缓存（骨架占位，N1 填实体）。
#[derive(Debug)]
pub struct GlyphCache {
    _private: (),
}
