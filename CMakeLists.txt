# Assembly layer: cmr / project / PolyOrch assembly / SDK resolution / options / add_subdirectory /
# install hooks (T-A 2026-10-10 adjudication 1-B+: build rules stay out of the root, except the ALIAS umbrella;
# soft line threshold 120 + assembly-growth exemption marker, check = scripts/gate-docs.sh ⑤). Design and rulings:
# .omo/plans/visiaengine-cmake-project.md + .omo/plans/2026-10-10-visia-embed.md
cmake_minimum_required(VERSION 3.25)  # PolyOrch embed floor (D39; adjudication 5-B)

# Version single source: the workspace Cargo.toml (do not copy iceoryx2's five-literal debt -- mismatch = hard configure error)
file(READ "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml" _ws)
string(REGEX MATCH "version = \"([0-9]+\\.[0-9]+\\.[0-9]+)\"" _m "${_ws}")
if(NOT _m)
  message(FATAL_ERROR "Failed to read the Cargo.toml workspace version (single-source contract broken)")
endif()
project(visiaengine VERSION ${CMAKE_MATCH_1} LANGUAGES CXX)

include(cmake/VisiaEngineSDK.cmake)
include(cmake/VisiaEnginePolyOrch.cmake)
visiaengine_ensure_polyorch()  # D39 sentinel reuse/self-embed (first-wins; if the superbuild added it first, reuse its copy)
visiaengine_ensure_polyorch_rust()  # T-B1: USE_PIXI translation -> polyorch_rust_setup(FROM pixi|system)
# R1 (2026-10-15): the SDK resolution moved behind the PolyOrch assembly -- its MSRV check
# delegates the comparison to polyorch_rust_version_ok (supplier FindRust), defined only once
# the sentinel's module list is loaded; the 0.2.0 floor FATAL above gates the copy before the call.
visiaengine_resolve_sdks() # report line: Rust/Qt resolution results and sources (auto|pixi|system|prebuilt)

include(cmake/VisiaEngineFfi.cmake)
visiaengine_setup_ffi()  # cargo step + the visiaengine::c link face + the fake in-tree Config bridge (T-C: capi engine = PolyOrch)
include(cmake/VisiaEngineCrates.cmake)
visiaengine_crates()  # T-B: workspace crate face through PolyOrch (geo + core, dependency edge visible)
include(cmake/VisiaEngineWasm.cmake)
visiaengine_wasm()     # T-G4: the js/wasm lane through PolyOrch (build/glue/test, dedicated cache)

# enable_testing() must run before ALL add_subdirectory calls (a late add_test in the Qt dir is silently dropped -- measured)
enable_testing()

# Install tree (packaging-round v1.3 P2; S-c beta guard retired here -- its
# three-line message moved into README's install section, P4 accounting).
include(cmake/VisiaEngineInstall.cmake)
visiaengine_install_tree()

include(cmake/VisiaEngineBindings.cmake)     # R9/R10: option + registration macros + the rs trio
if(VISIAENGINE_EXAMPLES AND NOT VISIAENGINE_ARTIFACT_PATH)
  # Prebuilt state = pure consumption, no cargo build surface: the examples domain bows out entirely (same contract as the old contract table; zombie empty-command targets = measured and blocked)
  enable_language(C)  # top-level scope (inside a function it does not propagate = PIT-16, measured); the C examples need real .c compilation
  add_subdirectory(examples/c)
  add_subdirectory(examples/cpp)
endif()

if(VISIAENGINE_QT)
  add_subdirectory(examples/qt) # Qt host-only band (the binding-side widget.hpp lives in bindings/qt, header-only)
endif()

if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/bindings/c/probe/CMakeLists.txt")
  add_subdirectory(bindings/c/probe) # ctest probe (auto-skips until C2 lands)
endif()

visiaengine_setup_bindings()  # rs trio + global steps + the central disk<->table final audit (must run after ALL add_subdirectory calls)

include(cmake/VisiaEnginePackaging.cmake)  # CPack assembly: tarball = the install tree as-is (no new install rules)
