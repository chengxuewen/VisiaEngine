# PolyOrch crate-face wiring for VisiaEngine workspace rlibs (T-B era, table
# form since the 9-crate rollout).
#
# History: born as the T-B "pilot" (geo test-only), grew verbs, then the core
# crate, then the table -- renamed off "pilot" (2026-10-12): a file that owns
# production targets is not a pilot.
#
# Constraints honored here:
#  * Hosts do not invent cmake features -- everything below is either a
#    PolyOrch public function call or the host's own wrapper idiom
#    (custom targets invoking cargo via the PolyOrch-resolved toolchain,
#    the same shape as the visiaengine-example-*-run window buttons).
#  * Plain rlibs match NO D40 artifact kind (cargo emits lib<name>_.rlib,
#    underscore), so "build" is a custom cargo button, not a fake handle.
#  * clean is rm -rf BASE_DIR: all rlibs SHARE one cache dir
#    (.cargo-target-rlibs) and every clean button clears that whole dir --
#    idempotent and honest ("clean the rlib face"), never a sibling wipe.
#  * ide_mount hangs on the crate's own build target only (no cross-crate
#    source mounts: dependencies are expressed as edges between build
#    targets; rust-analyzer owns cross-crate navigation via the workspace).

option(VISIAENGINE_POLYORCH_CRATES "Register PolyOrch-driven targets for the workspace rlib face" ON)

# The dependency table -- single source of truth for this face.
# Format: <crate>@<dep>@<dep>   (entries separated by newline-safe ';'; the '@'
# separator is used INSIDE an entry because CMake's foreach splits list values
# on ';' -- a ';'-joined dep list would shatter the entry into fake items).
set(VE_RLIB_FACE
    "visiaengine-core"
    "visiaengine-io-hdr"
    "visiaengine-io-text@visiaengine-core"
    "visiaengine-io-gltf@visiaengine-core"
    "visiaengine-io-points@visiaengine-core"
    "visiaengine-io-tiles@visiaengine-core"
    "visiaengine-geo@visiaengine-core"
    "visiaengine-render@visiaengine-core"
    "visiaengine-render-wgpu@visiaengine-core@visiaengine-render@visiaengine-io-gltf@visiaengine-geo@visiaengine-io-text")

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
  set(_rlib_dir "${CMAKE_BINARY_DIR}/.cargo-target-rlibs")

  # shared clean: every crate's clean button clears the shared rlib face.
  # No FOLDER: the action target is an implementation detail (the 9 crate
  # aliases are the user-facing entries); hiding it keeps the crates tree
  # free of a non-crate node. Same for the two aggregates below.
  polyorch_rust_clean(
      NAME visiaengine-rlibs-clean
      BASE_DIR "${_rlib_dir}")

  foreach(_entry ${VE_RLIB_FACE})
    string(REPLACE "@" ";" _kv "${_entry}")
    list(GET _kv 0 _c)
    list(LENGTH _kv _n)
    set(_deps "")
    if(_n GREATER 1)
      list(SUBLIST _kv 1 -1 _deps)
    endif()

    # build button (cargo direct through the PolyOrch-resolved toolchain)
    add_custom_target(${_c}-build
        COMMAND ${CMAKE_COMMAND} -E env "PATH=${_cargo_bin}:$ENV{PATH}"
                ${POLYORCH_RUST_CARGO} build -p ${_c}
                --manifest-path "${CMAKE_SOURCE_DIR}/crates/${_c}/Cargo.toml"
                --target-dir "${_rlib_dir}"
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
        COMMENT "cargo build -p ${_c} (rlib; shared rlib-face cache)"
        VERBATIM)
    set_target_properties(${_c}-build PROPERTIES FOLDER "crates/${_c}")
    # dependency edges: visible build order (cargo resolves the same order
    # inside its lane; the edge makes it visible in the host graph)
    foreach(_d ${_deps})
      add_dependencies(${_c}-build ${_d}-build)
    endforeach()

    # tests (polyorch verb; drives lib+deps compilation in the shared cache)
    polyorch_rust_test(
        PACKAGE ${_c}
        NAME    ${_c}-test
        MANIFEST "${CMAKE_SOURCE_DIR}/crates/${_c}/Cargo.toml"
        BASE_DIR "${_rlib_dir}"
        FOLDER "crates/${_c}")

    # per-crate clean alias -> the shared face clean (each button is the
    # same honest action: clear the rlib cache; idempotent across crates)
    add_custom_target(${_c}-clean
        COMMAND ${CMAKE_COMMAND} -E echo "cleaning the shared rlib face (${_rlib_dir})"
        DEPENDS visiaengine-rlibs-clean
        VERBATIM)
    set_target_properties(${_c}-clean PROPERTIES FOLDER "crates/${_c}")

    # IDE sources on the crate's own build node (metadata-authoritative)
    polyorch_rust_ide_mount(TARGET ${_c}-build
        MANIFEST "${CMAKE_SOURCE_DIR}/crates/${_c}/Cargo.toml"
        CRATE ${_c})
    list(APPEND _all_builds ${_c}-build)
    list(APPEND _all_tests  ${_c}-test)
  endforeach()

  # face-level aggregates (the "all" entries of the three surfaces; hidden
  # from the grouping tree like rlibs-clean -- the per-crate buttons are the
  # user-facing entries). Ninja/make order the 9 via the dependency edges
  # already registered between the build buttons.
  add_custom_target(visiaengine-rlibs-build
      COMMENT "building all 9 workspace rlibs (dependency-ordered)"
      VERBATIM)
  add_dependencies(visiaengine-rlibs-build ${_all_builds})
  add_custom_target(visiaengine-rlibs-test
      COMMENT "running every workspace rlib's spec suite"
      VERBATIM)
  add_dependencies(visiaengine-rlibs-test ${_all_tests})
endfunction()
