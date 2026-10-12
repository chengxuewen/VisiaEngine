/* E813_section_clip -- SDK consumption, C section clipping (the CAPI-20 host-face acceptance).
 *  Headless evidence identity (same system as E811): readback pixel-family counts in three acts = baseline / half-cut / restore, exits in seconds.
 *  Note: capi pick = positions x IDENTITY frame (origin lives on the render path = established fact) --
 *  z is baked into positions to merge the two frames, do not learn it as a bug. */
#include <stdio.h>
#include <stdlib.h>

#include "visiaengine.h"

#define VW 160
#define VH 120

static const float POS_BACK[12] = {-4.0f, -4.0f, 0.0f, 4.0f, -4.0f, 0.0f,
                                   4.0f,  4.0f,  0.0f, -4.0f, 4.0f, 0.0f};
static const float POS_FRONT[12] = {-4.0f, -4.0f, 2.0f, 4.0f, -4.0f, 2.0f,
                                    4.0f,  4.0f,  2.0f, -4.0f, 4.0f, 2.0f};
static const float NRM[12] = {0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.0f,
                              0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.0f};
static const uint32_t IDX[6] = {0, 1, 2, 0, 2, 3};

static int fail(const char *what) {
    printf("E813 FAIL %s\n", what);
    return 1;
}

static int add_wall(uint64_t ve, const float *pos, const float *color, uint64_t *out) {
    VeMeshDesc d = {0};
    double o0[3] = {0.0, 0.0, 0.0}; /* positions already carry z (two frames merged) -> origin stays zero */
    d.struct_size = sizeof(VeMeshDesc);
    d.positions = pos;
    d.normals = NRM;
    d.indices = IDX;
    d.n_positions = 4;
    d.n_indices = 6;
    d.base_color = color;
    d.origin = o0;
    return visiaengine_add_mesh(ve, &d, out);
}

static long count_fam(const uint8_t *img, int red) {
    long n = 0;
    for (long i = 0; i < (long)VW * VH; ++i) {
        uint8_t r = img[i * 4], g = img[i * 4 + 1], b = img[i * 4 + 2];
        if (red ? (r > 120 && g < 80 && b < 80) : (g > 120 && r < 100 && b < 100)) n++;
    }
    return n;
}

int main(void) {
    uint64_t ve = visiaengine_create_headless(VW, VH);
    if (!ve) return fail("create");
    static const float RED[4] = {1.0f, 0.0f, 0.0f, 1.0f};
    static const float GREEN[4] = {0.0f, 1.0f, 0.0f, 1.0f};
    uint64_t front = 0, back = 0;
    if (add_wall(ve, POS_FRONT, RED, &front) != VE_OK) return fail("add front");
    if (add_wall(ve, POS_BACK, GREEN, &back) != VE_OK) return fail("add back");
    uint8_t *img = malloc((size_t)VW * VH * 4);
    if (!img) return fail("oom");

    /* baseline: the front wall's red spreads the center; pick hits the front */
    if (visiaengine_render(ve) != VE_OK) return fail("render0");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback0");
    long r0 = count_fam(img, 1);
    if (r0 < 3000) return fail("the red-wall baseline is absent");
    if (visiaengine_pick(ve, 80.0f, 60.0f) != front) return fail("the front wall not hit by default");

    /* half knife z <= 1: red all dead (the z = 2 face discards whole patches), green unruffled, pick's retry loop hits the back wall */
    VeClipPlane p = {0.0, 0.0, -1.0, 1.0};
    if (visiaengine_set_clips(ve, &p, 1) != VE_OK) return fail("set_clips");
    if (visiaengine_get_clips(ve, NULL, 0) != 1) return fail("readback count");
    if (visiaengine_render(ve) != VE_OK) return fail("render1");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback1");
    if (count_fam(img, 1) != 0) return fail("the clipped-side red not dead");
    if (count_fam(img, 0) < 3000) return fail("the kept-side green was mistaken-killed");
    if (visiaengine_pick(ve, 80.0f, 60.0f) != back) return fail("the retry loop missed the back wall");

    /* three-plane AND (x >= 0 and y >= 0 and z <= 1): green ~ quarter; the z constraint lifts the front-wall occlusion (the true cause
       of the measured two-plane g2 = 0 = the red quarter covering green on the same screen [PIT-8 probe ahead]) */
    VeClipPlane corner[3] = {{0.0, 1.0, 0.0, 0.0}, {1.0, 0.0, 0.0, 0.0}, {0.0, 0.0, -1.0, 1.0}};
    if (visiaengine_set_clips(ve, corner, 3) != VE_OK) return fail("set corner");
    if (visiaengine_render(ve) != VE_OK) return fail("render2");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback2");
    long g2 = count_fam(img, 0);
    printf("PROBE g2=%ld\n", g2); /* kept after settling (the terminal is the human-verified face) */
    if (g2 < 800 || g2 > 4000) return fail("the corner-domain green family is out of interval");

    /* value-domain rejection forms: n > 4 / NULL with n > 0 / zero normal (the degenerate gate) */
    VeClipPlane five[5] = {{0, 1, 0, 0}, {0, 1, 0, 0}, {0, 1, 0, 0}, {0, 1, 0, 0}, {0, 1, 0, 0}};
    if (visiaengine_set_clips(ve, five, 5) != VE_ERR_ARG) return fail("n > 4 not rejected");
    if (visiaengine_set_clips(ve, NULL, 1) != VE_ERR_ARG) return fail("NULL with n > 0 not rejected");
    VeClipPlane zero = {0.0, 0.0, 0.0, 1.0};
    if (visiaengine_set_clips(ve, &zero, 1) != VE_ERR_ARG) return fail("zero normal not rejected");

    /* the only clearing form n = 0: the red wall hits again + get = 0 */
    if (visiaengine_set_clips(ve, NULL, 0) != VE_OK) return fail("clear");
    if (visiaengine_get_clips(ve, NULL, 0) != 0) return fail("not empty after clear");
    if (visiaengine_pick(ve, 80.0f, 60.0f) != front) return fail("not restored after clearing");
    if (visiaengine_destroy(ve) != VE_OK) return fail("destroy");
    free(img);
    printf("OK e813 clip base_red=%ld corner_green=%ld domain=rejected\n", r0, g2);
    return 0;
}
