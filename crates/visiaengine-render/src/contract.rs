//! Render contract: backend trait + frame IR (pure data, zero backend types -- invariant 2).
//! Camera math lives in the sibling module `camera` (REND-10..16).
//! Clauses: docs/sdd/render.md REND-01..05.

/// Mesh resource identifier (opaque; the real resource table belongs to a later slice).
pub type MeshId = u64;

/// Material resource identifier.
pub type MaterialId = u64;

/// GPU texture resource identifier (WGPU-14).
pub type TextureId = u64;

/// GPU instance-table resource identifier (REND-27, 4c).
pub type InstanceId = u64;
/// The GPU resource-creation failure face (v0 carries backend rejection only; device loss such as OOM goes through the callback side).
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("backend resource creation failed: {reason}")]
pub struct BackendError {
    pub reason: String,
}

/// CPU-side mesh upload description (pure borrows; positions/normals equal length is guaranteed by the caller, REND-06).
#[derive(Clone, Copy, Debug)]
pub struct MeshDesc<'a> {
    /// UV channel 0 (WGPU-14; default = empty, the upload end zero-fills -- the Flat pipeline does not read it).
    pub uv: &'a [[f32; 2]],
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub indices: &'a [u32],
}

/// Building instance (REND-27, 4c; scope ruling 4: translate + z extrusion, no rotational affine).
/// Layout pin = byte-identical to WGSL `Instance` 32B ([f32;3] alignment 4 vs vec3 alignment 16
/// coincidentally interlock: offset 0..12 / height 12..16 / color 16..28 / pad 28..32;
/// Pod compile-time guarantee, any divergence goes red -- test_pod_layout_32b double lock).
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    /// origin-local offset (D7: far world coordinates live in DrawCommand::origin, REND-28)
    pub offset: [f32; 3],
    /// z extrusion height (unit-box bottom aligned at z = 0, >= 0)
    pub height: f32,
    /// instance color (multiplied into the material.base_color chain, REND-27 ruling point c: alpha is not instanced)
    pub color: [f32; 3],
    _pad: f32,
}

impl Instance {
    #[must_use]
    pub const fn new(offset: [f32; 3], height: f32, color: [f32; 3]) -> Self {
        Self {
            offset,
            height,
            color,
            _pad: 0.0,
        }
    }
}

/// Instance-table upload description (borrow-form like MeshDesc).
#[derive(Clone, Copy, Debug)]
pub struct InstanceDesc<'a> {
    pub data: &'a [Instance],
}

/// Tile-expansion table handle (REND-30, 4de; same counting domain as InstanceId, the naming expresses the new resource family [ruling point c]).
pub type TableId = u64;

/// Line segment (the WGPU-17 tile-expansion source). Layout = same-width vec4 fields erase CPU/GPU alignment ambiguity [4de R1]:
/// `a[0..16] b[16..32] color[32..44] width_px[44..48]`, the WGSL struct is the same 48B form.
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct StrokeSeg {
    pub a: [f32; 4],
    pub b: [f32; 4],
    pub color: [f32; 3],
    pub width_px: f32,
}

impl StrokeSeg {
    #[must_use]
    pub const fn new(a: [f32; 3], b: [f32; 3], color: [f32; 3], width_px: f32) -> Self {
        Self {
            a: [a[0], a[1], a[2], 0.0],
            b: [b[0], b[1], b[2], 0.0],
            color,
            width_px,
        }
    }
}

/// Label glyph quad (REND-33, S2): **64B = 4 x vec4 triple layout lock** (same system as StrokeSeg).
/// pos/metrics are both entity-local; uv normalized over the io-text atlas. Color = **linear-domain GPU value**
/// (the sRGB->linear conversion lives on the producer face, the CORE-16 chokepoint extended = the label table [CAPI-22 face]).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LabelMark {
    /// (xyz entity-local anchor, w = pad).
    pub pos: [f32; 4],
    /// Linear RGBA.
    pub color: [f32; 4],
    /// Atlas UV (u0, v0, u1, v1).
    pub uv: [f32; 4],
    /// (w_px, h_px, dx right-positive, dy up-positive): the glyph's screen patch relative to the anchor's top-left.
    pub metrics: [f32; 4],
}

impl LabelMark {
    /// Assembler (the explicit field-grouped form; the layout-lock test pins the offsets).
    #[must_use]
    pub const fn new(pos: [f32; 3], color: [f32; 4], uv: [f32; 4], metrics: [f32; 4]) -> Self {
        Self {
            pos: [pos[0], pos[1], pos[2], 0.0],
            color,
            uv,
            metrics,
        }
    }
}

/// Label-table upload description.
#[derive(Clone, Copy, Debug)]
pub struct LabelTableDesc<'a> {
    /// Glyph quad sequence (concatenated layout products, multiple labels = the same table continued).
    pub data: &'a [LabelMark],
}

/// Line-table upload description.
#[derive(Clone, Copy, Debug)]
pub struct StrokeTableDesc<'a> {
    pub data: &'a [StrokeSeg],
}

/// Point marker (the WGPU-18 splat source; 32B in the same shape as the Instance family).
#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct PointMark {
    pub pos: [f32; 3],
    pub radius_px: f32,
    pub color: [f32; 3],
    _pad: f32,
}

impl PointMark {
    #[must_use]
    pub const fn new(pos: [f32; 3], color: [f32; 3], radius_px: f32) -> Self {
        Self {
            pos,
            radius_px,
            color,
            _pad: 0.0,
        }
    }
}

/// Point-table upload description.
#[derive(Clone, Copy, Debug)]
pub struct PointTableDesc<'a> {
    pub data: &'a [PointMark],
}
/// Capability bits (the Tier-matrix hooks, architecture.md 6; the v0 enum keeps it simple).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Mesh3D,
    LineStrip,
    OrthoCamera,
}

/// Physical viewport. The scale factor serves HiDPI (logical/physical sizes separated).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    width: u32,
    height: u32,
    scale_factor: f32,
}

impl Viewport {
    /// `scale_factor` must be > 0 (caller boundary validation; a 0 scale is a host bug).
    #[must_use]
    pub const fn new(width: u32, height: u32, scale_factor: f32) -> Self {
        debug_assert!(scale_factor > 0.0);
        Self {
            width,
            height,
            scale_factor,
        }
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Logical size = physical / scale (winit semantics).
    #[must_use]
    pub const fn logical_size(&self) -> (f32, f32) {
        (
            self.width as f32 / self.scale_factor,
            self.height as f32 / self.scale_factor,
        )
    }
}

/// Viewport rectangle (REND-35, 5b): one scissor-locked region inside the full-surface frame.
/// **does not touch Frame** (ruling a: Frame keeps the full-screen semantics, the 44th fallback wave stays unserved) -- rect rides render-call parameters,
/// each pass carries its own camera while the Frame shares one surface/depth. Coordinates = surface physical pixels, origin at the top-left.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ViewportRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ViewportRect {
    /// Construction validates non-degeneracy (width/height > 0, guaranteed by the caller -- a zero-size rect is a host bug).
    #[must_use]
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Whether a given surface physical pixel is inside (input-routing hit-test, minimal stateless engine form).
    #[must_use]
    pub const fn contains(&self, px: u32, py: u32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    /// In-region local coordinates (relative to the region origin) -- screen_to_ray_* reuses it by passing this rect's size as w/h.
    #[must_use]
    pub const fn local(&self, px: u32, py: u32) -> Option<(f32, f32)> {
        if self.contains(px, py) {
            Some(((px - self.x) as f32, (py - self.y) as f32))
        } else {
            None
        }
    }

    /// Normalized rect -> pixel rect (5b second wave [REND-37]: the set_map host language = normalized, auto-follows resize).
    /// Policy nailed: edges = round (half-cross goes into-one consistently), right/bottom edges min(surface bound), size floor keeps 1
    /// (merges with the render_view_rects scissor max(1)); dirty frac (negative / over 1 / NaN) is clamped into domain without panic.
    #[must_use]
    pub fn from_frac(fx: f32, fy: f32, fw: f32, fh: f32, surface_w: u32, surface_h: u32) -> Self {
        let cl = |v: f32| v.clamp(0.0, 1.0);
        let (fx, fy) = (cl(fx), cl(fy));
        let (fw, fh) = (cl(fw), cl(fh));
        let x0 = (fx * surface_w as f32).round() as i64;
        let y0 = (fy * surface_h as f32).round() as i64;
        let x1 = ((fx + fw) * surface_w as f32).round().min(surface_w as f32) as i64;
        let y1 = ((fy + fh) * surface_h as f32).round().min(surface_h as f32) as i64;
        let w = (x1 - x0).max(1).min(surface_w as i64 - x0.max(0));
        let h = (y1 - y0).max(1).min(surface_h as i64 - y0.max(0));
        Self {
            x: x0.max(0) as u32,
            y: y0.max(0) as u32,
            width: w.max(1) as u32,
            height: h.max(1) as u32,
        }
    }

    /// Full-frame single viewport (the rect form of the old full-screen semantics -- for bit-equal canary use).
    #[must_use]
    pub const fn full(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}

/// Camera projection (the projection side of the unified 2D/2.5D/3D entry; interpolated switching belongs to a later slice).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Camera {
    Ortho {
        half_width: f32,
        half_height: f32,
        near: f32,
        far: f32,
    },
    Perspective {
        fov_y_rad: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
}

impl Camera {
    #[must_use]
    pub const fn ortho(half_width: f32, half_height: f32, near: f32, far: f32) -> Self {
        Self::Ortho {
            half_width,
            half_height,
            near,
            far,
        }
    }

    #[must_use]
    pub const fn perspective(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::Perspective {
            fov_y_rad,
            aspect,
            near,
            far,
        }
    }

    #[must_use]
    pub const fn is_orthographic(&self) -> bool {
        matches!(self, Self::Ortho { .. })
    }
}

/// Render command IR v0 (new primitive = new variant; consumers match exhaustively).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrawCommand {
    ClearColor {
        rgba: [f32; 4],
    },
    DrawMesh {
        mesh: MeshId,
        material: MaterialId,
        /// Layer/model-level world origin (D7-2); the local mesh is relative to it.
        origin: [f64; 3],
        /// origin-local pose, column-major f64 (REND-18).
        transform: [[f64; 4]; 4],
    },
    /// Instanced batch draw (REND-28, 4c): N instances of one mesh x material,
    /// the origin's D7 semantics are the same as DrawMesh (instance offsets are origin-local f32).
    DrawInstances {
        mesh: MeshId,
        material: MaterialId,
        /// Instance-table handle (the create_instances product, REND-27).
        instances: InstanceId,
        origin: [f64; 3],
        transform: [[f64; 4]; 4],
    },
    /// Line-table batch tile-expansion (REND-30, 4de): color/width live in the table, no material attached [ruling point a].
    DrawStrokes {
        table: TableId,
        origin: [f64; 3],
        transform: [[f64; 4]; 4],
    },
    /// Point-marker batch splat (REND-30): the circle mask at the FS, color/radius live in the table.
    DrawPoints {
        table: TableId,
        origin: [f64; 3],
        transform: [[f64; 4]; 4],
    },
    /// Text-label batch patch (REND-33, S2): world-anchored + screen-constant size; color/atlas coordinates live in the table,
    /// no material attached (the tile-expansion family ruling a same scheme); the atlas texture is held by the backend singleton channel (WGPU-24).
    DrawLabels {
        table: TableId,
        origin: [f64; 3],
        transform: [[f64; 4]; 4],
    },
}

impl DrawCommand {
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::ClearColor { .. } => "clear-color",
            Self::DrawMesh { .. } => "draw-mesh",
            Self::DrawInstances { .. } => "draw-instances",
            Self::DrawStrokes { .. } => "draw-strokes",
            Self::DrawPoints { .. } => "draw-points",
            Self::DrawLabels { .. } => "draw-labels",
        }
    }
}

/// One frame's input (the architecture-3 BUILD segment product).
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub viewport: Viewport,
    /// Projection-type discriminator bit (the backend math does not consume it, hosts/demos do, REND-17).
    pub camera: Camera,
    /// Pure-rotation view basis (translation goes through eye via the D7 rebase path, REND-17).
    pub view_rot: [[f32; 4]; 4],
    /// World camera position, f64 (drops to f32 only after subtracting DrawMesh.origin in f64).
    pub eye: [f64; 3],
    pub proj: [[f32; 4]; 4],
    /// 1 screen pixel corresponds to how many world units @ reference depth (REND-29, given by the host; the tile-expansion family width multiplier.
    /// ortho top view = 2*zoom/width exact; perspective = approximate [4de not-installed-1]. The tiled-triangle families do not read it = zero impact).
    pub px_world_scale: f32,
    /// Directional-light shadows (REND-31): None = off and **bitwise zero regression** (the backend dummy early-exit position).
    pub shadow: Option<ShadowSetup>,
    /// Section clipping (REND-32): None / `EMPTY` = keep all and **bitwise zero regression** (the backend count = 0 always-bound early exit).
    pub clip: Option<ClipSetup>,
    /// Eye-dome lighting post-pass (WGPU-33): None = off with byte-identical legacy path (whole pass skipped).
    pub edl: Option<EdlSetup>,
    /// General post-effect chain (WGPU-34): effects compose in Vec order; empty = legacy path (byte-identical).
    pub post: Vec<PostEffect>,
    pub commands: Vec<DrawCommand>,
}

/// Eye-dome lighting configuration (WGPU-33). Semantics: point-cloud
/// silhouette-legibility post-pass — one fullscreen pass, depth-gradient edge
/// pixels darkened by `strength` (clamped at 0.7). Constructor contract:
/// `new` rejects non-finite / non-positive strength (None = explicit reject).
/// Multiview (render_view_rects) ignores this field for now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdlSetup {
    pub strength: f32,
}

impl EdlSetup {
    /// Domain-reject constructor: non-finite / <=0 -> None (explicit reject, no clamp).
    #[must_use]
    pub fn new(strength: f32) -> Option<Self> {
        if strength.is_finite() && strength > 0.0 {
            Some(Self { strength })
        } else {
            None
        }
    }
}

/// General post-processing effect (WGPU-34 / REND-43). Effects compose in
/// `Frame.post` order — each fullscreen pass reads the previous output. Empty
/// vec = byte-identical legacy path (no intermediates, depth Discard).
/// Constructor contract mirrors `EdlSetup::new`: domain-reject via Option.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PostEffect {
    /// Bright-pass 9-tap blur + additive composite. `strength` scales the
    /// bloom contribution; fixed threshold via `bloom` constructor.
    Bloom { strength: f32 },
    /// Depth-edge silhouette outline, selection-orange overlay at `strength`
    /// mix ratio; `width` = edge detect radius in texels.
    Outline { width: f32 },
    /// Global tone-mapping curve (REND-44): `mode` 0 = Reinhard c/(1+c),
    /// 1 = ACES (Narkowicz filmic approx). Color-only pass (depth bound but
    /// ignored, shared bgl shape).
    Tonemap { mode: u32 },
    /// Depth-only screen-space ambient occlusion (WGPU-36): 16-tap disc of
    /// `radius` texels; occluded taps darken the pixel by `intensity`.
    /// v1 honesty: depth-only (no normal buffer) — creases/near-contact
    /// regions darken; HBAO/normal-buffer upgrade = ticket.
    Ssao { radius: f32, intensity: f32 },
    /// Screen-space depth haze (WGPU-38): blends toward a constant haze
    /// color by LINEAR view distance in meters (SSAO-family view_z
    /// linearization; raw depth is useless under perspective — probe-verified).
    /// Background (depth 1.0 = 1000 m) always takes full haze. Screen-space
    /// ONLY (post chain); scene-space fog = separate ticket.
    Haze { far_start: f32, far_full: f32 },
}

impl PostEffect {
    /// Domain-reject constructor: non-finite / <=0 -> None (explicit reject, no clamp).
    #[must_use]
    pub fn bloom(strength: f32) -> Option<Self> {
        if strength.is_finite() && strength > 0.0 {
            Some(Self::Bloom { strength })
        } else {
            None
        }
    }

    /// Domain-reject constructor: non-finite / <=0 -> None (explicit reject, no clamp).
    #[must_use]
    pub fn outline(width: f32) -> Option<Self> {
        if width.is_finite() && width > 0.0 {
            Some(Self::Outline { width })
        } else {
            None
        }
    }

    /// Domain-reject constructor: mode > 1 -> None (0 = Reinhard, 1 = ACES).
    #[must_use]
    pub fn tonemap(mode: u32) -> Option<Self> {
        if mode <= 1 {
            Some(Self::Tonemap { mode })
        } else {
            None
        }
    }

    /// Domain-reject constructor: non-finite / <=0 radius, or intensity
    /// outside 0.0..=4.0 -> None (explicit reject, no clamp).
    #[must_use]
    pub fn ssao(radius: f32, intensity: f32) -> Option<Self> {
        if radius.is_finite() && radius > 0.0 && (0.0..=4.0).contains(&intensity) {
            Some(Self::Ssao { radius, intensity })
        } else {
            None
        }
    }

    /// Domain-reject constructor: non-finite / far_start < 0 /
    /// far_start >= far_full -> None (explicit reject, no clamp).
    /// far_full is ALLOWED to exceed 1.0 (clamp-to-depth-1 keeps the top
    /// band below full haze when the author wants a soft ceiling).
    #[must_use]
    pub fn haze(far_start: f32, far_full: f32) -> Option<Self> {
        if far_start.is_finite() && far_full.is_finite() && far_start >= 0.0 && far_start < far_full
        {
            Some(Self::Haze {
                far_start,
                far_full,
            })
        } else {
            None
        }
    }
}

/// The backend contract (object-safe: the host holds `Box<dyn RenderBackend>` for polymorphic dispatch).
pub trait RenderBackend {
    fn name(&self) -> &'static str;
    fn supports(&self, capability: Capability) -> bool;
    fn resize(&mut self, viewport: Viewport);
    fn render(&mut self, frame: &Frame);
    /// CPU->GPU mesh upload, returns an opaque resource id (monotonic, never reused, REND-07).
    fn create_mesh(&mut self, desc: &MeshDesc<'_>) -> Result<MeshId, BackendError>;
    /// Material registration (v0 = base color solid, PBR in later rounds, REND-08).
    fn create_material(&mut self, base_color: [f32; 4]) -> Result<MaterialId, BackendError>;

    /// Textured-material face (WGPU-14): the default body forwards to solid color (backends without texture support comply with zero changes).
    fn create_material_desc(&mut self, desc: &MaterialDesc) -> Result<MaterialId, BackendError> {
        self.create_material(desc.base_color)
    }

    /// Embedded texture upload (downstream of GLTF-11): default = unsupported (the Err semantics are orthogonal to the supports face).
    fn upload_texture(&mut self, _desc: &TextureDesc<'_>) -> Result<TextureId, BackendError> {
        Err(BackendError {
            reason: "texture upload unsupported by this backend".into(),
        })
    }

    /// Instance-table upload (REND-27, 4c): default = unsupported (the explicit rejection protocol same shape as upload_texture).
    fn create_instances(&mut self, _desc: &InstanceDesc<'_>) -> Result<InstanceId, BackendError> {
        Err(BackendError {
            reason: "instancing unsupported by this backend".into(),
        })
    }

    /// Line-table upload (REND-30, 4de): default = unsupported (family protocol).
    fn create_strokes(&mut self, _desc: &StrokeTableDesc<'_>) -> Result<TableId, BackendError> {
        Err(BackendError {
            reason: "strokes unsupported by this backend".into(),
        })
    }

    /// Point-table upload (REND-30): default = unsupported (family protocol).
    fn create_points(&mut self, _desc: &PointTableDesc<'_>) -> Result<TableId, BackendError> {
        Err(BackendError {
            reason: "points unsupported by this backend".into(),
        })
    }

    /// Label-table upload (the sixth family protocol member, REND-33): the default body = explicit Err (silent skips blocked).
    fn create_labels(&mut self, _desc: &LabelTableDesc<'_>) -> Result<TableId, BackendError> {
        Err(BackendError {
            reason: "labels unsupported by this backend".into(),
        })
    }

    /// Point-table destroy (a V2.1 flowing-lane need; the minimal destroy face for dynamic table replacement -- the static mesh/strokes
    /// products stay put, only points serve per-frame rebuild). The default body = no-op (the family protocol default form).
    fn destroy_points(&mut self, _id: TableId) {}
}

/// Shadow depth bias (the [0,1] convention; negative = pulled away toward the light, prevents self-shadow acne).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowBias {
    pub constant: f32,
    pub slope: f32,
}

/// Directional-light shadow configuration (REND-31, 4f). **Construction contract**: `proj/view_rot/eye` may only be produced through
/// `CameraRig::perspective/ortho_frame` (the PIT-5 lesson promoted to a contract -- GL [-1,1]-domain hand-made matrices
/// entering the pipeline = silently clipped frames / all-shadow; symptom-level deadlock on the WGPU-19 pixel face); eye rides f64 D7 like the Frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowSetup {
    pub proj: [[f32; 4]; 4],
    pub view_rot: [[f32; 4]; 4],
    pub eye: [f64; 3],
    /// Normalized light direction (the fs Lambert and the shadow math share one source; the LIGHT shader constant retires into the UBO)
    pub light_dir: [f32; 3],
    /// Source angular size (world units; the PCSS penumbra source, 0 = hard PCF)
    pub size: f32,
    pub bias: ShadowBias,
}

impl ShadowSetup {
    /// Tuning baseline (if the P2 measurement changes it, this constant and the clause body change in sync).
    pub const DEFAULT_BIAS: ShadowBias = ShadowBias {
        constant: -1.2,
        slope: -1.5,
    };
}

/// Section-clip configuration (REND-32, B2 band). **Semantics**: plane coefficients `[nx,ny,nz,d]` in world-space f64,
/// the normal points to the **kept side**, the keep predicate is `dot(n,P)+d >= 0` (on-plane = kept, closed interval); multiple planes **AND**
/// (any negative side clips). Construction contract: `new` normalizes normals and rejects zero normals / non-finite / >`MAX_PLANES`.
/// Precision discipline: world coefficients held in f64 (1mm precision needs f64 at the 3857-campus magnitude), lowered to f32 on the shader side via
/// [`clip_to_local`] (dropped only after the f64 origin subtraction, same as the D7/rebase path).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipSetup {
    /// Normalized plane coefficients [nx,ny,nz,d] (the first `count` are live).
    pub planes: [[f64; 4]; Self::MAX_PLANES],
    /// Active plane count 0..=4.
    pub count: usize,
}

impl ClipSetup {
    /// Plane cap (the §1b ruling; a six-plane box = two-band backlog, raising to 6 touches only this one place).
    pub const MAX_PLANES: usize = 4;
    /// Empty clip (keeps all). Equivalent to the `Frame.clip=None` semantics position.
    pub const EMPTY: Self = Self {
        planes: [[0.0; 4]; Self::MAX_PLANES],
        count: 0,
    };

    /// Build from world coefficients: normalize + reject degenerates (zero normal / non-finite / over the cap -> None).
    #[must_use]
    pub fn new(planes: &[[f64; 4]]) -> Option<Self> {
        if planes.len() > Self::MAX_PLANES {
            return None;
        }
        let mut out = Self {
            planes: [[0.0; 4]; Self::MAX_PLANES],
            count: 0,
        };
        for (slot, src) in out.planes.iter_mut().zip(planes) {
            let [nx, ny, nz, d] = *src;
            if !(nx.is_finite() && ny.is_finite() && nz.is_finite() && d.is_finite()) {
                return None;
            }
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len == 0.0 {
                return None;
            }
            *slot = [nx / len, ny / len, nz / len, d / len];
        }
        out.count = planes.len();
        Some(out)
    }

    /// CPU predicate (for tests/picking): whether point P (world space) is kept.
    #[must_use]
    pub fn keeps(&self, p: [f64; 3]) -> bool {
        self.planes[..self.count]
            .iter()
            .all(|pl| pl[0] * p[0] + pl[1] * p[1] + pl[2] * p[2] + pl[3] >= 0.0)
    }
}

/// World plane -> **model-space** f32 coefficients (the REND-32 precision-discipline face). The shader judges at the raw vertex q
/// (under the compose_mvp column-major column-vector convention world = M*q + origin), the keep condition transforms identically:
/// `dot(n, Mq+o)+d = dot(Mᵀn, q) + (dot(n, o+t_M)+d)` -- the normal is pushed correctly by the linear part Mᵀ,
/// d is merged into origin + the M translation column (t_world = origin + t_M).
/// `d_local = dot(n,origin)+d` and `n_m` are multiplied through fully in f64 before dropping to f32 (same shape as D7/rebase),
/// when the plane passes exactly through origin the local d = 0 **precisely** (clause-level eradication of far-coordinate f32 jitter).
/// Under non-uniform scale n_m is not normalized: multiplying both sides of the inequality by the same positive scalar keeps the semantics, **no division means no flip risk** (det < 0 flip = rejected, hosts must not scale negatively).
#[must_use]
pub fn clip_to_local(plane: [f64; 4], origin: [f64; 3], model: &[[f64; 4]; 4]) -> ([f32; 3], f32) {
    let [nx, ny, nz, d] = plane;
    // n_m_i = Sigma_j n_j * m[j][i] (under column-major [col][row] this is exactly Mᵀn)
    let mut n_m = [0.0f64; 3];
    for (j, nj) in [nx, ny, nz].iter().enumerate() {
        for i in 0..3 {
            n_m[i] += nj * model[j][i];
        }
    }
    // translation merged in: t_world = origin + the fourth column of M (the homogeneous translation component)
    let dl = d
        + nx * (origin[0] + model[3][0])
        + ny * (origin[1] + model[3][1])
        + nz * (origin[2] + model[3][2]);
    ([n_m[0] as f32, n_m[1] as f32, n_m[2] as f32], dl as f32)
}

/// Material description (WGPU-14 pipeline-variant key source). N6 (REND-45):
/// GGX PBR fields land; `specular` (mock-up [4ab①]) is RETIRED — kept for IR
/// compat only, the shader ignores it (WGPU-14/WGPU-35 note).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialDesc {
    pub base_color: [f32; 4],
    pub texture: Option<TextureId>,
    pub repeat: [f32; 2],
    /// Mock-up Lambert coefficient [4ab①]; retired by GGX (WGPU-35).
    pub specular: f32,
    /// GGX perceptual roughness: alpha = roughness*roughness (WGPU-35).
    pub roughness: f32,
    /// Metallic factor: F0 = mix(vec3(0.04), base.rgb, metallic) (WGPU-35).
    pub metallic: f32,
}

/// CPU-side RGBA8 texture borrow (produced by io-gltf/geo -> the intermediate IR of the backend upload).
#[derive(Clone, Copy, Debug)]
pub struct TextureDesc<'a> {
    pub rgba: &'a [u8],
    pub width: u32,
    pub height: u32,
}
