---
name: code-simplification
description: "Reduce complexity in Rust/TS code. Chesterton's Fence analysis, Rule of 500 enforcement, dead code elimination, borrow checker simplification patterns. Complements ponytail for Rust-specific over-engineering. Use after ponytail-audit, before PRs, or when diagnosing complexity smells."
---

# code-simplification — Complexity Reduction

> Chesterton's Fence + Rule of 500 + Rust borrow checker patterns.
> Delete only what deserves deletion. Never delete what you don't understand.

## Trigger Conditions

- The user says "太复杂" / "简化" (too complex / simplify), "refactor", "dead code"
- After ponytail-audit finds removable items
- A file exceeds 500 lines (`wc -l` > 500)
- `cargo clippy -- -W clippy::cognitive_complexity` reports high complexity
- A function exceeds 50 lines (`wc -L` > 50 per function)

## Chesterton's Fence Flow

```
spot complexity → find the reason → legitimate reason? → YES: keep, add a comment / NO: delete
```

### Reason-finding tools

| Source | Command |
|------|------|
| git blame | `git blame -L <line>,<line> <file>` |
| commit messages | `git log --all -S "<code>" --oneline` |
| issues/PRs | `gh search issues "<keyword>" --repo $(git remote get-url origin)` |
| ADR decisions | `.agents/memorys/decisions.md` |
| community reference | `grep_app_searchGitHub` |

### Fence comments

When keeping justified complexity, mark it with a comment:

```rust
// chesterton: BufferPool uses unsafe zero-copy because the &[u8] emitted by the
// GStreamer appsink must be forwarded directly to the WebRTC TrackLocal to avoid memcpy latency.
// Tried the safe Vec<u8> copy approach → 1080p60 frame-drop rate went from 0% → 1.2%.
// git: a1b2c3d "zero-copy buffer pool for 60fps stability"
unsafe {
    pool.copy_to_track(track, data);
}
```

## Rule of 500

| Check | Command | Threshold | Action |
|--------|------|------|------|
| file lines | `wc -l <file>` | >500 | split into multiple domain modules |
| function lines | `grep -n '^fn ' <file>`, estimate blocks | >50 | extract helper functions |
| parameter count | `grep -c ','` per fn signature | >5 | merge params into a struct |
| nesting depth | eyeball indentation | >4 | early return / propagate with `?` |
| trait impl count | `grep -c 'impl.*for' <file>` | >3 | separate into impl submodules |
| match arm count | `grep -c '=>'` per match block | >10 | `enum_dispatch` / strategy pattern |
| pub exposure count | `grep -c 'pub' <file>` | >20 | narrow visibility, finer-grained modules |

### Execution

```
# 1. scan for over-threshold files
find crates/ -name '*.rs' -exec wc -l {} + | sort -rn | head -20

# 2. apply Chesterton's Fence to each over-threshold file
# 3. split, extract, delete
# 4. verify: cargo clippy -- -D warnings && cargo test -p <crate>
```

## Rust-specific patterns

### Borrow-checker simplification

```rust
// BEFORE: unnecessary Arc<Mutex<>> nesting
let data: Arc<Mutex<Vec<Arc<Mutex<Option<Box<dyn Trait>>>>>>> = ...;

// AFTER: single ownership, borrow on demand
let mut data: Vec<Box<dyn Trait>> = Vec::new();
// when sharing is needed: let data = Arc::new(RefCell::new(data));
// ponytail: RefCell suffices for single-thread; switch to RwLock for real multithreading
```

### WebRTC backend abstraction layer

```rust
// BEFORE: duplicated glue code per backend
#[cfg(feature = "backend-webrtc-rs")]
fn create_pc() -> RTCPeerConnection { ... }
#[cfg(feature = "backend-webrtc-sys")]
fn create_pc() -> RTCPeerConnection { ... }
#[cfg(feature = "backend-stub")]
fn create_pc() -> RTCPeerConnection { ... }

// AFTER: extract the common part to a shared module; backends implement only the differences
// chesterton: the three-backend cfg duplication is the architectural cost of the D15 decision;
// it cannot be eliminated, only minimized by compressing it into backend/ submodules.
```

### Config path simplification

```rust
// BEFORE: deeply nested config
let port = config.server.listen.port.unwrap_or(8080);

// AFTER: flatten + sane default
let port = env::var("APP_LISTEN_PORT").ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(8080);
```

## Relation to ponytail

| ponytail | code-simplification |
|----------|---------------------|
| global audit → ranking | single file/module deep-dive |
| "can this be deleted?" | "why does this exist?" |
| deletion decisions | split + refactor decisions |
| one line per finding | structural refactoring |

**Workflow**: `/ponytail-audit` → rank → `/code-simplification` item by item

## Verification gates

```bash
# 1. compiles
cargo check --workspace --all-features

# 2. Clippy zero warnings
cargo clippy --workspace --all-features -- -D warnings

# 3. all tests pass
cargo test --workspace

# 4. file line count decreased
git diff --stat | grep -E '\+[0-9]+.*-' | tail -5

# 5. no new pub API (simplification must not enlarge the public surface)
# ponytail: only checks pub fn count, advisory only — refactors may naturally add some
```

## Anti-pattern detection

| Anti-pattern | Detection command | Automated fix |
|--------|---------|-----------|
| uncommented `unwrap()` | `grep -r 'unwrap()' crates/ --include='*.rs' \| grep -v '//.*unwrap'` | replace with `?` or `.context()` |
| `clone()` to satisfy the borrow checker | `grep -r '\.clone()' crates/ --include='*.rs'` | analyze the borrow chain |
| `Box<dyn Trait>` where generics work | clippy `boxed_local` lint | change to `impl Trait` |
| feature-gated dead code | `cargo deadlinks` / `cargo udeps` | remove the dead feature |
| duplicate impl blocks | eyeball check | merge or extract a macro |
| `as` casts | `grep -r ' as ' crates/ --include='*.rs'` | prefer `.into()` / `.try_into()` |

## Output format

```
## Simplification report

file: crates/<name>/src/backend/mod.rs
before: 487 lines → after: 312 lines (-36%)

### Removed
- [lines 45-78] unused trait → deleted (git blame: introduced early, since abandoned)
- [lines 203-206] dead code `#[cfg(all(nonexistent, feature = "..."))]`

### Split
- [mod.rs] → `backend/impl_a.rs`, `backend/impl_b.rs`, `backend/shared.rs`

### Kept (Chesterton's Fence)
- [lines 180-195] `#[cfg]` multi-backend duplication — architectural cost, cannot be eliminated
- [line 300] `unsafe` buffer pool — performance constraint, see comment

### Verification
✅ cargo clippy -- -D warnings
✅ cargo test -p <crate>
```

## Forbidden

- deleting code you don't understand
- merging unrelated modules
- simplifying an `unsafe` block without understanding its memory semantics
- removing "looks unused" code without git blame evidence
- deleting feature-gated code without first confirming the CI variants
