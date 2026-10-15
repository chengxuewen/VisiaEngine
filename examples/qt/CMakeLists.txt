# examples/qt -- the Qt6 real-window example (host-only band; E703 split from bindings/qt: the widget is binding surface and lives in bindings, the example lives in examples)
find_package(Qt6 COMPONENTS Widgets REQUIRED)
find_package(X11 REQUIRED)   # conda has no qt6 x11extras -> talk to the host Xlib directly (the minimal call surface)
find_package(visiaengine REQUIRED)

# T-H6: the target carries the example head word; the FILE keeps the bare E name (R10 amended)
add_executable(visiaengine-example-E703_qt_viewer E703_qt_viewer.cpp)
target_link_libraries(visiaengine-example-E703_qt_viewer PRIVATE visiaengine::qt Qt6::Widgets ${X11_LIBRARIES})
# the widget header rides the visiaengine::qt face (T-H2: the raw include-dir anti-pattern retired)  # the widget.hpp relocation/split fix (S3)
if(DEFINED VISIAENGINE_FFI_STEP)
  add_dependencies(visiaengine-example-E703_qt_viewer ${VISIAENGINE_FFI_STEP})
endif()
set_target_properties(visiaengine-example-E703_qt_viewer PROPERTIES
  BUILD_RPATH "${Qt6_DIR}/../../../lib"
  FOLDER "examples/qt")  # vocabulary = the on-disk language directory (this file self-registers outside the macro; do not hand-write a band number)
add_test(NAME example_E703_qt_viewer COMMAND visiaengine-example-E703_qt_viewer --frames 6)
set_tests_properties(example_E703_qt_viewer PROPERTIES
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
  ENVIRONMENT "QT_QPA_PLATFORM=xcb;QT_QPA_PLATFORM_PLUGIN_PATH=${Qt6_DIR}/../../../lib/qt6/plugins"
  SKIP_RETURN_CODE 77
  LABELS "example;native;display")

# Interactive run step (same scheme as the visiaengine-example-*-run family: zero args = resident window [the E703 header note, 0 = interactive]; the polyorch run surface probes and falls back to :0)
polyorch_example_run(TARGET visiaengine-example-E703_qt_viewer GUI
  WORKING_DIRECTORY ${CMAKE_SOURCE_DIR} FOLDER "examples/qt")
add_dependencies(visiaengine-example-E703_qt_viewer-run visiaengine-example-E703_qt_viewer)

# T-H8 native face: F5 for the Qt example
if(COMMAND polyorch_native_debug)
  _polyorch_vscode_debug_gate(NATIVE _qdebug)
  if(_qdebug)
    polyorch_native_debug(TARGET visiaengine-example-E703_qt_viewer
        CWD "${CMAKE_SOURCE_DIR}" SEG "debug/qt")
  endif()
endif()
