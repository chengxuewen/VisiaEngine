# examples/qt —— Qt6 真窗例（宿主专带；E703 自 bindings/qt 分家：widget 是绑定面住 bindings，例子住 examples）
find_package(Qt6 COMPONENTS Widgets REQUIRED)
find_package(X11 REQUIRED)   # conda 无 qt6 x11extras→宿主 Xlib 直连（最小调用面）
find_package(visiaengine REQUIRED)

add_executable(E703_qt_viewer E703_qt_viewer.cpp)  # 名=源=目标（R10）
target_link_libraries(E703_qt_viewer PRIVATE visiaengine::capi Qt6::Widgets ${X11_LIBRARIES})
target_include_directories(E703_qt_viewer PRIVATE ${CMAKE_SOURCE_DIR}/bindings/qt)  # widget.hpp 搬家分家修正（S3）
if(DEFINED VISIAENGINE_CARGO_STEP)
  add_dependencies(E703_qt_viewer ${VISIAENGINE_CARGO_STEP})
endif()
set_target_properties(E703_qt_viewer PROPERTIES
  BUILD_RPATH "${Qt6_DIR}/../../../lib"
  FOLDER "examples/qt")  # 词汇表=磁盘语言目录（本件走宏外自注册，勿再手写带号）
add_test(NAME example_E703_qt_viewer COMMAND E703_qt_viewer --frames 6)
set_tests_properties(example_E703_qt_viewer PROPERTIES
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
  ENVIRONMENT "QT_QPA_PLATFORM=xcb;QT_QPA_PLATFORM_PLUGIN_PATH=${Qt6_DIR}/../../../lib/qt6/plugins"
  SKIP_RETURN_CODE 77
  LABELS "example;native;display")
