# VisiaEnginePolyOrchPilot.cmake -- T-B pilot (2026-10-10): the first CMake
# target that drives cargo THROUGH PolyOrch instead of VisiaEngine's own bridge.
#
# Pilot crate: visiaengine-geo. Shape note (measured): geo is a plain rlib -- no
# [lib] crate-type -- so polyorch_rust_build's STATIC/SHARED artifact machinery
# cannot target it without touching the product crate (out of scope for a pilot);
# polyorch_rust_test drives the SAME PolyOrch path (rust setup resolution, cargo
# command wrapper, env isolation, BASE_DIR under the build tree) end to end.
# The capi artifact-form migration is T-C's job.
#
# Rollback: this whole file + its two wiring lines in the root. Nothing else knows.

option(VISIAENGINE_POLYORCH_PILOT "Register the PolyOrch-driven geo test pilot target" ON)

function(visiaengine_polyorch_pilot)
  if(NOT VISIAENGINE_POLYORCH_PILOT)
    return()
  endif()
  if(NOT COMMAND polyorch_rust_test)
    message(STATUS "PolyOrch pilot: rust face not available (sentinel/skip path) -- pilot off")
    return()
  endif()
  # T-B follow-through (user question 2026-10-12): full verb set for the pilot.
  # Shape constraint (measured twice): visiaengine-geo is a PLAIN rlib -- cargo
  # emits libvisiaengine_geo.rlib (underscore), which matches NO PolyOrch artifact
  # kind (bin/static/shared/example); a default-pair build() call registers
  # -static/-shared handles whose staging copies chase files cargo never writes.
  # The honest verb set is therefore: test (drives lib+deps compilation through
  # the wrapper -- cargo test builds the lib first) + clean, scoped to a
  # DEDICATED BASE_DIR so the clean cannot wipe the capi/examples shared cache
  # (polyorch_rust_clean rm -rf's the whole target dir it is given).
  if(NOT POLYORCH_RUST_FOUND)
    return()
  endif()
  set(_geo_manifest "${CMAKE_SOURCE_DIR}/crates/visiaengine-geo/Cargo.toml")
  set(_geo_dir "${CMAKE_BINARY_DIR}/.cargo-target-geo")
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
endfunction()
