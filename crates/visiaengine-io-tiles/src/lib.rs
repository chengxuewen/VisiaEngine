//! # visiaengine-io-tiles
//!
//! Tile streaming Phase 0 (architecture 7 blueprint, super-band B3):
//! slippy tile math + hand-rolled MVT (Mapbox Vector Tile) decoder.
//! Zero network deps — HTTP sources are Phase 1 (dated commitment in the
//! whitepaper reword); this crate is the Rust-native foundation layer.

pub mod geometry;
pub mod mvt;
pub mod source;
pub mod tiles;

pub use geometry::{GeoTile, TileFeature, TileGeom};
pub use mvt::{MvtError, MvtFeature, MvtLayer, MvtTile, MvtValue, decode_tile};
pub use source::{FileSource, HttpSource, LruCache, SourceError, TileSource};
pub use tiles::{TileId, WORLD_EXTENT};
