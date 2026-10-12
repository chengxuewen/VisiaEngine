# VisiaEngine

> **A multi-dimensional spatial visualization engine** — a unified 2D / 2.5D / 3D rendering pipeline, built to "see the essence of data."

**Status: whitepaper v0.1.0 finalized + Phase 1 MVP complete + batch 4 rendering hardening + batch 5 documentation rehoming (2026-09).** Full positioning: [docs/whitepaper.md](docs/whitepaper.md).

## What It Is

An open-source, lightweight, embeddable spatial visualization **engine kernel** (not a game engine):

- Unified multi-dimensional viewport: top-down map (2D), tilted view (2.5D), immersive 3D free switching (currently a flat entity table; scene tree = Alpha-tier candidate), with consistent interaction semantics for layers/entities/annotations
- **Spatial data as first-class citizens**: GeoJSON, mainstream coordinate projections (built in); tiled-vector MVT (directory/HTTP sources delivered); raster tiles (PNG tiled basemap delivered); **tiled vectors and raster basemaps stacked under one handle** delivered (each of the two mount orders has its own live pixel gate, E817); WMS/WMTS serving (in progress, see whitepaper §3.2 roadmap); glTF/point-cloud/BIM lightweight display; ODR/OSC via official simulation plugins
- **Rust kernel + wgpu rendering**: memory safety, no GC; first-class Vulkan / Metal / DX12 / WebGPU, with GL 3.3+ / GLES 3.0+ / WebGL2 fallback tier
- **SDK form factor**: embed into Qt / Flutter / C# (WPF/Unity) / Web via the C API without hijacking the host main loop; startup size (native cdylib, stripped) measured 7.25 MB ≤ 10 MB (evidence: docs/reference/evidence/2026-09-24-native-size.md)

## Architecture (One Sentence)

`visiaengine-core` (data model / spatial index / coordinate systems / scene graph) → `visiaengine-render` (rendering abstraction traits) → `visiaengine-render-wgpu` (default backend: a from-scratch pipeline on wgpu, settled by D4; backend abstraction kept pluggable). Hosts depend only on the C API; backend choice never breaks the compatibility promise.

## Repository Structure

```
├── AGENTS.md / SKILL.md   # agent knowledge base and skill registry
├── crates/                # 5 pure-core crates: core / render / render-wgpu / io-gltf / geo (D6 naming)
├── examples/              # examples tiered by language: rs (Rust tutorial series) / c / cpp / qt — one-click via the ctest unified registry
├── bindings/              # outward implementation surface: c (capi crate + handwritten header + probe) / cpp (RAII header) / qt (widget) / js (wasm+demo)
├── pixi.toml / pixi.lock  # dev environment single source (D5: conda-forge, incl. rust toolchain)
├── docs/sdd/              # behavioral contract clauses (bidirectionally traced to tests)
├── docs/
│   ├── whitepaper.md      # whitepaper v0.1.0 (primary source for positioning and business strategy)
│   ├── architecture.md    # architecture design baseline v0.1 (design entry after the D4 ruling)
│   └── reference/         # open-source reference library + decision evidence snapshots (evidence/)
└── .agents/               # project memory / rules / skills (agent toolchain)
```

## Development Status

**Phase 1 status (numbers as reported by the gates)**: **11 crates** (9 in `crates/` + capi + wasm; workspace has 12 packages incl. the examples shell package), **199** SDD behavioral contract clauses all green (`scripts/spec-trace.sh` bidirectional tracing; `gate-docs` locks these numbers to live reports). Rendering pipeline: glTF/GeoJSON/point-cloud(PLY)/tiled-vector (MVT directory/HTTP source mounting + viewport scheduling + pump loading) + material textures (GGX PBR, WGPU-35/REND-45) + GPU instancing (100k building blocks in one draw) + screen-space line width / true-round dot expansion slice + translucent two-pass painter's ordering + directional-light PCSS soft shadows + point-cloud EDL rim (`` `--edl` ``, WGPU-33) + postprocessing framework (bloom/outline/tonemap/SSAO/depth-haze, WGPU-34/36/38, REND-43/44) + HDRI loading (.hdr RGBE, IO-17) + SH-9 environment lighting (demo grade, WGPU-39/CAPI-41) + text annotation (incl. CJK subset fonts) + profile clipping + main/aux dual-projection minimap / click navigation + continuous 2D↔3D projection morph + picking (incl. point cloud REND-38 + line-feature screen space REND-46) / parametric-curve tube/ribbon (REND-47) / measurement loop + D7 far-coordinate rebase (pixel-verified). Offscreen golden: no SKIP on real hardware; **ctest unified example registry (44 entries, incl. the SDL3 window family xvfb substate) + three gates (abi/trace/docs) + cmake-smoke four states (incl. installed tree) + pack-check installed-tree hygiene + bench artifact chain** as machine gates; the tutorial E-number system lives in [docs/tutorials.md](docs/tutorials.md) (filename = header comment = index, single source).  Example gallery (44 cards, thumbnails self-rendered by the examples): `pixi run gallery` → `build/gallery/index.html`. **Host embedding**: C ABI 49 entry points (+5 scene-tree ports + HTTP/raster tile sources + 3 property hot-update ports + .hdr environment + rigid-body node transforms) (event push / point cloud / profile / font annotation / flight / minimap navigation, mirrored three ways in wasm·hpp·header) (human-reviewed handwritten header + C demo fleet actually run) + Web surface (visiaengine-wasm dual mirror, runnable demo page) — proven on a Qt host ✅ (header-only widget + real-window smoke, three states); **install tree delivered** (`cmake --install` produces 9 items: library + three headers + licenses + `find_package(visiaengine)` how-to page + pkg-config receipt; out-of-tree consumers compiled and run in both C/C++ forms, re-prefix rebuild passes, the library ships its own SONAME so `NEEDED` records the bare name); npm/pip packaging deferred to Alpha.

```bash
bash bootstrap.sh && source pixi.sh && pixi run ci   # environment, three steps
pixi run cmake --preset default && pixi run cmake --build --preset default   # C++/IDE entry (root CMake facade)

> Output territories: `build/` = gallery-only · `target/` = cargo-direct only · `cmake-build*/` = CMake preset build trees (T-A0' 2026-10-10). VSCode: install CMake Tools and work via presets (cmake.useCMakePresets=already configured).
# Opening the project in an IDE gives: test panel = the unified example registry for all languages (ctest --preset default); the build menu carries per-example cargo-build_*/cargo-run_* steps; Rust examples additionally have rust-analyzer Run/Debug
```

## Business Model

Open Core: the core runtime is **permanently open source under MIT OR Apache-2.0** (the license of a released version is irrevocable); the commercial products are the **Visia Studio** editor and enterprise plugins/support.

## Roadmap

| Phase | Content |
|------|------|
| MVP (current) | core library skeleton, glTF+GeoJSON loading, 2D↔3D switching demo, host-embedding examples |
| Alpha | spatial index / coordinate system maturation, 2.5D white-model, C API stabilization + binding generation |
| Beta | Visia Studio preview, ODR/OSC plugins, real-time digital-twin data binding |
| 1.0 | editor+runtime official release, plugin marketplace, commercial support launch |

## License

The core runtime is dual-licensed open source under **MIT OR Apache-2.0** (texts at [LICENSE-MIT](LICENSE-MIT) / [LICENSE-APACHE](LICENSE-APACHE)). The license of a released version is **irrevocable and never retroactively changed**. Visia Studio and enterprise plugins are licensed separately, commercially.

## Contact

- Repo: gitee.com/chengxuewen/VisiaEngine
- Website / email: TBD
