//! visiaengine-io-text —— 文字标注装载（S2 带，档①引擎自管）。
//!
//! 三件纯 CPU 面：① [`FontFace`] fontdue 包装（宿主注入字节，IO-07）② [`GlyphCache`]
//! LRU atlas 装箱（IO-08）③ [`layout`] LTR 排布产 LabelMark（IO-09）。GPU 态住 render-wgpu。
//! 非目标（挂账触发制）：halo 描边 / 世界立牌 billboard / 换行富文本 bidi / 标签避让排布。

mod cache;
mod face;
mod layout;

pub use cache::{GlyphCache, GLYPH_ATLAS_PX};
pub use face::{FontFace, fontdue_smoke};
pub use layout::{layout, LabelQuad};
