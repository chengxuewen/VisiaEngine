# visiaengine-geo behavioral contract (SDD)

> Clause header lines `## GEO-NN:` are the traceability anchors; tests carry `// spec: GEO-NN`.
> Fixture: `resources/data/park.geojson` (self-produced in this repo: 2 polygons [one with two holes] + 1 line + 1 MultiPoint + 1 point = 5 features, simplestyle properties).
> Projection semantics: the EPSG:3857 definition is itself a **spherical approximation** (auxiliary sphere WGS84 a=6378137); the ~8.7 mm deviation from ellipsoidal Mercator at 50°N is standard-definition behavior, not an implementation error. Datum assumed WGS84 (mandated by RFC7946).

## GEO-01: parses_feature_count
`load_geojson("resources/data/park.geojson")` -> 5 features, name order preserved (buildingA..lamp).

## GEO-02: webmercator_known_points
Forward/inverse reference points on the spherical form: (0,0)->(0,0); (180°,0)->x=20037508.342789244; (10°,50°)->(1113194.9079327357, 6446275.841017158)±0.01m.

## GEO-03: polygon_exterior_holes_preserved
buildingA -> exterior ring 5 points (closed) + 2 holes with order and point counts preserved, rings not merged.

## GEO-04: line_and_multipoint_kinds
road -> Line(len 4); grove -> MultiPoint(len 2); lamp -> Point; kinds not mixed.

## GEO-05: layer_bbox_union
park layer bbox = 3857 union of all features: x∈[261658.6,261704.2]±1m, y magnitude band (5.65879e6,5.65891e6) (fixture construction values).

## GEO-06: malformed_err_not_panic
park bytes cut in half -> GeoError::Parse, zero panic.

## GEO-07: geometrycollection_flattened
Top-level GeometryCollection{Polygon,Point} -> 2 features (Poly+Point), the GC itself does not become a feature.

## GEO-08: extreme_latitude_rejected
lat=89.0 -> GeoError::InvalidCoord (|φ|>85.0511° is the 3857 divergent domain).

## GEO-09: fill_area_matches_polygon
Unit square ring [-1,1]² tessellated -> total triangle area ≈4 ±5%, the barycenter (0,0) falls inside some triangle (with edge tolerance).

## GEO-10: hole_interior_uncovered
Square ring with a central 0.4² hole -> the interior sample point (0,0) lies inside no output triangle.

## GEO-11: stroke_expands_to_width
(0,0)-(10,0) line, explicit stroke_width_m=2 (the default 3 asserts no concrete value) -> output y span 2±0.1, x span 10±0.1 (butt caps).

## GEO-12: simplestyle_six_keys_parsed
Six keys: fill / fill-opacity / stroke / stroke-width / marker-color / marker-radius; colors accept #rrggbb and rgb(r,g,b) (parse product = **raw sRGB-domain values**, linearization lives at the backend upload throat [CORE-16] — zero conversion in the geo domain). park assertions: A fill=[1,0,0,1] α=0.8, road stroke=#808080, lamp marker=#228B22.

## GEO-13: missing_props_default_style
Missing/unknown keys (e.g. "color") -> default style (fill blue [0,0.45,1,1], stroke white), no error.

## GEO-14: tessellate_input_is_origin_local
D7 discipline turned into an interface: `tessellate` input is **origin-subtracted local coordinates** (the engine provides `GeoKind::shifted` as the world->local primitive, consumers must not hand-modify vertices); a (1e7,0)-offset square ring localized and tessellated ≡ the origin square ring tessellation (corresponding vertices <1e-3 meters).

## GEO-15: lenient loading = typed drop counting
`parse_geojson_lenient(bytes)` -> `(doc, LoadReport)`: dirty geometries are counted at the point of origin in five classes — out_of_bounds/non_finite/unsupported/null_geometry/nested_collection (mapped from typed GeoError variants, not reverse-parsed from reason strings); `total_dropped()` = sum of the five; document-level errors (syntax/IO) still Err for the whole file. `parse_geojson_with(bytes, RepairPolicy)` is the explicit-policy entry.

## GEO-16: legacy entry FastFail semantics frozen
`parse_geojson` = `RepairPolicy::FastFail`: the first dirty geometry makes the whole document Err (the GEO-08 rejection surface kept as-is); `geometry:null` features keep the old silent skip (not counted); on purely clean input both entries give identical features results and an all-zero report.

## GEO-17: feature properties enter columns, rows align with flattened parts
Every successfully parsed feature part (counted per part after GC flattening) occupies one `AttrSet` row, `doc.attrs().len() == features().len()` always holds; `geometry:null`/dropped dirty parts occupy no row (failure precedes add_row). JSON scalars (number/string/bool) enter the matching typed column; object/array/null values are skipped and do not enter columns. GC sub-part rows copy the feature-level name/props. Host query accessors: `doc.attr_f64/attr_str/attr_bool(feature_idx, name)`.

## GEO-18: styles read via typed columns
The six simplestyle keys are resolved through `style_from_attrs(&AttrSet, row)` — the `style.rs` module has zero serde_json dependency (the parse boundary is fixed at lib.rs, machine gate `pixi run gate-style`); the six-key output matches per-key the pre-columnarization semantics (regression net = the existing GEO-11/12/13 assertions). The convenience entry `parse_style(json)` runs the same implementation through a one-off AttrSet.

## GEO-19: v8 paint alias fallback (implementation of the D9 ruling)
Each style key accepts a MapLibre v8 static paint alias fallback when its simplestyle primary key is absent: fill->fill-color, stroke->line-color, stroke-width->line-width, marker-color->circle-color, marker-radius->circle-radius (fill-opacity read directly under the same name). **Primary key takes precedence over the alias** (when both exist the primary key wins, rule fixed). Value formats reuse the GEO-12 color parsing and GEO-13 numeric semantics; v8 expressions/stops/data-driven are explicitly outside this clause's coverage (D9 boundary).

## GEO-20: scalar->color-ramp extension keys (materialized at parse time)
When `visia:color-column` (property column name) + `visia:color-lo/hi` (numeric interval) + `visia:color-low/high` (endpoint colors, GEO-12 format) are all present and the interval is valid: the column value is linearly interpolated t=clamp((v−lo)/(hi−lo)) and materialized into `style.fill`/`style.marker_color` (zero render changes — per-feature constant-color form). Any missing element / `hi<=lo` / referenced column absent or non-numeric -> silently disabled (six-key results preserved). The `visia:` prefix is a self-owned extension namespace outside the D9 boundary; per-vertex GPU LUT (DEM/point-cloud fields) is not in this clause and stays in the backlog.

## GEO-21: polyline planar distance
`planar_distance(&[Vec3]) -> f64`: 3857-meter planar polyline length (sum of segments between adjacent finite points). Non-finite points are skipped and form no segment (defense against NaN propagation at large coordinates); empty/single point = 0. **Semantics frozen**: the Mercator planar distance carries a cos φ distortion with latitude — geodetic semantics belong to "coordinate-system completion" (Alpha roadmap), this clause does not overreach.

## GEO-22: ring planar area
`ring_area(&[Vec3]) -> f64`: shoelace absolute value, direction-agnostic (both GeoJSON winding conventions yield the same value), same value whether or not the closing point is duplicated. Vertices <3 or any non-finite point -> 0 (no guessing at incomplete polygons).

## GEO-23: parse domain panics on no input (property surface)
`parse_geojson` (FastFail) and `parse_geojson_lenient` over **arbitrary byte streams and arbitrary f64 coordinate domains** (incl. NaN/inf/out-of-range/illegal JSON structure): zero panic — Err/drop are legal exits, panic is a contract violation. proptest 512 cases × 3 properties coverage (random stream / all-f64 coordinates / clean-domain dual-policy consistency: with no dirty parts total_dropped==0 and identical part counts).

## GEO-24: GeoPart output switch (4de)
`tessellate -> Vec<GeoPart>`: `{Fill(TessPart), Strokes(Vec<LineStrip>), Markers(Vec<Marker>)}` — lines/points **no longer CPU-expand into strips/quads** (`tess_stroke`/`quad` retired, `PartKind` shrinks to `{Fill}`). Neutral types (geo has zero render-IR dependency, the layering invariant; the expansion conversion lives at the consumer = the three-sites-isomorphic note pending extraction after 4de). `LineStrip{pts (adjacent points = segment, rings output closed with duplicated first/last point), color:[f32;3], width_px}` / `Marker{pos, color, radius_px}` — **px unit semantics [adjudication point a correction]**: MapLibre stroke-width/circle-radius mean pixels natively, the old `_m` fields were a misreading; style key names (incl. D9 aliases) untouched, `stroke_width_px` default 1.5 (the former 3.0 world units re-interpreted back to the correct unit), `radius_px` default 4.0, stroke-width value floor 0.5px. `stroke_width_px==0` -> no Strokes part emitted (stroke-off semantics preserved faithfully).

## GEO-25: text style three keys (after S2/B2)
`StyleRecord +{ text_field: Option<Box<str>>, text_size_px (default 14), text_color (host sRGB plane = same discipline as GEO-12, linearization at the consumer throat CORE-16) }` (**Copy->Clone**, zero ripple measured). Parsing (style_from_attrs, the v8 layout domain has no simplestyle counterpart = primary key is the canonical form): `text-field` values unwrap braces `"{prop}"` -> column name; **a bare string = constant text**; the resolution rule sits at the consumer (CAPI-22 / engine mount): **property column exists (checked via `attrs.names()`) -> column semantics** (column exists but the row value is missing = **skip, do not fall back to the literal**, preventing `"{name}"` from leaking as text); column absent -> literal. `text-size` clamp [1,256]; `text-color` reuses parse_color. Output form: the loading side produces `LabelMark`s merged into **one table, one DrawLabels per doc** (origin = layer center, anchor = bbox center + **center alignment** (MapLibre default), z=0.1 always top layer). No font loaded = the entire segment collects nothing and outputs nothing, no error (dormant form, paired with the CAPI-22 timing gate).

## GEO-26: fill-opacity -> full transparency chain (④)
Style `fill-opacity` (GEO-12 primary key + domain clamp [0,1]) -> `with_alpha(fill, op)` -> part material `base_color.a` -> (REND-36 single predicate) transparent pipeline + painter pass (WGPU-28). **park.geojson current lock**: buildingA α=0.8 red fill on gray ground = a linear-domain composite value that is computable (checkerboard-gate same family), WGPU-11 multi-part fitting gate **zero re-recording** before and after alpha takes effect (measured position within the conservative-threshold margin, this clause is the reference re-measurement record). A per-vertex alpha path does not exist (part-level material single value); stroke/marker are always opaque (fs hard a=1.0, semi-transparent lines/points = a separate ruling in a later band).
