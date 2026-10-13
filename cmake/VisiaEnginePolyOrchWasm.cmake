# PolyOrch wasm lane for VisiaEngine (T-G4): the js binding's build engine,
# expressed as PolyOrch public-function calls only (the user-ruled
# no-invented-features constraint). The ci chain (scripts/web-build.sh:
# WEB-SIZE print, node mirror test, wasm-opt accounting) stays the
# measurement lane -- this file adds the IDE/graph face next to it.
#
# Facts pinned by the T-G2 probe round (2026-10-12):
#  * triple injection rides POLYORCH_RUST_CARGO_TARGET (the setup-consumed
#    knob; polyorch_rust_build has no triple keyword);
#  * the D29 naming table normalizes dashes -> underscores for the wasm
#    module (visiaengine_wasm.wasm);
#  * polyorch_rust_test on a wasm triple routes to the HOST (the supplier's
#    probe carve-out): the #[ignore] web_mirror leg stays ignored here and
#    runs in the ci chain where the glue exists;
#  * the glue depends on the wasm-bindgen CLI matching the crate pin
#    (0.2.127, J1) -- the version lock-compare in tool_bootstrap enforces
#    it; a mismatched local install is DEMOTED, not used.
#
# No wasm->capi edge: the wasm32 build compiles the capi path-dep itself,
# for wasm32, inside the dedicated .cargo-target-wasm dir; the host capi
# handles are different-triple artifacts and an edge would force pointless
# host builds (adjudicated in the plan, Momus-reviewed).
#
# FOLDER map (position-mirror): every user-facing button lives under the
# crate's on-disk directory; the face aggregate is FOLDER-less (hidden,
# rlibs precedent).

option(VISIAENGINE_POLYORCH_WASM "Register PolyOrch-driven targets for the wasm/js lane" ON)

function(visiaengine_polyorch_wasm)
  if(NOT VISIAENGINE_POLYORCH_WASM)
    return()
  endif()
  if(NOT COMMAND polyorch_rust_build OR NOT POLYORCH_RUST_FOUND)
    return()
  endif()

  set(_src "${CMAKE_SOURCE_DIR}/bindings/wasm/visiaengine-wasm")
  set(_manifest "${_src}/Cargo.toml")
  if(NOT EXISTS "${_manifest}")
    message(STATUS "visiaengine wasm lane: no manifest, off")
    return()
  endif()
  if(NOT POLYORCH_RUST_CARGO_TARGET)
    # this function is only reached from the host graph; the wasm lane needs
    # the knob set for its build -- but the knob is global per-directory
    # scope, so the lane registers inside its own directory scope by
    # convention (the root calls us AFTER the crates face, where the knob
    # is still empty). Set + restore around the calls that consume it.
    set(POLYORCH_RUST_CARGO_TARGET wasm32-unknown-unknown)
  endif()

  set(_wasm_cache "${CMAKE_BINARY_DIR}/.cargo-target-wasm")
  set(_fold "bindings/wasm/visiaengine-wasm")

  # one dedicated cache dir for the wasm32 lane (clean is rm -rf BASE_DIR --
  # mixing into a host cache would make any clean wipe host artifacts)
  polyorch_rust_build(TARGET visiaengine-wasm
      PACKAGE visiaengine-wasm CRATE visiaengine-wasm SHARED
      MANIFEST "${_manifest}"
      BASE_DIR "${_wasm_cache}"
      FOLDER "${_fold}")
  polyorch_rust_clean(
      NAME visiaengine-wasm-clean
      BASE_DIR "${_wasm_cache}"
      FOLDER "${_fold}")

  # host-side test button (wasm triples test on the host; the ignored
  # mirror leg stays ignored -- it is the ci chain's job, where glue exists)
  polyorch_rust_test(
      PACKAGE visiaengine-wasm
      NAME    visiaengine-wasm-test
      MANIFEST "${_manifest}"
      BASE_DIR "${_wasm_cache}"
      FOLDER "${_fold}")

  # js glue: web (the demo page) + nodejs (the mirror harness's flavor),
  # typescript decls, the version-locked tool, wasm-opt when present.
  # OUT_DIR lands under the build tree's web/ face for easy serving.
  # Tool resolution: the version lock-compare demands CLI == crate pin
  # (0.2.127). The pixi wasm-spike env carries the pinned pair -- pass it
  # as PREFIX when present (the caller-parameter form of the pixi-env
  # probe order; discovery stays prefix-scoped when PREFIX is given).
  set(_bindgen_prefix "")
  file(GLOB _pixi_wbg "$ENV{HOME}/.pixi/envs/*/bin/wasm-bindgen"
                      "${CMAKE_SOURCE_DIR}/.pixi/envs/*/bin/wasm-bindgen")
  foreach(_wbg ${_pixi_wbg})
    execute_process(COMMAND "${_wbg}" --version
        OUTPUT_VARIABLE _wbg_v ERROR_QUIET OUTPUT_STRIP_TRAILING_WHITESPACE)
    # banner shape "wasm-bindgen 0.2.127": first-triple extraction (the
    # bootstrap's own lock-compare convention), not an exact-string match
    set(_wbg_tri "")
    if(_wbg_v MATCHES "([0-9]+\\.[0-9]+\\.[0-9]+)")
      set(_wbg_tri "${CMAKE_MATCH_1}")
    endif()
    if(_wbg_tri STREQUAL "0.2.127")
      get_filename_component(_bindgen_prefix "${_wbg}" DIRECTORY)
      get_filename_component(_bindgen_prefix "${_bindgen_prefix}" DIRECTORY)
      break()
    endif()
  endforeach()
  set(_bgpkw "")
  if(_bindgen_prefix)
    set(_bgpkw PREFIX "${_bindgen_prefix}")
  endif()
  polyorch_rust_wasm_bindgen(TARGET visiaengine-wasm
      NAME_PREFIX visiaengine-wasm
      FLAVOR web nodejs
      TYPESCRIPT
      OUT_DIR "${CMAKE_BINARY_DIR}/web"
      WASM_BINDGEN_VERSION 0.2.127
      ${_bgpkw}
      WASM_OPT AUTO
      FOLDER "${_fold}")

  # the demo/serve button: assemble web-demo/ (page + park data + the
  # web flavor as pkg/) and serve :8017 -- the scripts/web-serve.sh lane
  # retired to this graph-internal form (T-H5.4): stale glue cannot be
  # served, the DEPENDS edge replays it first.
  polyorch_rust_wasm_serve(TARGET visiaengine-wasm
      NAME_PREFIX visiaengine-wasm
      DIRECTORY "${CMAKE_BINARY_DIR}/web-demo"
      SOURCES  "${CMAKE_SOURCE_DIR}/examples/wasm/index.html"
      DATA     "${CMAKE_SOURCE_DIR}/resources/data/park.geojson"
      FLAVOR   web
      PORT     8017
      FOLDER   "${_fold}")

  # IDE sources: the WP6 sub-handle is an IMPORTED target (no SOURCES
  # property -- measured), so the mount rides the MEDIATOR (the crates
  # face mounts its custom build nodes; the mediator carries the build
  # inputs and IS the IDE node for this lane)
  polyorch_rust_ide_mount(TARGET visiaengine-wasm-build
      MANIFEST "${_manifest}"
      CRATE visiaengine-wasm
      FOLDER "${_fold}")

endfunction()
