# visiaengine-render behavioral contract (SDD)

> Clause header lines `## REND-NN:` are the traceability anchors; tests carry `// spec: REND-NN`.

## REND-01: backend_trait_object_safe
`RenderBackend` must be object-safe: `Box<dyn RenderBackend>` is constructible in crate tests (the precondition for multi-backend dispatch / host ownership).

## REND-02: stub_impl_without_wgpu
The trait is implemented with a pure-CPU stub inside this crate's test module (this crate has no wgpu dependency = a construction fact) — proving **the contract surface carries zero backend types** (invariant ②); the stub handles ClearColor frames without panic.

## REND-03: ir_variants_exhaustive_construct
All IR `DrawCommand` variants are constructible and `kind()` distinguishes them one-to-one (v0: ClearColor / DrawMesh).

## REND-04: viewport_roundtrip
`Viewport::new(w, h, scale_factor)` accessors round-trip losslessly; `logical_size()` = physical size / scale factor.

## REND-05: camera_projection_variants
After constructing `Camera::Ortho/Perspective`, `is_orthographic()` discriminates correctly (the projection-side foundation of the unified 2D/3D entry; the switch animation belongs to a later slice).

## REND-06: mesh_desc_constructible
`MeshDesc{positions:&[[f32;3]], normals:&[[f32;3]], indices:&[u32]}` constructs with pure borrowing; the length constraint is guaranteed by documentation (positions/normals equal length).

## REND-07: create_mesh_returns_distinct_ids
Trait method `create_mesh(&MeshDesc) -> Result<MeshId, BackendError>`; two stub calls yield distinct MeshIds (monotonic, no reuse).

## REND-08: create_material_returns_distinct_ids
Trait method `create_material([f32;4]) -> Result<MaterialId, BackendError>`; same semantics as REND-07.

## REND-09: frame_view_proj_fields_roundtrip
`Frame` gains `view/proj: [[f32;4];4]` (column-major raw data, zero matrix library on the contract surface); the identity matrix round-trips losslessly.

## REND-10: camera_rig_view_matrix_lookat
`CameraRig::look_at(eye,target,up)` -> view matrix: eye maps to the origin, target's view-space z component <0 (right-handed-convention assertion).

## REND-11: persp_proj_matches_glam_std
90°FOV/aspect1/near1/far∞(large) perspective matrix versus hand-computed standard form (RH-GL depth 0..1 variant, implementation choice declared in the camera.rs header comment).

## REND-12: ortho_zoom_scales_halfextents
`CameraRig::ortho_frame(zoom)` projection half-width/half-height scales linearly with zoom (x-axis [1,0,0,0] column component = 1/zoom).

## REND-13: orbit_90deg_equivalence
orbit_delta(45°)×2 and orbit_delta(90°)×1 give equal eye positions within f64 tolerance 1e-6.

## REND-14: degenerate_near_far_rejected
near>=far -> `proj_matrix` returns None (constructive rejection, zero panic).

## REND-15: switch_midpoint_continuity
`mix_rig(a, b, t)` per-component interpolation in parameter space: the field-by-field difference between `mix(0.5)` and `mix(0.5001)` is <1e-3 (t continuous domain, no jumps).

## REND-16: switch_endpoints_exact
`mix_rig(a,b,0)==a`, `mix_rig(a,b,1)==b` field-exact (endpoint identity).

> v0 note: yaw is linearly interpolated (demo angle domain <180°); wrap-to-shortest-arc belongs to a later camera slice (ponytail marker at the implementation).

## REND-17: frame_view_rotation_eye_split
`Frame.view` splits into `view_rot:[[f32;4];4]` (pure rotation) + `eye:[f64;3]` (world camera position); the `camera` kind-discriminant field is kept; raw-data construction round-trips losslessly (D7 implementation state).

## REND-18: drawmesh_carries_origin
`DrawMesh +{ origin: [f64;3] }` (layer-level world origin, D7 ②); transform semantics become the **origin-local** pose (f64); all construction sites updated in compile sync.

## REND-19: rebase_identity_at_zero_origin
`rebase::compose_mvp(proj, view_rot, eye, origin, model_local)`: with origin=0 and model = the world matrix, ≡ the old P·V·M composition (1e-4 tolerance).

## REND-20: far_origin_precision_preserved
origin=(1e7,0,0), vertex local 0.5, camera 10m in front of origin: post-compose clip-domain relative error <1e-4; the control legacy route (world straight f32 cast) errs ≥0.25m.

## REND-21: perspective screen ray
`screen_to_ray_persp(rig, px, py, w, h) -> Option<Ray>`: top-left-origin pixel frame (px=0 -> NDC −1, py=0 top edge -> +1); angular extents tx=tan(fov_y/2)·aspect, ty=tan(fov_y/2), dir=normalized(fwd+right·tx·nx+up·ty·ny), origin=eye. The view basis is constructed analytically under the same UP convention as look_at; degenerate input (w/h≤0, illegal fov, pole fwd∥UP) -> None.

## REND-22: orthographic screen ray + D7 far-field preservation
`screen_to_ray_ortho`: origin=eye+right·(hw·nx)+up·(hh·ny) (hw=rig.zoom, hh=zoom·h/w, same convention as REND-12), dir=fwd. Both entries run all-f64 (no f32 step-size disaster for origins/directions at large world coordinates — the D7 assertion surface).

## REND-23: scene picking = nearest hit + on-the-fly culling
`pick_meshes(Ray, &[MeshCandidate{entity, positions, indices, world}]) -> Option<PickHit{entity, triangle, t, point}>`: per candidate the **world AABB is computed on the fly** (local 8 corners transformed, [E3D:A5] lazy compute, not stored) -> `ray_aabb` cull -> per-triangle `ray_triangle` front-face hit taking the global min t. Empty-geometry / out-of-range-index candidates are skipped without panic.

## REND-24: picking world composition
Candidate vertices = local f32 transformed by `world` (column-major f64, same shape as `DrawMesh.transform`) before intersection — composed scaling/translation lands both the hit point and t in the world frame (f64, D7 domain). Empty candidate array = None.

## REND-25: MeshDesc.uv optional semantics
`MeshDesc.uv: &'a [[f32;2]]`: an empty slice = the whole mesh zero-filled (0,0) (the Flat legacy shape, zero change); non-empty and ≠ len(positions) = backend rejection (no silent padding, preventing misaligned sampling).

## REND-26: RenderBackend texture-extension default bodies
`create_material_desc(&MaterialDesc)` default = forwarding to `create_material(base_color)` (texture/repeat/specular dropped = safe degradation); `upload_texture(&TextureDesc)` default = explicit Err (backends without a texture plane do not pretend success). Both are default bodies = trait backward compatibility, existing impls zero change.

## REND-27: instance table contract (4c)
`create_instances(&InstanceDesc{data:&[Instance]}) -> Result<InstanceId, BackendError>` default body = explicit Err (backends without an instance plane do not pretend success, the same protocol as upload_texture). `Instance` = 32B Pod (`repr(C)`: offset[0..12]/height[12..16]/color[16..28]/pad[28..32], byte-for-byte isomorphic to the WGSL `struct Instance`, locked by the offset_of+bytes double assertion); constructed only via `Instance::new` (zeroed-pad guarantee). The instance color is RGB — alpha always lives in the material (ruling point c).

## REND-28: DrawInstances variant (4c)
`DrawCommand::DrawInstances{mesh, material, instances: InstanceId, origin, transform}` — the D7 origin semantics are the same kind as DrawMesh (f64 far-coordinate layer, instance offsets are origin-local f32); `kind()="draw-instances"`; the consumer-side exhaustive-match synchronizer extends its arms (compile-time forced explicit handling, a new backend missing an arm = compile failure). v0 has no instance-level culling/sorting (full single draw, same DrawMesh discipline).

## REND-29: Frame.px_world_scale (4de)
`Frame +{ px_world_scale: f32 }` — how many world units 1 screen pixel ≙ @ the reference depth (**host-provided**: ortho top-view = 2·zoom/width exact; perspective approximate = the 4de not-built ① note). The width/radius multiplier of the expansion families (Strokes/Points); the triangle-family pipelines never read it = zero impact. Default at construction sites = 1.0.

## REND-30: expansion tables and command family (4de)
`TableId=u64` alias (same counting domain as InstanceId, the new resource-family naming). `StrokeSeg` 48B Pod (`a[0..16] b[16..32] color[32..44] width_px[44..48]` — vec4-wide same-width fields eliminate CPU/GPU alignment ambiguity [R1]) / `PointMark` 32B Pod (pos/radius_px/color/pad); `create_strokes`/`create_points` default bodies = explicit Err (family protocol members 4/5). `DrawStrokes{table,origin,transform}` / `DrawPoints{...}`: **color/width live in the table, not on the material** [ruling point a] (materials carry no base/shade semantics for the expansion families — the lighting chain does not participate, color outputs directly); same D7 origin; kind()="draw-strokes"/"draw-points".

## REND-31: ShadowSetup contract (4f)
`Frame +{ shadow: Option<ShadowSetup> }`: `None` = off with **bitwise zero regression** (the backend's dummy early-exit position, key-position semantics go into WGPU-19); `Some` = directional light. `ShadowSetup{proj, view_rot, eye(f64 D7 same as Frame), light_dir, size, bias}` — **constructor contract [PIT-5 upgraded to contract]**: the triple may only be produced by `CameraRig::perspective/ortho_frame` (its wgpu [0,1] domain is locked by the REND-11/12 behavioral assertions); hand-built GL [-1,1] matrices = symptom-level death (all-black/all-white), the pixel deadlock sits on the WGPU-19 surface. `size` = the light's angular size in world units (0 -> hard PCF); `bias` default `ShadowSetup::DEFAULT_BIAS{-1.2,-1.5}` (tuning baseline, after the P2 measured setting the two sites stay in sync).

## REND-32: ClipSetup cross-section clipping contract (B2)
`Frame +{ clip: Option<ClipSetup> }`: `None`/`EMPTY` = keep everything with **bitwise zero regression** (backend all-zero always-bind + count=0 early exit, the R4 guardrail form, behavior surface = WGPU-21). Plane coefficients `[nx,ny,nz,d]` in **world-frame f64**, the normal points to the kept side, the keep predicate is `dot(n,P)+d ≥ 0` (**on the plane = kept**, closed interval); multiple planes **AND-combined** (any negative side clips; OR/union = a non-goal on the tab). `MAX_PLANES=4` (§1b: the single-plane/corner-cut/3-axis-box common tier, six-face box = two bands). Constructor contract: `ClipSetup::new` normalizes normals and **rejects** zero normals / non-finite coefficients / over-cap (None = explicit rejection, no truncation, no swallowing). Precision discipline [§1d]: world coefficients must be f64 (at the park's 3857 magnitude, f32 plane-position jitter ~1m = unusable), the shader side converts to **model-space** via `clip_to_local(plane, origin, model)`: `n_m = Mᵀn`, `d_m = d + dot(n, origin+t_M)` (the world=M·q+o predicate identity, q = raw vertex); all multiplication in f64 before dropping to f32 (isomorphic to D7/rebase; when the plane passes through origin the local d is exactly 0 as a clause-level assertion, the bitwise-same-predicate at far origin = the IR foundation; non-uniform scaling not normalized = both sides multiply by the same positive scalar, semantics unchanged). `keeps(p)` = the CPU predicate (shared by the test/pick entries, the CAPI-20 consumption surface).

## REND-33: LabelMark/DrawLabels label-table contract (S2)
`LabelMark` **64B=4×vec4 Pod** layout locked (`pos(xyz+pad) color(linear rgba) uv(u0v0u1v1) metrics(w_px,h_px,dx right,dy up)` — byte offsets pinned at pos@0/color@16/uv@32/metrics@48, same discipline as REND-30). **Color = linear domain** (sRGB->linear lives on the producer io-text/CAPI surface, the CORE-16 throat extension = the label table). `DrawCommand +{ DrawLabels{table,origin,transform} }` the 6th variant: `table=TableId` (the create_labels product), origin = D7 world anchor, transform = pose (identity in most cases); `kind()="draw-labels"`; **no material attached** (same lineage as expansion-family ruling a — color lives in the table). `RenderBackend::create_labels` default body = explicit Err (family protocol member 6, silent skipping sealed). The atlas texture **does not enter the signature** (a backend singleton channel, WGPU-24 — the command carries only the quad-table handle). The exhaustive synchronizer's four arms updated (contract_spec stub.render/kind self-check/mesh origin test + mesh_core clear-find/draw-loop placeholder arms, image production = N3).

## REND-34: flyTo pose sampling + easing (⑤a)
`Easing{Linear, CubicInOut(smoothstep 3x²−2x³), CubicOut(1−(1−x)³)}`: `ease:[0,1]→[0,1]` **monotonically increasing + endpoints bit-exact + out-of-domain clamped** (the three locks asserted bitwise). `CameraRig::fly_sample(from,to,t,easing)`: a **pure function** — the wall-clock driver lives in engine (the settled layering: no std::time in the render crate, zero dt obligations on the host's render() [the factual resolution of D8]); `t=0/1` take the fast path returning from/to verbatim (the endpoint keeps yaw's original number, same lineage as the mix_rig endpoint semantics REND-16); **yaw shortest arc** = to.yaw normalized by integer multiples of ±2π to from.yaw+Δ(Δ∈(−π,π]) before interpolating (attitude mod 2π equivalence; |Δ|≤π = zero intervention, pure linear — the wrap correction sits in the sampler and **does not touch mix_rig**, the REND-15/16 legacy assertions untouched). near/far do not pass through the pose (CAPI-23 has no depth domain = always from's current values, the PIT-5/REND-14 lock rides along [Momus-A1]). Mid-flight reroute (CAPI-23 re-entry) from = the current sampling snapshot = continuity by construction [Momus-A3].

## REND-35: ViewportRect viewport rectangle (⑤b)
`ViewportRect{x,y,width,height}` (u32 physical pixels, top-left origin): a standalone type that **does not touch Frame** (ruling a — Frame keeps the full-screen semantics, avoiding the wave-44 fallback; the rect travels as a render-call parameter `render_view_rects`, each projection carrying its own Frame × shared resource tables). `full(w,h)` = the full-screen form (the canary bitwise-equality entry); `contains/local` = the minimal shape of input routing (the host takes area-local px after hit-test, `screen_to_ray_*` reuses by passing the rect size as w/h, the REND-21/22 signatures zero change). Overlap between viewports is undefined (v0 discipline: non-overlapping compositions).

## REND-36: transparent material semantics (④)
**Single predicate = material alpha<1** (`MatGpu.alpha=base_color[3]`, the srgb_to_linear a-channel passthrough empirically proven) -> that DrawMesh/DrawInstances part takes the transparent pipeline (WGPU-27); alpha==1 always takes the opaque route = **construction-level golden fidelity** (the sort touches it zero). **Material a==1 ∧ texel a<1 stays opaque** — texture alpha participates in the `a=base.a·texel.a` composite only inside the material-a<1 pipeline (no hidden semantics [Momus-A3]); per-vertex alpha declared as post-poned. The caster projects semi-transparent parts **as if opaque** (v0 semantics, three.js made the same choice; soft shadows = the Alpha tier trigger). The pick domain is alpha-independent (zero touch to geometric intersection). The host's only route to change alpha = re-upload the material (remove+add); the hot-edit entry `set_material_alpha` is tabbed (trigger = interactive brightness tuning).

## REND-37: two navigation-math pieces (⑤b second wave)
`ray_ground_intersect(Ray) -> Option<Vec3>`: intersection with the z=0 ground plane (**navigation semantics, not picking** — entity hits go through the ray-triangle REND-21/22 domain); the domain rejects three shapes = `d.z≥−1e-9` (the horizontal/upward/near-horizontal t-explosion guardrails) ∧ `t≤0` (backward/underground start) ∧ non-finite. `ViewportRect::from_frac(fx,fy,fw,fh,S,H)`: ratio table -> pixel rect (set_map's host language = ratios, the geometric base of auto-following resize); policy pinned = edges round, right/bottom edge min(surface bound), size floor kept at 1 (merging with scissor max(1)), dirty frac values clamped in-domain without panic. **Shared-edge seam law**: adjacent pieces satisfy `x+width==next.x` (round half-crossing consistency by construction) — relied on by E816/canary.

## REND-38: point-cloud screen-space picking (B2)
`pick_points(mvp,w,h,px,py,r_px,cands) -> Option<PointHit>`: a screen-space nearest-point predicate — retained positions projected into the screen pixel domain through the render-identical MVP (`compose_mvp` D7 chain, **local points projected directly** = the origin−eye translation is already inside the matrix, zero secondary accumulation on the predicate side); among those within the `r_px` radius the nearest competes by view depth (NDC z, the RH [0,1] domain, same PIT-5 shape); beyond-domain (behind / past far) and `cw≤0` do not participate. The hit returns `(entity, screen_d2, view_z, point_index)` — point_index lets the caller run the per-point clip `keeps` filter (same lineage as the mesh route's per-point semantics, not a whole-cloud coarse sieve). **Mesh preferred**: the engine pick route reaches the point cloud only after the mesh misses (the REND-39 canary construction guarantee). Default radius 6px (same screen-pixel domain as REND-30 `radius_px`). Retention surface: the engine keeps `positions` per point-cloud entity (marks were originally GPU-table take-and-drop); the `set_visible` domain extends to point-cloud entities (completing the CAPI-13 "render/pick filtering" semantics). ponytail note: linear scan; hover throttling = upgrade slot 1, BVH = upgrade slot 2.

## REND-39: pick-domain extension with zero regression (B2)
After point picking lands, mesh picking behavior is **byte-unchanged** (the mesh-preferred order + the point cloud only backstops after a full mesh miss) — the existing pick-test family (E401/E402/golden_pick_window_mirror) is an always-installed canary; `PointHit` is a standalone type touching `PickHit` zero. After a point cloud's `set_visible(false)` it vanishes from both render and pick, `is_visible` reads back consistently (the CAPI-13 domain-extension note).

## REND-40: EdlSetup contract (N1.5)
`Frame +{ edl: Option<EdlSetup> }`: `None` = off with **bitwise zero regression** (behavior surface = WGPU-33: no extra pass, the depth StoreOp unchanged, no intermediate resources built). `EdlSetup{strength: f32}` (Copy/Debug/ PartialEq, same family as ShadowSetup/ClipSetup); constructor contract `EdlSetup::new` rejects non-finite/≤0 (None = explicit rejection, no truncation, no swallowing). `strength` semantics = the edge-darkening multiplier (0..∞; the pipeline side caps at clamp 0.7 against over-darkening); the multi-viewport path ignores it this band (the WGPU-33 on-account note). Host-surface note: the capi engine reaches the N1.5 `--edl` tier via the CAPI band (the E204 example = 0.35 probe-pinned).

## REND-41: projection morphTime (N2.1)
`morph_proj(ortho, persp, t)` (the render::morph pure function): at t=0 bit-for-bit equal to the ortho matrix, at t=1 bit-for-bit equal to the persp matrix (endpoint canary = byte-level, `morph_spec::morph_endpoints_bitwise`); intermediate values interpolate element-wise by smoothstep (k=t·t·(3−2t), the REND-34 family). **Single law in the projection domain**: view_rot/eye/commands pass through untouched; px_world_scale interpolates linearly between the two ends (morph_px_scale: the ortho exact 2·hw/W -> persp 1.0). The Camera enum family takes ortho at t≤0.5 and persp at t>0.5 (the view-block uniform tier; the real interpolation lives in the proj matrix — the enum/matrix division of labor archived). Slider semantics: Engine::set_morph_time maps NaN->0 and clamps out-of-domain (a **deliberate fork** from EdlSetup's explicit rejection — t is a slider, strength is a threshold). Multi-viewport: each Frame carries its own mode/morph, mutually independent. The machine-honoring anchor of whitepaper §3.1 "smooth transition within the same viewport" = endpoint byte-equality + mid-path non-degeneracy (morph_spec + E304 --frames).

## REND-42: entity animation keyframe sampling (N2.2)
`anim_origin(keyframes, t)` (the render::anim pure function): the endpoint-identity law — t≤ first key -> the first origin verbatim, t≥ last key -> the last origin verbatim (bit-equal, the mix_rig/REND-16 convention); within an interval, linear interpolation between adjacent keys (easing = the host time-axis orchestration policy, the sampler does not presume); a single-key table = a constant. All f64 (the D7 far-origin precision test on record). **Zero engine state / zero ABI** (the C5 split ruling: 6a = sampler + demo; the entity-position hot-update C entry = 6b, trigger-based). Consumption shape = the host samples per frame then routes through the existing origin path (the E305 trajectory replay).

## REND-43: PostEffect post chain IR (N4)
`enum PostEffect{Bloom{strength}, Outline{width}}` (Clone/Copy/Debug/PartialEq, EdlSetup family); `Frame +{ post: Vec<PostEffect> }` — **empty vec = byte-identical legacy path** (no intermediate resources, depth Discard unchanged; behavior clause = WGPU-34 zero-effect byte law). Effects compose in **Vec order** (each fullscreen pass reads the previous pass output; <=2 effects = this band's stage cap). Constructor contract = EdlSetup::new same-shape **explicit reject**: `PostEffect::bloom(strength)` / `PostEffect::outline(width)` reject non-finite / <=0 -> None (no clamp, no swallow). Semantics: Bloom.strength = additive bloom contribution multiplier (threshold = shader constant THRESHOLD 0.25, sample domain = linear-decoded values); Outline.width = edge-detect texel radius, doubling as the color-mix strength (orange [1.0, 0.83, 0.29] selection color, clamped 0..1). The `edl` slot stays untouched (point-cloud-specific shipped clause, REND-40 zero-regression law carried forward); post chain may stack with edl in one frame (edl runs first). Multiview path ignores it this band (WGPU-34 on-account note).

## REND-44: Tonemap post effect (N5)
`PostEffect +{ Tonemap { mode: u32 } }` (Clone/Copy/Debug/PartialEq, same family lineage); constructor contract `PostEffect::tonemap(mode)` domain-rejects `mode > 1 -> None` (0 = Reinhard, 1 = ACES (the Narkowicz filmic approximation); 2 = AgX, tabbed as explicit rejection). Shader `tonemap.wgsl` fullscreen pass (same shape as bloom/outline vs_main); Reinhard: `c/(1+c)`; ACES: `(x*(2.51x+0.03))/(x*(2.43x+0.59)+0.14)` per component; params=(mode_f32,0,0,0); color-only (the depth slot is bound but never read = the shared-bgl shape discipline). **LDR-input honesty note**: the pipeline outputs LDR (0..1 after sRGB encoding), tonemapping on LDR input = contrast reshaping (mood); true HDR display needs a float-texture path (ticket). Consumption = the pipeline's per-(effect,format) lazy pipelines (the N4 WGPU-34 orchestration zero change, effect_idx=2 third slot); the zero-effect law unchanged (post empty = old path bitwise). Tests = post_chain.rs tonemap double gate (off-vs-reinhard / off-vs-aces / rein-vs-aces differentials + determinism + bloom-after-tonemap compose) + the contract_spec domain-reject gate; the E507 example = live proof on the four-state window surface.

## REND-45: MaterialDesc v2 + 48B material block + VeMeshDesc extension (N6)
`MaterialDesc +{ roughness: f32, metallic: f32 }` (v2; dielectric defaults 1.0/0.0 — the legacy-equivalence band of the WGPU-35 ambient-constant probe setting). **The mock `specular` retires**: the IR field is kept (33 literal construction sites stay compatible) but shaders never consume it (the WGPU-14 note sentence closes — "real PBR = its own round" is exactly this band). The material GPU block 32B->48B (layout see WGPU-35; the WGSL vec4/vec2 alignment law — roughness@24, not 32; the first-draft CPU block mislaid at @32 was caught on the spot by the roughness-ladder all-zero probe = the layout triple-lock discipline honored again). **The C surface (Momus erratum honored: there is no VeMaterialDesc, the carrier = VeMeshDesc)**: `VeMeshDesc +{ roughness: f32, metalness: f32 }` (values not pointers; the struct_size forward gate naturally governs — old host + new engine = zero-initialized tail -> `roughness≤0 maps to 1.0`, the dielectric default law); three-surface mirror: the hand-written header typedef + wasm addMesh gaining the roughness/metalness two parameters (the same ≤0->1.0 law) + the hpp thin forwarder zero change (the C struct passes straight through). glTF loading: roughness_factor/metallic_factor pass through (the mock inversion `(1−m)·r` retires). Test anchors: contract_spec (v2 fields) + E508 + the capi add_mesh lineage (7-parameter signature).

## REND-46: Screen-space stroke pick predicate (W band)
`pick_strokes(mvp,w,h,px,py,cands) -> Option<StrokeHit{seg_index, screen_d2, view_z}>`: constant-width fat-line pick mirroring REND-38's screen-space family. Both endpoints of each `StrokeSeg` project through the render-identical MVP (`compose_mvp` D7 chain; segments are world-baked by the caller — tile-layer path bakes `origin + M` at upload, so the predicate adds nothing). A segment is REJECTED when either endpoint fails the [0,1] RH depth-domain guard (behind near / beyond far / cw<=0) — v1 does NOT clamp partially-offscreen segments (documented limitation, no silently wrong hits). Hit rule: classic point-to-segment distance in screen px <= `width_px/2 + 2px` slop (input-comfort margin, same spirit as the point-pick radius). Among hits the smallest endpoint-min view depth wins (conservative front-stroke rule; equal-depth ties keep scan order). Hit granularity = segment INDEX within the candidate table — per-segment entity association is a caller concern (StrokeTableDesc is a whole-table surface; entity mapping joins when a per-seg entity table exists). Clip filtering stays caller policy (WGPU-21). Engine priority: mesh > stroke > point (REND-39 ordering law carried forward). ponytail: linear scan, per-seg O(1) after projection.

## REND-47: Parametric curves — Bezier/CatmullRom + tube/ribbon (V2.1)
The `curve` module (pure math, +0 ABI — products travel the ordinary MeshDesc loading route). Sampling: `bezier3` (cubic, endpoints exact), `catmull_rom` (uniform CR, endpoint duplication; chordal = upgrade slot), `sample_*` (n equal divisions). Meshing: `tube(center, radius, sides)` = the **parallel-transport frame** (rotation-minimizing; vertical-tangent fallback; degenerate input = empty Vec contract) ring-welded quad topology; `ribbon_xz(center, width)` = a constant-height road ribbon in the XZ plane (the center y preserved). OpenDRIVE prerequisite (V4.2 depends on this band) — the curve base of road meshes.
