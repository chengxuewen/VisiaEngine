# cargo step + the visiaengine::capi umbrella contract + the fake in-tree Config bridge (§2.3; the iceoryx2 shape,
# design = .omo/plans/visiaengine-cmake-project.md v1.0)

function(visiaengine_setup_cargo)
  set(_src ${CMAKE_SOURCE_DIR}/bindings/c/visiaengine-capi)
  if(VISIAENGINE_ARTIFACT_PATH)
    set(_dir "${VISIAENGINE_ARTIFACT_PATH}")
    set(_flags "")
  else()
    set(_rust_dir "${CMAKE_SOURCE_DIR}/target/cmake-rust") # repo-level shared target-dir:
    # multiple build dirs reuse the same cargo incremental surface (cargo isolates debug/release under profile subdirs; the lock serializes safely)
    if(CMAKE_BUILD_TYPE STREQUAL "Release")
      set(_profile "release")
      set(_flags "--release")
    else()
      set(_profile "debug")
      set(_flags "")
    endif()
    set(_dir "${_rust_dir}/${_profile}")
    # Exposed to the examples domain (the rs steps in VisiaEngineBindings.cmake reuse the same artifact-dir contract)
    set(VISIAENGINE_RUST_DIR "${_rust_dir}" PARENT_SCOPE)
    set(VISIAENGINE_RUST_PROFILE "${_profile}" PARENT_SCOPE)
  endif()

  # Per-OS artifact names (the six-row naming table only; no install semantics)
  if(WIN32)
    set(_shared "visiaengine.dll")
    set(_static "visiaengine.lib")
  elseif(APPLE)
    set(_shared "libvisiaengine.dylib")
    set(_static "libvisiaengine.a")
  else()
    set(_shared "libvisiaengine.so")
    set(_static "libvisiaengine.a")
  endif()

  if(NOT VISIAENGINE_ARTIFACT_PATH)
    # T-C (2026-10-10): the capi build engine moved to PolyOrch. polyorch_rust_build
    # with no kind keyword dispatches the WP6 default pair -> handles
    # visiaengine-capi-shared (cdylib) / visiaengine-capi-static (staticlib), with
    # buildable mediators <handle>-build; the cargo env isolation (the IDE direct-call
    # PATH guard above, historically this function's job) now lives in the supplier
    # command wrapper -- probe-verified end to end (cold 1m21s, incremental 0.1s).
    if(NOT COMMAND polyorch_rust_build OR NOT POLYORCH_RUST_FOUND)
      message(FATAL_ERROR
        "visiaengine_setup_cargo: the capi build needs the PolyOrch rust face. "
        "Call visiaengine_ensure_polyorch() + visiaengine_ensure_polyorch_rust() "
        "BEFORE visiaengine_setup_cargo() (the root orders this), or use the "
        "prebuilt lane -DVISIAENGINE_ARTIFACT_PATH=...")
    endif()
    polyorch_rust_build(TARGET visiaengine-capi
        PACKAGE visiaengine-capi CRATE visiaengine
        MANIFEST "${_src}/Cargo.toml"
        FOLDER "bindings/c/visiaengine-capi")
    # ALL semantics kept: the default build must produce the artifacts (install-tree
    # leg, umbrella link, IDE "build all"). Wrapper carries no FOLDER (root position rule).
    add_custom_target(visiaengine-cargo ALL)
    add_dependencies(visiaengine-cargo visiaengine-capi-shared-build visiaengine-capi-static-build)
    set(VISIAENGINE_CARGO_STEP "visiaengine-cargo" PARENT_SCOPE)
    # T-G1: the capi spec suite gets its button (crates-face precedent: rusttest
    # under the crate's FOLDER; default BASE_DIR = the same .cargo-target the
    # T-C handles use -- one cache dir serves the whole capi lane).
    polyorch_rust_test(
        PACKAGE visiaengine-capi
        NAME    visiaengine-capi-rusttest
        MANIFEST "${_src}/Cargo.toml"
        FOLDER  "bindings/c/visiaengine-capi")
  endif()

  # Install-tree consumer point (VisiaEngineInstall.cmake): the artifact path and
  # its OS-specific file name are exported here so the package config and the
  # install rule never re-derive the name (single source = this function).
  # T-C: under the polyorch lane the cdylib location comes from the handle
  # (IMPORTED_LOCATION is configure-readable for single-config generators --
  # probe-verified); the prebuilt lane keeps the name-table path.
  if(VISIAENGINE_ARTIFACT_PATH)
    set(VISIAENGINE_ARTIFACT "${_dir}/${_shared}")
  else()
    get_target_property(VISIAENGINE_ARTIFACT visiaengine-capi-shared IMPORTED_LOCATION)
  endif()
  set(VISIAENGINE_ARTIFACT "${VISIAENGINE_ARTIFACT}" PARENT_SCOPE)
  set(VISIAENGINE_SHARED_NAME "${_shared}" PARENT_SCOPE)

  add_library(visiaengine_capi_umbrella INTERFACE)
  add_library(visiaengine::capi ALIAS visiaengine_capi_umbrella)
  target_include_directories(visiaengine_capi_umbrella INTERFACE
    "$<BUILD_INTERFACE:${_src}/include>")
  target_link_libraries(visiaengine_capi_umbrella INTERFACE "${VISIAENGINE_ARTIFACT}")
  if(UNIX AND NOT APPLE) # in-tree consumers skip LD_LIBRARY_PATH (do not copy the predecessor's manual-env debt)
    get_filename_component(_adir "${VISIAENGINE_ARTIFACT}" DIRECTORY)
    target_link_options(visiaengine_capi_umbrella INTERFACE "-Wl,-rpath,${_adir}")
  endif()

  # Fake in-tree Config (the iceoryx2 two-state trick; the install-generated one = packaging round):
  # a subdirectory find_package(visiaengine) hits this file -> the targets already exist in-tree, so it succeeds
  set(visiaengine_DIR "${CMAKE_SOURCE_DIR}/cmake" CACHE PATH "visiaengine in-tree config" FORCE)

  # C++ header-only RAII facade (S4; permanent surface = consumers of ::capi also get ::cpp, independent of the examples gate)
  add_library(visiaengine_cpp_iface INTERFACE)
  add_library(visiaengine::cpp ALIAS visiaengine_cpp_iface)
  target_include_directories(visiaengine_cpp_iface INTERFACE
    "$<BUILD_INTERFACE:${CMAKE_SOURCE_DIR}/bindings/cpp/include>")
  target_link_libraries(visiaengine_cpp_iface INTERFACE visiaengine::capi)

  message(STATUS "visiaengine::capi umbrella in place: ${_dir}/${_shared} · visiaengine::cpp header surface in place")
endfunction()
