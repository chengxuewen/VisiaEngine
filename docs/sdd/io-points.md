# IO-*: point-cloud loading plane (visiaengine-io-points) behavioral contract

> Fixture source of truth: `resources/data/pcl_*.ply` = `#[ignore]`-emit from the io-points test builder (same lineage as texquad.glb, regenerable).
> Value-domain note: color in this file = the **sRGB host plane** (CORE-16); coordinate output = **origin-local f32 + origin f64** (D7/IO-06).

## IO-01: PLY ascii header/property parsing
`parse_pcl(bytes, lenient)`: magic `ply` + header line set (`format ascii|binary_little_endian|binary_big_endian 1.0`); `element vertex N` required; property set: `x/y/z` (float|double, any one missing = Err for the whole file); `red/green/blue` (uchar → /255 normalization, or float passed directly = the host sRGB plane) optional, missing = a light-gray default [0.7,0.75,0.8] for the whole cloud. Unknown **scalar** property columns = column-level drop (dropped_unsupported count, points not dropped); `list`-typed properties / `face` / `camera` and other elements = element-level drop counting. An illegal header (no ply header / missing end_header / missing vertex) = Err for the whole file regardless of lenient (file-level fate).

## IO-02: binary_little_endian payload parsing
The header property order drives the per-point stride (`from_le_bytes` explicit endianness — host-order dependence forbidden, the defense line against the x86-always-LE "false pass"); supported type surface: float(4)/double(8)/uchar(1) (int8/uint8/int16/uint16/int32/uint32 scalar widths folded into the reads); `binary_big_endian` = UnsupportedFormat Err for the whole file (v0 provides no endianness flip — add it when evidence appears); payload length ≠ N×stride → IO-04 truncation semantics take over.

## IO-03: Four-way classification of dirty point-cloud data (domain redefinition, not inheriting GEO-15)
`PclReport { dropped_non_finite, dropped_out_of_domain, dropped_unsupported, truncated_points, kept }`:
1. `dropped_non_finite` — point level: any coordinate NaN/±Inf;
2. `dropped_out_of_domain` — point level: double coordinates beyond f32 capacity (±>3.4e38); the 1e38/FLT_MAX sentinel check is **reserved for the LAS side and not enabled in this band** (the PLY plane covers true overflow only);
3. `dropped_unsupported` — column/element level (defined by IO-01), not point level;
4. `truncated_points` — file level (IO-04).
**Duplicate points do not form a class**: multi-return / high-density near-duplicates are legitimate physical data (LAS `number_of_returns` is first-class semantics), dedup = a downstream policy, not a repair; zero dedup API in this band, this note welds the door shut against re-litigation. FastFail = lenient=false: any drop class >0 → Err for the whole file (same lineage as the GEO-16 dual strategy); Lenient keeps points and continues.

## IO-04: Truncation dual semantics (self-pinned in this repo, no GEO-15 precedent)
Declared N > actual complete point count (ascii trailing partial line / binary tail bytes shorter than one stride): `FastFail` → `Err(Truncated)` (the GLTF-06 truncation-must-Err lineage); `Lenient` → accept **the complete prefix** (a residual point takes no partial line), `truncated_points = N - kept > 0`, report honestly. Positive-assertion obligation: byte-level programmatically truncated tests must pin both directions — "Lenient prefix = the complete point count" and "FastFail = Err" (self-pinned semantics = the test carries its own definition).

## IO-05: Capacity guard (the only point-count gate in this band)
`MAX_POINTS_CAP = 4_000_000` (named constant; after the initial value is calibrated by a release bench, a one-time value change is allowed: constant + this clause's number + the test domain value, three places in one commit). Declared points or actually-kept points exceeding the cap → `Err(OverCapacity { declared, cap })`, **not** truncated acceptance (taking everything then half-swallowing = false memory savings). The C entry / wasm share this gate (the engine-side load path is unique). add_points (CAPI-18) is not governed by this gate — raw pass-through = the host keeps its own books.

## IO-06: Product data model (zero render dependency, discipline of the io-* family)
`PclCloud { positions: Vec<[f32;3]>(origin-local), colors: Vec<[f32;3]>(sRGB plane), radius_px: f32(uniform across the cloud, consumed by mount), origin: [f64;3](bbox center, D7), meta: AttrSet(single row-level row: point_count/format/bbox_min_{x,y,z}/bbox_max_{x,y,z}, 8 columns total), report: PclReport }`. The f64->f32 conversion happens at the loading layer (subtract first, then convert, f64 intermediate precision); render/pick have zero dependency on this crate (dependency arrow io->core is one-way). The whole cloud = one entity, one DrawPoints (the mount obligation on the engine side, management domain = outside items, same lineage as CAPI-18).
