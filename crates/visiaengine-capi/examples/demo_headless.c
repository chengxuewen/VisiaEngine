/* demo_headless.c —— canonical 10 行嵌入样板（文档件，[FFI-R:EP-附]）。
 * 构建与运行见 scripts/gate-abi.sh；亦可直接：
 *   x86_64-conda-linux-gnu-cc -I include demo_headless.c -L target/debug \
 *     -lvisiaengine -o target/demo_headless && LD_LIBRARY_PATH=target/debug ./target/demo_headless */
#include <stddef.h>
#include <stdio.h>

#include "visiaengine.h"

int main(int argc, char **argv) {
    if (visiaengine_abi_version() >> 16 != 1) { puts("FAIL abi major"); return 1; } /* ABI 门 [FFI-R:v13-FEAS-2] */
    uint64_t ve = visiaengine_create_headless(160, 120);
    if (!ve) { puts("FAIL create"); return 1; }
    if (visiaengine_load_gltf(ve, argc > 1 ? argv[1] : "resources/data/twoprim.glb") != VE_OK) {
        printf("FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    for (int i = 0; i < 3; ++i) {
        if (visiaengine_render(ve) != VE_OK) { puts("FAIL render"); return 1; }
    }
    static unsigned char buf[160 * 120 * 4];
    if (visiaengine_readback(ve, buf, sizeof buf) != VE_OK) { puts("FAIL readback"); return 1; }
    size_t mid = (size_t)((60 * 160 + 80) * 4);
    unsigned lum = (unsigned)(buf[mid] + buf[mid + 1] + buf[mid + 2]);
    if (lum <= 40) { printf("FAIL dark center lum=%u\n", lum); return 1; }
    uint64_t hit = visiaengine_pick(ve, 80.0f, 60.0f);
    if (hit == VE_MISS) { puts("FAIL pick miss on lit pixel"); return 1; }
    visiaengine_destroy(ve);
    puts("OK capi headless");
    return 0;
}
