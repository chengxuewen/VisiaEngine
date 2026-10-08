/* E817_two_layer_map · SDK 消费·C 双层地图（矢量瓦片叠在栅格底图之上）。
 * band-T 条款修订句（CAPI-28/29/39/40）的活体门；无头证据身份（E813/E816 同制）：
 * 五段活体门，秒退。
 * 段1 空帧 canary；段2 栅格单独=底图族在位而矢量族恒 0；段3 矢量单独=反之；
 * 段4 两型同挂（先矢后栅 + 先栅后矢两形）=两族同框（本带主张的视觉见证）；
 * 段5 同型重挂=画面逐字节不变（替换非堆叠）。
 * 矢量族谓词逐字复用 examples/rs/E206_tile_viewer.rs prove() 的已实测形。底图族用
 * 排除桶（非角标非矢量三色）而非色盒：E206 的 water/seam 谓词与底图配色相撞
 * （raster_only 帧实测 water=69120、seam=34560），而 vector_only 帧 other=0 ——
 * 排除桶因此是零猜测的底图见证（PIT-8 探针先行）。 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "visiaengine.h"

#define VW 480
#define VH 360

/* Fixture geography (probe-measured, band T): the committed vector tiles sit at
 * z10 x=0..2,y=0..2 while the original basemap fixture sits at x=850..851,
 * y=388..389 -- opposite sides of the planet, which is why stacking THOSE two
 * shows one layer per frame (measured: coverage collapsed to the first mount).
 * `resources/data/raster_over` is the same four 79-byte PNGs re-keyed into the
 * vector region (x=0..1,y=0..1) so both kinds genuinely share one frame. */
#define RASTER_DIR "resources/data/raster_over"
#define VECTOR_DIR "resources/data/tiles"
#define TILE_Z 10

static int fail(const char *what) {
    printf("E817 FAIL %s\n", what);
    return 1;
}

/* Vector families, verbatim from examples/rs/E206_tile_viewer.rs prove(). */
static int is_water(const uint8_t *p) { return p[2] > 90 && p[2] > p[0] + 20; }
static int is_road(const uint8_t *p) { return p[0] > 150 && p[1] > 80 && p[2] < 90; }
static int is_poi(const uint8_t *p) { return p[0] > 150 && p[1] < 90 && p[2] < 90; }

typedef struct { long corner, water, road, poi, basemap; } census;

static uint8_t *grab(uint64_t ve) {
    if (visiaengine_render(ve) != 0) { printf("E817 FAIL render\n"); exit(1); }
    size_t len = (size_t)VW * VH * 4;
    uint8_t *img = malloc(len);
    if (!img || visiaengine_readback(ve, img, len) != 0) { printf("E817 FAIL readback\n"); exit(1); }
    return img;
}

static void census_of(const uint8_t *img, census *c) {
    memset(c, 0, sizeof(*c));
    const int cor0 = img[0], cor1 = img[1], cor2 = img[2]; /* self-calibrated corner */
    for (long i = 0; i < (long)VW * VH; ++i) {
        const uint8_t *p = img + 4 * i;
        if (p[0] == cor0 && p[1] == cor1 && p[2] == cor2) c->corner++;
        else if (is_road(p)) c->road++;
        else if (is_poi(p)) c->poi++;
        else if (is_water(p)) c->water++;
        else c->basemap++; /* exclusion bucket = basemap witness */
    }
}

static void dump(const char *tag, const census *c) {
    printf("E817 %-18s corner=%ld water=%ld road=%ld poi=%ld basemap=%ld\n",
           tag, c->corner, c->water, c->road, c->poi, c->basemap);
}

static int mount_raster(uint64_t ve) {
    return visiaengine_load_raster_dir(ve, RASTER_DIR, TILE_Z) == 4 ? 0 : -1;
}
static int mount_vector(uint64_t ve) {
    return visiaengine_load_mvt_dir(ve, VECTOR_DIR, TILE_Z) == 9 ? 0 : -1;
}

int main(void) {
    uint64_t ve = visiaengine_create_headless(VW, VH);
    if (!ve) return fail("create");
    census c;
    uint8_t *img;

    /* [1] empty-frame canary: no family may claim a pixel */
    img = grab(ve);
    census_of(img, &c);
    dump("empty", &c);
    free(img);
    if (c.corner != (long)VW * VH || c.road || c.poi || c.basemap) return fail("empty canary");

    /* [2] basemap alone: its family covers the frame, vector families exactly 0 */
    if (mount_raster(ve)) return fail("mount raster");
    img = grab(ve);
    census_of(img, &c);
    dump("raster_only", &c);
    free(img);
    if (c.basemap < 1000) return fail("basemap family missing");
    if (c.road || c.poi) return fail("basemap frame leaked a vector family");
    visiaengine_destroy(ve);

    /* [3] vectors alone: orange/red present, basemap witness exactly 0 */
    ve = visiaengine_create_headless(VW, VH);
    if (!ve) return fail("create 2");
    if (mount_vector(ve)) return fail("mount vector");
    img = grab(ve);
    census_of(img, &c);
    dump("vector_only", &c);
    free(img);
    if (c.road < 5000 || c.poi < 500) return fail("vector families missing");
    if (c.basemap) return fail("vector frame leaked the basemap witness");

    /* [4a] both kinds, vector first: camera keeps the 3x3 framing */
    if (mount_raster(ve)) return fail("mount raster over vector");
    img = grab(ve);
    census_of(img, &c);
    dump("stacked_v_then_r", &c);
    free(img);
    /* measured 2026-10-08 (lavapipe, 480x360, vector-framed): road=9782 poi=1314
       basemap=26030 -- thresholds = baseline minus 40% (pixel-gate rule) */
    if (c.road < 5800 || c.poi < 780) return fail("vectors vanished under the basemap");
    if (c.basemap < 15600) return fail("basemap vanished under the vectors");
    visiaengine_destroy(ve);

    /* [4b] reverse order -- the one that used to erase the first layer */
    ve = visiaengine_create_headless(VW, VH);
    if (!ve) return fail("create 3");
    if (mount_raster(ve)) return fail("mount raster first");
    if (mount_vector(ve)) return fail("mount vector over raster");
    img = grab(ve);
    census_of(img, &c);
    dump("stacked_r_then_v", &c);
    free(img);
    /* measured same run (basemap-framed, so the camera magnifies and fewer vector
       pixels land on screen -- presence, not size, is the claim): road=5343 poi=684
       basemap=60322 -- thresholds at minus 40% */
    if (c.road < 3200 || c.poi < 400) return fail("vectors erased by the raster-first mount");
    if (c.basemap < 36000) return fail("basemap erased by the vector-first mount");

    /* [5] same-kind re-mount replaces (never stacks): byte-identical frame */
    img = grab(ve);
    if (mount_raster(ve) || mount_vector(ve)) return fail("re-mount");
    uint8_t *again = grab(ve);
    int same = memcmp(img, again, (size_t)VW * VH * 4) == 0;
    free(again);
    free(img);
    visiaengine_destroy(ve);
    if (!same) return fail("re-mount changed the frame (stacking, not replacing)");

    printf("OK two-layer map: basemap and vector tiles share one frame in both mount orders\n");
    return 0;
}
