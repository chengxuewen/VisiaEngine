# 门面：IDE/消费者入口（权威构建 = cargo + pixi [D5]；本文件禁载编译规则，
# 纯度由 scripts/gate-docs.sh 断言锁）。设计与裁决：.omo/plans/visiaengine-cmake-project.md
cmake_minimum_required(VERSION 3.22)

# 版本单源：workspace Cargo.toml（iceoryx2 五处字面量之债不抄——失配=configure 硬错）
file(READ "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml" _ws)
string(REGEX MATCH "version = \"([0-9]+\\.[0-9]+\\.[0-9]+)\"" _m "${_ws}")
if(NOT _m)
  message(FATAL_ERROR "Cargo.toml workspace version 读取失败（单源合同破坏）")
endif()
project(visiaengine VERSION ${CMAKE_MATCH_1} LANGUAGES CXX)

include(cmake/VisiaEngineSDK.cmake)
visiaengine_resolve_sdks() # 汇报行：Rust/Qt 解析结果与来源（auto|pixi|system|prebuilt）

include(cmake/VisiaEngineCargo.cmake)
visiaengine_setup_cargo()  # cargo-step + 伞 visiaengine::capi + 假 Config 桥

# enable_testing() 必在全部 add_subdirectory 前（Qt 目录的 add_test 晚调即静默丢——实锤）
enable_testing()

include(cmake/VisiaEngineBindings.cmake)     # R9/R10：option + 注册宏 + rs 三件套
if(VISIAENGINE_EXAMPLES)
  enable_language(C)  # 顶层作用域（function 内不外传=PIT-16 实锤）；C 例真身需 .c 编译
  add_subdirectory(examples/c)
  add_subdirectory(examples/cpp)
endif()

if(NOT VISIAENGINE_QT_SDK STREQUAL "OFF")
  add_subdirectory(examples/qt) # Qt 宿主专带（绑定面 widget.hpp 在 bindings/qt，头-only）
endif()

if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/bindings/c/probe/CMakeLists.txt")
  add_subdirectory(bindings/c/probe) # ctest 探针（C2 落地前自动跳过）
endif()

visiaengine_setup_bindings()  # rs 三件套+全局步+中央 glob⇄表 终账（必须在全部 add_subdirectory 后）
