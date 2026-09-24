//! TileSet scheduler (IO-13/14, tile-streaming Phase 1).
//!
//! `visible()` is pure slippy enumeration (IO-11 reuse); `ensure()` loads
//! missing tiles synchronously (Phase 0 posture holds: pull-model engine, the
//! host drives frames — HTTP lands behind the same sync facade via ureq);
//! `decoded()` reads through the LRU cache.

use crate::mvt::{MvtTile, decode_tile};
use crate::source::{LruCache, SourceError, TileSource};
use crate::tiles::TileId;
use thiserror::Error;

/// Scheduler-level error: source failures are wrapped; decode failures surface
/// as Source-independent variants (IO-10 taxonomy passes through).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TilesError {
    #[error("source: {0}")]
    Source(#[from] SourceError),
    #[error("decode: {0}")]
    Decode(String),
}

/// ensure() accounting (IO-14): loaded vs cache-hit counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnsureStats {
    pub loaded: usize,
    pub cached: usize,
}

/// Viewport-driven tile set: enumerate → ensure → decode.
pub struct TileSet {
    source: Box<dyn TileSource>,
    cache: LruCache,
    decoded_cache: std::collections::HashMap<TileId, MvtTile>,
}

impl TileSet {
    /// Construct with a capacity-bounded cache (tiles).
    pub fn new(source: Box<dyn TileSource>) -> Result<Self, TilesError> {
        Ok(Self {
            source,
            cache: LruCache::new(64),
            decoded_cache: std::collections::HashMap::new(),
        })
    }

    /// Pure enumeration: all tiles covering `bbox` (3857 meters) at zoom `z`.
    /// Row-major order (y outer, x inner) — stable paint order for callers.
    /// No world wrap: tiles outside [0, 2^z) are skipped (caller policy to wrap).
    #[must_use]
    pub fn visible(bbox: (f64, f64, f64, f64), z: u8) -> Vec<TileId> {
        let n = 1u64 << z;
        let world = crate::tiles::WORLD_EXTENT;
        let tile = world / n as f64;
        let half = world * 0.5;
        // Half-open coverage: a tile is included when its CLOSED cell area
        // intersects the bbox. Max edges landing exactly on a tile boundary do
        // NOT pull in the next tile (zero-area touch = no coverage).
        // Boundary-snapping helper: f within relative-eps of an integer IS that
        // integer (f64 world math wobbles ~1e-14 at 2^z scale). Returns
        // (snapped, was_exact_boundary).
        let snap = |f: f64| -> (f64, bool) {
            let fl = f.floor();
            let eps = f.abs() * 1e-12;
            if f - fl < eps {
                (fl, true) // just above an integer → that integer, boundary
            } else if f.ceil() - f < eps {
                (f.ceil(), true) // just below an integer → that integer, boundary
            } else {
                (f.ceil(), false) // interior → next tile start (ceil), not boundary
            }
        };
        // Half-open coverage [min_edge, max_edge): tiles whose open cell area
        // intersects the bbox. Max edge exactly on a boundary → exclusive WITHOUT
        // +1 (zero-area touch pulls no extra tile); otherwise +1 past floor.
        let (x0f, _) = snap((bbox.0 + half) / tile);
        let (x1f, x1b) = snap((bbox.2 + half) / tile);
        let x0 = x0f.max(0.0);
        let x1 = if x1b { x1f } else { x1f + 1.0 };
        // y: 3857 y-up vs slippy y-down — invert (bbox top max_y → smaller slippy y)
        let (y0f, _) = snap((half - bbox.3) / tile);
        let (y1f, y1b) = snap((half - bbox.1) / tile);
        let y0 = y0f.max(0.0);
        let y1 = if y1b { y1f } else { y1f + 1.0 };
        let mut out = Vec::new();
        let (x1e, y1e) = ((x1 as u64).min(n), (y1 as u64).min(n));
        for y in (y0 as u64)..y1e {
            for x in (x0 as u64)..x1e {
                if let Some(id) = TileId::new(z, x as u32, y as u32) {
                    out.push(id);
                }
            }
        }
        out
    }

    /// Load every missing tile into the cache (sync). Idempotent: second call
    /// with the same ids is all cache hits. First source error aborts (typed).
    pub fn ensure(&mut self, ids: &[TileId]) -> Result<EnsureStats, TilesError> {
        let mut stats = EnsureStats {
            loaded: 0,
            cached: 0,
        };
        for id in ids {
            if self.cache.get(id).is_some() {
                stats.cached += 1;
                continue;
            }
            let bytes = self.source.load(id.z, id.x, id.y)?;
            self.cache.insert(*id, bytes);
            stats.loaded += 1;
        }
        Ok(stats)
    }

    /// Decode-through-cache: returns the decoded tile, decoding on first access.
    pub fn decoded(&mut self, id: &TileId) -> Option<&MvtTile> {
        if !self.decoded_cache.contains_key(id) {
            let bytes = self.cache.get(id)?.to_vec();
            let tile = decode_tile(&bytes)
                .map_err(|e| TilesError::Decode(e.to_string()))
                .ok()?;
            self.decoded_cache.insert(*id, tile);
        }
        self.decoded_cache.get(id)
    }
}
