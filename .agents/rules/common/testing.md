---
paths:
  - "**/*"
---

# Testing Requirements

## Minimum Test Coverage: 80%

Test Types (ALL required):
1. **Unit Tests** - Individual functions, utilities, components
2. **Integration Tests** - API endpoints, database operations
3. **E2E Tests** - Critical user flows (framework chosen per language)

## Test-Driven Development

MANDATORY workflow:
1. Write test first (RED)
2. Run test - it should FAIL
3. Write minimal implementation (GREEN)
4. Run test - it should PASS
5. Refactor (IMPROVE)
6. Verify coverage (80%+)

## Troubleshooting Test Failures

1. Use **tdd-guide** agent
2. Check test isolation
3. Verify mocks are correct
4. Fix implementation, not tests (unless tests are wrong)

## Agent Support

- **tdd-guide** - Use PROACTIVELY for new features, enforces write-tests-first

## Test Structure (AAA Pattern)

Prefer Arrange-Act-Assert structure for tests:

```typescript
test('calculates similarity correctly', () => {
  // Arrange
  const vector1 = [1, 0, 0]
  const vector2 = [0, 1, 0]

  // Act
  const similarity = calculateCosineSimilarity(vector1, vector2)

  // Assert
  expect(similarity).toBe(0)
})
```

### Test Naming

Use descriptive names that explain the behavior under test:

```typescript
test('returns empty array when no markets match query', () => {})
test('throws error when API key is missing', () => {})
test('falls back to substring search when Redis is unavailable', () => {})
```

## Runnable Check

```bash
# Verify: tests pass
cargo test --workspace
# Verify: coverage
cargo tarpaulin --workspace 2>/dev/null || echo "tarpaulin not installed"
```

## This repository's three-tier testing doctrine (E3D:D5 discipline, 2026-09-03 plan v1.1 batch 0a)

| Tier | Definition | Command | Gate location |
|----|------|------|---------|
| **T1 pure unit** | logic assertions with no GPU/window (parsing/math/scene) | `cargo test --workspace` | `pixi run ci` |
| **T2 frame-budget auto-smoke** | real execution of `--frames N` offscreen/window examples | `xvfb-run -a pixi run smoke-clear` (the whole smoke-* family) | `pixi run ci` (L2 segment) |
| **T3 human interaction (the honesty tier)** | interaction verification that cannot be automated (mouse picking, fly camera, etc.) — test bodies are `#[ignore]`d and **must print a human-check step checklist at run time**; when reporting, state plainly "T3 not run / human-verified" | `cargo test --workspace -- --ignored` (an empty set is currently allowed; a legitimately empty command run = pass) | manual check before merge |

Principle: every "tests pass" claim must name the tier; a T3 item must never be passed off as covered by T1/T2 green lights (the testing-surface projection of the verification-honesty clause).

## Visual-example double insurance (2026-09-16 lesson escalation: the case family where interactive windows were never pixel-verified)

Exit-code + counter gates on window/scene examples (`loaded N entities`/rc=0) are **not picture evidence** — three cases this session: E201's camera welded to the wrong position with an all-gray screen, E301's keyboard/mouse writing hard-coded variables, the golden family's center-luminance >40 gate fooled by the dark background's own score of 57. Discipline:

1. **Headless pixel gate**: every visual example carries at least one offscreen image assertion (color/count level, conservative threshold = measured −40%, catches regressions only, never chases aesthetics); precedent `crates/visiaengine-render-wgpu/tests/gltf_scene.rs` (load→fit→red/green counts). A new example lands its gate in the same commit it is born in.
2. **Three self-proofs for assertion predicates**: it can hit the positive case (the old camera position must go red once); it does not feed on background values (the threshold sits above the background with clearance); for luminance-band families use the "non-background passes" shape with care (the shape that was fooled in this case).
3. **T3 human-verification checklist**: the interaction surface (drag/scroll/close-window/resize) is machine-unreachable, so delivery text carries the human verification steps (the window-example projection of the existing `#[ignore]` print-at-runtime discipline); the verification environment = the user's channel form (PIT-22), not an export hand-added in the tool shell.
4. **ctest forwarding shell**: `-R` must come with a "≥1 selected" assertion + `--timeout` (cross-cited with PIT-23); an empty match exiting 0 = an always-false green.

```bash
# Verify: visual-example pixel gates are present (example→test mapping checked by hand + forwarding-shell assertions grepped)
grep -rn 'out of [1-9]' scripts/smoke-rs.sh        # forwarding-shell ≥1 assertion in place
grep -rn 'lum >\|red >\|green >' crates/*/tests/*.rs | wc -l   # pixel-gate count (reconcile on add/remove)
```

### HashMap iteration-order testing discipline (2026-09-28, N1.4 band, hard evidence)
For multi-object assertions driven by HashMap iteration (e.g. pump eviction order): unit tests **must not** make observable assertions on iteration order — green in a local single run, blown up in a full CI run (a close relative of the PIT-7 family of false reds). Deterministic semantics (budget = items per I/O pass) are locked with **scripted source counts** (attempts.len()==budget); the order itself is not locked.
**Verify**: run the same test five times in a row (`for i in 1 2 3 4 5; do cargo test ... ; done`) — all green before committing.
**Blocking condition**: fixing a HashMap-order flake without the 5-run loop.

## Web/wasm verification channel (PIT-44 registered, 2026-10-08)

`pixi run ci` (11 segments) does **not** cover the web surface. Three hard rules for wasm:

1. **Claim source**: any "WEB ✓ / MIRROR ✓" wording may come only from the actual output of `pixi run web-check` in the **same terminal**
   (plus `node scripts/web-mirror.mjs` when the roster is needed). A host cargo test does not count.
2. **Compile-surface unreachability**: wasm-bindgen-only constraints like `&mut Vec<T>` / `Vec<T>` out-params
   compile on the host target and fail on wasm32. Any change to signatures under `bindings/wasm/visiaengine-wasm/src/`
   requires one `pixi run web-check` run inside the slice (not cargo check).
3. **Staleness guardrail present**: `scripts/web-mirror.mjs` compares the mtime of the source directory against `pkg-node/*.d.ts`,
   turning red on a newer source and naming the remedy. That check going red ≠ a broken gate — it means "you are reporting today's green with yesterday's artifacts".

```bash
# Verify: guardrail in place + claim source traceable
grep -c "stale artifact" scripts/web-mirror.mjs          # expect ≥1 (literal must match web-mirror.mjs's own message)
grep -n "^ci = " pixi.toml                     # expect: no web-check (the reason this line exists)
```

## XTEST input probe = T3-input promoted to T2 (band K, 2026-10-08)

`pixi run keys-probe` (scripts/keys-probe.py + scripts/xinject.c) injects real
key/wheel/drag events into a live window on a **private** Xvfb display, then fails
when the window stays alive but the picture never changes. Measured output of the
committed set: E506 key 4 = 23.90%, E507 keys 2/3/1 = 5.87/29.85/29.85%,
E510 drag = 8.95% (+ title `phase=43.0`), E511 clock at a 4s gap = 1.79%, and
Escape closes all four through the same input path. Break probe recorded: stubbing
`self.haze = !self.haze` in E506 printed `changed=0.00%` and exited 1.

| Now machine-checked (T2) | Still human (T3) |
|---|---|
| a key/handler is wired to something the frame reads | does the result *look right* (aesthetics, colour taste) |
| the title/state echo updates | is the motion smooth (frame pacing, not pixel deltas) |
| drag/wheel change the view at all | does it read as the thing it claims to be |
| Escape closes the window | resize-while-dragging, multi-monitor, WM oddities |
| zero-arg run is resident (C15 breach detector built in) | anything needing a second app or a real GPU feel |

Harness-truth rule (band V, three false alarms in one session, each from MY run and
not from the code): before calling a gate red, confirm the gate was pointed at the
thing the example claims -- (1) display-family examples must run under Xvfb (their
`--frames N` is a window frame counter, not a headless lane: batch 4 read 3/3 FAILED
with no DISPLAY); (2) run the lane with the **registry argv** (`_args_<stem>` in
cmake/VisiaEngineBindings.cmake), not an ad-hoc number -- E305's marker threshold is
calibrated for 24 frames and panics at 4 on both the migrated and the pre-migration
build (batch 11); **fourth sighting (V4): E201's argv is
`resources/data/twoprim.glb --frames 3` -- a POSITIONAL ASSET FIRST. My ad-hoc
`--frames 1` dropped the model path, which silently switched that example into its
window lane and produced winit's "neither WAYLAND_DISPLAY nor DISPLAY is set" --
an error that reads like a rendering bug and is actually a lane-selection bug. Rule:
copy the registry argv VERBATIM, argument order included.**; (3) X keysym names are lowercase (`bracketright`), winit-style
`BracketRight` silently does nothing while the injector prints ok.

Rules baked into the tool (keep them if you port it): private display, never
`$DISPLAY` (PIT-22); assertions happen before cleanup; per-case settle/gap so a slow
clock is not read as "nothing changed"; SKIP exit 0 only for a missing toolchain
(Xvfb / xwd / cc / `XTest.h`) -- and `--strict` turns that skip into exit 1, which is
how ci.yml calls it: `xwd` ships in the SYSTEM package `x11-apps`, not in pixi, so a
runner that installs only `xvfb` would otherwise skip the whole probe and still print
green (PIT-25). A live window with no delta is exit 1.
It stays **out of `pixi run ci`** (same policy as `pack-check`) and is wired into
`.github/workflows/ci.yml` next to `scripts/window-probe-all.sh`, which had zero
callers until this band.

## Batch-migration gates (band V, 2026-10-08)

Mechanical edits across many examples are only as safe as the pixel gate attached to
them. The order that worked (and caught every real defect this band):

1. **Pre-check determinism before using it as evidence**: capture the same window twice
   back-to-back on the *unmodified* build. Equal -> a byte-identity gate is legal.
   Unequal (E305/E510/E511: the scene is animated) -> byte gates are BANNED for that
   example; use keys-probe percentages, time-progress ("the frame must change"), or the
   interaction tri-state (idle / drag / wheel must be three distinct hashes).
2. Capture the before-frame BEFORE editing. If you forgot, rebuild from git (checkout ->
   build -> capture -> restore), so the comparison is still pre-vs-post, not post-vs-post.
3. Card `cmp` only proves the headless lane; the migrated code is usually the *window*
   lane. A static sha match does not prove input still works (a shared bootstrap can pass
   frames while killing handlers) -- hence the interaction tri-state in every batch.
4. Run display-family examples under a display and with the **registry argv**
   (`_args_<stem>` in cmake/VisiaEngineBindings.cmake). Both mistakes happened this band
   and each produced a red that was about my harness, not the code.
5. For import-pruning steps inside a codemod, the real judge is `pixi run lint`
   (`cargo clippy --workspace --all-targets -- -D warnings`, pixi.toml L76) -- a symbol
   kept alive only by a comment mention becomes an unused-import **error** there. Prove
   the prune predicate bites before trusting it: plant a sample, the check must hit it
   (conventions.md C19 addendum).

## Engine-behavior gates ride the page, not the screenshot (2026-10-19, review-fix round)

Pages whose examples depend on engine state (pick surfaces, clip semantics, visibility) carry a
`?selftest`-style probe and report the verdict through the gate beacon; the gate asserts on the
payload (e.g. E401: RED+GREEN present in the 165-cell grid), not just boot+frames. A screenshot
or a frame counter cannot see a dead pick -- gate-web ran green for days over a page whose pick
was 0-for-320000 (PIT-78). Fit-changed framings are an API break for fixed-coordinate consumers:
sweep for hits, never assume the center (PIT-80).
