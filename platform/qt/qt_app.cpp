// qt_app —— Qt6 真窗 demo（[E3D:C3] 几行接入配方兑现；跑不通即 API 未完成 [E3D:D7]）。
// 用法：qt_app [--frames N]（0=交互）；载入 texquad.glb（纹理族）+ park.geojson（扩片族）。
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

    QApplication app(argc, argv); // 唯一线程=引擎 owner（CAPI-03）
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
    w.setWindowTitle("VisiaEngine × Qt6 — texquad + park（鼠标轨道/滚轮缩放/拖窗 resize）");
    w.show();
    w.start(frames);
    const int rc = app.exec();
    if (visiaengine_destroy(ve) != VE_OK) {
        std::fprintf(stderr, "FAIL destroy\n");
        return 1;
    }
    std::printf("OK qt pump frames=%d rc=%d\n", frames, rc);
    std::printf("T3 人检清单：①棋盘纹理四边形可见且色=checker ②park 描边/圆点覆盖其上"
                " ③鼠标左键拖=轨道、滚轮=缩放常数不随 zoom 变粗 ④拖窗 resize 无残影无花屏\n");
    return rc;
}
