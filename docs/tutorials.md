# Tutorial index (the E-numbered series)

> Single-source discipline [E3D:D1]: **file name = header comment = this index**, the three-way agreement is machine-locked by `scripts/gate-docs.sh`.
> Empty numbers = reserved insertion slots. Every example runs directly after `source pixi.sh`; the "related clauses" column points into `docs/sdd/`.

> **Quad-consumer policy (B5, 2026-09-24)**: every new example band delivers FOUR consumers of one artifact —
> (1) example source, (2) a row in this index with prose, (3) a CI-runnable test (ctest / smoke argv),
> (4) a wall entry (one line in `examples/wasm/wall.json`, the manifest the web wall renders from).
> Machine locks: gate-docs ⑧ (wall manifest == disk registry set) + ⑦ (README count). Adding an
> example without registering it breaks the build (R9 anti-glob); forgetting the wall.json line breaks gate-docs.

| No. | Theme | File | Related clauses | How to run |
|---|---|---|---|---|
| E101 | Window and first frame (winit×wgpu clear-color loop) | `examples/rs/E101_clear.rs` | REND-01..08 / WGPU-01.. | `pixi run smoke-clear` |
| E201 | Data loading · glTF (assembly + orbit camera; the first of the 4-move combo) | `examples/rs/E201_load_gltf.rs` | GLTF-01..11 / WGPU-14 | `pixi run smoke-load-gltf` |
| E202 | Data loading · GeoJSON (parse/project/tessellate/style confluence) | `examples/rs/E202_geo_viewer.rs` | GEO-01..24 / D7 | `pixi run smoke-geo-viewer` |
| E203 | Properties and measurement (planar 3857 distance/area CLI) | `examples/rs/E203_measure_cli.rs` | GEO-21/22 | `pixi run smoke-measure` |
| E204 | Data loading · point cloud (spiral self-assert dual-mode + `--file` PLY load route + `--pick-check` point-pick three asserts B2 + `--edl` EDL outline WGPU-33) | `examples/rs/E204_pcl_viewer.rs` | IO-01..06 / CAPI-18/19 / WGPU-18 / REND-38/39 | `pixi run smoke-pcl-viewer` |
| E205 | Annotation text · labels dual-mode (io-text full-chain live example; zero-arg resident window / `--frames` white-ink asserts) | `examples/rs/E205_text_labels.rs` | IO-07..09 / REND-33 / WGPU-24/25 | `pixi run smoke-text-labels` |
| E206 | Data loading · vector tiles (MVT decode -> GeoTile -> three render families; FileSource loading of bundled synthetic tiles; `--source URL` = live proof of HTTP pump loading) | `examples/rs/E206_tile_viewer.rs` | IO-10..12/16 / GEO-27 | `pixi run smoke-tile-viewer` |
| E301 | Camera and view (stepless perspective<->orthographic switching) | `examples/rs/E301_switch_camera.rs` | REND-10..17 | `pixi run smoke-switch-camera` |
| E302 | Camera flight · preset tour (dual-mode: 1/2/3 fly between views / drag-scroll interrupts = flight cancel; `--frames` endpoints + shortest arc + render three asserts) | `examples/rs/E302_fly_camera.rs` | REND-34 / CAPI-23 | `pixi run smoke-fly-camera` |
| E303 | Split-screen cockpit · main perspective + top-view minimap (dual-mode: 1/2/3 flight with the small window following / drag-scroll takes over; `--frames` four per-region asserts) | `examples/rs/E303_split_screen.rs` | REND-35 / WGPU-26 | `pixi run smoke-split-screen` |
| E304 | Projection morph · continuous 2D<->3D transition (window `[/]` slides t with the title echoing it; `--frames` endpoint byte-equality + mid-path non-degeneracy) | `examples/rs/E304_projection_morph.rs` | REND-41 | `cargo run --example E304_projection_morph -- --frames 4` |
| E305 | Entity animation · trajectory replay (keyframe sampling anim_origin; `--frames` = trajectory density, endpoint-law asserts) | `examples/rs/E305_entity_anim.rs` | REND-42 | `cargo run --example E305_entity_anim -- --frames 24` |
| E403 | Interaction · rubber-band box select (drag = select all inside the box, sampled-ray predicate; right-click/R clears) | `examples/rs/E403_box_select.rs` | REND-23/38 | `pixi run smoke-box-select` |
| E306 | Camera · map-control preset (the pan/zoom/tilt trio, starting from the orthographic top view; `--frames` semantic asserts) | `examples/rs/E306_map_controls.rs` | REND-29/12 | `cargo run --example E306_map_controls -- --frames 1` |
| E401 | Interaction · pick highlight (ray -> pick -> CPU overwrite closed loop) | `examples/rs/E401_pick_demo.rs` | REND-21..24 / WGPU-12/13 | `pixi run smoke-pick` |
| E402 | Interaction · hover/multi-select window (hover = orange preview, click = yellow multi-select toggle, drag = orbit, R = clear) | `examples/rs/E402_pick_interactive.rs` | REND-21/23 / WGPU-12 | `pixi run smoke-pick-hover` |
| E501 | Materials and light/shadow (instanced city + PCSS soft shadows; dual-mode = zero-arg interactive window / `--frames` headless) | `examples/rs/E501_shadow_demo.rs` | REND-31 / WGPU-14..20 | `pixi run smoke-shadow-demo` |
| E502 | Materials and light/shadow · color calibration (the sRGB full-chain round trip's "what you see is what you get" live proof; four palette boards headless) | `examples/rs/E502_color_tuning.rs` | CORE-16 / WGPU-14 | `pixi run smoke-color-tuning` (in-example self-asserts) |
| E503 | Materials and light/shadow · the expansion families' own lesson (constant screen line width / true round points · a zoom show) | `examples/rs/E503_stroke_points.rs` | REND-29/30 / WGPU-17/18 | `ctest -L display` (the cmake-smoke xvfb sub-state) |
| E504 | Glass and water · transparency live example (dual-mode: 1/2 = water alpha material re-upload; `--frames` water-film attenuation band + always-top label asserts, incl. the a=1.0 control semantic lock) | `examples/rs/E504_glass_water.rs` | REND-36 / WGPU-27/28 | `pixi run smoke-glass-water` |
| E505 | CJK Chinese labels (a Noto Sans SC subset font · the same entry the host injects fonts through; `--frames` white-glyph pixel gate) | `examples/rs/E505_cjk_labels.rs` | IO-07..09 / CAPI-21 | `cargo run --example E505_cjk_labels -- --frames 1` |
| E506 | Post framework: bloom/outline chain (1/2=toggle, 3=all off; `--frames` off-vs-bloom/off-vs-outline pixel asserts + off determinism canary) | `examples/rs/E506_postprocessing.rs` | REND-43 / WGPU-34 | `pixi run smoke-post` |
| E507 | HDR + tonemap: first N5 consumer (1=off 2=reinhard 3=aces 0=bloom; `--frames` off-vs-reinhard/off-vs-aces/compose pixel asserts + fixture HDR proof; io-hdr RGBE loader IO-17) | `examples/rs/E507_hdr_tonemap.rs` | REND-44 / WGPU-34 / IO-17 | `pixi run smoke-hdr` |
| E508 | PBR materials: first N6 GGX consumer (drag=orbit wheel=zoom resident window; `--frames` metallic-vs-dielectric diff + roughness ladder peak/spread probe-pinned asserts) | `examples/rs/E508_pbr_materials.rs` | REND-45 / WGPU-35 | `pixi run smoke-pbr` |
| E509 | SSAO ambient occlusion: depth-only 16-tap AO (1=toggle 2=int+ 3=int- 0=off; drag=orbit wheel=zoom resident window; `--frames` off-vs-on crease-band pixel assert) | `examples/rs/E509_ssao.rs` | WGPU-36 / REND-43 | `pixi run smoke-ssao` |
| E510 | Parametric curves · route flow (CatmullRom ribbon + tube/ribbon guardrails; zero-arg = resident window, drag orbits / wheel zooms / flow markers; `--frames` = endpoint law + flow-density pixel asserts) | `examples/rs/E510_route_flow.rs` | REND-47 | `pixi run smoke-curve` |
| E511 | Rigid node animation (two door panels rotating in the transform plane; zero-arg = resident window, θ cosine swinging doors, drag orbits / wheel near-far / Esc quits; `--frames` = rotation pixel diff + static-leaf canary asserts) | `examples/rs/E511_gate.rs` | CAPI-42/43 | `pixi run smoke-gate` |
| E601 | Scale and performance (the 100k-building-block single-draw stress example) | `examples/rs/E601_bench_twin.rs` | REND-27/28 / WGPU-16 / [6b] | `pixi run smoke-bench-twin` (`pixi run bench` emits the artifact) |
| E701 | Binding mirror · C headless (embedding boilerplate + attribute-read closed loop name/opacity/missing≠0; if it does not run, the API is not finished [E3D:D7]) | `examples/c/E701_demo_headless.c` | CAPI-01..08 | `bash scripts/gate-abi.sh` |
| E702 | Binding mirror · C X11 (attach real window, dual-mode = zero-arg resident / `--frames` quick exit) | `examples/c/E702_demo_x11.c` | CAPI-06 | `bash scripts/smoke-x11.sh` |
| E703 | Binding mirror · Qt real window, two generations (Qt 5.15/6; the widget-host band; the texquad+park two families) | `examples/qt/E703_qt_viewer.cpp` | CAPI-06 | `pixi run smoke-qt` |
| E704 | Binding mirror · C event callback (the CAPI-17 push port's three states: monotonic progress / error at the same instant / NULL detaches) | `examples/c/E704_host_callback.c` | CAPI-17 | `ctest` (the native headless family) |
| E801 | SDK consumption · C+SDL3 real window (dual-mode = zero-arg interactive window with resize linkage / `--frames` quick exit) | `examples/c/E801_sdl_window.c` | CAPI-06 | `ctest -L display` (the cmake-smoke xvfb sub-state) |
| E810 | SDK consumption · C attribute traversal survey (name/opacity/missing≠zero-value on display) | `examples/c/E810_attr_survey.c` | CAPI-04/10/11 | `ctest -R example_E810` |
| E811 | SDK consumption · visibility twin (the C face; enumeration-domain invariance / query round trip / out-of-value rejection) | `examples/c/E811_entity_hide_c.c` | CAPI-13/14 | `ctest -R example_E811_entity_hide_c` |
| E811 | SDK consumption · visibility twin (the C++ face; the hpp forwarder acceptance) | `examples/cpp/E811_entity_hide_cpp.cpp` | CAPI-13/14 | `ctest -R example_E811_entity_hide_cpp` |
| E812 | SDK consumption · procedural add/remove (distinct configurations / degenerate zero commit / re-entry rejected) | `examples/cpp/E812_mesh_add.cpp` | CAPI-15/16 | `ctest -R example_E812` |
| E802 | SDK consumption · C++ headless (the hpp facade's live acceptance; readback -> PPM on disk) | `examples/cpp/E802_offscreen.cpp` | CAPI-04/05 | `ctest -R example_E802` |
| E814 | SDK consumption · C text labels (timing gate / utf8 multi-byte / white-ink live gates) | `examples/c/E814_labels_headless.c` | CAPI-21/22 | `ctest -R example_E814_labels_headless` |
| E815 | SDK consumption · C camera flight (idle-is-done / teleport / monotonic progress / cancel / domain-reject live gates) | `examples/c/E815_fly_camera.c` | CAPI-23/24 | `ctest -R example_E815_fly_camera` |
| E816 | SDK consumption · C minimap navigation (canary byte-for-byte / the corner top-view family / minimap pick / navigation arriving with angle and distance preserved / value-domain reject — five live-gate segments) | `examples/c/E816_minimap_nav.c` | CAPI-25..27 | `ctest -R example_E816_minimap_nav` |
| E817 | SDK consumption · C two-layer map (raster basemap and vector tiles in one frame: empty-frame canary / no leakage between the single families / both families in one frame under both mount orders / re-mount of the same kind byte-unchanged — five live-gate segments) | `examples/c/E817_two_layer_map.c` | CAPI-28/29/39/40 (the band-T revision sentences) | `ctest -R example_E817_two_layer_map` |
| E813 | SDK consumption · cross-section clipping (half cut / three-plane AND at a corner / value-domain reject / clear restores, three readback segments as live gates) | `examples/c/E813_section_clip.c` | CAPI-20 | `ctest -R example_E813_section_clip` |
| E901 | Vertical-slice seed · twin city (geo basemap × instanced × PCSS; the interactive window carries the C clip knife / [/] knife height / L labels; `--frames` offscreen PNG + asserts) | `examples/rs/E901_twin_city.rs` | the whole batch-4 deliverable surface + REND-32/33 live-key demos | `pixi run smoke-twin-city` (new in T2) |

## IDE running (the CMake target face)

After `pixi run cmake --preset default` (or open the project in an IDE and pick the default/qt preset; CI uses ci-strict):

- **The Rust E family (examples/rs) comes as a trio**: PolyOrch build handle `visiaengine-example-<E#>` (mediator
  `visiaengine-example-<E#>-build`, the D40 EXAMPLE kind -- the cargo call lives in the supplier wrapper),
  run step `visiaengine-example-<E#>-run` (builds the handle first, then polyorch_example_run executes the staged binary with the
  DISPLAY fallback), ctest entry `example_<E#>` (runs the staged binary directly, no cargo at test time) —
  the single home of argv is the registry in `cmake/VisiaEngineBindings.cmake`, and the pixi `smoke-*` tasks
  are all `ctest -R` forwarding shells. The cargo environment's self-sufficiency is locked by the cmake-smoke state-④ triple anchor (bare PATH =
  IDE calls need no pixi activation); per-example breakpoint debugging runs through rust-analyzer Run/Debug,
  and with the `debug` preset on, F5 rows ride the PolyOrch managed lane instead (`visiaengine-example-<E#>` launch + preLaunchTask pairs).
- **The native E family is the real thing (examples/{c,cpp,qt})**: add_executable compiles what you change, one at a time,
  ctest entries from the same source, FOLDER = the disk-directory mirror (examples/<language> auto-derived; root-level steps sit bare at the top), identity semantics belong to LABELS.
- **ctest**: the headless family always truly runs; the default testPreset already skips the display family wholesale via `-LE display`,
  and the true run goes through the xvfb sub-state (S4 joins the cmake-smoke display band).

New examples must register in the same round: Rust example = a registry line in Bindings.cmake + an
`[[example]]` block in `examples/rs`; C/C++ example = a `visiaengine_add_example()` line in the corresponding
`examples/<lang>/CMakeLists.txt` (the anti-glob central ledger makes configure a hard error; comment-style disabling is forbidden).
E703 (the Qt host band) exists under the ci-strict preset (or -DVISIAENGINE_QT=ON).

## Human-verification form (the dual-mode convention · one entry for IDE manual runs and automated tests)

Every example splits through the unified argv dual mode: the IDE's `visiaengine-example-*-run` (rs) / `run_*` (native) targets **start with zero arguments = the human-verification form**;
ctest / `--frames N` = the automated quick-exit form (the argv single-source discipline: registry parameters serve only ctest, the run steps take zero args — C15).

| Human-verification form | Examples | Zero-arg startup behavior | What to look at |
|---|---|---|---|
| **Window stays up** | E101 / E201 / E202 / E205 / E301 / E302 / E303 / E304 / E306 / E402 / E403 / E501 / E502 / E503 / E504 / E506 / E507 / E508 / E509 / E702 / E801 / E901 | opens an interactive window, never auto-exits (Esc/close to exit) | verify the picture by eye: E502's four palette boards = the color chain, E304 `[/]` slides the projection morph (the title echoes t); for the rest the title itself is the operation hint |
| **Self-exiting self-proving** | E203 / E305 / E401 / E601 / E701 / E704 / E802 / E810 / E811_c / E811_cpp / E812 / E813 / E814 / E815 / E816 / E817 | runs, prints an `OK …` line, exits | the terminal output is the acceptance surface (E305 `--frames N` density = the trajectory marker count) |

**The headless evidence examples' "exits in seconds = designed"**: their artifacts are terminal assertion lines or PNG/PPM files on disk, not interactive windows; for interactive acceptance use the window example of the same capability domain (pick→E402, color→E502 zero-arg window, lighting→E501 zero-arg window, constant width→E503). The native run steps cover every example (the DISPLAY family
through the GUI run buttons' `:0` probe fallback; the headless family runs directly, fine in an IDE without DISPLAY).

Native-example window classification (the census of the same round as the 2026-09-17 E702 fix): real window examples = E702/E801/E703, three (zero-arg all stay resident);
the remaining C/C++ examples (E701/E704/E802/E810/E811×2/E812/E813/E814) = headless evidence examples, whose human-verification face is the terminal `OK …` line / PPM file on disk
— exiting in seconds is their designed identity.

## Installed SDK consumption (install tree, packaging-round v1.3 P2)

The in-tree lanes above build from source (`add_subdirectory`). Since P2 the repo
also produces an install tree that a consumer can use without the Rust toolchain.
Verified on this machine (Linux); commands and their measured output:

```bash
# produce a tree
pixi run cmake -S . -B build/sdk -G Ninja -DCMAKE_BUILD_TYPE=Release
pixi run cmake --install build/sdk --prefix /opt/visia       # 9 files
pixi run pack-check                                          # hygiene gate on the tree

# CMake consumer
find_package(visiaengine REQUIRED)        # name = project() name, lowercase
target_link_libraries(app PRIVATE visiaengine::c)   # or visiaengine::cpp
# pkg-config consumer (plain C)
export PKG_CONFIG_PATH=/opt/visia/lib/pkgconfig
cc app.c $(pkg-config --cflags --libs visiaengine) -o app
```

Tree layout (header names are identical to the in-tree spellings, so the same
source compiles in both states): `lib/libvisiaengine.so`, `include/visiaengine.h`,
`include/visiaengine/visiaengine.hpp`, `include/visiaengine_widget.hpp`,
`lib/cmake/visiaengine/{visiaengineConfig,visiaengineConfigVersion}.cmake`,
`lib/pkgconfig/visiaengine.pc`, `share/licenses/visiaengine/LICENSE-{MIT,APACHE}`.

Notes that are promises, not caveats:
* The library carries a SONAME, so consumers record the bare `libvisiaengine.so`
  instead of an absolute path; both the CMake config and the `.pc` derive the
  prefix from their own location, so moving the tree only requires re-running
  the consumer's configure step (probed: installed at prefix A, rebuilt against
  prefix A moved to B, runs).
* `visiaengine_widget.hpp` ships as a header only. It includes `<QWidget>` and
  `<X11/Xlib.h>`; a Qt host resolves those itself (see `examples/qt`), which is
  exactly what the installed tree expects. No Qt/X11 dependency is declared on
  the consumer's behalf.
* Not shipped by design: the static archive (no consumer, and the system-library
  closure would be a hand-maintained second truth), and any npm/pip packaging
  (Alpha ledger). macOS/Windows install forms are declared gaps.
