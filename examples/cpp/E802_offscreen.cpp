// E802_offscreen -- SDK consumption, C++ headless example (the 8x band; the hpp facade's living acceptance).
// Contract: readback magnitude probe (the CAPI-05 -5 form) -> center luminance assertion (render_spec same-source lum > 40)
// -> PPM written to target/e802_out.ppm (hand-written ten-line P6, zero new dependencies) + reread header check.
// If it does not run, the API is unfinished [E3D:D7]; the shell-semantics contract lives in the hpp header comment (B9: no SDD account).
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
    if (!eng.valid()) return fail("create", nullptr);  // create failed = no thread-diagnostic handle to query (CAPI-04)

    const char *path = (argc > 1) ? argv[1] : "resources/data/twoprim.glb";
    if (eng.load_gltf(path) != VE_OK) return fail("load", eng.last_error());
    if (eng.render() != VE_OK) return fail("render", eng.last_error());

    std::vector<uint8_t> buf(static_cast<size_t>(W) * H * 4);
    if (eng.readback(nullptr, 0) != VE_ERR_SIZE) return fail("size-probe semantics drift", nullptr);
    if (eng.readback(buf.data(), buf.size()) != VE_OK) return fail("readback", eng.last_error());

    const size_t mid = (static_cast<size_t>(H / 2) * W + W / 2) * 4;  // RGBA
    const unsigned lum = buf[mid] + buf[mid + 1] + buf[mid + 2];
    if (lum <= 40) return fail("the center pixel backgrounded", nullptr);

    std::ofstream ppm("target/e802_out.ppm", std::ios::binary);
    if (!ppm) return fail("ppm open (target/ must exist at the repo root)", nullptr);
    ppm << "P6\n" << W << " " << H << "\n255\n";
    for (size_t i = 0; i < static_cast<size_t>(W) * H; ++i)
        ppm << buf[i * 4] << buf[i * 4 + 1] << buf[i * 4 + 2];
    ppm.close();

    std::ifstream back("target/e802_out.ppm", std::ios::binary);
    std::string head;
    getline(back, head);
    if (head != "P6") return fail("ppm header check", head.c_str());

    std::printf("OK E802_offscreen center_lum=%u bytes=%zu\n", lum, buf.size());
    return 0;
}
