# P1 evidence: industry convergence on RTC granularity (input to D7)

**Date** 2026-09-03 | **Forensics** live web/source pulls in this session + the reference-project library profiles

| system | first-hand evidence | mechanism | owning layer |
|---|---|---|---|
| CesiumJS | `packages/engine/Source/Shaders/Builtin/Functions/translateRelativeToEye.glsl` (raw full text fetched) | global absolute coordinates split into high/low 2×f32 (EncodedCartesian3) -> shader translation relative to eye; cites AGI's "Precisions, Precisions" | viewer-origin rebase (the storage side carries the burden) |
| 3D Tiles | the 1.1 spec's tile.transform 4×4 column-major (a widely known clause) | content units/local + tile-level origin | layered origin |
| MapLibre GL | in-tile 0-8192 extent + per-tile `u_matrix` | tile-local storage + uniform-relative translation | layered origin |
| deck.gl | the official high-precision strategy: subtract a CPU common origin + offsets via uniform (profile + docs site) | layer-level origin | layered origin |
| UE5 | the LWC docs + the 5.0 release-notes direction: per-component origin rebasing **deprecated**, global double + draw-time rebase | the counter-example of an over-specialized origin layer (maintainability) | layered origin (post-convergence) |

**Four-scheme comparison conclusion** (scheme A = layered origin wins): B's high/low only props up the baggage of "must store globally"; C's shader f64 is banned by WebGPU, killing the Web route; D's per-frame vertex-VBO rewrite churns, killing large models. A takes the strengths of the first three: the f32 upload surface is naturally web-compatible, the origin level = entity level (glTF model/tile) is semantically natural, and rebase decisions collapse into 1 uniform path.
**Precision budget**: local≤64km -> f32 step ≤7.6mm; |origin−eye|≤100km -> same magnitude. Map-grade sub-millimeter far-future needs = a tile-subdivision-granularity question (decided by io-geo), not an engine-mechanism question.
