# VisiaEnginePolyOrch.cmake -- PolyOrch embed shim (phase-2 of the 2026-10-10
# embed plan; PolyOrch-side contract = D39, pinned at submodule SHA c8410c9).
#
# Semantics:
#   * Sentinel present in this configure (an outer scope -- a superbuild that
#     embeds VisiaEngine -- already ran add_subdirectory(PolyOrch)): REUSE it.
#     First-wins (D39): the winning copy's code is live everywhere; this shim
#     only reports which copy won.
#   * No sentinel: add VisiaEngine's own pinned submodule copy.
#
# Version-conflict policy (user-adjudicated 2026-10-10, plan #0): the pinned
# submodule SHA is a "lowest tested version" declaration, NOT an enforcement --
# the effective copy is whatever won first-wins in this build tree. The API
# floor is guarded by COMMAND existence probes (works against pre-D39 bare
# sentinels too); a missing command FATALs with the fix for the superbuild
# maintainer. No version-range negotiation until PolyOrch ships 1.0 + semver.

# Optional escape hatch: a superbuild may point at its own checkout.
set(VISIAENGINE_POLYORCH_DIR "" CACHE PATH
    "PolyOrch checkout to embed when no PolyOrch::PolyOrch sentinel exists (default: the pinned submodule at <repo>/PolyOrch)")

function(visiaengine_ensure_polyorch)
    if(TARGET PolyOrch::PolyOrch)
        get_target_property(_po_dir PolyOrch::PolyOrch INTERFACE_POLYORCH_CMAKE_DIR)
        get_target_property(_po_ver PolyOrch::PolyOrch INTERFACE_POLYORCH_VERSION)
        if(NOT _po_dir)
            message(WARNING
                "VisiaEngine: reusing a PRE-D39 PolyOrch sentinel (no "
                "INTERFACE_POLYORCH_CMAKE_DIR property). Location discovery "
                "falls back to PolyOrch_SOURCE_DIR -- upgrade the supplier.")
        endif()
        message(STATUS
            "VisiaEngine: reusing PolyOrch ${_po_ver} at ${_po_dir} "
            "(first-wins: registered by an outer scope)")
        return()
    endif()

    if(VISIAENGINE_POLYORCH_DIR)
        set(_po "${VISIAENGINE_POLYORCH_DIR}")
    else()
        # CMAKE_CURRENT_LIST_DIR inside a FUNCTION is the defining file's dir
        # (cmake/), so "../PolyOrch" would escape the repo (measured). Pin from
        # the top-level source dir instead.
        set(_po "${CMAKE_SOURCE_DIR}/PolyOrch")
    endif()
    if(NOT EXISTS "${_po}/CMakeLists.txt")
        message(FATAL_ERROR
            "VisiaEngine: no PolyOrch::PolyOrch sentinel in this configure, "
            "and no checkout at '${_po}'. Fix: run 'git submodule update --init "
            "--recursive' (pins the tested SHA), or set -DVISIAENGINE_POLYORCH_DIR=<path> "
            "to supply a different PolyOrch checkout.")
    endif()
    message(STATUS "VisiaEngine: embedding vendored PolyOrch at ${_po}")
    add_subdirectory("${_po}" "${CMAKE_BINARY_DIR}/_polyorch" EXCLUDE_FROM_ALL)
endfunction()

# T-B1 (2026-10-10): translate the USE_PIXI master switch into PolyOrch's rust
# setup. Callers wanting the rust face call visiaengine_ensure_polyorch_rust()
# AFTER this module's ensure call. FROM pixi REQUIRES polyorch_pixi_setup()
# first (supplier FATALs otherwise) -- the ordering below is load-bearing.
function(visiaengine_ensure_polyorch_rust)
    if(TARGET PolyOrch::PolyOrch AND COMMAND polyorch_rust_setup)
        if(NOT POLYORCH_RUST_FOUND)
            if(VISIAENGINE_USE_PIXI)
                # module path does not propagate out of add_subdirectory (measured):
                # include by absolute path via the sentinel's D39 property.
                get_target_property(_po_mod PolyOrch::PolyOrch INTERFACE_POLYORCH_CMAKE_DIR)
                include("${_po_mod}/PolyOrchPixiHelpers.cmake")
                polyorch_pixi_setup()
                polyorch_rust_setup(FROM pixi)
            else()
                polyorch_rust_setup(FROM system)
            endif()
            message(STATUS "VisiaEngine: PolyOrch rust face ready "
                    "(route = ${VISIAENGINE_USE_PIXI}, cargo = ${POLYORCH_RUST_CARGO})")
        endif()
    endif()
endfunction()
