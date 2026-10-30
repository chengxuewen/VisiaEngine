# VisiaEngine reference-project library

**Data snapshot: 2026-09-03 measured** (GitHub stars/last-commit via the shields endpoint, versions via the crates.io/npm registry; qualitative descriptions come from model knowledge and memos verified that day; anything marked *UNCERTAIN* must be rechecked before use).
Purpose: the evidence base for the final rendering-backend ruling (D4), whitepaper §4, and the MVP architecture design. **Each file keeps only the part "useful to VisiaEngine" — it is not an encyclopedia.**

## Index

| document | subject | one sentence | relevance |
|------|------|--------|:-----:|
| [wgpu-rust-gpu-stack.md](wgpu-rust-gpu-stack.md) | wgpu/linebender family/glow | ecosystem health check of the main dependency candidates | ★★★ |
| [angle-dawn.md](angle-dawn.md) | ANGLE, Dawn | the industrial floor of legacy-platform translation layers | ★★★ |
| [bevy.md](bevy.md) | Bevy 0.19 | default backend candidate A (research-memo grade) | ★★★ |
| [maplibre.md](maplibre.md) | MapLibre Native+GL JS | the living benchmark of the same lane (maps) + dual-license lessons | ★★★ |
| [cesium.md](cesium.md) | CesiumJS/3D Tiles | unified 2D/3D + streaming + the Open Core structural template | ★★★ |
| [deck-gl.md](deck-gl.md) | deck.gl/kepler.gl | the "data-visualization layer" shape baseline (the 2.5D half of the sky) | ★★★ |
| [scene-graph-lineage.md](scene-graph-lineage.md) | OSG/OpenSG/Coin/VSG/osgEarth | scene-graph school dictionary + maintenance-collapse specimens | ★★☆ |
| [godot.md](godot.md) | Godot 4.7 | editor architecture / backend-strategy isomorphism check / the "all-free control group" | ★★☆ |
| [unreal.md](unreal.md) | UE 5.8/UE6 | Nanite geometry-streaming lessons + license-trust teaching | ★★☆ |
| [unity.md](unity.md) | Unity 6 | the pricing-collapse sample + the origin of the pipeline-fork ban | ★☆☆ |
| [three-js.md](three-js.md) | three.js r185 | the library-shape precedent with the highest isomorphism to Visia | ★★★ |
| [qt-3d.md](qt-3d.md) | Qt3D/Quick3D/QRhi | empirical cold-field proof of official-vendor domain 3D + RHI multi-backend engineering | ★★☆ |
| [flutter-embedder.md](flutter-embedder.md) | Flutter embedder | the gold standard of C-API host embedding | ★★☆ |
| [gdal-proj-geos.md](gdal-proj-geos.md) | GDAL/PROJ/GEOS | the geo-data trio: link C or pure Rust | ★★☆ |
| [qgis.md](qgis.md) | QGIS | layer tree / plugin ecosystem / dual-language-binding UX dictionary | ★★☆ |
| [openusd.md](openusd.md) | OpenUSD | digital-twin scene organization and exchange standard | ★☆☆ |
| [av-sim-opendrive.md](av-sim-opendrive.md) | CARLA/esmini | the direct reference implementations for the ODR/OSC plugin | ★☆☆ |
| [bim-ifc.md](bim-ifc.md) | IfcOpenShell/web-ifc/OCCT | the BIM line: display-layer boundaries and the license minefield | ★☆☆ |
| [ffi-patterns.md](ffi-patterns.md) | SDL3/Vulkan/sokol/Filament/DuckDB/godot etc., 28 sources | C ABI + binding-layering pattern file (the citation register for the batch-2/7 plans' [FFI-R] references) | ★★★ |
| [easy3d.md](easy3d.md) | Easy3D v2.6.1 (GPL, read-only archive) | the cautionary-tale library for host embedding + positive examples for property/pass/tutorial design | ★★☆ |
| [team-gap-analysis-2026-09-28b.md](team-gap-analysis-2026-09-28b.md) | three.js 608-example benchmark + machine census of this repo's 34 examples | the register of the example gap (supersedes the 09-24 edition's example-census section; coverage mapping + missing-domain->feature mapping + three quick wins + action order) | ★★★ |
| [team-gap-analysis-2026-09-24.md](team-gap-analysis-2026-09-24.md) | this repo vs 24 archived projects + the GitHub ecosystem | feature-gap ledger (S/A/B grading; the example-census section was refreshed by 09-28b) | ★★☆ |

## Evidence memos (evidence/)

Dated snapshots, **never updated**; the landing points of the summary files' and D4's citation chains — do not delete or rename.

- [2026-09-03-wgpu-direct.md](evidence/2026-09-03-wgpu-direct.md) — the wgpu in-house-route evidence memo (building blocks/precedents/person-months, 6/6 spot-checked through)
- [2026-09-03-bevy-embed.md](evidence/2026-09-03-bevy-embed.md) — full evidence on Bevy embedding feasibility (2/2 spot-checked through)
- [2026-09-03-angle-integration.md](evidence/2026-09-03-angle-integration.md) — full ANGLE-integration evidence (commit level + the Qt6 refutation + the XP verdict)
- [legacy-platforms.md](legacy-platforms.md) — the legacy/embedded verdict matrix (main-session synthesis: the wgpu native-GL correction / the Rust Win10 floor / G610=Valhall->Panvk)

## Writing discipline

New project files: template = profile(lineage/governance/funding)/architecture essentials(write only what's relevant)/strengths-weaknesses/adopts-and-avoids/sources+snapshot date. Numbers must be same-day measured; anything the license does to the Open Core boundary is bolded.
