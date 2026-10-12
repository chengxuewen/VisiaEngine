# Easy3D — the C++ OpenGL full-stack reference: the cautionary-tale library for host embedding + the positive examples for data/render design

**Snapshot 2026-09-03, measured** (`.refinfo/Easy3D/` local archive, 345M / 1419 C++ files / 52 tutorials): **GPL-3.0 — everything here is concept-reference only, zero code/text transplant**. Positioning = a 3D modeling/geometry-processing/rendering research library (with Python bindings). The team analysis covers 4 domains × 27 conclusions, 34 evidence paths C14-measured: 32 true, 2 clerical slips (line-number level, mechanisms intact).

## Profile
Data structures (PointCloud/SurfaceMesh/PolyMesh/Graph, a shared columnar property system) + a high-level OpenGL wrapper (shader/texture/pass effects) + a self-sufficient GLFW viewer (event loop kept inside). **Not an SDK shape**: the viewer is deeply coupled to GLFW, Qt integration = rewriting the whole thing at the example layer — precisely the mirror of what VisiaEngine does in the opposite direction.

## For VisiaEngine (three tiers, numbering = domain.sequence; A core layer, B render layer, C embed layer, D process)

### Adopt (take the concept, rewrite in our own shape)
- **A1/A2 columnar named-property system**: typed closed-enum columns + geometry/user properties in one namespace -> directly solves our geo `serde_json::Map` style-hot-path stringification. Evidence core/property.h L460/587/628, graph.cpp:37-41. Queue: style round.
- **A4 load report + repair-typed counting**: the builder keeps manifoldness, end_surface(log) reports "what and how much was fixed"; ours: io-gltf/geo return `LoadReport` + `RepairPolicy`. Evidence surface_mesh_builder.h:40/122-125, all 5 loaders go through the builder. GIS dirty data is exactly this disease. Queue: doable as a small slice.
- **B6 Camera three increments**: perspective/ortho share the zoom factor (continuous switching, a GIS hard need), pivot turntable, dpi_scaling. Evidence camera.h:143/342-354. Queue: camera round.
- **B7 cheap value-packing**: scalar->color-ramp LUT + clamp (state.h:58-61 ✓ GeoJSON heatmap's ready-made answer), texture fractional repeat, polygon-offset line-vs-face Z-fighting resistance (viewer.cpp:1805 ✓) [fractional repeat ✅ 4ab WGPU-15; polygon-offset ✅ 4de WGPU-17 bias{-1,-1}], big-texture banding (texture.h:159). Queue: with each owning round.
- **C5 binding-surface pruning method**: of 39 renderer headers only 4 are hand-bound (camera/drawables/renderer/state, drawables merged), an unused/ quarantine belt, GLOB commented out to prevent full-sweep (python/CMakeLists.txt:6 ✓) -> **C ABI first edition = minimal usable set + graveyard process**. Queue: capi design input.
- **D1 numbered tutorial series**: 7 theme segments + reserved empty numbers + a 4-move combo per data type + dir=target=title = document ID single source (CMakeLists 91 lines ✓/T101 only 2 files ✓) -> the blueprint for teaching-oriented examples; 900s = GIS/twin/AV/BIM vertical slices. ✅ **landed in batch 5 (2026-09-14)**: eleven pieces E101..E901 + the empty number E402 + the gate-docs triple lock; E901 = the twin vertical-slice seed (PNG human-checked).
- **D3 zero-drift documentation pipeline**: the Doxyfile EXAMPLE_PATH harvests tutorial source (L854 ✓) -> mdbook `{{#include}}` + rustdoc + cbindgen comment conventions. Queue: docs round.
- **D5 three-layer testing**: unit / frame-budget auto-smoke (duration=1500ms ✓ main.cpp:107) / **explicitly labeled manual interaction + an honest banner** (L133 ✓). Queue: adopt immediately (discipline costs zero).

### Port (mechanism-level transplant, language rewrite)
- **A5 render resources mounted per entity + State binding color by property name**: changing a style never touches the geometry layer (renderer.h:138/154, state.h:49-61 ✓). Also: lazy bbox compute (recomputed on access after invalidate, model.h:64-68 ✓ — the source of the "compute on the fly, never store" concept, used by pick). Queue: PBR-round ownership refactor.
- **B3 pass trio** (ranked by GIS/twin relevance): Shadow/PCSS (⚠ **GL[-1,1] -> wgpu[0,1] is the PIT-5 minefield**) [✅ Shadow/PCSS landed in 4f (2026-09-14, WGPU-19/20+REND-31 depth domain upgraded to contract)]; SSAO/DDP = 4g backlog], 3-pass SSAO + shared-geometry FBO (ambient_occlusion.h:106-114 ✓), DDP transparency (dual_depth_peeling.h:98 ✓; wgpu dual-source is viable, fixed peel count, mutually exclusive with MSAA = naturally compatible with our non-MSAA golden line). Queue: Alpha tier.
- **B4 line-width math**: world radius × distance, camera-space expansion `cross(view_dir,axes)*radius*0.5` (width_control.geom:67-68 ✓), GS->VS instancing. Queue: the direct solution for the screen-space line-width item.
  - ✅ **ported in 4de (2026-09-14, WGPU-17)**: vs_stroke `perp=cross(view dir, axis)` + the px multiplier (fallback to right on degenerate); the depth bias lives in `DepthStencilState.bias` (the wgpu-30 location, not its GL polygonOffset literal translation).
- **B1 named pipeline cache + negative cache + F5 hot reload** (shader_manager.h:119/124 ✓). Queue: PBR round.
- **D4 central resource resolver**: compile-time default + env override + the fixture <10MB budget (their 27M lesson) (resource.h ✓). Queue: before examples multiply.

### Excel (where they fail or do it badly; doing it right is how we surpass them)
- **C1/C2 redo the embedding path entirely**: their Qt relies on the QApplication::notify private hack (Mapple main.cpp:53 ✓), the ImGui build loses mouse coordinates (upstream proven), the GLFW pointer leaks into the public ABI (viewer.h:38 ✓) -> our capi = the single `visiaengine_on_input()` entry + redraw callback + zero window types in the renderer layer (their renderer carries zero glfw references ✓, proof the purity is reachable). **Embedding-slice plan constraint**.
- **C3/C4 few-line integration recipe**: create() is render-ready (zero init ceremony); the Qt side = a Widget holds the engine object + the parent container manages lifetime (T204 viewer.h:47 ✓), rwh isomorphic and free of their subclassing cost.
- **D2 tutorials truly executed**: their tutorials are only compiled, tests default OFF and never enter CI (BUILD_TESTS=OFF ✓) — our `--frames N` + golden already overtook; take their two disciplines "full compile coverage + every feature must ship a runnable example".
- **A3 handle safety**: their raw int indices all die after GC; our slab+generation is naturally FFI-ABA-immune — maintain and write into the capi spec.
- **B5/B2 material system**: their #include mechanism is used zero times out of the box (0 hits repo-wide ✓), the mega-shader bool bucket -> our WGSL minimal include/const preprocessor (~50 lines) + pipeline-key variant dispatch; the material DSL (P4) stays untouched until 3+ shapes exist.
- **D7 binding tutorials = ABI contract tests**: their python/tutorials mirror the same NNN numbers (sphinx_gallery ✓) -> our capi tutorial that doesn't run = an unfinished API.
- **A6/A7 negative-space commandments**: don't prebuild a scene graph (their flat vector without Node ran 10 years ✓); don't genericize Scalar (their traits->concrete-class retreat history ✓ template counts 25/30/15/10).

## Current-state ruling
Not a competitor (GPL + geometry-algorithm positioning), but the **highest-value dual textbook (negative + positive) for the next milestone (host embedding)**: the 8 findings of the C domain are merged into 5 archived entries, all pre-writing our capi constraints; the A/D domains hand over four immediately pickable practicals: properties, load reports, tutorials, testing.

## Sources
team easy3d-analysis (4 members × 36 rounds, 2026-09-03); every cited path measured by the orchestrator with grep/sed. Archive read-only, `.refinfo/` never enters the repo.
