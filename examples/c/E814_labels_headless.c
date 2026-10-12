/* E814_labels_headless -- SDK consumption, C text labels (the CAPI-21/22 living gate, tier 1).
 *  Headless evidence identity (same system as E813): timing gate -> font injection -> dual-context labels -> readback white-ink count, exits in seconds. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "visiaengine.h"

#define VW 160
#define VH 120

static int fail(const char *what) {
    printf("E814 FAIL %s\n", what);
    return 1;
}

int main(void) {
    uint64_t ve = visiaengine_create_headless(VW, VH);
    if (!ve) return fail("create");
    VeLabelSpec sp = {0};
    sp.struct_size = sizeof sp;
    sp.pos[0] = 0.0; sp.pos[1] = 0.0; sp.pos[2] = 0.5;
    sp.color[0] = sp.color[1] = sp.color[2] = 1.0f;
    sp.color[3] = 1.0f;
    sp.size_px = 24.0f;
    sp.text = "VISIA";
    uint64_t h = 0;
    if (visiaengine_add_label(ve, &sp, &h) != VE_ERR_ARG) return fail("timing gate (must reject before the font is loaded)");
    FILE *f = fopen("resources/data/DejaVuSans.ttf", "rb");
    if (!f) return fail("open fixture font");
    static unsigned char buf[1024 * 1024];
    size_t n = fread(buf, 1, sizeof buf, f);
    fclose(f);
    if (n == 0 || n == sizeof buf) return fail("font size");
    if (visiaengine_load_font(ve, buf, n) != VE_OK) return fail("load_font");
    if (visiaengine_add_label(ve, &sp, &h) != VE_OK) return fail("add_label");
    sp.pos[1] = 8.0;
    sp.text = "塔-07"; /* UTF-8 Chinese covered by DejaVu (no CJK glyph = adv-only zero patches, the whole entry must still return 0) */
    uint64_t h2 = 0;
    if (visiaengine_add_label(ve, &sp, &h2) != VE_OK) return fail("utf8 multibyte");
    if (visiaengine_render(ve) != VE_OK) return fail("render");
    static uint8_t img[VW * VH * 4];
    if (visiaengine_readback(ve, img, sizeof img) != VE_OK) return fail("readback");
    long white = 0;
    for (long i = 0; i < (long)VW * VH; ++i) {
        if (img[i * 4] > 120 && img[i * 4 + 1] > 120 && img[i * 4 + 2] > 120) white++;
    }
    if (white < 60) fail("white ink absent (label not rendered)"); /* probe-measured threshold [PIT-8] */
    printf("OK e814 labels white=%ld gate=timing-gate+utf8\n", white);
    if (visiaengine_destroy(ve) != VE_OK) return fail("destroy");
    return 0;
}
