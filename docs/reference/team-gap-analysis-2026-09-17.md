# Four-way benchmark gap audit · team research synthesis (2026-09-17)

> Method: team-mode four-lane parallel dissection (`.refinfo/three.js` / `.refinfo/Easy3D` / `.refinfo/bevy` + the example/documentation ecosystem cross-compared across the four), lead cross-verified synthesis.
> Every conclusion carries double-sided file evidence; the S-tier claims were re-verified with seven greps (color management/hierarchy/text/capi callback/blend/animation/async = all 0 hits confirmed).
> The raw four volumes live in the session of that time (analysts: threejs-gap / easy3d-gap / bevy-gap / examples-gap); this file is the synthesis of record.
> ⚠ Three untruths in the examples volume were corrected during synthesis: E901 = twin_city (not a wasm demo); E403 is an in-use number (not reserved empty); E202 is the GeoJSON point-marker domain (not the point-cloud data family).

## One sentence (plain-language edition)

**On engineering discipline and the SDK skeleton we lead; the three big vacuums are picture correctness (color), the map half of text, and the "host-experience API" (event push / async loading / scene hierarchy).** Each of the three reference points illuminates a different class of problem: three.js shows "the renderer is still juvenile" (color/material/PBR/post), bevy shows "the host is still primitive" (events/resources/serialization), Easy3D shows "GIS promises unfulfilled" (point cloud/text/transparency/section — of which terrain/VSM/GGX are three items Easy3D itself does not have either, so they belong to self-initiated work, do not bill them under a benchmark label).

## Consensus matrix (only entered if ≥2 volumes independently agree)

### S tier (strategic gaps: the product's face or positioning promises)

| # | gap | volume | evidence anchor | this repo's current state |
|---|------|--------|---------|---------|
| S1 | **the full color-management chain** (sRGB->linear->tone map->output) | threejs | `.refinfo/three.js/src/math/ColorManagement.js` + the ToneMapping family | all grayscale = drawing linear domain as sRGB; the golden is locking wrong values, every round of delay grows the re-record cost. Cost M |
| S2 | **the text/annotation plane** (the map half) | threejs/easy3d/examples, three volumes agree | text_renderer(T309)/Sprite/23 text examples | zero glyph hits; E202 cannot draw even one POI label. Cost L |
| S3 | **the event-push half** (loading state is a black box to the host) | bevy/threejs | EventDispatcher; the bevy full chain loadable->loaded(AddAsset)->LoadFailed | capi has zero callbacks; on_ready is a pull-poll form, the loading middle state has nowhere to live. Cost S-M, **a pure port addition, no semantic change** |
| S4 | **the resource pipeline** (async/cancel/cache/handle invalidation/unload) | bevy/threejs | asset/LoadState + Guard against handle reuse + the LoaderCycle dependency graph; LoadingManager+AbortSignal | synchronous load blocks the host thread; GLB-only; no progress/cancel. Cost L |
| S5 | **the point-cloud data family** (a whitepaper first-class citizen, unfulfilled) | easy3d (incl. the old-table accounting correction) | point_cloud_io_{ply,las,...}+kdtree+normals | the demo layer once cashed the check, the data-family layer never did: no IO/types/normals/spatial index. Cost medium-large |

### A tier (feature debt: next-phase mainline)

| # | gap | volume | key point | cost |
|---|------|-----|------|------|
| A1 | scene hierarchy parent | threejs/easy3d | twin "device->component", layer-group transforms/group visibility need a parent-child chain; a two-level lightweight form suffices (parent: EntityId, does not break the flat C ABI handle surface) | M |
| A2 | real PBR GGX + glTF channels | threejs/bevy/easy3d | currently a 4-field MaterialDesc; normal/emissive/occlusion/metallic all discarded (dumb data at load); the Alpha tier is on the books, light the channels one by one | M |
| A3 | transparent sorting (+DDP) | easy3d | blend:None as the single route, empirically; white-model transparency + layer compositing are both hard needs; first back-to-front sorted blend, DDP = upgrade slot | medium |
| A4 | cross-section clipping | easy3d | clip_plane+T307; **already queued, B2 standby order**; the uniform plane + fs discard shortest route | small-medium |
| A5 | texture decode (PNG/JPEG) + the point-cloud PLY entry | threejs | upload_texture only eats raw bytes; io-gltf does not resolve external image URIs (the half root-cause of GLB-only) | M |
| A6 | the attribute-push half (attr_changed/dirty) | bevy | currently full re-query (the E810 route); change notification missing | S |
| A7 | the pick half (hover push) | bevy | polling is viable; E402 already proved the CPU ray route | S |
| A8 | animation player (minimal two tracks) | threejs/bevy | glTF animation tracks are dropped outright at io-gltf; the time dimension of twin demos/AV simulation = zero; take only the transform track + visibility track (handle-directed; the PropertyBinding reflection form does not hold across a C ABI), no skinning | L |
| A9 | projection/CRS coordinate systems | whitepaper/geo domain | web_mercator single projection; the WMS/WMTS promise is the same family; a georeferencing hard need, **self-initiated, off-benchmark** | medium-large |
| A10 | scene serialization (lightweight IR: save/restore/partial update) | bevy | no port for host-state persistence; the format goes glTF extension / a self-designed JSON, **do not copy .scene RON** | medium |
| A11 | automatic LOD switching | bevy/threejs/Easy3D, three houses | entity-level Instance LOD exists, distance-switch scheduling does not; same issue as tile streaming | medium |

### B tier (strengthening/experience, when the moment fits)

public geometry-primitive ports (box/sphere/cylinder into capi, a twin body-creation hard need, cost S) · BVH spatial index (trigger = bench evidence, the picking.rs note on the books) · camera damping + keyframe roaming (the mix_rig primitives exist, the residue = orchestration) · SSAO/EDL (4g backlog; the EDL trigger = S5 landing) · rustdoc auto-generation (**lowest cost, immediate payoff** — bevy 438 examples + full docs.rs, three 658 pages vs this repo's 0 auto-reference pages) · gizmo/collision (the 1.0 editor tier) · multi-viewport (with the site round) · llms.txt (extremely low cost) · migration guide (recorded as a vote before GA).

### C tier (explicitly not done, anti-benchmark-anxiety)

the ECS full kit (pull model + push patch is the correct SDK shape; bevy itself proves ECS does not run on a library thread) · render graph / TSL-style DSL (fixed pipeline + variant keys + default rejection, 2 wgsl files suffice; the only forward slot = re-evaluate should styles ever need runtime expression compilation) · XR (host-held) · hot-reload (desktop tier) · the WebGPU/GL dual-backend tax (already paid by wgpu native, same verdict as D4) · a 30-pass post framework (hook-point enum slot as substitute) · skinning · KO/SM private formats · GaussianSplat/Water/Sky web showpieces · the scripting game layer.

### Benchmark-invalidation statement (accounting hygiene, the easy3d volume's methodology)

terrain/TIN/raster, VSM/CSSM, GGX: three items **without Easy3D evidence** (zero grep hits) -> moved into the whitepaper's self-initiated track, strictly forbidden to bill under the "Easy3D borrow" label (the reverse application of the PIT-20 phantom-API discipline). One isomorphic correction in the bevy volume: bevy is not without visual regression — the Pixel Eagle screenshot service + frame-event RON + 438-example single-source metadata; this repo's golden advantage holds but the wording narrows from "exclusive" to "exclusive among gate families".

## Ordering suggestions (decision points, item by item)

- **D-1 which band next**: the current queue has B2 = cross-section clipping (A4 queued) vs interjection candidates = the point-cloud data family (S5) / color management (S1). Lead recommendation: **move S1 color management half a beat ahead** — reason: golden re-record cost grows monotonically with each round, and it is the only pure-rendering round in the S tier that does not touch the API surface; the clipping band keeps its queue place.
- **D-2 the event-push port (S3)**: can ride with any band (CAPI minor version, pure addition); suggest RED->GREEN in the same batch as the color round or the clipping round.
- **D-3 the text plane (S2)**: a standalone planned round (the font-atlas selection leads), not stuffed into the existing queue.
- **D-4 the resource pipeline + async (S4)**: a large undertaking, suggest standing it up as its own Alpha-tier case (with MVT/tile streaming = the same design sheet).
- **D-5 the point-cloud family (S5)**: interject or queue = the user's commercial-cadence ruling (the whitepaper promise is already in arrears; the easy3d volume argues for interjection).
- **D-6 bundle the low-cost immediate wins**: rustdoc + llms.txt + geometry-primitive ports + attribute dirty notification — one small "docs and sugar" band can take them all.

## Reverse-advantage grand total (four volumes merged and deduplicated, the do-not-regress list)

1. D7 far-coordinate f64 rebase (three = pure f32, large coordinates are disqualifying; bevy also eyes the same projection_zoom kind but without a far-origin discipline) 2. **the SDD 115-clause <-> tests bidirectional lock** = genuinely exclusive (all three others: 0) 3. the gate-docs/gate-abi four-way-consistent gate family (bevy's single-source metadata is closest yet still has no machine-consistency check) 4. `--frames`+golden three-way self-proof = the examples are the CI (bevy's 438 examples crush in count but only partially automated; Easy3D never runs; this repo's 19 all execute) 5. the multi-language example matrix, 5 routes (every other house single-language only) 6. the pure pull model: the host decides render timing + zero main-loop hijacking 7. construction guarantees of no aliasing / no GC / no data race + 162 tests in seconds 8. dual license versus GPL contagion (Easy3D) / pure-JS runtime (three) — automotive HMI knocks the opponents out 9. the style spec as a first-class citizen (the D9 v8 subset) 10. the LoadReport repair-policy loading (all three others let dirty data in silently) 11. the placed() configuration single-source construction lock (interaction correctness joins the machine-gate culture).

**Close**: at about 1/30 the complexity we gained aliasing-free, panic-free crossing, FFI-capable determinism — that is the architecture tax the three others pay, and we did not. The benchmark verdict = **fill the three S-tier items (color/text/event push), guard all 11 signature advantages, refuse the C-tier anxiety**.
