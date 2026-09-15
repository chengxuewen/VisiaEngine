# E 系 examples → IDE 可运行 target + ctest 族（C-1=stub launcher 裁决落地）
# 设计=.omo/plans/cmake-example-targets.md v1.1；Easy3D 采纳面：名=产物=标题、
# FOLDER 分组、中心函数、严格注册表（反其注释式禁用：glob 无表项=configure 硬错）
#
# 合同：
#  - Rust 产物目录 = VISIAENGINE_RUST_DIR/<profile>/examples/（与 cargo-step 共享
#    增量面；unix 无哈希硬链 S1 实证 inode 相同）；每 crate 单发 cargo（同包多
#    --example 合法，跨包 -p×--example 是交叉乘积会报 no example target）
#  - C demo 真身=原生 ve_real_ target 链伞 visiaengine::capi（rpath 免 LD_LIBRARY_PATH）
#  - launcher stub：chdir 仓根（G4）+ 显示族 DISPLAY 预检 exit 77（G6，ctest 记
#    Skipped）+ argv 逐字转发——pixi smoke 十一路零改动双轨保底
#  - ctest 参数照抄 pixi.toml smoke 行（Momus A-1：E203/E701/E702 argv[1]=数据路径，
#    空=走二进制内默认相对路径，禁全局 --frames 一刀切）

option(VISIAENGINE_EXAMPLES "E 系 example → IDE target + ctest（默认 ON）" ON)
if(VISIAENGINE_EXAMPLES)
  enable_language(C)  # C demo 真身需 .c 编译——必顶文件作用域（function 内 enable_language 不外传，实锤）
endif()

# 契约表：_ex_<name>=crate（Rust）或 "capi-c"（C demo）；_args_<name>=ctest 参数
# （;分隔，照抄 smoke 行）；显示族另登记 _ex_DISPLAY_<name>
set(_ex_E101_clear         "visiaengine-render-wgpu")
set(_args_E101_clear       "--frames;3")
set(_ex_DISPLAY_E101_clear ON)
set(_ex_E201_load_gltf     "visiaengine-render-wgpu")
set(_args_E201_load_gltf   "resources/data/twoprim.glb;--frames;3")
set(_ex_DISPLAY_E201_load_gltf ON)
set(_ex_E202_geo_viewer    "visiaengine-render-wgpu")
set(_args_E202_geo_viewer  "resources/data/park.geojson;--frames;3")
set(_ex_DISPLAY_E202_geo_viewer ON)
set(_ex_E203_measure_cli   "visiaengine-geo")
set(_args_E203_measure_cli "")
set(_ex_E301_switch_camera "visiaengine-render-wgpu")
set(_args_E301_switch_camera "resources/data/twoprim.glb;--frames;3")
set(_ex_DISPLAY_E301_switch_camera ON)
set(_ex_E401_pick_demo     "visiaengine-render-wgpu")
set(_args_E401_pick_demo   "--frames;2")
set(_ex_E501_shadow_demo   "visiaengine-render-wgpu")
set(_args_E501_shadow_demo "")
set(_ex_E601_bench_twin    "visiaengine-render-wgpu")
set(_args_E601_bench_twin  "--count;5000;--frames;1")  # debug 时长帽，仿 smoke-bench-twin
set(_ex_E901_twin_city     "visiaengine-render-wgpu")
set(_args_E901_twin_city   "")
set(_ex_E701_demo_headless "capi-c")
set(_args_E701_demo_headless "")
set(_ex_E702_demo_x11      "capi-c")
set(_args_E702_demo_x11    "")
set(_ex_DISPLAY_E702_demo_x11 ON)

set(_ve_band_1 "examples/1x-基础")
set(_ve_band_2 "examples/2x-数据")
set(_ve_band_3 "examples/3x-相机")
set(_ve_band_4 "examples/4x-交互")
set(_ve_band_5 "examples/5x-光影")
set(_ve_band_6 "examples/6x-规模")
set(_ve_band_7 "examples/7x-宿主")
set(_ve_band_9 "examples/9x-切片")

# stub = IDE Run 真身 + ctest 一条（同一入口，无平行测试路径）
function(_ve_stub _n _realpath _dep _nd _args)
  string(SUBSTRING ${_n} 1 1 _band)  # E1xx → 1
  if(_nd)
    set(_kind "display")
  else()
    set(_kind "headless")
  endif()
  add_executable(${_n} "${CMAKE_SOURCE_DIR}/cmake/ve_example_launcher.cpp")
  target_compile_definitions(${_n} PRIVATE
    VE_TARGET="${_realpath}"
    VE_REPO="${CMAKE_SOURCE_DIR}"
    VE_NEEDS_DISPLAY=${_nd})
  set_target_properties(${_n} PROPERTIES FOLDER "${_ve_band_${_band}}")
  add_dependencies(${_n} ${_dep})
  add_test(NAME example_${_n} COMMAND ${_n} ${_args})
  set_tests_properties(example_${_n} PROPERTIES
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    SKIP_RETURN_CODE 77
    LABELS "example;${_kind}")
endfunction()

function(visiaengine_setup_examples)
  if(NOT VISIAENGINE_EXAMPLES)
    return()
  endif()
  if(VISIAENGINE_ARTIFACT_PATH)
    message(STATUS "VISIAENGINE_EXAMPLES：prebuilt 伞路径无 cargo 构建面，跳过")
    return()
  endif()
  if(CMAKE_VERSION VERSION_LESS 3.24)
    message(FATAL_ERROR "examples 族需 CMake >= 3.24（ctest SKIP_RETURN_CODE：无 display 机 77=Skipped 非 Failed）")
  endif()

  # ── Rust 族：glob ⇄ 表校验（盘上每件须命名合规 + 有表项）──
  set(_items "")
  foreach(_crate visiaengine-render-wgpu visiaengine-geo)
    file(GLOB _rs "${CMAKE_SOURCE_DIR}/crates/${_crate}/examples/*.rs")
    list(SORT _rs)
    foreach(_f ${_rs})
      get_filename_component(_n ${_f} NAME_WE)
      if(NOT _n MATCHES "^E[0-9][0-9][0-9]_[A-Za-z0-9_]+$")
        message(FATAL_ERROR "example 文件命名不合规（^E<NNN>_name$）：${_f}")
      endif()
      if(NOT DEFINED _ex_${_n})
        message(FATAL_ERROR "example '${_n}' 未入契约表（禁注释式禁用）")
      endif()
      list(APPEND _items ${_n})
    endforeach()
  endforeach()

  # 构建步：按 crate 分组，每组单发 cargo（共享 target/cmake-rust 增量面）
  get_filename_component(_cargo_bin "${VISIAENGINE_CARGO}" DIRECTORY)  # rustc 兄弟目录入 PATH（IDE 直调守卫，cargo-step 同款）
  set(_rflags "")
  if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(_rflags "--release")
  endif()
  set(_ex_dir "${VISIAENGINE_RUST_DIR}/${VISIAENGINE_RUST_PROFILE}/examples")
  set(_bins "")
  foreach(_n ${_items})
    list(APPEND _bins "${_ex_dir}/${_n}")
  endforeach()
  set(_cmds "")
  foreach(_crate visiaengine-render-wgpu visiaengine-geo)
    set(_sel "")
    foreach(_n ${_items})
      if(_ex_${_n} STREQUAL ${_crate})
        list(APPEND _sel --example ${_n})
      endif()
    endforeach()
    if(_sel)
      list(APPEND _cmds COMMAND ${CMAKE_COMMAND} -E env "PATH=${_cargo_bin}:$ENV{PATH}"
                   ${VISIAENGINE_CARGO} build ${_rflags}
                   --target-dir ${VISIAENGINE_RUST_DIR} -p ${_crate} ${_sel})
    endif()
  endforeach()
  add_custom_target(visiaengine-examples-step ALL
    ${_cmds}
    BYPRODUCTS ${_bins}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    VERBATIM USES_TERMINAL)
  foreach(_n ${_items})
    if(_ex_DISPLAY_${_n})
      set(_nd 1)
    else()
      set(_nd 0)
    endif()
    _ve_stub(${_n} "${_ex_dir}/${_n}" visiaengine-examples-step ${_nd} "${_args_${_n}}")
  endforeach()

  # ── C demo 族：真身=原生 target 链伞（rpath 免 LD_LIBRARY_PATH），E702 需 X11 ──
  find_package(X11 QUIET)   # E702 消费面；缺位=该件优雅跳过（双层门控）
  file(GLOB _cs "${CMAKE_SOURCE_DIR}/crates/visiaengine-capi/examples/*.c")
  list(SORT _cs)
  set(_realdir "${CMAKE_BINARY_DIR}/ve-real")
  set(_cok "")
  foreach(_f ${_cs})
    get_filename_component(_n ${_f} NAME_WE)
    if(NOT _n MATCHES "^E[0-9][0-9][0-9]_[A-Za-z0-9_]+$")
      message(FATAL_ERROR "example 文件命名不合规（^E<NNN>_name$）：${_f}")
    endif()
    if(NOT DEFINED _ex_${_n})
      message(FATAL_ERROR "example '${_n}' 未入契约表（禁注释式禁用）")
    endif()
    if(_ex_DISPLAY_${_n})
      set(_nd 1)
    else()
      set(_nd 0)
    endif()
    if(_ex_DISPLAY_${_n} AND NOT X11_FOUND)
      message(STATUS "example ${_n}：需 X11 而 find_package 未命中，跳过（tools 装 xorg-libx11 后点亮）")
      continue()
    endif()
    add_executable(ve_real_${_n} ${_f})
    target_link_libraries(ve_real_${_n} PRIVATE visiaengine::capi)
    if(_ex_DISPLAY_${_n})
      target_link_libraries(ve_real_${_n} PRIVATE X11::X11)
    endif()
    set_target_properties(ve_real_${_n} PROPERTIES
      RUNTIME_OUTPUT_DIRECTORY ${_realdir} OUTPUT_NAME ${_n} FOLDER "examples/_producers")
    # OUTPUT_NAME：产物文件=规范名（target 保留 ve_real_ 前缀免与 stub 撞名）
    add_dependencies(ve_real_${_n} ${VISIAENGINE_CARGO_STEP})  # 伞 .so 排序桥（probe 同款）
    _ve_stub(${_n} "${_realdir}/${_n}${CMAKE_EXECUTABLE_SUFFIX}"
             ve_real_${_n} ${_nd} "${_args_${_n}}")
    list(APPEND _cok ${_n})
  endforeach()

  list(LENGTH _items _nr)
  list(LENGTH _cok _nc)
  message(STATUS "visiaengine examples：Rust=${_nr} · C=${_nc} → IDE target + ctest（L=example）")
endfunction()
