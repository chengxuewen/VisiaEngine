# Gap Analysis — Team Research Synthesis (2026-09-24)

> Method: team-mode three-lane parallel dissection (refinfo archives / GitHub web ecosystem / internal self-audit), lead cross-validated and synthesized.
> Volumes (full text, machine-grep citations): `/tmp/opencode/gap-volumes/{refinfo,self-audit,web-ecosystem,lane1-promises}.md` (session artifacts; key findings consolidated below).
> Supersedes `team-gap-analysis-2026-09-17.md` as the current baseline. That file's S1 (color), S2 (labels), S3 (events), S5-half (PLY points), A3 (transparency), A4 (clipping) are now CLOSED — delivered by the color-events / io-text / pcl-data / clip / translucent / flyto / multiview / minimap bands.
> Scope rulings: MediaServo + OpenCTK excluded (read-only history / provenance unknown — never cited as anchors).

## One-line summary

The 09-17 vacuums are mostly filled (color ✓, events ✓, labels ✓, points half ✓). The new vacuums changed location: the heaviest debt is now in the **GIS family the engine is named for** (tile streaming / terrain / LOD = all zero), and the **storefront** is the worst among all peers measured — 31 examples of world-class quality (all machine-executed with pixel gates) with zero web visibility.

## Example-count census (measured 2026-09-24)

| Project | Examples | Form | Machine-executed in CI? |
|---|---|---|---|
| three.js | 609 pages + 36 addon dirs | web gallery, one page each | no |
| Cesium Sandcastle | 309 | live-coding web gallery | no |
| PlayCanvas | ~290 modules / ~20 cats | web gallery | no |
| bevy | 458 files / 31 cats | source tree | partial |
| Babylon.js | 143 official samples | Playground | no |
| MapLibre GL JS | 140 example pages (+207 API pages) | **docs page = live demo = CI screenshot test** | yes |
| Easy3D | 52 tutorials | source tree | **never run** |
| wgpu | ~39 | source tree | no |
| threepp | 22 | source tree | no |
| **VisiaEngine** | **31** (30 E-numbers; 17 rs + 11 native) | source tree, dual-mode (IDE human-check / ctest argv), golden pixel gates | **all** |

Quality dimension: our discipline leads every peer measured (spec-trace 152↔152 machine lock, golden three-probe predicates, three-way filename lock, dual-mode examples). Surface dimension: behind even wgpu; an order of magnitude behind the web players.

## S-level gaps (new, old ones closed)

| # | Gap | Evidence | Note |
|---|-----|----------|------|
| S1' | **MVT / WMTS / HTTP tile sources + tile scheduler + LRU cache** | whitepaper 3.2 says "built-in" in present tense; grep `mvt/wmts/vector-tile/tile-sched/LRU` = 0 code. Largest doc-vs-code contradiction; the GIS core promise | Big engineering; needs its own planning band, or whitepaper reword to Beta tier |
| S2' | **No browsable web gallery** | Every peer's front door is a visual index; ours is a source tree. Assets already exist (wasm demo + golden-frame PNGs) — pure packaging, cheapest differentiator | Cost S-M |
| S3' | **Scene serialization** | Visia Studio (Beta roadmap) hard-depends; grep `serializ/save/snapshot/ron` = 0; untracked anywhere | Add to backlog |
| S4' | **Scene graph parent/child hierarchy** | Whitepaper/README repeat "one scene tree" 4×; `core/src/scene.rs` is a flat slab. Unblocks subtree animation + per-subtree instancing | Untracked |

## A-level (function debt, band candidates)

- **Point picking absent** (NEW, 3 independent anchors): pick family is mesh-only; points render via splat but cannot be picked — the "point cloud as first-class citizen" last mile. Cost S.
- **Entity/object animation API** (NEW evidence): digital-twin scenario = time-varying data; only the camera animates today. glTF animation tracks silently dropped at load (0 SDD clauses, untracked). Skinning/morph stays C-level (don't do).
- **Async / cancellable / progress loading**: all loaders sync-block the host thread; greps `JoinHandle/progress-callback` = 0. (Progress events exist for the load pipeline internally; host-side async surface absent.)
- **Flutter + C# bindings**: MVP roadmap names "Qt/Flutter/Web embed examples"; `bindings/` = c/cpp/js/qt only. Gap written nowhere — must be tracked or the MVP claim reworded.
- **True PBR GGX**, **SSAO**, **LOD distance switching**, **terrain/TIN/raster**, **multi-light + IBL**, **LAS** — all pre-existing tracked items, unchanged.
- **capability_query**: architecture invariant #5 names it; zero code.
- **Gizmo / transform editing** (NEW, 3 anchors): Easy3D tutorial + three.js TransformControls + osgManipulator.

## B-level (strengthen, opportunistic)

Heatmap · fog/sky · volume · box-selection · undo/redo · touch input · XR (revisit trigger only) · GPU compute / occlusion culling · DRACO/KTX2 decode · io-opendrive · Potree-style huge-point-cloud streaming (Easy3D ToDo analog) · GaussianSplatting (record-only in archives; stays C unless a customer ticket) · outline/EDL/wireframe · cargo-generate standalone template (wgpu pattern) · in-browser editable playground (Babylon pattern, defer).

## 21 zero-coverage example domains (three.js taxonomy heat list)

Tile streaming · terrain · heatmap · fog/sky · volume · LOD · outline/EDL/wireframe · postprocessing · particles · entity animation · skinning/morph (C-level) · scene-graph editing · serialization · loaders beyond gltf/geojson/ply (MVT/WMTS/LAS/3DTiles/STL/OBJ/IFC) · multi-light/IBL · XR (C-level) · touch · box-selection · gizmo · undo/redo · GPU compute.

Stark cluster: **the GIS family the engine is named for**. Fundamentals (camera/pick/clip/measure/instancing/text/embedding) are unusually strong.

## Doc-vs-code drift (adjudicate: reword docs, or open a band)

1. WMS/WMTS "built-in" → zero code. Reword whitepaper to Beta tier, or open the S1' band.
2. Flutter MVP example → zero code; reword or track.
3. 10MB startup budget → promised in two doc faces; native never measured (one measurement closes it).
4. capability_query / rstar-dep-in-diagram / cosmic-text-vs-fontdue → design-baseline drift (mitigating notes exist for some).

## C-level (don't do — anti-benchmark-anxiety, unchanged from 09-17)

ECS bundle · render-graph/TSL DSL · XR host-side · hot-reload (desktop) · WebGPU/GL dual backend · 30-pass post framework · skinning animation · private formats · GaussianSplat/Water/Sky web effects · scripting game layer.

## Top new infra pattern worth adopting

**MapLibre "one artifact, three consumers"**: every example = (a) docs page with prose+code, (b) live demo, (c) CI screenshot test. Extends — does not fight — our existing SDD/ctest locks. Highest-leverage pattern found this round.

## Recommended action order (awaiting adjudication)

1. **S2' web gallery band** (S-M): static site reusing golden PNGs + the 31-example registry auto-generating an index page. Pure packaging, no engine work.
2. **Point-picking mini-band** (S): extend pick family to point clouds.
3. **MVT/tile streaming** (L): biggest true gap; own planning band (MapLibre pattern reusable).
4. **Doc reconciliation mini-band** (S): whitepaper reword + 10MB measurement + track the untracked gaps (S3'/S4'/entity-anim/Flutter).
5. **MapLibre tri-consumer pattern**: adopt as standing policy for future example bands.

## Reverse advantages (verified this round — do not regress)

spec-trace 152↔152 machine lock · golden pixel gates + three-probe predicate discipline · three-way filename lock · dual-mode examples (human IDE / CI ctest same source) · five-language binding mirroring (C header/hpp/wasm+d.ts/CAPI mirror) · D7 far-origin f64 rebase · LoadReport repair-policy loading · placed() single-source transform law. Conclusion: **discipline ahead of all peers measured; surface behind even wgpu.**
