# Evidence Memo — "Hand-rolled renderer directly on wgpu" path for VisiaEngine

**Purpose**: evidence package for final backend ruling (C1 decision table), wgpu-direct lane only. Bevy lane covered by sibling investigation.
**Date**: 2026-09-03. All URLs accessed 2026-09-03 unless noted. Live API data: crates.io, api.github.com.
**Verdict headline**: Technically viable — every needed building block exists and the core ones (wgpu, lyon, cosmic-text, gltf, proj, rstar) are actively maintained. But the *renderer infrastructure* layer (frame graph, pipeline caching, 3D Tiles streaming, GPU culling) has **no mature off-the-shelf Rust option** — that is the cost center, and precedent histories say it takes years.

---

## 1. Precedents of exactly this shape

### 1a. Rerun (closest structural match: wgpu-based, embeddable-as-SDK Rust viewer)

| Fact | Evidence |
|---|---|
| Architecture split: logging SDKs (Python/C++/Rust/JS) → Arrow storage → separate Viewer. `re_renderer` is a standalone wgpu renderer crate: *"A custom wgpu based renderer… it can be used standalone… No dependencies on re_viewer or Rerun chunk store libraries"*, goals include *"Automatic resource re-use & caching"*, *"WebGL compatible quality tier… without WebGPU support"* | https://github.com/rerun-io/rerun/blob/main/ARCHITECTURE.md ; https://github.com/rerun-io/rerun/blob/main/crates/viewer/re_renderer/README.md |
| SDK↔viewer decoupling = **no loop hijack by design**: C++ SDK either `spawn`s a viewer process, `connect_grpc` to a running one, or `save`s to file; data buffers in memory until then. The viewer itself is an egui+wgpu app owning its own loop | https://github.com/rerun-io/rerun/blob/main/rerun_cpp/README.md ("Connecting / Buffering" sections) |
| Web: *"we use WebGPU when available on the Web, but automatically fall back to a WebGL based emulation layer (with a more limited feature set)"* via wgpu | ARCHITECTURE.md, "wgpu" section |
| Timeline: repo created **2022-04-08**; first public release **v0.2.0 on 2023-02-14** (~10 months, but built on egui which emilk had developed since 2019); oldest blog post **2022-06-15**; **86 releases, still 0.37.0 (2026-09-01) — never reached 1.0** | api.github.com/repos/rerun-io/rerun (created_at); /releases?per_page=100 (no 1.x tag exists); https://rerun.io/blog |
| Scale: monthly-ish releases (0.36.0 2026-08-10 → 0.37.0 2026-09-01), funded company, multiple language SDKs (rerun_py, rerun_cpp, rerun_js, rerun_notebook in repo root) | GitHub releases API; repo contents API |

**Reading for VisiaEngine**: rerun proves wgpu + custom high-level renderer + multi-language SDK + web delivery is a shipping product architecture, and its spawn/connect model is the cleanest answer to "must not hijack host main loop". Caveat: re_renderer targets *visualization primitives* (lines/points/meshes, dynamic data), not geo-tiling/terrain/3D-Tiles streaming, and even a funded team is pre-1.0 after 3.5+ years.

### 1b. Other wgpu-based embeddable renderers/engines

| Project | Shape | Status (2026-09-03) |
|---|---|---|
| **Vello** (linebender) | GPU compute-centric **2D vector renderer** on wgpu; a *component*, not a viewer. Three variants: Vello (experimental GPU), Vello CPU, **Vello Hybrid** = *"the main Vello implementation for production use-cases"* | repo created 2020-04-15; first release **v0.1.0 only on 2024-03-05** (~4 yrs); now v0.10.0 (2026-08-14), still 0.x; 4,309 stars; Apache-2.0/MIT; pushed same day. Badge: wgpu v29.0.1. https://github.com/linebender/vello |
| **Fyrox** | Full 2D/3D game engine + editor (wgpu rendering), *"feature-rich, production-ready"*; embeddable as library but engine-shaped | repo 2019-03-30 (formerly rg3d); 9,538 stars; MIT; pushed 2026-09-01. https://github.com/FyroxEngine/Fyrox |
| **egui** | Immediate-mode GUI, *"completely platform agnostic"*; official `egui-wgpu` renderer + `egui-winit` integration; the de-facto pattern for rendering into someone-else's wgpu surface without owning the loop | 30,380 stars; Apache-2.0; pushed 2026-09-02; repo since 2019-01-13. https://github.com/emilk/egui |
| **pixels** (parasyte) | Minimal wgpu frame-draw canvas for apps that keep their own loop — direct precedent for "render without hijacking" | 2,133 stars; MIT; pushed 2026-08-23. https://github.com/parasyte/pixels |
| **three-d** (asny) | Simple 2D/3D renderer on wgpu incl. web | v0.19.0 (2026-04-17); 1,664 stars; MIT. https://github.com/asny/three-d |
| **nightshade** | *"GPU-driven wgpu renderer with a built-in frame graph"* — the only Rust frame-graph renderer found | v0.57.0 (2026-08-04) but **49 stars** — hobbyist scale. https://github.com/matthewjberger/nightshade |

**Reading**: every successful wgpu-direct project either (a) stayed a component (vello, lyon, egui), (b) narrowed scope hard (rerun = viz primitives; pixels = pixel buffer), or (c) spent many years as a full engine (Fyrox). None shipped GIS/tiling/3D-Tiles on wgpu.

### 1c. What the geo incumbents chose

| Engine | Architecture choice | Evidence |
|---|---|---|
| **CesiumJS** | **Custom engine core** on WebGL (not an off-the-shelf renderer): *"uses WebGL for hardware-accelerated graphics… tuned for dynamic-data visualization"*; modular `@cesium/engine` (core+rendering+data APIs) split from widgets | https://github.com/CesiumGS/cesium (README); repo created 2012-03-02 (14 yrs of continuous engine investment) |
| **MapLibre Native** | **Custom engine core** (fork of Mapbox GL Native, itself ~2014): *"GPU-accelerated vector tile rendering"*; forked Dec 2020 at mgbl 1.6.0 after Mapbox relicensed | https://github.com/maplibre/maplibre-native (README + FORK.md); repo 2020-11-20 |
| MapLibre GL→Metal history | iOS build exposes selectable renderer: bazel flag `--//:renderer=metal` (README "iOS" section) — Metal is a first-class iOS renderer option. Precise migration dates: **UNCERTAIN** (maplibre.org/news index is JS-rendered; not verified this session) | README (accessed 2026-09-03) |
| **Embedded/FFI story** | MapLibre Native ships per-platform bindings; **maplibre-native-qt** = "MapLibre Native Qt Bindings and Qt Location Plugin", 136 stars, pushed 2026-08-31 — direct precedent for embedding a native map engine into a Qt host | https://github.com/maplibre/maplibre-native-qt |

**Reading**: both geo incumbents run **custom engine cores**, validating the *shape* of the wgpu-direct option; but note both carry decade-scale engineering mass (Cesium since 2012; Mapbox GL lineage since ~2014) — they are existence proofs of feasibility, not of cheapness.

---

## 2. Building blocks inventory (crates.io + GitHub API, accessed 2026-09-03)

### Core rendering

| Block | Crate / repo | Version (latest) | Last release | Stars | License | Maintenance read |
|---|---|---|---|---|---|---|
| Graphics API | **wgpu** / gfx-rs/wgpu | 30.0.1 | 2026-08-22 (v30.0.0 2026-07-01, v29.0.3 2026-05-02 → ~quarterly breaking releases) | 17,918 | Apache-2.0 | Very active. Backends: Vulkan/Metal/DX12 first-class; OpenGL/WebGL2 best-effort; WebGPU on wasm ✅ (README "Supported Platforms"). Backing is real: Firefox and Deno embed wgpu for their WebGPU impl (Wikipedia "WebGPU") |
| Shader translation/WGSL | **naga** (in wgpu repo) | 30.0.1 | 2026-08-22 | (same repo) | Apache-2.0 | WGSL is the W3C WebGPU shading language; naga 33.6M downloads — mature |
| Frame graph | — | **No live option.** `fade` on crates.io is an unrelated Fly.io VM tool (15 stars, dead 2022); `render-graphs` 404; nightshade has one built-in (49 stars) | — | — | — | **Gap: build your own** |
| Pipeline/bind-group caching | wgpu_render_manager | 30.0.2 | 2026-07-29 | 0 | MIT | New, unproven, tracks wgpu majors — signal that people hand-roll this |

### 2D geometry / tessellation

| Block | Crate | Version | Last release | Stars | License | Read |
|---|---|---|---|---|---|---|
| Path tessellation | **lyon** | 1.0.19 | 2026-03-08 | 2,596 | Apache/MIT (repo "NOASSERTION") | Mature, maintained; the standard for GPU 2D path fill/stroke |
| Ear-clipping | **earcutr** | 0.5.0 | 2025-05-29 | 48 | ISC | **archived repo** (frewsxcv/earcutr archived=True) but 11.6M downloads; fork risk if bugs surface |
| GPU 2D (optional) | **vello** | 0.10.0 | 2026-08-14 | 4,309 | Apache-2.0/MIT | Pre-1.0; Vello Hybrid targeted at production |

### Text (CJK-capable)

| Block | Crate | Version | Last release | Stars | License | Read |
|---|---|---|---|---|---|---|
| Text layout+shaping | **cosmic-text** (System76) | 0.19.0 | 2026-04-22 | 2,139 | Apache-2.0 | Pure Rust, multi-line, complex scripts; 7.9M dl — strongest CJK option |
| Alternative layout | **parley** (Linebender) | 0.11.1 | 2026-08-16 | 721 | Apache-2.0 | Newer, active, pairs with **swash** 0.2.10 (shaping/raster, 871 stars, 2026-07-17) |

### Geospatial

| Block | Crate | Version | Last release | Stars | License | Read |
|---|---|---|---|---|---|---|
| CRS reprojection | **proj** (georust) | 0.31.0 | 2025-08-29 | 185 | Apache-2.0 | Bindings to upstream PROJ (C dep); active repo push 2026-06-17 |
| Pure-Rust alt | proj4rs (3liz) | 0.1.10 | 2026-03-06 | 78 | none declared | Weaker license story |
| Geo I/O (zero-copy) | **geozero** | 0.15.1 | 2025-12-11 | 470 | Apache-2.0 | GeoJSON/MVT/FlatGeobuf/WKT in one streaming API — ideal ingest layer |
| GeoJSON | geojson (georust) | — | repo push 2026-04-29 | 345 | Apache-2.0 | Solid |
| FlatGeobuf | flatgeobuf | 6.0.1 | 2025-12-28 | 819 | BSD-2 | Solid |
| Vector tiles (MVT) | **open-vector-tile** | 1.11.1 | 2026-02-10 | **10** | NOASSERTION | Thin adoption; `mvt` (DougLau) 0.15.0 2026-08-01, 25 stars; `mapbox_vector_tile` dead (2019). **Decode exists, ecosystem is thin** |
| 3D Tiles | **ogc_3d_tiles** | 0.1.0 | 2026-06-23 | — (**26 downloads**) | — | Spec parsing only; **bevy_3d_tiles** 0.4.4 (2026-08-13, 2 stars) is the only streaming renderer and is **Bevy-coupled**. **Effective gap on wgpu-direct lane** |
| Point clouds | **las** (las-rs) | 0.11.1 | 2026-08-25 | 105 | MIT | Maintained. `pcd` crate dead (0.0.0, 2018) |
| Draco (glTF ext) | draco-core / draco-gltf | 1.2.0 / 0.2.0 | 2026-08-01 / 2026-07-29 | 5 | Apache-2.0 | Pure-Rust, brand-new, low adoption — verify before depending |
| Geometry ops | geo | — | push 2026-09-01 | 1,921 | Apache/MIT | Solid |
| Spatial index | **rstar** | — | push 2026-08-23 | 555 | Apache-2.0 | Solid R-tree |
| GEOS bindings | geos | — | push 2026-03-03 | 147 | MIT | C dep; fine for ops, not rendering |

### 3D assets / embedding

| Block | Crate | Version | Last release | Stars | License | Read |
|---|---|---|---|---|---|---|
| glTF 2.0 | **gltf** | 1.4.1 | **2024-05-10** (repo push 2026-05-11) | 641 | Apache-2.0 | Stable format; crate releases infrequent but repo alive |
| Host window interop | **raw-window-handle** | 0.6.2 | 2024-05-17 | 431 | Apache-2.0 | Stable ABI surface — the key to "render into host window without hijack" (96.5M dl) |
| Windowing (if needed) | winit | 0.31.0-beta.2 (0.30 stable line) | 2026-03-02 | 6,136 | Apache-2.0 | Active |
| CPU fallback | softbuffer | — | push 2026-08-05 | 502 | Apache-2.0 | Linebender; escape hatch |

### GPU-driven culling for many tiles
- wgpu exposes the primitives: `draw_indexed_indirect`, `multi_draw_indexed_indirect`, `draw_indexed_indirect_count`, gated by `DownlevelFlags::INDIRECT_EXECUTION` / `Features::INDIRECT_FIRST_INSTANCE` (wgpu source, wgpu/src/api/render_pass.rs + wgpu-types/src/instance.rs, trunk @ 2026-09-03).
- **No turnkey Rust crate for tile frustum/Hi-Z culling found** (crates.io searches + code search). Bevy uses indirect drawing internally (bevy_render draw_state.rs) but that's the Bevy lane. On wgpu-direct you implement culling yourself — pattern is well-known (compute-pass culling → indirect buffers), but it is bespoke work.

---

## 3. Effort reality — grounded in precedent histories

### Precedent timelines (all dates from GitHub API, accessed 2026-09-03)

| Project | Start (repo created) | First public artifact | To "mature" |
|---|---|---|---|
| rerun | 2022-04-08 | v0.2.0 **2023-02-14** (~10 mo; but riding egui since 2019 by same author + Arrow stack) | **still 0.37, pre-1.0 after 3.5 yrs**, 86 releases |
| vello | 2020-04-15 (research lineage) | v0.1.0 **2024-03-05** (~4 yrs) | v0.10 (2026-08) — 2D only, still 0.x |
| MapLibre Native | fork 2020-11-20 | inherited | lineage = Mapbox GL Native **since ~2014** |
| CesiumJS | 2012-03-02 | — | 14 yrs, commercially backed |
| Fyrox | 2019-03-30 | — | ~7 yrs, still iterating |
| wgpu itself | 2018-09-13 | — | 8 yrs to current maturity |

### MVP estimate (VisiaEngine MVP = GeoJSON 2D + glTF 3D + ortho↔perspective transition + C API embedded in one desktop host)

**Estimate, not citation** — synthesized from the above component inventory (what exists vs. what must be hand-built) and precedent velocities:

| Work package | Exists? | Person-months (senior, 1 FTE) |
|---|---|---|
| wgpu device/surface/resize/frame orchestration on host window (raw-window-handle) | wgpu yes; integration no | 1.5–3 |
| Render pass orchestration + pipeline caching | **no mature crate** | 2–4 |
| GeoJSON ingest (geozero) → styling → lyon tessellation → batched 2D pipeline | blocks yes, glue no | 2–3 |
| glTF PBR pipeline (IBL, shadows optional) on wgpu | gltf crate yes; PBR shaders no | 3–5 |
| Unified camera: ortho↔perspective blend + geo-referencing (proj) | proj yes; camera math no | 1–2 |
| CJK text overlay (cosmic-text glyph atlas → wgpu) | cosmic-text yes; atlas/renderer glue no | 2–3 |
| C API surface + host embedding + packaging (≤10MB budget, cdylib/staticlib) | no | 2–4 |
| Picking, MSAA/DPI, color correctness | no | 2–3 |
| **Sum (solo sequential)** | | **≈ 15–27 PM** |
| **Team of 2–3 parallelized** | | **≈ 8–14 calendar months** to honest MVP |

Sanity anchors: rerun took ~10 months to *first public release* (with egui/Arrow tailwind and a funded team) and is pre-1.0 at 3.5 yrs; vello took ~4 years to v0.1 for a *2D-only* renderer. An MVP at 8–14 months for a 2–3 person team is consistent with these velocities **only because** lyon/cosmic-text/gltf/geozero/proj/rstar carry the decode/geometry/text load; the moment scope touches 3D Tiles streaming or GPU tile culling, add 3–6 PM (see gaps below).

---

## 4. WebGPU / web path (2026 state)

| Claim | Evidence |
|---|---|
| Browser support only became universal **mid-2025**: Chrome/Edge April 2023; **Safari 26 June 2025; Firefox 141 July 2025**; spec = W3C Candidate Recommendation Draft | Wikipedia "WebGPU" (accessed 2026-09-03) |
| wgpu compiles to wasm against browser WebGPU (WebGPU ✅ Web column); WebGL2 available as downlevel fallback | wgpu README "Supported Platforms" |
| Production proof of wgpu-on-web with graceful fallback: rerun ships web viewer — *"same code for native as for web… fall back to a WebGL based emulation layer (with a more limited feature set)"*; Firefox/Deno's own WebGPU impls are wgpu | rerun ARCHITECTURE.md; Wikipedia "WebGPU" (Implementations) |
| Firefox shipped WebGPU **on Windows first** (platform rollout order) | **UNCERTAIN** — not verified this session |

**Reading**: SDK-into-browser via wgpu+wasm is real and precedented (rerun), but the WebGPU-only-browser window is ~1 year old; a WebGL2 fallback tier costs a second quality path (rerun explicitly maintains one).

---

## 5. Hard problems the wgpu-direct path must own (things an engine would absorb)

1. **Render-graph / pass scheduling** — no live Rust frame-graph crate (fade=dead unrelated project; nightshade=49★). You own design, maintenance, and every future pass added.
2. **Pipeline & shader-compile caching** — wgpu/naga give you compilation, not caching; first-frame jank and startup shader compile cost are yours (re_renderer lists *"Automatic resource re-use & caching"* as a *goal they had to build*; wgpu_render_manager exists precisely because people hand-roll it).
3. **Resource lifecycle** — buffer/texture residency, streaming eviction, GPU memory budgets: no allocator/manager layer above wgpu. re_renderer states *"assumes that most data may change every frame"* — that policy took them years.
4. **Multi-window surfaces** — wgpu gives one surface per window; window↔surface↔swapchain-lost handling (resize/minimize/device-lost) is manual everywhere.
5. **HDR / color management** — no engine-level tonemapping or display-P3 pipeline; Linebender maintains a separate `color` crate for this (part of vello ecosystem) — i.e. even they treat it as standalone work. **UNCERTAIN detail: crate status not re-verified this session.**
6. **MSAA / DPI scaling** — wgpu exposes multisampling and surface sizing; policy (when to MSAA, how to scale text/vectors at 200% DPI) is application code.
7. **wgpu upgrade treadmill** — major releases ~quarterly with breaking changes (v29.0.3 2026-05 → v30.0.0 2026-07 → v30.0.1 2026-08); an SDK that ships binaries must pin and re-verify on its own schedule.

---

## 6. Remaining risks (decision-table inputs)

| # | Risk | Severity | Mitigation / evidence |
|---|---|---|---|
| R1 | 3D Tiles / point-cloud streaming ecosystem gap: ogc_3d_tiles 26 downloads; only streaming renderer (bevy_3d_tiles) is Bevy-coupled; draco-core 5★ | High | Own the streaming layer, or re-scope MVP away from 3D Tiles (GeoJSON+glTF MVP avoids it) |
| R2 | Frame-graph / renderer-infra is bespoke forever (no community gravity) | High | Keep renderer thin (rerun-style re_renderer scope), accept nightshade-scale simplicity initially |
| R3 | wgpu breaking-change treadmill vs. stable SDK promise | Medium | Pin majors; quarterly upgrade slots |
| R4 | Geo crates are thin-adoption (open-vector-tile 10★, mvt 25★) → fork/vendoring risk | Medium | geozero as abstraction seam; vendoring is cheap (small crates) |
| R5 | Pre-1.0 polish tail: rerun 3.5 yrs still 0.x | Medium (expectation mgmt) | Version the SDK independently of renderer maturity |
| R6 | WebGL2 fallback doubles web QA surface | Medium | Ship WebGPU-only web tier initially (browser support now universal ≥ mid-2025) |
| R7 | CJK text rendering quality on GPU glyph atlas is a known long pole | Medium | cosmic-text (2,139★, Apache-2.0, System76-backed) reduces this materially vs. any GL-era stack |

---

## 7. Source register (accessed 2026-09-03)

| Source | Used for |
|---|---|
| https://github.com/rerun-io/rerun/blob/main/ARCHITECTURE.md | rerun architecture, wgpu+web strategy, re_renderer |
| https://github.com/rerun-io/rerun/blob/main/crates/viewer/re_renderer/README.md | re_renderer scope/philosophy |
| https://github.com/rerun-io/rerun/blob/main/rerun_cpp/README.md | spawn/connect/buffer model (no loop hijack) |
| https://rerun.io/blog (post index JSON) | first blog post 2022-06-15 |
| api.github.com/repos/{rerun-io/rerun, linebender/vello, gfx-rs/wgpu, maplibre/maplibre-native, CesiumGS/cesium, emilk/egui, FyroxEngine/Fyrox, nical/lyon, maplibre/maplibre-native-qt} | created_at, stars, license, pushed_at |
| api.github.com …/releases (rerun, vello, wgpu) | release timelines |
| crates.io/api/v1/crates/{34 crates} | versions, updated_at, downloads, repo links |
| crates.io search: "vector tile", "3d tiles", "frame graph wgpu", "draco", "b3dm" | ecosystem gap mapping |
| https://github.com/gfx-rs/wgpu README (trunk) | backend support matrix, env vars |
| wgpu source (trunk): wgpu/src/api/render_pass.rs, wgpu-types/src/instance.rs | indirect/multi-draw API surface |
| https://github.com/linebender/vello README + releases | status, variants, v0.10.0 |
| https://github.com/maplibre/maplibre-native README + FORK.md | custom engine, fork lineage, metal renderer flag |
| https://github.com/maplibre/maplibre-native-qt | Qt embedding precedent |
| https://github.com/CesiumGS/cesium README | custom WebGL engine, @cesium/engine split |
| https://github.com/emilk/egui README | platform-agnostic core, egui-wgpu/egui-winit integrations |
| https://en.wikipedia.org/wiki/WebGPU | browser ship dates, W3C status, Firefox/Deno use wgpu |
| grep.app code search "draw_indexed_indirect" (Rust) | indirect drawing usage: wgpu, bevy_render, vulkano |

### Explicit UNCERTAIN items
- MapLibre Native GL→Metal migration exact dates (maplibre.org/news is JS-rendered; README only shows the `renderer=metal` build flag).
- Firefox WebGPU platform rollout order (Windows-first claim unverified).
- Linebender `color` crate current status (referenced, not fetched).
- gltf 1.4.1 "last release 2024-05-10" is crates.io `updated_at`; repo pushed 2026-05-11 — treat as slow-release-but-alive.
- Effort table (§3) is synthesis, not measurement.
