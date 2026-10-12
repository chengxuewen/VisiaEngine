/* E704 -- binding mirror, C event callback -- the CAPI-17 push-entry acceptance example: progress/error/unregister tri-states all ride through.
 *  Build and run: ctest example_E704_host_callback (the gate/cmake-smoke road); manual run:
 *   cc -I bindings/c/visiaengine-capi/include examples/c/E704_host_callback.c \
 *      -L target/debug -lvisiaengine -o target/e704 && LD_LIBRARY_PATH=target/debug ./target/e704 */
#include <stdint.h>
#include <stdio.h>

#include "visiaengine.h"

/* Synchronous semantics = fired on the calling thread (the C-side living proof of the clause body): plain statics suffice, no atomic duty. */
static uint64_t g_done_last;
static uint64_t g_prog;
static uint64_t g_error;
static uint64_t g_last_b;

static void on_event(void *user, uint32_t event, uint64_t a, uint64_t b) {
    (void)user;
    if (event == VE_EVT_LOAD_PROGRESS) {
        if (++g_prog == 1) { g_done_last = 0; }
        if (a <= g_done_last) { puts("FAIL progress not monotonic"); }
        g_done_last = a;
        g_last_b = b;
    } else if (event == VE_EVT_LOAD_ERROR) {
        ++g_error;
        if ((int64_t)a != VE_ERR_IO) { printf("FAIL error code=%llu\n", (unsigned long long)a); }
    } else {
        printf("FAIL unknown event %u\n", event);
    }
}

int main(void) {
    if (visiaengine_abi_version() >> 16 != 1) { puts("FAIL abi major"); return 1; }
    uint64_t ve = visiaengine_create_headless(160, 120);
    if (!ve) { puts("FAIL create"); return 1; }
    if (visiaengine_set_event_callback(ve, on_event, NULL) != 0) { puts("FAIL set"); return 1; }
    /* (1) progress state: park.geojson = 5 features, the terminal event must be done == total */
    if (visiaengine_load_geojson(ve, "resources/data/park.geojson") != VE_OK) {
        printf("FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    if (g_prog < 5 || g_done_last != g_last_b) {
        printf("FAIL progress face prog=%llu terminal=(%llu,%llu)\n",
               (unsigned long long)g_prog, (unsigned long long)g_done_last,
               (unsigned long long)g_last_b);
        return 1;
    }
    /* (2) error state: missing file = the return value and LOAD_ERROR at the same instant */
    if (visiaengine_load_geojson(ve, "resources/data/does-not-exist.geojson") == VE_OK) {
        puts("FAIL the missing file unexpectedly succeeded");
        return 1;
    }
    if (g_error != 1) { printf("FAIL no error event error=%llu\n", (unsigned long long)g_error); return 1; }
    /* (3) unregister state: load after re-registering NULL = progress silent */
    const uint64_t snapshot = g_prog;
    if (visiaengine_set_event_callback(ve, NULL, NULL) != 0) { puts("FAIL null set"); return 1; }
    if (visiaengine_load_geojson(ve, "resources/data/park.geojson") != VE_OK) { puts("FAIL reload"); return 1; }
    if (g_prog != snapshot) { puts("FAIL still firing after unregister"); return 1; }
    if (visiaengine_render(ve) != VE_OK) { puts("FAIL render"); return 1; }
    printf("OK e704 events prog=%llu error=%llu\n", (unsigned long long)g_prog,
           (unsigned long long)g_error);
    return visiaengine_destroy(ve) == 0 ? 0 : 1;
}
