// template.cpp -- headless example, C++ raw blank skeleton (B8: copying a file inherits no library; new examples cp then edit the theme zone,
// rewriting the first line as // E8xx_name -- title keeps the triple lock. If it does not run, the API is unfinished [E3D:D7].)
#include <cstdio>
#include <vector>

#include <visiaengine/visiaengine.hpp>

int main(void) {
    // -- skeleton zone: the RAII engine (move-only; the destructor auto-destroys; no-throw = return codes pass through the shell) --
    auto eng = visiaengine::Engine::create_headless(160, 120);
    if (!eng.valid()) { puts("create failed"); return 1; }

    // -- theme zone: load/render/readback/assert (see E802: the magnitude probe -5 form + center luminance lum > 40) --
    if (eng.render() != VE_OK) { printf("render: %s\n", eng.last_error()); return 1; }
    std::vector<uint8_t> buf(160 * 120 * 4);
    if (eng.readback(buf.data(), buf.size()) != VE_OK) { puts("readback"); return 1; }
    // -- theme zone end --
    puts("OK E8xx_template");
    return 0;
}
