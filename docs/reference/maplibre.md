# MapLibre (Native + GL JS) — the living benchmark of the map lane

**Snapshot 2026-09-03, measured**: maplibre-native 2.2k★ (last-commit ≈09-02); maplibre-gl-js 12k★, npm v6.7.0 (2026-09-02, frequent releases); the control mapbox-gl-js 12k★. BSD family. Forked 2020-11 (the Mapbox GL license-change event) — **a living case study of Open Core trust narratives**.

## Profile
- **Native**: a C++ in-house rendering core (vector-tile pipeline), first-class on mobile (iOS Metal flag `--//:renderer=metal`, Android GLES; Vulkan *UNCERTAIN state unverified*), desktop relies on community bindings (maplibre-native-qt 136★ active — the direct precedent of Qt-host embedding).
- **GL JS**: the browser standard piece; Style Spec v8 = the de-facto standard of declarative map-style DSLs; CustomLayerInterface opens the GPU pipeline to third-party layers.

## For VisiaEngine
- **Adopt (heavy)**: ① the JSON semantics of Style Spec v8 (layers/sources/paint/layout separation, the expression language, zoom functions) — the mature answer for Visia's "layers + styles" subsystem has already been validated once, copy the structure not the baggage; ② the MVT decode pipeline and glyph/sprite atlas management = the complete requirements checklist of 2D map rendering; ③ the GL JS custom layer = the correct posture for host-extensible rendering (aligns with the deck.gl file); ④ the two cores (Native/JS) not sharing a source = the counter-example of Visia's "one kernel, many hosts", the differentiation argument lives here.
- **Avoid**: ① a map specialist: no true 3D scene semantics (extrusion is a tile property, not a spatial entity) — exactly the whitepaper 1.1 gap; ② Native desktop being a second-class citizen (thin Qt binding, Windows marginal); ③ upstream governance is a fork community left over from the Meta era, its roadmap passive.
- **Current-state ruling**: the 2D/2.5D map capability baseline reference = maplibre (performance, style coverage); MVP acceptance can be benchmarked against it.

## Sources
img.shields.io / npm registry (2026-09-03); README/FORK.md in `evidence/2026-09-03-wgpu-direct` §1c (fetched the same day).
