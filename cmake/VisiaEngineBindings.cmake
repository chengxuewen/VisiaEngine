# The visiaengine example registration scheme (R9/R10, v1.3): the Rust trio + C/C++/Qt native examples + the unified ctest list.
# The anti-glob explicit-registration discipline is inherited from the late VisiaEngineExamples.cmake (a file on disk but unregistered = hard error).
# Naming (R10): real target = E-number name | step = cargo-<verb>_<object> | ctest = example_<target base name>; rs/native are separated by LABELS.
# Design = .omo/plans/bindings-restructure-cpp-examples.md v1.3 §S3.

option(VISIAENGINE_EXAMPLES "the whole examples/ family (rs steps + C/C++/Qt examples + ctest entries)" ON)

if(VISIAENGINE_EXAMPLES AND CMAKE_VERSION VERSION_LESS 3.24)
  # Inherited from the contract-table text (named for migration at sem5): without SKIP_RETURN_CODE, the display family goes red instead of Skip on X-less machines
  message(FATAL_ERROR "the examples family needs CMake >= 3.24 (ctest SKIP_RETURN_CODE: 77 = Skipped, not Failed)")
endif()

# Native example (C/C++/Qt exe) registration macro: real target = name = ctest entry; the display family's LABELS hand the -LE/-L routing off
function(visiaengine_add_example _n)
  # ARGS must be multivalue: a single-value slot swallows only "--frames" (list flattening, measured in the E801 timeout case)
  cmake_parse_arguments(_A "DISPLAY;X11;QT;SDL3;CPP" "" "ARGS" ${ARGN})
  if(_A_DISPLAY)
    set(_lbl "example;native;display")
  else()
    set(_lbl "example;native;headless")
  endif()
  set(_ext ".c")
  if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/${_n}.cpp")
    set(_ext ".cpp")
  endif()
  add_executable(${_n} "${CMAKE_CURRENT_SOURCE_DIR}/${_n}${_ext}")
  target_link_libraries(${_n} PRIVATE visiaengine::capi)
  if(_A_X11)
    target_link_libraries(${_n} PRIVATE X11::X11)
  endif()
  if(_A_CPP)
    target_link_libraries(${_n} PRIVATE visiaengine::cpp)   # hpp-surface consumption = the shell's live acceptance test
  endif()
  if(_A_SDL3)
    target_link_libraries(${_n} PRIVATE SDL3::SDL3)
    get_target_property(_sdll SDL3::SDL3 IMPORTED_LOCATION)
    if(_sdll)
      get_filename_component(_sdldir ${_sdll} DIRECTORY)      # conda libs are not on the system ld path -- carry the rpath along
      set_property(TARGET ${_n} APPEND PROPERTY BUILD_RPATH ${_sdldir})
    endif()
  endif()
  if(DEFINED VISIAENGINE_CARGO_STEP)
    add_dependencies(${_n} ${VISIAENGINE_CARGO_STEP})  # umbrella .so ordering bridge (same as the probe)
  endif()
  get_filename_component(_lang "${CMAKE_CURRENT_SOURCE_DIR}" NAME)  # FOLDER = the on-disk language directory (the band number is already encoded in the E-number's first digit; no duplicate vocabulary table)
  set_target_properties(${_n} PROPERTIES FOLDER "examples/${_lang}")
  add_test(NAME example_${_n} COMMAND ${_n} ${_A_ARGS})
  set_tests_properties(example_${_n} PROPERTIES
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    SKIP_RETURN_CODE 77
    LABELS "${_lbl}")
  # Run steps = the human-verification form, emitted for every example (the rs family's cargo-run_* symmetry): zero-arg stays resident --
  # the DISPLAY family goes through run-gui (probes and falls back to :0 when the IDE has no DISPLAY), the headless family runs directly in a terminal where the output is the acceptance surface
  if(_A_DISPLAY)
    add_custom_target(run_${_n}
      COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR}
              ${CMAKE_SOURCE_DIR}/scripts/run-gui.sh $<TARGET_FILE:${_n}>
      USES_TERMINAL)
  else()
    add_custom_target(run_${_n}
      COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR} $<TARGET_FILE:${_n}>
      USES_TERMINAL)
  endif()
  add_dependencies(run_${_n} ${_n})
  set_target_properties(run_${_n} PROPERTIES FOLDER "examples/${_lang}")
  set_property(GLOBAL APPEND PROPERTY VE_REGISTERED_EXAMPLES ${_n})
endfunction()

function(visiaengine_setup_bindings)
  if(NOT VISIAENGINE_EXAMPLES)
    return()
  endif()
  if(VISIAENGINE_ARTIFACT_PATH)
    message(STATUS "visiaengine bindings: the prebuilt umbrella has no cargo surface; the examples domain bows out (probe/umbrella/::cpp stay)")
    return()
  endif()

  # -- Rust family: argv has its single home here (the pixi smoke is only a `ctest -R` forwarding shell; the transcription surface drops to zero) --
  set(_rs_items E101_clear E201_load_gltf E202_geo_viewer E203_measure_cli E204_pcl_viewer E205_text_labels E206_tile_viewer E302_fly_camera E303_split_screen E304_projection_morph E305_entity_anim E403_box_select E306_map_controls E504_glass_water
                E301_switch_camera E401_pick_demo E402_pick_interactive E501_shadow_demo
                E502_color_tuning E503_stroke_points E505_cjk_labels E506_postprocessing E507_hdr_tonemap E508_pbr_materials E509_ssao E510_route_flow E511_gate E601_bench_twin
                E901_twin_city)
  set(_disp_E101_clear ON)
  set(_disp_E201_load_gltf ON)
  set(_disp_E202_geo_viewer ON)
  set(_disp_E301_switch_camera ON)
  set(_disp_E402_pick_interactive ON)
  set(_disp_E503_stroke_points ON)
  set(_args_E101_clear "--frames;3")
  set(_args_E201_load_gltf "resources/data/twoprim.glb;--frames;3")
  set(_args_E202_geo_viewer "resources/data/park.geojson;--frames;3")
  set(_args_E301_switch_camera "resources/data/twoprim.glb;--frames;3")
  set(_args_E504_glass_water "--frames;1")
  set(_args_E303_split_screen "--frames;1")
  set(_args_E304_projection_morph "--frames;4")
  set(_args_E305_entity_anim "--frames;24")
  set(_disp_E510_route_flow ON)
  set(_args_E510_route_flow "--frames;1")
  set(_disp_E511_gate ON)
  set(_args_E511_gate "--frames;1")
  set(_disp_E403_box_select ON)
  set(_args_E306_map_controls "--frames;1")
  set(_args_E403_box_select "--frames;3")

  set(_args_E302_fly_camera "--frames;1")
  set(_args_E205_text_labels "--frames;1")
  set(_args_E204_pcl_viewer "--pick-check;--frames;3")  # self-asserting dual-half coverage + REND-38 point-pick triple assert (B2)
  set(_args_E206_tile_viewer "--frames;1")  # MVT decode -> render three-family pixel asserts (B3)
  set(_args_E401_pick_demo "--frames;2")
  set(_args_E402_pick_interactive "--frames;3")
  set(_args_E501_shadow_demo "--frames;1")  # dual-mode example: CI runs headless (default = interactive window); the registry only serves ctest
  set(_args_E502_color_tuning "--frames;1")
  set(_args_E503_stroke_points "--frames;3")
  set(_args_E505_cjk_labels "--frames;1")
  set(_args_E506_postprocessing "--frames;3")  # post-chain pixel asserts (WGPU-34; headless path)
  set(_args_E507_hdr_tonemap "--frames;3")
  set(_args_E508_pbr_materials "--frames;3")  # tonemap + hdr fixture asserts (REND-44/IO-17; headless path)
  set(_args_E509_ssao "--frames;3")  # SSAO off-vs-on pixel assert (WGPU-36; headless path)
  set(_args_E601_bench_twin "--count;5000;--frames;1")
  set(_args_E901_twin_city "--frames;1")  # dual-mode example (same scheme as E501): CI runs headless single-frame + PNG assert; no args = resident window

  # Disk<->table two-way audit (examples/rs side; table = _rs_items above)
  file(GLOB _rsf "${CMAKE_SOURCE_DIR}/examples/rs/E[0-9][0-9][0-9]_*.rs")
  foreach(_f ${_rsf})
    get_filename_component(_n ${_f} NAME_WE)
    if(NOT _n IN_LIST _rs_items)
      message(FATAL_ERROR "examples/rs/${_n}.rs is not in the registry (comment-style disabling forbidden)")
    endif()
  endforeach()

  get_filename_component(_cargo_bin "${VISIAENGINE_CARGO}" DIRECTORY)  # the rustc sibling directory (the PIT-18 guard shape)
  set(_cenv ${CMAKE_COMMAND} -E env "PATH=${_cargo_bin}:$ENV{PATH}")

  # T-D (2026-10-12): the examples build through the PolyOrch rust face (D40 EXAMPLE
  # kind). Engine story: handles rs-<n> (+ mediators rs-<n>-build) are the ONLY cargo
  # build site -- the ALL wrapper aggregates mediator dependencies instead of issuing
  # its own `cargo build`; run buttons and ctest execute the staged binary directly
  # (cmake-build*/bin/ via the RUNTIME staging), no cargo process at test time.
  # The run-gui DISPLAY fallback (vscode-server finding) rides the button lane.
  set(_rs_mediators "")
  foreach(_n ${_rs_items})
    if(NOT EXISTS "${CMAKE_SOURCE_DIR}/examples/rs/${_n}.rs")
      message(FATAL_ERROR "registry entry ${_n} has no file on disk (examples/rs/${_n}.rs)")
    endif()
    if(_disp_${_n})
      set(_lbl "example;rs;display")
    else()
      set(_lbl "example;rs;headless")
    endif()
    polyorch_rust_build(TARGET rs-${_n}
        PACKAGE examples CRATE ${_n}
        EXAMPLE MANIFEST "${CMAKE_SOURCE_DIR}/examples/rs/Cargo.toml"
        FOLDER "examples/rs")
    list(APPEND _rs_mediators rs-${_n}-build)
    add_custom_target(cargo-run_${_n}
      COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR}
              ${CMAKE_SOURCE_DIR}/scripts/run-gui.sh
              $<TARGET_FILE:rs-${_n}>
      VERBATIM USES_TERMINAL)  # the run-gui wrapper probes and falls back to local :0 when the IDE has no DISPLAY (CI never passes here)  # zero-argv = interactive default (registry argv is ctest-only). USES_TERMINAL rides run only (Ninja console-pool lesson)
    add_dependencies(cargo-run_${_n} rs-${_n}-build)
    set_target_properties(cargo-run_${_n} PROPERTIES FOLDER "examples/rs")
    add_test(NAME example_${_n}
      COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR}
              ${_cenv} $<TARGET_FILE:rs-${_n}> ${_args_${_n}})
    set_tests_properties(example_${_n} PROPERTIES WORKING_DIRECTORY ${CMAKE_SOURCE_DIR} LABELS "${_lbl}")
    if(PolyOrch_RUST_VSCODE_DEBUG)
      # T-E lane: run() registers the CodeLLDB launch + preLaunchTask pair.
      polyorch_rust_run(TARGET rs-${_n})
    endif()
  endforeach()

  add_custom_target(cargo-build_examples ALL)
  add_dependencies(cargo-build_examples ${_rs_mediators})
  set_target_properties(cargo-build_examples PROPERTIES FOLDER "examples")
  add_custom_target(cargo-clean_examples
    COMMAND ${_cenv} ${VISIAENGINE_CARGO} clean -p examples
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR} VERBATIM)

  # B5 developer targets: example gallery (static site) + local preview server.
  # Serve root = repo root (card source links are repo-root relative by design);
  # open http://127.0.0.1:8931/build/gallery/index.html after generating.
  find_package(Python3 COMPONENTS Interpreter QUIET)
  if(Python3_Interpreter_FOUND)
    add_custom_target(gallery
      COMMAND ${Python3_EXECUTABLE} ${CMAKE_SOURCE_DIR}/scripts/gen_gallery.py
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
      COMMENT "Generating example gallery (build/gallery/index.html)"
      VERBATIM)
    add_custom_target(gallery-serve
      COMMAND ${Python3_EXECUTABLE} -m http.server 8931 --bind 0.0.0.0
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
      COMMENT "Serving repo root at http://<host>:8931 (LAN-reachable; Ctrl-C to stop)"
      VERBATIM)
    # no FOLDER: the position-mirror rule (FOLDER = on-disk directory) has no
    # tools/ source dir; targets are identified by name (gate: cmake-smoke D14/R9).

  else()
    message(STATUS "visiaengine bindings: python3 not found - gallery targets skipped")
  endif()

  # Central disk<->table final audit: every source file under examples/{c,cpp}/ must be on the macro-registration list (the qt file self-registers the same way in examples/qt)
  set(_skip_note "")
  get_property(_reg GLOBAL PROPERTY VE_REGISTERED_EXAMPLES)
  foreach(_d c cpp)
    file(GLOB _fs "${CMAKE_SOURCE_DIR}/examples/${_d}/E[0-9][0-9][0-9]_*.c"
                  "${CMAKE_SOURCE_DIR}/examples/${_d}/E[0-9][0-9][0-9]_*.cpp")
    foreach(_f ${_fs})
      get_filename_component(_n ${_f} NAME_WE)
      if(NOT _n IN_LIST _reg)
        message(FATAL_ERROR "examples/${_d}/${_n} is unregistered (a missing visiaengine_add_example line = hard error)")
      endif()
    endforeach()
  endforeach()

  message(STATUS "visiaengine bindings: Rust=${_rs_items} · native=[registered in each examples/<lang>/CMakeLists]")
endfunction()
