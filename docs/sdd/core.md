# visiaengine-core Behavioral Contract (SDD)

> Test annotations carry `// spec: CORE-NN`; bidirectional traceability gate: `scripts/spec-trace.sh`.
> Coordinate re-basing (RTC) tests are deferred until after the P1 ruling (open point in architecture.md); not pre-written.

## CORE-01: create_get_roundtrip
`spawn()` allocates an entity, `insert(id, Component)` attaches it, `get(id)` returns the same-value component; idempotent within the same frame.

## CORE-02: delete_invalidates_handle
After `despawn(id)`, `get`/`insert`/`mark_dirty` for that id all return `CoreError::NotFound`, and `is_alive` is false.

## CORE-03: stale_generation_rejected
After slot recycling the old handle carries a stale generation; any access returns `NotFound` (must not touch the new resident data).

## CORE-04: slot_reuse_after_delete
Slots freed by despawn are reused by subsequent spawns: same `slot`, `generation` strictly +1.

## CORE-05: iteration_yields_alive_only
`alive_ids()` yields exactly all living entities — no empty slots, no deleted entities.

## CORE-06: dirty_flag_coalesces
Multiple `mark_dirty` calls on the same entity in the same frame record dirtiness once (the `take_dirty` result is deduplicated to a single entry).

## CORE-07: take_dirty_clears
`take_dirty()` yields the current dirty list and clears the flags; an immediate second take returns empty.

## CORE-08: f64_position_preserved
`Vec3` is all-f64: coordinates at ≥1e7 magnitude and arbitrary binary fractions round-trip with zero loss (no precision dropped, no f32 conversion).

## CORE-09: component_missing_err
`get` on a living entity without the component returns the typed `CoreError::MissingComponent{ component: "Transform" }`, no panic.

## CORE-10: insert_replaces_component
Repeated `insert` of the same component type on one entity is replace semantics (get always yields the last value), not multi-value stacking.

## CORE-11: AttrSet three-type typed columns
`AttrSet` provides `add_row() -> row index` plus `set_f64/set_str/set_bool(row,name,v) -> bool` writes and `f64/str_value/bool(row,name) -> Option<T>` read accessors. Missing data (column absent / wrong-type read / row out of range / empty slot) always returns `None`, never panics — separating the semantics of "attribute absent" from "value is 0/false" (style-key resolution depends on this).

## CORE-12: Row alignment is a storage invariant
Column row count always equals `AttrSet::len()`; `set_*` to an out-of-range row implicitly grows rows (all columns resize in sync); rewriting the same name on the same row = overwrite. Row-number allocation and the element↔row 1:1 mapping are maintained by the host (see GEO-17 on the geo side).

## CORE-13: First write locks the column type
Once a column is created its type is locked; subsequent wrong-type `set_*` return `false` and leave the data unchanged (the render/style sides can rely on the type being stable per column name).

## CORE-14: Ray x triangle (Möller–Trumbore)
`ray_triangle(Ray{origin:Vec3,dir}, a,b,c) -> Option<t>`: CCW front-face hit (back-face culled; `ray_triangle_double` is the double-sided entry), returns the distance parameter t (hit point = origin + t*dir). Behind the ray (t≤0), parallel, out of bounds (u/v/u+v), degenerate (det≈0, zero area) all return None, never panic. All f64 (D7 far-origin safety).

## CORE-15: Ray x AABB culling entry
`ray_aabb(Ray, min, max) -> bool` (slab method): origin inside the box = true; parallel-axis out-of-bounds = false; entirely behind the ray = false. Used for per-entity on-the-fly culling (bbox is not stored persistently, lazy-compute semantics per [E3D:A5]; upgrade slot = BVH, trigger = bench evidence, same lineage as the CORE-12 row-alignment discipline).

## CORE-16: Color-space conversion entries (sRGB ↔ linear)
`srgb_to_linear/linear_to_srgb([f32;4]) -> [f32;4]` (IEC 61966-2-1 piecewise exact form: threshold 0.04045, linear segment /12.92, power segment ((c+0.055)/1.055)^2.4; endpoints 0/1 bit-exact, alpha always passthrough). Contract split: host/CSS convention values = sRGB-encoded, engine lighting domain = linear; these entries cover the scalar half, while textures/frame targets are handled by the hardware half of the Srgb formats (render-wgpu). Round-trip ≤2e-7, monotonic across 256 levels.

## CORE-17: Group node creation (spawn_group)
`Scene::spawn_group() -> EntityId`: allocates a new slot and writes `Component::Group`. Group entities have no mesh and take part in neither rendering nor picking; they serve purely as tree containers carrying offset inheritance and visibility aggregation. Group entities appear alongside ordinary entities in the `alive_ids()` list (consumers filter by `Component::Group`).

## CORE-18: Parent-child relation setting (set_parent)
`Scene::set_parent(child, parent: Option<EntityId>) -> Result<(), CoreError>`:
- parent=Some(g) requires g to carry `Component::Group` (non-Group = rejected with MissingComponent, zero partial writes);
- **Cycle detection**: walk the ancestor chain upward from g; encountering child = reject (NotFound); chain length > MAX_TREE_DEPTH(=16) = reject (prevents infinite loops);
- parent=None = detach to root (Slot.parent reset to None, the child's own offset preserved);
- dead child = NotFound; living child without components = legal (a pure container can hold any entity, including sub-Groups).
**despawn semantics**: after g is deleted, its direct children's parent is automatically reset to None (detached to root, no cascade delete).

## CORE-19: Parent query (get_parent)
`Scene::get_parent(id) -> Result<Option<EntityId>, CoreError>`: dead id = NotFound; alive = Some(parent) or None (root).

## CORE-20: Group offset read/write (group_offset / set_group_offset)
`Scene::set_group_offset(g, offset: [f64; 3]) -> Result<(), CoreError>`: g must carry Component::Group (otherwise rejected with MissingComponent); offset is a world-meter translation and does not trigger dirty flags on children (accumulated live at render time, see CORE-21).
`Scene::group_offset(g) -> Result<[f64; 3], CoreError>`: reads g's own offset (not the accumulation).

## CORE-21: Effective offset accumulation (effective_offset)
`Scene::effective_offset(id) -> [f64; 3]`: walks the parent chain from id accumulating each level's `Slot.offset`, up to the root (parent=None) or truncated at MAX_TREE_DEPTH. O(depth ≤16). Dead intermediate links are skipped (slot_of failure = break, no error — tolerating concurrent host modification edges).
Consumers (render loop / pick / is_hidden) use `draw_origin = it.origin + scene.effective_offset(it.entity)` in place of `it.origin`.
`Scene::is_descendant_of(d, a) -> bool`: true when d or any ancestor of d equals a (d==a returns true, reflexivity).
