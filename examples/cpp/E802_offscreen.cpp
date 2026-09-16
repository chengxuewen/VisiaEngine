// E802_offscreen —— SDK 消费·C++ headless 例（8x 带；hpp 门面的活体验收）。
// 合同：readback 量值探针（CAPI-05 -5 形）→ 中心亮度断言（render_spec 同源 lum>40）
// → PPM 落盘 target/e802_out.ppm（P6 十行手写零新依赖）+ 复读头校验。
// 跑不通即 API 未完成 [E3D:D7]；壳语义合同在 hpp 头注释（B9：不开 SDD 账）。
#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <vector>

#include <visiaengine/visiaengine.hpp>

namespace {
int fail(const char *what, const char *detail) {
    std::printf("E802 FAIL %s: %s\n", what, detail ? detail : "");
    return 1;
}
}  // namespace

int main(int argc, char **argv) {
    constexpr uint32_t W = 160, H = 120;
    auto eng = visiaengine::Engine::create_headless(W, H);
    if (!eng.valid()) return fail("create", nullptr);  // create 未成=无线程诊断句柄可查（CAPI-04）

    const char *path = (argc > 1) ? argv[1] : "resources/data/twoprim.glb";
    if (eng.load_gltf(path) != VE_OK) return fail("load", eng.last_error());
    if (eng.render() != VE_OK) return fail("render", eng.last_error());

    std::vector<uint8_t> buf(static_cast<size_t>(W) * H * 4);
    if (eng.readback(nullptr, 0) != VE_ERR_SIZE) return fail("size-probe 语义漂移", nullptr);
    if (eng.readback(buf.data(), buf.size()) != VE_OK) return fail("readback", eng.last_error());

    const size_t mid = (static_cast<size_t>(H / 2) * W + W / 2) * 4;  // RGBA
    const unsigned lum = buf[mid] + buf[mid + 1] + buf[mid + 2];
    if (lum <= 40) return fail("中心像素背景化", nullptr);

    std::ofstream ppm("target/e802_out.ppm", std::ios::binary);
    if (!ppm) return fail("ppm 打开（target/ 需在仓根）", nullptr);
    ppm << "P6\n" << W << " " << H << "\n255\n";
    for (size_t i = 0; i < static_cast<size_t>(W) * H; ++i)
        ppm << buf[i * 4] << buf[i * 4 + 1] << buf[i * 4 + 2];
    ppm.close();

    std::ifstream back("target/e802_out.ppm", std::ios::binary);
    std::string head;
    getline(back, head);
    if (head != "P6") return fail("ppm 头校验", head.c_str());

    std::printf("OK E802_offscreen center_lum=%u bytes=%zu\n", lum, buf.size());
    return 0;
}
