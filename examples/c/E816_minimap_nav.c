/* E816_minimap_nav · SDK 消费·C 小地图导航（CAPI-25..27 宿主面验收）。
 * 无头证据身份（E813 同制）：五段活体门，秒退。
 * 段1 canary=map 开→关 readback 逐字节回旧路；段2 角区双族（顶视=红面盖中
 * ±4 ∧ 绿地板露边 [PIT-8 探针实测 1596/756@2026-09-18]）；段3 pick 小图区
 * 命中地板（顶层优先路由，主区 pick 不变）；段4 navigate_click 起飞→
 * fly_state 轮询到站→get_camera 落点数值（保角保距仅换 target=裁决 e）；
 * 段5 区外/无图拒 + 清空复原（终态 canary + 中心 pick 恒中红面）。 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "visiaengine.h"

#define VW 160
#define VH 120

/* 场景=两水平面（E813 同制）：地板 ±8 z=0（绿）+ 红面 ±4 z=2（主视/顶视双盖中心）
 * positions 含 z → origin 恒零（两帧合一注记同 E813） */
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

/* 角区 [100,156)×[72,114)（fx.625/fy.6/fw.35/fh.35 @160×120）双族计数 */
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

    /* 段1 canary：旧路基线入 ref → 开图再关（NULL）→ 逐字节相等 */
    if (visiaengine_render(ve) != VE_OK) return fail("render0");
    if (visiaengine_readback(ve, ref, (size_t)VW * VH * 4) != VE_OK) return fail("readback0");
    VeMapView mv = {0};
    mv.struct_size = sizeof(VeMapView);
    mv.fx = 0.625f; mv.fy = 0.6f; mv.fw = 0.35f; mv.fh = 0.35f;
    mv.zoom = 8.0; /* 顶视半宽：地板 ±8 恰好框住，红面 ±4 居中 */
    if (visiaengine_set_map(ve, &mv) != VE_OK) return fail("set_map");
    if (visiaengine_set_map(ve, NULL) != VE_OK) return fail("clear_map");
    if (visiaengine_render(ve) != VE_OK) return fail("render1");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback1");
    if (memcmp(img, ref, (size_t)VW * VH * 4) != 0) return fail("canary 逐字节破裂");

    /* 段2 开图：角区=顶视双族（红面居中+绿地板露边）；主区红族不塌 */
    if (visiaengine_set_map(ve, &mv) != VE_OK) return fail("set_map2");
    if (visiaengine_render(ve) != VE_OK) return fail("render2");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback2");
    long gcorner = 0, rcorner = 0;
    count_corner(img, &gcorner, &rcorner);
    long rmain = count_red_all(img);
    printf("PROBE gcorner=%ld rcorner=%ld rmain=%ld\n", gcorner, rcorner, rmain); /* 保留 */
    if (gcorner < 900) return fail("角区顶视绿地板缺席");              /* 实测 1596 −40% */
    if (rcorner < 300 || rcorner > 1200) return fail("角区红面族区间外"); /* 实测 756 */
    if (rmain < 3000) return fail("主区红族被双投误伤");

    /* 段3 pick 路由：小图区（绿露边位）=顶层优先命中地板；主区中心恒中红面 */
    if (visiaengine_pick(ve, 150.0f, 80.0f) != floor_id) return fail("小图 pick 未中地板");
    if (visiaengine_pick(ve, 80.0f, 60.0f) != top_id) return fail("主区 pick 变形");

    /* 段4 导航：起飞(600ms)→render 即 tick→fly_state 到站→位姿读回 */
    VeCameraPose before = {0};
    before.struct_size = sizeof(VeCameraPose);
    if (visiaengine_get_camera(ve, &before) != VE_OK) return fail("get_camera before");
    if (visiaengine_navigate_click(ve, 140.0f, 80.0f, 600) != VE_OK) return fail("navigate");
    if (visiaengine_fly_state(ve, NULL) != 0) return fail("起飞后应飞中");
    int ticks = 0;
    while (visiaengine_fly_state(ve, NULL) == 0 && ticks < 400) {
        if (visiaengine_render(ve) != VE_OK) return fail("render fly");
        ticks++;
    }
    if (ticks >= 400) return fail("飞行未到站");
    VeCameraPose after = {0};
    after.struct_size = sizeof(VeCameraPose);
    if (visiaengine_get_camera(ve, &after) != VE_OK) return fail("get_camera after");
    printf("PROBE target=(%.3f,%.3f,%.3f) ticks=%d\n", after.target[0], after.target[1],
           after.target[2], ticks); /* 瞬移形同式实测 (3.579,3.714,0) */
    /* 落点=地面 z0（地板内带）；保角保距仅换 target（裁决 e，f64 逐位） */
    if (after.target[2] != 0.0) return fail("落点 z 非地面");
    if (!(after.target[0] > 2.5 && after.target[0] < 4.6)) return fail("落点 x 域外");
    if (!(after.target[1] > 2.6 && after.target[1] < 4.8)) return fail("落点 y 域外");
    if (after.dist != before.dist || after.fov != before.fov) return fail("保距破裂");
    if (after.yaw != before.yaw || after.pitch != before.pitch) return fail("保角破裂");

    /* 段5 值域拒 + 清空复原（无图导航拒=路由零残留正证；canary 重取基线形——
       飞行后相机已挪，旧 ref 不可比 [写后自查根修]） */
    if (visiaengine_set_map(ve, NULL) != VE_OK) return fail("clear2");
    if (visiaengine_render(ve) != VE_OK) return fail("render-refly");
    if (visiaengine_readback(ve, ref, (size_t)VW * VH * 4) != VE_OK) return fail("readback-refly");
    if (visiaengine_navigate_click(ve, 10.0f, 60.0f, 0) != VE_ERR_ARG) return fail("区外未拒");
    VeMapView bad = mv; bad.zoom = -1.0;
    if (visiaengine_set_map(ve, &bad) != VE_ERR_ARG) return fail("zoom≤0 未拒");
    if (visiaengine_navigate_click(ve, 140.0f, 80.0f, 0) != VE_ERR_ARG) return fail("无图导航未拒");
    if (visiaengine_set_map(ve, &mv) != VE_OK) return fail("reopen");
    if (visiaengine_set_map(ve, NULL) != VE_OK) return fail("reclear");
    if (visiaengine_render(ve) != VE_OK) return fail("render5");
    if (visiaengine_readback(ve, img, (size_t)VW * VH * 4) != VE_OK) return fail("readback5");
    if (memcmp(img, ref, (size_t)VW * VH * 4) != 0) return fail("终态 canary 破裂");

    if (visiaengine_destroy(ve) != VE_OK) return fail("destroy");
    free(img); free(ref);
    printf("OK e816 canary=byte-equal corner=%ldg+%ldr nav=arrived ticks=%d\n",
           gcorner, rcorner, ticks);
    return 0;
}
