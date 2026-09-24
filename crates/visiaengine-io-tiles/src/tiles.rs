//! Slippy tile math (IO-11): z/x/y <-> Web Mercator (EPSG:3857) bbox, pure functions.
//!
//! Semantics pinned here (clause body in docs/sdd/io-tiles.md):
//! - z=0 covers the whole world (one tile, bbox = full 3857 extent)
//! - x/y must be in [0, 2^z): out-of-range = None (wrap-around is a caller
//!   policy, never silent here)
//! - bbox order: (min_x, min_y, max_x, max_y) in 3857 meters
//! - zoom is an integer z (float zoom flooring is the scheduler's job, kept
//!   out of the pure math)

/// World extent in EPSG:3857 meters (WGS84 datum sphere approximation).
pub const WORLD_EXTENT: f64 = 40_075_016.685_578_49; // 2*pi*R, R=6378137

/// A slippy tile identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileId {
    pub z: u8,
    pub x: u32,
    pub y: u32,
}

impl TileId {
    /// Validate + construct: x/y must be within [0, 2^z). None = out of range.
    #[must_use]
    pub fn new(z: u8, x: u32, y: u32) -> Option<Self> {
        let n: u64 = 1 << z;
        if (x as u64) < n && (y as u64) < n {
            Some(Self { z, x, y })
        } else {
            None
        }
    }

    /// 3857 bbox (min_x, min_y, max_x, max_y). z=0 = whole world.
    #[must_use]
    pub fn bbox(&self) -> (f64, f64, f64, f64) {
        let n = f64::from(1u32 << self.z);
        let half = WORLD_EXTENT * 0.5;
        let tile = WORLD_EXTENT / n;
        let min_x = -half + f64::from(self.x) * tile;
        let max_x = min_x + tile;
        // 3857 y grows north; slippy y=0 is the TOP row (northmost).
        let max_y = half - f64::from(self.y) * tile;
        let min_y = max_y - tile;
        (min_x, min_y, max_x, max_y)
    }
}
