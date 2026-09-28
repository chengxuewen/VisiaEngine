# Example-Gap Analysis — three.js Benchmark (2026-09-28b)

> Method: team-mode two-lane (three.js web taxonomy / VisiaEngine machine census), lead cross-validated.
> Volumes: `/tmp/opencode/threejs-vs-visiaengine-report.md` + `/tmp/opencode/visia-census-report.md` (session artifacts).
> Supersedes the example-census sections of `team-gap-analysis-2026-09-24.md` (that file's S/A/B feature-ledger remains valid where not refreshed here).
> Scope: examples + the engine features they map to. Scope ruling from 09-24 still holds (MediaServo/unknown-provenance archives never cited as anchors).

## One-line summary

VisiaEngine holds a 34-example, 9-band, machine-executed inventory that is CLEAN three-way (files = tutorials = registries, zero drift, all gates live) and LEADS three.js on GIS/tiles/GeoJSON lanes, pixel-gate verification, and the C-API embedding story — but trails it 34:608 on breadth, with the sharpest gaps in postprocessing (59 three.js examples, zero ours), IBL/PBR, HDRI/tonemapping, and the E4xx interaction band (2 examples, our thinnest).

## Volume comparison

| | three.js | VisiaEngine |
|---|---|---|
| Examples | 608 / 14 categories | 34 / 9 bands |
| Biggest lanes | loaders 65 · postprocessing 59 · materials 55 · text 47 · IBL/lights 35 | SDK-consume 10 · data-loading 6 · camera 5 |
| Machine visual QA | none (manual) | all examples: golden pixel gates + dual-mode (window / ctest argv) |
| Per-band census | — | E1xx 1 · E2xx 6 · E3xx 5 · E4xx **2 (thinnest)** · E5xx 4 · E6xx 1 · E7xx 4 · E8xx 10 · E9xx 1 |

## Coverage map (three.js category → VisiaEngine status)

GOOD ×10: glTF loaders · PLY · MVT tiles (+HTTP pump) · GeoJSON→projection pipeline · instancing · picking (mesh+point) · multiview (novel: same-canvas) · fly camera · color management · offscreen readback.

PARTIAL ×5: labels (no billboard/CJK/rich text) · clipping (1-plane AND; no 6-face box/stencil caps) · shadows (directional PCSS only) · camera controls (no map-controls preset / pointer-lock / gizmo) · animation (origin keyframes only).

NONE ×9: postprocessing · env/IBL/PMREM · GGX PBR · HDRI/tonemapping · skeletal/morph-targets · OIT · volume · GPGPU/compute · XR · physics (deliberate non-goal) · CSS/DOM overlay.

## Reverse advantages (verified — do not regress)

- **GIS/tiles**: three.js has ONE 3D-Tiles demo and zero MVT/GeoJSON pipeline — our E206 + io-tiles + HTTP pump is effectively a three.js gap.
- **Machine-verified examples**: every example runs under golden pixel gates + dual-mode discipline; three.js has no automated visual QA.
- **C-API embedding**: 44-entry C surface + three-face mirror + wasm story; three.js is JS-first with no embed surface.
- **Gallery**: 34 cards + per-example detail pages + sticky category nav (N1.6/N2.3), locally served.

## Missing domain → missing engine feature (TOP 9, ranked by demo value × size)

| # | Missing example domain | Required engine feature | Size |
|---|---|---|---|
| 1 | Postprocessing family (59 three.js examples: bloom/outline/SSAO/DOF) | fullscreen pass-chain framework — EDL already proved the intermediate-target pattern; framework unlocks each further effect at M each | M |
| 2 | IBL / environment lighting (25 examples) | PMREM prefilter + GGX BRDF + env sampler | L |
| 3 | HDRI + tonemapping | .hdr/.exr decode + ACES/AgX curve in FS tail | S-M |
| 4 | Box selection | NONE — pick primitives exist (REND-21..24); rect sweep = consumer work | **S** |
| 5 | Label billboarding + CJK demo | anchor-space billboard; CJK = load_font already supports host-injected TTF (zero demo today) | M |
| 6 | 6-face clipping box + stencil caps | REND-32 AND-chain extension + stencil pass | M |
| 7 | OIT | depth-peel or weighted-blend (fixes E504 ordering on complex scenes) | M-L |
| 8 | Morph targets / skeletal animation | per-vertex target attrs / joint palette (REND-42 covers origin replay only) | M/L |
| 9 | Transform gizmo · volume · GPGPU · XR | screen-space handles · 3D textures+raymarch · compute · WebXR (low priority for GIS/DT SDK) | M-L→XL |

## Feature-free quick wins (pure consumer examples, all S)

1. **Box-select example** (E402 extension): rect sweep + toggle-select loop over existing pick infra.
2. **CJK font demo**: load_font with a CJK TTF — engine capable today, zero demos.
3. **Map-controls preset**: orbit+pan+zoom composed into a one-call "map controls" shape (misc_controls_map analog).

## Census integrity (machine truth)

- Three-way census: 34 files (33 E-numbers; E811 C/C++ twins) = 33 tutorials rows = 33 registered (20 rs + 13 native; E703 via bare add_executable third registry) = 34 gallery cards. Zero drift, R9 anti-glob intact.
- Gates covering the census axes already exist (three-way filename/header/tutorials lock; R9 anti-glob; gallery manifest==registry; README==live counts) — no new gates needed.
- Drift: none (gate-docs live-verified this session: "E 33 three-way / README=182↔spec=182 / signatures 44 / gallery 34 cards").

## Recommended action order (awaiting adjudication)

1. **S quick-wins band**: box-select + CJK demo + map-controls preset (pure consumer, fills E4xx thinnest band).
2. **Postprocessing framework band** (M): highest visible win; EDL pattern de-risks it.
3. **HDRI + tonemap** (S-M): small cost, large mood gain, pairs with 2.
4. **IBL/GGX** (L): formal Alpha-tier item.
