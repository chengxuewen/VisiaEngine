# Gap-closure roadmap — where VisiaEngine stands against peers

**Written 2026-10-08.** This file exists because the project ledger cited two audit artefacts that
were never in the repository (`.agents/memorys/status.md` claimed a "double master copy" of a
2026-09-30 audit and of this roadmap; neither path had ever been committed). The citations are now
repaired and this is the artefact they point at.

**Every number below is machine-derived.** Re-verify with:

```bash
bash scripts/spec-trace.sh | tail -1                                   # clause count, two-way trace
grep -oE '^## (CORE|REND|WGPU|GLTF|GEO|IO|CAPI|WEB)-[0-9]+' docs/sdd/*.md |
  sed 's/.*## //' | cut -d- -f1 | sort | uniq -c                       # per-family breakdown
grep -oE 'visiaengine_[a-z0-9_]+\(' bindings/c/visiaengine-capi/include/visiaengine.h |
  sort -u | wc -l                                                      # C ABI entries
ls examples/*/[E]*.* | wc -l                                           # example files
ctest --test-dir target/cmake-smoke -N | tail -1                       # ctest registry
cargo metadata --no-deps --format-version 1 |
  python3 -c 'import json,sys;print(len(json.load(sys.stdin)["packages"]))'   # workspace packages
```

---

## 1. Census (measured 2026-10-08)

| Axis | Value |
|---|---|
| Workspace | 12 packages: 9 crates under `crates/` + `visiaengine-capi` + `visiaengine-wasm` + `examples` shell |
| Behaviour contracts | **199** clauses, two-way traceable to tests: REND 47 · WGPU 34 · GEO 27 · CORE 21 · CAPI 41 · IO 18 · GLTF 11 |
| C ABI surface | **49** entries, abi `(1<<16)\|16`, mirrored across header / hpp / wasm / d.ts |
| Examples | **43** files in 9 bands (E1xx 1 · E2xx 6 · E3xx 6 · E4xx 3 · E5xx 11 · E6xx 1 · E7xx 4 · E8xx 10 · E9xx 1), each with a ctest consumer path and a gallery card |
| Gates | `pixi run ci` = 11 segments; plus `pack-check` and `web-check` as independent tasks |
| Shipped consumption routes | in-tree `add_subdirectory`, **installed CMake tree** (`find_package(visiaengine)`) and `pkg-config` (D23) |

## 2. The 2026-09-24 S-level gaps, re-checked line by line

| Then | Evidence then | Now |
|---|---|---|
| S1' MVT / HTTP tile sources + scheduler + LRU ("largest doc-vs-code contradiction") | grep = 0 code | **Closed.** `visiaengine-io-tiles`: MVT wire decoder (no new dependency), `TileSet` visible-enumerate + `ensure`/`pump` + LRU; CAPI-28/29/35 (dir + HTTP) and CAPI-39/40 (raster basemap) |
| S2' no browsable web gallery | peers all have one | **Closed.** 43 cards + per-example detail pages, thumbnails rendered by the examples themselves (`pixi run gallery`) |
| S3' scene serialisation | grep = 0 | **Still open** (re-grep today: 0). Hard dependency of the Visia Studio Beta tier; no plan filed. See §4 |
| S4' scene graph parent/child | `core` was a flat slab | **Closed** for the promise that mattered: CAPI-30..34 (groups, reparent with cycle/depth guard, group offsets, ancestor-aware visibility). Rotation/scale inheritance deliberately not inherited — trigger-gated |

## 3. Coverage map vs the three.js benchmark (09-28b), refreshed

| Peer category | Our status | Anchor |
|---|---|---|
| Postprocessing (bloom / outline / SSAO / DOF family) | **Chain framework delivered**; bloom, outline, tonemap, SSAO, depth-haze ship as effects in it. DOF/SSR not | WGPU-34/36/38, REND-43/44; `E506`, `E509` |
| Environment / IBL lighting | **Demo-level, not PMREM**: SH-9 irradiance from `.hdr` (RGBE) | WGPU-39, IO-17, CAPI-41; `E507` |
| Materials / PBR | GGX BRDF + textures + instanced variants | WGPU-35, REND-45 |
| Loaders | glTF (GLB), GeoJSON, PLY ascii/bin_le, MVT, raster tiles, `.hdr` | GLTF-\*, GEO-\*, IO-\* |
| Camera / navigation | orbit rig, 2D↔3D continuous projection morph, eased `fly_to`, minimap + click-to-navigate, split screen | REND-34, CAPI-23..27, ⑤b |
| Selection / measurement | hover, multi-pick, **box select**, planar measurement, point pick, stroke pick (predicate; engine wiring ticket-gated) | REND-21..24, 38, 46; `E402`, `E403` |
| Clipping | 4-plane AND chain, all primitive families + shadow casters, pick-aware | REND-32, CAPI-20 |
| Curves | Bezier / Catmull-Rom with tube + ribbon extrusion | REND-47; `E510` |
| Text | rasterised labels with anchor rules, CJK via host-supplied font subset | IO-07..09, GEO-25; `E205`, `E505` |
| Points cloud | 100k-scale PLY, EDL outline, splat points | IO-\*, WGPU-33; `E204` |

### Not done, written plainly (the peer categories we simply lack)

| Gap | Verified state today |
|---|---|
| Skinned meshes / morph targets | no joint palette, no per-vertex targets; only rigid node transforms (CAPI-42/43) and origin-track replay (REND-42) |
| Vector + raster tile layers **at the same time** | single layer slot in the capi engine (`engine.rs:1196/1991/2021` all assign one slot) → basemap and overlay are mutually exclusive; band T of `.omo/plans/ledger-tiles-keys-2026-10-08.md` fixes it |
| Scene serialisation | 0 code |
| Spatial index (quadtree / kd-tree / R-tree) | 0 code (grep for kdtree/quadtree/rtree/spatial_index across `crates/` returns nothing) |
| 3D Tiles, terrain/DEM/heightmap, OGC 3D formats | 0 code |
| Order-independent transparency | 0 code (painter sort + per-object alpha only) |
| 6-face clip box, stencil caps on clipped volumes | 0 code |
| Label halo/backdrop, collision avoidance, multi-line + rich text, text-update entry | open — labels are already world-anchored with view `right`/`up` expansion and screen-constant size (`mesh.wgsl:359-372`, WGPU-24/25), so this is the polish lane, not a missing-primitive lane |
| LAS / LAZ point clouds | PLY only; band V2.3 stays closed until a real `.las` fixture lands |
| Browsable docs site | gallery is a build artefact, not a published site |

## 4. Deliberate gaps and their triggers

A gap stays closed until its trigger is real; that is a decision, not neglect (see D-entries).

| Gap | Trigger to open | Size |
|---|---|---|
| Scene serialisation | a Studio or save/restore demo demand | M |
| Spatial index | pick/hit-test cost measured on a real scene exceeding a frame budget | M |
| Skinning + morph targets | a rigged glTF fixture + a demo ticket | L (own plan) |
| LAS | one genuine scan file in `resources/data/` | M |
| OIT / 6-face clip / stencil caps | a scene where painter sort visibly breaks | M / M-L |
| npm / PyPI packages, tarball, RUNPATH scrub | GitHub mirror activation + org name | M |
| win / mac install forms | CI matrix day | M |
| C binding generator (auto header) | C API stabilisation push | M |
| Flutter / C# hosts | first external host asking | M each |
| Docs site | decided as its own band | S-M |

## 5. Reverse advantages — do not regress these

1. **Every example is machine-executed in CI** with golden pixel gates and dual-mode entry
   (zero-arg = human window, `--frames N` = ctest). The 09-24 census found no peer doing this:
   three.js (609 pages) and Cesium (309) run none, Easy3D's 52 tutorials "never run".
2. **Contract-to-test two-way trace** on 199 behaviours (`scripts/spec-trace.sh`), so a clause
   without a test and a test without a clause both fail the build.
3. **C ABI discipline**: one header, 49 entries, five-face mirror (header / hpp / wasm / d.ts /
   roster) and an abi word checked at runtime; `gate-abi` counts symbols from the built library,
   not from source text.
4. **Zero-dependency formats**: MVT decoder and PLY reader are hand-written rather than pulling
   crates; the shipped SDK cdylib is 7.25 MB stripped (measured, `docs/reference/evidence/`).
5. **Consumable install tree** with a SONAME'd library and a relocatable package config (D23).
6. **Gates must be seen red**: every assertion in this repo ships with a break probe recorded in
   its commit (the discipline that found the constant-true pixel gate and the phantom citations
   behind §0 of this file).

## 6. Tier alignment with the whitepaper

| Whitepaper tier | Status |
|---|---|
| MVP (glTF + GeoJSON + 2D↔3D + host embed) | delivered |
| Alpha: spatial index / coordinates | **open** (§4) |
| Alpha: 2.5D white model + oblique view | open; note the fixed underlay offset chosen for stacked tiles must be revisited when tilt lands |
| Alpha: C API stabilisation + multi-language binding generation | C face stable and mirrored; **generation open** (hand-written header by decision) |
| Beta: ODR/OSC, real-time twin binding, Studio preview | open, each its own band |

## 7. Standing declines (anti-benchmark-anxiety)

Transform gizmos, volumetric raymarching, GPGPU/compute pipelines, WebXR, and "keep up with 609
examples" as a goal in itself. Numbers of peer examples are not a metric we optimise; coverage of
our own four stated domains (GIS / digital twin / AV simulation / BIM display) is.
