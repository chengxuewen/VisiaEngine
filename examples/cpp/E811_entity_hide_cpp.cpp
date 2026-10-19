// E811_entity_hide_cpp -- SDK consumption, C++ visibility twin (the hpp facade's CAPI-13/14 acceptance face).
// If it does not run, the API is unfinished [E3D:D7]. Twin lineage: examples/c/E811_entity_hide_c.c (same theme, same code).
#include <cstdio>
#include <cstdint>

#include <visiaengine/visiaengine.hpp>

int main() {
    auto eng = visiaengine::Engine::create_headless(160, 120);
    if (!eng.valid()) { puts("E811cpp FAIL create"); return 1; }
    if (eng.load_gltf("resources/data/twoprim.glb") != VE_OK) {
        printf("E811cpp FAIL load: %s\n", eng.last_error());
        return 1;
    }
    /* the mount fit chooses the framing: sweep for the first hit (the fit
     * moved twoprim off the old screen center) */
    uint64_t h = visiaengine::kMiss;
    for (int y = 0; y < 120 && h == visiaengine::kMiss; y += 2)
        for (int x = 0; x < 160 && h == visiaengine::kMiss; x += 2)
            h = eng.pick(static_cast<float>(x), static_cast<float>(y));
    if (h == visiaengine::kMiss) { puts("E811cpp FAIL nothing pickable"); return 1; }
    const float hx = 80.0f, hy = 60.0f; (void)hx; (void)hy;
    /* a pixel that hits h, for the hide/self-pick checks below */
    float px = 80.0f, py = 60.0f;
    for (int y = 0; y < 120; y += 2)
        for (int x = 0; x < 160; x += 2)
            if (eng.pick(static_cast<float>(x), static_cast<float>(y)) == h) {
                px = static_cast<float>(x); py = static_cast<float>(y);
                goto found;
            }
found:;

    if (eng.entity_visible(h) != 1) { puts("E811cpp FAIL not visible by default"); return 1; }
    if (eng.set_visible(h, false) != VE_OK) { puts("E811cpp FAIL hide"); return 1; }
    if (eng.entity_visible(h) != 0) { puts("E811cpp FAIL the query lags"); return 1; }
    if (eng.pick(px, py) == h) { puts("E811cpp FAIL hidden piece hits itself"); return 1; }
    if (eng.entity_count() != 2) { puts("E811cpp FAIL enumeration domain drifted"); return 1; }
    if (eng.set_visible(h, true) != VE_OK) { puts("E811cpp FAIL show"); return 1; }
    if (eng.pick(px, py) != h) { puts("E811cpp FAIL restore failed"); return 1; }

    puts("OK E811_entity_hide_cpp visibility=roundtrip");
    return 0;  // the destructor auto-destroys (the CAPI-03 RAII projection; moved-from double-destroy immunity lives in the hpp layer)
}
