# wgpu + the Rust GPU stack — ecosystem health check

**Snapshot 2026-09-03** (shields/crates.io measured): wgpu 18k★, v30.0.1 (2026-08-22), 33.6M dl, Apache-2.0, main branch active. vello 4.3k★ (0.10.0, 2026-08-14). glow 0.18.0 (2026-07-09, **39.1M dl — even higher than wgpu**). three-d 0.19.0. fyrox 1.0.1 (2026-03-28). The `angle` crate on crates.io: 0.5.0, **updates ceased 2022-03 = dead** (no maintained ready-made ANGLE binding; integration must self-build bindgen).

## Profile
- **wgpu**: the Rust-native WebGPU API implementation + WGSL(naga), first-class Vulkan/Metal/DX12 backends, best-effort WebGL2, wasm -> browser WebGPU. The WebGPU of Firefox/Deno is wgpu (see the WebGPU file). Governance: the gfx-rs org, quarterly releases carrying breaking changes (v29->v30 gap ~2 months).
- **the linebender family**: vello (GPU-compute 2D vector, Vello Hybrid is the production form) + lyon (the de-facto standard of GPU path tessellation) + cosmic-text/parley/swash (text) + fontique/color. Led by Raph Levien.
- **glow**: the unified GL/GLES/WebGL loader — the standard Rust entry for the legacy rendering path (if taking GLES/ANGLE).

## For VisiaEngine
- **Adopt**: wgpu's API surface = the semantic vocabulary of the `visia-render` trait design (texture binding/limits/adapter selection logic map directly); the WebGL2 backstop and the downlevel capability-flag handling are a textbook for the SDK compatibility matrix; linebender's `color` (color management) /`fontique` (font discovery) are the modularization demonstration of "infrastructure as independent small crates".
- **Avoid**: ① treating vello as a ready-made 2D engine — 6 years without 1.0, it is a component, not a product; ② the quarterly breaking cadence would punch through an SDK stability promise -> **must pin the major version + keep the abstraction layer so wgpu types cannot escape visia-render** (exactly why the trait abstraction exists); ③ glow's 39M dl is the reminder: much of the Rust world's GPU load still runs on the GL path, the legacy-tier problem is not a pseudo-need.
- **Current-state ruling**: wgpu is uncontested as the main stack; Rust 2D vector rendering has no "map-grade" finished product (vello has no tile/style concepts) -> an in-house 2D style layer is unavoidable work, consistent with the gap list in `evidence/2026-09-03-wgpu-direct` §2.

## Sources
img.shields.io + crates.io (2026-09-03); `evidence/2026-09-03-wgpu-direct` §2/§4/§5.
