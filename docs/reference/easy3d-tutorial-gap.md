# Easy3D tutorials band-gap analysis (benchmark archive)

> Generated: 2026-09-15 (the cmake-example-targets round S5 archive, C-3 ruling = yes).
> Source: read-only dissection of `.refinfo/Easy3D/tutorials/` (52 items, leading numbered bands 1-7) + the 11 E-series items of this repo.
> Purpose: the case-formation basis for later render/feature rounds. **Citations describe mechanisms only; MediaServo/Easy3D internal numbering is forbidden in new clauses**.

## Band mapping, current state

| Easy3D band | count | our counterpart | gap (case candidates) |
|---|---|---|---|
| 1xx data types (cloud/mesh/graph/poly × base/connectivity/property/IO) | 15 | E201/E202/E203 **+ E701 property-read segment (D13 landed)** | ~~AttrSet property-query example~~ ✅ 2026-09-15 the three C entries; **point-cloud example = skipped by user ruling** (overlaps E202, no proprietary source); GeoJSON write-back IO |
| 2xx viewer/UI (default/imgui/wx/Qt/multi-viewport/camera interpolation/real camera) | 7 | E101/E301/E702/E703 | multi-viewport (one scene, two views); camera-interpolation roaming; HUD overlay |
| 3xx drawables (texture/scalar field/vector field/IMG viewer/section/animation/text) | 12 | E201 texquad | **cross-section clipping** (clip plane); scalar-field coloring; text rendering (depends on the font plane, long-term) |
| 4xx interaction (picker×2/point select/virtual scan/object manipulation/collision) | 6 | E401/E203 (+E402 empty number reserved) | hover/multi-select UI (= the standing E402 candidate); object manipulation (gizmo -> the D7 rebase surface); collision query |
| 5xx rendering (AO/hard shadow/soft shadow/transparency/EDL/depth map) | 6 | E501(PCSS) | semi-transparent sorting (blend order = a pipeline topic); EDL point-cloud lighting; SSAO (= batch 4g backlog) |
| 6xx algorithms-core (subdivision/convex split/curves) | 3 | E202 embedded tess | standalone subdivision demo; curve band (buffered lane-line class) |
| 7xx algorithms-cloud (normals/reconstruction/plane extraction) | 3 | E203 (plane measurement) | normal-estimation demo; plane-extraction demo |

## Easy3D structure adopted (this repo's cmake surface)

Central function (`visiaengine_setup_examples`) / name=file=target=ctest (zero mapping tables) /
FOLDER band grouping / an explicit contract line per example / a two-tier optional-dependency gate (Qt+X11) / registration is obligation
(the bidirectional glob<->table check = a hard configure error, versus their comment-style disabling).

## Already overtaken (do not regress)

- Easy3D's 52 examples consume argv zero -> structurally non-CI-able; our whole family supports `--frames N` +
  the same ctest entry + a 77-Skip contract on machines without X.
- Their tutorials carry zero add_test; we register 13 ctest entries, resident in the ci segment.
- Their resource paths = compile-time absolute macros + runtime tree-walking; we hold the repo-root chdir contract (stub single point).

## Suggested case-formation order (merged into the Alpha queue view)

1. point-cloud example + AttrSet property example (data planes complete = pure demonstration cost, filling the 1xx band holes)
2. cross-section clipping (a 2.5D GIS hard need; the pipeline depth/stencil surface is in place)
3. E402 hover/multi-select (closing the interaction band, the empty number is reserved)
4. semi-transparent sorting + SSAO (merging with the 4g backlog)
5. multi-viewport/camera interpolation (roaming-demo direction, batch 7 site material is reusable)
