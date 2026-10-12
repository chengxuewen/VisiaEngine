/* E811_entity_hide_c -- SDK consumption, C visibility twin (the CAPI-13/14 host-face acceptance).
 *  If it does not run, the API is unfinished [E3D:D7]. Twin lineage: examples/cpp/E811_entity_hide_cpp.cpp.
 *
 *  Contract trio: a hidden piece does not hit itself on pick / the enumeration domain is unchanged / visibility query round-trip (with idempotence and out-of-range rejection).
 *  Shape 0 is legal (the deliberate CAPI-01 division) -- existence criteria use only query/enumeration receipts, never value-by-0. */
#include <stdio.h>

#include "visiaengine.h"

int main(void) {
    uint64_t ve = visiaengine_create_headless(160, 120);
    if (!ve) { puts("E811 FAIL create"); return 1; }
    if (visiaengine_load_gltf(ve, "resources/data/twoprim.glb") != VE_OK) {
        printf("E811 FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    uint64_t h = visiaengine_pick(ve, 80.0f, 60.0f);
    if (h == VE_MISS) { puts("E811 FAIL no entity at center (geometry baseline drift)"); return 1; }

    if (visiaengine_entity_visible(ve, h) != 1) { puts("E811 FAIL not visible by default"); return 1; }
    if (visiaengine_entity_set_visible(ve, h, 0) != VE_OK) { puts("E811 FAIL hide"); return 1; }
    if (visiaengine_entity_visible(ve, h) != 0) { puts("E811 FAIL the query does not reflect the hide"); return 1; }
    if (visiaengine_entity_set_visible(ve, h, 0) != VE_OK) { puts("E811 FAIL idempotent re-set"); return 1; }
    if (visiaengine_pick(ve, 80.0f, 60.0f) == h) { puts("E811 FAIL hidden piece still hits itself"); return 1; }
    if (visiaengine_entity_count(ve) != 2) { puts("E811 FAIL enumeration domain polluted by visibility"); return 1; }
    if (visiaengine_entity_set_visible(ve, h, 2) != VE_ERR_ARG) { puts("E811 FAIL out-of-range value not rejected"); return 1; }
    if (visiaengine_entity_set_visible(ve, 0xdeadbeefULL, 0) != VE_ERR_ARG) {
        puts("E811 FAIL unknown shape not rejected"); return 1;
    }
    if (visiaengine_entity_set_visible(ve, h, 1) != VE_OK) { puts("E811 FAIL show"); return 1; }
    if (visiaengine_pick(ve, 80.0f, 60.0f) != h) { puts("E811 FAIL no hit after restore"); return 1; }

    if (visiaengine_destroy(ve) != VE_OK) { puts("E811 FAIL destroy"); return 1; }
    puts("OK E811_entity_hide_c visibility=roundtrip");
    return 0;
}
