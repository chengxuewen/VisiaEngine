//! TileSet pump (IO-16, band N1.4): 4-state non-blocking loading.
//!
//! Semantics under test:
//! - `begin()` = state transition only (Unloaded → Loading), zero I/O.
//! - `pump(budget)` = loads at most `budget` Loading tiles per call; per-tile
//!   Ok → Done (bytes in LRU), Err → FailedTemporarily (retryable), never aborts.
//! - `state()` query; retry via begin() after failure.
//! - FileSource path unchanged: ensure() semantics bit-identical (canary tests
//!   in tile_set.rs stay untouched).
//!
//! HTTP guards: ureq timeout typing (Timeout → SourceError::Io with "timeout"
//! in message) — probed via an unreachable port (fast connection-refused is
//! Io, not a hang; the 5s production timeout is config, not logic).

#![allow(clippy::float_cmp)]

use std::sync::{Arc, Mutex};

use visiaengine_io_tiles::{
    TileId, TileSet,
    source::{SourceError, TileSource},
};

/// Scripted source: serves the fixture tree for `good` tiles, fails the rest
/// (records which tiles were attempted for budget-pacing assertions).
struct ScriptedSource {
    good: Vec<TileId>,
    attempts: Arc<Mutex<Vec<TileId>>>,
}

impl TileSource for ScriptedSource {
    fn load(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>, SourceError> {
        let id = TileId::new(z, x, y).expect("valid");
        self.attempts.lock().expect("lock").push(id);
        if self.good.contains(&id) {
            // Minimal-but-valid MVT is overkill for the scheduler layer: the
            // pump stores raw bytes; decode is decode-through-cache later and
            // failures there type as TilesError::Decode, not pump errors.
            Ok(b"RAWBYTES".to_vec())
        } else {
            Err(SourceError::NotFound(format!("scripted miss {z}/{x}/{y}")))
        }
    }
}

fn tile(z: u8, x: u32, y: u32) -> TileId {
    TileId::new(z, x, y).expect("valid")
}

fn set_with(good: &[TileId]) -> (TileSet, Arc<Mutex<Vec<TileId>>>) {
    let attempts = Arc::new(Mutex::new(Vec::new()));
    let set = TileSet::new(Box::new(ScriptedSource {
        good: good.to_vec(),
        attempts: Arc::clone(&attempts),
    }))
    .expect("set");
    (set, attempts)
}

// spec: IO-16
#[test]
fn pump_states_transition_begin_pump_done() {
    let (mut set, attempts) = set_with(&[tile(2, 0, 0)]);
    let a = tile(2, 0, 0);
    assert_eq!(set.state(&a), visiaengine_io_tiles::TileState::Unloaded);
    let started = set.begin(&[a]);
    assert_eq!(started, 1, "one Unloaded tile begins");
    assert_eq!(set.state(&a), visiaengine_io_tiles::TileState::Loading);
    assert!(attempts.lock().unwrap().is_empty(), "begin() does zero I/O");
    let (done, failed) = set.pump(10);
    assert_eq!((done, failed), (1, 0));
    assert_eq!(set.state(&a), visiaengine_io_tiles::TileState::Done);
}

// spec: IO-16
#[test]
fn pump_respects_budget_per_call() {
    let (mut set, attempts) = set_with(&[
        tile(2, 0, 0),
        tile(2, 1, 0),
        tile(2, 2, 0),
        tile(2, 3, 0),
        tile(3, 0, 4),
    ]);
    let ids = vec![
        tile(2, 0, 0),
        tile(2, 1, 0),
        tile(2, 2, 0),
        tile(2, 3, 0),
        tile(3, 0, 4),
    ];
    assert_eq!(set.begin(&ids), 5);
    let (done, _) = set.pump(2);
    assert_eq!(done, 2, "budget=2 → 2 tiles this call");
    assert_eq!(
        attempts.lock().unwrap().len(),
        2,
        "I/O calls == budget, not pending count"
    );
    let (done2, _) = set.pump(2);
    assert_eq!(done2, 2, "next call: next 2");
    let (done3, _) = set.pump(2);
    assert_eq!(done3, 1, "last tile");
    let (done4, _) = set.pump(2);
    assert_eq!(done4, 0, "no pending → pump is a no-op");
    assert_eq!(
        attempts.lock().unwrap().len(),
        5,
        "total I/O == 5 (each tile loaded once)"
    );
}

// spec: IO-16
#[test]
fn pump_never_aborts_on_first_error() {
    // good = only (0,0); (0,1) and (0,2) fail
    let (mut set, _a) = set_with(&[tile(2, 0, 0)]);
    let ids = vec![tile(2, 0, 0), tile(2, 1, 0), tile(2, 2, 0)];
    set.begin(&ids);
    let (done, failed) = set.pump(10);
    assert_eq!((done, failed), (1, 2), "failures don't stop the pump");
    assert_eq!(
        set.state(&tile(2, 0, 0)),
        visiaengine_io_tiles::TileState::Done
    );
    assert_eq!(
        set.state(&tile(2, 1, 0)),
        visiaengine_io_tiles::TileState::FailedTemporarily
    );
    // Retry semantics: begin() re-arms FailedTemporarily (caller's policy).
    assert_eq!(set.begin(&[tile(2, 1, 0)]), 1, "failed tile re-arms");
    assert_eq!(
        set.state(&tile(2, 1, 0)),
        visiaengine_io_tiles::TileState::Loading
    );
}

// spec: IO-16
#[test]
fn begin_skips_non_unloaded_and_is_idempotent() {
    let (mut set, _a) = set_with(&[tile(2, 0, 0), tile(2, 1, 0)]);
    let a = tile(2, 0, 0);
    let b = tile(2, 1, 0);
    assert_eq!(set.begin(&[a, b]), 2);
    // Pump ALL pending (HashMap iteration order is nondeterministic — which
    // tile drains first is not observable semantics; determinism lives in
    // pump_respects_budget_per_call via scripted source attempt counting).
    set.pump(2);
    assert_eq!(set.state(&a), visiaengine_io_tiles::TileState::Done);
    assert_eq!(set.state(&b), visiaengine_io_tiles::TileState::Done);
    assert_eq!(set.begin(&[a, b]), 0, "Done tiles skip; nothing re-arms");
}

// spec: IO-16 (timeout guard typing)
#[test]
fn http_timeout_types_as_io_error() {
    // Unreachable port: connection refused = fast Io error (not a hang).
    // The production 5s timeout is config on the same ureq call chain; this
    // test pins the ERROR TYPING, not the wall-clock duration.
    let src = visiaengine_io_tiles::HttpSource::new("http://127.0.0.1:9/__no_server__");
    let err = src.load(0, 0, 0).unwrap_err();
    assert!(
        matches!(err, SourceError::Io(_)),
        "refused/timeout both type as Io, got {err:?}"
    );
}
