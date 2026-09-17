/* E813_section_clip · SDK 消费·C 剖面裁切（CAPI-20 宿主面验收）。
 * 无头证据身份（E811 同制）：readback 像素族计数三段=基线/半裁/复原，秒退。
 * 注：capi pick=positions×IDENTITY 帧（origin 住 render 路=既有事实）——
 * z 烘进 positions 令两帧合一，勿学成 bug。 */
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
    double o0[3] = {0.0, 0.0, 0.0}; /* positions 已含 z（两帧合一）→ origin 恒零 */
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

    /* 基线：前墙红铺中心；pick 中前 */
    if (visiaengine_render(ve) != VE_OK) return fail("render0");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback0");
    long r0 = count_fam(img, 1);
    if (r0 < 3000) return fail("红墙基线缺席");
    if (visiaengine_pick(ve, 80.0f, 60.0f) != front) return fail("默认未中前墙");

    /* 半刀 z≤1：红全灭（z=2 整面弃片）、绿自若、pick 重试环中后墙 */
    VeClipPlane p = {0.0, 0.0, -1.0, 1.0};
    if (visiaengine_set_clips(ve, &p, 1) != VE_OK) return fail("set_clips");
    if (visiaengine_get_clips(ve, NULL, 0) != 1) return fail("读回计数");
    if (visiaengine_render(ve) != VE_OK) return fail("render1");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback1");
    if (count_fam(img, 1) != 0) return fail("裁侧红未灭");
    if (count_fam(img, 0) < 3000) return fail("保留侧绿被误杀");
    if (visiaengine_pick(ve, 80.0f, 60.0f) != back) return fail("重试环未中后墙");

    /* 三面 AND（x≥0 ∧ y≥0 ∧ z≤1）：绿≈四分之一；z 约束拆前墙遮蔽（二面形实测 g2=0
       的真因=红四分之一同屏盖绿 [PIT-8 探针先行]） */
    VeClipPlane corner[3] = {{0.0, 1.0, 0.0, 0.0}, {1.0, 0.0, 0.0, 0.0}, {0.0, 0.0, -1.0, 1.0}};
    if (visiaengine_set_clips(ve, corner, 3) != VE_OK) return fail("set corner");
    if (visiaengine_render(ve) != VE_OK) return fail("render2");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback2");
    long g2 = count_fam(img, 0);
    printf("PROBE g2=%ld\n", g2); /* 定着后保留（终端即人验面） */
    if (g2 < 800 || g2 > 4000) return fail("角域绿族区间外");

    /* 值域拒形：n>4 / NULL∧n>0 / 零法向（退化门） */
    VeClipPlane five[5] = {{0, 1, 0, 0}, {0, 1, 0, 0}, {0, 1, 0, 0}, {0, 1, 0, 0}, {0, 1, 0, 0}};
    if (visiaengine_set_clips(ve, five, 5) != VE_ERR_ARG) return fail("n>4 未拒");
    if (visiaengine_set_clips(ve, NULL, 1) != VE_ERR_ARG) return fail("NULL∧n>0 未拒");
    VeClipPlane zero = {0.0, 0.0, 0.0, 1.0};
    if (visiaengine_set_clips(ve, &zero, 1) != VE_ERR_ARG) return fail("零法向未拒");

    /* 唯一清空形 n=0：红墙复中 + get=0 */
    if (visiaengine_set_clips(ve, NULL, 0) != VE_OK) return fail("clear");
    if (visiaengine_get_clips(ve, NULL, 0) != 0) return fail("clear 后未空");
    if (visiaengine_pick(ve, 80.0f, 60.0f) != front) return fail("清空后未复原");
    if (visiaengine_destroy(ve) != VE_OK) return fail("destroy");
    free(img);
    printf("OK e813 clip base_red=%ld corner_green=%ld domain=rejected\n", r0, g2);
    return 0;
}
