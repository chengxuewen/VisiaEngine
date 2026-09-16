// E812_mesh_add · SDK 消费·C++ 程序化增删例（CAPI-15/16 活体门）。
// 跑不通即 API 未完成 [E3D:D7]。
//
// 契约面：枚举域随增删伸缩 / 位形互异（含 slot0gen0=0 合法谱）/ 退化零提交 / 旧句柄再入拒。
#include <cstdint>
#include <cstdio>
#include <vector>

#include <visiaengine/visiaengine.hpp>

namespace {
constexpr float POS[4][3] = {{0, 0, 0}, {1, 0, 0}, {1, 1, 0}, {0, 1, 0}};
constexpr float NRM[4][3] = {{0, 0, 1}, {0, 0, 1}, {0, 0, 1}, {0, 0, 1}};
constexpr uint32_t IDX[6] = {0, 1, 2, 0, 2, 3};

visiaengine::MeshDesc quad_desc(const uint32_t *idx, uint64_t n_idx) {
    return visiaengine::MeshDesc{
        sizeof(visiaengine::MeshDesc),
        &POS[0][0], &NRM[0][0], idx, 4, n_idx,
        nullptr, nullptr,
    };
}
}  // namespace

int main() {
    auto eng = visiaengine::Engine::create_headless(160, 120);
    if (!eng.valid()) { puts("E812 FAIL create"); return 1; }

    float col[4] = {1.0f, 0.0f, 0.0f, 1.0f};
    double org[3] = {0.0, 0.0, 0.0};
    uint64_t h1 = 0, h2 = 0;
    {
        auto d = quad_desc(IDX, 6);
        d.base_color = col;
        d.origin = org;
        if (eng.add_mesh(d, &h1) != VE_OK) { puts("E812 FAIL add#1"); return 1; }
        if (eng.add_mesh(d, &h2) != VE_OK) { puts("E812 FAIL add#2"); return 1; }
    }
    if (h1 == h2) { puts("E812 FAIL 位形撞车"); return 1; }
    if (eng.entity_count() != 2) { puts("E812 FAIL 计数不随增"); return 1; }
    if (eng.render() != VE_OK) { puts("E812 FAIL render"); return 1; }

    // 退化路：索引越界 = 拒绝且计数不动（零提交契约）
    {
        const uint32_t oob[3] = {0, 9, 2};
        auto d = quad_desc(oob, 3);
        d.base_color = col;
        d.origin = org;
        uint64_t sink = 42;
        if (eng.add_mesh(d, &sink) >= 0 || sink != 42) {
            puts("E812 FAIL 退化污染 out/计数"); return 1;
        }
    }
    if (eng.entity_count() != 2) { puts("E812 FAIL 退化提交"); return 1; }

    if (eng.remove_entity(h1) != VE_OK) { puts("E812 FAIL remove#1"); return 1; }
    if (eng.remove_entity(h1) != VE_ERR_ARG) { puts("E812 FAIL 再入未拒"); return 1; }
    if (eng.remove_entity(h2) != VE_OK) { puts("E812 FAIL remove#2"); return 1; }
    if (eng.entity_count() != 0) { puts("E812 FAIL 计数不随删"); return 1; }

    puts("OK E812_mesh_add addremove=roundtrip");
    return 0;
}
