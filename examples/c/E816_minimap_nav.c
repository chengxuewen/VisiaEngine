/* E816_minimap_nav -- SDK consumption, C minimap navigation (the CAPI-25..27 host-face acceptance).
 *  Headless evidence identity (same system as E813): five living-gate segments, exits in seconds.
 *  Segment 1 canary = map on -> off, readback byte-identical back to the old road; segment 2 corner-region dual families (top view = red face covers the center
 *  +/- 4 and green ground shows edges [PIT-8 probe measured 1596/756@2026-09-18]); segment 3 pick in the minimap region
 *  hits the ground (top-layer-first routing, main-region pick unchanged); segment 4 navigate_click takes off ->
 *  fly_state polled to arrival -> get_camera landing numbers (angle and distance kept, only the target swapped = ruling e);
 *  segment 5 outside-region/no-map rejection + clearing restore (terminal canary + the center pick always hits the red face). */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "visiaengine.h"

#define VW 160
#define VH 120

/* Scene = two horizontal planes (same system as E813): ground +/- 8 at z = 0 (green) + red face +/- 4 at z = 2 (covers center in both main/top views)
 *  positions carry z -> origin stays zero (the two-frames-merged note same as E813) */
static const float POS_FLOOR[12] = {-8.0f, -8.0f, 0.0f, 8.0f, -8.0f, 0.0f,
                                    8.0f,  8.0f,  0.0f, -8.0f, 8.0f,  0.0f};
static const float POS_TOP[12]   = {-4.0f, -4.0f, 2.0f, 4.0f, -4.0f, 2.0f,
                                    4.0f,  4.0f,  2.0f, -4.0f, 4.0f,  2.0f};
static const float NRM[12] = {0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.0f,
                              0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 1.0f};
static const uint32_t IDX[6] = {0, 1, 2, 0, 2, 3};

static int fail(const char *what) {
    printf("E816 FAIL %s\n", what);
    return 1;
}

static int add_rect(uint64_t ve, const float *pos, const float *color, uint64_t *out) {
    VeMeshDesc d = {0};
    double o0[3] = {0.0, 0.0, 0.0};
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

/* corner region [100,156) x [72,114) (fx.625/fy.6/fw.35/fh.35 @160x120) dual-family counts */
static void count_corner(const uint8_t *img, long *green, long *red) {
    *green = 0; *red = 0;
    for (int y = 72; y < 114; ++y)
        for (int x = 100; x < 156; ++x) {
            const uint8_t *p = img + 4 * ((size_t)y * VW + x);
            if (p[1] > 120 && p[0] < 100 && p[2] < 100) (*green)++;
            else if (p[0] > 120 && p[1] < 80 && p[2] < 80) (*red)++;
        }
}
static long count_red_all(const uint8_t *img) {
    long n = 0;
    for (long i = 0; i < (long)VW * VH; ++i)
        if (img[i * 4] > 120 && img[i * 4 + 1] < 80 && img[i * 4 + 2] < 80) n++;
    return n;
}

int main(void) {
    uint64_t ve = visiaengine_create_headless(VW, VH);
    if (!ve) return fail("create");
    static const float RED[4]   = {1.0f, 0.0f, 0.0f, 1.0f};
    static const float GREEN[4] = {0.0f, 1.0f, 0.0f, 1.0f};
    uint64_t floor_id = 0, top_id = 0;
    if (add_rect(ve, POS_FLOOR, GREEN, &floor_id) != VE_OK) return fail("add floor");
    if (add_rect(ve, POS_TOP, RED, &top_id) != VE_OK) return fail("add top");
    uint8_t *img = malloc((size_t)VW * VH * 4);
    uint8_t *ref = malloc((size_t)VW * VH * 4);
    if (!img || !ref) return fail("oom");

    /* segment 1 canary: baseline of the old road into ref -> open the map then close (NULL) -> byte-identical */
    if (visiaengine_render(ve) != VE_OK) return fail("render0");
    if (visiaengine_readback(ve, ref, (size_t)VW * VH * 4) != VE_OK) return fail("readback0");
    VeMapView mv = {0};
    mv.struct_size = sizeof(VeMapView);
    mv.fx = 0.625f; mv.fy = 0.6f; mv.fw = 0.35f; mv.fh = 0.35f;
    mv.zoom = 8.0; /* top-view half-width: the ground +/- 8 framed exactly, the red face +/- 4 centered */
    if (visiaengine_set_map(ve, &mv) != VE_OK) return fail("set_map");
    if (visiaengine_set_map(ve, NULL) != VE_OK) return fail("clear_map");
    if (visiaengine_render(ve) != VE_OK) return fail("render1");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback1");
    if (memcmp(img, ref, (size_t)VW * VH * 4) != 0) return fail("canary byte-identity broken");

    /* segment 2 map on: corner = top-view dual families (red face centered + green ground edges); the main-region red family does not collapse */
    if (visiaengine_set_map(ve, &mv) != VE_OK) return fail("set_map2");
    if (visiaengine_render(ve) != VE_OK) return fail("render2");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback2");
    long gcorner = 0, rcorner = 0;
    count_corner(img, &gcorner, &rcorner);
    long rmain = count_red_all(img);
    printf("PROBE gcorner=%ld rcorner=%ld rmain=%ld\n", gcorner, rcorner, rmain); /* kept */
    if (gcorner < 900) return fail("top-view green ground absent in the corner");              /* measured 1596 minus 40% */
    if (rcorner < 300 || rcorner > 1200) return fail("the corner red-face family is out of interval"); /* measured 756 */
    if (rmain < 3000) return fail("the main-region red family was hurt by the dual projection");

    /* segment 3 pick routing: minimap region (green-edge position) = top-layer-first hits the ground; the main-region center always hits the red face */
    if (visiaengine_pick(ve, 150.0f, 80.0f) != floor_id) return fail("minimap pick missed the ground");
    if (visiaengine_pick(ve, 80.0f, 60.0f) != top_id) return fail("the main-region pick deformed");

    /* segment 4 navigation: takeoff (600ms) -> render is the tick -> fly_state arrival -> pose readback */
    VeCameraPose before = {0};
    before.struct_size = sizeof(VeCameraPose);
    if (visiaengine_get_camera(ve, &before) != VE_OK) return fail("get_camera before");
    if (visiaengine_navigate_click(ve, 140.0f, 80.0f, 600) != VE_OK) return fail("navigate");
    if (visiaengine_fly_state(ve, NULL) != 0) return fail("should be flying after takeoff");
    int ticks = 0;
    while (visiaengine_fly_state(ve, NULL) == 0 && ticks < 400) {
        if (visiaengine_render(ve) != VE_OK) return fail("render fly");
        ticks++;
    }
    if (ticks >= 400) return fail("the flight never arrived");
    VeCameraPose after = {0};
    after.struct_size = sizeof(VeCameraPose);
    if (visiaengine_get_camera(ve, &after) != VE_OK) return fail("get_camera after");
    printf("PROBE target=(%.3f,%.3f,%.3f) ticks=%d\n", after.target[0], after.target[1],
           after.target[2], ticks); /* the teleport form, same expression, measured (3.579,3.714,0) */
    /* landing = ground z0 (inside the ground band); angle and distance kept with only the target swapped (ruling e, f64 bit by bit) */
    if (after.target[2] != 0.0) return fail("the landing z is not the ground");
    if (!(after.target[0] > 2.5 && after.target[0] < 4.6)) return fail("the landing x is out of domain");
    if (!(after.target[1] > 2.6 && after.target[1] < 4.8)) return fail("the landing y is out of domain");
    if (after.dist != before.dist || after.fov != before.fov) return fail("distance-keep broken");
    if (after.yaw != before.yaw || after.pitch != before.pitch) return fail("angle-keep broken");

    /* segment 5 value-domain rejection + clearing restore (no-map navigation rejected = the positive proof of zero routing residue; the canary re-takes the baseline form --
       the camera has moved after the flight, the old ref is not comparable [write-then-self-check root fix]) */
    if (visiaengine_set_map(ve, NULL) != VE_OK) return fail("clear2");
    if (visiaengine_render(ve) != VE_OK) return fail("render-refly");
    if (visiaengine_readback(ve, ref, (size_t)VW * VH * 4) != VE_OK) return fail("readback-refly");
    if (visiaengine_navigate_click(ve, 10.0f, 60.0f, 0) != VE_ERR_ARG) return fail("outside the region not rejected");
    VeMapView bad = mv; bad.zoom = -1.0;
    if (visiaengine_set_map(ve, &bad) != VE_ERR_ARG) return fail("zoom <= 0 not rejected");
    if (visiaengine_navigate_click(ve, 140.0f, 80.0f, 0) != VE_ERR_ARG) return fail("navigation without a map not rejected");
    if (visiaengine_set_map(ve, &mv) != VE_OK) return fail("reopen");
    if (visiaengine_set_map(ve, NULL) != VE_OK) return fail("reclear");
    if (visiaengine_render(ve) != VE_OK) return fail("render5");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback5");
    if (memcmp(img, ref, (size_t)VW * VH * 4) != 0) return fail("the terminal canary broke");

    if (visiaengine_destroy(ve) != VE_OK) return fail("destroy");
    free(img); free(ref);
    printf("OK e816 canary=byte-equal corner=%ldg+%ldr nav=arrived ticks=%d\n",
           gcorner, rcorner, ticks);
    return 0;
}
