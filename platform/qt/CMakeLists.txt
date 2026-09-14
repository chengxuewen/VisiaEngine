# 纯消费者（Qt 轮 v1.3：伞/假 Config/cargo-step 全交 cmake-project 轮）
find_package(Qt6 COMPONENTS Widgets REQUIRED)
find_package(X11 REQUIRED)   # conda 无 qt6 x11extras→宿主 Xlib 直连（最小调用面）
find_package(visiaengine REQUIRED)

add_executable(qt_app qt_app.cpp)
target_link_libraries(qt_app PRIVATE visiaengine::capi Qt6::Widgets ${X11_LIBRARIES})
target_include_directories(qt_app PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
if(DEFINED VISIAENGINE_CARGO_STEP)
  add_dependencies(qt_app ${VISIAENGINE_CARGO_STEP})
endif()
set_target_properties(qt_app PROPERTIES
  BUILD_RPATH "${Qt6_DIR}/../../../lib") # conda Qt6 库目录（相对锚定，免全局 LD 设置）
