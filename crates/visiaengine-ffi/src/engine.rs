//! VeEngine entity (plan v1.4 §3): headless image output made real (I2).
//! Load = CPU parse -> MeshCore upload; picking = REND-21..24 reuse; policy = input -> camera.
//! The bytes-first load entry is reserved for batch 7 ([FFI-R:EP-附]/P2): I2 goes path, upgraded to bytes before J1.

use visiaengine_core::{EntityId, Scene};

use crate::ffi::enc_entity; // CAPI-10 reverse-lookup key = host-visible shape (CAPI-08 encoding single source)
use visiaengine_io_text::GLYPH_ATLAS_PX;
use visiaengine_render::{
    Camera, CameraRig, DrawCommand, Frame, MaterialDesc, MaterialId, MeshCandidate, MeshDesc,
    MeshId, PointCloudCandidate, PointMark, PointTableDesc, RenderBackend, StrokeSeg,
    StrokeTableDesc, Viewport, ViewportRect, pick_meshes, pick_points, ray_ground_intersect,
    rebase::compose_mvp, screen_to_ray_ortho, screen_to_ray_persp,
};

/// B2 point-picking screen radius (REND-38 default; same screen-pixel domain as REND-30 radius_px).
const PICK_RADIUS_PX: f32 = 6.0;
use visiaengine_render_wgpu::HeadlessBackend;
use visiaengine_render_wgpu::MultiClearPolicy;
use visiaengine_render_wgpu::surface::Swapchain;

/// Input -> camera engine policy (not event passthrough): press+move = orbit, wheel = shared zoom.
const ORBIT_RATE: f64 = 0.005;
const ZOOM_RATE: f64 = 0.1;
const FIT_PAD: f64 = 1.25;

struct DrawItem {
    mesh: MeshId,
    material: MaterialId,
    entity: EntityId,
    origin: [f64; 3],
    /// V2.2: node-local transform (IDENTITY for legacy loads; glTF mounts
    /// carry the node-local TRS — parent chain composes at draw time).
    transform: [[f64; 4]; 4],
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

/// Render-target state (CAPI-06: headless and window coexist exclusively in one engine; a failed attach leaves the current state untouched).
enum TargetMode {
    Headless,
    Window(Swapchain),
}

/// Projection mode (geo = ortho fit, gltf = persp, rebuilt on viewport change).
#[derive(Clone, Copy, PartialEq)]
pub enum Proj {
    Persp,
    Ortho,
}

/// N2.1 (REND-41): morphTime envelope — None = legacy discrete Proj::mode
/// (every legacy path byte-identical by construction); Some(t) = continuous
/// projection morph, t=0 → ortho bytes, t=1 → persp bytes (endpoint canaries).
/// Slider semantics: set_morph_time clamps into [0,1] (diverges from
/// EdlSetup's reject — t is a slider, strength is a threshold; clause notes).
#[derive(Clone, Copy, Debug, PartialEq)]
struct MorphT(f64);

/// CAPI-23 thruster state: from = takeoff snapshot / to = target / start-stop clocks / duration.
struct Fly {
    from: CameraRig,
    to: CameraRig,
    start_ms: f64,
    dur_s: f64,
}

/// Dual-platform monotonic millisecond clock: native = process-anchor delta (Instant monotonic semantics kept), wasm = Date.now()
/// (std::time is unavailable at runtime on wasm32-unknown-unknown -- Web is this SDK's main stage,
/// and this shim keeps flyTo from hitting capi_guard -4 frozen-death).
#[must_use]
fn now_ms() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::LazyLock;
        static T0: LazyLock<std::time::Instant> = LazyLock::new(std::time::Instant::now);
        #[allow(clippy::cast_precision_loss)]
        return T0.elapsed().as_secs_f64() * 1000.0;
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now()
    }
}

pub struct Engine {
    pub w: u32,
    pub h: u32,
    rig: CameraRig,
    mode: Proj,
    /// N2.1 (REND-41): Some(t) = continuous projection morph active.
    morph: Option<MorphT>,
    zoom: f64,
    down: bool,
    last: (f32, f32),
    backend: HeadlessBackend,
    target: TargetMode,
    scene: Scene, // EntityId generator (slot+gen in the same space as the core slab, shared by pick/entity_at)
    items: Vec<DrawItem>,
    /// geo tile-expansion family commands (GEO-24/WGPU-17/18: table built at mount, replayed per frame alongside items)
    extra_cmds: Vec<DrawCommand>,
    /// CAPI-12: load_geojson retention face (one doc per load, row = feature index GEO-17)
    geo_docs: Vec<visiaengine_geo::GeoDocument>,
    /// CAPI-10: entity (C u64 shape, same encoding as the pick output; gen included for ABA immunity) -> (doc ordinal, row)
    attr_of: std::collections::HashMap<u64, (usize, usize)>,
    /// N2.4 (CAPI-36): host-driven property diff-update override layer.
    /// Checked BEFORE geo_docs/pcl_meta in attr_* reads; typed columns
    /// (first-write-typed per CORE-11/12 — a type-mismatched update is
    /// rejected, not coerced).
    attr_overrides: std::collections::HashMap<u64, visiaengine_core::AttrSet>,
    /// CAPI-13: hidden-entity shape table (render/pick filter domain; enumeration domain unchanged, cleanup rides CAPI-16 deletion)
    hidden: Vec<u64>,
    /// Scene tree (CAPI-30): group-node display names (engine-local).
    node_names: std::collections::HashMap<u64, String>,
    /// CAPI-20: section-clip world planes (None/empty = keep all; render feeds Frame.clip per frame,
    /// and the pick hit-point keeps predicate's negative side = exclude-and-retry [WGPU-21 pipeline face shares the same world frame])
    clip: Option<visiaengine_render::ClipSetup>,
    /// CAPI band N1.5: EDL opt-in switch (None=off, byte-identical legacy path).
    edl: Option<visiaengine_render::EdlSetup>,
    /// N4 post-effect chain (empty = the bit-for-bit legacy path).
    post: Vec<visiaengine_render::PostEffect>,
    /// Host-set clear color (linear-domain rgb triple; None = the deep-slate
    /// default, the bit-for-bit legacy path).
    clear: Option<[f32; 3]>,
    /// 4f shadow pass (None = the legacy lit path bit-for-bit).
    shadow: Option<visiaengine_render::ShadowSetup>,
    /// CAPI-23: flight thruster (wall clock lives in the engine -- the layering decision behind zero std::time in the render crate;
    /// None = merged idle/done state)
    fly: Option<Fly>,
    /// 5b second wave: minimap (None = off; old hosts perceive nothing)
    map: Option<MapView>,
    frame_cache: Option<Vec<u8>>,
    /// CAPI-19: point-cloud cloud-level meta (shape -> single-row AttrSet; attr_* prefixed-lookup domain, geo layer checked first).
    pcl_meta: std::collections::HashMap<u64, visiaengine_core::AttrSet>,
    /// B2 point-picking: the retained world shape of each point-cloud entity (entity bits -> origin/transform/positions).
    /// marks were originally discarded once absorbed into the GPU table; retention = data face of the CPU screen-space nearest-point predicate.
    /// ponytail: linear scan per pick; hover-throttle = rung 1, BVH = rung 2 if
    /// interaction patterns demand (1M pts ≈ 30-100 ms/query on weak hosts).
    point_clouds: std::collections::HashMap<u64, PointCloudState>,
    /// P1 tile layer (CAPI-28/29): None = no tile face (render touches zero); Some = scheduler + batch geometry.
    /// Mounted tile layers, draw order == mount order (band T, plan
    /// ledger-tiles-keys-2026-10-08). One layer per kind (Adjudication 2 = A):
    /// `set_tile_view` drives the vector layer, `set_raster_view` the raster one.
    layers: Vec<TileLayer>,
    /// I band (CAPI-41): SH-9 environment irradiance (linear RGB), pushed to
    /// the backend each render pass; None = legacy constant ambient.
    env_sh: Option<[[f32; 3]; 9]>,
    /// CAPI-21: label font (None = text pipeline dormant -- styles/entry exist, zero output [GEO-25/CAPI-22])
    font: Option<visiaengine_io_text::FontFace>,
    /// glyph atlas persisting across loads (dirty -> full re-upload before render [WGPU-24])
    glyphs: visiaengine_io_text::GlyphCache,

    /// CAPI-17 event anchors (fn/user raw pointers in usize form; firing domain = owner thread,
    /// same scheme as the CAPI-03 affinity gate, hence the Send wrapper is safe).
    evt: Option<EvtSink>,
}

/// B2 point-picking data face: the retained world state of a single point-cloud entity (same scheme as DrawPoints: origin+transform).
struct PointCloudState {
    entity: EntityId,
    origin: [f64; 3],
    transform: [[f64; 4]; 4],
    positions: Vec<[f32; 3]>,
}

/// P1: union bbox of tile ids (3857 meters).
fn union_bbox(ids: &[visiaengine_io_tiles::TileId]) -> (f64, f64, f64, f64) {
    let mut acc: Option<(f64, f64, f64, f64)> = None;
    for id in ids {
        let (x0, y0, x1, y1) = id.bbox();
        acc = Some(match acc {
            None => (x0, y0, x1, y1),
            Some((a, b, c, d)) => (a.min(x0), b.min(y0), c.max(x1), d.max(y1)),
        });
    }
    acc.unwrap_or((0.0, 0.0, 0.0, 0.0))
}

/// N1.4 (CAPI-35): rebuild HTTP-layer batch geometry from the Done tile set.
/// Same per-tile geometry semantics as load_mvt_dir (poly→mesh, line→stroke,
/// point→point); center baked from the current ids' union bbox (PIT-8 domain).
fn rebuild_tile_batch(layer: &mut TileLayer, done_ids: Vec<visiaengine_io_tiles::TileId>) {
    let (min_x, min_y, max_x, max_y) = union_bbox(&done_ids);
    let center = [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0, 0.0];
    let (mut wpos, mut widx) = (Vec::new(), Vec::new());
    let mut strokes = Vec::new();
    let mut points = Vec::new();
    for id in &done_ids {
        let Some(tile) = layer.set.decoded(id) else {
            continue;
        };
        let lite = visiaengine_io_tiles::GeoTile::from_layers(*id, &tile.layers);
        for f in &lite.features {
            let class = f.attrs.get("class").cloned().unwrap_or_default();
            match &f.geom {
                visiaengine_io_tiles::TileGeom::Poly(ring) => {
                    let start = wpos.len() as u32;
                    for p in ring {
                        wpos.push([(p[0] - center[0]) as f32, (p[1] - center[1]) as f32, 0.0]);
                    }
                    for i in 1..ring.len().saturating_sub(1) as u32 {
                        widx.extend([start, start + i, start + i + 1]);
                    }
                }
                visiaengine_io_tiles::TileGeom::Line(pts) => {
                    let (color, width) = if class == "boundary" {
                        ([0.30, 0.55, 0.55], 3.0)
                    } else {
                        ([0.95, 0.62, 0.18], 9.0)
                    };
                    for w in pts.windows(2) {
                        strokes.push(StrokeSeg::new(
                            [
                                (w[0][0] - center[0]) as f32,
                                (w[0][1] - center[1]) as f32,
                                0.0,
                            ],
                            [
                                (w[1][0] - center[0]) as f32,
                                (w[1][1] - center[1]) as f32,
                                0.0,
                            ],
                            color,
                            width,
                        ));
                    }
                }
                visiaengine_io_tiles::TileGeom::Point(p) => {
                    points.push(PointMark::new(
                        [(p[0] - center[0]) as f32, (p[1] - center[1]) as f32, 0.0],
                        [0.9, 0.25, 0.35],
                        9.0,
                    ));
                }
                visiaengine_io_tiles::TileGeom::MultiPoint(_) => {}
            }
        }
    }
    layer.center = center;
    layer.wpos = wpos;
    layer.widx = widx;
    layer.strokes = strokes;
    layer.points = points;
}

/// Which family a mounted tile layer belongs to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum LayerKind {
    Vector,
    Raster,
}

/// World units the raster basemap sits under vector geometry **when both kinds
/// are mounted** (Adjudication 3 = A). Applied through DrawMesh.origin at draw
/// time, so a single-kind scene keeps byte-identical output (the band canary);
/// revisit toward a no-depth-write basemap pipeline when tilt/3D work lands.
const RASTER_UNDERLAY_Z: f64 = -0.05;

/// P1 tile layer (CAPI-28/29): scheduler + scene-center batch geometry.
/// Render-loop consumes wpos/widx/strokes/points on first frame after mount;
/// uploaded caches GPU table IDs (invalidate only on re-mount, not on set_tile_view).
struct TileLayer {
    /// Which family this layer belongs to (Adjudication 2 = A: kinds are how the
    /// C API addresses layers, so no handle/index surface was added).
    kind: LayerKind,
    set: visiaengine_io_tiles::TileSet,
    ids: Vec<visiaengine_io_tiles::TileId>,
    /// N1.4 (CAPI-35): HTTP layer = discovery mode. `ids` stays empty (no dir
    /// scan exists over HTTP); set_tile_view enumerates freely and the render
    /// loop pumps the scheduler per frame. FileSource layers keep mount-time
    /// ids and the alive-filter (behavior unchanged = canary).
    http: bool,
    /// Batch vertices baked into the scene-center small-value-range frame (PIT-8 safe domain); Draw origin = center.
    center: [f64; 3],
    wpos: Vec<[f32; 3]>,
    widx: Vec<u32>,
    strokes: Vec<StrokeSeg>,
    points: Vec<PointMark>,
    /// R band (CAPI-39): flat ground quads per raster tile (verts relative to
    /// center, uv 0..1). Empty for MVT layers.
    raster_quads: Vec<[([f32; 3], [f32; 2]); 4]>,
    /// R band GPU products (texture + quad mesh + textured material per
    /// tile), built once on the first render pass after mount.
    raster_gpu: Vec<(
        visiaengine_render::TextureId,
        visiaengine_render::MeshId,
        visiaengine_render::MaterialId,
    )>,
    // Upload products (tables built at render time; cached ids prevent re-upload)
    uploaded: Option<(
        visiaengine_render::MeshId,
        visiaengine_render::MaterialId,
        visiaengine_render::TableId,
        visiaengine_render::TableId,
    )>,
}

/// 5b second wave CAPI-25: minimap viewport (normalized rect + world half-height; None = off = old single-frame path, derived follow of the main target).
#[derive(Clone, Copy, Debug)]
pub struct MapView {
    pub fx: f32,
    pub fy: f32,
    pub fw: f32,
    pub fh: f32,
    pub zoom: f64,
}

/// SAFETY: anchors fire only on the owner thread after gating (callback-thread contract in the CAPI-17 clause body).
struct EvtAnchor(usize, usize);
unsafe impl Send for EvtAnchor {}

/// Event sink dual form: native = C anchors (Mutex table needs Send); wasm32 = JS closures (RefCell single-threaded,
/// js_sys::Function legitimately non-Send) -- the Send domain of Engine stays exactly cfg-scoped.
#[cfg(not(target_arch = "wasm32"))]
type EvtSink = EvtAnchor;
#[cfg(target_arch = "wasm32")]
type EvtSink = Box<dyn FnMut(u32, u64, u64)>;

impl Engine {
    #[must_use]
    /// Fresh Scene with slot 0 reserved: handle 0 = "invalid/root" sentinel,
    /// consistent with engine-pool slot-base-1 convention (CAPI-01).
    fn new_scene() -> Scene {
        let mut s = Scene::new();
        let dummy = s.spawn();
        let _ = s.despawn(dummy); // consume slot0gen0
        s
    }

    pub fn new_headless(w: u32, h: u32) -> Option<Self> {
        Some(Self {
            w,
            h,
            rig: CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            mode: Proj::Persp,
            morph: None,
            zoom: 1.0,
            down: false,
            last: (0.0, 0.0),
            backend: HeadlessBackend::new(w, h)?,
            target: TargetMode::Headless,
            scene: Self::new_scene(),
            items: Vec::new(),
            extra_cmds: Vec::new(),
            geo_docs: Vec::new(),
            attr_of: std::collections::HashMap::new(),
            attr_overrides: std::collections::HashMap::new(),
            hidden: Vec::new(),
            node_names: std::collections::HashMap::new(),
            clip: None,
            edl: None,
            post: Vec::new(),
            clear: None,
            shadow: None,
            fly: None,
            map: None,
            font: None,
            glyphs: visiaengine_io_text::GlyphCache::new(),
            frame_cache: None,
            evt: None,
            pcl_meta: std::collections::HashMap::new(),
            point_clouds: std::collections::HashMap::new(),
            layers: Vec::new(),
            env_sh: None,
        })
    }

    #[allow(clippy::too_many_arguments)] // private upload sink (+1 after normals parameterization; splitting the facade would fragment it)
    #[allow(clippy::too_many_arguments)]
    fn upload(
        &mut self,
        entity: EntityId,
        positions: &[[f32; 3]],
        indices: &[u32],
        normals: Option<&[[f32; 3]]>,
        mat: &MaterialDesc,
        uv: &[[f32; 2]],
        origin: [f64; 3],
        transform: [[f64; 4]; 4],
    ) -> Result<(), String> {
        let flat: Vec<[f32; 3]>;
        let nrm = match normals {
            Some(n) => n,
            None => {
                flat = vec![[0.0f32, 0.0, 1.0]; positions.len()];
                &flat[..]
            }
        };
        let mesh = self
            .backend
            .create_mesh(&MeshDesc {
                positions,
                normals: nrm,
                indices,
                uv,
            })
            .map_err(|e| format!("create_mesh: {e:?}"))?;
        let material = self
            .backend
            .create_material_desc(mat)
            .map_err(|e| format!("create_material_desc: {e:?}"))?;
        self.items.push(DrawItem {
            mesh,
            material,
            entity,
            origin,
            transform,
            positions: positions.to_vec(),
            indices: indices.to_vec(),
        });
        Ok(())
    }

    // ===== Loading (CAPI-04) =====

    /// CAPI-17: register the anchor (cb=0 -> unregister).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_event_anchor(&mut self, cb: usize, user: usize) {
        self.evt = if cb == 0 {
            None
        } else {
            Some(EvtAnchor(cb, user))
        };
    }

    /// CAPI-17 wasm form: closure registration (js_sys::Function wrapped into the wasm bridge).
    #[cfg(target_arch = "wasm32")]
    pub fn set_event_fn(&mut self, f: Option<Box<dyn FnMut(u32, u64, u64)>>) {
        self.evt = f;
    }

    /// CAPI-17: event firing point (synchronous direct call inside the owner thread; re-entry = caller no-go zone).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn emit(&mut self, event: u32, a: u64, b: u64) {
        if let Some(EvtAnchor(f, u)) = self.evt {
            let f: unsafe extern "C" fn(*mut std::ffi::c_void, u32, u64, u64) =
                unsafe { std::mem::transmute(f) };
            unsafe { f(u as *mut _, event, a, b) };
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn emit(&mut self, event: u32, a: u64, b: u64) {
        if let Some(f) = self.evt.as_mut() {
            f(event, a, b);
        }
    }

    /// glTF path entry (CAPI-04).
    pub fn load_gltf(&mut self, path: &str) -> Result<usize, String> {
        let doc = visiaengine_io_gltf::load_gltf(path).map_err(|e| format!("{path}: {e}"))?;
        self.mount_gltf(&doc)
    }

    /// glTF bytes entry (batch 7 js face reuse; [FFI-R:EP-附] bytes-first honored).
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    pub fn load_gltf_bytes(&mut self, data: &[u8]) -> Result<usize, String> {
        let doc = visiaengine_io_gltf::load_gltf_bytes(data).map_err(|e| format!("bytes: {e}"))?;
        self.mount_gltf(&doc)
    }

    /// primitive granularity = entity (aligned with the GLTF-01 family; world baked into upload vertices,
    /// DrawItem.origin = 0 -- headless demo small coordinate domain, rebase is identically legal).
    fn mount_gltf(&mut self, doc: &visiaengine_io_gltf::GltfDocument) -> Result<usize, String> {
        let n = doc.entities().len();
        if n == 0 {
            return Err("no primitives".to_string());
        }
        // Load-time framing fit (the E202 gray-screen family law, gltf leg --
        // same shape as mount_geo/E201's BBox gate): node transforms place
        // entities anywhere (hierarchy.glb's leaf sits at x=11); without a
        // fit the initial [0,0,10]->[0,0,0] rig shows only origin-hugging
        // fixtures (twoprim) and pick finds nothing -- render AND pick see
        // the same fitted rig, so the E401 invariant stays constructive.
        let mut bmin = [f64::MAX; 3];
        let mut bmax = [f64::MIN; 3];
        for e in doc.entities() {
            for pt in &e.mesh.positions {
                let v = [
                    e.world[0][0] * f64::from(pt[0])
                        + e.world[1][0] * f64::from(pt[1])
                        + e.world[2][0] * f64::from(pt[2])
                        + e.world[3][0],
                    e.world[0][1] * f64::from(pt[0])
                        + e.world[1][1] * f64::from(pt[1])
                        + e.world[2][1] * f64::from(pt[2])
                        + e.world[3][1],
                    e.world[0][2] * f64::from(pt[0])
                        + e.world[1][2] * f64::from(pt[1])
                        + e.world[2][2] * f64::from(pt[2])
                        + e.world[3][2],
                ];
                for i in 0..3 {
                    bmin[i] = bmin[i].min(v[i]);
                    bmax[i] = bmax[i].max(v[i]);
                }
            }
        }
        if bmin[0] <= bmax[0] {
            let center = [
                (bmin[0] + bmax[0]) / 2.0,
                (bmin[1] + bmax[1]) / 2.0,
                (bmin[2] + bmax[2]) / 2.0,
            ];
            let radius = ((bmax[0] - bmin[0])
                .max(bmax[1] - bmin[1])
                .max(bmax[2] - bmin[2])
                / 2.0)
                .max(0.5);
            let dist = (radius * 4.0).max(6.0);
            // a low depression angle, not a top-down: thin +z-facing scenes
            // (twoprim's quad) would otherwise present edge-on and vanish from
            // BOTH render and pick (the E201 pitch law, scan proof .35 -> red)
            self.rig = CameraRig::look_at(
                [center[0], center[1] + dist * 0.55, center[2] + dist * 0.85],
                center,
                [0.0, 1.0, 0.0],
            );
            self.mode = Proj::Persp;
        }
        // GLTF-11 texture slots -> GPU texture ids (deduped by image slot; one image across primitives shared)
        let mut tex_ids = Vec::with_capacity(doc.textures().len());
        for t in doc.textures() {
            tex_ids.push(
                self.backend
                    .upload_texture(&visiaengine_render::TextureDesc {
                        rgba: &t.rgba,
                        width: t.width,
                        height: t.height,
                    })
                    .map_err(|e| format!("upload_texture: {e:?}"))?,
            );
        }
        let mut done = 0usize;
        for e in doc.entities() {
            done += 1; // CAPI-17: per-entity progress (terminal done==n, same scheme as the returned count)
            self.emit(crate::ffi::VE_EVT_LOAD_PROGRESS, done as u64, n as u64);
            let id = self.scene.spawn();
            // local vertices unchanged; headless persp camera +/-10 domain, twoprim/hierarchy same scale is legal
            let mat = MaterialDesc {
                base_color: e.mesh.base_color,
                texture: e.mesh.texture.and_then(|s| tex_ids.get(s).copied()),
                repeat: [1.0, 1.0],
                // N6 (WGPU-35): glTF factors map straight through — the mock
                // `(1-metallic)*roughness` Lambert inversion is retired.
                specular: 0.0,
                roughness: e.mesh.roughness_factor,
                metallic: e.mesh.metallic_factor,
            };
            self.upload(
                id,
                &e.mesh.positions,
                &e.mesh.indices,
                Some(&e.mesh.normals),
                &mat,
                &e.mesh.uv,
                [0.0; 3],
                e.local,
            )?;
        }
        Ok(n)
    }

    /// GeoJSON path entry (CAPI-04).
    pub fn load_geojson(&mut self, path: &str) -> Result<usize, String> {
        let doc = visiaengine_geo::load_geojson(path).map_err(|e| format!("{path}: {e}"))?;
        self.mount_geo(doc)
    }

    /// GeoJSON bytes entry (js face; Lenient policy -- dirty pieces are discarded so bad data cannot drag down the whole layer,
    /// report export = M2).
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    pub fn load_geojson_bytes(&mut self, data: &[u8]) -> Result<usize, String> {
        let (doc, _rep) =
            visiaengine_geo::parse_geojson_lenient(data).map_err(|e| format!("bytes: {e}"))?;
        self.mount_geo(doc)
    }

    /// feature granularity = entity (one piece may have multiple parts sharing an id); layer bbox center
    /// = origin (D7 shifted discipline, same as geo_viewer); ortho fit framing.
    fn mount_geo(&mut self, doc: visiaengine_geo::GeoDocument) -> Result<usize, String> {
        let [x0, y0, x1, y1] = doc.layer_bbox().ok_or("empty layer")?;
        let origin = [(x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0];
        let radius = ((x1 - x0) / 2.0).max((y1 - y0) / 2.0) * FIT_PAD;
        // degenerate layers (single point/line = radius 0) share the lower bound with the eye -- ortho zoom=0 = a pre-existing must-crash hole,
        // caught back by the S2 label single-point test (all legacy geo tests are multi-point so it was never stepped in)
        self.zoom = radius.max(10.0);
        self.rig = CameraRig::look_at(
            [origin[0], origin[1], radius.max(10.0)],
            origin,
            [0.0, 1.0, 0.0],
        );
        self.mode = Proj::Ortho;
        let neg = [-origin[0], -origin[1]];
        let doc_no = self.geo_docs.len(); // CAPI-12: the ordinal of this load within the retention face
        let mut rows: Vec<(u64, (usize, usize))> = Vec::new();
        let mut n = 0usize;
        // GEO-25: label requests (font dormant = the whole section collects nothing, zero output no error)
        let mut label_marks: Vec<visiaengine_render::LabelMark> = Vec::new();
        let cols = doc.attrs();
        for (row, f) in doc.features().iter().enumerate() {
            let id = self.scene.spawn();
            rows.push((enc_entity(id), (doc_no, row))); // CAPI-10: key = host-visible shape; row = GEO-17 flattened index
            n += 1;
            // CAPI-17: feature-granularity progress (total = features.len, terminal state equals n)
            self.emit(
                crate::ffi::VE_EVT_LOAD_PROGRESS,
                n as u64,
                doc.features().len() as u64,
            );
            if let (Some(face), Some(field)) = (self.font.as_ref(), f.style.text_field.as_deref()) {
                // column semantics first (column present but the row value missing = skip, no literal fallback); column absent = constant text
                let is_col = cols.names().any(|c| c == field);
                let text: Option<&str> = if is_col {
                    cols.str_value(row, field)
                } else {
                    Some(field)
                };
                if let (Some(t), Some([x0, y0, x1, y1])) = (text, f.world_bbox) {
                    let (ax, ay) = ((x0 + x1) / 2.0 + neg[0], (y0 + y1) / 2.0 + neg[1]);
                    let (quads, pen) = visiaengine_io_text::layout(
                        t,
                        face,
                        &mut self.glyphs,
                        f.style.text_size_px,
                    );
                    if !quads.is_empty() {
                        let lin = visiaengine_core::srgb_to_linear(f.style.text_color);
                        let anchor = [ax as f32, ay as f32, 0.1]; // always-top 0.1 (no depth competition, pure ordering semantics)
                        label_marks.extend(quads.iter().map(|q| {
                            visiaengine_render::LabelMark::new(
                                anchor,
                                lin,
                                [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                                [
                                    q.size_px[0],
                                    q.size_px[1],
                                    q.top_left_px[0] - pen / 2.0, // MapLibre default center anchor
                                    q.top_left_px[1],
                                ],
                            )
                        }));
                    }
                }
            }
            let parts = visiaengine_geo::tessellate(&f.kind.shifted(neg), &f.style)
                .map_err(|e| format!("tessellate: {e}"))?;
            // [PIT-8 extraction site] LineStrip->StrokeSeg / Marker->PointMark conversion and
            // geo_viewer/geo_pipeline same shape in three places -- the third consumer has appeared; extract a shared helper after 4de.
            for gp in parts {
                match gp {
                    visiaengine_geo::GeoPart::Fill(p) => {
                        if p.positions.is_empty() || p.indices.len() < 3 {
                            continue;
                        }
                        let mat = MaterialDesc {
                            base_color: p.color,
                            texture: None,
                            repeat: [1.0, 1.0],
                            specular: 0.0,
                            roughness: 1.0, // N6: dielectric legacy band (WGPU-35 probe ledger)
                            metallic: 0.0,
                        };
                        self.upload(
                            id,
                            &p.positions,
                            &p.indices,
                            None,
                            &mat,
                            &[],
                            origin,
                            IDENTITY,
                        )?;
                    }
                    visiaengine_geo::GeoPart::Strokes(strips) => {
                        let mut segs = Vec::new();
                        for s in strips {
                            for w in s.pts.windows(2) {
                                segs.push(StrokeSeg::new(
                                    [w[0][0], w[0][1], 0.0],
                                    [w[1][0], w[1][1], 0.0],
                                    s.color,
                                    s.width_px,
                                ));
                            }
                        }
                        if segs.is_empty() {
                            continue;
                        }
                        let table = self
                            .backend
                            .create_strokes(&StrokeTableDesc { data: &segs })
                            .map_err(|e| format!("strokes: {e}"))?;
                        self.extra_cmds.push(DrawCommand::DrawStrokes {
                            table,
                            origin,
                            transform: IDENTITY,
                        });
                    }
                    visiaengine_geo::GeoPart::Markers(ms) => {
                        let marks: Vec<PointMark> = ms
                            .iter()
                            .map(|m| {
                                PointMark::new([m.pos[0], m.pos[1], 0.0], m.color, m.radius_px)
                            })
                            .collect();
                        if marks.is_empty() {
                            continue;
                        }
                        let table = self
                            .backend
                            .create_points(&PointTableDesc { data: &marks })
                            .map_err(|e| format!("points: {e}"))?;
                        self.extra_cmds.push(DrawCommand::DrawPoints {
                            table,
                            origin,
                            transform: IDENTITY,
                        });
                    }
                }
            }
        }
        if !label_marks.is_empty() {
            let table = self
                .backend
                .create_labels(&visiaengine_render::LabelTableDesc { data: &label_marks })
                .map_err(|e| format!("labels: {e}"))?;
            self.extra_cmds.push(DrawCommand::DrawLabels {
                table,
                origin,
                transform: IDENTITY,
            });
        }
        self.geo_docs.push(doc); // CAPI-12 retention face (partial failure = not pushed, the reverse-lookup mapping is not published synchronously)
        for (id, r) in rows {
            self.attr_of.insert(id, r);
        }
        Ok(n)
    }

    // ===== Input/camera (CAPI-05) =====

    pub fn apply_input(&mut self, kind: u32, px: f32, py: f32, wheel: f32) -> Option<bool> {
        // CAPI-23 interrupt semantics (MapLibre A branch): pointer/wheel cancel at once; keys (5) and unknown codes do not interrupt
        if matches!(kind, 1 | 2 | 4) {
            self.fly = None;
        }
        // CAPI-25 companion: pointer/wheel inside the minimap region = consumed no-op (v0 has no drag-pan semantics; navigation goes through navigate_click)
        if matches!(kind, 1..=4)
            && self
                .map_rect()
                .is_some_and(|r| r.contains(px.max(0.0) as u32, py.max(0.0) as u32))
        {
            return Some(true);
        }
        match kind {
            2 => {
                self.down = true;
                self.last = (px, py);
                Some(true)
            }
            3 => {
                self.down = false;
                Some(true)
            }
            1 => {
                if !self.down {
                    return Some(false);
                }
                let (dx, dy) = (f64::from(px - self.last.0), f64::from(py - self.last.1));
                self.last = (px, py);
                self.rig.orbit_delta(-dx * ORBIT_RATE, dy * ORBIT_RATE);
                Some(true)
            }
            4 => {
                // ortho zooms via half-extents (rig.zoom); persp has no
                // half-extents -- wheel there drives the eye distance (the
                // E201 native law: wheel = dist scaling). Before this, the
                // gltf mount fit set Persp and the wheel was a silent no-op
                // on every fitted gltf page (E201/E811/E813 user catch).
                if matches!(self.mode, Proj::Ortho) {
                    self.zoom *= (f64::from(wheel) * ZOOM_RATE).exp();
                    self.rig.zoom = self.zoom.max(0.001);
                } else {
                    self.rig.dist = (self.rig.dist / (f64::from(wheel) * ZOOM_RATE).exp()).max(0.1);
                }
                Some(true)
            }
            5 => Some(false),
            _ => None,
        }
    }

    // ===== attach (CAPI-06) =====

    /// Build a surface from raw handles and take over the render target (attach counts complete only when configure succeeds).
    ///
    /// # Safety
    /// See `Swapchain::from_raw_handles` (host window liveness duty).
    pub unsafe fn attach_raw(
        &mut self,
        display: Option<raw_window_handle::RawDisplayHandle>,
        window: raw_window_handle::RawWindowHandle,
    ) -> Result<(), String> {
        // build-then-swap: a failure preserves the current headless target (attach atomicity, CAPI-06)
        let sw =
            unsafe { self.backend.attach_surface(display, window) }.map_err(|e| e.to_string())?;
        self.target = TargetMode::Window(sw);
        Ok(())
    }

    // ===== Render/readback (CAPI-04) =====

    /// Tree-aware visibility: entity itself OR any ancestor is in the hidden set.
    /// O(depth ≤ MAX_TREE_DEPTH=16); returns true if entity or any ancestor is hidden.
    fn is_hidden(&self, entity: visiaengine_core::EntityId) -> bool {
        let mut cur = Some(entity);
        let mut depth = 0usize;
        while let Some(c) = cur {
            if self.hidden.contains(&enc_entity(c)) {
                return true;
            }
            cur = match self.scene.get_parent(c) {
                Ok(Some(p)) => Some(p),
                _ => None,
            };
            depth += 1;
            if depth > 16 {
                break;
            }
        }
        false
    }

    /// Mount a layer by kind, replacing that kind's previous occupant. Same-kind
    /// replace is the behaviour the single-slot engine already had (and what the
    /// clause now says); GPU products of a replaced layer are dropped un-freed,
    /// exactly as before (fix belongs to the day a re-mount flow is demanded).
    fn mount_layer(&mut self, layer: TileLayer) {
        match self.layers.iter().position(|l| l.kind == layer.kind) {
            Some(at) => self.layers[at] = layer,
            None => self.layers.push(layer),
        }
    }

    /// Per-kind addressing for the two view entries (no index is exposed).
    fn layer_of_mut(&mut self, kind: LayerKind) -> Option<&mut TileLayer> {
        self.layers.iter_mut().find(|l| l.kind == kind)
    }

    pub fn render(&mut self) -> Result<(), String> {
        self.advance_fly();
        let [cr, cg, cb] = self.clear.unwrap_or([0.05, 0.07, 0.10]);
        let mut commands = vec![DrawCommand::ClearColor { rgba: [cr, cg, cb, 1.0] }];
        for it in &self.items {
            if self.is_hidden(it.entity) {
                continue; // CAPI-13 + CORE-18 tree visibility
            }
            let eff = self.scene.effective_offset(it.entity);
            commands.push(DrawCommand::DrawMesh {
                mesh: it.mesh,
                material: it.material,
                origin: add_origin(it.origin, eff),
                transform: it.transform,
            });
        }
        if self.glyphs.take_dirty() {
            let g = GLYPH_ATLAS_PX;
            self.backend
                .set_glyph_atlas(self.glyphs.pixels(), g, g)
                .map_err(|e| format!("atlas: {e}"))?;
        }
        // P1 tile layers: lazy GPU upload (once) + per-frame draw commands.
        // Stack order == mount order; `mixed` gates the basemap underlay.
        let mixed = self.layers.iter().any(|l| l.kind == LayerKind::Raster)
            && self.layers.iter().any(|l| l.kind == LayerKind::Vector);
        for layer in self.layers.iter_mut() {
            // N1.4 (CAPI-35): HTTP layers advance their pump each frame
            // (budget 4 tiles/frame = frame-budget guard, gap-analysis C4).
            // FileSource layers are fully Done via mount ensure() → no-op.
            if layer.http {
                let (done, failed) = layer.set.pump(4);
                if done + failed > 0 {
                    let _ = (done, failed); // accounting only; failures retry via set_tile_view re-arm
                }
                // Rebuild batch when the Done set grew (new geometry arrived).
                let done_ids: Vec<visiaengine_io_tiles::TileId> = layer
                    .ids
                    .iter()
                    .filter(|id| layer.set.state(id) == visiaengine_io_tiles::TileState::Done)
                    .copied()
                    .collect();
                let done_now = done_ids.len();
                if done_now > 0 && layer.uploaded.is_none() {
                    rebuild_tile_batch(layer, done_ids);
                }
                // Review fix #3: an LRU-evicted Done tile renders as a hole —
                // when the state map disagrees with decode availability,
                // re-arm the evicted tiles (they return to Unloaded and the
                // pump re-fetches them next frames).
                let evicted: Vec<visiaengine_io_tiles::TileId> = layer
                    .ids
                    .iter()
                    .filter(|id| {
                        layer.set.state(id) == visiaengine_io_tiles::TileState::Done
                            && !layer.set.decoded_cached(id)
                    })
                    .copied()
                    .collect();
                if !evicted.is_empty() {
                    layer.set.evict_done(&evicted);
                }
            }
            // R band (CAPI-39): raster layers upload per-tile textures +
            // textured quad meshes once, then draw per-tile (texture bind is
            // per-mesh in this engine's MaterialDesc, hence one mesh+material
            // pair per tile).
            if !layer.raster_quads.is_empty() && layer.raster_gpu.is_empty() {
                for (ti, quad) in layer.raster_quads.iter().enumerate() {
                    let Some(id) = layer.ids.get(ti).copied() else {
                        continue;
                    };
                    let Some(visiaengine_io_tiles::scheduler::TilePayload::Raster {
                        rgba,
                        width,
                        height,
                    }) = layer.set.decoded_raster(&id)
                    else {
                        continue;
                    };
                    let (rgba, width, height) = (rgba.clone(), *width, *height);
                    let tex = self
                        .backend
                        .upload_texture(&visiaengine_render::TextureDesc {
                            rgba: &rgba,
                            width,
                            height,
                        })
                        .map_err(|e| format!("raster tex: {e:?}"))?;
                    let mut pos = Vec::with_capacity(4);
                    let mut uv = Vec::with_capacity(4);
                    for (p, t) in quad {
                        pos.push(*p);
                        uv.push(*t);
                    }
                    let normals = vec![[0.0f32, 0.0, 1.0]; 4];
                    let mesh = self
                        .backend
                        .create_mesh(&MeshDesc {
                            uv: &uv,
                            positions: &pos,
                            normals: &normals,
                            indices: &[0, 1, 2, 0, 2, 3],
                        })
                        .map_err(|e| format!("raster quad: {e:?}"))?;
                    let material = self
                        .backend
                        .create_material_desc(&MaterialDesc {
                            base_color: [1.0, 1.0, 1.0, 1.0],
                            texture: Some(tex),
                            repeat: [1.0, 1.0],
                            specular: 0.0,
                            roughness: 1.0,
                            metallic: 0.0,
                        })
                        .map_err(|e| format!("raster mat: {e:?}"))?;
                    layer.raster_gpu.push((tex, mesh, material));
                }
            }
            let underlay = if mixed { RASTER_UNDERLAY_Z } else { 0.0 };
            for (_, mesh, material) in &layer.raster_gpu {
                commands.push(DrawCommand::DrawMesh {
                    mesh: *mesh,
                    material: *material,
                    origin: [layer.center[0], layer.center[1], layer.center[2] + underlay],
                    transform: IDENTITY,
                });
            }
            if layer.uploaded.is_none() && !layer.wpos.is_empty() {
                let normals = vec![[0.0f32, 0.0, 1.0]; layer.wpos.len()];
                let mesh = self
                    .backend
                    .create_mesh(&MeshDesc {
                        uv: &[],
                        positions: &layer.wpos,
                        normals: &normals,
                        indices: &layer.widx,
                    })
                    .map_err(|e| format!("tile mesh: {e:?}"))?;
                let material = self
                    .backend
                    .create_material([0.16, 0.38, 0.62, 1.0])
                    .map_err(|e| format!("tile material: {e:?}"))?;
                let st = self
                    .backend
                    .create_strokes(&StrokeTableDesc {
                        data: &layer.strokes,
                    })
                    .map_err(|e| format!("tile strokes: {e:?}"))?;
                let pt = self
                    .backend
                    .create_points(&PointTableDesc {
                        data: &layer.points,
                    })
                    .map_err(|e| format!("tile points: {e:?}"))?;
                layer.uploaded = Some((mesh, material, st, pt));
            }
            if let Some((mesh, material, st, pt)) = layer.uploaded {
                commands.push(DrawCommand::DrawMesh {
                    mesh,
                    material,
                    origin: layer.center,
                    transform: IDENTITY,
                });
                commands.push(DrawCommand::DrawStrokes {
                    table: st,
                    origin: layer.center,
                    transform: IDENTITY,
                });
                commands.push(DrawCommand::DrawPoints {
                    table: pt,
                    origin: layer.center,
                    transform: IDENTITY,
                });
            }
        }
        commands.append(&mut self.extra_cmds.clone());
        let (hw, hh) = self.half_extents();
        // N2.1 (REND-41): projection matrices — discrete mode OR lerped morph.
        // Morph computes BOTH and interpolates element-wise with smoothstep;
        // view_rot/eye/commands pass through untouched (projection-only law).
        let persp = self
            .rig
            .perspective(
                self.rig.fov_y as f32,
                self.w as f32 / self.h as f32,
                0.1,
                1000.0,
            )
            .ok_or("persp degenerate")?;
        let ortho = self
            .rig
            .ortho_frame(
                hw as f32,
                self.w as f32,
                self.h as f32,
                0.1,
                (hw * 2.0).max(1000.0) as f32,
            )
            .ok_or("ortho degenerate")?;
        let (proj, camera, px_world_scale) = match self.morph {
            Some(MorphT(t)) => {
                // Shared pure fns (REND-41; render::morph — single source of
                // truth, example E304 reuses the exact same interpolation).
                let lerped = visiaengine_render::morph_proj(&ortho, &persp, t);
                // Camera enum consumers: backend feeds ONE view-block uniform
                // kind; mid-morph keeps the perspective kind (proj matrix
                // carries the true interpolation — split documented in clause).
                let cam = if t <= 0.5 {
                    Camera::ortho(hw as f32, hh as f32, 0.1, (hw * 2.0).max(1000.0) as f32)
                } else {
                    Camera::perspective(
                        self.rig.fov_y as f32,
                        self.w as f32 / self.h as f32,
                        0.1,
                        1000.0,
                    )
                };
                let ps_ortho = 2.0 * hw as f32 / self.w as f32; // ortho exact scale
                (lerped, cam, visiaengine_render::morph_px_scale(ps_ortho, t))
            }
            None => {
                let (p, c, ps) = match self.mode {
                    Proj::Persp => {
                        // REND-29 (the 4de not-installed-1 debt paid): the
                        // screen-expansion families (labels/points/strokes)
                        // scale by px_world_scale -- under perspective it is
                        // the target-depth value 2*dist*tan(fov/2)/H, not a
                        // placeholder 1.0 (1.0 made a 10px label 10 world
                        // meters tall).
                        let d = [
                            self.rig.eye()[0] - self.rig.target[0],
                            self.rig.eye()[1] - self.rig.target[1],
                            self.rig.eye()[2] - self.rig.target[2],
                        ];
                        let dist_t = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                        let tan_half = (self.rig.fov_y * 0.5).tan().max(1e-3);
                        (
                            persp,
                            Camera::perspective(
                                self.rig.fov_y as f32,
                                self.w as f32 / self.h as f32,
                                0.1,
                                1000.0,
                            ),
                            (2.0 * dist_t * tan_half / f64::from(self.h.max(1))) as f32,
                        )
                    }
                    Proj::Ortho => (
                        ortho,
                        Camera::ortho(hw as f32, hh as f32, 0.1, (hw * 2.0).max(1000.0) as f32),
                        2.0 * hw as f32 / self.w as f32,
                    ),
                };
                (p, c, ps)
            }
        };
        let eye = self.rig.eye();
        let frame = Frame {
            viewport: Viewport::new(self.w, self.h, 1.0),
            camera,
            view_rot: self.rig.view_rotation(),
            eye,
            proj,
            // ortho: world width 2*hw spans W px -> px_world_scale = 2*hw/W (REND-29 exact path;
            // mid-morph = linear interpolation of the two ends, REND-41)
            px_world_scale,
            shadow: self.shadow,
            clip: self.clip,
            edl: self.edl,
            post: self.post.clone(),
            commands,
        };
        // 5b second wave: map = Some -> main full-face + minimap corner window, dual projection; None -> the old single-frame path (canary construction fidelity)
        if let (Some(r), Some(mrig)) = (self.map_rect(), self.map_rig()) {
            let (rw, rh) = (r.width.max(1), r.height.max(1));
            let proj = mrig
                .ortho_frame(
                    mrig.zoom as f32,
                    rw as f32,
                    rh as f32,
                    0.1,
                    (mrig.zoom * 2.0).max(1000.0) as f32,
                )
                .ok_or("map ortho degenerate")?;
            let map_frame = Frame {
                viewport: Viewport::new(rw, rh, 1.0),
                camera: Camera::ortho(
                    mrig.zoom as f32,
                    (mrig.zoom * f64::from(rh) / f64::from(rw.max(1))) as f32,
                    0.1,
                    (mrig.zoom * 2.0).max(1000.0) as f32,
                ),
                view_rot: mrig.view_rotation(),
                eye: mrig.eye(),
                proj,
                px_world_scale: (2.0 * mrig.zoom / f64::from(rw)) as f32,
                shadow: self.shadow,
                clip: self.clip,
                edl: self.edl,
                post: self.post.clone(),
                commands: frame.commands.clone(),
            };
            let passes = [(frame, ViewportRect::full(self.w, self.h)), (map_frame, r)];
            return match &mut self.target {
                TargetMode::Headless => {
                    let img = self
                        .backend
                        .render_to_pixels_rects(&passes, MultiClearPolicy::FirstClearRestLoad)
                        .ok_or("render failed")?;
                    self.frame_cache = Some(img.rgba);
                    Ok(())
                }
                TargetMode::Window(sw) => self
                    .backend
                    .render_swapchain_multi(&passes, sw)
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
            };
        }
        match &mut self.target {
            TargetMode::Headless => {
                let img = self
                    .backend
                    .render_to_pixels(&frame)
                    .ok_or("render failed")?;
                self.frame_cache = Some(img.rgba);
                Ok(())
            }
            TargetMode::Window(sw) => self
                .backend
                .render_swapchain(&frame, sw)
                .map(|_| ())
                .map_err(|e| e.to_string()),
        }
    }

    /// readback buffer requirement (unrendered = 0, callers attribute -2/-5 accordingly).
    #[must_use]
    pub fn frame_bytes(&self) -> Option<usize> {
        self.frame_cache.as_ref().map(Vec::len)
    }

    pub fn copy_frame(&self, buf: &mut [u8]) {
        if let Some(f) = &self.frame_cache {
            buf[..f.len()].copy_from_slice(f);
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.w = w;
        self.h = h;
        self.backend.resize(Viewport::new(w, h, 1.0));
        if let TargetMode::Window(sw) = &mut self.target {
            sw.resize(w, h); // reconfigured at the next frame's configure (CAPI-06 resize path)
        }
        self.frame_cache = None;
    }

    fn half_extents(&self) -> (f64, f64) {
        let aspect = f64::from(self.w) / f64::from(self.h);
        (self.zoom, self.zoom / aspect.max(1e-6))
    }

    pub fn entity_count(&self) -> u32 {
        let mut ids: Vec<EntityId> = self.items.iter().map(|i| i.entity).collect();
        ids.dedup();
        ids.len() as u32
    }

    pub fn entity_at(&self, index: u32) -> Option<EntityId> {
        let mut seen: Vec<EntityId> = Vec::new();
        for it in &self.items {
            if !seen.contains(&it.entity) {
                seen.push(it.entity);
            }
        }
        seen.into_iter().nth(index as usize)
    }

    /// CAPI-39 (R band): mount a RASTER tile directory (PNG tiles,
    /// FileSource::raster). Same discovery scan as load_mvt_dir but for
    /// .png; decodes via the scheduler raster lane; builds one flat ground
    /// quad per tile at z=0 (world-frame 3857, vertices baked to the batch
    /// center — PIT-8 safe domain), each textured with its tile image.
    /// Camera fit mirrors load_mvt_dir (ortho top-down). Returns the tile
    /// count. claim discipline (N1.2): flat raster basemap — NOT terrain,
    /// no drape, no LOD.
    /// Single raster-tile bytes injection (the wasm face; the C dir scan
    /// wraps the same quad-baking tail). Kind = Raster.
    pub fn load_raster_bytes(
        &mut self,
        z: u8,
        x: u32,
        y: u32,
        bytes: &[u8],
    ) -> Result<u64, String> {
        let id = visiaengine_io_tiles::TileId::new(z, x, y)
            .ok_or_else(|| format!("tile id out of range: {z}/{x}/{y}"))?;
        let source = visiaengine_io_tiles::MemSource::new();
        source.insert(z, x, y, bytes.to_vec());
        let mut set = visiaengine_io_tiles::TileSet::new(Box::new(source))
            .map_err(|e| format!("tiles: {e:?}"))?;
        set.ensure(&[id]).map_err(|e| format!("tiles ensure: {e:?}"))?;
        if set.decoded_raster(&id).is_none() {
            return Err(format!("raster decode failed: {z}/{x}/{y}"));
        }
        // Raster quad tail, single tile (mirrors load_raster_dir's batch).
        let (min_x, min_y, max_x, max_y) = union_bbox(&[id]);
        let center = [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0, 0.0];
        let tile_w = visiaengine_io_tiles::WORLD_EXTENT / f64::from(1u32 << z);
        let mut quads: Vec<[([f32; 3], [f32; 2]); 4]> = Vec::new();
        if set.decoded_raster(&id).is_some() {
            let (x0, y1) = (
                -visiaengine_io_tiles::WORLD_EXTENT / 2.0 + f64::from(id.x) * tile_w,
                visiaengine_io_tiles::WORLD_EXTENT / 2.0 - f64::from(id.y) * tile_w,
            );
            let (x1, y0) = (x0 + tile_w, y1 - tile_w);
            quads.push([
                ([((x0 - center[0]) as f32), ((y1 - center[1]) as f32), 0.0], [0.0f32, 0.0]),
                ([((x1 - center[0]) as f32), ((y1 - center[1]) as f32), 0.0], [1.0, 0.0]),
                ([((x1 - center[0]) as f32), ((y0 - center[1]) as f32), 0.0], [1.0, 1.0]),
                ([((x0 - center[0]) as f32), ((y0 - center[1]) as f32), 0.0], [0.0, 1.0]),
            ]);
        }
        let count = quads.len() as u64;
        if self.layers.is_empty() {
            let radius = ((max_x - min_x) / 2.0).max((max_y - min_y) / 2.0) * FIT_PAD;
            self.mode = Proj::Ortho;
            self.zoom = radius.max(10.0);
            self.rig = CameraRig::look_at(
                [center[0], center[1], radius.max(10.0)],
                center,
                [0.0, 1.0, 0.0],
            );
        }
        self.mount_layer(TileLayer {
            kind: LayerKind::Raster,
            set,
            ids: vec![id],
            http: false,
            center,
            wpos: Vec::new(),
            widx: Vec::new(),
            strokes: Vec::new(),
            points: Vec::new(),
            raster_quads: quads,
            raster_gpu: Vec::new(),
            uploaded: None,
        });
        Ok(count)
    }

    pub fn load_raster_dir(&mut self, path: &str, z: u8) -> Result<u64, String> {
        let source = visiaengine_io_tiles::FileSource::raster(path);
        let mut set = visiaengine_io_tiles::TileSet::new(Box::new(source))
            .map_err(|e| format!("tiles: {e:?}"))?;
        let mut ids: Vec<visiaengine_io_tiles::TileId> = Vec::new();
        let zdir = std::path::Path::new(path).join(z.to_string());
        let entries =
            std::fs::read_dir(&zdir).map_err(|e| format!("read_dir {}: {e}", zdir.display()))?;
        for xentry in entries.flatten() {
            let xname = xentry.file_name().to_string_lossy().into_owned();
            let Ok(x) = xname.parse::<u32>() else {
                continue;
            };
            let xdir = zdir.join(&xname);
            let Ok(files) = std::fs::read_dir(&xdir) else {
                continue;
            };
            for f in files.flatten() {
                let fname = f.file_name().to_string_lossy().into_owned();
                if let Some(ystr) = fname.strip_suffix(".png")
                    && let Ok(y) = ystr.parse::<u32>()
                    && let Some(id) = visiaengine_io_tiles::TileId::new(z, x, y)
                {
                    ids.push(id);
                }
            }
        }
        if ids.is_empty() {
            return Err(format!("no .png tiles under {}/{z}", zdir.display()));
        }
        ids.sort_by_key(|i| (i.y, i.x));
        set.ensure(&ids)
            .map_err(|e| format!("tiles ensure: {e:?}"))?;
        // Raster batch: per-tile quad (two triangles, uv 0..1), vertices baked
        // to the union-bbox center. Tile world size from slippy math.
        let (min_x, min_y, max_x, max_y) = union_bbox(&ids);
        let center = [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0, 0.0];
        let tile_w = visiaengine_io_tiles::WORLD_EXTENT / f64::from(1u32 << z);
        let mut quads: Vec<[([f32; 3], [f32; 2]); 4]> = Vec::new();
        for id in &ids {
            let payload = set.decoded_raster(id);
            if payload.is_none() {
                continue;
            }
            let (x0, y1) = (
                -visiaengine_io_tiles::WORLD_EXTENT / 2.0 + f64::from(id.x) * tile_w,
                visiaengine_io_tiles::WORLD_EXTENT / 2.0 - f64::from(id.y) * tile_w,
            );
            let (x1, y0) = (x0 + tile_w, y1 - tile_w);
            let q = [
                (
                    [((x0 - center[0]) as f32), ((y1 - center[1]) as f32), 0.0],
                    [0.0f32, 0.0],
                ),
                (
                    [((x1 - center[0]) as f32), ((y1 - center[1]) as f32), 0.0],
                    [1.0, 0.0],
                ),
                (
                    [((x1 - center[0]) as f32), ((y0 - center[1]) as f32), 0.0],
                    [1.0, 1.0],
                ),
                (
                    [((x0 - center[0]) as f32), ((y0 - center[1]) as f32), 0.0],
                    [0.0, 1.0],
                ),
            ];
            quads.push(q);
        }
        let count = quads.len() as u64;
        // Adjudication 4 = A: the FIRST mounted layer frames the view; a later
        // mount must not steal a camera the host already positioned.
        if self.layers.is_empty() {
            let radius = ((max_x - min_x) / 2.0).max((max_y - min_y) / 2.0) * FIT_PAD;
            self.mode = Proj::Ortho;
            self.zoom = radius.max(10.0);
            self.rig = CameraRig::look_at(
                [center[0], center[1], radius.max(10.0)],
                center,
                [0.0, 1.0, 0.0],
            );
        }
        // Reuse the TileLayer shell for camera-fit + view bookkeeping; the
        // raster quads ride the layer's set/ids (their decode lane is raster,
        // render-loop branch keys on the layer kind below).
        self.mount_layer(TileLayer {
            kind: LayerKind::Raster,
            set,
            ids,
            http: false,
            center,
            wpos: Vec::new(),
            widx: Vec::new(),
            strokes: Vec::new(),
            points: Vec::new(),
            raster_quads: quads,
            raster_gpu: Vec::new(),
            uploaded: None,
        });
        Ok(count)
    }

    /// CAPI-40 (R band): feed a 3857 bbox to the raster layer's view (same
    /// enumerate + camera-fit semantics as set_tile_view; raster layer keeps
    /// mount-time quads — re-view re-centers the camera only, v1 honest scope).
    pub fn set_raster_view(
        &mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<u64, String> {
        let Some(layer) = self.layer_of_mut(LayerKind::Raster) else {
            return Err("raster layer not mounted (load_raster_dir first)".into());
        };
        if !(min_x.is_finite()
            && min_y.is_finite()
            && max_x.is_finite()
            && max_y.is_finite()
            && min_x < max_x
            && min_y < max_y)
        {
            return Err("raster view domain (finite, min<max)".into());
        }
        let z_of = layer.ids.first().map(|i| i.z).unwrap_or(0);
        let ids = visiaengine_io_tiles::TileSet::visible((min_x, min_y, max_x, max_y), z_of);
        // Keep only tiles that actually exist in the mounted set (dir scan
        // = discovery; invisible/unmounted tiles are NOT fabricated).
        let mounted: std::collections::HashSet<_> = layer.ids.iter().copied().collect();
        let visible: Vec<_> = ids.into_iter().filter(|i| mounted.contains(i)).collect();
        let n = visible.len() as u64;
        // Camera recenters on the requested bbox (v1: quads unchanged).
        let cx = (min_x + max_x) / 2.0;
        let cy = (min_y + max_y) / 2.0;
        let radius = ((max_x - min_x) / 2.0).max((max_y - min_y) / 2.0) * FIT_PAD;
        self.mode = Proj::Ortho;
        self.zoom = radius.max(10.0);
        self.rig = CameraRig::look_at([cx, cy, radius.max(10.0)], [cx, cy, 0.0], [0.0, 1.0, 0.0]);
        Ok(n)
    }

    /// CAPI-42/43 (V2.2): node-local transform write/read on a mounted glTF
    /// entity. Column-major f64 x16 (GLTF-08 shape, same as GltfEntity.local).
    /// The parent chain composes at draw time (world = P·local chain walk).
    /// Finite gate on all 16; unknown entity = Err.
    pub fn set_node_transform(&mut self, entity: u64, m: [[f64; 4]; 4]) -> Result<(), String> {
        for row in &m {
            for v in row {
                if !v.is_finite() {
                    return Err("node transform: non-finite element".into());
                }
            }
        }
        let id = dec_entity(entity);
        let item = self
            .items
            .iter_mut()
            .find(|it| it.entity == id)
            .ok_or_else(|| format!("unknown entity {entity:#018x}"))?;
        item.transform = m;
        Ok(())
    }

    pub fn get_node_transform(&self, entity: u64) -> Result<[[f64; 4]; 4], String> {
        let id = dec_entity(entity);
        self.items
            .iter()
            .find(|it| it.entity == id)
            .map(|it| it.transform)
            .ok_or_else(|| format!("unknown entity {entity:#018x}"))
    }

    /// CAPI-41 (I band): decode an .hdr (RGBE equirect) file and install its
    /// SH-9 irradiance projection as the global environment. Replaces the
    /// legacy constant ambient (WGPU-39); demo-grade claim — PMREM/LUT =
    /// separate ticket. Returns 0 on success.
    pub fn load_env_hdr(&mut self, path: &str) -> Result<(), String> {
        let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
        self.load_env_hdr_bytes(&bytes)
    }

    /// Bytes entry (the wasm face has no filesystem; the C path form wraps this).
    pub fn load_env_hdr_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let img = visiaengine_io_hdr::decode_hdr(bytes).map_err(|e| e.to_string())?;
        let sh = visiaengine_io_hdr::project_sh9(&img, 256);
        self.env_sh = Some(sh);
        self.backend.set_env_sh(sh);
        Ok(())
    }

    /// Screen picking (reuses REND-21..24; ortho/persp dual paths; B2: point-cloud fallback when the mesh misses).
    /// Hit = EntityId. Point-cloud path same scheme: hidden filter + clip keeps negative-side exclusion (REND-38).
    /// Pick with the world-space hit point (the E203 measuring family).
    /// Same routing as `pick` (minimap region first), point = surface hit.
    pub fn pick_hit(&self, px: f32, py: f32) -> Option<(EntityId, [f64; 3])> {
        if let (Some(r), Some(mrig)) = (self.map_rect(), self.map_rig())
            && let Some((lx, ly)) = r.local(px.max(0.0) as u32, py.max(0.0) as u32)
        {
            if let Some((id, pt)) = self.pick_geo(&mrig, true, lx, ly, r.width, r.height) {
                return Some((id, pt));
            }
            return self.pick_points_in(&mrig, lx, ly, r.width, r.height).map(|id| (id, [0.0; 3]));
        }
        let ortho = matches!(self.mode, Proj::Ortho);
        if let Some((id, pt)) = self.pick_geo(&self.rig, ortho, px, py, self.w, self.h) {
            return Some((id, pt));
        }
        self.pick_points_in(&self.rig, px, py, self.w, self.h)
            .map(|id| (id, [0.0; 3]))
    }

    pub fn pick(&self, px: f32, py: f32) -> Option<EntityId> {
        // 5b second wave routing: minimap region first (visual top = input top); everything else = main rig original semantics
        if let (Some(r), Some(mrig)) = (self.map_rect(), self.map_rig())
            && let Some((lx, ly)) = r.local(px.max(0.0) as u32, py.max(0.0) as u32)
        {
            if let Some((id, _)) = self.pick_geo(&mrig, true, lx, ly, r.width, r.height) {
                return Some(id);
            }
            return self.pick_points_in(&mrig, lx, ly, r.width, r.height);
        }
        let ortho = matches!(self.mode, Proj::Ortho);
        if let Some((id, _)) = self.pick_geo(&self.rig, ortho, px, py, self.w, self.h) {
            return Some(id);
        }
        self.pick_points_in(&self.rig, px, py, self.w, self.h)
    }

    /// B2 point-cloud picking (REND-38 predicate + CAPI-13 hidden filter + CAPI-20 clip negative-side exclusion).
    /// Reached only when the mesh missed = REND-39 canary construction guarantee (mesh hit path untouched).
    fn pick_points_in(
        &self,
        rig: &CameraRig,
        px: f32,
        py: f32,
        w: u32,
        h: u32,
    ) -> Option<EntityId> {
        if self.point_clouds.is_empty() {
            return None;
        }
        let ortho = matches!(self.mode, Proj::Ortho);
        let (hw, _hh) = self.half_extents();
        let proj = if ortho {
            rig.ortho_frame(
                hw as f32,
                w as f32,
                h.max(1) as f32,
                0.1,
                (hw * 2.0).max(1000.0) as f32,
            )?
        } else {
            rig.perspective(rig.fov_y as f32, w as f32 / h.max(1) as f32, 0.1, 1000.0)?
        };
        let mvp = compose_mvp(
            &proj,
            &rig.view_rotation(),
            &rig.eye(),
            &[0.0; 3],
            &IDENTITY,
        );
        let cands: Vec<PointCloudCandidate<'_>> = self
            .point_clouds
            .values()
            .filter(|pc| !self.is_hidden(pc.entity))
            .map(|pc| PointCloudCandidate {
                entity: pc.entity,
                origin: add_origin(pc.origin, self.scene.effective_offset(pc.entity)),
                transform: &pc.transform,
                positions: &pc.positions,
            })
            .collect();
        let mut hit = pick_points(&mvp, w as f32, h as f32, px, py, PICK_RADIUS_PX, &cands);
        // clip keeps negative-side exclusion (REND-38, point-level keeps carry the same semantics as the mesh path): if the hit point is clipped,
        // re-search (including the next-nearest point of the same cloud) -- the predicate carries point_index, world position = M*local + origin.
        if let (Some(c), Some(cl)) = (&self.clip, hit.map(|h| (h.entity, h.point_index))) {
            let pc = self.point_clouds.get(&enc_entity(cl.0));
            if let Some(pc) = pc {
                let m = pc.transform;
                let lp = pc.positions[cl.1];
                let world = [
                    m[0][0] * lp[0] as f64
                        + m[1][0] * lp[1] as f64
                        + m[2][0] * lp[2] as f64
                        + m[3][0]
                        + pc.origin[0],
                    m[0][1] * lp[0] as f64
                        + m[1][1] * lp[1] as f64
                        + m[2][1] * lp[2] as f64
                        + m[3][1]
                        + pc.origin[1],
                    m[0][2] * lp[0] as f64
                        + m[1][2] * lp[1] as f64
                        + m[2][2] * lp[2] as f64
                        + m[3][2]
                        + pc.origin[2],
                ];
                if !c.keeps(world) {
                    // Whole-cloud candidate screen: nearest point clipped → drop the
                    // cloud entirely and retry (same shape as mesh skip-retry loop).
                    let mut cands2 = cands;
                    cands2.retain(|c2| c2.entity != cl.0);
                    hit = pick_points(&mvp, w as f32, h as f32, px, py, PICK_RADIUS_PX, &cands2);
                }
            }
        }
        hit.map(|h| h.entity)
    }

    /// Pick core (rig/projection form/local px fully parameterized: shared by the main viewport and the minimap; returns (shape, hit point xy)).
    fn pick_geo(
        &self,
        rig: &CameraRig,
        ortho: bool,
        px: f32,
        py: f32,
        w: u32,
        h: u32,
    ) -> Option<(EntityId, [f64; 3])> {
        let ray = if ortho {
            screen_to_ray_ortho(rig, px, py, w as f32, h.max(1) as f32)?
        } else {
            screen_to_ray_persp(rig, px, py, w as f32, h.max(1) as f32)?
        };
        let mut skip: Vec<EntityId> = Vec::new();
        loop {
            let cands: Vec<MeshCandidate> = self
                .items
                .iter()
                .filter(|it| !self.is_hidden(it.entity) && !skip.contains(&it.entity))
                .map(|it| MeshCandidate {
                    entity: it.entity,
                    positions: &it.positions,
                    indices: &it.indices,
                    world: &it.transform,
                    // same composition the render loop bakes (engine.rs:789):
                    // T(origin + group_offset) * transform -- picking must see
                    // what rendering shows (the E401 review-round catch, group leg)
                    origin: add_origin(it.origin, self.scene.effective_offset(it.entity)),
                })
                .collect();
            let hit = pick_meshes(ray, &cands)?;
            if let Some(c) = &self.clip {
                let p = hit.point;
                if !c.keeps([p.x, p.y, p.z]) {
                    skip.push(hit.entity);
                    continue;
                }
            }
            return Some((hit.entity, [hit.point.x, hit.point.y, hit.point.z]));
        }
    }

    /// CAPI-13: visibility toggle (strict existence-domain validation; render/pick filtered, enumeration domain unchanged).
    /// CAPI-23: takeoff/reroute. Pose six components + fov, **near/far always the current rig values** ([Momus-A1]
    /// depth domain does not fly); `dur_ms=0` = teleport form (lands immediately, not an error value); re-entry mid-flight = reroute
    /// (from = current pose snapshot, t reset -- first frame continuous to avoid a jump [Momus-A3]).
    /// Domain rejection: non-finite / dist<=0 / zoom<=0 / fov outside (0,pi).
    #[allow(clippy::too_many_arguments)]
    pub fn fly_to(
        &mut self,
        target: [f64; 3],
        yaw: f64,
        pitch: f64,
        dist: f64,
        zoom: f64,
        fov: f64,
        dur_ms: u64,
    ) -> Result<(), String> {
        let ok = target.iter().all(|v| v.is_finite())
            && [yaw, pitch, dist, zoom, fov].iter().all(|v| v.is_finite())
            && dist > 0.0
            && zoom > 0.0
            && fov > 0.0
            && fov < std::f64::consts::PI;
        if !ok {
            return Err("fly_to: pose domain (finite/dist>0/zoom>0/fov in (0,pi))".into());
        }
        let to = CameraRig {
            target,
            yaw,
            pitch,
            dist,
            zoom,
            fov_y: fov,
            near: self.rig.near,
            far: self.rig.far,
        };
        self.fly_rig(to, dur_ms);
        Ok(())
    }

    /// Thruster landing core (shared road of fly_to/navigate_click; domain checks done by the caller).
    fn fly_rig(&mut self, to: CameraRig, dur_ms: u64) {
        if dur_ms == 0 {
            self.rig = to;
            self.zoom = to.zoom;
            self.fly = None;
            return;
        }
        self.fly = Some(Fly {
            from: self.rig,
            to,
            start_ms: now_ms(),
            dur_s: dur_ms as f64 / 1000.0,
        });
    }

    /// CAPI-24 semantics source: not flying = done (merged idle/done single primary domain).
    #[must_use]
    pub fn fly_state_done(&self) -> bool {
        self.fly.is_none()
    }

    /// Progress while flying [0,1) (done/idle = None -- the engine form of the out-not-written path).
    #[must_use]
    pub fn fly_progress(&self) -> Option<f64> {
        self.fly
            .as_ref()
            .map(|f| ((now_ms() - f.start_ms) / 1000.0 / f.dur_s).clamp(0.0, 1.0))
    }

    /// Per-frame advance (first line of render): t>=1 drops straight to the to original value (endpoint exact, same scheme as REND-16);
    /// otherwise fly_sample lands on the rig (zoom synced -- the ortho family consumption face).
    fn advance_fly(&mut self) {
        let Some(f) = &self.fly else { return };
        let t = (now_ms() - f.start_ms) / 1000.0 / f.dur_s;
        let rig = if t >= 1.0 {
            f.to
        } else {
            CameraRig::fly_sample(&f.from, &f.to, t, visiaengine_render::Easing::CubicInOut)
        };
        self.rig = rig;
        self.zoom = rig.zoom;
        if t >= 1.0 {
            self.fly = None;
        }
    }

    /// CAPI-20: world plane coefficient group [nx,ny,nz,d] (normal points to the kept side, `dot(n,P)+d>=0` keeps,
    /// AND combination). n=0 = the only clear form; degenerate/non-finite/over-4 rejected (ClipSetup::new same-source gate).
    /// N2.1 test seam (rs-only): discrete projection switch (morph = None).
    /// Same shape family as set_morph_time(None) + mode writes; kept public
    /// for tests/examples that need the discrete reference path.
    pub fn set_projection(&mut self, mode: Proj) {
        self.mode = mode;
        self.morph = None;
    }

    /// Current discrete mode (true = persp). The morph envelope outranks it when active.
    #[must_use]
    pub fn projection_is_persp(&self) -> bool {
        matches!(self.mode, Proj::Persp)
    }

    /// N2.1 test seam (rs-only): full RGBA bytes of the last rendered frame
    /// (None = nothing rendered yet). Byte-comparison consumer for the
    /// endpoint canaries.
    #[must_use]
    pub fn frame_rgba(&self) -> Option<&[u8]> {
        self.frame_cache.as_deref()
    }

    /// N2.1 (REND-41): projection morphTime slider. t=0 → ortho bytes,
    /// t=1 → persp bytes (endpoint bitwise canaries); mid values lerp the
    /// projection matrices element-wise (smoothstep easing). Slider
    /// semantics: NaN/inf clamp to 0/1, out-of-range clamps (diverges from
    /// EdlSetup's reject — documented in the clause). None value clears the
    /// morph (returns to discrete Proj::mode).
    pub fn set_morph_time(&mut self, t: Option<f64>) {
        self.morph = t.map(|v| {
            if v.is_nan() {
                MorphT(0.0)
            } else {
                MorphT(v.clamp(0.0, 1.0))
            }
        });
    }

    /// Post-effect chain (flat triplets [kind, p1, p2, ...]): 0=bloom(strength)
    /// 1=outline(width) 2=tonemap(mode) 3=ssao(radius,intensity) 4=haze(start,full).
    /// Empty = clear. Invalid kind / non-finite param = Err (zero side effects).
    pub fn set_post_effects(&mut self, flat: &[f64]) -> Result<(), String> {
        if !flat.len().is_multiple_of(3) {
            return Err("set_post_effects: len%3 != 0".into());
        }
        let mut post = Vec::new();
        for t in flat.as_chunks::<3>().0 {
            let f = |v: f64, name: &str| -> Result<f32, String> {
                let x = v as f32;
                if x.is_finite() { Ok(x) } else { Err(format!("set_post_effects: {name} non-finite")) }
            };
            post.push(match t[0] as u32 {
                0 => visiaengine_render::PostEffect::Bloom { strength: f(t[1], "strength")? },
                1 => visiaengine_render::PostEffect::Outline { width: f(t[1], "width")? },
                2 => visiaengine_render::PostEffect::Tonemap { mode: t[1] as u32 },
                3 => visiaengine_render::PostEffect::Ssao {
                    radius: f(t[1], "radius")?,
                    intensity: f(t[2], "intensity")?,
                },
                4 => visiaengine_render::PostEffect::Haze {
                    far_start: f(t[1], "far_start")?,
                    far_full: f(t[2], "far_full")?,
                },
                _ => return Err(format!("set_post_effects: kind {} unknown", t[0])),
            });
        }
        self.post = post;
        Ok(())
    }

    /// EDL eye-dome lighting, None = off (the legacy path bit-for-bit).
    /// PCSS shadow toggle from a normalized light direction (the E501
    /// construction, engine-side): None = off (bit-identical legacy path).
    /// Non-finite / zero vector = ignored (keeps the current state).
    pub fn set_shadow(&mut self, light_dir: Option<[f64; 3]>) {
        let Some(ld) = light_dir else {
            self.shadow = None;
            return;
        };
        if !ld.iter().all(|v| v.is_finite()) {
            return;
        }
        let n = (ld[0] * ld[0] + ld[1] * ld[1] + ld[2] * ld[2]).sqrt();
        if n < 1e-6 {
            return;
        }
        let eye = [ld[0] / n * 80.0, ld[1] / n * 80.0, ld[2] / n * 80.0];
        let lrig = visiaengine_render::CameraRig::look_at(eye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let (w, h) = (self.w.max(1), self.h.max(1));
        self.shadow = Some(visiaengine_render::ShadowSetup {
            proj: lrig.ortho_frame(40.0, w as f32, h as f32, 1.0, 160.0).expect("light proj"),
            view_rot: lrig.view_rotation(),
            eye: lrig.eye(),
            light_dir: [ld[0] as f32 / n as f32, ld[1] as f32 / n as f32, ld[2] as f32 / n as f32],
            size: 2.0,
            bias: visiaengine_render::ShadowSetup::DEFAULT_BIAS,
        });
    }

    /// Host-set clear color (srgb domain rgb triple). The default deep slate
    /// stays until the host sets one; restore = set_clear_color(None).
    pub fn set_clear_color(&mut self, rgb: Option<[f64; 3]>) {
        self.clear = rgb.map(|c| [c[0] as f32, c[1] as f32, c[2] as f32]);
    }

    pub fn set_edl(&mut self, strength: Option<f64>) {
        self.edl = strength.map(|v| {
            visiaengine_render::EdlSetup::new(v as f32).expect("edl strength")
        });
    }

    pub fn set_clips(&mut self, planes: &[[f64; 4]]) -> Result<(), String> {
        if planes.len() > visiaengine_render::ClipSetup::MAX_PLANES {
            return Err("set_clips: n>MAX(4)".into());
        }
        if planes.is_empty() {
            self.clip = None;
            return Ok(());
        }
        self.clip = Some(
            visiaengine_render::ClipSetup::new(planes)
                .ok_or("set_clips: degenerate plane (zero normal / non-finite)")?,
        );
        Ok(())
    }

    /// CAPI-25: enable/disable the minimap (normalized rect 0..1, fw/fh>0, zoom>0; domain rejection with zero side effects; None = clear back to the old path).
    pub fn set_map(&mut self, rect: Option<(f32, f32, f32, f32)>, zoom: f64) -> Result<(), String> {
        let Some((fx, fy, fw, fh)) = rect else {
            self.map = None;
            return Ok(());
        };
        let ok = [fx, fy, fw, fh].iter().all(|v| v.is_finite())
            && (0.0..=1.0).contains(&fx)
            && (0.0..=1.0).contains(&fy)
            && fw > 0.0
            && fh > 0.0
            && fx + fw <= 1.0 + 1e-6
            && fy + fh <= 1.0 + 1e-6
            && zoom.is_finite()
            && zoom > 0.0;
        if !ok {
            return Err("set_map: frac/zoom domain".into());
        }
        self.map = Some(MapView {
            fx,
            fy,
            fw,
            fh,
            zoom,
        });
        Ok(())
    }

    /// The current minimap pixel rect (evaluation point of the normalized rect auto-following resize).
    #[must_use]
    pub fn map_rect(&self) -> Option<ViewportRect> {
        self.map
            .map(|m| ViewportRect::from_frac(m.fx, m.fy, m.fw, m.fh, self.w, self.h))
    }

    /// Minimap top-view rig (follows the main target; pitch of about 86 degrees near-vertical, up+Y near-orthogonal to the view direction to avoid degeneracy).
    #[must_use]
    fn map_rig(&self) -> Option<CameraRig> {
        let m = self.map?;
        let [tx, ty, _] = self.rig.target;
        Some(CameraRig::orbit(
            [tx, ty, 0.0],
            0.0,
            1.5,
            m.zoom * 2.0,
            m.zoom,
            std::f64::consts::FRAC_PI_3,
            0.1,
            m.zoom * 20.0,
        ))
    }

    /// CAPI-26: minimap click navigation (two stages: entity hit first -> ground z=0 fallback; the main rig keeps angle and distance while swapping the target).
    pub fn navigate_click(&mut self, px: f32, py: f32, dur_ms: u64) -> Result<(), String> {
        let rect = self.map_rect().ok_or("navigate_click: map not set")?;
        let (lx, ly) = rect
            .local(px.max(0.0) as u32, py.max(0.0) as u32)
            .ok_or("navigate_click: outside map rect")?;
        let mrig = self.map_rig().ok_or("navigate_click: no map")?;
        let ray = screen_to_ray_ortho(&mrig, lx, ly, rect.width as f32, rect.height.max(1) as f32)
            .ok_or("navigate_click: ray degenerate")?;
        let (tx, ty) = if let Some(hit) =
            self.pick_geo(&mrig, true, lx, ly, rect.width, rect.height)
        {
            (hit.1[0], hit.1[1])
        } else {
            let g = ray_ground_intersect(ray).ok_or("navigate_click: no ground under cursor")?;
            (g.x, g.y)
        };
        let mut to = self.rig;
        to.target = [tx, ty, 0.0];
        self.fly_rig(to, dur_ms);
        Ok(())
    }

    /// CAPI-27: main camera pose readback (host HUD / arrival assertions).
    #[must_use]
    pub fn camera_pose(&self) -> CameraRig {
        self.rig
    }

    /// CAPI-21: load/replace the label font (bytes = full TTF/OTF form; parse failure = Err, the existing font untouched).
    pub fn set_font(&mut self, bytes: &[u8]) -> Result<(), String> {
        let f =
            visiaengine_io_text::FontFace::from_bytes(bytes).map_err(|e| format!("font: {e:?}"))?;
        self.font = Some(f);
        self.glyphs = visiaengine_io_text::GlyphCache::new(); // font swap = atlas voided and rebuilt
        Ok(())
    }

    /// CAPI-20 readback (post-normalization coefficients; truncation/counting semantics live on the FFI face).
    #[must_use]
    pub fn clips(&self) -> Vec<[f64; 4]> {
        self.clip
            .map(|c| c.planes[..c.count].to_vec())
            .unwrap_or_default()
    }

    pub fn set_visible(&mut self, entity: u64, visible: bool) -> Result<(), String> {
        // B2 + S2: domain = items | point_clouds | scene-alive (group nodes, S3 tree entities).
        if !self.items.iter().any(|i| enc_entity(i.entity) == entity)
            && !self.point_clouds.contains_key(&entity)
            && !self.scene.is_alive(dec_entity(entity))
        {
            return Err(format!("unknown entity bits {entity:#018x}"));
        }
        self.hidden.retain(|h| *h != entity);
        if !visible {
            self.hidden.push(entity);
        }
        Ok(())
    }

    /// CAPI-14: visibility query (unknown shape = None, same domain as the setter).
    #[must_use]
    pub fn is_visible(&self, entity: u64) -> Option<bool> {
        if !self.scene.is_alive(dec_entity(entity)) {
            return None;
        }
        Some(!self.is_hidden(dec_entity(entity)))
    }

    /// CAPI-15: procedural mesh add (call-time copy; degenerate = zero commit, spawn rollback leaks no slots).
    // N6: +roughness/metalness pushed it over the 7-arg clippy threshold.
    #[allow(clippy::too_many_arguments)]
    pub fn add_mesh(
        &mut self,
        positions: &[[f32; 3]],
        normals: Option<&[[f32; 3]]>,
        indices: &[u32],
        base_color: [f32; 4],
        origin: [f64; 3],
        roughness: f32,
        metalness: f32,
    ) -> Result<u64, String> {
        if positions.is_empty() || indices.is_empty() {
            return Err("add_mesh: empty geometry".into());
        }
        if let Some(n) = normals
            && n.len() != positions.len()
        {
            return Err("add_mesh: normals length mismatch".into());
        }
        if let Some(mx) = indices.iter().max()
            && (*mx as usize) >= positions.len()
        {
            return Err("add_mesh: index out of range".into());
        }
        let id = self.scene.spawn();
        let mat = MaterialDesc {
            base_color,
            texture: None,
            repeat: [1.0, 1.0],
            specular: 0.0, // retired mock slot (WGPU-35)
            roughness,
            metallic: metalness,
        };
        match self.upload(id, positions, indices, normals, &mat, &[], origin, IDENTITY) {
            Ok(()) => Ok(enc_entity(id)),
            Err(e) => {
                let _ = self.scene.despawn(id); // generation +1: a failed shape is never reused and colliding out
                Err(e)
            }
        }
    }

    /// CAPI-19: point-cloud file load (path convenience entry = the file skin of the bytes form; wasm takes bytes direct).
    pub fn load_pcl(
        &mut self,
        path: &str,
        lenient: bool,
    ) -> Result<(u64, visiaengine_io_points::PclReport), String> {
        let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
        self.load_pcl_bytes(&bytes, lenient)
    }

    /// mount_pcl: one entity one DrawPoints (origin = f64 bbox center, D7 zero new math);
    /// cloud-level meta registered in pcl_meta (attr prefixed-lookup domain); failure spawn rollback follows the add_mesh scheme.
    pub fn load_pcl_bytes(
        &mut self,
        bytes: &[u8],
        lenient: bool,
    ) -> Result<(u64, visiaengine_io_points::PclReport), String> {
        let cloud = visiaengine_io_points::parse_pcl(bytes, lenient).map_err(|e| e.to_string())?;
        let id = self.scene.spawn();
        let marks: Vec<PointMark> = cloud
            .positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let c = cloud.colors.get(i).copied().unwrap_or([0.7, 0.75, 0.8]);
                PointMark::new(*p, c, cloud.radius_px)
            })
            .collect();
        let table = match self.backend.create_points(&PointTableDesc { data: &marks }) {
            Ok(t) => t,
            Err(e) => {
                let _ = self.scene.despawn(id);
                return Err(format!("create_points: {e:?}"));
            }
        };
        self.extra_cmds.push(DrawCommand::DrawPoints {
            table,
            origin: cloud.origin,
            transform: IDENTITY,
        });
        let bits = enc_entity(id);
        self.pcl_meta.insert(bits, cloud.meta);
        // B2: retain world-space positions for CPU point picking (same origin/transform as DrawPoints)
        self.point_clouds.insert(
            bits,
            PointCloudState {
                entity: id,
                origin: cloud.origin,
                transform: IDENTITY,
                positions: cloud.positions.clone(),
            },
        );
        self.emit(
            crate::ffi::VE_EVT_LOAD_PROGRESS,
            cloud.report.kept,
            cloud.report.kept,
        );
        Ok((bits, cloud.report))
    }

    /// CAPI-18: point-cloud passthrough (one entity one DrawPoints; origin = [0,0,0] host-space local --
    /// non-finite accepted as-is = host responsibility domain, only the load side mounts RepairPolicy; failure despawn rollback same as CAPI-15).
    pub fn add_points(&mut self, raw: &[crate::ffi::VePointMark]) -> Result<u64, String> {
        let marks: Vec<PointMark> = raw
            .iter()
            .map(|m| PointMark::new(m.pos, m.color, m.radius_px))
            .collect();
        let id = self.scene.spawn();
        let table = match self.backend.create_points(&PointTableDesc { data: &marks }) {
            Ok(t) => t,
            Err(e) => {
                let _ = self.scene.despawn(id); // generation +1: a failed shape is not reused (same scheme as add_mesh)
                return Err(format!("create_points: {e:?}"));
            }
        };
        let _ = table; // DrawPoints command consumption (push below)
        let bits = enc_entity(id);
        // B2: retain positions for CPU point picking (origin=[0,0;3] host-local, same as DrawPoints)
        self.point_clouds.insert(
            bits,
            PointCloudState {
                entity: id,
                origin: [0.0; 3],
                transform: IDENTITY,
                positions: raw.iter().map(|m| m.pos).collect(),
            },
        );
        Ok(bits)
    }

    // ── Scene tree (CAPI-30..34) ────────────────────────────────────────────

    /// CAPI-30: Create a group node (Component::Group). `name` stored engine-local.
    pub fn create_group(&mut self, name: Option<&str>) -> Result<u64, String> {
        let id = self.scene.spawn_group();
        let bits = enc_entity(id);
        if let Some(n) = name {
            self.node_names.insert(bits, n.to_string());
        }
        Ok(bits)
    }

    /// CAPI-31: Reparent. `parent=0` = detach to root.
    pub fn set_parent(&mut self, child: u64, parent: u64) -> Result<(), String> {
        let c = dec_entity(child);
        let p = if parent == 0 {
            None
        } else {
            Some(dec_entity(parent))
        };
        self.scene.set_parent(c, p).map_err(|e| e.to_string())
    }

    /// CAPI-32: Get parent handle (0 = root/no parent).
    pub fn get_parent(&self, entity: u64) -> Result<u64, String> {
        let e = dec_entity(entity);
        Ok(self
            .scene
            .get_parent(e)
            .map_err(|e2| e2.to_string())?
            .map_or(0, enc_entity))
    }

    /// CAPI-33: Set group world translation offset (metres). NaN/Inf → reject (zero write).
    pub fn set_group_offset(
        &mut self,
        group: u64,
        dx: f64,
        dy: f64,
        dz: f64,
    ) -> Result<(), String> {
        if !(dx.is_finite() && dy.is_finite() && dz.is_finite()) {
            return Err("offset must be finite".into());
        }
        let g = dec_entity(group);
        self.scene
            .set_group_offset(g, [dx, dy, dz])
            .map_err(|e| e.to_string())
    }

    /// CAPI-34: Read group own offset (not accumulated).
    pub fn get_group_offset(&self, group: u64) -> Result<[f64; 3], String> {
        let g = dec_entity(group);
        self.scene.group_offset(g).map_err(|e| e.to_string())
    }

    /// CAPI-28: mount an MVT tile directory (FileSource; same scheme = geometry ready at load time, the view fed afterwards).
    /// Single-tile bytes injection (the wasm face; the C dir scan wraps the
    /// same ensure+decode+fit tail). z/x/y domain-checked by TileId::new.
    pub fn load_tile_bytes(
        &mut self,
        z: u8,
        x: u32,
        y: u32,
        bytes: &[u8],
    ) -> Result<u64, String> {
        let id = visiaengine_io_tiles::TileId::new(z, x, y)
            .ok_or_else(|| format!("tile id out of range: {z}/{x}/{y}"))?;
        let source = visiaengine_io_tiles::MemSource::new();
        source.insert(z, x, y, bytes.to_vec());
        let mut set = visiaengine_io_tiles::TileSet::new(Box::new(source))
            .map_err(|e| format!("tiles: {e:?}"))?;
        set.ensure(&[id]).map_err(|e| format!("tiles ensure: {e:?}"))?;
        let decoded = set
            .decoded(&id)
            .map(|t| visiaengine_io_tiles::GeoTile::from_layers(id, &t.layers));
        self.mount_tile_layer(set, vec![id], vec![(id, decoded)])
    }

    /// Shared tile-layer mount tail (dir scan and bytes injection both land
    /// here): bbox center fit on the first layer + GeoTile baking.
    fn mount_tile_layer(
        &mut self,
        set: visiaengine_io_tiles::TileSet,
        ids: Vec<visiaengine_io_tiles::TileId>,
        decoded: Vec<(visiaengine_io_tiles::TileId, Option<visiaengine_io_tiles::GeoTile>)>,
    ) -> Result<u64, String> {
        // scene center = center of the union bbox of all tiles (batch vertices baked into a center small-value-range frame).
        let (min_x, min_y, max_x, max_y) = union_bbox(&ids);
        let center = [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0, 0.0];
        // P1 camera fit (same pattern as mount_geo): ortho top-down at batch
        // center. Guarded for the same reason as the raster mount (card 4 = A).
        if self.layers.is_empty() {
            let radius = ((max_x - min_x) / 2.0).max((max_y - min_y) / 2.0) * FIT_PAD;
            self.mode = Proj::Ortho;
            self.zoom = radius.max(10.0);
            self.rig = CameraRig::look_at(
                [center[0], center[1], radius.max(10.0)],
                center,
                [0.0, 1.0, 0.0],
            );
        }
        let (mut wpos, mut widx) = (Vec::new(), Vec::new());
        let mut strokes = Vec::new();
        let mut points = Vec::new();
        for (_, lite) in &decoded {
            let Some(lite) = lite else { continue };
            for f in &lite.features {
                let class = f.attrs.get("class").cloned().unwrap_or_default();
                match &f.geom {
                    visiaengine_io_tiles::TileGeom::Poly(ring) => {
                        let start = wpos.len() as u32;
                        for p in ring {
                            wpos.push([(p[0] - center[0]) as f32, (p[1] - center[1]) as f32, 0.0]);
                        }
                        for i in 1..ring.len().saturating_sub(1) as u32 {
                            widx.extend([start, start + i, start + i + 1]);
                        }
                    }
                    visiaengine_io_tiles::TileGeom::Line(pts) => {
                        let (color, width) = if class == "boundary" {
                            ([0.30, 0.55, 0.55], 3.0)
                        } else {
                            ([0.95, 0.62, 0.18], 9.0)
                        };
                        for w in pts.windows(2) {
                            strokes.push(StrokeSeg::new(
                                [
                                    (w[0][0] - center[0]) as f32,
                                    (w[0][1] - center[1]) as f32,
                                    0.0,
                                ],
                                [
                                    (w[1][0] - center[0]) as f32,
                                    (w[1][1] - center[1]) as f32,
                                    0.0,
                                ],
                                color,
                                width,
                            ));
                        }
                    }
                    visiaengine_io_tiles::TileGeom::Point(p) => {
                        points.push(PointMark::new(
                            [(p[0] - center[0]) as f32, (p[1] - center[1]) as f32, 0.0],
                            [0.9, 0.25, 0.35],
                            9.0,
                        ));
                    }
                    visiaengine_io_tiles::TileGeom::MultiPoint(_) => {}
                }
            }
        }
        let count = ids.len() as u64;
        let layer = TileLayer {
            kind: LayerKind::Vector,
            set,
            ids,
            http: false,
            center,
            wpos,
            widx,
            strokes,
            points,
            raster_quads: Vec::new(),
            raster_gpu: Vec::new(),
            uploaded: None,
        };
        self.mount_layer(layer);
        let _ = decoded;
        Ok(count)
    }

    pub fn load_mvt_dir(&mut self, path: &str, z: u8) -> Result<u64, String> {
        let source = visiaengine_io_tiles::FileSource::new(path);
        let mut set = visiaengine_io_tiles::TileSet::new(Box::new(source))
            .map_err(|e| format!("tiles: {e:?}"))?;
        // enumerate the tiles actually present in the directory (z fixed; scan {path}/{z}/*/*.mvt)
        let mut ids: Vec<visiaengine_io_tiles::TileId> = Vec::new();
        let zdir = std::path::Path::new(path).join(z.to_string());
        let entries =
            std::fs::read_dir(&zdir).map_err(|e| format!("read_dir {}: {e}", zdir.display()))?;
        for xentry in entries.flatten() {
            let xname = xentry.file_name().to_string_lossy().into_owned();
            let Ok(x) = xname.parse::<u32>() else {
                continue;
            };
            let xdir = zdir.join(&xname);
            let Ok(files) = std::fs::read_dir(&xdir) else {
                continue;
            };
            for f in files.flatten() {
                let fname = f.file_name().to_string_lossy().into_owned();
                if let Some(ystr) = fname.strip_suffix(".mvt")
                    && let Ok(y) = ystr.parse::<u32>()
                    && let Some(id) = visiaengine_io_tiles::TileId::new(z, x, y)
                {
                    ids.push(id);
                }
            }
        }
        if ids.is_empty() {
            return Err(format!("no .mvt tiles under {}/{z}", zdir.display()));
        }
        ids.sort_by_key(|i| (i.y, i.x));
        set.ensure(&ids)
            .map_err(|e| format!("tiles ensure: {e:?}"))?;
        let decoded: Vec<_> = ids
            .iter()
            .map(|id| {
                (
                    *id,
                    set.decoded(id)
                        .map(|t| visiaengine_io_tiles::GeoTile::from_layers(*id, &t.layers)),
                )
            })
            .collect();
        self.mount_tile_layer(set, ids, decoded)
    }

    /// CAPI-35: mount an HTTP tile source (N1.4). HTTP has no directory
    /// listing = discovery-driven mounting: the mount only builds the
    /// scheduler (HttpSource + 5s timeout guard); the tile set is discovered
    /// by set_tile_view viewport enumeration, pumped per render frame
    /// (default 4 tiles/frame budget). Returns 0 = mounted (no count
    /// semantics — forked from CAPI-28 in the clause: nothing to scan).
    pub fn load_mvt_http(&mut self, root_url: &str, z: u8) -> Result<u64, String> {
        if root_url.is_empty() || z > 30 {
            return Err("tile http domain (non-empty url, z<=30)".into());
        }
        let source = visiaengine_io_tiles::HttpSource::new(root_url);
        let set = visiaengine_io_tiles::TileSet::new(Box::new(source))
            .map_err(|e| format!("tiles: {e:?}"))?;
        self.mount_layer(TileLayer {
            kind: LayerKind::Vector,
            set,
            ids: Vec::new(),
            http: true,
            center: [0.0, 0.0, 0.0],
            wpos: Vec::new(),
            widx: Vec::new(),
            strokes: Vec::new(),
            points: Vec::new(),
            raster_quads: Vec::new(),
            raster_gpu: Vec::new(),
            uploaded: None,
        });
        Ok(0)
    }

    /// CAPI-29: feed the viewport bbox (3857) -> visible->ensure -> rebuild batch geometry (rebuilt only when the visible set changes).
    pub fn set_tile_view(
        &mut self,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Result<u64, String> {
        let Some(layer) = self.layer_of_mut(LayerKind::Vector) else {
            return Err("tile layer not mounted (load_mvt_dir first)".into());
        };
        if !(min_x.is_finite()
            && min_y.is_finite()
            && max_x.is_finite()
            && max_y.is_finite()
            && min_x < max_x
            && min_y < max_y)
        {
            return Err("tile view domain (finite, min<max)".into());
        }
        let z_of = layer.ids.first().map(|i| i.z).unwrap_or(0);
        let ids = visiaengine_io_tiles::TileSet::visible((min_x, min_y, max_x, max_y), z_of);
        if layer.http {
            // N1.4 discovery mode: any enumerated tile is admissible (no dir
            // scan happened); arm the pump, report as "visible" — actual byte
            // arrival paces through the render-loop pump budget.
            layer.ids = ids.clone();
            // scene center = current view bbox center (bakes a stable
            // small-value frame for the batch; PIT-8 safe domain).
            // Review fix #4: the batch vertices are baked RELATIVE to center;
            // moving the center without invalidating the uploaded GPU tables
            // rendered the whole layer shifted by the center delta. Drop the
            // upload-cache so the rebuild path re-fires with fresh offsets
            // (old GPU ids leak until re-mount — id-monotonic REND-07 has no
            // remove face; one layer's tables = accepted, ledgered).
            let new_center = [(min_x + max_x) / 2.0, (min_y + max_y) / 2.0, 0.0];
            if layer.center != new_center {
                layer.center = new_center;
                layer.uploaded = None;
                // Vertices baked to the OLD center are stale too; drop them so
                // the rebuild regenerates the full batch from the Done set.
                layer.wpos.clear();
                layer.widx.clear();
                layer.strokes.clear();
                layer.points.clear();
            }
            let started = layer.set.begin(&ids);
            let _ = started;
            return Ok(ids.len() as u64);
        }
        let alive: Vec<visiaengine_io_tiles::TileId> =
            ids.into_iter().filter(|i| layer.ids.contains(i)).collect();
        let stats = layer
            .set
            .ensure(&alive)
            .map_err(|e| format!("tiles ensure: {e:?}"))?;
        let visible_count = alive.len() as u64;
        let _ = stats;
        layer.ids = layer.ids.clone(); // keep full known set (batch rebuild uses mounted set)
        Ok(visible_count)
    }

    /// CAPI-22: single world-anchored label (same scheme as add_points = management domain outside items, shapes enumerable,
    /// v0 deletion stays off the remove face = explicit account). **Font not loaded = explicit rejection** (the data entry carries no dormant meaning,
    /// paired with GEO-25 style dormancy: style dormant / entry rejects). Empty text / size out of domain = rejected with zero commit.
    pub fn add_label(
        &mut self,
        world: [f64; 3],
        text: &str,
        rgba_srgb: [f32; 4],
        size_px: f32,
    ) -> Result<u64, String> {
        let Some(face) = self.font.as_ref() else {
            return Err("add_label: font not loaded (CAPI-21 first)".into());
        };
        if text.is_empty() || size_px <= 0.0 || !size_px.is_finite() {
            return Err("add_label: empty text / size domain".into());
        }
        let (quads, pen) = visiaengine_io_text::layout(text, face, &mut self.glyphs, size_px);
        if quads.is_empty() {
            return Err("add_label: no raster quads".into());
        }
        let lin = visiaengine_core::srgb_to_linear(rgba_srgb);
        let marks: Vec<visiaengine_render::LabelMark> = quads
            .iter()
            .map(|q| {
                visiaengine_render::LabelMark::new(
                    [world[0] as f32, world[1] as f32, world[2] as f32],
                    lin,
                    [q.uv0[0], q.uv0[1], q.uv1[0], q.uv1[1]],
                    [
                        q.size_px[0],
                        q.size_px[1],
                        q.top_left_px[0] - pen / 2.0,
                        q.top_left_px[1],
                    ],
                )
            })
            .collect();
        let id = self.scene.spawn();
        let table = match self
            .backend
            .create_labels(&visiaengine_render::LabelTableDesc { data: &marks })
        {
            Ok(t) => t,
            Err(e) => {
                let _ = self.scene.despawn(id);
                return Err(format!("create_labels: {e:?}"));
            }
        };
        self.extra_cmds.push(DrawCommand::DrawLabels {
            table,
            origin: [0.0; 3],
            transform: IDENTITY,
        });
        Ok(enc_entity(id))
    }

    /// CAPI-16: delete an entity (items/attr_of/hidden three-face cleanup; old handle re-entry = same scheme as double destroy).
    pub fn remove_entity(&mut self, entity: u64) -> Result<(), String> {
        let Some(pos) = self
            .items
            .iter()
            .position(|i| enc_entity(i.entity) == entity)
        else {
            return Err(format!("unknown entity bits {entity:#018x}"));
        };
        let item = self.items.remove(pos);
        self.scene
            .despawn(item.entity)
            .map_err(|e| format!("remove_entity: {e:?}"))?;
        self.attr_of.remove(&entity);
        self.hidden.retain(|h| *h != entity);
        Ok(())
    }

    /// CAPI-10: entity-keyed three-type attribute reads (shape encoded the same as the pick output);
    /// the four absence paths (no row / no column / type mismatch / empty cell) are all None rather than zero values.
    /// N2.4 (CAPI-36 rs-face): property diff-update — typed-reject write.
    /// `f64/str/bool` select the column type; writing a DIFFERENT type to an
    /// existing column (override or source) is rejected (CORE-11/12
    /// first-write-typed), reported as `Err("type-conflict: ...")`. A missing
    /// column is CREATED (first write types it). Zero effect on the render
    /// face — attributes are query/host data (read back via attr_*).
    /// Unknown entity (nothing loaded under it) = reject.
    pub fn update_entity_props_f64(
        &mut self,
        entity: u64,
        name: &str,
        v: f64,
    ) -> Result<(), String> {
        let row = self.override_row(entity)?;
        let row_attrs = self.attr_overrides.get_mut(&entity).expect("row seeded");
        if !row_attrs.set_f64(row, name, v) {
            // Type conflict against an existing typed column.
            if let Some(cur) = row_attrs.f64(row, name) {
                let _ = cur;
                return Ok(()); // same-type overwrite = the diff update itself
            }
            return Err(format!(
                "type-conflict: column '{name}' holds a different type"
            ));
        }
        Ok(())
    }

    /// str variant (see update_entity_props_f64).
    pub fn update_entity_props_str(
        &mut self,
        entity: u64,
        name: &str,
        v: impl Into<String>,
    ) -> Result<(), String> {
        let row = self.override_row(entity)?;
        let row_attrs = self.attr_overrides.get_mut(&entity).expect("row seeded");
        if !row_attrs.set_str(row, name, v) {
            return Err(format!(
                "type-conflict: column '{name}' holds a different type"
            ));
        }
        Ok(())
    }

    /// bool variant (see update_entity_props_f64).
    pub fn update_entity_props_bool(
        &mut self,
        entity: u64,
        name: &str,
        v: bool,
    ) -> Result<(), String> {
        let row = self.override_row(entity)?;
        let row_attrs = self.attr_overrides.get_mut(&entity).expect("row seeded");
        if !row_attrs.set_bool(row, name, v) {
            return Err(format!(
                "type-conflict: column '{name}' holds a different type"
            ));
        }
        Ok(())
    }

    /// One row (row 0) per entity in the override store, seeded on first
    /// update. Unknown entity (nothing loaded under it) = reject.
    fn override_row(&mut self, entity: u64) -> Result<usize, String> {
        if !self.attr_of.contains_key(&entity) && !self.pcl_meta.contains_key(&entity) {
            return Err(format!("update_entity_props: unknown entity {entity}"));
        }
        self.attr_overrides.entry(entity).or_insert_with(|| {
            let mut a = visiaengine_core::AttrSet::new();
            a.add_row();
            a
        });
        Ok(0)
    }

    #[must_use]
    pub fn attr_f64(&self, entity: u64, name: &str) -> Option<f64> {
        // N2.4: override layer reads FIRST (diff-update wins over source).
        if let Some(ov) = self.attr_overrides.get(&entity)
            && let Some(v) = ov.f64(0, name)
        {
            return Some(v);
        }
        if let Some(&(d, r)) = self.attr_of.get(&entity) {
            return self.geo_docs.get(d)?.attr_f64(r, name);
        }
        self.pcl_meta.get(&entity).and_then(|m| m.f64(0, name))
    }

    #[must_use]
    pub fn attr_str(&self, entity: u64, name: &str) -> Option<&str> {
        // N2.4: override layer first. Borrow dance: overrides hold String —
        // fall through to source when the override column misses.
        if let Some(ov) = self.attr_overrides.get(&entity)
            && ov.str_value(0, name).is_some()
        {
            return ov.str_value(0, name);
        }
        if let Some(&(d, r)) = self.attr_of.get(&entity) {
            return self.geo_docs.get(d)?.attr_str(r, name);
        }
        self.pcl_meta
            .get(&entity)
            .and_then(|m| m.str_value(0, name))
    }

    #[must_use]
    pub fn attr_bool(&self, entity: u64, name: &str) -> Option<bool> {
        // N2.4: override layer first.
        if let Some(ov) = self.attr_overrides.get(&entity)
            && let Some(v) = ov.bool(0, name)
        {
            return Some(v);
        }
        if let Some(&(d, r)) = self.attr_of.get(&entity) {
            return self.geo_docs.get(d)?.attr_bool(r, name);
        }
        self.pcl_meta.get(&entity).and_then(|m| m.bool(0, name))
    }
}

#[cfg(all(feature = "web", target_arch = "wasm32"))]
impl Engine {
    /// Canvas engine construction (batch 7 J1: async device + canvas surface in one go).
    pub async fn new_canvas(canvas: &web_sys::HtmlCanvasElement) -> Option<Self> {
        let w = canvas.width().max(1);
        let h = canvas.height().max(1);
        let mut backend = HeadlessBackend::new_async(w, h).await?;
        let sw = backend.attach_canvas(canvas).ok()?;
        Some(Self {
            w,
            h,
            rig: CameraRig::look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            mode: Proj::Persp,
            morph: None,
            zoom: 1.0,
            down: false,
            last: (0.0, 0.0),
            backend,
            target: TargetMode::Window(sw),
            scene: Self::new_scene(),
            items: Vec::new(),
            extra_cmds: Vec::new(),
            geo_docs: Vec::new(),
            attr_of: std::collections::HashMap::new(),
            attr_overrides: std::collections::HashMap::new(),
            hidden: Vec::new(),
            node_names: std::collections::HashMap::new(),
            clip: None,
            edl: None,
            post: Vec::new(),
            clear: None,
            shadow: None,
            fly: None,
            map: None,
            font: None,
            glyphs: visiaengine_io_text::GlyphCache::new(),
            frame_cache: None,
            evt: None,
            pcl_meta: std::collections::HashMap::new(),
            point_clouds: std::collections::HashMap::new(),
            layers: Vec::new(),
            env_sh: None,
        })
    }
}

/// Decode a C-API u64 handle back to EntityId (inverse of enc_entity).
/// All u64 values are valid bit-forms (CAPI-01: slot0gen0=0 is a legal entity);
/// callers that need a "no entity" sentinel use 0 before entering the scene-tree
/// path (where 0 = "detach to root" in set_parent / get_parent).
#[allow(clippy::cast_possible_truncation)]
const fn dec_entity(bits: u64) -> visiaengine_core::EntityId {
    visiaengine_core::EntityId::from_raw((bits >> 32) as u32, bits as u32)
}

const fn add_origin(base: [f64; 3], offset: [f64; 3]) -> [f64; 3] {
    [
        base[0] + offset[0],
        base[1] + offset[1],
        base[2] + offset[2],
    ]
}

const IDENTITY: [[f64; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

#[cfg(test)]
mod attr_tests {
    //! CAPI-10/11/12 (RED first): retention face + entity->(doc,row) reverse lookup + three-type read entries.
    use super::*;
    use std::fs;

    /// Minimal two-layer piece: A with all three types (str/f64/bool), B name only (empty cell = the None contract face).
    const GEO: &str = r#"{"type":"FeatureCollection","features":[
     {"type":"Feature","properties":{"name":"buildingA","height":12.5,"active":true},
      "geometry":{"type":"Polygon","coordinates":[[[0.0,0.0],[1.0,0.0],[1.0,1.0],[0.0,0.0]]]}},
     {"type":"Feature","properties":{"name":"buildingB"},
      "geometry":{"type":"Polygon","coordinates":[[[2.0,2.0],[3.0,2.0],[3.0,3.0],[2.0,2.0]]]}}
    ]}"#;

    fn tmp_layer(tag: &str, body: &str) -> String {
        let dir = std::env::temp_dir().join(format!(
            "ve-attr-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0),
        ));
        fs::create_dir_all(dir.clone()).unwrap();
        let path = dir.join("layer.geojson");
        fs::write(&path, body).unwrap();
        path.to_str().unwrap().to_owned()
    }

    fn loaded() -> Engine {
        let mut e = Engine::new_headless(8, 8).expect("headless engine");
        e.load_geojson(&tmp_layer("a", GEO)).expect("load geojson");
        e
    }

    #[test]
    fn attr_three_types_hit_via_entity() {
        // spec: CAPI-10
        let e = loaded();
        let a = enc_entity(e.entity_at(0).expect("feature 0 entity"));
        let b = enc_entity(e.entity_at(1).expect("feature 1 entity"));
        assert_eq!(e.attr_str(a, "name"), Some("buildingA"));
        assert_eq!(e.attr_f64(a, "height"), Some(12.5));
        assert_eq!(e.attr_bool(a, "active"), Some(true));
        assert_eq!(e.attr_str(b, "name"), Some("buildingB"));
    }

    #[test]
    fn attr_missing_is_none_never_zero() {
        // spec: CAPI-10
        let e = loaded();
        let b = enc_entity(e.entity_at(1).expect("feature 1 entity"));
        assert_eq!(e.attr_f64(b, "height"), None); // row present, cell empty
        assert_eq!(e.attr_f64(b, "nope"), None); // column absent
        assert_eq!(e.attr_f64(b, "name"), None); // type mismatch (str column asked for f64)
        assert_eq!(e.attr_bool(b, "name"), None); // type mismatch (str column asked for bool)
    }

    #[test]
    fn attr_str_content_exact() {
        // spec: CAPI-11
        let e = loaded();
        let a = enc_entity(e.entity_at(0).expect("entity"));
        assert_eq!(e.attr_str(a, "name"), Some("buildingA"));
        assert_eq!(e.attr_str(a, "missing"), None);
    }

    #[test]
    fn attr_multi_load_rows_independent() {
        // spec: CAPI-12
        let mut e = loaded();
        let old = enc_entity(e.entity_at(0).expect("old entity"));
        let second = r#"{"type":"FeatureCollection","features":[
          {"type":"Feature","properties":{"name":"towerC","height":99.0},
           "geometry":{"type":"Polygon","coordinates":[[[5.0,5.0],[6.0,5.0],[6.0,6.0],[5.0,5.0]]]}}
        ]}"#;
        assert_eq!(e.load_geojson(&tmp_layer("b", second)), Ok(1));
        assert_eq!(e.attr_str(old, "name"), Some("buildingA")); // old row does not bleed
        let c = enc_entity(e.entity_at(2).expect("new entity"));
        assert_eq!(e.attr_f64(c, "height"), Some(99.0));
        assert_eq!(e.attr_str(c, "name"), Some("towerC"));
        assert_eq!(e.attr_bool(old, "active"), Some(true)); // column isolation across docs
    }
}

#[cfg(test)]
mod mut_spec_tests {
    //! CAPI-13..16 (RED first): visibility filtering / query / procedural add / delete-ABA.
    use super::*;
    use crate::ffi::enc_entity;

    const TWOPRIM: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/twoprim.glb"
    );

    fn eng() -> Engine {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        assert_eq!(e.load_gltf(TWOPRIM).unwrap(), 2, "twoprim = 2 entities");
        e
    }

    fn quad() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0.0, 0.0, 1.0]; 4],
            vec![0, 1, 2, 0, 2, 3],
        )
    }

    // spec: CAPI-20
    #[test]
    fn clip_filters_pick_retry_and_set_domain() {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        let (pos, nrm, idx) = quad();
        // note: the capi pick domain = positions x IDENTITY (origin lives on the render path = established fact) --
        // the test bakes z into positions to merge the two frames, sidestepping that gap without asserting on borrowed health
        let flat: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 8.0 - 4.0, p[1] * 8.0 - 4.0, 0.0])
            .collect();
        let mut frontz = flat.clone();
        for q in &mut frontz {
            q[2] = 2.0;
        }
        let front = e
            .add_mesh(
                &frontz,
                Some(&nrm),
                &idx,
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                1.0,
                0.0,
            )
            .expect("front");
        let back = e
            .add_mesh(
                &flat,
                Some(&nrm),
                &idx,
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                1.0,
                0.0,
            )
            .expect("back");
        let hitk = |e: &Engine| e.pick(80.0, 60.0).map(enc_entity);
        assert_eq!(hitk(&e), Some(front), "default hits the front wall");
        // world plane z<=1 keeps: the front wall (z=2) hit point is clipped -> the retry loop hits the back wall
        assert_eq!(e.set_clips(&[[0.0, 0.0, -1.0, 1.0]]), Ok(()));
        assert_eq!(
            hitk(&e),
            Some(back),
            "clipping the front wall must retry onto the back wall (not a miss)"
        );
        assert_eq!(
            e.render(),
            Ok(()),
            "clip enters the frame and the render path works"
        );
        assert_eq!(e.clips(), vec![[0.0, 0.0, -1.0, 1.0]]);
        // n=0 clear = the only clearing form -> the front wall hits again
        assert_eq!(e.set_clips(&[]), Ok(()));
        assert_eq!(hitk(&e), Some(front), "clear restores");
        assert!(e.clips().is_empty());
        // degenerate rejection (same source as FFI): zero normal / non-finite / >4
        assert!(e.set_clips(&[[0.0, 0.0, 0.0, 1.0]]).is_err());
        assert!(e.set_clips(&[[f64::NAN, 0.0, 0.0, 0.0]]).is_err());
        assert!(e.set_clips(&[[0.0, 1.0, 0.0, 0.0]; 5]).is_err());
    }

    fn geo(fc: &str) -> visiaengine_geo::GeoDocument {
        visiaengine_geo::parse_geojson_lenient(fc.as_bytes())
            .unwrap()
            .0
    }

    const DEJAVU: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/DejaVuSans.ttf"
    ));

    // spec: GEO-25
    #[test]
    fn geo_labels_mount_only_with_font_and_text_style() {
        let fc = r#"{"type":"FeatureCollection","features":[
          {"type":"Feature","properties":{"text-field":"{name}"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}
        ]}"#;
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        // no font: styles present yet zero output (mounting passes, no error)
        assert_eq!(e.mount_geo(geo(fc)).unwrap(), 1);
        assert_eq!(e.label_cmd_count(), 0, "no font = zero label output");
        // font loaded + reload: table produced into the queue
        e.set_font(DEJAVU).expect("font");
        let mut e2 = Engine::new_headless(160, 120).expect("adapter");
        e2.set_font(DEJAVU).expect("font");
        assert_eq!(e2.mount_geo(geo(fc)).unwrap(), 1);
        assert_eq!(e2.label_cmd_count(), 1, "one doc one label-table command");
        assert_eq!(e2.render(), Ok(()), "label-frame render path works");
        // column present (the first row name built the column) but this row is empty -> column semantics skip, no literal "{name}" text
        let miss = r#"{"type":"FeatureCollection","features":[
          {"type":"Feature","properties":{"text-field":"{name}","name":"X"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}},
          {"type":"Feature","properties":{"text-field":"{name}"},"geometry":{"type":"Point","coordinates":[11.0,51.0]}}
        ]}"#;
        let mut e3 = Engine::new_headless(160, 120).expect("adapter");
        e3.set_font(DEJAVU).expect("font");
        e3.mount_geo(geo(miss)).unwrap();
        assert_eq!(e3.label_cmd_count(), 1, "one table merged");
        assert_eq!(
            e3.label_mark_total(),
            1,
            "missing-value row = skip (X one patch, second row zero patches)"
        );
        // constant text (column absent = literal form)
        let lit = r#"{"type":"FeatureCollection","features":[
          {"type":"Feature","properties":{"text-field":"Gate"},"geometry":{"type":"Point","coordinates":[10.0,50.0]}}
        ]}"#;
        let mut e4 = Engine::new_headless(160, 120).expect("adapter");
        e4.set_font(DEJAVU).expect("font");
        e4.mount_geo(geo(lit)).unwrap();
        assert_eq!(e4.label_cmd_count(), 1, "literal text produces a label");
    }

    impl Engine {
        /// quad count among the labels in extra_cmds (GEO-25 zero-output / host introspection entry).
        #[must_use]
        pub fn label_cmd_count(&self) -> usize {
            self.extra_cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::DrawLabels { .. }))
                .count()
        }

        /// total quads of the label table (column/literal semantics resolution entry, host-inspectable).
        #[must_use]
        pub fn label_mark_total(&self) -> usize {
            self.backend.label_mark_total()
        }
    }

    // spec: CAPI-23
    #[test]
    fn fly_to_wallclock_advance_cancel_and_reroute() {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        // dur=0 = teleport form (immediate landing, not an error); near/far stay at current values [A1]
        let near0 = e.rig.near;
        assert!(
            e.fly_to([1.0, 2.0, 3.0], 0.3, 0.2, 12.0, 25.0, 1.0, 0)
                .is_ok()
        );
        assert!(e.fly_state_done(), "done right after teleport");
        assert!((e.rig.target[0] - 1.0).abs() < 1e-9, "teleport lands");
        assert!(
            (e.rig.near - near0).abs() < 1e-12,
            "near stays at the from value"
        );
        // domain rejection: dist<=0 / non-finite / fov>=pi
        assert!(e.fly_to([0.0; 3], 0.0, 0.0, -1.0, 25.0, 1.0, 500).is_err());
        assert!(
            e.fly_to([0.0; 3], 0.0, 0.0, 10.0, 25.0, f64::NAN, 500)
                .is_err()
        );
        assert!(e.fly_to([0.0; 3], 0.0, 0.0, 10.0, 25.0, 4.0, 500).is_err());
        // takeoff 200ms: progress monotonic -> arrival lands exact (t>=1 returns the to original value directly)
        assert!(
            e.fly_to([5.0, 5.0, 0.0], 0.7, 0.5, 40.0, 30.0, 1.1, 200)
                .is_ok()
        );
        assert!(!e.fly_state_done());
        let p1 = e.fly_progress().expect("t1");
        std::thread::sleep(std::time::Duration::from_millis(120));
        let _ = e.render();
        let p2 = e.fly_progress().unwrap_or(1.0);
        assert!(p2 >= p1, "progress monotonic {p1}->{p2}");
        let mut guard = 0;
        while !e.fly_state_done() && guard < 60 {
            let _ = e.render();
            std::thread::sleep(std::time::Duration::from_millis(20));
            guard += 1;
        }
        assert!((e.rig.target[0] - 5.0).abs() < 1e-9, "arrival lands exact");
        assert!((e.rig.dist - 40.0).abs() < 1e-9, "dist fully landed");
        // input interrupt (A branch): pointer-down cancel -> done + no progress + can re-fly; keys (5) do not interrupt
        assert!(e.fly_to([0.0; 3], 0.0, 0.0, 50.0, 30.0, 1.1, 5000).is_ok());
        assert!(
            e.apply_input(5, 0.0, 0.0, 0.0).is_some(),
            "key input consumed"
        );
        assert!(
            !e.fly_state_done(),
            "keys do not interrupt flight [settled A branch]"
        );
        assert!(
            e.apply_input(2, 10.0, 10.0, 0.0).is_some(),
            "pointer press still consumed"
        );
        assert!(e.fly_state_done(), "cancel = not flying (idle/done merged)");
        assert!(e.fly_progress().is_none(), "no progress after cancel");
        // reroute continuity: leg1 mid-flight -> leg2 from = current sample => zero first-frame jump
        assert!(
            e.fly_to([9.0, 9.0, 0.0], 0.7, 0.5, 44.0, 30.0, 1.1, 3000)
                .is_ok()
        );
        std::thread::sleep(std::time::Duration::from_millis(60));
        let _ = e.render();
        let mid = e.rig.target;
        assert!(
            e.fly_to([-9.0, -9.0, 0.0], 0.7, 0.5, 44.0, 30.0, 1.1, 3000)
                .is_ok()
        );
        let _ = e.render();
        let d = (e.rig.target[0] - mid[0]).abs();
        assert!(
            d < 0.5,
            "reroute first frame continuous (jump distance {d})"
        );
    }

    // spec: CAPI-25
    #[test]
    fn map_set_render_routing_and_domains() {
        let mut e = Engine::new_headless(200, 100).expect("adapter");
        let (pos, _nrm, idx) = quad();
        let big: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 20.0, p[1] * 20.0, 0.0])
            .collect();
        e.add_mesh(&big, None, &idx, [0.1, 0.6, 0.2, 1.0], [0.0; 3], 1.0, 0.0)
            .expect("ground"); // first-piece shape may = 0 (CAPI-01 legal division, success judged by rc -- this pit on the books for the third time)
        // domain rejection triple (fx+fw>1 / zoom non-positive / dirty NaN)
        assert!(
            e.set_map(Some((0.7, 0.0, 0.4, 0.4)), 40.0).is_err(),
            "over the 1 boundary must reject"
        );
        assert!(
            e.set_map(Some((0.7, 0.0, 0.25, 0.25)), 0.0).is_err(),
            "zoom domain"
        );
        assert!(
            e.set_map(Some((f32::NAN, 0.0, 0.2, 0.2)), 30.0).is_err(),
            "frac domain"
        );
        assert!(e.map_rect().is_none(), "zero residue after rejection");
        // navigate with no map = rejected (no covert reroute)
        assert!(e.navigate_click(150.0, 50.0, 500).is_err());
        // enable the map (right 1/4 portrait strip) + render: both the map region and the main region carry content (non-background pixels)
        e.set_map(Some((0.72, 0.02, 0.26, 0.96)), 30.0)
            .expect("map on");
        e.render().expect("render dual");
        let img = e.frame_cache.clone().expect("cache");
        let bg = |i: usize| img[i * 4] < 30 && img[i * 4 + 1] < 40 && img[i * 4 + 2] < 60;
        let nonbg = |x: u32, y: u32| !bg((y * 200 + x) as usize);
        let cnt = |x0: u32, x1: u32| -> u32 {
            (10..90)
                .flat_map(|y| (x0..x1).filter(move |x| nonbg(*x, y)))
                .count() as u32
        };
        let (main_c, map_c) = (cnt(10, 130), cnt(150, 195));
        println!("PROBE map main={main_c} map={map_c}");
        assert!(main_c > 200, "main-view ground absent {main_c}");
        assert!(map_c > 200, "minimap top-view ground absent {map_c}");
        // routing: a press inside the minimap = consumed with the pose unchanged; a drag in the main region = yaw changes
        let yaw0 = e.camera_pose().yaw;
        assert_eq!(
            e.apply_input(2, 180.0, 50.0, 0.0),
            Some(true),
            "minimap press consumed"
        );
        let _ = e.apply_input(1, 190.0, 50.0, 0.0);
        assert!(
            (e.camera_pose().yaw - yaw0).abs() < 1e-12,
            "minimap drag no-op"
        );
        let _ = e.apply_input(2, 60.0, 50.0, 0.0);
        let _ = e.apply_input(1, 90.0, 50.0, 0.0);
        assert!(
            (e.camera_pose().yaw - yaw0).abs() > 1e-6,
            "main-region drag took effect"
        );
        // the minimap pick hits a ground entity (top-view center = near the main target)
        let hit = e.pick(180.0, 50.0);
        assert!(hit.is_some(), "pick inside the minimap must hit");
        // disable the map back to the old path (canary construction position) + the rect vanishes
        e.set_map(None, 0.0).expect("clear");
        assert!(e.map_rect().is_none());
        e.render().expect("render legacy again");
    }

    // spec: CAPI-27
    #[test]
    fn camera_pose_readback_is_single_source_of_truth() {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        let p0 = e.camera_pose();
        // idle read = steady state; read after teleport = new value (host HUD assertion face)
        assert!(e.fly_state_done());
        assert!((e.camera_pose().target[0] - p0.target[0]).abs() < 1e-12);
        let tgt = [3.0, 4.0, 0.0];
        e.fly_to(tgt, p0.yaw, p0.pitch, p0.dist, p0.zoom, p0.fov_y, 0)
            .expect("tp");
        let p1 = e.camera_pose();
        assert_eq!(
            [p1.target[0], p1.target[1]],
            [3.0, 4.0],
            "readback = landed [CAPI-27]"
        );
        assert!((p1.dist - p0.dist).abs() < 1e-12, "distance kept");
    }

    // spec: CAPI-26
    #[test]
    fn navigate_click_flies_to_ground_target() {
        let mut e = Engine::new_headless(200, 100).expect("adapter");
        let (pos, _nrm, idx) = quad();
        let big: Vec<[f32; 3]> = pos
            .iter()
            .map(|p| [p[0] * 20.0, p[1] * 20.0, 0.0])
            .collect();
        e.add_mesh(&big, None, &idx, [0.1, 0.6, 0.2, 1.0], [0.0; 3], 1.0, 0.0)
            .expect("ground");
        e.set_map(Some((0.5, 0.0, 0.5, 1.0)), 40.0).expect("map");
        let t0 = e.camera_pose();
        // click at the minimap upper-left quarter = world (target + (-0.25*2*zoom_x, +0.25*2*zoom_y))
        // rect=(100,0,100,100); local click (25,75) -> NDC (-0.5,+0.5) -> dworld (-40, +20*aspect... per the ortho definition)
        e.navigate_click(125.0, 75.0, 0).expect("nav teleport");
        let t1 = e.camera_pose();
        assert!(
            (t1.target[0] - t0.target[0]).abs() > 1.0,
            "target must move got {t1:?}"
        );
        assert!((t1.target[2]).abs() < 1e-9, "lands at z=0");
        assert!(
            (t1.dist - t0.dist).abs() < 1e-12 && (t1.yaw - t0.yaw).abs() < 1e-12,
            "distance and angle kept [ruling e]"
        );
        // a click outside the region = rejected with zero change
        let t2 = e.camera_pose();
        assert!(
            e.navigate_click(50.0, 50.0, 0).is_err(),
            "main-region clicks through this entry must reject"
        );
        let t3 = e.camera_pose();
        assert!((t3.target[0] - t2.target[0]).abs() < 1e-12);
        // timed flight: done convergence + pose = target
        e.navigate_click(175.0, 25.0, 1).expect("nav fly dur=1ms");
        assert!(!e.fly_state_done() || e.camera_pose().target != t3.target);
        let mut guard = 0;
        while !e.fly_state_done() && guard < 200 {
            let _ = e.render();
            std::thread::sleep(std::time::Duration::from_millis(5));
            guard += 1;
        }
        assert!(e.fly_state_done(), "flight must arrive");
    }

    // spec: CAPI-13
    #[test]
    fn hide_filters_pick_keeps_enumeration() {
        let mut e = eng();
        // the mount fit chooses the framing, so the "hit somewhere" point is
        // swept, not assumed center (the fit moved twoprim off the old center)
        let hit = (0..120u32)
            .step_by(2)
            .flat_map(|y| (0..160u32).step_by(2).map(move |x| (x, y)))
            .find_map(|(x, y)| e.pick(x as f32, y as f32).map(|h| (h, x, y)))
            .expect("twoprim pickable after fit (CAPI-04 family)")
            .0;
        let h = enc_entity(hit);
        assert_eq!(
            e.set_visible(h, true),
            Ok(()),
            "re-setting within the default-visible domain = idempotent"
        );
        assert_eq!(e.set_visible(h, false), Ok(()));
        assert_eq!(
            e.set_visible(h, false),
            Ok(()),
            "repeated hide is idempotent"
        );
        assert_ne!(
            e.pick(80.0, 60.0).map(enc_entity).unwrap_or(u64::MAX),
            h,
            "after hiding, the center must not hit the body itself"
        );
        assert_eq!(e.entity_count(), 2, "enumeration domain unchanged");
        assert!(
            (0..2u32).any(|k| e.entity_at(k) == Some(hit)),
            "hidden items are still enumerable"
        );
        assert_eq!(e.is_visible(h), Some(false));
    }

    // spec: CAPI-14
    #[test]
    fn visible_getter_roundtrip_and_unknown() {
        let mut e = eng();
        let h0 = enc_entity(e.entity_at(0).unwrap());
        assert_eq!(e.is_visible(h0), Some(true), "visible by default");
        assert_eq!(e.set_visible(h0, false), Ok(()));
        assert_eq!(e.is_visible(h0), Some(false));
        assert_eq!(
            e.is_visible(u64::MAX >> 1),
            None,
            "unknown shape = None (FFI layer projects -1)"
        );
    }

    // spec: CAPI-15
    #[test]
    fn add_mesh_enumeration_and_degenerate_no_partial() {
        let mut e = eng();
        let (pos, nrm, idx) = quad();
        let h = e
            .add_mesh(
                &pos,
                Some(&nrm),
                &idx,
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0],
                1.0,
                0.0,
            )
            .expect("add ok");
        assert_ne!(h, 0, "0 = failure sentinel, success must give a shape");
        assert_eq!(e.entity_count(), 3);
        assert!(
            e.render().is_ok(),
            "an added piece enters the command stream"
        );
        // degenerate three paths: index out of range / empty geometry / normals length mismatch -- all with no partial commit
        assert!(
            e.add_mesh(&pos, Some(&nrm), &[0, 9, 2], [1.0; 4], [0.0; 3], 1.0, 0.0)
                .is_err()
        );
        assert!(
            e.add_mesh(&[], None, &[], [1.0; 4], [0.0; 3], 1.0, 0.0)
                .is_err()
        );
        assert!(
            e.add_mesh(
                &pos,
                Some(&[[0.0, 0.0, 1.0]; 3]),
                &idx,
                [1.0; 4],
                [0.0; 3],
                1.0,
                0.0
            )
            .is_err()
        );
        assert_eq!(e.entity_count(), 3, "degenerate = zero commit");
        assert!(e.remove_entity(h).is_ok());
        assert_eq!(e.entity_count(), 2);
    }

    // spec: CAPI-16
    #[test]
    fn remove_reentry_and_aba_isolation() {
        let mut e = eng();
        let h0 = enc_entity(e.entity_at(0).unwrap());
        assert_eq!(e.remove_entity(h0), Ok(()));
        assert_eq!(e.entity_count(), 1);
        assert!(
            e.remove_entity(h0).is_err(),
            "old handle re-entry = VE_ERR_ARG scheme"
        );
        assert_eq!(
            e.is_visible(h0),
            None,
            "a query on a deleted shape rejects the same way"
        );
        let (pos, nrm, idx) = quad();
        let h_new = e
            .add_mesh(&pos, Some(&nrm), &idx, [1.0; 4], [0.0; 3], 1.0, 0.0)
            .expect("re-spawn in the same slot: generation+1, new shape");
        assert_ne!(
            h_new, h0,
            "ABA: an old handle never collides into the new generation"
        );
        assert!(
            e.remove_entity(h0).is_err(),
            "the new entity is present, the old handle stays dead"
        );
        assert_eq!(e.remove_entity(h_new), Ok(()));
    }
}

#[cfg(test)]
mod point_pick_tests {
    //! B2 REND-38/39 engine-level integration (super-band B2): add_points -> pick hits a point-cloud entity.
    //! The mesh-priority construction guarantee (REND-39): with a mesh hit the point-cloud path stays untouched = existing pick tests serve as canary.
    use super::*;
    use crate::ffi::enc_entity;

    fn eng() -> Engine {
        Engine::new_headless(320, 240).expect("adapter")
    }

    fn ve_point(pos: [f32; 3]) -> crate::ffi::VePointMark {
        crate::ffi::VePointMark {
            pos,
            radius_px: 3.0,
            color: [0.8, 0.8, 0.8],
        }
    }

    /// Top-view camera position (pitch=90°): target = origin, points project to the screen center.
    fn rig_over(e: &mut Engine) {
        let _ = e.fly_to(
            [0.0, 0.0, 0.0],
            0.0,
            std::f64::consts::FRAC_PI_2,
            10.0,
            5.0,
            1.0,
            0,
        );
    }

    // spec: REND-38
    #[test]
    fn pick_hits_point_cloud_entity() {
        let mut e = eng();
        let bits = e.add_points(&[ve_point([0.0, 0.0, 0.0])]).expect("add");
        rig_over(&mut e);
        let hit = e.pick(160.0, 120.0);
        assert_eq!(
            hit.map(enc_entity),
            Some(bits),
            "center pick must hit point cloud"
        );
    }

    // spec: REND-38
    #[test]
    fn pick_misses_far_corner() {
        let mut e = eng();
        e.add_points(&[ve_point([0.0, 0.0, 0.0])]).expect("add");
        rig_over(&mut e);
        assert!(
            e.pick(2.0, 2.0).is_none(),
            "corner far from point must miss"
        );
    }

    // spec: REND-38
    #[test]
    fn hidden_cloud_not_pickable() {
        let mut e = eng();
        let bits = e.add_points(&[ve_point([0.0, 0.0, 0.0])]).expect("add");
        rig_over(&mut e);
        e.set_visible(bits, false).expect("hide");
        assert!(e.pick(160.0, 120.0).is_none(), "hidden cloud must not pick");
    }

    // spec: REND-38
    #[test]
    fn clip_negative_side_excluded() {
        let mut e = eng();
        e.add_points(&[ve_point([0.0, 0.0, 0.0])]).expect("add");
        rig_over(&mut e);
        // Plane [n=(0,0,1), d=-0.5]: keeps z >= 0.5 → point at z=0 excluded.
        e.set_clips(&[[0.0, 0.0, 1.0, -0.5]]).expect("clip");
        assert!(
            e.pick(160.0, 120.0).is_none(),
            "clipped-out point must not be pickable (keeps-negative mirror)"
        );
    }

    // spec: REND-39
    #[test]
    fn mesh_wins_over_points() {
        let mut e = eng();
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
        ];
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        let mesh_bits = e
            .add_mesh(
                &pos,
                None,
                &idx,
                [0.9, 0.5, 0.3, 1.0],
                [0.0, 0.0, 0.0],
                1.0,
                0.0,
            )
            .expect("mesh");
        e.add_points(&[ve_point([2.0, 2.0, 0.0])]).expect("pts");
        rig_over(&mut e);
        let hit = e.pick(160.0, 120.0).map(enc_entity);
        assert_eq!(
            hit,
            Some(mesh_bits),
            "mesh hit must win (REND-39 mesh-first)"
        );
    }

    // spec: REND-38
    #[test]
    fn cloud_offset_screen_position_hits() {
        let mut e = eng();
        // No mesh: mesh-miss fallback path. Point at world (2,2,0), yaw=0 pitch=90°
        // top-down → +x=right, +y=up; zoom=5 → 32 px/unit on 320×240 → screen (224,56).
        let bits = e.add_points(&[ve_point([2.0, 2.0, 0.0])]).expect("pts");
        rig_over(&mut e);
        // Probe-measured projection (PIT-8: probe first, assert second — yaw=0/pitch=90°
        // axis orientation is NOT aligned to naive +x=right): point lands at (200,72).
        let hit = e.pick(200.0, 72.0).map(enc_entity);
        assert_eq!(hit, Some(bits), "off-center cloud hits at its probed px");
        // Center stays empty (6px radius ≪ 40px real offset).
        assert!(e.pick(160.0, 120.0).is_none(), "center must not bleed-hit");
    }
}

#[cfg(test)]
mod fit_probe {
    use super::*;
    const TP: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/data/twoprim.glb"
    );
    #[test]
    fn fit_probe_print() {
        let mut e = Engine::new_headless(160, 120).expect("adapter");
        let n = e.load_gltf(TP).unwrap();
        println!("entities={n}");
        println!("rig eye={:?} target={:?}", e.rig.eye(), e.rig.target);
        let mut hits = 0;
        let mut first = None;
        for y in (0..120u32).step_by(4) {
            for x in (0..160u32).step_by(4) {
                if e.pick(x as f32, y as f32).is_some() {
                    hits += 1;
                    first = first.or(Some((x, y)));
                }
            }
        }
        println!("hits={hits} first={first:?}");
    }
}
