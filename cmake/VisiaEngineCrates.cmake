# PolyOrch crate-face wiring for VisiaEngine workspace rlibs (T-B era, table
# form since the 9-crate rollout).
#
# History: born as the T-B "pilot" (geo test-only), grew verbs, then the core
# crate, then the table -- renamed off "pilot" (2026-10-12): a file that owns
# production targets is not a pilot. R3b (2026-10-15): the hand-rolled cargo
# build loop retired onto the supplier's R3 LIB kind (PolyOrch v0.3.0, D42).
#
# Constraints honored here:
#  * Hosts do not invent cmake features -- everything below is a PolyOrch
#    public function call. The per-crate clean ALIASES are the host's own
#    wrapper idiom (echo + DEPENDS: no cargo invocation, no env surface,
#    so no command wrapper is required there).
#  * R3b (v0.3.0, D42): the rlib lane rides the LIB kind -- cargo --lib
#    through the supplier command wrapper (PATH + host-env --unset
#    isolation, the PIT-14 class the hand loop's bare PATH=... wrap used to
#    leak). The handle is INTERFACE IMPORTED: ordering + the POLYORCH_RUST_LIB
#    path, never a link input. The consumers are cargo-side (ffi/examples
#    compile these crates through cargo's own dependency graph -- no CMake
#    target reads the .rlib path; verified at conversion time against
#    Install/Packaging/gate-pack).
#  * Names are a contract: the face mediator is <TARGET>-build, byte-identical
#    to the retired hand button -- the whole button surface keeps its spelling.
#  * clean is rm -rf BASE_DIR: all rlibs SHARE one cache dir
#    (.cargo-target-rlibs, kept as BASE_DIR on build and test so the three
#    surfaces ride one cache). Every clean button clears that whole dir --
#    idempotent and honest ("clean the rlib face"), never a sibling wipe of
#    the shared .cargo-target the ffi/examples lanes use.
#  * IDE source mount: the face mounts the mediator's sources at creation
#    (metadata-authoritative list) -- the retired per-crate
#    polyorch_rust_ide_mount call duplicated exactly that.

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

function(visiaengine_crates)
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
  # shared rlib-face cache -- one BASE_DIR across build, test and clean
  # (the face routes cargo's --target-dir; the location is unchanged from
  # the pre-R3b hand loop).
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

    # build button: the R3 LIB kind (v0.3.0). cargo --lib rides the supplier
    # wrapper; CRATE carries the dash->underscore lib-name normalization
    # (artifact: ${_rlib_dir}/<profile>/lib<crate>.rlib -- same dir the hand
    # loop stamped; only the argv engine changed, and --lib is a no-op
    # restriction here: none of the nine crates carries a bin target).
    string(REPLACE "-" "_" _crate "${_c}")
    polyorch_rust_build(
        TARGET  ${_c}
        PACKAGE ${_c}
        CRATE   ${_crate}
        LIB
        MANIFEST "${CMAKE_SOURCE_DIR}/crates/${_c}/Cargo.toml"
        BASE_DIR "${_rlib_dir}"
        FOLDER   "crates/${_c}")
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

    # (IDE sources: mounted by polyorch_rust_build itself -- the mediator's
    # own metadata-authoritative list, same content as the retired explicit
    # polyorch_rust_ide_mount call, which this comment now documents away.)
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
