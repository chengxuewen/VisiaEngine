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
  polyorch_rust_test(
    PACKAGE visiaengine-geo
    NAME    visiaengine-geo-rusttest
    MANIFEST "${CMAKE_SOURCE_DIR}/crates/visiaengine-geo/Cargo.toml"
    FOLDER "crates/visiaengine-geo")
endfunction()
