// E812_mesh_add -- SDK consumption, C++ procedural add/delete example (the CAPI-15/16 living gate).
// If it does not run, the API is unfinished [E3D:D7].
//
// Contract face: the enumeration domain stretches with adds/deletes / shapes mutually distinct (including the legal slot0gen0 = 0 scheme) / degenerate zero commit / old-handle re-entry rejected.
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
    if (h1 == h2) { puts("E812 FAIL shape collision"); return 1; }
    if (eng.entity_count() != 2) { puts("E812 FAIL the count did not grow with the add"); return 1; }
    if (eng.render() != VE_OK) { puts("E812 FAIL render"); return 1; }

    // degenerate road: index out of range = rejected with the count unmoved (the zero-commit contract)
    {
        const uint32_t oob[3] = {0, 9, 2};
        auto d = quad_desc(oob, 3);
        d.base_color = col;
        d.origin = org;
        uint64_t sink = 42;
        if (eng.add_mesh(d, &sink) >= 0 || sink != 42) {
            puts("E812 FAIL degenerate polluted out/count"); return 1;
        }
    }
    if (eng.entity_count() != 2) { puts("E812 FAIL degenerate committed"); return 1; }

    if (eng.remove_entity(h1) != VE_OK) { puts("E812 FAIL remove#1"); return 1; }
    if (eng.remove_entity(h1) != VE_ERR_ARG) { puts("E812 FAIL re-entry not rejected"); return 1; }
    if (eng.remove_entity(h2) != VE_OK) { puts("E812 FAIL remove#2"); return 1; }
    if (eng.entity_count() != 0) { puts("E812 FAIL the count did not shrink with the delete"); return 1; }

    puts("OK E812_mesh_add addremove=roundtrip");
    return 0;
}
