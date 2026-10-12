// visiaengine.hpp -- C++ header-only RAII facade (a thin shell over the 17 entries, S4; B9 ruling: no SDD account opened).
// Three shell semantics (this comment IS the contract, the semantic bodies are anchored at CAPI-nn):
//   (1) no-throw: methods forward the C ABI return codes one by one (VE_OK/VE_ERR_*), exceptions have zero presence --
//       usable by -fno-exceptions hosts; diagnostics go through last_error() (thread-bound semantics same as the C side, must not be used as branch criteria).
//   (2) move-only: Engine owns the VeEngine handle exclusively; the destructor auto-destroys (re-entry on released returns -1 per CAPI-03,
//       silently -- the RAII face neither swallows errors nor double-destroys: moved-from is nulled and no longer destructs).
//   (3) attach ordering: after create, before the first render (the CAPI-06 contract does not deform through the shell).
#ifndef VISIAENGINE_HPP
#define VISIAENGINE_HPP

#include <cstddef>
#include <cstdint>
#include <utility>

#include "visiaengine.h"

namespace visiaengine {

// non-owning shape forwarding (VeInput/VeMeshDesc/error codes pass straight through the shell)
using Input = VeInput;
using MeshDesc = VeMeshDesc;  // CAPI-15 value structure (the C header typedef alias brought into the namespace)
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

    /// headless engine (offscreen size = initial viewport); on failure returns !valid(), read diagnostics from last_error()
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

    // -- B1 data band (CAPI-13..16 thin forwarding; shape 0 is legal = CAPI-01 division) --
    int32_t set_visible(std::uint64_t entity, bool visible) {
        return visiaengine_entity_set_visible(h_, entity, visible ? 1 : 0);
    }
    int32_t entity_visible(std::uint64_t entity) { return visiaengine_entity_visible(h_, entity); }
    // CAPI-23/24 camera flight (B9 thin forwarding; pose domain validation lives in the engine)
    int32_t fly_to(const VeCameraPose &pose, std::uint64_t dur_ms) {
        return visiaengine_fly_to(h_, &pose, dur_ms);
    }
    int32_t fly_state(double *out_t01) {
        return visiaengine_fly_state(h_, out_t01);
    }

    // -- 5b second wave minimap (CAPI-25..27 thin forwarding; VeMapView = header POD used directly, cfg = nullptr closes the map) --
    int32_t set_map(const VeMapView *cfg) { return visiaengine_set_map(h_, cfg); }
    /* CAPI-28/29: tile layer (MVT directory mount + viewport bbox fed to the scheduler). Thin forwarding. */
    int32_t load_mvt_dir(const char *path, uint32_t z) { return visiaengine_load_mvt_dir(h_, path, z); }
    /* CAPI-30..34: scene-tree thin forwarding. */
    uint64_t create_group(const char *name = nullptr) {
        uint64_t out = 0;
        return visiaengine_create_group(h_, name, &out) == 0 ? out : 0;
    }
    int32_t set_parent(uint64_t child, uint64_t parent) {
        return visiaengine_set_parent(h_, child, parent);
    }
    uint64_t get_parent(uint64_t entity) { return visiaengine_get_parent(h_, entity); }
    int32_t set_group_offset(uint64_t group, double dx, double dy, double dz) {
        return visiaengine_set_group_offset(h_, group, dx, dy, dz);
    }
    int32_t get_group_offset(uint64_t group, double out[3]) {
        return visiaengine_get_group_offset(h_, group, out);
    }

    /* CAPI-36..38: typed property diff-update (see header clause). */
    int32_t update_entity_attr_f64(uint64_t entity, const char *name, double v) {
        return visiaengine_update_entity_attr_f64(h_, entity, name, v);
    }
    int32_t update_entity_attr_str(uint64_t entity, const char *name, const char *v) {
        return visiaengine_update_entity_attr_str(h_, entity, name, v);
    }
    int32_t update_entity_attr_bool(uint64_t entity, const char *name, int32_t v) {
        return visiaengine_update_entity_attr_bool(h_, entity, name, v);
    }
    /* CAPI-35: HTTP tile source (discovery-driven; see header clause). */
    int32_t set_tile_source_http(const char *url, uint32_t z) {
        return visiaengine_set_tile_source_http(h_, url, z);
    }
    int32_t set_tile_view(double min_x, double min_y, double max_x, double max_y) {
        return visiaengine_set_tile_view(h_, min_x, min_y, max_x, max_y);
    }
    /* CAPI-42/43 (V2.2): node-local rigid transform (column-major x16). */
    int32_t set_node_transform(uint64_t entity, const double *m) {
        return visiaengine_set_node_transform(h_, entity, m);
    }
    int32_t get_node_transform(uint64_t entity, double *out) {
        return visiaengine_get_node_transform(h_, entity, out);
    }
    /* CAPI-41 (I band): .hdr environment -> SH-9 irradiance (demo-grade). */
    int32_t load_env_hdr(const char *path) {
        return visiaengine_load_env_hdr(h_, path);
    }
    /* CAPI-39/40 (R band): raster basemap (flat quads — not terrain). */
    int32_t load_raster_dir(const char *path, uint32_t z) {
        return visiaengine_load_raster_dir(h_, path, z);
    }
    int32_t set_raster_view(double min_x, double min_y, double max_x, double max_y) {
        return visiaengine_set_raster_view(h_, min_x, min_y, max_x, max_y);
    }
    int32_t navigate_click(float px, float py, std::uint64_t dur_ms) {
        return visiaengine_navigate_click(h_, px, py, dur_ms);
    }
    int32_t get_camera(VeCameraPose &pose) {
        pose.struct_size = sizeof(VeCameraPose);
        return visiaengine_get_camera(h_, &pose);
    }

    // CAPI-20 section clipping (B9 thin forwarding, no SDD account; VeClipPlane = header POD used directly)
    // CAPI-21/22 text labels (thin forwarding; VeLabelSpec header POD used directly)
    int32_t load_font(const std::uint8_t *data, std::size_t len) {
        return visiaengine_load_font(h_, data, len);
    }
    int32_t add_label(const VeLabelSpec &spec, std::uint64_t *out_entity) {
        return visiaengine_add_label(h_, &spec, out_entity);
    }

    int32_t set_clips(const VeClipPlane *planes, std::size_t n) {
        return visiaengine_set_clips(h_, planes, n);
    }
    int32_t get_clips(VeClipPlane *buf, std::size_t cap) const {
        return visiaengine_get_clips(h_, buf, cap);
    }

    int32_t add_mesh(const MeshDesc &desc, std::uint64_t *out_entity) {
        return visiaengine_add_mesh(h_, &desc, out_entity);
    }
    int32_t remove_entity(std::uint64_t entity) { return visiaengine_remove_entity(h_, entity); }
    // CAPI-18 thin forwarding (the out shape may legally be 0, do not treat it as a sentinel -- CAPI-01; degenerate desc already rejected at the C gate)
    int32_t add_points(const VePointMark *marks, std::uint64_t count, std::uint64_t *out_entity) {
        const VePointsDesc d{sizeof(VePointsDesc), marks, count};
        return visiaengine_add_points(h_, &d, out_entity);
    }
    // CAPI-19 thin forwarding (report out-parameter by reference; struct_size stamped by the shell)
    int32_t load_pcl(const char *path, VePclPolicy policy, std::uint64_t *out_entity, VePclReport *report) {
        report->struct_size = sizeof(VePclReport);
        return visiaengine_load_pcl(h_, path, policy, out_entity, report);
    }
    // CAPI-17 thin forwarding (the callback lifetime is guaranteed by the host: the engine does not own cb; the three shell semantics live in the header comment)
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
