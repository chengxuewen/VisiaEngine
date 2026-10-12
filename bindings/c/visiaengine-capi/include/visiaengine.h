/* visiaengine.h -- hand-written human-reviewed skeleton (I0); the 14 entry signatures follow .omo/plans/visiaengine-host-embed.md v1.4 §2,
 * filled function by function from I1 on: one line of purpose + one line of threadsafety per function ([FFI-R:CS-M2]). */
#ifndef VISIAENGINE_H
#define VISIAENGINE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {   /* C++ host guard (Qt/Flutter embed surface; batch C2 ctest probe proved the gap) */
#endif

/* Handles are opaque values: only 0=invalid/failure and UINT64_MAX=miss carry public meaning;
 * no decomposition, no cross-process/cross-restart persistence (encoding and generation rules live in the CAPI-01 contract, the bit layout
 * is not exposed [FFI-R:CS-R1]). */
typedef uint64_t VeEngine;

typedef struct { size_t struct_size; uint32_t kind;
                 float px, py, wheel; uint32_t button, mods; } VeInput;

#define VE_OK            0
#define VE_ERR_ARG      (-1)   /* invalid/stale/foreign handle, null pointer, struct_size too small, kind out of range */
#define VE_ERR_STATE    (-2)   /* not attached / non-owner thread (thread affinity: entries callable only on the owner thread) */
#define VE_ERR_IO       (-3)
#define VE_ERR_PANIC    (-4)   /* fence catch-all; read diagnostics via last_error */
#define VE_ERR_SIZE     (-5)   /* buffer/dimension runtime magnitude */
#define KIND_PTR_MOVE 1u
#define KIND_PTR_DOWN 2u
#define KIND_PTR_UP   3u
#define KIND_WHEEL    4u
#define KIND_KEY      5u
#define VE_MISS       UINT64_MAX  /* pick/entity_at miss and out-of-range sentinel */

/* threadsafety per entry: owner thread; create records the owner. Exceptions={abi_version, last_error}.
 * Host duty: when input thread != paint thread (Qt-like), forward to the owner thread. */
uint32_t   visiaengine_abi_version(void);                       /* (major<<16)|minor; v0 major=1 */
uint64_t   visiaengine_create_headless(uint32_t w, uint32_t h); /* 0=failure (read diagnostics via last_error) */
int32_t    visiaengine_destroy(uint64_t ve);                    /* re-entry on released = -1 (double destroy not swallowed) */
int32_t    visiaengine_attach(uint64_t ve, uint64_t win, uint64_t display_or_hinstance, int32_t kind); /* 0=x11 1=win32 2=cocoa; repeat attach = release old target then reconfigure */
int32_t    visiaengine_load_gltf(uint64_t ve, const char *path);
int32_t    visiaengine_load_geojson(uint64_t ve, const char *path);
int32_t    visiaengine_on_input(uint64_t ve, const VeInput *in); /* sole input entry; 1=consumed 0=not this engine's event */
int32_t    visiaengine_render(uint64_t ve);                     /* pull-model one frame (host decides rAF/paint timing itself) */
int32_t    visiaengine_readback(uint64_t ve, uint8_t *buf, uint64_t len);
int32_t    visiaengine_viewport(uint64_t ve, uint32_t w, uint32_t h);
const char *visiaengine_last_error(uint64_t ve);                /* thread-bound; valid until the next error write on that thread; must not be used as branch criteria */
int32_t    visiaengine_entity_count(uint64_t ve);
uint64_t   visiaengine_pick(uint64_t ve, float px, float py);   /* entity handle (foreign space); VE_MISS=miss */
uint64_t   visiaengine_entity_at(uint64_t ve, uint32_t index);
/* Attribute read (CAPI-10/11): entity=pick/entity_at output shape; returns 1=hit 0=absent (out/buffer untouched) <0=error.
   str: writes NUL-terminated into buf[cap]; cap too small = -5 with zero partial write (no length-probe mode), host convention 256B. */
int32_t    visiaengine_attr_f64(uint64_t ve, uint64_t entity, const char *key, double *out);
int32_t    visiaengine_attr_str(uint64_t ve, uint64_t entity, const char *key, char *buf, uint64_t cap);
int32_t    visiaengine_attr_bool(uint64_t ve, uint64_t entity, const char *key, int32_t *out);

/* -- B1 data band (CAPI-13..16, abi minor=2) --
   visibility: strict visible in {0,1}; hidden items invisible to render/pick but the enumeration domain is unchanged (layer-toggle family). */
int32_t visiaengine_entity_set_visible(uint64_t ve, uint64_t entity, int32_t visible);
int32_t visiaengine_entity_visible(uint64_t ve, uint64_t entity); /* 1/0/-1 */

/* Procedural mesh add (CAPI-15): struct_size forward-compat gate; pointers read only during the call;
   normals may be NULL = engine synthesizes +Z. Return-code scheme: 0=success writing *out_entity (shape may = 0,
   host must not judge validity by 0 -- enumeration domain / pick outputs are the existence criteria, CAPI-01 division),
   <0 = failure with zero partial write (same discipline as attr_str). */
typedef struct VeMeshDesc {
    size_t    struct_size;
    const float    *positions;   /* [x,y,z] × n_positions */
    const float    *normals;     /* may be NULL; otherwise same length */
    const uint32_t *indices;
    uint64_t  n_positions;
    uint64_t  n_indices;
    const float    *base_color;  /* 4-tuple, sRGB/CSS convention values (backend chokepoint converts to linear, CORE-16) */
    const double   *origin;      /* 3-tuple (D7 far-coordinate semantics) */
    /* N6 (REND-45) GGX PBR values -- struct_size forward-compat gate governs; legacy zero-init
       hosts get roughness<=0 mapped to the 1.0 dielectric default engine-side. */
    float roughness;   /* GGX perceptual roughness (alpha = r*r), <=0 → 1.0 */
    float metalness;   /* metallic factor, 0 = dielectric */
} VeMeshDesc;
int32_t  visiaengine_add_mesh(uint64_t ve, const VeMeshDesc *desc, uint64_t *out_entity);
/* Delete (CAPI-16): cleans the items/attributes/visibility three faces; slot generation +1; re-entry with an old handle = -1, same scheme as double destroy. */
int32_t  visiaengine_remove_entity(uint64_t ve, uint64_t entity);
/* Event push (CAPI-17): register/replace/NULL-unregister; load_* loops fire per-element **synchronously**
   PROGRESS(a=done b=total, monotonic with terminal state done==total); the failure exit fires at the same moment as the return value
   ERROR(a=return code in signed form b=0). Firing thread = calling thread (after the async pipeline lands = loader thread,
   id/order unchanged -- forward-compatibility declaration). Re-entering visiaengine_* inside a callback = undefined (forbidden). */
typedef enum { VE_EVT_LOAD_PROGRESS = 1, VE_EVT_LOAD_ERROR = 2 } VeEvent;
typedef void (*VeEventCb)(void *user, uint32_t event, uint64_t a, uint64_t b);
int32_t  visiaengine_set_event_callback(uint64_t ve, VeEventCb cb, void *user);
/* Point-cloud passthrough (CAPI-18): one entity one DrawPoints; non-finite accepted as-is = host responsibility (only the load side
   discards by four-class classification); management domain = outside items (same scheme as pure geo markers, v0 explicit account). Color = host sRGB face (CORE-16). */
typedef struct {
    float pos[3];      /* host-space local coordinates (origin=0) */
    float radius_px;   /* screen pixels (REND-30 semantics) */
    float color[3];    /* sRGB/CSS convention */
} VePointMark;
typedef struct {
    size_t             struct_size; /* forward-compat gate = sizeof(VePointsDesc) */
    const VePointMark *marks;
    uint64_t           count;
} VePointsDesc;
int32_t  visiaengine_add_points(uint64_t ve, const VePointsDesc *desc, uint64_t *out_entity);
/* Point-cloud file load (CAPI-19): v0 = PLY (ascii/binary_le); four dirty-data classes per IO-03 (column/element-level
   unsupported does not trigger FastFail -- only point-level discard/truncation rejects wholesale); cloud = single entity, meta (point_count/
   format/bbox 8 columns) via attr_* prefixed queries; capacity gate 4M declared-first-rejected (IO-05). */
typedef enum { VE_PCL_FASTFAIL = 0, VE_PCL_LENIENT = 1 } VePclPolicy;
typedef struct {
    size_t   struct_size;
    uint32_t dropped_non_finite;
    uint32_t dropped_out_of_domain;
    uint32_t dropped_unsupported; /* column/element-level count, does not trigger FastFail (IO-03 landing clarified) */
    uint32_t truncated_points;
    uint64_t kept;
} VePclReport;
int32_t  visiaengine_load_pcl(uint64_t ve, const char *path, VePclPolicy policy,
                              uint64_t *out_entity, VePclReport *out_report);
/* Section clipping (CAPI-20): world-coefficient planes [n,d] (normal points to the kept side, dot(n,P)+d>=0 keeps, on-plane =
   kept; multiple planes AND). Normalization done engine-side (readback always unit form); n=0 = the only clear form; n>4 reject-before-read.
   render pipeline fs discard (shadow casters clipped the same) + pick hit-point retry filter consistent within the frame. */
typedef struct { double nx, ny, nz, d; } VeClipPlane;
int32_t  visiaengine_set_clips(uint64_t ve, const VeClipPlane *planes, size_t n);
/* Readback: return >=0 = current plane count (errors = dedicated negative codes); buf NULL = count only; cap truncation = write min but return the true count. */
int32_t  visiaengine_get_clips(uint64_t ve, VeClipPlane *buf, size_t cap);
/* Camera fly (CAPI-23/24, 5a): the wall-clock driver lives in the engine (host render() is the tick, zero dt duty --
   the correct answer to the D8 no-main-loop fact). Pose = six pose components + fov, all f64; near/far stay out of pose (depth domain always
   current values, PIT-5/REND-14 locked). dur_ms=0 = teleport form (not an error). Re-entry mid-flight = reroute (from = current
   sampled pose, first frame continuous). Interrupt: any pointer/wheel input cancels (MapLibre A branch); keys do not interrupt. */
typedef struct {
    size_t struct_size; /* forward-compat gate = sizeof(VeCameraPose) */
    double target[3];   /* orbit center (world space, D7) */
    double yaw, pitch;  /* radians; flight takes the shortest arc (mod 2pi pose-equivalent) */
    double dist;        /* spherical distance >0 (perspective) */
    double zoom;        /* ortho half-width >0 (ortho family consumes) */
    double fov;         /* perspective vertical FOV in (0, pi) */
} VeCameraPose;
int32_t  visiaengine_fly_to(uint64_t ve, const VeCameraPose *pose, uint64_t dur_ms);
/* State: 1=done (includes idle) / 0=flying; out_t01 written only while flying [0,1) (NULL = state only). */
int32_t  visiaengine_fly_state(uint64_t ve, double *out_t01);
/* Text labels (CAPI-21/22, tier-1 engine self-managed): load_font injects TTF/OTF bytes (replacement-style; no default
   font = text pipeline dormant). add_label world anchor f64 [D7], color = host sRGB/CSS face (CORE-16 converts to linear at the
   chokepoint), text = NUL-terminated UTF-8; success rc=0 writes *out_entity (shape 0 is legal = CAPI-01, judge success by
   rc = CAPI-15 scheme). Management domain = outside items (same scheme as add_points CAPI-18): not in remove/enumeration,
   explicit account v0. Font not loaded = add_label explicit rejection (timing gate; dormant semantics belong only to the GEO-25 style domain). */
typedef struct {
    size_t             struct_size; /* forward-compat gate = sizeof(VeLabelSpec) */
    double             pos[3];
    float              color[4];    /* sRGB/CSS */
    float              size_px;
    const char        *text;        /* UTF-8, NUL terminated */
} VeLabelSpec;
int32_t  visiaengine_load_font(uint64_t ve, const uint8_t *data, size_t len);
int32_t  visiaengine_add_label(uint64_t ve, const VeLabelSpec *spec, uint64_t *out_entity);
/* Minimap navigation (CAPI-25..27, 5b second wave): main + fixed secondary viewport on one surface, dual projection (first real
   consumption of the 5b multi-pass base). Minimap rig = top view following a target derived from the main one (zero sync entries); the host reports only screen px,
   in/out-of-region decision and dual-projection routing live inside the engine (non-host dispatch = VeInput untouched, the strict struct_size gate does not break old hosts). */
typedef struct {
    size_t struct_size; /* forward-compat gate = sizeof(VeMapView) */
    float  fx, fy, fw, fh; /* normalized 0..1 (auto-follows resize); fw/fh>0 */
    double zoom;           /* top-view half-width >0 (smaller = farther) */
} VeMapView;
/* CAPI-25: enable/disable the minimap. cfg NULL = clear back to the old single-frame full-face path (byte-wise canary = old behavior);
   domain rejection (normalized out of [0,1] / zero size / zoom<=0 / NaN) = VE_ERR_ARG with zero side effects. */
int32_t  visiaengine_set_map(uint64_t ve, const VeMapView *cfg);
/* CAPI-26: minimap click navigation (set_map first, then call; outside region / no map = reject with no covert reroute). Two stages:
   top-view ortho ray picks an entity first -> ground z=0 fallback; the main rig keeps angle and distance, swapping only the target (ruling e);
   dur_ms goes through the CAPI-23 flight path (0 = teleport). px = host screen coordinates (top-left origin, same system as on_input). */
int32_t  visiaengine_navigate_click(uint64_t ve, float px, float py, uint64_t dur_ms);
/* CAPI-27: main camera pose readback (host HUD / arrival assertions / single source for "current camera = render truth").
   out pre-stamped by the host with struct_size=sizeof(VeCameraPose) (checked before writing); NULL = rejected. */
int32_t  visiaengine_get_camera(uint64_t ve, VeCameraPose *out);

/* CAPI-28: mount an MVT tile directory (FileSource tree {root}/{z}/{x}/{y}.mvt; z in [0,30]).
   Return = mounted tile count (>0 success); 0/negative = error (directory absent / no tiles / z out of domain), error string via TLS. */
int32_t  visiaengine_load_mvt_dir(uint64_t ve, const char *path, uint32_t z);
/* CAPI-29: feed the viewport bbox (EPSG:3857 meters, min<max finite) -> scheduler visible->ensure.
   Return = visible tile count; negative = rejected (not mounted / out of domain). When the visible set changes, batch geometry is rebuilt (takes effect on the next render). */
int32_t  visiaengine_set_tile_view(uint64_t ve, double min_x, double min_y,
                                   double max_x, double max_y);

/* CAPI-39 (R band): mount a RASTER tile directory ({root}/{z}/{x}/{y}.png;
   flat ground quads — flat raster basemap, NOT terrain/drape/LOD; N1.2
   wording discipline). Returns mounted tile count; 0/negative = error.
   CAPI-40 (R band): feed a viewport bbox (3857 meters) to the raster layer;
   returns the count of mounted tiles visible in that view. */
int32_t  visiaengine_load_raster_dir(uint64_t ve, const char *path, uint32_t z);
int32_t  visiaengine_set_raster_view(uint64_t ve, double min_x, double min_y,
                                     double max_x, double max_y);

/* CAPI-41 (I band): load an .hdr environment (RGBE equirect) and install its
   SH-9 irradiance as the global env light (demo-grade; replaces the legacy
   constant ambient). 0 = ok, negative = error (unreadable/invalid file). */
int32_t  visiaengine_load_env_hdr(uint64_t ve, const char *path);

/* CAPI-42/43 (V2.2): node-local rigid transform write/read on a mounted glTF
   entity — column-major f64 x16 (GLTF-08 shape). The parent chain composes
   at draw time. 0 = ok; negative = error (NULL / non-finite / unknown). */
int32_t  visiaengine_set_node_transform(uint64_t ve, uint64_t entity, const double *m);
int32_t  visiaengine_get_node_transform(uint64_t ve, uint64_t entity, double *out);

/* CAPI-36..38 (N2.4): typed property diff-update — host-driven live-data
   writes on loaded entities (digital-twin face). Typed columns: f64 / str
   (UTF-8) / bool (1/0). First write on a name TYPES the column; writing a
   different type to an existing column = VE_ERR_ARG (type-conflict, zero
   partial write — CORE-11/12 first-write-typed). Reads via attr_* see the
   override first (diff wins over source). Unknown entity / NULL name /
   non-UTF8 = VE_ERR_ARG. Zero render-face effect. */
int32_t  visiaengine_update_entity_attr_f64(uint64_t ve, uint64_t entity,
                                            const char *name, double v);
int32_t  visiaengine_update_entity_attr_str(uint64_t ve, uint64_t entity,
                                            const char *name, const char *v);
int32_t  visiaengine_update_entity_attr_bool(uint64_t ve, uint64_t entity,
                                             const char *name, int32_t v);

/* CAPI-35 (N1.4): mount an HTTP MVT tile source (URL template
   {root}/{z}/{x}/{y}.mvt). Discovery-driven: HTTP has no directory listing —
   the tile set is DISCOVERED by set_tile_view viewport enumeration, and the
   render loop pumps the scheduler (4 tiles/frame budget; 5s per-request
   timeout guard). Returns 0 = mounted (count semantics intentionally differ
   from CAPI-28: there is no directory to scan); negative = rejected
   (ve=0 / url NULL or empty / z>30), error string via TLS. */
int32_t  visiaengine_set_tile_source_http(uint64_t ve, const char *url, uint32_t z);

/* CAPI-30: create a group node (scene-tree container). name=NULL = anonymous; out_entity receives the u64 handle.
   Returns VE_OK; failure (panic/ve=0/out=NULL/name not UTF-8) = negative error code. */
int32_t  visiaengine_create_group(uint64_t ve, const char *name, uint64_t *out_entity);
/* CAPI-31: reparent. child attaches under parent; parent=0 = detach back to root. 0=success, negative=rejected. */
int32_t  visiaengine_set_parent(uint64_t ve, uint64_t child, uint64_t parent);
/* CAPI-32: query the parent. Returns the parent handle (0=root/no parent/err). */
uint64_t visiaengine_get_parent(uint64_t ve, uint64_t entity);
/* CAPI-33: set the group translation (world meters, f64x3). group must be a group node created by CAPI-30. 0=success. */
int32_t  visiaengine_set_group_offset(uint64_t ve, uint64_t group,
                                       double dx, double dy, double dz);
/* CAPI-34: read the group's own translation (non-accumulated). out needs a pre-allocated 3xf64. 0=success, negative=rejected. */
int32_t  visiaengine_get_group_offset(uint64_t ve, uint64_t group, double *out);

#ifdef __cplusplus
}  /* extern "C" */
#endif

#endif /* VISIAENGINE_H */
