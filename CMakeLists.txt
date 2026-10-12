# 装配层：cmr / project / SDK 解析 / PolyOrch 装配 / options / add_subdirectory /
# install 钩子（T-A 2026-10-10 裁决 1-B+：编译规则禁入根，ALIAS 伞除外；行数软阈值
# 120 + assembly-growth 豁免标注，检查=scripts/gate-docs.sh ⑤）。设计与裁决：
# .omo/plans/visiaengine-cmake-project.md + .omo/plans/2026-10-10-visia-embed.md
cmake_minimum_required(VERSION 3.25)  # PolyOrch embed floor (D39; adjudication 5-B)

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

include(cmake/VisiaEnginePolyOrch.cmake)
visiaengine_ensure_polyorch()  # D39 哨兵复用/自嵌（first-wins；superbuild 先加则复用其份）

# enable_testing() 必在全部 add_subdirectory 前（Qt 目录的 add_test 晚调即静默丢——实锤）
enable_testing()

# Install tree (packaging-round v1.3 P2; S-c beta guard retired here -- its
# three-line message moved into README's install section, P4 accounting).
include(cmake/VisiaEngineInstall.cmake)
visiaengine_install_tree()

include(cmake/VisiaEngineBindings.cmake)     # R9/R10：option + 注册宏 + rs 三件套
if(VISIAENGINE_EXAMPLES AND NOT VISIAENGINE_ARTIFACT_PATH)
  # 预构建态=纯消费无 cargo 构建面：例子域整体退场（旧契约表同款合同；僵尸空命令目标=实测堵截）
  enable_language(C)  # 顶层作用域（function 内不外传=PIT-16 实锤）；C 例真身需 .c 编译
  add_subdirectory(examples/c)
  add_subdirectory(examples/cpp)
endif()

if(VISIAENGINE_QT)
  add_subdirectory(examples/qt) # Qt 宿主专带（绑定面 widget.hpp 在 bindings/qt，头-only）
endif()

if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/bindings/c/probe/CMakeLists.txt")
  add_subdirectory(bindings/c/probe) # ctest 探针（C2 落地前自动跳过）
endif()

visiaengine_setup_bindings()  # rs 三件套+全局步+中央 glob⇄表 终账（必须在全部 add_subdirectory 后）

include(cmake/VisiaEnginePackaging.cmake)  # CPack assembly: tarball = the install tree as-is (no new install rules)
