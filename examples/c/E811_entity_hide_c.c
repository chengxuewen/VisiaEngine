/* E811_entity_hide_c · SDK 消费·C 显隐孪生（CAPI-13/14 宿主面验收）。
 * 跑不通即 API 未完成 [E3D:D7]。孪生谱系：examples/cpp/E811_entity_hide_cpp.cpp。
 *
 * 契约三件：隐藏件 pick 不命中自体 / 枚举域不变 / 可见性查询往返（含幂等与越值拒绝）。
 * 位形 0 合法（CAPI-01 有意分工）——存在性判据只用查询/枚举回执，禁按值判。 */
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
    if (h == VE_MISS) { puts("E811 FAIL 中心无实体（几何基线漂移）"); return 1; }

    if (visiaengine_entity_visible(ve, h) != 1) { puts("E811 FAIL 默认可见"); return 1; }
    if (visiaengine_entity_set_visible(ve, h, 0) != VE_OK) { puts("E811 FAIL hide"); return 1; }
    if (visiaengine_entity_visible(ve, h) != 0) { puts("E811 FAIL 查询未反映隐藏"); return 1; }
    if (visiaengine_entity_set_visible(ve, h, 0) != VE_OK) { puts("E811 FAIL 幂等重设"); return 1; }
    if (visiaengine_pick(ve, 80.0f, 60.0f) == h) { puts("E811 FAIL 隐藏件仍命中自体"); return 1; }
    if (visiaengine_entity_count(ve) != 2) { puts("E811 FAIL 枚举域被显隐污染"); return 1; }
    if (visiaengine_entity_set_visible(ve, h, 2) != VE_ERR_ARG) { puts("E811 FAIL 越值未拒"); return 1; }
    if (visiaengine_entity_set_visible(ve, 0xdeadbeefULL, 0) != VE_ERR_ARG) {
        puts("E811 FAIL 未知位形未拒"); return 1;
    }
    if (visiaengine_entity_set_visible(ve, h, 1) != VE_OK) { puts("E811 FAIL show"); return 1; }
    if (visiaengine_pick(ve, 80.0f, 60.0f) != h) { puts("E811 FAIL 复原后未命中"); return 1; }

    if (visiaengine_destroy(ve) != VE_OK) { puts("E811 FAIL destroy"); return 1; }
    puts("OK E811_entity_hide_c visibility=roundtrip");
    return 0;
}
