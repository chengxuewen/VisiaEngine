# visiaengine 例子注册法（R9/R10，v1.3）：Rust 三件套 + C/C++/Qt 原生例 + ctest 统一清单。
# 反-glob 显式注册纪律承自已故 VisiaEngineExamples.cmake（盘上有文件未注册=硬错）。
# 命名（R10）：真身=E 号原名 | 步骤=cargo-<动作>_<对象> | ctest=example_<目标基名>；rs/native 由 LABELS 分。
# 设计=.omo/plans/bindings-restructure-cpp-examples.md v1.3 §S3。

option(VISIAENGINE_EXAMPLES "examples/ 全家桶（rs 步骤 + C/C++/Qt 例 + ctest 条目）" ON)

if(VISIAENGINE_EXAMPLES AND CMAKE_VERSION VERSION_LESS 3.24)
  # 承自契约表原文（sem5 点名迁移）：SKIP_RETURN_CODE 缺位=无 X 机 display 族红而非 Skip
  message(FATAL_ERROR "examples 族需 CMake >= 3.24（ctest SKIP_RETURN_CODE：77=Skipped 非 Failed）")
endif()

# 原生例（C/C++/Qt exe）注册宏：真身=名字=ctest 条目；display 族 LABELS 交 -LE/-L 分流
function(visiaengine_add_example _n)
  cmake_parse_arguments(_A "DISPLAY;X11;QT;SDL3;CPP" "ARGS" "" ${ARGN})
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
    target_link_libraries(${_n} PRIVATE visiaengine::cpp)   # hpp 面消费=壳的活体验收
  endif()
  if(_A_SDL3)
    target_link_libraries(${_n} PRIVATE SDL3::SDL3)
    get_target_property(_sdll SDL3::SDL3 IMPORTED_LOCATION)
    if(_sdll)
      get_filename_component(_sdldir ${_sdll} DIRECTORY)      # conda 库不在系统 ld 路——rpath 自带
      set_property(TARGET ${_n} APPEND PROPERTY BUILD_RPATH ${_sdldir})
    endif()
  endif()
  if(DEFINED VISIAENGINE_CARGO_STEP)
    add_dependencies(${_n} ${VISIAENGINE_CARGO_STEP})  # 伞 .so 排序桥（probe 同款）
  endif()
  get_filename_component(_lang "${CMAKE_CURRENT_SOURCE_DIR}" NAME)  # FOLDER=磁盘语言目录同律（带号已编码在 E 号首位，不重复建词表）
  set_target_properties(${_n} PROPERTIES FOLDER "examples/${_lang}")
  add_test(NAME example_${_n} COMMAND ${_n} ${_A_ARGS})
  set_tests_properties(example_${_n} PROPERTIES
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    SKIP_RETURN_CODE 77
    LABELS "${_lbl}")
  set_property(GLOBAL APPEND PROPERTY VE_REGISTERED_EXAMPLES ${_n})
endfunction()

function(visiaengine_setup_bindings)
  if(NOT VISIAENGINE_EXAMPLES)
    return()
  endif()
  if(VISIAENGINE_ARTIFACT_PATH)
    message(STATUS "visiaengine bindings：prebuilt 伞无 cargo 面，例子域退场（probe/伞/::cpp 常驻）")
    return()
  endif()

  # ── Rust 族：argv 唯一户口（pixi smoke 只做 `ctest -R` 转发壳，抄写面归零）──
  set(_rs_items E101_clear E201_load_gltf E202_geo_viewer E203_measure_cli
                E301_switch_camera E401_pick_demo E501_shadow_demo E601_bench_twin E901_twin_city)
  set(_disp_E101_clear ON)
  set(_disp_E201_load_gltf ON)
  set(_disp_E202_geo_viewer ON)
  set(_disp_E301_switch_camera ON)
  set(_args_E101_clear "--frames;3")
  set(_args_E201_load_gltf "resources/data/twoprim.glb;--frames;3")
  set(_args_E202_geo_viewer "resources/data/park.geojson;--frames;3")
  set(_args_E301_switch_camera "resources/data/twoprim.glb;--frames;3")
  set(_args_E401_pick_demo "--frames;2")
  set(_args_E601_bench_twin "--count;5000;--frames;1")

  # 盘⇄表双向对账（examples/rs 面；表=上方 _rs_items）
  file(GLOB _rsf "${CMAKE_SOURCE_DIR}/examples/rs/E[0-9][0-9][0-9]_*.rs")
  foreach(_f ${_rsf})
    get_filename_component(_n ${_f} NAME_WE)
    if(NOT _n IN_LIST _rs_items)
      message(FATAL_ERROR "examples/rs/${_n}.rs 未入注册表（禁注释式禁用）")
    endif()
  endforeach()

  get_filename_component(_cargo_bin "${VISIAENGINE_CARGO}" DIRECTORY)  # rustc 兄弟目录（PIT-18 守卫形）
  set(_cenv ${CMAKE_COMMAND} -E env "PATH=${_cargo_bin}:$ENV{PATH}")

  foreach(_n ${_rs_items})
    if(NOT EXISTS "${CMAKE_SOURCE_DIR}/examples/rs/${_n}.rs")
      message(FATAL_ERROR "注册表项 ${_n} 盘上无件（examples/rs/${_n}.rs）")
    endif()
    if(_disp_${_n})
      set(_lbl "example;rs;display")
    else()
      set(_lbl "example;rs;headless")
    endif()
    add_custom_target(cargo-build_${_n}
      COMMAND ${_cenv} ${VISIAENGINE_CARGO} build -p examples --example ${_n}
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR} VERBATIM)
    add_custom_target(cargo-run_${_n}
      COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR}
              ${_cenv} ${VISIAENGINE_CARGO} run -p examples --example ${_n} -- ${_args_${_n}}
      VERBATIM USES_TERMINAL)  # USES_TERMINAL 只挂 run（要见声）；build/ALL 步不挂（Ninja 串行池自伤）
    foreach(_t cargo-build_${_n} cargo-run_${_n})
      set_target_properties(${_t} PROPERTIES FOLDER "examples/rs")
    endforeach()
    add_test(NAME example_${_n}
      COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR}
              ${_cenv} ${VISIAENGINE_CARGO} run -p examples --example ${_n} -- ${_args_${_n}})
    set_tests_properties(example_${_n} PROPERTIES WORKING_DIRECTORY ${CMAKE_SOURCE_DIR} LABELS "${_lbl}")
  endforeach()

  add_custom_target(cargo-build_examples ALL
    COMMAND ${_cenv} ${VISIAENGINE_CARGO} build -p examples --examples
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR} VERBATIM)
  set_target_properties(cargo-build_examples PROPERTIES FOLDER "examples/rs")
  add_custom_target(cargo-clean_examples
    COMMAND ${_cenv} ${VISIAENGINE_CARGO} clean -p examples
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR} VERBATIM)
  set_target_properties(cargo-clean_examples PROPERTIES FOLDER "examples/rs")

  # 中央盘⇄表终账：examples/{c,cpp}/ 全部源文件必须在宏注册名单内（qt 件自注册同款在 examples/qt）
  set(_skip_note "")
  get_property(_reg GLOBAL PROPERTY VE_REGISTERED_EXAMPLES)
  foreach(_d c cpp)
    file(GLOB _fs "${CMAKE_SOURCE_DIR}/examples/${_d}/E[0-9][0-9][0-9]_*.c"
                  "${CMAKE_SOURCE_DIR}/examples/${_d}/E[0-9][0-9][0-9]_*.cpp")
    foreach(_f ${_fs})
      get_filename_component(_n ${_f} NAME_WE)
      if(NOT _n IN_LIST _reg)
        message(FATAL_ERROR "examples/${_d}/${_n} 未注册（visiaengine_add_example 缺行=硬错）")
      endif()
    endforeach()
  endforeach()

  message(STATUS "visiaengine bindings：Rust=${_rs_items} · 原生=[注册于各 examples/<lang>/CMakeLists]")
endfunction()
