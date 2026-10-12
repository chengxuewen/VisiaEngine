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
    # IDE direct-invocation guard: conda cargo finds rustc via PATH; on a bare PATH = "could not execute rustc"
    # (2026-09-15, measured: the pixi run wrapper masked it; it first blew up when the user's IDE invoked /usr/bin/cmake directly) --
    # the build command is always prefixed with cargo's own directory (the rustc sibling is measured present), harmless in SYSTEM mode.
    get_filename_component(_cargo_bin "${VISIAENGINE_CARGO}" DIRECTORY)
    add_custom_target(cargo-build_capi ALL
      COMMAND ${CMAKE_COMMAND} -E env "PATH=${_cargo_bin}:$ENV{PATH}"
              ${VISIAENGINE_CARGO} build -p visiaengine-capi ${_flags}
              --target-dir ${_rust_dir}
      BYPRODUCTS ${_dir}/${_shared} ${_dir}/${_static}
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
      VERBATIM USES_TERMINAL)
    # Root-level build step carries no FOLDER (position rule: root scope = a bare top-level listing; semantics carried by the name)
    set(VISIAENGINE_CARGO_STEP "cargo-build_capi" PARENT_SCOPE)
  endif()

  # Install-tree consumer point (VisiaEngineInstall.cmake): the artifact path and
  # its OS-specific file name are exported here so the package config and the
  # install rule never re-derive the name (single source = this function).
  set(VISIAENGINE_ARTIFACT "${_dir}/${_shared}" PARENT_SCOPE)
  set(VISIAENGINE_SHARED_NAME "${_shared}" PARENT_SCOPE)

  add_library(visiaengine_capi_umbrella INTERFACE)
  add_library(visiaengine::capi ALIAS visiaengine_capi_umbrella)
  target_include_directories(visiaengine_capi_umbrella INTERFACE
    "$<BUILD_INTERFACE:${_src}/include>")
  target_link_libraries(visiaengine_capi_umbrella INTERFACE "${_dir}/${_shared}")
  if(UNIX AND NOT APPLE) # in-tree consumers skip LD_LIBRARY_PATH (do not copy the predecessor's manual-env debt)
    target_link_options(visiaengine_capi_umbrella INTERFACE "-Wl,-rpath,${_dir}")
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
