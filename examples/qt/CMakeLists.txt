# examples/qt -- the Qt6 real-window example (host-only band; E703 split from bindings/qt: the widget is binding surface and lives in bindings, the example lives in examples)
find_package(Qt6 COMPONENTS Widgets REQUIRED)
find_package(X11 REQUIRED)   # conda has no qt6 x11extras -> talk to the host Xlib directly (the minimal call surface)
find_package(visiaengine REQUIRED)

add_executable(E703_qt_viewer E703_qt_viewer.cpp)  # name = source = target (R10)
target_link_libraries(E703_qt_viewer PRIVATE visiaengine::qt Qt6::Widgets ${X11_LIBRARIES})
# the widget header rides the visiaengine::qt face (T-H2: the raw include-dir anti-pattern retired)  # the widget.hpp relocation/split fix (S3)
if(DEFINED VISIAENGINE_CARGO_STEP)
  add_dependencies(E703_qt_viewer ${VISIAENGINE_CARGO_STEP})
endif()
set_target_properties(E703_qt_viewer PROPERTIES
  BUILD_RPATH "${Qt6_DIR}/../../../lib"
  FOLDER "examples/qt")  # vocabulary = the on-disk language directory (this file self-registers outside the macro; do not hand-write a band number)
add_test(NAME example_E703_qt_viewer COMMAND E703_qt_viewer --frames 6)
set_tests_properties(example_E703_qt_viewer PROPERTIES
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
  ENVIRONMENT "QT_QPA_PLATFORM=xcb;QT_QPA_PLATFORM_PLUGIN_PATH=${Qt6_DIR}/../../../lib/qt6/plugins"
  SKIP_RETURN_CODE 77
  LABELS "example;native;display")

# Interactive run step (same scheme as cargo-run_*/run_E801: zero args = resident window [the E703 header note, 0 = interactive]; run-gui probes and falls back to :0)
add_custom_target(run_E703_qt_viewer
  COMMAND ${CMAKE_COMMAND} -E chdir ${CMAKE_SOURCE_DIR}
          ${CMAKE_SOURCE_DIR}/scripts/run-gui.sh $<TARGET_FILE:E703_qt_viewer>
  USES_TERMINAL)
add_dependencies(run_E703_qt_viewer E703_qt_viewer)
set_target_properties(run_E703_qt_viewer PROPERTIES FOLDER "examples/qt")
