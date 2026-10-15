# examples/qt -- the Qt real-window example (host-only band; E703 split from bindings/qt: the widget is binding surface and lives in bindings, the example lives in examples)
# R2 fold (adjudication 15): registration rides the visiaengine_add_example family macro -- the Qt
# generation (6 or >= 5.15) is resolved by the SDK and consumed through the VISIAENGINE_QT_*
# canonical vars inside the macro, so this file carries no generation spelling. The old hand-rolled
# block here is exactly where the E703 debug-row omission happened; the fold closes that class.
find_package(X11 REQUIRED)   # conda has no qt x11extras (either generation) -> talk to the host Xlib directly (the minimal call surface; the widget holds its own Display*)

visiaengine_add_example(E703_qt_viewer DISPLAY QT X11 ARGS "--frames;6")  # zero-arg = resident window (the -run button); the ctest leg runs 6 frames headless-ready (xcb env from the macro)
