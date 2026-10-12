# GDAL / PROJ / GEOS — the geo-data trio (link C or go pure Rust)

**Snapshot 2026-09-03, measured**: OSGeo/gdal 6k★ (active yesterday); OSGeo/PROJ 2k★; libgeos/geos 1.5k★ (note: geos main development lives on OSGeo git, GitHub is a mirror). Licenses: **GDAL MIT, PROJ MIT/Centers for OGC, GEOS LGPL-2.1** — LGPL static linking carries obligations for SDK distribution (dynamic-link or avoid), **GEOS entering the core package is a license minefield**.

## Profile
- **GDAL**: unified raster/vector IO (200+ drivers) + coordinate reprojection pipeline + WMS/WMTS client + virtualization (COG/GPKG/Parquet geo-cloud formats) — 30 years of draining the format swamp.
- **PROJ**: the coordinate engine (full EPSG code library, pipeline expressions, time-dependent datums). **GEOS**: the JTS port of topological operations.
- Niche fact: PostGIS/QGIS/mapserver and even Cesium pipelines all rest on these three.

## For VisiaEngine
- **Adopt**: ① the API shape of GDAL's dataset/driver/virtual subsystems = the most mature "extensible IO layer" design (Visia's IO registry borrows directly, copying the semantics without linking C); ② the PROJ pipeline (+proj=pipeline expressions) = the **declarative composition** design of coordinate transforms; ③ COG (Cloud Optimized GeoTIFF) + HTTP range = the cloud format standard for raster streaming, supported directly by Visia's imagery layer.
- **Avoid/decide**: ① the pure-Rust path is health-checked (`evidence/2026-09-03-wgpu-direct` §2): projection = the proj binding (C dep, maintained, OK 185★ georust/proj 0.31, 2025-08) **or** proj4rs (weak license statement) — **there is no trusted pure-Rust full-EPSG piece**; Web Mercator+UTM in pure Rust (geozero/wgs84 class) is a viable start, full projection will eventually need a C dependency; ② **RK3588/Win7 cross-caution** (the legacy line): every C dependency adds compile/packaging cost on each legacy platform — the SDK's projection capability should be feature-gated, the core package not force-linking PROJ/GEOS; ③ WMS/WMTS has no mature pure-Rust piece (the geoserver ecosystem = server-side; a client is just hand-written HTTP, KISS).
- **Current-state ruling**: on format coverage the MVP promises only GeoJSON/glTF/(MVT decode via open-vector-tile, see the R4 thin risk in `evidence/2026-09-03-wgpu-direct`); GDAL's full capability stays in the enterprise-plugin tier (a natural line for the Open Core boundary — "heavy format support" is exactly a paid-plugin candidate).

## Sources
shields (2026-09-03); OSGeo official; georust crate metadata.
