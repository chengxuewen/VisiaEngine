# PolyOrch crate-face wiring for VisiaEngine workspace members (T-B era).
# History: born as the T-B "pilot" (geo test-only), grew verbs, then the
# core crate and the dependency edge -- renamed off "pilot" (2026-10-12):
# a file that owns production targets is not a pilot.
#
# Constraints honored here:
#  * Hosts do not invent cmake features -- everything below is either a
#    PolyOrch public function call or the host's own wrapper idiom
#    (custom targets invoking cargo via the PolyOrch-resolved toolchain,
#    the same shape as the cargo-run_ window buttons).
#  * Plain rlibs match NO D40 artifact kind (cargo emits lib<name>_.rlib,
#    underscore), so "build" is a custom cargo button, not a fake handle.
#  * polyorch_rust_clean rm -rf's the whole BASE_DIR it is given -- every
#    crate gets a DEDICATED cache dir or clean wipes its siblings.
#  * ide_mount hangs on the crate's own build target only (no cross-crate
#    source mounts: the dependency is expressed as an edge, not by
#    duplicating the dependency's sources into the dependent's tree;
#    rust-analyzer owns cross-crate navigation via the workspace).

option(VISIAENGINE_POLYORCH_CRATES "Register PolyOrch-driven targets for the workspace crate face" ON)

function(visiaengine_polyorch_crates)
  if(NOT VISIAENGINE_POLYORCH_CRATES)
    return()
  endif()
  if(NOT COMMAND polyorch_rust_test)
    message(STATUS "PolyOrch crates: rust face not available (sentinel/skip path) -- off")
    return()
  endif()
  if(NOT POLYORCH_RUST_FOUND)
    return()
  endif()
  get_filename_component(_cargo_bin "${POLYORCH_RUST_CARGO}" DIRECTORY)

  # ---- visiaengine-core: build / test / clean / ide-mount -----------------
  set(_core_manifest "${CMAKE_SOURCE_DIR}/crates/visiaengine-core/Cargo.toml")
  set(_core_dir "${CMAKE_BINARY_DIR}/.cargo-target-core")
  add_custom_target(visiaengine-core-build
      COMMAND ${CMAKE_COMMAND} -E env "PATH=${_cargo_bin}:$ENV{PATH}"
              ${POLYORCH_RUST_CARGO} build -p visiaengine-core
              --manifest-path "${_core_manifest}"
              --target-dir "${_core_dir}"
      WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
      COMMENT "cargo build -p visiaengine-core (rlib; workspace lane consumes it via cargo's own graph)"
      VERBATIM)
  set_target_properties(visiaengine-core-build PROPERTIES FOLDER "crates/visiaengine-core")
  polyorch_rust_test(
      PACKAGE visiaengine-core
      NAME    visiaengine-core-rusttest
      MANIFEST "${_core_manifest}"
      BASE_DIR "${_core_dir}"
      FOLDER "crates/visiaengine-core")
  polyorch_rust_clean(
      NAME visiaengine-core-clean
      BASE_DIR "${_core_dir}"
      FOLDER "crates/visiaengine-core")
  polyorch_rust_ide_mount(TARGET visiaengine-core-build
      MANIFEST "${_core_manifest}" CRATE visiaengine-core)

  # ---- visiaengine-geo: build / test / clean / ide-mount + core edge ------
  set(_geo_manifest "${CMAKE_SOURCE_DIR}/crates/visiaengine-geo/Cargo.toml")
  set(_geo_dir "${CMAKE_BINARY_DIR}/.cargo-target-geo")
  add_custom_target(visiaengine-geo-build
      COMMAND ${CMAKE_COMMAND} -E env "PATH=${_cargo_bin}:$ENV{PATH}"
              ${POLYORCH_RUST_CARGO} build -p visiaengine-geo
              --manifest-path "${_geo_manifest}"
              --target-dir "${_geo_dir}"
      WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
      COMMENT "cargo build -p visiaengine-geo (rlib; depends on visiaengine-core-build)"
      VERBATIM)
  # the dependency edge IS the point: IDE shows geo-build -> core-build, and
  # a core rebuild reorders geo's button. Cargo resolves the same order
  # inside its own lane; this edge makes it visible in the host graph.
  add_dependencies(visiaengine-geo-build visiaengine-core-build)
  set_target_properties(visiaengine-geo-build PROPERTIES FOLDER "crates/visiaengine-geo")
  polyorch_rust_test(
      PACKAGE visiaengine-geo
      NAME    visiaengine-geo-rusttest
      MANIFEST "${_geo_manifest}"
      BASE_DIR "${_geo_dir}"
      FOLDER "crates/visiaengine-geo")
  polyorch_rust_clean(
      NAME visiaengine-geo-clean
      BASE_DIR "${_geo_dir}"
      FOLDER "crates/visiaengine-geo")
  polyorch_rust_ide_mount(TARGET visiaengine-geo-build
      MANIFEST "${_geo_manifest}" CRATE visiaengine-geo)
endfunction()
