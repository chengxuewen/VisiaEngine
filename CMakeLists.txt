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

if(NOT VISIAENGINE_QT_SDK STREQUAL "OFF")
  add_subdirectory(platform/qt) # Qt 轮 v1.3 落地后自然点亮（本批 bare 预设=OFF）
endif()

enable_testing()
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/cmake/probe/CMakeLists.txt")
  add_subdirectory(cmake/probe) # ctest 探针（C2 落地前自动跳过）
endif()

include(cmake/VisiaEngineExamples.cmake)
visiaengine_setup_examples()  # E 系例→IDE 可跑 target（VISIAENGINE_EXAMPLES=OFF 退役）
