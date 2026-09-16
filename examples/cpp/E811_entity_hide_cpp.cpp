// E811_entity_hide_cpp · SDK 消费·C++ 显隐孪生（hpp 门面的 CAPI-13/14 验收面）。
// 跑不通即 API 未完成 [E3D:D7]。孪生谱系：examples/c/E811_entity_hide_c.c（同题同码）。
#include <cstdio>
#include <cstdint>

#include <visiaengine/visiaengine.hpp>

int main() {
    auto eng = visiaengine::Engine::create_headless(160, 120);
    if (!eng.valid()) { puts("E811cpp FAIL create"); return 1; }
    if (eng.load_gltf("resources/data/twoprim.glb") != VE_OK) {
        printf("E811cpp FAIL load: %s\n", eng.last_error());
        return 1;
    }
    const uint64_t h = eng.pick(80.0f, 60.0f);
    if (h == visiaengine::kMiss) { puts("E811cpp FAIL 中心无实体"); return 1; }

    if (eng.entity_visible(h) != 1) { puts("E811cpp FAIL 默认可见"); return 1; }
    if (eng.set_visible(h, false) != VE_OK) { puts("E811cpp FAIL hide"); return 1; }
    if (eng.entity_visible(h) != 0) { puts("E811cpp FAIL 查询滞后"); return 1; }
    if (eng.pick(80.0f, 60.0f) == h) { puts("E811cpp FAIL 隐藏件命中自体"); return 1; }
    if (eng.entity_count() != 2) { puts("E811cpp FAIL 枚举域漂移"); return 1; }
    if (eng.set_visible(h, true) != VE_OK) { puts("E811cpp FAIL show"); return 1; }
    if (eng.pick(80.0f, 60.0f) != h) { puts("E811cpp FAIL 复原失败"); return 1; }

    puts("OK E811_entity_hide_cpp visibility=roundtrip");
    return 0;  // 析构自动 destroy（CAPI-03 RAII 投影；moved-from 双毁免疫在 hpp 层）
}
