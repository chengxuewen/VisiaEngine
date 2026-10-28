---
name: observability-and-instrumentation
description: "Observability and instrumentation for this engine: structured logging, RED metrics, tracing, symptom-based alerting, and the existing web beacon/gate-web telemetry. Use when adding instrumentation, debugging runtime behavior, designing alerts, or extending the web-wall heartbeat/beacon."
---

# Observability & Instrumentation (this workspace)

## Overview

- Current state: the **web wall** already has beacon/heartbeat telemetry — `examples/wasm/common.js` samples the data-frame to prove the rAF loop turns, each page emits a post-await beacon gated by `VE_GATE_FRAMES` (8 frames), and `pixi run web-check` (the `gate-web` chain: `scripts/web-build.sh` + `cargo test -p visiaengine-wasm -- --ignored`) consumes it with **per-page fault isolation**.
- The **Rust side has no `tracing`/OpenTelemetry yet**; logging is ad-hoc. Adding a tracing stack is a dependency change and must pass `pixi run audit`.
- This skill = how to instrument without regressing the existing beacon/gate, and how to grow from ad-hoc logs to structured telemetry + symptom-based alerts.
- Single source of truth for the gate: `pixi.toml` `web-check` and `scripts/web-build.sh`; do not invent a parallel gate.

## When to Use

- Adding logging/metrics/tracing to Rust crates or the wasm binding.
- Extending the web-wall heartbeat/beacon.
- Debugging runtime behavior (frame stalls, load failures, per-page regressions).
- Designing alerts or SLO-ish checks for a long-running/engine path.

## Process (steps + checkpoints + exit)

1. **Read the current instrumentation** (do not assume):
   - `examples/wasm/common.js` (heartbeat), the page `VE_GATE_FRAMES` beacons, `pixi run web-check` / `scripts/web-build.sh`.
   - `Cargo.toml` (is `tracing`/`log` already a dependency? today: no).
   - Checkpoint: you can state where telemetry exists today and what the gate asserts.
2. **Prefer structured logs over `println!`** in library code: emit key=value fields (frame id, page id, phase) so a broken page's beacon is attributable. Use `tracing` with spans once the dependency is added (feature-gate it so `--no-default-features` still builds).
3. **Define RED metrics** (Rate, Errors, Duration) for the long-running/engine path; measure at boundaries, not in hot inner loops.
4. **Keep per-page fault isolation**: one page's beacon cut must redden only that page's leg (the existing behavior). Never collapse all pages into a single shared beacon.
5. **Alerts on symptoms, not causes**: alert on "frames stopped turning" / "load never settled", not "CPU high". A cause alert fires when the system is healthy-but-busy.
6. **Wire into the existing gate**: extend `web-check`/`gate-web`; adding a tracing dependency requires `pixi run audit` green.
7. **Exit condition**: the gate output + a concrete beacon/metric/trace sample quoted as evidence. "Looks instrumented" is not evidence.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "`println!` is enough" | Unstructured lines cannot be filtered per page/frame; use key=value or spans. |
| "add tracing later" | Later is after an incident; instrument the path you are touching now. |
| "alert when CPU is high" | That is a cause, not a symptom; alert on the user-visible failure. |
| "one beacon for all pages is simpler" | It destroys per-page isolation; a single failure reddens every leg. |
| "skip web-check, it's slow" | `web-check` is the telemetry gate; skipping it hides a stalled loop. |

## Red Flags

- New `println!`/`eprintln!` in library code where structured logging is expected.
- A beacon with no page/frame identifier (unattributable).
- A single shared beacon replacing per-page beacons.
- Cause-based alerts ("memory > X") presented as the health signal.
- A tracing/metrics dependency added without `pixi run audit`.
- Bypassing `web-check`/`gate-web`.

## Verification (evidence)

- `pixi run web-check` green output; the beacon/heartbeat log line for the affected page.
- For Rust tracing: a captured span/metric sample and the `pixi run audit` result when a dependency was added.
- Per-page isolation: show that cutting one page's beacon reddens only that leg.

## References (read on demand)

- `examples/wasm/common.js` — heartbeat implementation.
- `scripts/web-build.sh` + `pixi.toml` `web-check` — the gate-web chain.
- `.agents/rules/rust/patterns.md` — error/observability conventions.
- `docs/env.md` — toolchain + task reference.
