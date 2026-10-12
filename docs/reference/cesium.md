# Cesium — unified 2D/3D + streaming + the Open Core structural template

**Snapshot 2026-09-03, measured**: cesium 16k★ (≈09-02 active); npm 1.145.0 (2026-09-01, ~weekly); the 3d-tiles spec repo 2.6k★ (August). Apache-2.0. repo created 2012-03 — **14 years of continuous engine investment**.

## Profile
An in-house WebGL geo-3D engine (globe/terrain/imagery/3D Tiles/time-dynamic entities). The business = Cesium ion (closed-source cloud services: conversion/hosting/assets) + an open-source runtime — **an Open Core isomorphic with Visia's plan**: the whole software stack is open source, the money is in the data-pipeline service. Cesium for Unreal/Unity = its own streaming renderer **pluginized into game engines** — the other answer to "don't hijack the host" (the library enters the host, rendering runs through the host's pipeline).

## For VisiaEngine
- **Adopt (heavy)**: ① **3D Tiles = an OGC standard + streaming LOD (SSE-driven) + b3dm/pnts evolution (1.1 GLB-ification)** — Visia's point-cloud/oblique-photography/BIM streaming adopts the spec outright instead of inventing its own; ② the quadtree scheduling and request merging of terrain/imagery, the `EllipsoidTerrainProvider` abstraction; ③ SampledProperty / the time-dynamic model = the data-shape answer for digital-twin IoT binding; ④ the productized semantics of the 2D/2.5D/3D **SceneMode switch** (the morph animation) = the interaction-spec template for Visia §3.1's promise; ⑤ ion's "free tier + pay-as-you-go" pricing skeleton (a commercialization-phase reference, *data points from public pages, not item-by-item rechecked*).
- **Avoid**: ① the JS kernel -> native embedding is nearly impossible (plays straight into Visia's selling point, see the whitepaper 1.1 correction sentence); ② the API surface's historical obesity (14 years of baggage: deprecated layers piled up); ③ ion's closed-source conversion pipeline = hidden lock-in on data-format coverage; if Visia copies its shape it must avoid its lock.
- **Current-state ruling**: the most complete "spatial-data visualization engine" requirements-list source; the MVP's GeoJSON/vector streaming is benchmarked against Cesium's same-class behavior (the Apache license permits read-and-learn).

## Sources
shields/npm measured (2026-09-03); the cesium README / 3d-tiles spec public.
