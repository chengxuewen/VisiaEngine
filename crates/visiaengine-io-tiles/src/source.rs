//! Tile source abstraction + LRU byte cache (B3 Phase 0).
//!
//! Phase 0 ships `FileSource` (local directory tree, `{root}/{z}/{x}/{y}.mvt`).
//! `HttpSource` is a typed stub — real network fetching is Phase 1 (dated
//! commitment in the whitepaper reword; zero network deps this band per D5).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::tiles::TileId;

/// Source-level load error (distinct from MvtError which is decode-side).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SourceError {
    #[error("tile file not found: {0}")]
    NotFound(String),
    #[error("tile read failed: {0}")]
    Io(String),
    #[error("source kind unsupported in Phase 0: {0}")]
    Unsupported(&'static str),
}

/// Tile byte source. Sync facade this phase (pull-model engine: the host drives
/// frames; a thread pool would be YAGNI for file sources). The trait is the
/// seam where an async HTTP implementation lands in Phase 1 without touching
/// the scheduler.
pub trait TileSource {
    fn load(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>, SourceError>;
}

/// Local directory source: `{root}/{z}/{x}/{y}.mvt`.
pub struct FileSource {
    root: PathBuf,
}

impl FileSource {
    #[must_use]
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }
}

impl TileSource for FileSource {
    fn load(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>, SourceError> {
        let path = self
            .root
            .join(z.to_string())
            .join(x.to_string())
            .join(format!("{y}.mvt"));
        let bytes = std::fs::read(&path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => SourceError::NotFound(path.display().to_string()),
            _ => SourceError::Io(format!("{}: {e}", path.display())),
        })?;
        Ok(bytes)
    }
}

/// HTTP placeholder — Phase 1. Constructible but every load fails with a typed
/// error (no silent dead-ends; the scheduler can probe capability).
#[derive(Debug, Clone, Copy, Default)]
pub struct HttpSource;

impl TileSource for HttpSource {
    fn load(&self, _z: u8, _x: u32, _y: u32) -> Result<Vec<u8>, SourceError> {
        Err(SourceError::Unsupported(
            "HTTP tile fetching lands in Phase 1",
        ))
    }
}

/// LRU byte cache keyed by tile id (stdlib-only: HashMap + monotonically
/// increasing stamp; ~40 lines, no `lru` crate per the zero-new-deps ruling).
///
/// ponytail: O(n) eviction scan on overflow — fine at tile counts (hundreds);
/// swap to an ordered map if entry counts ever reach tens of thousands.
pub struct LruCache {
    max_entries: usize,
    stamp: u64,
    entries: HashMap<TileId, (Vec<u8>, u64)>,
}

impl LruCache {
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries: max_entries.max(1),
            stamp: 0,
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: TileId, bytes: Vec<u8>) {
        self.stamp += 1;
        if self.entries.len() >= self.max_entries && !self.entries.contains_key(&id) {
            // evict least-recently-used (min stamp)
            if let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, (_, st))| *st)
                .map(|(k, _)| *k)
            {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(id, (bytes, self.stamp));
    }

    #[must_use]
    pub fn get(&mut self, id: &TileId) -> Option<&[u8]> {
        self.stamp += 1;
        let e = self.entries.get_mut(id)?;
        e.1 = self.stamp;
        Some(&e.0)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod source_tests {
    use super::*;

    fn tile(z: u8, x: u32, y: u32) -> TileId {
        TileId::new(z, x, y).expect("valid")
    }

    // spec: IO-12 (source + cache behavior)
    #[test]
    fn file_source_reads_and_types_missing() {
        let dir = std::env::temp_dir().join(format!("ve_io_tiles_it_{}", std::process::id()));
        let tdir = dir.join("2").join("1");
        std::fs::create_dir_all(&tdir).expect("mkdir");
        std::fs::write(tdir.join("1.mvt"), b"HELLETILE").expect("write");
        let src = FileSource::new(&dir);
        assert_eq!(src.load(2, 1, 1).unwrap(), b"HELLETILE");
        assert!(matches!(src.load(2, 0, 0), Err(SourceError::NotFound(_))));
        std::fs::remove_dir_all(&dir).ok();
    }

    // spec: IO-12
    #[test]
    fn http_source_is_typed_stub() {
        let err = HttpSource.load(0, 0, 0).unwrap_err();
        assert!(matches!(err, SourceError::Unsupported(_)));
    }

    // spec: IO-12
    #[test]
    fn lru_evicts_least_recently_used() {
        let mut c = LruCache::new(2);
        c.insert(tile(0, 0, 0), b"a".to_vec());
        c.insert(tile(1, 0, 0), b"b".to_vec());
        assert_eq!(c.get(&tile(0, 0, 0)), Some(&b"a"[..])); // touch a → b is LRU
        c.insert(tile(1, 1, 1), b"c".to_vec()); // evicts b
        assert!(c.get(&tile(1, 0, 0)).is_none(), "b evicted");
        assert!(c.get(&tile(0, 0, 0)).is_some(), "a survives (touched)");
        assert!(c.get(&tile(1, 1, 1)).is_some(), "c present");
        assert_eq!(c.len(), 2);
    }
}
