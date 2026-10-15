# cargo step + the visiaengine::c link-face contract + the fake in-tree Config bridge (§2.3; the iceoryx2 shape,
# design = .omo/plans/visiaengine-cmake-project.md v1.0)

function(visiaengine_setup_ffi)
  set(_src ${CMAKE_SOURCE_DIR}/crates/visiaengine-ffi)
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
    # visiaengine-ffi-shared (cdylib) / visiaengine-ffi-static (staticlib), with
    # buildable mediators <handle>-build; the cargo env isolation (the IDE direct-call
    # PATH guard above, historically this function's job) now lives in the supplier
    # command wrapper -- probe-verified end to end (cold 1m21s, incremental 0.1s).
    if(NOT COMMAND polyorch_rust_build OR NOT POLYORCH_RUST_FOUND)
      message(FATAL_ERROR
        "visiaengine_setup_ffi: the capi build needs the PolyOrch rust face. "
        "Call visiaengine_ensure_polyorch() + visiaengine_ensure_polyorch_rust() "
        "BEFORE visiaengine_setup_ffi() (the root orders this), or use the "
        "prebuilt lane -DVISIAENGINE_ARTIFACT_PATH=...")
    endif()
    polyorch_rust_build(TARGET visiaengine-ffi
        PACKAGE visiaengine-ffi CRATE visiaengine
        MANIFEST "${_src}/Cargo.toml"
        FOLDER "crates/visiaengine-ffi")
    # ALL semantics kept: the default build must produce the artifacts (install-tree
    # leg, umbrella link, IDE "build all").
    # The name is the LANE word (ffi), not the tool word -- the retired cargo-*
    # coinage class (T-H5.5 lineage).
    # FOLDER: the T-H10 root-group rule is for aggregates WITHOUT an on-disk
    # home (rlibs spans 9 crates, example spans 5 lanes). This one HAS a home:
    # it bundles exactly the visiaengine-ffi crate's mediators, and its lane
    # sibling visiaengine-ffi-test already lives in crates/visiaengine-ffi --
    # home exists => go home (the rule's corollary, user-adjudicated 2026-10-14).
    add_custom_target(visiaengine-ffi-build ALL)
    add_dependencies(visiaengine-ffi-build visiaengine-ffi-shared-build visiaengine-ffi-static-build)
    set_target_properties(visiaengine-ffi-build PROPERTIES FOLDER "crates/visiaengine-ffi")
    set(VISIAENGINE_FFI_STEP "visiaengine-ffi-build" PARENT_SCOPE)
    # T-G1: the capi spec suite gets its button (crates-face precedent: -test
    # under the crate's FOLDER; default BASE_DIR = the same .cargo-target the
    # T-C handles use -- one cache dir serves the whole capi lane).
    polyorch_rust_test(
        PACKAGE visiaengine-ffi
        NAME    visiaengine-ffi-test
        MANIFEST "${_src}/Cargo.toml"
        FOLDER  "crates/visiaengine-ffi")
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
    get_target_property(VISIAENGINE_ARTIFACT visiaengine-ffi-shared IMPORTED_LOCATION)
  endif()
  set(VISIAENGINE_ARTIFACT "${VISIAENGINE_ARTIFACT}" PARENT_SCOPE)
  set(VISIAENGINE_SHARED_NAME "${_shared}" PARENT_SCOPE)

  add_library(visiaengine-c INTERFACE)
  add_library(visiaengine::c ALIAS visiaengine-c)
  target_include_directories(visiaengine-c INTERFACE
    "$<BUILD_INTERFACE:${CMAKE_SOURCE_DIR}/bindings/c/include>")
  target_link_libraries(visiaengine-c INTERFACE "${VISIAENGINE_ARTIFACT}")
  if(UNIX AND NOT APPLE) # in-tree consumers skip LD_LIBRARY_PATH (do not copy the predecessor's manual-env debt)
    get_filename_component(_adir "${VISIAENGINE_ARTIFACT}" DIRECTORY)
    target_link_options(visiaengine-c INTERFACE "-Wl,-rpath,${_adir}")
  endif()

  # Fake in-tree Config (the iceoryx2 two-state trick; the install-generated one = packaging round):
  # a subdirectory find_package(visiaengine) hits this file -> the targets already exist in-tree, so it succeeds
  set(visiaengine_DIR "${CMAKE_SOURCE_DIR}/cmake" CACHE PATH "visiaengine in-tree config" FORCE)

  # C++ header-only RAII facade (S4; permanent surface = consumers of ::capi also get ::cpp, independent of the examples gate)
  add_library(visiaengine-cpp INTERFACE)
  add_library(visiaengine::cpp ALIAS visiaengine-cpp)
  target_include_directories(visiaengine-cpp INTERFACE
    "$<BUILD_INTERFACE:${CMAKE_SOURCE_DIR}/bindings/cpp/include>")
  target_link_libraries(visiaengine-cpp INTERFACE visiaengine::c)
  # T-H: the IDE face. Carrier lesson (measured via the cmake-file-api): an
  # INTERFACE target never enters the generator's target list (no artifact ->
  # invisible to the File API AND the IDE tree) and an IMPORTED target cannot
  # take SOURCES -- so the visible face rides a custom target (the same
  # carrier class as the crates-face build buttons); the INTERFACE library
  # stays the LINK surface (consumers keep linking visiaengine::cpp).
  # polyorch_header_face supplies the disk<->table audit on both carriers;
  # the HEADERS list is the single source, written once.
  # the C face viewer (the handwritten header is the C world's single most
  # load-bearing file -- viewer + disk<->table audit on it)
  add_custom_target(visiaengine-c-sources
      COMMAND ${CMAKE_COMMAND} -E echo "visiaengine::c sources: ${CMAKE_SOURCE_DIR}/bindings/c/include"
      VERBATIM)
  polyorch_header_face(TARGET visiaengine-c-sources
      DIRECTORY "${CMAKE_SOURCE_DIR}/bindings/c/include"
      HEADERS visiaengine.h)
  set_target_properties(visiaengine-c-sources PROPERTIES FOLDER "bindings/c")

  set(_cpp_headers visiaengine/visiaengine.hpp)
  add_custom_target(visiaengine-cpp-sources
      COMMAND ${CMAKE_COMMAND} -E echo "visiaengine::cpp sources: ${CMAKE_SOURCE_DIR}/bindings/cpp/include"
      VERBATIM)
  polyorch_header_face(TARGET visiaengine-cpp-sources
      DIRECTORY "${CMAKE_SOURCE_DIR}/bindings/cpp/include"
      HEADERS ${_cpp_headers})
  set_target_properties(visiaengine-cpp-sources PROPERTIES FOLDER "bindings/cpp")
  if(EXISTS "${CMAKE_SOURCE_DIR}/bindings/qt/visiaengine_widget.hpp")
    # the qt link face (the T-H commit promised ::qt but never landed it --
    # repaired here per the T-H2 plan's BUILD ruling): the widget eats the
    # RAII facade, mirroring how ::cpp eats ::c
    add_library(visiaengine-qt INTERFACE)
    add_library(visiaengine::qt ALIAS visiaengine-qt)
    target_include_directories(visiaengine-qt INTERFACE
      "$<BUILD_INTERFACE:${CMAKE_SOURCE_DIR}/bindings/qt>")
    target_link_libraries(visiaengine-qt INTERFACE visiaengine::cpp)
    add_custom_target(visiaengine-qt-sources
        COMMAND ${CMAKE_COMMAND} -E echo "visiaengine::qt sources: ${CMAKE_SOURCE_DIR}/bindings/qt/visiaengine_widget.hpp"
        VERBATIM)
    polyorch_header_face(TARGET visiaengine-qt-sources
        DIRECTORY "${CMAKE_SOURCE_DIR}/bindings/qt"
        HEADERS visiaengine_widget.hpp)
    set_target_properties(visiaengine-qt-sources PROPERTIES FOLDER "bindings/qt")
  endif()

  message(STATUS "visiaengine::c link face in place: ${_dir}/${_shared} · visiaengine::cpp / ::qt header surfaces in place")
endfunction()
