# 纯消费者（Qt 轮 v1.3：伞/假 Config/cargo-step 全交 cmake-project 轮）
find_package(Qt6 COMPONENTS Widgets REQUIRED)
find_package(X11 REQUIRED)   # conda 无 qt6 x11extras→宿主 Xlib 直连（最小调用面）
find_package(visiaengine REQUIRED)

add_executable(E703_qt_viewer E703_qt_viewer.cpp)  # C-2：原 qt_app，归 E7xx 宿主带（名=源=目标）
target_link_libraries(E703_qt_viewer PRIVATE visiaengine::capi Qt6::Widgets ${X11_LIBRARIES})
target_include_directories(E703_qt_viewer PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
if(DEFINED VISIAENGINE_CARGO_STEP)
  add_dependencies(E703_qt_viewer ${VISIAENGINE_CARGO_STEP})
endif()
set_target_properties(E703_qt_viewer PROPERTIES
  BUILD_RPATH "${Qt6_DIR}/../../../lib" # conda Qt6 库目录（相对锚定，免全局 LD 设置）
  FOLDER "examples/7x-宿主")
add_test(NAME example_E703_qt_viewer COMMAND E703_qt_viewer --frames 6)
set_tests_properties(example_E703_qt_viewer PROPERTIES
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
  ENVIRONMENT "QT_QPA_PLATFORM=xcb;QT_QPA_PLATFORM_PLUGIN_PATH=${Qt6_DIR}/../../../lib/qt6/plugins"  # conda Qt 插件路configure 锤定
  LABELS "example;display")  # 需 X：无显示真窗红——本目录 ctest 由 smoke-qt 三态门守；bare 族不涵盖本件（不同构建立录）
