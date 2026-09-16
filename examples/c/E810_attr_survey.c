/* E810 · SDK 消费·C 属性遍历 survey —— 数据带验收例（零新口：CAPI-04/10 族的宿主全景）。
 * 跑不通即 API 未完成 [E3D:D7]。
 *
 * park.geojson 逐实体：name(str) + fill-opacity(f64，D9 原始键) + 故意读缺失键（missing≠0 契约现场）。
 * ctest: example_E810_attr_survey（headless，仓根 WORKING_DIRECTORY 由注册宏担保）。 */
#include <stdio.h>
#include <string.h>

#include "visiaengine.h"

int main(void) {
    uint64_t ve = visiaengine_create_headless(160, 120);
    if (!ve) { puts("E810 FAIL create"); return 1; }
    if (visiaengine_load_geojson(ve, "resources/data/park.geojson") != VE_OK) {
        printf("E810 FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    int n = visiaengine_entity_count(ve);
    int names = 0, opac = 0;
    for (int i = 0; i < n; ++i) {
        uint64_t ent = visiaengine_entity_at(ve, (uint32_t)i);
        char name[256];
        if (visiaengine_attr_str(ve, ent, "name", name, sizeof name) == 1) {
            ++names;
            double opacity = -1.0;
            int has_op = visiaengine_attr_f64(ve, ent, "fill-opacity", &opacity) == 1;
            if (has_op) ++opac;
            /* 缺失≠零值契约：读不存在键，out 必须保持原值（此处哨兵 -1.0） */
            double sentinel = -1.0;
            if (visiaengine_attr_f64(ve, ent, "definitely_missing_key", &sentinel) != 0 ||
                sentinel != -1.0) {
                puts("E810 FAIL 缺失路污染 out");
                return 1;
            }
            printf("ATTR i=%d name=%s fill-opacity=%s\n", i, name,
                   has_op ? "" : "(none)");
        }
    }
    if (names < 2 || opac < 1) {
        printf("E810 FAIL 属性面收缩 names=%d opac=%d / n=%d\n", names, opac, n);
        return 1;
    }
    if (visiaengine_destroy(ve) != VE_OK) { puts("E810 FAIL destroy"); return 1; }
    printf("OK E810_attr_survey names=%d opacity=%d entities=%d\n", names, opac, n);
    return 0;
}
