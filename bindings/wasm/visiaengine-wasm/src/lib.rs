//! wasm32-exclusive crate (native `cargo test --workspace` compiles it as an empty shell).
//! JS/wasm face (batch 7 route A): a **pure-marshaling thin leaf** above Engine -- any policy logic
//! appearing there is a violation ([FFI-R:BS-5] dual-face single-source discipline); the constant/error-code getters are all
//! JS mirrors of the Rust single source (CAPI-09 machine reconciliation).

#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

use visiaengine::{Engine, enc_entity};
use visiaengine::{
    KIND_KEY, KIND_NO_SUCH, KIND_PTR_DOWN, KIND_PTR_MOVE, KIND_PTR_UP, KIND_WHEEL, MISS,
    VE_ERR_ARG, VE_ERR_IO, VE_ERR_PANIC, VE_ERR_SIZE, VE_ERR_STATE, visiaengine_abi_version,
};

/// Engine JS handle (GC reclaim = Drop = destruct; the no-free-function discipline is guaranteed naturally by the lifecycle on the wasm side).
#[wasm_bindgen]
pub struct VisiaEngine {
    inner: Engine,
}

#[wasm_bindgen]
impl VisiaEngine {
    /// Async factory: the canvas size carries its own attach (Web GPU construction has no synchronous path -- Promise native
    /// [FFI-R:v13-FEAS-2]). Failure (no adapter) = null.
    #[wasm_bindgen(js_name = fromCanvas)]
    pub async fn from_canvas(canvas: &web_sys::HtmlCanvasElement) -> Option<VisiaEngine> {
        Engine::new_canvas(canvas)
            .await
            .map(|inner| VisiaEngine { inner })
    }

    #[wasm_bindgen(js_name = loadGltfBytes)]
    pub fn load_gltf_bytes(&mut self, data: &[u8]) -> i32 {
        match self.inner.load_gltf_bytes(data) {
            Ok(_) => 0,
            Err(_) => VE_ERR_IO,
        }
    }

    #[wasm_bindgen(js_name = loadGeoJsonBytes)]
    pub fn load_geojson_bytes(&mut self, data: &[u8]) -> i32 {
        match self.inner.load_geojson_bytes(data) {
            Ok(_) => 0,
            Err(_) => VE_ERR_IO,
        }
    }

    /// Pull-model one frame (called inside the host rAF).
    pub fn render(&mut self) -> i32 {
        match self.inner.render() {
            Ok(()) => 0,
            Err(_) => VE_ERR_STATE,
        }
    }

    /// The sole input entry (flat parameters -- VeInput never crosses the wasm boundary, the struct_size issue dissolves naturally).
    pub fn input(&mut self, kind: u32, px: f32, py: f32, wheel: f32) -> i32 {
        match self.inner.apply_input(kind, px, py, wheel) {
            Some(consumed) => i32::from(consumed),
            None => 0,
        }
    }

    pub fn pick(&mut self, px: f32, py: f32) -> u64 {
        self.inner.pick(px, py).map_or(MISS, enc_entity)
    }

    /// Pick with the world-space hit point: [entityAsF64, x, y, z] (entity
    /// handles are small integers, exact in f64; empty = miss). The E203
    /// measuring family's wasm face.
    #[wasm_bindgen(js_name = pickPoint)]
    pub fn pick_point(&mut self, px: f32, py: f32) -> Box<[f64]> {
        match self.inner.pick_hit(px, py) {
            Some((id, pt)) => Box::new([enc_entity(id) as f64, pt[0], pt[1], pt[2]]),
            None => Box::new([]),
        }
    }

    #[wasm_bindgen(js_name = entityAt)]
    pub fn entity_at(&mut self, index: u32) -> u64 {
        self.inner.entity_at(index).map_or(MISS, enc_entity)
    }

    #[wasm_bindgen(js_name = entityCount)]
    pub fn entity_count(&self) -> i32 {
        self.inner.entity_count() as i32
    }

    #[wasm_bindgen(js_name = viewport)]
    pub fn viewport(&mut self, w: u32, h: u32) -> i32 {
        if w == 0 || h == 0 {
            return VE_ERR_SIZE;
        }
        self.inner.resize(w, h);
        0
    }

    // -- B1 data band mirror (CAPI-13..16 pure-marshaling thin leaves [FFI-R:BS-5]) --
    #[wasm_bindgen(js_name = setEntityVisible)]
    pub fn set_entity_visible(&mut self, entity: u64, visible: bool) -> i32 {
        match self.inner.set_visible(entity, visible) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    #[must_use]
    #[wasm_bindgen(js_name = entityVisible)]
    pub fn entity_visible(&self, entity: u64) -> i32 {
        match self.inner.is_visible(entity) {
            Some(true) => 1,
            Some(false) => 0,
            None => -1,
        }
    }

    /// Post-effect chain (flat triplets [kind,p1,p2,...]): 0=bloom 1=outline
    /// 2=tonemap 3=ssao 4=haze; empty = clear. 0 = ok, -1 = rejected.
    #[wasm_bindgen(js_name = setPostEffects)]
    pub fn set_post_effects(&mut self, flat: &[f64]) -> i32 {
        match self.inner.set_post_effects(flat) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// Host-set clear color (srgb 0..1 rgb; negative r = restore the default).
    #[wasm_bindgen(js_name = setClearColor)]
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64) -> i32 {
        if r < 0.0 {
            self.inner.set_clear_color(None);
        } else {
            self.inner.set_clear_color(Some([r, g, b]));
        }
        0
    }

    /// PCSS shadows from a light direction; all-zero/negative-x = off.
    #[wasm_bindgen(js_name = setShadow)]
    pub fn set_shadow(&mut self, lx: f64, ly: f64, lz: f64) -> i32 {
        if lx < 0.0 || (lx == 0.0 && ly == 0.0 && lz == 0.0) {
            self.inner.set_shadow(None);
        } else {
            self.inner.set_shadow(Some([lx, ly, lz]));
        }
        0
    }

    /// EDL eye-dome lighting (negative strength = off).
    #[wasm_bindgen(js_name = setEdl)]
    pub fn set_edl(&mut self, strength: f64) -> i32 {
        if strength < 0.0 {
            self.inner.set_edl(None);
        } else {
            self.inner.set_edl(Some(strength));
        }
        0
    }

    /// Discrete projection switch (N2.1 seam): true = persp, false = ortho; clears the morph envelope.
    #[wasm_bindgen(js_name = setProjection)]
    pub fn set_projection(&mut self, persp: bool) -> i32 {
        self.inner.set_projection(if persp {
            visiaengine::Proj::Persp
        } else {
            visiaengine::Proj::Ortho
        });
        0
    }

    /// Current discrete mode (true = persp). Morph outranks when active.
    #[wasm_bindgen(js_name = getProjection)]
    pub fn get_projection(&self) -> i32 {
        i32::from(self.inner.projection_is_persp())
    }

    // -- N2.1 projection-morph mirror (REND-41 thin leaf) --
    /// Continuous projection morph, t clamps into [0,1] (NaN -> 0); 0 = ortho bytes, 1 = persp bytes.
    #[wasm_bindgen(js_name = setMorphTime)]
    pub fn set_morph_time(&mut self, t: f64) -> i32 {
        self.inner.set_morph_time(Some(t));
        0
    }

    /// Back to the legacy discrete projection mode (morph off).
    #[wasm_bindgen(js_name = clearMorph)]
    pub fn clear_morph(&mut self) -> i32 {
        self.inner.set_morph_time(None);
        0
    }
    // -- B2 clip band mirror (CAPI-20 pure-marshaling thin leaf [FFI-R:BS-5]) --
    /// Flat 4n f64 coefficient face ([] = the only clearing form); bounds/angle misalignment/degenerate = -1 = same scheme as the C face.
    #[wasm_bindgen(js_name = setClips)]
    pub fn set_clips(&mut self, planes: &[f64]) -> i32 {
        if planes.len() % 4 != 0 {
            return -1;
        }
        let arr: Vec<[f64; 4]> = planes
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        match self.inner.set_clips(&arr) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// Read back flat 4n (post-normalization unit form = same scheme as the C face).
    #[must_use]
    #[wasm_bindgen(js_name = getClips)]
    pub fn get_clips(&self) -> Vec<f64> {
        self.inner.clips().into_iter().flatten().collect()
    }

    // -- 5a flight band mirror (CAPI-23/24 thin leaves) --
    /// Takeoff/reroute: pose flat 8 = [target3, yaw, pitch, dist, zoom, fov]; durMs = 0 is the teleport form.
    /// 0 = takeoff, -1 = domain rejection (same as the C face).
    #[wasm_bindgen(js_name = flyTo)]
    pub fn fly_to(&mut self, pose: &[f64], dur_ms: u32) -> i32 {
        if pose.len() != 8 {
            return -1;
        }
        match self.inner.fly_to(
            [pose[0], pose[1], pose[2]],
            pose[3],
            pose[4],
            pose[5],
            pose[6],
            pose[7],
            u64::from(dur_ms),
        ) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// State: 1 = done (includes idle) / 0 = flying (same scheme as the C face; no handle error domain).
    #[must_use]
    #[wasm_bindgen(js_name = flyState)]
    pub fn fly_state(&self) -> i32 {
        if self.inner.fly_state_done() { 1 } else { 0 }
    }

    /// Progress [0,1] (done/idle = 1.0 single-master value domain, same scheme as the C face's merged "done does not write out" semantics).
    #[must_use]
    #[wasm_bindgen(js_name = flyProgress)]
    pub fn fly_progress(&self) -> f64 {
        self.inner.fly_progress().unwrap_or(1.0)
    }

    // -- S2 text band mirror (CAPI-21/22 thin leaves) --
    /// Font injection (bytes = full TTF/OTF form; replacement-style). 0 = success, negative = error code (same as the C face).
    #[wasm_bindgen(js_name = loadFont)]
    pub fn load_font(&mut self, data: &[u8]) -> i32 {
        match self.inner.set_font(data) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// World-anchored label: pos3 + srgb color 4 + utf8 text; success = shape bigint (0 is legal = CAPI-01 web projection),
    /// rejection/failure = MISS (0xFFFF...) same scheme as addMesh.
    #[must_use]
    #[wasm_bindgen(js_name = addLabel)]
    pub fn add_label(&mut self, pos: &[f64], color: &[f32], text: &str, size_px: f32) -> u64 {
        if pos.len() != 3 || color.len() != 4 {
            return crate::MISS;
        }
        let c = [color[0], color[1], color[2], color[3]];
        match self
            .inner
            .add_label([pos[0], pos[1], pos[2]], text, c, size_px)
        {
            Ok(bits) => bits,
            Err(_) => crate::MISS,
        }
    }
    // -- 5b second wave minimap mirror (CAPI-25..27 thin leaves) --
    /// CAPI-25 open the minimap: normalized rect fx/fy/fw/fh in [0,1] (fw/fh>0) + zoom>0; 0 = on, -1 = domain rejection. Close via clearMap().
    #[wasm_bindgen(js_name = setMap)]
    pub fn set_map(&mut self, fx: f32, fy: f32, fw: f32, fh: f32, zoom: f64) -> i32 {
        match self.inner.set_map(Some((fx, fy, fw, fh)), zoom) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-25 close the minimap (back to the old single-frame full-face path). 0 = success.
    #[wasm_bindgen(js_name = clearMap)]
    pub fn clear_map(&mut self) -> i32 {
        match self.inner.set_map(None, 0.0) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    // -- P1 tile mirror (CAPI-28/29 thin leaves) --
    /// CAPI-28 mount an MVT tile directory (wasm = path semantics sandbox-limited; for local tests).
    /// Return = mounted tile count; negative = rejected.
    #[wasm_bindgen(js_name = loadMvtDir)]
    pub fn load_mvt_dir(&mut self, path: &str, z: u32) -> i32 {
        match u8::try_from(z) {
            Ok(z) => match self.inner.load_mvt_dir(path, z) {
                Ok(n) => n as i32,
                Err(_) => -1,
            },
            Err(_) => -1,
        }
    }

    /// CAPI-36..38 typed property diff-update (N2.4). 0=ok; -1=rejected
    /// (type-conflict / unknown entity / empty name).
    #[wasm_bindgen(js_name = updateEntityAttrF64)]
    pub fn update_entity_attr_f64(&mut self, entity: u64, name: &str, v: f64) -> i32 {
        match self.inner.update_entity_props_f64(entity, name, v) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    #[wasm_bindgen(js_name = updateEntityAttrStr)]
    pub fn update_entity_attr_str(&mut self, entity: u64, name: &str, v: &str) -> i32 {
        match self
            .inner
            .update_entity_props_str(entity, name, v.to_string())
        {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    #[wasm_bindgen(js_name = updateEntityAttrBool)]
    pub fn update_entity_attr_bool(&mut self, entity: u64, name: &str, v: bool) -> i32 {
        match self.inner.update_entity_props_bool(entity, name, v) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-35 HTTP tile source (N1.4, discovery-driven; the wasm-side
    /// HttpSource is a typed stub — real bytes flow through host JS fetch;
    /// this bridge pre-registers the domain-reject shape). 0 = mounted; <0 = rejected.
    #[wasm_bindgen(js_name = setTileSourceHttp)]
    pub fn set_tile_source_http(&mut self, url: &str, z: u32) -> i32 {
        match u8::try_from(z) {
            Ok(z) => match self.inner.load_mvt_http(url, z) {
                Ok(_) => 0,
                Err(_) => -1,
            },
            Err(_) => -1,
        }
    }

    /// CAPI-29 viewport bbox (3857 meters) -> scheduler visible->ensure. Return = visible count; negative = rejected.
    #[wasm_bindgen(js_name = setTileView)]
    pub fn set_tile_view(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> i32 {
        match self.inner.set_tile_view(min_x, min_y, max_x, max_y) {
            Ok(n) => n as i32,
            Err(_) => -1,
        }
    }

    // -- R band raster mirror (CAPI-39/40 thin leaves; tiled quad = non-terrain) --
    /// CAPI-39: mount a PNG raster tile directory. Return = mounted count; negative = rejected.
    /// Single MVT tile bytes injection (the wasm face; 0 = mounted, negative = rejected).
    #[wasm_bindgen(js_name = loadTileBytes)]
    pub fn load_tile_bytes(&mut self, z: u32, x: u32, y: u32, data: &[u8]) -> i32 {
        match (
            u8::try_from(z),
            self.inner.load_tile_bytes(z as u8, x, y, data),
        ) {
            (Ok(_), Ok(n)) => n as i32,
            _ => -1,
        }
    }

    /// Single raster-tile bytes injection (0 = mounted quads, negative = rejected).
    #[wasm_bindgen(js_name = loadRasterBytes)]
    pub fn load_raster_bytes(&mut self, z: u32, x: u32, y: u32, data: &[u8]) -> i32 {
        match (
            u8::try_from(z),
            self.inner.load_raster_bytes(z as u8, x, y, data),
        ) {
            (Ok(_), Ok(n)) => n as i32,
            _ => -1,
        }
    }

    #[wasm_bindgen(js_name = loadRasterDir)]
    pub fn load_raster_dir(&mut self, path: &str, z: u32) -> i32 {
        match u8::try_from(z) {
            Ok(z) => match self.inner.load_raster_dir(path, z) {
                Ok(n) => n as i32,
                Err(_) => -1,
            },
            Err(_) => -1,
        }
    }

    /// CAPI-40: raster-layer viewport bbox. Return = visible mounted tile count; negative = rejected.
    #[wasm_bindgen(js_name = setRasterView)]
    pub fn set_raster_view(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> i32 {
        match self.inner.set_raster_view(min_x, min_y, max_x, max_y) {
            Ok(n) => n as i32,
            Err(_) => -1,
        }
    }

    /// CAPI-42: write the node-local transform (column-major 16 f64). 0 = success; negative = rejected.
    #[wasm_bindgen(js_name = setNodeTransform)]
    pub fn set_node_transform(&mut self, entity: u64, m: &[f64]) -> i32 {
        if m.len() != 16 {
            return -1;
        }
        let mut mat = [[0.0f64; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                mat[r][c] = m[r * 4 + c];
            }
        }
        match self.inner.set_node_transform(entity, mat) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-43 mirror: node-local transform readback as 16 values (same
    /// row-major flatten the C face writes). wasm has no out-parameter channel
    /// — `&mut Vec<f64>` does not even compile for the wasm32 target — so the
    /// C form's "negative code + untouched out buffer" maps here to an EMPTY
    /// return (precedent: getCameraPose returns a Vec, no code).
    #[wasm_bindgen(js_name = getNodeTransform)]
    pub fn get_node_transform(&self, entity: u64) -> Vec<f64> {
        match self.inner.get_node_transform(entity) {
            Ok(mat) => mat.iter().flat_map(|row| row.iter()).copied().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// CAPI-41: .hdr environment load (SH-9 irradiance; demo grade). 0 = success; negative = rejected.
    #[wasm_bindgen(js_name = loadEnvHdr)]
    pub fn load_env_hdr(&mut self, path: &str) -> i32 {
        match self.inner.load_env_hdr(path) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// Bytes form of the HDR environment (the web face has no filesystem).
    #[wasm_bindgen(js_name = loadEnvHdrBytes)]
    pub fn load_env_hdr_bytes(&mut self, data: &[u8]) -> i32 {
        match self.inner.load_env_hdr_bytes(data) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    // -- Scene-tree mirror (CAPI-30..34 thin leaves) --
    /// CAPI-30: create a group node (returns the handle; 0 = err). name = NULL is passed as an empty string.
    #[wasm_bindgen(js_name = createGroup)]
    pub fn create_group(&mut self, name: &str) -> u64 {
        match self
            .inner
            .create_group(if name.is_empty() { None } else { Some(name) })
        {
            Ok(bits) => bits,
            Err(_) => 0,
        }
    }

    /// CAPI-31: reparent. parent = 0 detaches back to root. 0 = success, -1 = rejected.
    #[wasm_bindgen(js_name = setParent)]
    pub fn set_parent(&mut self, child: u64, parent: u64) -> i32 {
        match self.inner.set_parent(child, parent) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-32: query the parent node (0 = root/err).
    #[wasm_bindgen(js_name = getParent)]
    pub fn get_parent(&self, entity: u64) -> u64 {
        self.inner.get_parent(entity).unwrap_or(0)
    }

    /// CAPI-33: set the group offset. 0 = success, -1 = rejected.
    #[wasm_bindgen(js_name = setGroupOffset)]
    pub fn set_group_offset(&mut self, group: u64, dx: f64, dy: f64, dz: f64) -> i32 {
        match self.inner.set_group_offset(group, dx, dy, dz) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-34: read the group offset (returns [dx,dy,dz]; null on failure).
    #[wasm_bindgen(js_name = getGroupOffset)]
    pub fn get_group_offset(&self, group: u64) -> Option<wasm_bindgen::JsValue> {
        match self.inner.get_group_offset(group) {
            Ok(o) => Some(
                js_sys::Array::of3(
                    &js_sys::Number::from(o[0]),
                    &js_sys::Number::from(o[1]),
                    &js_sys::Number::from(o[2]),
                )
                .into(),
            ),
            Err(_) => None,
        }
    }

    /// CAPI-26 click navigation: top view picks an entity first -> ground fallback; the main rig keeps angle and distance while swapping the target; dur_ms = 0 teleports.
    #[wasm_bindgen(js_name = navigateClick)]
    pub fn navigate_click(&mut self, px: f32, py: f32, dur_ms: u32) -> i32 {
        match self.inner.navigate_click(px, py, u64::from(dur_ms)) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    /// CAPI-27 main camera pose readback: flat 8 = [target3, yaw, pitch, dist, zoom, fov] (same shape as flyTo).
    #[must_use]
    #[wasm_bindgen(js_name = getCameraPose)]
    pub fn get_camera_pose(&self) -> Vec<f64> {
        let r = self.inner.camera_pose();
        vec![
            r.target[0],
            r.target[1],
            r.target[2],
            r.yaw,
            r.pitch,
            r.dist,
            r.zoom,
            r.fov_y,
        ]
    }

    /// Flat triple face (js arrays = call-time copy, bindgen native); degenerate = the 0 sentinel is not allowed, same scheme as the C face.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    #[wasm_bindgen(js_name = addMesh)]
    pub fn add_mesh(
        &mut self,
        positions: &[f32],
        normals: &[f32],
        indices: &[u32],
        base_color: &[f32],
        origin: &[f64],
        roughness: f32,
        metalness: f32,
    ) -> u64 {
        if positions.len() % 3 != 0 || base_color.len() != 4 || origin.len() != 3 {
            return 0;
        }
        let pos: Vec<[f32; 3]> = positions
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        let nrm: Option<Vec<[f32; 3]>> = if normals.is_empty() {
            None
        } else if normals.len() != positions.len() {
            return 0;
        } else {
            Some(
                normals
                    .chunks_exact(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect(),
            )
        };
        let col = [base_color[0], base_color[1], base_color[2], base_color[3]];
        let org = [origin[0], origin[1], origin[2]];
        // the js-face sentinel = MISS(MAX): 0 is a legal shape and cannot be occupied (the web projection of the CAPI-01 division)
        // N6: roughness<=0 → 1.0 dielectric default (C-face zero-init hosts same law)
        let rough = if roughness <= 0.0 { 1.0 } else { roughness };
        match self
            .inner
            .add_mesh(&pos, nrm.as_deref(), indices, col, org, rough, metalness)
        {
            Ok(h) => h,
            Err(_) => u64::MAX,
        }
    }

    /// CAPI-18 mirror: flat point-cloud array (8 floats per point = pos x3 + radius_px + color x3, sRGB host face);
    /// degenerate/failure = MISS(MAX) same scheme as addMesh (shape 0 is legal and must not be occupied -- CAPI-01 web projection).
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    #[wasm_bindgen(js_name = addPoints)]
    pub fn add_points(&mut self, marks: &[f32]) -> u64 {
        if marks.len() % 8 != 0 || marks.is_empty() {
            return u64::MAX;
        }
        let raw: Vec<visiaengine::VePointMark> = marks
            .chunks_exact(8)
            .map(|m| visiaengine::VePointMark {
                pos: [m[0], m[1], m[2]],
                radius_px: m[3],
                color: [m[4], m[5], m[6]],
            })
            .collect();
        match self.inner.add_points(&raw) {
            Ok(h) => h,
            Err(_) => u64::MAX,
        }
    }

    /// CAPI-17 mirror: JS callback `(event, a, b) => void`; null = unregister. The u64 count face stays in the
    /// progress domain below 2^53 (element counts), lossless as a direct f64 -- the bigint discipline refers only to handle shapes.
    #[wasm_bindgen(js_name = setEventCallback)]
    pub fn set_event_callback(&mut self, cb: Option<js_sys::Function>) {
        match cb {
            Some(f) => self.inner.set_event_fn(Some(Box::new(move |ev, a, b| {
                let _ = f.call3(
                    &JsValue::NULL,
                    &JsValue::from(ev),
                    &JsValue::from_f64(f64::from(u32::try_from(a).unwrap_or(u32::MAX))),
                    &JsValue::from_f64(f64::from(u32::try_from(b).unwrap_or(u32::MAX))),
                );
            }))),
            None => self.inner.set_event_fn(None),
        }
    }

    /// CAPI-19 mirror: bytes form (a path cannot cross wasm -- host-side reads like load_geojson_bytes).
    /// The report does not cross the boundary for now (v0 hosts judge failure causes by the kept==0 sentinel MISS; the d.ts note rides the mirror test).
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    #[wasm_bindgen(js_name = loadPclBytes)]
    pub fn load_pcl_bytes(&mut self, bytes: &[u8], lenient: bool) -> u64 {
        match self.inner.load_pcl_bytes(bytes, lenient) {
            Ok((h, _)) => h,
            Err(_) => u64::MAX,
        }
    }

    #[wasm_bindgen(js_name = removeEntity)]
    pub fn remove_entity(&mut self, entity: u64) -> i32 {
        match self.inner.remove_entity(entity) {
            Ok(()) => 0,
            Err(_) => -1,
        }
    }

    #[wasm_bindgen(js_name = abiVersion)]
    pub fn abi_version() -> u32 {
        visiaengine_abi_version()
    }
}

// ===== Constant mirror getters (Rust single source; CAPI-09 value-reconciliation anchors) =====
// Plain fn exports (wasm-bindgen does not support const fn attribute macros -- measured rejection; the values remain compile-time constant references, the single source unchanged)
#[wasm_bindgen]
#[must_use]
pub fn kind_ptr_move() -> u32 {
    KIND_PTR_MOVE
}
#[wasm_bindgen]
#[must_use]
pub fn kind_ptr_down() -> u32 {
    KIND_PTR_DOWN
}
#[wasm_bindgen]
#[must_use]
pub fn kind_ptr_up() -> u32 {
    KIND_PTR_UP
}
#[wasm_bindgen]
#[must_use]
pub fn kind_wheel() -> u32 {
    KIND_WHEEL
}
#[wasm_bindgen]
#[must_use]
pub fn kind_key() -> u32 {
    KIND_KEY
}
#[wasm_bindgen]
#[must_use]
pub fn kind_no_such() -> u32 {
    KIND_NO_SUCH
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_arg() -> i32 {
    VE_ERR_ARG
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_state() -> i32 {
    VE_ERR_STATE
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_io() -> i32 {
    VE_ERR_IO
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_panic() -> i32 {
    VE_ERR_PANIC
}
#[wasm_bindgen]
#[must_use]
pub fn ve_err_size() -> i32 {
    VE_ERR_SIZE
}
