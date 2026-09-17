//! visiaengine-io-text —— 文字标注装载（S2 带，档①引擎自管）。
//!
//! 三件纯 CPU 面：① [`FontFace`] fontdue 包装（宿主注入字节，IO-07）② [`GlyphCache`]
//! R8 atlas shelf 缓存（IO-08）③ [`layout`] LTR 排布产 quad 列（IO-09）。
//! GPU 态住 render-wgpu；LabelMark 表在 render IR（REND-33）。
//! 非目标（挂账触发制）：halo / billboard 立牌 / 换行富文本 bidi / 标签避让。

mod cache;
mod face;
mod layout;

pub use cache::{GLYPH_ATLAS_PX, GlyphCache, GlyphSlot};
pub use face::{FaceError, FontFace};
pub use layout::{LabelQuad, layout};
