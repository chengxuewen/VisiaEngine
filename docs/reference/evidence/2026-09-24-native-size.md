# Native Startup Size Measurement — 2026-09-24

> Closes the long-standing "MVP 实测复核" item (architecture.md ⑨ build/delivery matrix: "总量标 ≤10MB（MVP 实测复核）" — the re-measurement had never been recorded).
> Method per super-band plan adjudication ⑧: release build → strip → sizes; `cargo tree` for composition. Machine: Linux dev box, conda-forge rust toolchain (pixi), release profile defaults (no LTO/panic-abort tuning — plain `pixi run cargo build --release -p visiaengine-capi`).

## Artifact sizes

| Artifact | Unstripped | Stripped |
|---|---:|---:|
| `libvisiaengine.so` (cdylib, dynamic host embedding) | 10,132,128 B (9.66 MB) | **7,605,216 B (7.25 MB)** |
| `libvisiaengine.a` (staticlib, static host linking) | 70,178,468 B (66.9 MB) | 37,315,738 B (35.6 MB) |

Reading: the **cdylib is the host-facing deliverable** (C API embedding per D6). At **7.25 MB stripped it is inside the ≤10 MB budget** with ~27% headroom. The staticlib number is not the startup-relevant figure (it carries debug/relocation overhead for static linking; hosts embedding statically link away what they already have), but is recorded for completeness.

## Dependency composition (`cargo tree -p visiaengine-capi -e normal`)

- Total unique normal-dependency packages: **142**
- Workspace crates in the link face: core, geo, io-gltf, io-points, io-text, render, render-wgpu (+ capi itself); direct external root: **wgpu** (everything GPU-related hangs off it), raw-window-handle, geojson, fontdue (io-text), gltf (io-gltf).

Top recurring packages by edge count (indicates fan-out, not bytes): bitflags (18), thiserror (14), log (11), rustix (10), proc-macro2 (10 — build-time only), num-traits (9), libc (9), wayland-backend (8, wgpu's platform backends), hashbrown (8).

Largest byte contributors were not individually profiled (adjudication ⑧: one-shot re-measurement, no cargo-bloat). The wgpu family is the known dominant block (GPU backend drivers: vulkan/gl/dx metal-less on this target) — consistent with the wasm evidence (`docs/reference/evidence/2026-09-03-bevy-embed.md`: wasm 13 MB raw / 3.4 MB gz untrimmed, the only prior measurement).

## Verdict vs the three doc claims

| Doc claim | Status after this measurement |
|---|---|
| whitepaper.md:63 "启动体积可控制在 10 MB 以内" | **TRUE for the cdylib deliverable** (7.25 MB stripped) — claim retained, evidence cited |
| README.md:14 "启动体积目标 ≤10 MB" | retained (target met by the same number) |
| architecture.md:139 "总量标 ≤10MB（MVP 实测复核）" | re-measurement **recorded here**; "复核" pending-note resolved |

Caveat: "总量" (total SDK surface) beyond the single cdylib (headers, bindings, wasm artifact) is not a startup cost; the ≤10 MB budget is interpreted as the native deliverable size, which this measurement closes. Re-run this measurement after major dependency changes (wgpu upgrades) — natural anchor: the quarterly wgpu upgrade window.
