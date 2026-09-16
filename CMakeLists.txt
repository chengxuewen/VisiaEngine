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

# install 门面=打包轮硬债（S-c β 终裁）：零 install() 规则期拒绝说谎式 exit-0（先例=树内掩盖两案）
if(NOT VISIAENGINE_EXPERIMENTAL_INSTALL)
  install(CODE "message(FATAL_ERROR \"visiaengine: install tree not supported (pre-1.0)\\n  开发态: add_subdirectory 本仓根（visiaengine::capi / ::cpp 伞）\\n  预构建: -DVISIAENGINE_ARTIFACT_PATH=<libvisiaengine.so 目录>\\n  发布态: 候打包轮计划（台账 §7 硬债）\")")
endif()

include(cmake/VisiaEngineBindings.cmake)     # R9/R10：option + 注册宏 + rs 三件套
if(VISIAENGINE_EXAMPLES AND NOT VISIAENGINE_ARTIFACT_PATH)
  # 预构建态=纯消费无 cargo 构建面：例子域整体退场（旧契约表同款合同；僵尸空命令目标=实测堵截）
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
