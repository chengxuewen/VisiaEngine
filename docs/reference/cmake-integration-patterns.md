# cmake × Rust integration shapes compared (iceoryx2 / slint / zenoh / MediaServo)

> Source: the team's 4 probes (2026-09-14, read-only forensics on the `.refinfo/` local archive; every file:line is re-verifiable).
> Serves the decision: VisiaEngine's CMake integration selection (the Qt-widget round's consumption surface + the packaging round's `bindings/` design).

## Lineage (sorted by cargo<->cmake coupling)

| project | mechanism | artifact->target mapping | headers | core lesson |
|---|---|---|---|---|
| **zenoh** (core repo) | **zero cmake** — the binding surface is fully externalized to sibling repos (zenoh-c/-python/…), handoff point = crates.io publication (release.yml:86-94) | n/a | no C header (the Rust API + feature gates are the contract, the `internal` gate = the binding escape hatch zenoh/Cargo.toml:47) | "no cmake in the core repo" can be a **design, not a gap**; repo boundary = version contract, not path coupling |
| **MediaServo** (prior work) | zero build-time integration: cargo driven by scripts, **renders the Config.cmake.in template at packaging time** (python substitutes @VERSION@ etc.) | one INTERFACE IMPORTED per component, the WIN32/APPLE/UNIX name mapping **lives only in the template** (mediaservoConfig.cmake.in:9-18) | hand-written (cbindgen abandoned, on record) + abi-drift external gate | **the biggest cautionary tale: the find_package path was never compiled by any test** — a cmake package = dark code that ships; SONAME symlink shim, manual LD_LIBRARY_PATH, .so overwrite ETXTBSY |
| **iceoryx2** | CMake manages cargo itself: `add_custom_target(ALL COMMAND cargo build …)` (top-level CMakeLists:190-201) + **escape hatch** `-DRUST_BUILD_ARTIFACT_PATH=` (prebuilt pure-consumption mode :153) | non-IMPORTED — an INTERFACE library + generator expressions stuffing absolute paths (iceoryx2-c/CMakeLists.txt:62-78); the comment confesses: the IMPORTED approach "all tried, none work" :56-60 | cbindgen in build.rs, path = OUT_DIR reverse string (byproduct-defs:13) | the two-state config trick (in-tree fake Config + FORCE _DIR / generated version after install) makes in-tree and installed consumption code **identical**; weaknesses: ordering fragility, no SOVERSION, DLLs installed into lib/ |
| **slint** | **Corrosion** (find_package -> FetchContent v0.6.1 fallback, api/cpp/CMakeLists:16-26) — one `corrosion_import_crate` call spins up everything | corrosion creates `slint_cpp-shared/-static` IMPORTED per crate-type, **per-OS naming / build-type->profile mapping with zero hand-written code** | a 1383-line cbindgen generator (only needed for a large FFI surface) | consumption converges on a single `Slint::Slint` INTERFACE umbrella; the example formula `if(NOT TARGET …) find_package(…)`; the cost = 250 of 531 lines are Android/Yocto/Skia special handling, **do not copy** |

## Qt-consumption precedent (unique to slint, zero precedent in MediaServo)

- `examples/cpp/platform_qt/main.cpp:44-73`: `winId()` -> per-platform `NativeWindowHandle{from_x11_xlib/win32/appkit}` — **isomorphic with our `visiaengine_attach(win,display,kind)`**, proving the integer-window-handle + kind-parameter route is canonical.
- Pump: `paintEvent->render()`, `request_redraw->requestUpdate()` (same file :202-211) + QTimer — = our design, no frame-callback push.
- `qt_viewer` (widget embedding) and `iot-dashboard` (zero-codegen pure umbrella link) = the two demo-consumption CMakeLists template styles.

## Landing for VisiaEngine (adjudication draft)

1. **Repo shape**: take the zenoh argument but compromise at our scale — the ABI implementation stays in `crates/visiaengine-ffi`, the distribution surface `bindings/` is reserved; **pre-1.0 do not copy its crates.io handoff** (workspace path + the gate-abi/CAPI-09 mirror gate = our version-contract stand-in).
2. **Qt-round consumption integration**: v0 uses the iceoryx2-style **thin self-management** (`add_custom_target cargo build --package visiaengine-ffi --target-dir <cmake-bin>/rust` + the `RUST_BUILD_ARTIFACT_PATH` escape hatch + the INTERFACE umbrella `visiaengine::capi` + BUILD/INSTALL double genex), **do not introduce corrosion** (the FetchContent network surface conflicts with the pixi/D5 offline discipline; corrosion graduating = a packaging-round agenda item, trigger = a second host language appears).
3. **Hard-gate inheritance** (the shared debt of all three cases): the find_package path **must have a real consumer compile smoke** (the Qt demo doubles up; into the ci segment) — MediaServo's headliner lesson.
4. **Zero-change gains**: add `links = "visiaengine_c"` to the capi Cargo.toml (duplicate-static-link foolproof, the slint one-liner); the header contract comments (owner thread/close invalidation) keep the current hand-written discipline.
5. **SKIP list**: COMPONENTS multi-component (a single library derives none), SOVERSION install trees, versioned include prefixes, cbindgen, dual .pc publishing (revisit at the packaging round), SONAME dev-symlink (v0 links the target artifact directly, so the problem never arises).
