// visiaengine.hpp —— C++ header-only RAII 门面（17 口薄壳，S4；B9 裁决：不开 SDD 账）。
// 三条壳语义（本注释即合同，语义本体锚在 CAPI-nn）：
//   ① no-throw：方法逐一转发 C ABI 返回码（VE_OK/VE_ERR_*），异常零存在——
//      -fno-exceptions 宿主可用；诊断经 last_error()（线程绑定语义同 C 侧，禁作分支判据）。
//   ② move-only：Engine 独占 VeEngine 句柄；析构自动 destroy（released 再入按 CAPI-03 返回 -1，
//      静默——RAII 面不吞错也不双毁：moved-from 置空不再析构）。
//   ③ attach 时序：create 之后、首帧 render 之前（CAPI-06 合同不因壳变形）。
#ifndef VISIAENGINE_HPP
#define VISIAENGINE_HPP

#include <cstddef>
#include <cstdint>
#include <utility>

#include "visiaengine.h"

namespace visiaengine {

// 非拥有位形转发（VeInput/VeMeshDesc/错误码直接穿壳）
using Input = VeInput;
using MeshDesc = VeMeshDesc;  // CAPI-15 值结构（C 头 typedef 别名入命名空间）
inline constexpr uint64_t kMiss = VE_MISS;

class Engine {
public:
    Engine(const Engine &) = delete;
    Engine &operator=(const Engine &) = delete;

    Engine(Engine &&o) noexcept : h_(o.h_) { o.h_ = 0; }
    Engine &operator=(Engine &&o) noexcept {
        if (this != &o) { destroy_now(); h_ = o.h_; o.h_ = 0; }
        return *this;
    }
    ~Engine() { destroy_now(); }

    /// headless 引擎（离屏尺寸=初 viewport）；失败返回 !valid()，诊断读 last_error()
    [[nodiscard]] static Engine create_headless(std::uint32_t w, std::uint32_t h) {
        return Engine{visiaengine_create_headless(w, h)};
    }

    [[nodiscard]] bool valid() const noexcept { return h_ != 0; }
    [[nodiscard]] std::uint64_t handle() const noexcept { return h_; }

    int32_t attach(std::uint64_t window, std::uint64_t display_or_hinstance,
                   std::int32_t kind) { return visiaengine_attach(h_, window, display_or_hinstance, kind); }
    int32_t load_gltf(const char *path) { return visiaengine_load_gltf(h_, path); }
    int32_t load_geojson(const char *path) { return visiaengine_load_geojson(h_, path); }
    int32_t on_input(const Input &in) { return visiaengine_on_input(h_, &in); }

    // ── B1 数据带（CAPI-13..16 薄转发；位形 0 合法=CAPI-01 分工）──
    int32_t set_visible(std::uint64_t entity, bool visible) {
        return visiaengine_entity_set_visible(h_, entity, visible ? 1 : 0);
    }
    int32_t entity_visible(std::uint64_t entity) { return visiaengine_entity_visible(h_, entity); }
    int32_t add_mesh(const MeshDesc &desc, std::uint64_t *out_entity) {
        return visiaengine_add_mesh(h_, &desc, out_entity);
    }
    int32_t remove_entity(std::uint64_t entity) { return visiaengine_remove_entity(h_, entity); }
    // CAPI-18 薄转发（out 位形合法可 0 禁当哨兵——CAPI-01；退化 desc 在 C 门已拒）
    int32_t add_points(const VePointMark *marks, std::uint64_t count, std::uint64_t *out_entity) {
        const VePointsDesc d{sizeof(VePointsDesc), marks, count};
        return visiaengine_add_points(h_, &d, out_entity);
    }
    // CAPI-17 薄转发（回调生命周期由宿主担保：引擎不拥有 cb；三壳语义住头注释）
    int32_t set_event_callback(VeEventCb cb, void *user) {
        return visiaengine_set_event_callback(h_, cb, user);
    }
    int32_t render() { return visiaengine_render(h_); }
    int32_t readback(std::uint8_t *buf, std::uint64_t len) { return visiaengine_readback(h_, buf, len); }
    int32_t viewport(std::uint32_t w, std::uint32_t h) { return visiaengine_viewport(h_, w, h); }
    const char *last_error() { return visiaengine_last_error(h_); }
    int32_t entity_count() { return visiaengine_entity_count(h_); }
    std::uint64_t pick(float px, float py) { return visiaengine_pick(h_, px, py); }
    std::uint64_t entity_at(std::uint32_t i) { return visiaengine_entity_at(h_, i); }
    int32_t attr_f64(std::uint64_t ent, const char *key, double *out) {
        return visiaengine_attr_f64(h_, ent, key, out);
    }
    int32_t attr_str(std::uint64_t ent, const char *key, char *buf, std::uint64_t cap) {
        return visiaengine_attr_str(h_, ent, key, buf, cap);
    }
    int32_t attr_bool(std::uint64_t ent, const char *key, std::int32_t *out) {
        return visiaengine_attr_bool(h_, ent, key, out);
    }

private:
    explicit Engine(std::uint64_t h) : h_(h) {}
    void destroy_now() { if (h_ != 0) { visiaengine_destroy(h_); h_ = 0; } }
    std::uint64_t h_;
};

[[nodiscard]] inline std::uint32_t abi_version() { return visiaengine_abi_version(); }

}  // namespace visiaengine

#endif /* VISIAENGINE_HPP */
