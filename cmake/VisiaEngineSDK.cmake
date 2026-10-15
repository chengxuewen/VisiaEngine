# SDK three-level resolution (section 2.2): AUTO|PIXI|SYSTEM|<explicit path>;
# every negative path hard-errors with a message that names the remediation.

set(VISIAENGINE_USE_PIXI "ON" CACHE BOOL "Allow pixi-sourced tools (permission, not a demand: ON resolves pixi first and falls back to system; OFF takes pixi out entirely -- cargo/Qt resolve to system only). T-B1 2026-10-10.")
set(VISIAENGINE_RUST_SDK "AUTO" CACHE STRING "Rust toolchain: AUTO|PIXI|SYSTEM|<absolute cargo path>")
# 2026-10-10 (T-A0' adjudication): the five-value VISIAENGINE_QT_SDK collapses into the two-value VISIAENGINE_QT --
# ON = want Qt (the resolution chain serves itself: pixi qt-spike first -> system -> neither = FATAL with two ways out), OFF = do not touch Qt.
# Source selection is not user input (AUTO's silent degradation belongs to the "nobody asked for Qt" world; a failed ON must be loud).
# Old-name detection: a stale cache silently flipping Qt=OFF is a false green -- point the way, loudly.
if(DEFINED VISIAENGINE_QT_SDK)
  message(WARNING
    "VISIAENGINE_QT_SDK has been renamed VISIAENGINE_QT (two-valued OFF|ON). Stale cache in this build dir "
    "VISIAENGINE_QT_SDK=${VISIAENGINE_QT_SDK} will be ignored; delete CMakeCache.txt and reconfigure, "
    "or pass -DVISIAENGINE_QT=ON")
endif()
set(VISIAENGINE_QT "OFF" CACHE BOOL "Want Qt6? (ON = pixi qt-spike first -> system -> neither = FATAL; OFF = no Qt setup)")
set(VISIAENGINE_PIXI_ENV "qt-spike" CACHE STRING "pixi environment name (falls back to default when that env dir is absent)")
set(VISIAENGINE_ARTIFACT_PATH "" CACHE PATH "Prebuilt capi artifact directory (setting it = pure consumption, no cargo step; a missing file = hard error)")
set_property(CACHE VISIAENGINE_RUST_SDK PROPERTY STRINGS AUTO PIXI SYSTEM)
set_property(CACHE VISIAENGINE_QT PROPERTY TYPE BOOL)

# pixi prefix discovery: an activated $CONDA_PREFIX wins; otherwise the in-repo .pixi/envs/<env> (same convention as the gate-abi family)
function(_ve_pixi_prefix out)
  # Master-gate choke point (T-B1): OFF empties the prefix so every downstream
  # branch (activated shell, envs loop, Qt pixi route) resolves to system for free.
  if(NOT VISIAENGINE_USE_PIXI)
    set(${out} "" PARENT_SCOPE)
    return()
  endif()
  if(DEFINED ENV{CONDA_PREFIX} AND EXISTS "$ENV{CONDA_PREFIX}/bin/cargo")
    set(${out} "$ENV{CONDA_PREFIX}" PARENT_SCOPE)
    return()
  endif()
  foreach(_env ${VISIAENGINE_PIXI_ENV} default)
    if(EXISTS "${CMAKE_SOURCE_DIR}/.pixi/envs/${_env}/bin/cargo")
      set(${out} "${CMAKE_SOURCE_DIR}/.pixi/envs/${_env}" PARENT_SCOPE)
      return()
    endif()
  endforeach()
  set(${out} "" PARENT_SCOPE)
endfunction()

function(_ve_msrv_check cargo_path rust_version)
  execute_process(COMMAND "${cargo_path}" --version
                  OUTPUT_VARIABLE _out OUTPUT_STRIP_TRAILING_WHITESPACE ERROR_QUIET)
  if(_out MATCHES "([0-9]+)\\.([0-9]+)\\.([0-9]+)")
    set(_v "${CMAKE_MATCH_1}.${CMAKE_MATCH_2}.${CMAKE_MATCH_3}")
    # R1 (2026-10-15): the comparison itself is the supplier's (PolyOrchFindRust's
    # polyorch_rust_version_ok, VERSION form = greater-equal); the host keeps the
    # probe, the parse, and the WARNING-only severity.
    polyorch_rust_version_ok("${_v}" _ok VERSION "${rust_version}")
    if(NOT _ok)
      message(WARNING "Resolved cargo ${_v} < workspace rust-version ${rust_version} (MSRV contract; upgrade the pixi environment or switch the SDK)")
    endif()
  endif()
endfunction()

function(visiaengine_resolve_sdks)
  _ve_pixi_prefix(_pixi)
  file(READ "${CMAKE_SOURCE_DIR}/Cargo.toml" _ws)
  string(REGEX MATCH "rust-version = \"([0-9.]+)\"" _rm "${_ws}")
  set(_msrv "${CMAKE_MATCH_1}")

  # ---- Rust / cargo (or the pure-consumption prebuilt mode) ----
  if(VISIAENGINE_ARTIFACT_PATH)
    set(_so "${VISIAENGINE_ARTIFACT_PATH}/libvisiaengine.so")
    if(NOT EXISTS "${_so}" AND WIN32)
      set(_so "${VISIAENGINE_ARTIFACT_PATH}/visiaengine.dll")
    endif()
    if(NOT EXISTS "${_so}")
      message(FATAL_ERROR "VISIAENGINE_ARTIFACT_PATH=${VISIAENGINE_ARTIFACT_PATH} has no capi artifact (the escape hatch never silently consumes stale artifacts)")
    endif()
    set(VISIAENGINE_CARGO "" CACHE INTERNAL "prebuilt mode has no cargo")
    message(STATUS "Rust SDK: prebuilt @ ${VISIAENGINE_ARTIFACT_PATH}")
  else()
    set(_cargo "")
    set(_kind "")
    if(VISIAENGINE_RUST_SDK STREQUAL "AUTO")
      if(_pixi AND EXISTS "${_pixi}/bin/cargo")
        set(_cargo "${_pixi}/bin/cargo")
        set(_kind "pixi")
      else()
        find_program(_found_cargo cargo)
        if(_found_cargo)
          set(_cargo "${_found_cargo}")
          set(_kind "system")
        endif()
      endif()
      if(NOT _cargo)
        message(FATAL_ERROR "AUTO found no cargo: the pixi environment is absent (run pixi install -e ${VISIAENGINE_PIXI_ENV} first) and system PATH has no cargo")
      endif()
    elseif(VISIAENGINE_RUST_SDK STREQUAL "PIXI")
      if(NOT VISIAENGINE_USE_PIXI)
        message(FATAL_ERROR
          "Contradiction: VISIAENGINE_USE_PIXI=OFF but VISIAENGINE_RUST_SDK=PIXI.\n"
          "  Drop -DVISIAENGINE_USE_PIXI=OFF, or drop the PIXI force (AUTO/SYSTEM).")
      endif()
      if(NOT _pixi OR NOT EXISTS "${_pixi}/bin/cargo")
        message(FATAL_ERROR "PIXI forced but the pixi environment is absent: run pixi install -e ${VISIAENGINE_PIXI_ENV} (no fall-back to system)")
      endif()
      set(_cargo "${_pixi}/bin/cargo")
      set(_kind "pixi")
    elseif(VISIAENGINE_RUST_SDK STREQUAL "SYSTEM")
      find_program(_found_cargo cargo)
      if(NOT _found_cargo)
        message(FATAL_ERROR "SYSTEM forced but system PATH has no cargo (no fall-back to pixi; switch to PIXI or give a path)")
      endif()
      # Anti-pollution: a pixi activation injects the conda cargo into PATH -- SYSTEM semantics demand a real system toolchain
      if((_pixi AND _found_cargo MATCHES "^${_pixi}/") OR
         (DEFINED ENV{CONDA_PREFIX} AND _found_cargo MATCHES "^$ENV{CONDA_PREFIX}/"))
        message(FATAL_ERROR "SYSTEM mode rejects a pixi-activated PATH cargo (${_found_cargo}); switch to PIXI or use a non-activated shell")
      endif()
      set(_cargo "${_found_cargo}")
      set(_kind "system")
    else()
      if(NOT EXISTS "${VISIAENGINE_RUST_SDK}")
        message(FATAL_ERROR "VISIAENGINE_RUST_SDK=${VISIAENGINE_RUST_SDK} is not an executable file (an explicit path is the escape hatch; a nonexistent one is an error)")
      endif()
      set(_cargo "${VISIAENGINE_RUST_SDK}")
      set(_kind "path")
    endif()
    _ve_msrv_check("${_cargo}" "${_msrv}")
    set(VISIAENGINE_CARGO "${_cargo}" CACHE INTERNAL "resolved cargo")
    message(STATUS "Rust SDK: ${_cargo} [${_kind}] · MSRV ${_msrv}")
  endif()

  # ---- Qt6 (T-A0' two-value contract) ----
  if(NOT VISIAENGINE_QT)
    message(STATUS "Qt: OFF (Qt not enabled; pass -DVISIAENGINE_QT=ON to want Qt)")
    return()
  endif()
  # ON = Qt is strictly required. The chain serves itself: pixi (qt-spike first, the env self-sufficiency principle) -> system -> neither = FATAL.
  # With USE_PIXI=OFF, _pixi is already empty (the master-switch cut-off point), so only the system path remains here -- no re-check needed.
  find_package(Qt6 COMPONENTS Widgets QUIET)
  set(_qkind "")
  if(NOT Qt6_FOUND AND _pixi)
    list(APPEND CMAKE_PREFIX_PATH "${_pixi}")
    find_package(Qt6 COMPONENTS Widgets QUIET)
    if(Qt6_FOUND)
      set(_qkind "pixi")
    endif()
  elseif(Qt6_FOUND)
    set(_qkind "system")
  endif()
  if(NOT Qt6_FOUND)
    message(FATAL_ERROR
      "VISIAENGINE_QT=ON but Qt6 was found on neither side:\n"
      "  pixi route: pixi install -e ${VISIAENGINE_PIXI_ENV} (qt6-main lives in the qt-spike environment)\n"
      "  system route: install Qt6 (qt6-base-dev / the official installer) and retry\n"
      "If you do not want Qt, pass -DVISIAENGINE_QT=OFF (the default)")
  endif()
  if(_qkind STREQUAL "system")
    message(STATUS "Qt: ${Qt6_VERSION} [system] ${Qt6_DIR}")
  elseif(_qkind)
    message(STATUS "Qt: ${Qt6_VERSION} [${_qkind}] ${Qt6_DIR}")
  endif()
endfunction()
