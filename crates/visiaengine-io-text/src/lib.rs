//! visiaengine-io-text -- text-label loading (S2 band, tier-1 engine self-managed).
//!
//! Three pure-CPU faces: (1) [`FontFace`] the fontdue wrapper (host-injected bytes, IO-07) (2) [`GlyphCache`]
//! the R8 atlas shelf cache (IO-08) (3) [`layout`] LTR layout emitting quad lists (IO-09).
//! GPU state lives in render-wgpu; the LabelMark table is in the render IR (REND-33).
//! Non-goals (backlog on demand): halo / billboard strips / wrapping rich-text bidi / label collision avoidance.

mod cache;
mod face;
mod layout;

pub use cache::{GLYPH_ATLAS_PX, GlyphCache, GlyphSlot};
pub use face::{FaceError, FontFace};
pub use layout::{LabelQuad, layout};
