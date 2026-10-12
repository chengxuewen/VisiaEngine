//! IO-07: the font face load contract. The host injects bytes (no default font); non-TTF / empty are rejected.
//! The rasterization domain `size_px > 0` is gated at the cache entry (this layer passes through to fontdue).

/// The engine-side wrapper of the host-injected font bytes (no default font; the text pipeline activates only on a successful load).
pub struct FontFace {
    inner: fontdue::Font,
}

// fontdue::Font has no Debug -- hand-written form (the error-message path needs the Result::unwrap_err bound)
impl core::fmt::Debug for FontFace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("FontFace(<fontdue::Font>)")
    }
}

/// Load-failure typology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceError {
    /// empty bytes
    Empty,
    /// not TTF / parse failure
    Parse,
}

impl FontFace {
    /// Parse font bytes (fontdue `default-features=false` + the hashbrown face: no SIMD/std).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FaceError> {
        if bytes.is_empty() {
            return Err(FaceError::Empty);
        }
        fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
            .map(|inner| Self { inner })
            .map_err(|_| FaceError::Parse)
    }

    /// Rasterize one glyph @ font-size px -> (metrics, bitmap w x h row-major top-to-bottom).
    #[must_use]
    pub fn rasterize(&self, ch: char, size_px: f32) -> (fontdue::Metrics, Vec<u8>) {
        self.inner.rasterize(ch, size_px)
    }
}
