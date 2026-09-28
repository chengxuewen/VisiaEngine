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

/// Per-tile pump lifecycle (IO-16, band N1.4). Four states, cesium-7 reduced:
/// `FailedTemporarily` vs a terminal failure split is kept (retry policy stays
/// the caller's); Unloading/ContentLoaded wait-states are not modeled (sync
/// pump: a tile is either being loaded this frame or not).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileState {
    /// Not requested (also the state of tiles absent from the map).
    Unloaded,
    /// begin() issued; awaiting pump budget.
    Loading,
    /// Bytes in the LRU cache.
    Done,
    /// Last pump attempt failed (source error recorded); begin() re-arms.
    FailedTemporarily,
}

/// Viewport-driven tile set: enumerate → ensure → decode.
pub struct TileSet {
    source: Box<dyn TileSource>,
    cache: LruCache,
    decoded_cache: std::collections::HashMap<TileId, MvtTile>,
    /// IO-16 (N1.4): per-tile pump state. Done tiles live in the LRU; the
    /// state map tracks the remaining lifecycle. Absent == Unloaded.
    pump_state: std::collections::HashMap<TileId, TileState>,
    /// Last per-tile source error (FailedTemporarily diagnostic; retried tiles
    /// overwrite it). Kept small: one entry per failed tile.
    pump_errors: std::collections::HashMap<TileId, String>,
}

impl TileSet {
    /// Construct with a capacity-bounded cache (tiles).
    pub fn new(source: Box<dyn TileSource>) -> Result<Self, TilesError> {
        Ok(Self {
            source,
            cache: LruCache::new(64),
            decoded_cache: std::collections::HashMap::new(),
            pump_state: std::collections::HashMap::new(),
            pump_errors: std::collections::HashMap::new(),
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

    /// IO-16 (N1.4): mark `ids` as Loading (state transition only, zero I/O).
    /// Skips anything not Unloaded/FailedTemporarily (Done/Loading are stable).
    /// Returns the count of tiles that transitioned.
    pub fn begin(&mut self, ids: &[TileId]) -> usize {
        let mut started = 0;
        for id in ids {
            match self.pump_state.get(id) {
                Some(TileState::Done) | Some(TileState::Loading) => {}
                _ => {
                    self.pump_state.insert(*id, TileState::Loading);
                    started += 1;
                }
            }
        }
        started
    }

    /// IO-16 (N1.4): load up to `budget` Loading tiles (≤ budget I/O calls this
    /// call). Per-tile: Ok → Done (+ LRU insert), Err → FailedTemporarily
    /// (error recorded, pump continues — never aborts, unlike ensure()).
    /// Returns (done_now, failed_now). No pending Loading tiles → (0, 0).
    pub fn pump(&mut self, budget: usize) -> (usize, usize) {
        let pending: Vec<TileId> = self
            .pump_state
            .iter()
            .filter(|(_, st)| **st == TileState::Loading)
            .map(|(id, _)| *id)
            .take(budget)
            .collect();
        let (mut done, mut failed) = (0usize, 0usize);
        for id in pending {
            match self.source.load(id.z, id.x, id.y) {
                Ok(bytes) => {
                    self.cache.insert(id, bytes);
                    self.pump_state.insert(id, TileState::Done);
                    self.pump_errors.remove(&id);
                    done += 1;
                }
                Err(e) => {
                    self.pump_state.insert(id, TileState::FailedTemporarily);
                    self.pump_errors.insert(id, e.to_string());
                    failed += 1;
                }
            }
        }
        (done, failed)
    }

    /// IO-16 (N1.4): pump-lifecycle state of one tile (Unloaded = absent map).
    #[must_use]
    pub fn state(&self, id: &TileId) -> TileState {
        self.pump_state
            .get(id)
            .copied()
            .unwrap_or(TileState::Unloaded)
    }

    /// IO-16 (N1.4): last recorded source error for a FailedTemporarily tile.
    #[must_use]
    pub fn pump_error(&self, id: &TileId) -> Option<&str> {
        self.pump_errors.get(id).map(String::as_str)
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
