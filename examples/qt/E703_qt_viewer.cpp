// E703_qt_viewer — Qt6 real-window demo (host band of the E series; formerly qt_app) ([E3D:C3] the few-line integration recipe, delivered; if it does not run, the API is not done [E3D:D7]).
// Usage: E703_qt_viewer [--frames N] (0 = interactive); loads texquad.glb (texture family) + park.geojson (expansion-slice family).
#include <QApplication>
#include <cstdio>
#include <cstdlib>
#include <string>

#include "visiaengine_widget.hpp"

int main(int argc, char** argv) {
    int frames = 0;
    for (int i = 1; i < argc; ++i) {
        if (std::string(argv[i]) == "--frames" && i + 1 < argc) {
            frames = std::atoi(argv[++i]);
        }
    }

    QApplication app(argc, argv); // single thread = engine owner (CAPI-03)
    const uint64_t ve = visiaengine_create_headless(960, 600);
    if (!ve) {
        std::fprintf(stderr, "FAIL create: %s\n", visiaengine_last_error(0));
        return 1;
    }
    if (visiaengine_load_gltf(ve, "resources/data/texquad.glb") != VE_OK ||
        visiaengine_load_geojson(ve, "resources/data/park.geojson") != VE_OK) {
        std::fprintf(stderr, "FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }

    VisiaEngineWidget w(ve);
    w.resize(960, 600);
    w.setWindowTitle("VisiaEngine × Qt6 — texquad + park (mouse orbit / wheel zoom / window-drag resize)");
    w.show();
    w.start(frames);
    const int rc = app.exec();
    if (visiaengine_destroy(ve) != VE_OK) {
        std::fprintf(stderr, "FAIL destroy\n");
        return 1;
    }
    std::printf("OK qt pump frames=%d rc=%d\n", frames, rc);
    std::printf("T3 manual checklist: (1) checker-textured quad visible with colour = checker (2) park outline/dot overlay on top"
                " (3) left-drag = orbit, wheel = zoom with line width constant, not thickening with zoom (4) window-drag resize leaves no ghosts, no glitches\n");
    return rc;
}
