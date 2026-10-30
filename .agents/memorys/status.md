# VisiaEngine Status

**Last updated**: 2026-10-29 | Phase: MVP closed + rendering hardening / host embedding / packaging band | Branch: main (a clean worktree is the normal state)

> The full history of the predecessor MediaServo (the D/PIT/C numbering system, the crate matrix, Phase records) lives in the local archive `.refinfo/MediaServo/.agents/memorys/` (gitignored, not distributed with clones). This directory accumulates VisiaEngine history from scratch starting 2026-09-03.

## Overview

| Item | State |
|----|------|
| Tech stack | ✅ Rust core + wgpu rendering (whitepaper v0.1.0, settled 2026-09-03); **backend = D4 final ruling: wgpu direct use, self-built pipeline `visiaengine-render-wgpu` (Bevy not adopted)**; SDK form (C API FFI); Open Core business model |
| License | ✅ landed: MIT OR Apache-2.0 dual-license canonical files (LICENSE-MIT/LICENSE-APACHE); the irrevocability promise is in the README |
| Source | `crates/` (Rust kernel + IO leaves + ffi) + the wasm binding + the examples shell package form the workspace; counts are gate-quoted — run `cargo metadata`, `pixi run test` or `scripts/spec-trace.sh` rather than copying numbers here |
| Positioning | ✅ multi-dimensional spatial visualization engine (unified 2D/2.5D/3D; GIS / digital twin / AV simulation / BIM), not a game engine |
| Agent toolchain | ✅ single source in `.agents/` + generated host layers; 16 instruction files; CodeBuddy rules generated and gated by `gate-agents`; MCP: codegraph / playwright / serena (github retired by D42); skills gated by `gate-skills` |

## Phase status

| Phase | State |
|-------|:----:|
| 0 project init (config neutralization / whitepaper / architecture baseline / reference library / licenses) | ✅ |
| 1 MVP | ✅ closed — MVP content + GeoJSON slice + host embedding + C ABI (49 entries) + install tree delivered; see README Development Status for the gate-quoted surface |
| 2 Alpha / 3 Beta / 4 1.0 | — whitepaper roadmap |

## Next steps

1. **S5 — Qoder rules discovery**: confirm the Rules panel, remove the two probes, then generate `.qoder/rules` (`2026-10-28-agent-audit-followups.md`).
2. **packaging follow-ups**: install tree / corrosion re-evaluation / npm-pip (`packaging-round.md`).
3. **repo-docs-web**: still backlog (`2026-10-28-repo-docs-web.md`).
4. **keep the gates green**: `pixi run ci` is the only gate while the GitHub mirror stays on standby.

## MVP content-round baseline (2026-09-03, G1-G4)

`pixi run ci` all green · spec-trace **66↔66** (five namespaces CORE/REND/WGPU/GLTF/GEO) · cube golden 7/7 on this machine's lavapipe real runs · L2 three smokes wired (where they run green = CI standby) · 11 commits on the surface (planned 9 + S2/G1 supplemental + G4 split-b, K5 elastic accounting) · PIT-5 filed (wgpu [0,1] silent clipping)

## GeoJSON slice baseline (2026-09-03, H1-H4)

geo crate 16 tests (GEO-01..14) · D7 full chain: `tessellate` takes local input (the `GeoKind::shifted` primitive) + `rebase::compose_mvp` f64 subtraction + far-origin cube/geo pixel-level end-to-end verification · Web Mercator spherical formulas (the EPSG:3857 definitional semantics into the SDD) · geo_viewer example + 4th smoke lane · 12 commits (planned 8 + RED supplemental / H1 close-out one each, covered by the elastic clause) · three incidents recorded honestly in messages (parallel edit of two view_rotation / a contract not written within its batch / the 51 claimed count)

## Plan v1.1 batches 0/1 baseline (2026-09-03, Easy3D-borrowing port, work-start round)

- batch 0: three-tier testing (T1/T2/T3) + example gate + D8 pre-registration into rules/decisions (`0464fd2`)
- batch 1a: LoadReport+RepairPolicy five error classes (GEO-15/16) + GLTF-09 mode filter (**incidentally fixing the POINTS fake-triangle legacy bug**) (RED `6892b81` + GREEN `07999cd`)
- batch 1b: testdata→resources/data migration, 14 synced sites, grep zeroed (`229f77f`)
- baseline: 66↔66 · 68 tests · ci exit=0 · golden no SKIP; boundary facts: JSON numeric out-of-domain rejected at the document layer = dual-strategy Err (clause note); non-finite coordinates unreachable at the parse entrance (the field keeps the web_mercator direct-call path)
- plan doc `.omo/plans/easy3d-adoption-plan.md` (v1.1 approved); next = batch 2 planning round (host embedding, incl. D8 finalization) or batch 3 (∥3c→3b) + batch 3.5 interaction slice

## Batch 3a baseline (2026-09-03, AttrSet attribute-column round)

- core::AttrSet (CORE-11/12/13): three-type closed column store, row-alignment invariant, missing ≠ zero, first-write sets the type
- geo (GEO-17/18): props no longer dropped after parsing (the host query port opens); styles read via typed columns, style.rs zero JSON dependency (the gate-style task enters the ci chain)
- baseline: **71↔71 · 74 passed · ci exit=0 · golden 9/9 no SKIP**; RED `070ce59` GREEN `28c250f`
- two lessons recorded honestly: the GREEN regression net caught the feed's plain-item branch bypassing the accept classifier; the first grep-gate predicate was tripped by its own comment text (predicate narrowed to code shapes)
- 3c ruled: **D9=B** (v8 paint alias subset, primary keys first / expression boundary frozen) → GEO-19 implemented (the `str_first/f64_first` fallback mapping)
- baseline refresh: **72↔72 · 76 passed · ci exit=0 (incl. the gate-style segment) · golden 9/9 no SKIP**
- 3b implemented (revised downgrade to CPU lerp, GEO-20 + the three-color render family golden): **73↔73 · 80 passed · ci exit=0 · SKIP 0**; batch 3 fully cleared
- lesson appended: grep -c zero-match = exit 1 breaks && chains (the 3b commit chain broke once; confirmed on re-run)
- 3.5a/b implemented (CORE-14/15 + REND-21/22 ray family, MT intersection + screen port): **77↔77 · 84 passed**; RED `929b3b6` GREEN `0db9d87`
- GREEN wins: a real bug of slab tmax variable misuse + two test-side ones (zoom default / f32 aspect noise); PIT-7 filed (deny network-flakiness false reds)
- 3.5d implemented (GEO-21/22 plane measurement, lint folded in via amend); 3.5c/e implemented (REND-23/24 pick_meshes + WGPU-12/13 highlight + **depth surface filled in** + E401/E403 + dual smoke lanes)
- baseline refresh: **83↔83 · 90 passed · ci exit=0 · golden no SKIP · smoke-pick/measure real local runs green**
- WGPU-13 legacy fix: the mesh pipeline previously had no depth buffer (single-object goldens masked it) — multi-entity occlusion is correct from here on
- interaction slice (all five 3.5 items) closed
- **batch 2 approval round in execution (2026-09-11)**: I0+J0 environment round `a163d0d` (host/wasm dual spike environments, wasm full-tree green with zero cfg fixes, PROBE-OK) → I1 `c68e57c` (handle/fence/thread/error-string/input-port — six-clause tests) → I2 `6bdd7e7` (the real out-frame chain + **headless readback row-alignment legacy bug fixed** + gate-abi 14/14 + demo 'OK capi headless'); baseline **88↔88 · 98 passed · ci=0**
- discipline #17 fourth-and-corrected×4 (two amends this round: mixed-scope commit / claiming green while clippy red / the 87 typo) — commit-message numbers must be pasted from same-terminal measured output
- next: I3 attach spike (x11 SurfaceTarget) ∥ I4 input/pick demo → batch 7 J1; push awaiting instruction

## Qt widget round baseline (2026-09-14, v1.3 Q1-Q3 all landed)

- Q1 (the `render_spec` lock): viewport switched to dims with three-surface assertions going green directly — "behavior existed, lock absent" recorded honestly
- Q2/Q3 (this commit): widget.hpp (header-only, zero moc / the PaintOnScreen trio) + qt_app real window (texquad+park) + smoke-qt tri-state; the QTimer 60Hz pull pump running through = **D8 trigger ① re-evaluation: maintain suspension** (no frame-skip negotiation demand; next trigger = video/decoded-frame-aligned hosts)
- conda Qt double-pit filed: qt-main is actually Qt5, the proper name is qt6-main; QX11Application not packaged → Xlib self-held Display* (the cross-connection xid contract fulfilled)
- baseline refresh: **143 passed · ci ten segments ✓ · SMOKE-QT ✓ (real window @:0) · 108↔108 · 14/14**
- Alpha remaining balance: npm/pip packaging (incl. the cmake install tree / corrosion re-evaluation D-11), real PBR, site, E402, wayland/Win/mac matrix

## CMake engineering-layer baseline (2026-09-14, C1-C3; Qt round v1.3 consumption surface pending)

- plan `.omo/plans/visiaengine-cmake-project.md` v1.0 (Momus OKAY 0B; the six→ten segment wording fixed) + Qt round v1.3 (build surface made public)
- C1: root facade (version single-source from Cargo.toml, hard-error lock) + SDK three-tier resolution (AUTO/PIXI/SYSTEM/path × Rust/Qt dual channels) + presets; **measured semantics hole during review**: the pixi-activated PATH made SYSTEM falsely pass → anti-pollution dual filters (rejection messages name the culprit); tools + cmake, ninja, gxx_linux-64 (R1 measured: add 2.6s / first install ~3min)
- C2: cargo-step (repo-level shared target-dir, rerun 0.16s) + umbrella `visiaengine::capi` (rpath injection, no LD_LIBRARY_PATH) + in-tree fake Config + ctest probe — **first run caught a real gap: the handwritten header had zero extern "C" guards (pure-C demos' three green rounds masked it); after the fix the Qt-side pre-embedded debt cleared**; the links account not opened, evidence-based write-off (cargo requires a build script)
- C3: cmake-smoke tri-state into ci (segment 10; machines without cmake SKIP) + gate-docs ⑤ facade purity (≤60 lines / no compile-rule wording) + README dual entry
- baseline: **142 passed · 108↔108 untouched · ci ten segments ✓ · GATE-ABI 14/14 · ctest 1/1 · dual negative-path messages ✓**
- interlude honestly: cmake message wrapping made a full-sentence grep assertion miss it (fixed to short-phrase assertions); the preset-schema field `output.outputOnFailure` wrong twice; calling conda cargo directly from a non-activated shell gave a fake rustc-missing (PATH semantics — the same lesson exists in the pytest family)

## Batch 5 docs/tutorials landing baseline (2026-09-14 close-out)

- plan `.omo/plans/visiaengine-docs-tutorials.md` v1.0 (Momus OKAY → approval → T1-T4 all landed; correction notes: cbindgen → handwritten header + signature gate, mdbook not done = site round)
- T1 (`1031eef`): E-number renames, 11 items (E402 kept as a reserved gap; measure E403→E203 returns to its band) + E301 header-comment paste residue fixed + tutorials.md
- T2 (`bfd2f57`): **E901_twin_city** vertical-slice seed (park real-scale 93×31m × 256 towers × PCSS × PNG human check; the nine smoke lanes +1); caster slope_bias → 0 hardened (the theoretical self-shadow plane removed; measured no difference in this scene)
- T3: gate-docs four checks into ci (coming online it immediately caught README 83≠108 — the gate's value proven) + README/architecture v0.2/AGENTS zero drift
- baseline: **142 passed · spec-trace 108↔108 · ci exit=0 (four gates) · smoke nine lanes ✓ · SKIP 0 · drift wording grep=0**
- lesson: PIT-10 filed (scene human-check three-probe lesson: the pixel predicate was right but the lit-area position assumption wrong — low sun angle × building height → a legitimately long shadow band in the foreground)
- **queue after batches 4/5**: 4g/4h SSAO/DDP/big-image striping/hot-reload = backlog (the sorter promises nothing); Alpha tier = Qt widget round / npm-pip packaging / real PBR (GGX) / site infrastructure / E402 interaction — all await user adjudication; R2 ruled = A accept status quo (user order 2026-09-15; package-size increment = the reasonable cost of the texture surface; the exit path stays closed); push awaiting

## Batch 4f Shadow/PCSS baseline + batch 4 close (2026-09-14)

- plan `.omo/plans/visiaengine-shadow-pcss.md` v1.0 (Momus OKAY 0B, 13 citations measured to hit → approval → P1-P4 all landed)
- P1 (`73c9c39..`→`7845601`): REND-31 ShadowSetup / Frame.shadow Option (33 construction points, 21 real insertions as fallback) + the PIT-5 depth-domain lesson promoted to contract (the rig-triple dedicated path)
- P2 (`f7e8..`→`3361392`): caster pre-pass (no-color-target dual pipelines ShadowMesh/ShadowInst, bias{-1,-1.5}, 1024² map lazily built); View block 176B (light_view_proj per-draw compose_mvp reuse [adjudication point a]); the three receiving bgls always bind params/map/cmp-sampler (resident dummies = dynamic branching, not layout branching); params-off loads the old LIGHT at the same position type = the sp.a.rgb retirement path bit-unchanged (R4 guardrail, the golden suite self-proves all green)
- P3 (`f91f759`): PCSS three stages [E3D:B3 single-pass port] (8-ring blocker textureLoad → penumbra clamp[texel,0.06] → 16-tap Poisson compare); monotonicity double-sided lock soft>2×hard ∧ hard<400 ∧ soft<6000
- P4 (this commit): shadow_demo example (the 16×16 city, dark=22647 assertion) + GL-degrade guard (wgpu-30 reality: GLES = a Gl sub-version single variant) + smoke's eighth lane
- **batch 4 fully closed**: 4ab✓ 4c✓ 4de✓ 4f✓ → batch 5 (docs landing) force-trigger condition met; SSAO/DDP/big-image striping/hot-reload = 4g/4h backlog (the sorter promises nothing)
- incidents honestly: two naga hard evidences noted into the archive (`0u32` literal suffix rejected → i32 inference + u32() indexing; duplicate match-arm masking = the triple silently unpropagated, located via the binding-missing error); the splice cut point ate a regex lookahead leaving an orphan function (cut out); PIT-7 third sighting (audit retried, 2 green); clippy double-clean (needless_borrow/collapsible_if/chunks_exact — third time same pattern)
- baseline: **142 passed · spec-trace 108↔108 · ci exit=0 · GATE-ABI ✓ · smoke eight lanes ✓ · SKIP 0 · web-check ✓ (1069627B/429401B same magnitude)**

## Batch 4de VS-expansion family baseline (2026-09-14 close-out)

- plan `.omo/plans/visiaengine-vs-expansion.md` v1.0 (Momus OKAY 0B → approval → N1-N4 all landed)
- N1 IR (`73c9c39..`→`213edbc`): REND-29 Frame.px_world_scale (26+ construction points compiler-backstopped) / REND-30 StrokeSeg 48B + PointMark 32B Pod layout locks + dual commands + default rejection
- N2/N3 (`6d8559c→8ac0ffb`, `aee0bc4→20edd3c`): View block 128B (mat+right+up+px_scale+eye_local; the triangle families read only the first 64B = a constructional zero-regression proof: goldens zero re-record); vs_stroke perp expansion [E3D:B4] + degenerate fallback right + bias{−1,−1} coverage [E3D:B7③]; vs_point/fs_point screen-basis splat + discard disk = real round points
- N4 (`40115b3`): geo GeoPart output switch (GEO-24: tess_stroke/quad retired, px-unit correction stroke_width_px=1.5 / radius_px=4, key names / D9 aliases untouched); capi extra_cmds + ortho px_scale=2·hw/W exact path; same for geo_viewer
- visual wins: strokes no longer swallowed by same-depth fills; points go square→round; line width scales constantly with no rebuild (a single Frame field)
- incidents honestly: #17 fifth offense intercepted (the N3 first version committed unformatted + a null-value false claim → folded in via a second amend, noted in the message); PIT-8 third stumble (strokes position window by intuition → corrected via probe; points' 76° pitch got it right once)
- baseline: **136 passed · spec-trace 105↔105 · ci exit=0 · GATE-ABI ✓ · smoke seven lanes ✓ · SKIP 0 · web-check ✓ (1053859B/424099B same magnitude)**
- batch 4 queue: **4de ✓ → 4f Shadow/PCSS ([E3D:B3], the PIT-5 depth-domain conversion written as a clause) awaiting its planning round**; batch 5's trigger was met long ago (examples=7 + the 4de gate examples)
- extraction-position note: LineStrip→StrokeSeg same shape in three places (engine/geo_viewer/geo_pipeline) — extract the helper when a fourth consumer appears (comment anchor inside tess.rs)

## Batch 4c instancing+bench baseline (2026-09-14 close-out)

- plan `.omo/plans/visiaengine-instancing-bench.md` v1.0 (Momus 0B → approval → K1-K3 all landed)
- K1 IR (`73c9c39`→`3fe7ccf`): Instance 32B Pod (offset/height/color + pad, triple layout locks) / DrawInstances same shape as D7 / create_instances default rejection
- K2 pipeline (`3f08d2d`→`d5062a5`): Variant{Flat,Textured,Instanced} three layout families (WGPU-14 numbered, zero breakage); storage binding5; vs_inst bottom-aligned extrusion (the axis-aligned-normal zero-error argument); silent-skip sealed
- K3 products (`4d34bd7`+`aeb5a37`): bench_twin headless 100k (release on this machine's lavapipe: upload 3.8ms / frame 217ms / single draw); pick_demo --bench settles the 3.5b debt; scripts/bench.sh → resources/bench/*.json (degradation >20% prints red, non-gating); unit_box_mesh made public
- legacy win: the root fix of the Bgra/Rgba mismatch in three window examples (I3 missed the sync, surviving 3 rounds — PIT-9 filed: the seven smoke lanes must run before any merge)
- baseline: **124 passed · spec-trace 100↔100 · ci ✓ (audit = PIT-7 retry once then green) · GATE-ABI ✓ · seven smoke lanes real runs ✓ · golden SKIP 0**
- batch 4 queue state: 4ab ✓ 4c ✓ → **4de (VS-expansion family: line width + point splat + polygon-offset + real points) awaiting its planning round**; 4f Shadow/PCSS follows; batch 5 docs-landing trigger met (examples≥6: now 7) — landing is forced at latest when batch 4 completes

## Batch 4ab material/texture slice baseline (2026-09-12 close-out)

- M1 io-gltf (`2f293fe`) · M2 RED→GREEN (`4449fb3`→`c9d80f3`) · M3 (`434e56e`+`7b92676` clause bodies synced)
- pipeline: 32B material block dual layout (Flat per-pixel, zero regression = constructional guarantee) + upload_texture (256 row-stride zero-padded) + textured dual pipelines keyed (format,textured); REND-25/26 + WGPU-14/15 clause bodies registered (specular = mock-up Lambert coefficient, not GGX)
- capi/example texture chain implemented; texquad.glb = the io-gltf builder #[ignore] emit, regenerable from the true source; gate-abi dual demo paths
- baseline: **117 passed · ci exit=0 · spec-trace 97↔97 · golden SKIP 0 · GATE-ABI 14/14 · web-check ✓**
- **R2 threshold breach awaiting adjudication**: WEB-SIZE raw=1062807B gz=431472B (baseline 733KB/294KB; gz **+47% > 15%**); the escape route "texture decode native-only, wasm takes raw RGBA bytes" was **not enabled unilaterally**; numbers recorded. **2026-09-15 user ruling = A accept** (the "texture decode native-only" exit stays closed)
- lesson: PIT-8 filed (the first-version predicate of pixel assertions = the intersection of geometry coverage × filtering × channel multiplication chains; probe-measure first, then write assertions; one stumble in each of two rounds)

## Batch 2 + batch 7 landing baseline (2026-09-11 close-out)

- batch 2 I0-I4 all cleared: capi 14 entry points (handle/fence/thread/error protocol/input/attach) + dual C demos real-ran + gate-abi/gate-trace into ci
- batch 7 J0-J3 all cleared: visiaengine-wasm standalone crate (isolation ruling: the no_mangle × bindgen wasm-ld mutual exclusion nailed) · WEB-BUILD dual glue · CAPI-09 mirror · demo page + T3 · **733KB raw / 294KB gz**
- baseline: **90↔90 · 100 passed · ci exit=0 · SMOKE-X11 ✓ web-check ✓**
- D8/D10 finalized (revision surfaces fully recorded); the README MVP four-piece account closed
- remaining balance: push awaiting (local-ahead keeps growing) · batch 4 render strengthening / batch 5 docs / batch 6 infrastructure standing by · the CI liveness of smoke-x11/web-check = the mirror-repo day

## CMake example-entry round baseline (2026-09-15, S1-S5 all landed)

- team mode: probe A (Easy3D anatomy, 6 adopted 5 rejected) + probe B (gap matrix G1-G8) → plan v1.0 → Momus OKAY (0B/3A fixed into v1.1) → user approval (C-1 stub / C-2 E703 / C-3 archive)
- S2 `be4f970`: VisiaEngineExamples.cmake contract table + glob⇄table dual validation (negative paths measured both directions) + ve_example_launcher.cpp (chdir / DISPLAY-77 / argv passthrough); the S1 spike proved cargo unix has no hash hardlinks, same inode
- S3 `59f5b61`: E701/702 natively cmake-ized (ve_real_ + OUTPUT_NAME canonical names + umbrella rpath); qt_app → **E703_qt_viewer** joins the 7x band; tools +xorg-libx11/xproto; the G8 revision recorded honestly (the cc path = a gate attribute, kept)
- S4 `ac60a07`: cmake-smoke `-LE display` (ci pops no real window); presets v7 + the qt-pixi testPreset (filter.exclude.label = a string regex); **the real gap = the enable_testing order** fixed (PIT-15); the tutorials.md IDE-run section
- baseline: **143 passed · 108↔108 · ci ten segments ✓ (PIT-7 5th sighting: audit TLS flapped, 3 reds, backoff, green) · ctest bare 12 registered (@:0 11+probe all Passed 2.4s; without X 6P/5S) · qt-build 13 registered, preset runs 7 · E703 ctest real window ✓ · SMOKE-QT ✓**
- new pits filed: PIT-15 (enable_testing order silently drops), PIT-16 (enable_language inside function doesn't propagate; presets authority = the shipped-with-package schema.yaml offline canon); D-12 finalized
- gap table archived at docs/reference/easy3d-tutorial-gap.md (7-band mapping + a 5-item entry-order proposal)

## capi attribute-read round baseline (2026-09-15, D13)

- team mode: readiness probe (point cloud = PURE-DEMO but overlaps E202 → user ruled skip; attributes = split verdict → the capi version approved) → plan v1.0 → Momus OKAY (0B/5A fixed into v1.1) → final C-1=3 ports / C-2=256B / C-3=E701 appended section
- S1+S2 `a5a830e`: Engine geo_docs + attr_of (the **u64 pose pass-through key**, zero decode) + FFI attr_f64/str/bool (1/0/-5 zero-partial-write parameter tables) + twelve dependent places (header / version 0x00010001 / gate-abi 17 / CAPI-01·02 revision sentences / README / AGENTS / wasm mirror) + clause 10/11/12 dual-target tests
- S3 `17e4e35`: E701 attribute closed-loop section with park real keys + deliberately reading a missing key = the missing-intact living billboard; dual paths (manual cc / cmake stub) measured to identical output
- new pit: PIT-17 (harness shared-pid tmp race; pid+nanosecond dual suffix); PIT-7 6th sighting (audit TLS try3 green)
- baseline: **151 passed (143+4 inline+4 FFI; the total actually computed from the ci log's 44 targets; the S2 commit message wrote 147 = a local count — #17 sixth offense, honestly corrected in this commit) · spec-trace 111↔111 · GATE-ABI 17/17 ✓ · ci ten segments ✓ · web_capi_pair_mirror ✓ · no regression in the ctest example family**

## CMake structure round baseline (2026-09-16, v1.3 S0–S5 all landed)

- two team-mode review rounds (red team 27 items + finalization review 10 items, each item user-adjudicated) → plan `.omo/plans/bindings-restructure-cpp-examples.md` v1.3 (corrosion rejected: the eight-tax list + conda first-crab + CTest #13 spinning; the c+ self-written steps adopt its naming grammar)
- final directory shape: `examples/{rs,c,cpp,qt}` (Rust examples = cargo example [[example]] ×9, not bin-ified; C/C++/Qt = cmake real bodies) + `bindings/{c,cpp,qt,js}` (capi/wasm one nest, crates/ = pure core 5) + `cmake/VisiaEngineBindings.cmake` (the R9 registry = argv's only residence; the anti-glob bidirectional hard error)
- run-surface convergence: the unified ctest list (bare 14 items: 9 rs forwards + native + probe); the 9 pixi smokes = forwarding shells (`smoke-rs.sh` carries the "≥1 selected" assertion plugging ctest's empty-match false green); cmake-smoke = tri-state tri-anchor (bare PATH {configure probe + cargo-build_capi + cargo-build_examples}) + three negative paths (SYSTEM / prebuilt missing / install lying) + Xvfb:78 display sub-state (the window family's first automated track; conda lacks xvfb-run = direct-launch form, the PIT-19 machine = PIT-19 clobber note, no false green)
- the C++ surface opens: `visiaengine.hpp` (76-line RAII; the three shells' semantics live in header comments; B9 opens no SDD account) + `visiaengine::cpp`; E801 (SDL3 x11 three frames) / E802 (hpp→readback→PPM) locked at birth; template.c/.cpp skeletons (B8 builds no library, keeping full gate-abi visibility)
- toolchain: presets dual-config EXPORT_COMPILE + the bare testPreset -LE display + `.clang-format` (LLVM+4 follows the mainstream) + `.vscode/settings.json` (presets always) + the `.gitignore` L56 negation deadlock fix (B4) + the install fail-loud guard (S-c β; the tree = a packaging-round hard debt with an acceptance template)
- three new pits filed: PIT-19 (pixi clobber → conda Xvfb XKB death) / PIT-20 (the ghost-API family founded) / PIT-21 (edit residue + grep -E assertions, promoted into edit-safety ###18)
- baseline: **151 passed · spec-trace 111↔111 · GATE-ABI ✓ · GATE-DOCS ✓ (E 14 items three-way) · CMAKE-SMOKE ✓ · web ✓ · smoke 11/11 · the prebuilt retirement verified · worktree clean**
- to-do: push awaiting (local-ahead += this round) · Phase B first band (the E8xx data-band API-gap list → capi port expansion) awaiting · the RA example Run lens human check (T3, risk ④ already demoted to the auxiliary track) · the packaging-round three items (install tree / corrosion re-evaluation / npm-pip)

## B1 data band baseline (2026-09-16, Phase B first band: CAPI-13..16 four ports + the four E8xx examples)

- port surface: visibility (strict 0/1, enum-domain invariance, render/pick filtering) / query / add_mesh (the VeMeshDesc struct_size forward gate, return code + out-parameter catalog, degenerate = zero commit) / remove (same catalog as double destroy, ABA isolation); abi minor = 2 (0x00010002), 17→21 entry points, dual-surface mirror (4 wasm bridges + the d.ts presence assertion) + 4 hpp thin forwards
- examples: E810 attr survey (the zero-new-port acceptance example) / E811 visibility twins (c/cpp — first use of language-suffixed target names) / E812 programmatic add/remove; ctest 14→18 items (17 numbered)
- design reversal logged: add_mesh's first version used 0 as the failure sentinel, colliding with the CAPI-01 clause "entity pose 0 is legal = deliberate ownership" — the return code + out catalog rewritten; two test sentinel assertions corrected per the clause (the interface-design volume of the ghost-API lesson)
- baseline: **160 passed (+7) · 115↔115 · GATE-ABI 21/21 · GATE-DOCS ✓ 17 items · CMAKE-SMOKE ✓ · web MIRROR ✓ · ci ten segments all green**

## lesson-review record (2026-09-16, structure-round + B1-band session batch review)

- new: PIT-22 (verification channel ≠ user channel, the DISPLAY family — the generalization of PIT-18), PIT-23 (the three always-true/swallowing CMake/ctest shapes — the E801 dead-hang case), edit-safety ###19 (pipeline tails eating rc · shipping-broken proven), testing.md "visual-example double insurance" (pixel gate + three self-proofs + the T3 checklist + forwarding-shell assertions), conventions C15 (sentinels check the clause-ownership table first / one parameter serves no two masters), C16 (decision presentation = plain words + diagram + directory + one-at-a-time)
- session mainline: the v1.3 structure round (S0-S5, seven commits) + the FOLDER position rule + E201/E301 interaction root fixes + the B1 data band (RED→GREEN, four ports, 21 entry points, ctest 18 items); baseline 160 passed · 115↔115 · GATE-ABI 21 · ci ten segments
- next queue (unchanged): push awaiting (7 ahead + this commit) → five T3 human checks (E201/E301/E801/test panel/RA lens) → the B2 cross-section planning round awaiting

## E202 gray-screen root fix (2026-09-17, the second sighting of the D7 camera-pose contract)

- root cause: the fitted camera missing in resumed() — DrawMesh world pose = origin+local (D7), the camera target stayed [0,0,0] against a 3857-world-meters park = millions of meters off → full-frame clear color; after an offscreen probe replicated the offending parameters (all four variants empty), the fix: target = layer bbox center + dist/zoom fit at load
- permanent gate: geo_pipeline::golden_geo_window_fit (spec WGPU-11 multi-example same clause, spec-trace 115↔115 unmoved) — the predicate's three self-proofs complete: canary at the offending camera = always 0 px; the fixed path measured 23222px, threshold 12000 (the −48% conservative position)
- same-family sweep: E901/E401/E201/E301 camera positions checked case by case, no collision (the E201/301 data lives in a local system, origin=0 self-consistent); the PIT-22 / double-insurance discipline caught a real case the second time (the gray-screen family's third sighting: E201/E301/E202)
- baseline: 161 passed · ten segments all green · ctest 18 items no regression

## E402 interactive-pick window band (2026-09-17, reserved number promoted + the unit_box winding legacy mine root-fixed)

- E402_pick_interactive: hover orange preview / left-click yellow toggle multi-select / drag orbit / wheel zoom / R clear / Esc quit;
  the placed() pose single source (pick world = render transform, the same matrix = the constructive root removal of the E401 double-bake 64px-shift lesson)
- **legacy mine**: unit_box_mesh born with reversed winding (geometry normal = − declared normal) — with GPU cull default-off all three consumers render zero
  visible complaints; the pick front-face rule (REND-23) misses entirely on first consumption; root fix = index mirroring flipped + the golden-lesson note, 162 passed
  constructional proof of zero pixel regression (with cull off, winding is not an observable)
- gate: golden_pick_window_mirror (spec WGPU-12 multi-example same clause) 512×320 window-shape mirror + central hit-chain reuse
  + outer-corner canary; measured (35,132,62)→(177,147,51) yellow-family flip locked
- registration surface: Cargo.toml / contract table / tutorials / pixi, four pieces + the gate-docs E402 reservation exemption revoked (the real item in place)
- baseline: **162 passed · 115↔115 · GATE-DOCS E 18 items three-way · ctest 19 items · ci ten segments ✓ · SMOKE E402 ✓ (real window)**

## Color + events band (2026-09-17, CORE-16/CAPI-17, plan color-events-band.md K1-K6)

- CORE-16 sRGB↔linear dual ports (f64 intermediate precision, endpoints bit-exact); the full-chain contract = host surface sRGB / IR linear / both GPU ends
  Srgb hardware-encoded; five chokepoints (material block / clear / strokes / points / instances table) + the io-gltf factor
  inverse transforms returned to their right place; the GL-degrade tier = fallback linear pass-through manifests
- domain re-pinned in seven places (all measured first): clear [13,18,25] hardware-rounding self-calibrated canary, shadow light/dark bimodal 95/140,
  texture checkerboard b bimodal 135 (the uv-broken semantics preserved), E501 boundary 128 dark threshold 4000 (measured 19642),
  io-gltf color pins ×2, the geo fit-gate tolerance shape
- **dug out and root-fixed**: lavapipe semantic hard evidence — clear load values interpreted in the linear domain (0.05→63 probe); the render-test band
  domain-fragility lesson (encoding lifts midtones)
- CAPI-17 event port (21→22 entry points, abi 0x00010003): the EvtSink cfg dual forms (native C anchor unsafe-Send /
  wasm Box<dyn FnMut>), monotonic-terminal progress lock / same-instant errors / NULL detaching, three-surface mirror (header / hpp / wasm + d.ts)
- example E 21 items three-way: E704_host_callback.c (event tri-state) / E502_color_tuning (**WYSIWYG four color boards byte-for-byte
  mutual-inverse living proof**, one sampling-column formula error fixed) / E503_stroke_points (constant-width showcase) + the permanent gate golden_zoom_width_invariant
  (two zoom tiers 10px/φ24 cross-tier lock; column sampling = the phase discipline independent of zoom)
- baseline: **167 passed · 117↔117 · GATE-ABI 22/22 · GATE-DOCS E 21 items · ctest 22 items · ci ten segments ✓ ·
  WEB MIRROR ✓**; structural incident self-caught: the four zombie _disp lines left by the E402 band cleared (another sighting of the edit-safety #18 family)

## Dual-mode human acceptance round (2026-09-17, user order "check every example + complete cmake run targets")

- native run steps cover every example: the INTERACTIVE keyword macro abolished, run_<name> emitted unconditionally (the DISPLAY family goes through the run-gui
  wrapper / :0 fallback; the headless family direct-run = the terminal-OK human-check face) — the E702 window example gets its first run target; the matrix =
  rs cargo-run_* ×12 + native run_* ×9 (+ the out-of-macro self-registered E703)
- E502 dual-mode-ized: the color-calibration example upgraded from "headless quick-exit" to a no-arg resident human-check window (four color boards 960×600, Esc/close to quit)
  + the --frames self-assertion path kept as-is (ctest); both paths build data from one source, config.format explicit Srgb preferred
- audit conclusion (every example run for real): window examples with no args = resident ✓ ×10; self-exiting examples = by design identity ✓ (terminal output / PNG landing is
  the human-check face; pick stand-in = E402, color = the E502 window, constant width = E503) — no hidden forks except E502
- tutorials.md "human-check forms" section: the dual-mode convention written down (IDE zero-args = human check / ctest argv = automated, C15 single source)
- baseline: 167 passed · 117↔117 · GATE-DOCS ✓ · CMAKE-SMOKE ✓ · qt-configure ✓ · ci ten segments all green

## Point-cloud data family band (2026-09-17, D16/hyperplan plan pcl-data-band.md, M0→G all collected)

- port surface: CAPI-18 add_points (23 entry points, minor=4) + CAPI-19 load_pcl (24 entry points, minor=5,
  the VePclReport four-class export / meta attr prefix query); three-surface mirror across the band (header / hpp / wasm loadPclBytes + d.ts /
  web-mirror pins ×2); the C15 value-ownership table written first (attr found=1/none=0, corrected by tests, registered)
- the io-points new crate (authorized by ruling C): PLY ascii/bin_le self-written ~420 lines, zero new dependencies; the four-class typing
  "point level triggers FastFail, column/element level counts only" landing clarified into the clause; truncation dual semantics bidirectional assertions;
  the spec-trace IO prefix + **the third-check machine lock** (the negative FOO-01→red measured, on record)
- examples/gates: E204 dual mode (the spiral self-assertion threshold scales with point count + the --file load path + the resident window);
  G1 100k batch integrity / G2 far-origin mutual-inverse point edition (byte-for-byte equal); the G3 --points branch, 1M measured, registered
- **toolchain root fix**: bench.sh dead links since the S1 package rename + the missing-item continue always-true (PIT-25 filed, MISSING→exit1);
  the rustfmt-fed-Cargo.toml accident (edit-safety #20 filed); the lesson of three test anchors reflowed by rustfmt = anchors must be read fresh
- baseline: **181 passed · spec-trace 125↔125 · GATE-ABI 24/24 · GATE-DOCS E 22 items three-way ·
  ctest 23 items · CMAKE-SMOKE ✓ · WEB ✓ · audit try1 ✓ (PIT-7 7th sighting) · 8 crates**
- remaining balance: the LAS second band (criterion = customer demo list) · the picking family trigger-gated · cap change window once · push awaiting

## B2 cross-section band baseline (2026-09-17, plan capi-cross-section-band v1.1 · Momus OKAY 0B/2A fixed in)

- port surface: CAPI-20 set_clips/get_clips (26 entry points, minor=6; n=0 is the only clear / bounds-check precedes dereference /
  the engine side reads back normalized unit shapes / cap truncation returns the true count [Momus-A2 cases into the RED catalog]); three-surface mirror across the band
  (header 32B Pod coefficient form / hpp two thin forwards / wasm setClips·getClips + d.ts + mirror abi pin 6)
- pipeline: the View block tail extension 192→256B (**zero new binding/bind-group/buffer** = the design dividend of origin+transform already being in the
  view_block signature); clipped() fs discard across all five pipelines + casters clipped alike (the fs_shadow empty-function body filled in, bgl binding0 +FRAGMENT); the expansion families per-pixel = "clip the world, not the types"; None/EMPTY
  all-zero always-bound early-exit = the R4 guardrail shape, the golden suite zero re-records
- IR: REND-32 world f64 coefficients + ClipSetup::new degenerate rejection; clip_to_local the **model-space identity**
  (n_m=Mᵀn/d_m=d+n·(o+t_M); two self-caught root fixes recorded honestly — the column-major Mᵀ and M's translation column)
- picking: the engine retry loop (hit on the negative side of keeps = excluded, keep searching; whatever is visible after cutting must be pickable); the capi pick domain
  = positions × IDENTITY, the existing fact noted as "two frames merged" into the example body and tests
- examples/gates: E813 headless three-segment living run (baseline red 10816 / half-cut red-dead-green-alive + the retry loop / corner triple-face 1764∈[800,4000]
  — the two-face shape g2=0 probe win = the front wall occludes on-screen, PIT-8 third sighting); tests/clip.rs five gates (canary byte-for-byte / None≡EMPTY / half-cut halves / full-cut zeroes / corner ¼ + far_origin 1e7 bit-equal + line/point color families + caster 6.5×)
- new pit: PIT-26 (from_raw_parts NULL+0 is UB, the debug precheck nailed — NULL counting paths must guard m>0);
  PIT-7 8th sighting (audit TLS flapped once, retry green)
- baseline: **191 passed · spec-trace 130↔130 · GATE-ABI 26/26 · GATE-DOCS ✓ E 23 items three-way
  · ctest 24 items · CMAKE-SMOKE ✓ · WEB MIRROR 3/3 + web_capi_pair_mirror ✓ · ci ten segments ✓ ·
  8 crates unchanged**
- non-goals on account: stencil caps (the BIM interior demo list) · six-face boxes · OR/union · the hardware CLIP_DISTANCES
  dual path (present in wgpu30 but DX12 not in the list) · interactive clip sliders (E901 keys pending demand) · LAS (ticket-gated, standing by)
- decisions registered: **D17 eight adjudication points** (discard single-path + re-eval trigger / ≤4 AND / no caps / Mᵀ identity / all families clipped / casters clipped alike / retry loop / E813)

## S2 text-label band baseline (2026-09-18, plan io-text-labels-band v1.1 · Momus OKAY 0B/A-1 fixed in)

- band meaning: tier ① + scheme I (engine self-managed fontdue rasterization, the user's explicit ruling) — label/caption rendering promoted from "SDK short board" to first-class
- port surface: CAPI-21 load_font (replacement style; failure keeps the old) + CAPI-22 add_label (the struct_size forward gate +
  the timing gate "no font = reject" + the full value-domain zero-commit catalog; 26→28 entry points, minor=7); three-surface mirror across the band (header POD / hpp thin
  forwards / wasm loadFont+addLabel→bigint+MISS / d.ts pins / mirror value pairs); the E701 text closed-loop section + gate-abi 28
- new crate: visiaengine-io-text, the 9th (face / GlyphCache R8-shelf-512²-1px-gutter-refreeze-on-full / layout
  LTR; the three clauses IO-07..09; fontdue locked default=false + hashbrown [the probe blind spot corrected against local sources])
- pipeline: REND-33 LabelMark 64B 4×vec4 layout lock + DrawLabels the 6th variant + create_labels default rejection;
  WGPU-24 standalone bgl(0+11+12+13) + R8 atlas + ClampToEdge + always-on-top (Always/no depth write) + SRC_ALPHA;
  WGPU-25 anchor verdict: the whole label lives or dies together; GEO-25 text-field "{prop}" unwrap / constant two forms + a missing column value skips, never prints the literal
- examples and three gates: E205 dual mode (white=667 / warm=348 probe thresholds; ortho cross-zoom assert_eq pixel-equal) /
  E814 C living run (timing gate + utf8 + white=3917) / labels.rs WGPU dual tests (atlas texel / constant size / anchor verdict bidirectional)
- lessons filed: PIT-28 (pen is px, cannot enter anchor world coordinates — caught by the hist probe, the three-port grep rule);
  #17 double-corrected (the plan's 140 = a phantom count from REND-32 double-counting, the real 139 pasted from the log; the "positions enumerable" assertion went wrong twice =
  the out-of-items domain semantics never checked against the table); the E0449 cfg(test) cross-crate invisibility lesson → the self-introspection port became a formal API
- baseline: **202 passed · spec-trace 139↔139 · GATE-ABI 28/28 · GATE-DOCS ✓ E 25 items three-way ·
  ctest 26 items · CMAKE-SMOKE ✓ · WEB MIRROR 3/3 (abi pin 0x…0007) · ci ten segments 0 red · 9 crates**
- R2 re-account: wasm gz 429401→490967 = +14.3% (inside the threshold, zero margin; the probe's +6% estimate was optimistic, recorded honestly)
- remaining balance: push awaiting · billboard/halo/declutter/wrapped-rich-text/text-update port = label follow-on bands, trigger-gated ·
  CJK = the host font-injection same port (the demo keeps a Latin fixture as an open account)

## E901 interaction close band (2026-09-18, D17-h/D18 demo close-out, single example file + zero ledger movement)

- the twin-city resident window gains keys: **C = clip knife ON/OFF (blade rises at z≤6m, shadows collapse with the blade) · [ / ] = blade height ±1m (clamp 0.5..15) · L = three tower-top labels (Tower-A/B/C, the io-text full-chain window debut) · Esc = quit**; the title echoes state live
- one construction-side lesson (post-write self-check root fix): the first version's "append then pop to restore the base" swap logic would **empty all commands** on the no-font branch — the correct form = the base never mixes labels, the redraw frame concatenates per switches (the show domain lives in rendering, not construction)
- verification: the headless path zero regression (bright=3019 dark=2426 original values) · 22s alive + real-window nodes ✓ · SMOKE ✓ · ctest #14 Passed · ci ten segments 0 red 202 passed · clauses/entry points/ctest counts all unmoved (pure consumer surface)

## ⑤a camera-flight band baseline (2026-09-18, plan camera-flyto-band v1.1 · Momus OKAY 0B/3A fixed in)

- port surface: CAPI-23 fly_to (pose f64 eight slots with the struct_size gate / teleport form / reroute continuity / near-far always current) +
  CAPI-24 fly_state (done includes idle, single source / out written only mid-flight); 28→30 entry points, minor=8; three-surface mirror across the band
  (wasm flyTo·flyState·flyProgress + d.ts + hpp); E701 no new section (E815 is a standalone example)
- IR: REND-34 Easing three families + the fly_sample pure function (endpoints return from/to originals as the fast path; the yaw shortest arc lives in the sampler's
  construction piece, mix_rig untouched = zero regression for REND-15/16 stock); the E302 counter-evidence lock (linear-midpoint two-sided comparison)
- **the wasm clock mine defused first**: std::time is unavailable at wasm32-unknown-unknown runtime → the now_ms() dual-form shim
  (native Instant process anchor / wasm js_sys::Date::now; js-sys 0.3 target dep) — compiling ≠ running
- examples: E302_fly_camera dual mode (1/2/3 presets flying between each other / drag+wheel cancel / R/Esc; headless four assertions, bright measured
  96, threshold 40) + E815_fly_camera.c headless living run (idle-is-done / teleport / progress monotonic 14 samples / cancel / domain reject ×4)
- lessons honestly: the wasm bridge pose flat 7/8 args hand-slip self-caught; the closure-borrow triple errors → expanded to direct writes; the Chinese-anchor heredoc
  stumble (#9 cross-band recurrence, sighting N); the T2 gate-docs miss (E815's tutorials line) recovered at T3 = the gates mutual-guard
- baseline: **209 passed · spec-trace 142↔142 · GATE-ABI 30/30 · GATE-DOCS ✓ E 27 items three-way ·
  ctest 28 items · CMAKE-SMOKE ✓ · WEB MIRROR 3/3 (abi pin 0x…0008) + pair_mirror ✓ ·
  ci ten segments 0 red · 9 crates unchanged**
- non-goals on account: multi-viewport split screen (the four-pit list D19 / trigger = split-screen demo list) · altitude arcs · multi-keyframe flythroughs ·
  inertial damping · fly event pushes · CJK embedding (the S2 remaining balance)

## ⑤b multi-viewport split-screen band baseline (2026-09-18, plan multiview-split-screen-band v1.1 · Momus OKAY + user "full band")

- pipeline: render_view_rects (N Frames × ViewportRects, single surface / single full-extent depth, multiple passes; first pass
  Clear paints the whole-area base color + later passes Load color + **depth Clear+Discard every pass**); casters do one shared-map trip;
  full-rect emits zero set_* = the old path's bit-equality canary; REND-35 ViewportRect standalone type, **zero Frame touch** (the 44-fallback wave exempt)
- the adjudication gate's conclusion: wgpu30 LoadOp::Clear = the whole attachment (AllClear redA=0, the live lock, consistent with the Vulkan conclusion) —
  the two probes' disagreement was decided by this machine's pixel gate (E303's first run again caught the depth-Load-swallowing mine → the every-pass-clear shape, overwriting the probes' expectation)
- legacy mine eradicated: the fov radian/degree unit (E501/E302 window projection distortion nailed, E302 bright 96→1987);
  camera.rs orbit doc nails the unit; the four multiview gates (adjudication / mutual non-aggression + gutter color / depth fence / shared shadow 328×328 threshold 150)
- examples: E303_split_screen dual mode (96×64 four assertions + the top-right square small-window follower; resident window ✓); four registration faces + the full gate suite
- lessons registered: "a window surface the pixel gate never looked at is an unverified surface" (D20's last item); the isolation-probe method (single projection full-screen ISO → multi-viewport
  differencing) located the inter-pass difference in two minutes; the U1 REND-35 body omission surfaced through spec-trace 143 (the target/body pairing discipline)
- baseline: **214 passed · spec-trace 144↔144 · GATE-ABI 30 (untouched) · GATE-DOCS ✓ E 28 items three-way ·
  ctest 29 items · CMAKE-SMOKE ✓ · ci ten segments 0 red · R2 untouched (pure rs band) · 9 crates**
- remaining balance: push awaiting (3 commits this band) · the capi multi-viewport wave two (CAPI-25/click-to-fly) · the shared viewer library entry awaiting ·
  the translucency ④ band / packaging round / LAS standing by

## ④ translucency band baseline (2026-09-18, plan translucent-water-band v1.1 · Momus OKAY 0B/3A fixed in)

- the three-zero discipline honored: **zero new ports / zero dependencies / zero IR fields** (entry count 30 · abi · R2 all silent; transparency decision = material alpha<1
  sorted by a backend side table, WGPU-28); material a==1 always takes the old road = the **golden family full-suite zero-re-record construction guarantee** (the labels family runs the two passes live)
- V0 blend domain settled: this machine's wgpu30 = linear domain (151 vs 83 checkerboard gate, decided on a single value, consistent with the Vulkan spec);
  the prediction arithmetic's first version missed the Lambert shade — the PIT-8 extended note "predictions carry all chain factors"
- pipeline: Variant extended FlatT/TexturedT/InstancedT (bgl/layout fully identical, only pipeline state: the SRC_ALPHA standard form +
  no depth write + Less preserved for occlusion); two passes = opaque vec order + trans view-depth descending (the f64 D7 key origin + M translation column,
  same-key submission order stable) + Labels always-on-top; **pass2 depth Load inherits occlusion** (domain-split from ⑤b's inter-viewport depth clearing)
- E504 bidirectional semantic lock: the water film = attenuating contrast (52→15 control band (10,30)), not destroying contrast; keys 1/2 = a living demo of
  the host path re-uploading materials to change alpha; the GEO-26 target + clause (park α=0.8, the WGPU-11 gate re-measured within zero-re-record margin)
- lesson: the transparency-order test's "near/far" naming was slapped by the camera pose once (z50 top-down → z7 = near; expectations re-labeled, the engine untouched);
  the amend triple-offense (fmt/clippy/dead Mats code — run the full local chain before the final verification, don't use amend as a lint loop)
- baseline: **220 passed · 148↔148 · GATE-ABI 30/30 · GATE-DOCS ✓ E 29 items three-way · ctest 30 items ·
  CMAKE-SMOKE ✓ · WEB MIRROR 3/3 · ci ten segments 0 red · D21 registered**
- remaining balance: push awaiting (the ④ band's four commits + prior accumulated) · set_material_alpha / soft shadows / OIT / per-instance alpha / per-vertex alpha
  all trigger-gated · ⑤b capi wave two unchanged

## ⑤b wave-two minimap navigation band baseline (2026-09-18, plan capi-minimap-nav-band v1.1 · Momus OKAY 0B/2A fixed in)

- port surface: CAPI-25 set_map (the VeMapView proportional rect; NULL = off = the old path; domain rejection zero side effects) + CAPI-26
  navigate_click (outside the region / no map = reject; entity→ground two-stage; keeps angles and distance [ruling e]) + CAPI-27
  get_camera (the VeCameraPose dual-use = input for fly / output for readback; hosts pre-set struct_size, check before writing);
  30→33 entry points, minor=9; three-surface mirror across the band (header / hpp three thin forwards / wasm 4 bridges setMap·clearMap·
  navigateClick·getCameraPose + d.ts roster + mirror pin 0x…0009)
- engine state: MapView + set_map/map_rect/map_rig (the top view derived from the main target) / navigate_click /
  camera_pose; the render dual-projection branch (the first consumption proof of ⑤b render_view_rects; None = the old-path canary);
  the pick minimap-region-priority routing; apply_input minimap region kind1..4 = consumed no-ops
- examples: E816_minimap_nav.c five-segment living gate (canary byte-for-byte / corner dual families 1596g+756r / minimap pick /
  600ms flight 257 ticks to (3.579,3.714,0) / value-domain reject + no-map reject); ctest #31
- lessons honestly: E816's first version treated two horizontal planes as "walls" — the top view = red covering green, g=42 exposed it via probe (PIT-8 fourth sighting;
  probe scene geometry before asserting); segment 5's end-state canary compared against the old ref = the flight moved the camera and must break (post-write self-check root fix,
  the baseline-refetch shape); the ffi_spec use block missed three function names (the regex newline anchor didn't hit = the edit-safety #9 family)
- baseline: **226 passed · spec-trace 152↔152 · GATE-ABI 33/33 · GATE-DOCS ✓ E 30 items
  three-way · ctest 31 items · CMAKE-SMOKE ✓ · WEB MIRROR 3/3 (abi pin 0x…0009) · ci ten segments
  0 red · 9 crates · R2 untouched (the web gz increment is accounted by what web-check prints)**

## Super-band five-in-a-row baseline (2026-09-24, plan super-band-2026-09-24 v1.1 · Momus OKAY + three red-team reviews + user nine-card item-by-item rulings all A)

- origin: three-team gap audit (`docs/reference/team-gap-analysis-2026-09-24.md` supersedes the 09-17 edition) → user "adopt everything", action order 1-5 → four-lane review (Momus OKAY 0 blocking + feasibility/consistency/risk three volumes, /tmp index) → two plan-killers (B2 VePickHit premise collapse, B1 thumbnail premise collapse) + one CRITICAL (B2 API shape), all fixed into v1.1
- **B4 documentation reconciliation** (`51f94bb`): the three drifts WMTS "built-in" / scene tree ×3 / capability_query reworded (EN, C17) + **the 10MB first measurement closed** (cdylib strip 7.25MB ≤10MB, a 27% margin, evidence/2026-09-24-native-size.md) + the gap-analysis canon landed
- **B2 point picking** (`03a864a`): ruling ④A = zero ABI change (the engine keeps positions + the CAPI-04 revision sentence + the kind account trigger-gated); REND-38 screen-space nearest-point predicate (the **local-point direct projection compose_mvp** = origin already baked into the matrix, don't double-count + the RH [0,1] depth-domain guard); REND-39 mesh-priority canary; CAPI-13 corrected to the CAPI-15 precedent; E204 `--pick-check` argv single master (C15②)
- **B1 Web gallery** (`2254bbc`): ruling ⑤ reversed during execution = the tool bin rebuilding scenes (drift tax) discarded → **example-side self-emit** (the gallery.rs helper ×10 dual-mode examples save_frame; examples = the card-image true source, zero drift); gen_gallery.py reads the single registry (the three shapes: cmake + per-directory CMakeLists + qt bare add_executable) + fail-loud, dual negative paths measured; 32 cards / 11 real images / build gitignored
- **B3 MVT tile streaming Phase 0** (`080e07e`): the 10th crate io-tiles — the 353-line handwritten protobuf wire decoder (ruling ①A, zero dependencies; the fixture generator's two wire errors surfaced via Rust/python cross comparison = the decoder error-free) + slippy tile math (neighbor-edge seamlessness law) + FileSource / typed HttpSource stub / stdlib LRU + the GEO-27 GeoTile mapping (y flip + D7 origin anchor); E206 walks the whole chain; **PIT-8 again justified**: the camera look_at sits at the world origin while tiles live at x=−2e7 m = everything off-screen + far < eye distance clips all
- **B5 one-example-four-meals policy** (`a58e689`): gate-docs six checks + ⑥ the gallery manifest↔on-disk registry set machine lock (negative path measured red); the policy text in three places (tutorials header / dev-workflow / gate comments, EN)
- **E206 user triple-measurement fixes** (`3f3fccd`/`3690a34`/`9eceb18`): ① no-args immediate exit = the C15 dual-mode breach (the window stub) → the E204 skeleton real window + Xvfb liveness probe; ② drag/wheel unresponsive = Redraw rebuilt tile_rig every frame hardcoded + rig not a field → the persisted rig + orbit_delta/zoom; ③ single tile too plain → the **3×3 neighborhood** (per-tile changing water patches / staggered roads / border linework = seams visible / center POI; vertices baked into small values around the scene center, f32 single world frame); the wheel root cause = **orthographic zoom's only definition is half-width while rig.zoom was never consumed** (look_at default zoom=1.0 + the render hardcoded tile_w, double-stacked)
- baseline: **259 passed · spec-trace 158↔158 · GATE-ABI 33/33 (the zero-ABI-change line held) · GATE-DOCS six checks ✓ 32 cards · ctest 24 (machine-reported) · 10 crates · zero new dependencies · ci ten segments rc=0 · worktree clean**
- to-do: push (11 ahead) · the Phase 1 tile band (C API + HTTP, the whitepaper date promise 2026-Q4) · the gallery T3 human check

## ⑤ Phase-1 tile streaming band baseline (2026-09-24, plan tile-streaming-phase1.md P1-S1–S4 all landed)

- port surface: CAPI-28 load_mvt_dir (the FileSource z-level scan; domain z≤30; return = mounted tile count) + CAPI-29 set_tile_view (feed the scheduler a 3857 bbox; return = visible count; not mounted = reject); 30→**35 entry points minor=10 (0x0001000A)**; three-surface mirror across the band (header 2 prototypes / hpp thin forwards / wasm loadMvtDir+setTileView + d.ts roster / mirror abi pin 0x…000A)
- engine state: the TileLayer structure (set:TileSet + ids + center/wpos/widx/strokes/points + uploaded:Option; render-loop integration = a later band, annotated `#[expect(dead_code)]`); load_mvt_dir scan+ensure+decoded→GeoTile batch build; set_tile_view only ensures, never rebuilds (batches = one-time at mount)
- io-tiles new pieces: `scheduler.rs` TileSet{visible half-open enumeration + boundary snap / ensure idempotent loading + accounting / decoded through-cache second tier} (IO-13/14); HttpSource::new(root) ureq implemented (IO-15, target-gated: wasm = typed stub, the getrandom/js conflict root-cause fixed); `tests/tile_set.rs` 6 gates; `tests/http_source.rs` 3+1 ignored; the stub tests in source.rs upgraded to new()+ domain assertions
- dependencies: ureq 3.4.2 (rustls, default-features=false) → webpki-roots (CDLA-Permissive-2.0) → **deny.toml += CDLA-Permissive-2.0**
- E206: load_tile retired (dead code) + the FileSource/decode_tile imports cleaned; load_viewport driven through the scheduler (the IO-13/14 consumption surface)
- gate-abi.sh: the expected count updated 33→35; web-mirror.mjs: abiVersion 0x0001000A + roster +2 bridges
- whitepaper: MVT Phase 1 honored (moved from "delivering" to "delivered" + the WMS/WMTS separate note); README synchronized 163 clauses / 35 entry points / 32 cards
- lessons filed: PIT-28 (pen px vs anchor world coordinates, the three-port grep rule) · **PIT-31** (ffi_spec domain tests cwd=crate dir, the relative-path trap) · **PIT-32** (the full ureq dependency dragged getrandom into the wasm target — fixed with a target-gate, wasm HttpSource = typed stub)
- baseline: **270 passed · spec-trace 163↔163 · GATE-ABI 35/35 · GATE-DOCS ✓ E 31 items · ci ten segments rc=0 · web MIRROR 3/3 · 9 crates · ureq 1 new-dep**
- remaining balance: push (all P1-S1..4 commits of this band) · the render-loop TileLayer integration (E206's prove shape into the engine main loop) · the HTTP source C API (**CAPI-35+** set_tile_source_http; CAPI-30..34 already consumed by the scene-tree band, re-accounted 2026-09-28) · the CJK font S2 balance · LAS ticket-gated, standing by

## ⑥ Scene-tree band baseline (2026-09-24, plan scene-tree-band.md S1-S4)

- port surface: CAPI-30 create_group (the out_entity catalog) + CAPI-31 set_parent (0=root, cycle / depth≤16 detection) + CAPI-32 get_parent + CAPI-33 set_group_offset (the finite gate) + CAPI-34 get_group_offset; 35→**40 entry points minor=11 (0x0001000B)**; three-surface mirror across the band (header 5 prototypes / hpp 5 thin forwards / wasm 5 bridges + d.ts roster / mirror abi pin 0x…000B)
- engine state: Slot gains parent:Option<EntityId> + offset:[f64;3] (core scene.rs, the Component::Group pure-marker variant, slot_of zero cost); Engine::new_scene (pre-occupies slot0gen0 → the handle is never zero); is_hidden (walks the ancestor chain, tree-aware visibility); the render loop origin = it.origin + scene.effective_offset(entity) (CORE-21 accumulation, O≤16)
- **design decision**: rotation/scale inheritance = trigger-gated (YAGNI: mainstream GIS engines = visibility + translation only); despawning a group auto-detaches children back to root (no cascade delete)
- lesson: PIT-31 (ffi_spec tests cwd=crate dir, the relative-path trap) **second sighting** (both the load_mvt_dir and set_group_offset domain tests tripped it) · **PIT-33** (the scene-tree handle=0 semantic collision: slot0gen0 legal vs the parent=0=root sentinel; fix = new_scene pre-occupies slot 0 + the set_parent parent==0 special case)
- baseline: **287 passed · spec-trace 173↔173 · GATE-ABI 40/40 · GATE-DOCS ✓ E 31 · ctest 33 items · ci 9/10 (audit = PIT-7 11th sighting, a persistent TLS outage, not momentary flake) · web MIRROR ✓ · 9 crates**
- remaining balance: push (ahead includes this band's S1/S2/S3 three commits + ⑤ P1 five commits ≈ 11) · rotation inheritance = open it when demanded · scene-tree name lookup (node_names stored in engine.rs, no C query port) · the CJK font S2 balance · LAS ticket-gated

## Gap research → hyperplan → Session N+1 baseline (2026-09-28, eight commits closed)

- team mode: four-lane research (.refinfo six families + self-audit) → adversarial planning 4 batches ×3 rounds (C1-C10 all signed + the concession ledger) → Momus OKAY → four-card adjudication (1a tile pump / 2 golden+EDL / 3 morph do / 4 no-deploy + three.js benchmark) → execution
- N1.1 `a5d3180`: the hpp fly_state forward + **the hpp⊇header mirror gate** (alias-table form: destroy→destroy_now, entity_set_visible→set_visible; breakage-probe self-proof) + the CAPI-30→35+ re-accounting + the legacy clippy debt cleared
- N1.2 `59c772a`: whitepaper L27/L103 + README L11/L38 precise rewording ("smooth/seamless/stepless" → delivered semantics; the L22 competitor sentence kept)
- N1.4 `47bbe54`: **CAPI-35 set_tile_source_http** (discovery-style loading) + the io-tiles 4-state pump (begin/pump(budget)/state) + HttpSource 5s timeout + loopback proxy-exempt (PIT-35) + the render loop 4 tiles/frame budget + three-surface mirror abi 0x0001000C + E206 --source real run 9/9 (4+4+1 across frames) + the rebuild_tile_batch extraction
- N1.3 `354a8e7`: **goldens-10 per-attribute goldens** (good/bad pair self-proof form; all predicates probe-pinned values: opacity 145/207, point 4672/232, clip upper/lower halves, shadow mid-band split)
- N1.5 `e55dd2f`: **EDL** (Frame.edl None = bit-for-bit zero-regression old path; intermediate texture + the PIT-33 use-domain law; ON 2790px + top row 331.0→322.9 measured; the WGPU-33+REND-40 clauses; E204 --edl)
- N1.6 `ca6dbee`: the gallery three.js-style detail pages ×32 (CARD 4 = no deploy, purely local)
- review fixes `60a24a3`: #1 detail-page full-name key (E811_c/_cpp split pages + collision fail-loud) + #2 C17 21 lines zeroed
- N2.0 `cd55fc1`: #3 the evict_done+decoded_cached reconciliation (PIT-34) + #4 center shifts clear uploaded/batch (misalignment eradicated)
- lessons: PIT-33/34/35/36 + edit-safety #21/22/23 + the testing.md HashMap-order discipline (registered this round)
- baseline: **297 passed · spec-trace 177↔177 · ABI 41/41 (0x0001000C) · GATE-DOCS ✓ · ci rc=0 · 8 commits ahead**
- remaining balance: push awaiting · N+2 = the morph band + entity animation 6a + gallery category expansion · deferrable = property_golden anchor fill / mirror-gate declaration form / multi-viewport EDL assert / point-radius floor

## Session N+2/N+3 baseline (2026-09-28 continued, five commits: the four N2 bands + sweep)

- N2.1 `6d65346`: **projection morph** (M, Momus OKAY + card A pure rs) — the render::morph pure function (endpoint byte-for-byte law) + Engine::set_morph_time (slider clamp semantics, an intentional fork from EdlSetup's rejection form) + the morph_spec 3 gates + E304 dual-mode example (window [/] slides t; the Xvfb liveness probe). Whitepaper §3.1 honored; README "switch" → "continuous projection morph" (the N1.2 honesty arc closed)
- N2.2 `2ca8551`: **entity animation 6a** (S, split by C5) — render::anim::anim_origin (endpoint identity + f64 far-origin precision) + the anim_spec 4 gates + E305 trajectory replay (24 markers, pixel gate). The example's first run caught the normalized-t misuse = the sampler time-domain contract surfaced
- N2.3 `f10baff`: the gallery sticky category nav (three.js browsing shape, the 9-domain anchor bar; the lesson of the cards key = restoring title numbers)
- N2.4 `8b18937`: **attribute hot-update CAPI-36..38** (M, 44 entry points minor=0x0001000D) — the override layer + override-first read + typed-reject (CORE-11/12) + three-surface mirror across the band + the attr_diff_spec 2 gates (fixtures copied the attr_ffi_spec precedent). The rs-only plan upgraded to three C ports = a test-reachability ruling (Engine ownership lives in the handle table)
- sweep `0d430c4`: review items #5-#8 deferred list zeroed (multi-viewport EDL debug_assert / point-radius dual bands / anchorless golden declaration / mirror-gate declaration-form predicate triple self-proofs; the column-zero [[nodiscard]] lesson = self-proof samples must take real lines)
- baseline: **306 passed · spec-trace 182↔182 · ABI 44/44 (0x0001000D) · gallery 34 cards + 34 detail pages + nav bar · ci rc=0 · 15 commits ahead**
- the digital-twin three bones: attribute hot-update ✓ / animation sampling 6a ✓ / serialization S3' on account (triggered by Studio's dependency)
- to-do: the trigger-gated queue fully empty (geometry diff / box-select / ODR / LAS / 7-state machine tickets standing by)
- **T3 human checks (2026-09-28 user measurement)**: E304 `[/]` slides the projection morph ✓ · E305 resident window drone loop replay ✓ (the window-surface gap found by the user, filled at eab5530) — two closed, the digital-twin demo surface double-verified

## N4 post-processing framework band baseline (2026-09-28, plan n4-post-framework-band.md v1.0 · Momus OKAY)

- IR: REND-43 PostEffect{Bloom{strength},Outline{width}} + Frame.post Vec (empty = the bit-for-bit zero-regression old path) + the bloom/outline constructors' domain-reject Option (same catalog as EdlSetup::new); **a 46-site Frame construction sweep** (python indent-aware insertion + manual contract.rs edits); the contract_spec + domain-reject gate
- pipeline: WGPU-34 stage-chain orchestration — ensure_stage_color fixed 2 slots (size+format cache) + per-(effect,format) lazy pipeline creation + bgl shares EDL's 4-binding shape (fullscreen_entries extracted as shared descriptors); the depth Store single flag `edl_on||post_on`; **effect i writes stage[i+1], the last effect writes the final view, stage[0] = chain input never a write target** (the first version writing stage[i] collided with wgpu's usage-scope RESOURCE+COLOR_TARGET in one pass = caught as a validation error)
- shaders: bloom.wgsl 9-tap luma-threshold additive (THRESHOLD=0.25 tiered by a probe in the linear-decode domain — sRGB encoded 138/px ≈ linear 0.25); outline.wgsl depth-gradient ratio (the WGPU-33 rel same-family threshold 0.08) + selection orange; vs_main same catalog across three shapes (fullscreen triangle strip from vertex_index)
- examples/gates: E506_postprocessing dual mode (keys 1/2/3 + title echo; --frames off determinism + bloom/outline pixel assertions probe=1289/1509px thresholds 600/700) + four-way consumption across the band (cmake registry disp default headless / tutorials line / gallery 37 cards / smoke-post); the post_chain.rs four gates (off bit-for-byte / bloom 432 / outline 678 / chain order 1038·1640, all probe values in the notes)
- **web-check caught a real mine**: the engine.rs wasm-only new_canvas constructor was mis-injected with post fields by the sweep script (unreachable by native check = behind the cfg gate) — the living evidence for always running web-check after CI (R2 gz=539641B accounted)
- baseline: **311 passed (+5) · spec-trace 184↔184 · GATE-ABI 44/44 · GATE-DOCS ✓ E 36 items three-way + 37 cards · ctest 37 items · CMAKE-SMOKE ✓ (Xvfb display ✓) · ci ten segments rc=0 · 9 crates**
- remaining balance: push awaiting · SSAO/DOF/SSR = framework-following, trigger-gated · multi-viewport post (on account like EDL) · >2-effect chains = non-goal

## T3 human-check addendum (2026-09-29, the N3-N6 full-band interaction surface)

- E508 drag-orbit / wheel-zoom ✓ (the rig hard-coding root-fixed = 733de61) · keys 1/2/3 effects ✓
- E507 tonemap/bloom keys ✓ · E403 box-select ✓ · E306 map controls ✓ · E304 morph (verified earlier) ✓
- conclusion: all five N3-N6 bands' interaction surfaces double-verified, no residual T3 items

## V-series gap-closure baseline (2026-09-30, hyperplan → full-closure adjudication → Block A/B execution)

- origin: hyperplan adversarial 5 members ×3 rounds (research base = the three.js 608-example census + refinfo 13 projects + self-audit) → the full-closure adjudication → plan v1.1 (`.omo/plans/gap-closure-full-2026-09-30.md`, 4 reviewers, 10 fixes, 4 risks) → execution
- audit canon (**2026-10-08 corrected**): this line originally cited the "09-30 audit" and `docs/gap-closure-roadmap.md` as "dual canons", but the former (team-gap-analysis-2026-**09-30**.md) **was never committed** (no such file in the full git history) and the latter did not exist at the time. The live on-disk canons = `docs/reference/team-gap-analysis-2026-09-24.md` (the S-grade gap table + example census) + `docs/reference/team-gap-analysis-2026-09-28b.md` (the three.js 608-example benchmark, superseding 09-24's example-count section) + `docs/gap-closure-roadmap.md` (actually written 2026-10-08: the clause-by-clause re-measured coverage map / not-done list / triggers / reverse advantages; the 09-30 synthesis judgment stays in the gitignored plan `.omo/plans/gap-closure-full-2026-09-30.md`, not citable from the repo)
- **W1** (`9fd64a2`): REND-46 stroke screen-space picking predicate (+5 pure-math tests; the engine wiring honestly on account — the C pick surface's EntityId vs segs having no entity table collides with CAPI-01; disguising it would be a breach)
- **W2** (`e24b308`): WGPU-38 depth haze — **the raw-depth perspective-compression mine surfaced by probe** (16m ≈ 0.994), SSAO view_z metric-domain linearization; the three-band geometry-aware dual lock (bg 12114 fully fogged / near 0 touched); E506 key 4
- **D** (`13035eb`): four drift reconciliations (MVT/morph/scene-tree/Flutter-C#) + **check-promises lint** (the proximity matcher; the self-test breakage probe caught the first version's line-level allowlist as too coarse) + llms.txt/docs-gen; **ci ten segments → eleven**
- **R** (`54f4e26`): **whitepaper L53's WMS/WMTS promise honored** (the raster surface) — IO-18 the `TilePayload{Mvt,Raster}` enum (anti-trap ruling = single-lane pump / LRU zero fork) + CAPI-39/40 the flat basemap (the N1.2 discipline: NOT terrain); five-surface mirror abi 0x0001000E
- **I** (`49c2a95`): **WGPU-39 SH-9 environment lighting** (demo-grade claim narrowed; the View block 256→416B tail scheme, zero new bindings; PIT-41/42/43 all three mines tripped and defused); CAPI-41; the E507 env probe
- **V2.1** (`88e2e90`): REND-47 Bezier/CatmullRom + tube/ribbon (+0-ABI consumption band) — **E510's endpoint-law assertion caught a real sampler bug on first run** (the last knot never landed); E510 four consumers + the resident-window shell added (`5e34bc0`, a new variant of the C15 fourth case = no window); the destroy_points engine port landed as a precondition
- **V2.2** (`68cc205`): **CAPI-42/43 node local transforms + a legacy root fix** — the glTF entity.world hierarchy matrices were discarded at mount time (multi-node hierarchies had never rendered correctly, masked by E201's single layer); DrawItem.transform wired through the full chain + rigid parent chains; five-surface mirror abi 0x00010010 (49 entry points); E511 rotation pixel proof
- **V2.3 LAS = conditional, not opened** (no real .las fixture in the repo); **V2.4 skinning+morph = L-grade, awaiting entry** (the F1 promotion on record; it requires its own plan → Momus → approval)
- baseline: **343 passed · spec-trace 199↔199 · ABI 49/49 (0x00010010) · ci eleven segments · gallery 43 cards · 3 commits ahead of origin (gitee auth failed, user side to refresh)**
- lessons registered: PIT-41 (multi-block patch truncation mine, the io-gltf wipeout incident) / PIT-42 (the WGSL uniform trio: closures / let-if / array stride) / PIT-43 (flag-offset drift = a probe's job); the background-agent five-losses review (rpm rate limits ×3 + stale-timeout ×2; the main session did W1 directly in 40 minutes vs agents wiped out in 3 rounds — small bands are better done directly than dispatched)

## Install-tree band baseline (2026-10-08, packaging-round v1.3 P1-P4)

- Scope as user cut it: install tree only (npm/pip K0/K3/K4 out). Nine adjudication
  cards walked one at a time with the new adjudication-walkthrough skill; Momus
  re-reviewed v1.3 -> OKAY (0 blocking, 1 non-blocking dangling probe-B citation,
  fixed). Decisions in **D23**.
- P1 `c2fab6d`: SONAME stamped by the repo's first `build.rs`; gate-abi asserts it
  (expected name derived live from `[lib] name`). Measured payoff: absolute-path
  linking now yields `NEEDED=libvisiaengine.so` (was the full build path).
- P2a `af2608f`: `cmake/VisiaEngineInstall.cmake` + `visiaengineConfig.cmake.in`;
  S-c beta fail-loud guard retired in the facade (43 lines, gate-docs 5 budget OK);
  `cmake-smoke` negative state inverted into a positive install state.
- P2b `2d5fb2d`: pkg-config face; `.pc` URL read from Cargo.toml repository
  (missing line = configure FATAL, probed); prefix uses the `pcfiledir` form after
  a real bug (configure-time bake reported /usr/local) was caught and turned into
  three text assertions.
- P3 `8da46b3`: `scripts/gate-pack.sh` + `pixi run pack-check` (out of ci). All
  three assertions seen red by break probes; RUNPATH reported not judged because
  the leak was traced to this env's relocated conda `.pc` files (tinfo.pc naming a
  sibling checkout prefix) feeding a `-sys` link line -- a publish-band assembly
  concern, not a repo defect (user adjudicated P3=A against my written rec).
- P4 (this commit): README install wording + numbers 40->43 with **two new locks**
  (gallery cards in gate-docs 7, ctest count in cmake-smoke; both probe-red);
  `docs/tutorials.md` installed-consumption section; D23; AGENTS COMMANDS line.
- Incidental root-fix caught by the band's own web channel (`PIT-44` + the
  `web-mirror.mjs` staleness guard): the V2.2 wasm bridge `get_node_transform(&mut
  Vec<f64>)` never compiled for wasm32 -- host tests and the artifact-reading
  mirror script both missed it for two bands.
- Baseline: **ci rc=0, 343 passed, 11 segments; spec-trace 199<->199; GATE-ABI
  (49 entries + SONAME) OK; GATE-DOCS 7 checks OK; CMAKE-SMOKE four states OK
  (install tree 9 files, .pc text checks, ctest count lock); pack-check OK;
  pixi run web-check rc=0 + MIRROR 3/3 same-terminal; 9 crates unchanged;
  zero new dependencies (patchelf deliberately NOT added).**
- Open (declared, not swallowed): npm/pip packaging, staticlib consumer day,
  win/mac install forms, publish-band RUNPATH cleaning + CPack tarball,
  T3 human verification for E510/E511/E506 key4/E507.

## Band L baseline (ledger reconciliation, 2026-10-08, plan ledger-tiles-keys v1.1)

- Scope as adjudicated one card at a time (7/7, all to the lazy option; protocol
  `adjudication-walkthrough`). Plan `.omo/plans/ledger-tiles-keys-2026-10-08.md`; Momus OKAY
  (0 blocking, 1 non-blocking path-name nit, fixed in-plan).
- `e8ddfb5`: `docs/gap-closure-roadmap.md` written for real (census + refreshed coverage map +
  open gaps with triggers + reverse advantages) and the two dangling audit citations repaired.
- `0ef26fc`: haze gates' background reference self-calibrated from the frame's own corner pixel,
  with a >2% frequency guard; corner read back [13,18,25] = bit-identical behaviour, now proven.
- `4618896`: three existence locks — cited `docs/**` paths must exist (check-promises), README
  package count must equal `cargo metadata` (gate-docs 8, README fixed 10→11 crates/12 packages),
  no `let clear: [u8;3] = [` literals (gate-docs 9). Each lock shown red by a planted probe.
- Ledger truth today: **199 clauses / 49 C entries / abi (1<<16)|16 / 12 packages / 43 examples
  = ctest 43 = gallery 43 cards / ci 11 segments**, all four recomputed by the gates.
- Found while planning band T: the tile gap is the **single layer slot** in the capi engine
  (basemap XOR vector), not scheduling — see PIT-46. Band T proceeds with zero ABI change
  (Adjudication 2 = A: per-kind addressing).

## Band T + Band K baseline (2026-10-08, plan ledger-tiles-keys v1.1, all 7 cards = lazy option)

- **Band T (two-layer map)**: capi Engine went from one tile slot to `layers: Vec<TileLayer>`
  with an explicit `LayerKind`, per-kind view addressing, first-mount-only camera fit and
  `RASTER_UNDERLAY_Z` applied only when both kinds are mounted. **Zero ABI change held**
  (49 entries, abi `(1<<16)|16` untouched, no wasm/hpp work). Clauses CAPI-28/29/39/40 each
  carry a Band T revision sentence; spec-trace stayed 199<->199 (no new clause ids).
  - `d79bf78` engine + `tests/tile_layers.rs` (2 tests). Discrimination: single-slot revert
    -> `COEXIST both=0`, `set_raster_view=-1`, red at `tile_layers.rs:90`.
  - `23abf75` **E817_two_layer_map.c** — the visual witness: 5-segment live gate, families
    measured `raster_only basemap=69120 road=0 poi=0` / `vector_only road=10739 poi=1314
    basemap=0` / stacked v-then-r `9782/1314/26030` / stacked r-then-v `5343/684/60322` /
    re-mount memcmp == 0. Break probe: single-slot semantics -> `road=0 poi=0` FAIL rc=1.
  - Two plan premises died on contact: the proof cannot live in examples/rs (Engine is a
    private `mod`), and the two fixtures sit on opposite sides of the planet, so stacking
    them never shared a frame -> added `resources/data/raster_over` (the same four 79-byte
    PNGs re-keyed into the vector region, 316 bytes) instead of claiming a gate that cannot
    pass. Basemap witness = the EXCLUSION bucket, because the basemap's own colours satisfy
    E206's water/seam predicates (PIT-8 fifth sighting).
  - README: stacking claim added + its two numbers 43->44 measured from the same terminal
    (gallery "44 cards (17 thumbnails, 44 detail pages)", cmake-smoke count lock 44==44).
- **Band K (input probe becomes a repo tool)**: `scripts/xinject.c` + `scripts/keys-probe.py`
  + `pixi run keys-probe` (out of the ci chain, pack-check policy) and the previously
  caller-less `scripts/window-probe-all.sh` wired into `.github/workflows/ci.yml` L2.
  Measured: E506 key4 23.90%, E507 keys 2/3/1 = 5.87/29.85/29.85%, E510 drag 8.95% +
  wheel 9.18% (title `phase=43.0`->`56.9`), E511 clock 1.79% at a 4s gap, Escape exits all
  four. Break probe: stubbed `self.haze = !self.haze` -> `changed=0.00%` + rc=1.
  Two port bugs found by running it for real: the X needle is the E number (titles use a
  space, not the file stem), and XWD `header_size` includes the variable-length name so no
  constant guards it -- the length identity `off + h*bpl == len` does.
- Close-out `7e139f0` (PIT-47): the two new ci.yml L2 lines could only ever SKIP -- `xwd` ships in the system package `x11-apps` (not pixi) and the runner installed only `xvfb`, so the probes' own skip-on-missing-toolchain convention = a green not earned (PIT-25's CI-wiring variant). Fixed: apt installs x11-apps with a do-not-remove comment, keys-probe gained `--strict` (skip -> exit 1 naming the missing tool; ci.yml uses it, local default still skips), xwd found via PATH instead of a hardcoded /usr/bin. Branch tests: stub-only-xwd-missing + --strict -> rc=1 "missing toolchain: xwd"; default -> SKIP rc=0; real run --strict -> 4/4 rc=0. And window-probe-all.sh, wired a session ago and never run here, now has: rc=0, 20 resident windows, E509 lit=100.0% vs a 5.0% floor.
- Ledger now: **199 clauses / 49 C entries / abi (1<<16)|16 / 12 packages / 44 examples =
  ctest 44 = gallery 44 cards / ci 11 segments / 9 crates**.

## Band V baseline (shared viewer bootstrap, 2026-10-08, plan viewer-shared-band)

- Adjudicated 5/5 = A/A/B/C/A (one card at a time, protocol adjudication-walkthrough).
  Momus REJECTed v1.0 on one blocking item (canary E101 collided with C3b's protected
  specimen) and 6 non-blocking nits; all fixed in v1.1, then executed.
- New shared piece `examples/rs/src/viewer.rs` (206 non-blank lines): `argv_frames` /
  `argv_frames_and_rest`, `Ctx::new(title,w,h,FormatPolicy)` (window+adapter+device+
  surface+config+MeshCore), `on_resize`, `present`. Scope by ruling: **G1+G2 only**
  (C2=A). FormatPolicy is a parameter because 4 examples (E204/E206/E502/E503) prefer an
  Srgb format while 22 take the surface default -- collapsing that is how PIT-9 was born.
- Migrated **24/29** rs examples: E502(canary) E205 E501 E504 E901 E202 E402 E403 E201
  E301 E204 E206 E302 E304 E305 E510 E511 E303 E306 E505 (the last three of those
  five in `--bootstrap-only` mode: their `render_view_rects` present is the ⑤b lesson).
  Example bodies: 11,480 -> 10,104 non-blank lines = **-1,376** (883 of it moved into
  the 206-line shared piece, the rest is duplicated blocks collapsed to one call).
- Deliberately NOT shared: interaction semantics (drag means pick in E402/E403, phase in
  E510, per-viewport follow in E303) and the `render_view_rects` present of the ⑤b family
  (that call IS those examples' lesson). Protected manual specimens: **E101 + E503**
  (measured: only 9 of 26 window examples were fully unshared today; E201 could not be a
  specimen because it already consumed `examples::BBox`).
- V3 queue is closed: the five non-migrated files are excluded by evidence, not by
  preference -- E101/E503 (specimens, C3=B), E203 (positional CLI, never in scope),
  E401/E601 (no G1 bootstrap to share: zero Instance/adapter/Surface/winit; they already
  use the production `HeadlessBackend`, shared one level above the viewer shell; E601's
  lone `Instance::new` is the IR struct, not a wgpu Instance). The codemod refuses them
  with `!! no ApplicationHandler impl` **before writing** -- both files byte-identical
  to backups afterwards, which is edit-safety #26 item 3 executing, not being described.
  199 clauses, 49 C entries, 12 packages, 44 examples all unchanged.
- Baseline at close: `pixi run ci` rc=0 (11 segments) 345 passed · `keys-probe` 4/4 ·
  GATE-DOCS ok (44 cards) · CMAKE-SMOKE ok · spec-trace 199<->199 · worktree clean.
- Method worth reusing: per batch = determinism pre-check (two back-to-back captures) ->
  static scenes may then use **live-window sha equality** as the gate; animated examples
  must use keys-probe percentages / time-progress + interaction tri-state instead.

## lesson-review record (2026-10-08, band V batches 12–14 + V3/V4 close-out session)

- new: PIT-48 (harness invocation errors swallowed by `2>/dev/null` = misread as the object-under-test being dead; assert rc first),
  C19 (self-check/prune predicates must be token-level + plant a breakage probe in the check itself before commissioning; includes the honest note: the parent codemod
  still carries the crude predicate, the repo's real judge is `lint -D warnings`), D24 (shared-code value counts in sites, not lines;
  triggers covered by measurement override the plan's archived steps; V4 net line change = zero).
- extended: testing.md harness-truth item (2) gains the **fourth sighting** (E201's argv has the positional asset first; the ad-hoc
  `--frames 1` dropped into the window lane — winit's error looks like a rendering bug but is a lane-selection bug).
- this review caught one of its own: the newly written C19 check command was **a fake gate in its first version** (the predicate couldn't even hit the positive case,
  and it depended on a session-level /tmp file); rewritten as the falsifiable form with a built-in breakage probe and actually run (planted sample=1, in-repo=0).
- queue unchanged: push (the V4 commit) · five T3 human checks · the npm/pip packaging round · LAS/skinning ticket-gated, standing by.

## Publish band ledger note (2026-10-09, plan next-band-queue todo 3)
- Live counters moved by this band, measured same-terminal: ci depends-on = **12**
  (`pack-check` promoted into the chain); install tree = **9 files**, unchanged --
  no new install rules; the CPack TGZ packages exactly what `cmake --install`
  produces (file-for-file diff recorded in `.omo/evidence/todo3-tarball-list.txt`,
  LOCAL artifact only, nothing published). Packaged lib: SONAME bare name +
  system-only NEEDED; RUNPATH names this env's sibling conda prefix = the D23 P3
  assembly concern, adjudicated report-only (no patchelf), decision note in
  `cmake/VisiaEnginePackaging.cmake`. Dated baselines above keep their
  as-of-close values (append-only history discipline).

## Next-band-queue baseline (2026-10-09, plan next-band-queue, todos 1-5)
- Four commits, one per todo, each leaving `pixi run ci` rc=0: `98bdbbe` census ·
  `0682fe3` keys-probe CASES 4→8 · `10c7dc7` CPack tarball + pack-check into ci ·
  `81d89d0` npm local artifact + gate-docs check 4 extended. Todo 5 = this write-back.
- **Todo 1 census** (`docs/reference/c-example-witness-census-2026-10-09.md`): the predicate
  was written into the doc *before* any counting, then applied to 17 tracked files =
  **6 WITNESS / 11 NO-witness** (`grep -c '| WIT[N]ESS |'` → 6, `'| N[O]-witness |'` → 11,
  sum 17). Structural claim: a pixel witness **requires a `readback` call**; 10 of 11
  NO-witness files have readback n=0 and the 11th (examples/cpp/template.cpp, readback=2)
  asserts no bound -- readback is necessary, not sufficient. The WITNESS six: E701, E802, E813, E814, E816, E817.
  Of the 11, eight still claim visible output (seven render call-sites E702/E704/E801/
  E812/E815/template.c/template.cpp + the Qt pump E703), so a blanking or distortion
  regression there stays green.
- **Todo 2 keys-probe 4→8 rows** (`scripts/keys-probe.py:63`): E303/E306/E504/E901 added,
  each drag + wheel + one key. Floors = 0.4 × the row's weakest *measured* action min,
  with deliberate downward rounding (E303 15.46→6.0, E306 25.07→10.0, E504 13.24→5.0,
  E901 5.96→2.3 -- integer-floor where the margin is wide, 1-dp at E901's lower bound);
  the floor is a
  per-action contract, not a row average. 8/8 green over 5 consecutive `--all` runs; each
  new row shown red by temporarily replacing its actions with `noop` (restore proven).
- **Keys-probe exclusions**, deliberate and recorded: E201_load_gltf, where Escape is a zoom
  key (`examples/rs/E201_load_gltf.rs:52`, `Escape => dist *= 0.9`), and E501_shadow_demo,
  which has zero keyboard handling (only a MouseWheel arm) -- the probe asserts Escape closes
  within 6s, so both would be permanently red. **Noop-ban rationale**: those four window rows
  must never use `noop`, because their idle frame is static, so a noop gate would print 0.00%
  and test the harness instead of the app. `noop` survives only on E511, whose scene clock is
  the thing under test.
- **Todo 3 publish band**: CPack TGZ packages the install tree **file-for-file identical to
  `cmake --install`, 9 files**, with no new install rules (diff recorded in
  `.omo/evidence/todo3-tarball-list.txt`; LOCAL artifact, nothing published or pushed).
  SONAME bare-name consumer witness: E701's `NEEDED` line reads `libvisiaengine.so`. RUNPATH
  naming this env's sibling conda prefix = the D23 P3 assembly concern, documented in
  `cmake/VisiaEnginePackaging.cmake` and adjudicated report-only -- **no patchelf**, no new
  tool dependency.
- **Todo 4 npm**: `build/npm/visiaengine-0.1.0.tgz` = **11 files**, version read from
  `Cargo.toml` `[workspace.package]` by `scripts/gen-npm-package.sh` (never hand-typed). The
  builder is a pixi task **out of ci** (same policy as gallery / keys-probe), and the
  internal-link gate was folded into **gate-docs check 4 in place** (`.md-only` → every
  markdown link in README / docs / llms.txt). So **ci 11→12 came from pack-check alone**;
  this todo added no segment.
- Ledger at close, every value recomputed same-terminal: **ci 12 segments · spec-trace
  199↔199 · ctest 44 · gallery 44 cards · install tree 9 files · crates/ 9 (workspace 12
  packages)**. Only the ci segment count moved this band.
- **Open items (registered, not fixed)**: `examples/c/E814_labels_headless.c:48` --
  `if (white < 60) fail(...)` drops the return, so that census row's pixel bound cannot
  redden ctest; keys-probe's single-shot `noop` invocation prints a cosmetically wrong
  usage/error line (no gate impact); an exit-key convention band for E201/E501 would unlock
  two more probe rows; the npm package name was never checked against a registry (deliberate
  offline stance, nothing pushed); the npm `exports` map is deferred.

## PolyOrch embedding round (2026-10-12, in force)

Supplier consumption landed end to end: submodule pinned PolyOrch 457234e (D39 contract + D40 EXAMPLE
kind), shim reuse/embed + USE_PIXI master switch + 0.2.0 version floor (T-B1/T-D, five cmake-smoke
negative paths pin the failure lanes). Build engine: capi (cdylib+staticlib handles, ALL wrapper,
staged artifacts in cmake-build*/lib|bin, install-tree guard moved to install time) and all 28 rs
examples (polyorch_rust_build EXAMPLE kind; ctest runs staged binaries directly). Debug: the `debug`
preset opens 29 CodeLLDB launch+task pairs in the committed .vscode managed regions (idempotent).
Presets re-shaped: default/release/prebuilt + debug + ci-strict (Qt is a plain VISIAENGINE_QT
bool, no Qt preset; legacy QT_SDK five-value cache renamed with a stale-detection WARNING).
T-A0..T-F + i18n E1-E5 committed (gate-i18n in ci; PIT-49/50 recorded; C20 in force).
Open: PolyOrch upstream push (gitee auth expired -- 457234e unpublished; fresh clones cannot
resolve the gitlink until pushed); cargo-build_examples/cargo-run_* naming lives in smoke anchors
swept; T-D deferred items none; examples' RA lane unaffected.

## Naming + taxonomy round (2026-10-12/13, T-H2/T-H3/T-H3.5 + user-caught PIT-51 fix, in force)

Bindings face normalized on the four-axis grammar: link faces
`visiaengine::c/::cpp/::qt` (real names `visiaengine-c/-cpp/-qt`; aliases are the
only contract -- `::capi` renamed per the iceoryx2-c/zenoh-c consumer-language
corpus), IDE viewers `visiaengine-{c,cpp,qt}-sources` (polyorch_header_face
viewer+audit; viewer-only STATUS contract on the PolyOrch side, 1ee0359),
aggregate `visiaengine-wasm-all`. Directory taxonomy single-axis: crates/ = the
rust set incl. visiaengine-ffi (ex bindings/c/visiaengine-capi, crate renamed)
and visiaengine-wasm (ex bindings/js/rust); bindings/ = pure consumer faces with
the C header at bindings/c/include/. qt link face repaired (the T-H commit
promised ::qt but never landed it -- Momus round-1 BLOCKER, user-adjudicated
BUILD; examples/qt re-linked off its raw include-dir anti-pattern). CMake
handle names follow the home (visiaengine-ffi-*; the user-caught PIT-51 fix).
Button grammar finalized (-build/-test/-clean plain verbs; the rusttest
coinage retired; the wasm glue buttons merged into the deliverable-word
aggregate visiaengine-wasm-js-build -- one button, all flavors, C22
deliverable-word clause). T-H5/T-H6 (2026-10-13): the four example
lanes share ONE head word -- visiaengine-example-<name>{,-run,-build}
across rs+c+cpp+qt (source files keep bare E names, R10 amended); wasm
gains the serve button (visiaengine-wasm-serve: assemble web-demo +
python http.server :8017, wasm MIME pinned, stale-glue replay edge);
PolyOrch gains polyorch_example_run + polyorch_rust_wasm_serve; the
cargo-run_/rs-/run_ prefixes are retired repo-wide; .vscode regen on
rename (delete + reconfigure). Machine gates: ctest 108/108, pixi run ci ALL GREEN,
gate-abi/gate-i18n green, negative audits both header lanes (hpp +
visiaengine.h delete->FATAL->restore).
Push backlog: PolyOrch 6 commits + V commits pending gitee credentials.

## T-H12 round (2026-10-14, the ffi aggregate cleanup)

visiaengine-cargo (last cargo- coinage) -> visiaengine-ffi-build (lane
word, symmetric with rlibs-build); smoke/install/preset texts follow,
zero-residue sweep green. The deletion probe REVERSED the redundancy
guess: the example->cargo ordering bridge is LOAD-BEARING (the link
face chains the artifact file path; file deps propagate no target
edges) -- kept with its evidence in the comment (edit-safety rule 33
born here). FOLDER re-homed per the T-H10 rule's corollary: an
aggregate with an on-disk home goes to it (crates/visiaengine-ffi,
where ffi-test already lives); the root group is now strictly the
homeless tree-level zone. Commits 2b14dc5/d6899b7.

## T-H11 round (2026-10-14, the web example wall -- P1+P2 landed)

examples/wasm/ is now a three.js-style wall: hub index.html (searchable
43-entry sidebar from wall.json; iframe viewer; placeholder cards with the
ctest how-to for native-only entries) + common.js bootstrap + 6 LIVE pages
(E101/E201/E202/E401/E811/E813 -- the entire wasm surface gets pressed by
the wall itself). wall.json = the single list source; gate-docs ⑧ locks
number-set equality wall<->registry + live-entry html existence. serve
globs the wall, stages park.geojson + hierarchy.glb, and the glue .js are
REAL file-level DEPENDS now (supplier 4dfa89b: the ';' in a -c one-liner
split into argv -- server exited 0 silently; and serve served an EMPTY
pkg/ without the js edge; post-hoc set_property(DEPENDS) never reaches the
Ninja graph -- constructor only). Live: [4/5] replays both flavors, all
probes 200, .wasm = application/wasm. gallery REMOVED at P3, same session (11-point list executed: targets/pixi tasks/gen_gallery.py/gate-docs ⑥⑦/smoke/README/tutorials/roadmap/rule text -- save_frame thumbnails KEPT per adjudication; wall.json is now the four-consumer card). Old text (P3 carries
the removal contract: 11-point list + judgment in the plan (Momus write-back); trigger
goes to Open Items at P3 time). Gates: suite 80/0/26, gate-docs/i18n ✓,
ci ALL GREEN.

## T-H10 round (2026-10-14, the aggregate opt-in flip + root-group migration)

The user caught the default-named polyorch-rust-all aggregate drifting
into the examples/rs folder: a redundant tree-wide button (every lane
already aggregates: visiaengine-rlibs-{build,test,clean},
visiaengine-example-build/-clean, wasm family) whose FOLDER was
rewritten by the last registering lane. Supplier flipped to strictly
opt-in + first-writer folder pin (PolyOrch 2f71338); V's own rs
aggregates moved to the root group (no FOLDER -- the rlibs sibling
pattern; fafe3c7). Verified: help shows zero polyorch-rust-all;
example-build rebuilds the full gallery (32/32); suite 80/0/26;
default preset green.



The knob family is namespace-ordered (PolyOrch_VSCODE_DEBUG_{ALL,RUST,
NODE,PYTHON,NATIVE}), ALL boolean -- ON forces every face (the full
octk_option port landed: six keywords + expression evaluator, user-
adjudicated; the OR_CONDITION cache-FORCE pins display=effect). The
NATIVE face gives c/cpp/qt executables CodeLLDB F5 rows via
polyorch_native_debug (self-target task rows; path computed at
registration; MC expanded). V: debug preset = 29 rs + 14 native
launch rows (Qt's E703 arrives with the opt-in lane, VISIAENGINE_QT=ON
-- same macro shape, wiring proven). Machine gates: suite 78/0/26,
t-option 10 cases, t-native-debug 5 legs; V build rc=0, example ctest
36/44 (8 = the headless display lane standing state), ci ALL GREEN.

## T-H7 pending gate (2026-10-14, the VSCode debug knob family round)

- **T-H7 legacy-knob removal gate** (the T-H7.1 removal contract): delete the
  three compat reads (PolyOrch_{RUST,NODE,PYTHON}_VSCODE_DEBUG -> the
  PolyOrch_VSCODE_DEBUG_* family) when BOTH hold: (a) zero grep hits for the
  three old names over both repos' formal surfaces, (b) one minor cycle since
  landing. One cleanup commit: compat reads + STATUS lines + this line,
  together.

## Stale-fixes round (executed 2026-10-15, team audit + 11 user-adjudicated points, serial workers, ci green per phase)

V-A: smoke-x11 include path fixed (the dir never existed; compiler-proven); `N` artifact removed; llms counts 49/44 via generator bump + NEW gate-docs leg 10 locking llms==README==live census (negative proof: planted `50 entries` observed red); web-serve.sh tombstone deleted; `gen-hdr` task registered (keys-probe precedent). Deps: rustls 0.23.45->0.23.46 clears the 4 RustSec advisories (de3fd05; upstream DB surfaced them against the pre-existing lock; diff = version+checksum+one in-range dep-ref). V-B: activation trio is now PolyOrch-managed generated content at root (`polyorch_pixi_setup(COPY_SCRIPTS)` + bootstrap cp for the cold window; hand copies git-rm'd + gitignored); bootstrap.bat delegates to pixi-init.bat (T-B2 mirrored), PIXI_PIN gone, env.md pin prose = CI + upstream default. V-C: modules renamed VisiaEngineFfi/Crates/Wasm + lowercase fn names (VisiaEnginePolyOrch stays as the SOLE seam file; conventions.md C21 check-body path fixed in the same commit with gate-liveness red proof); pre-T-C dead block + three SDK set-never-read exports deleted; Packaging/WARNING/README texts corrected; the `debug` preset became the FULL-FEATURE F5 tree (Qt+knob, own cmake-build-debug, runs in the qt-spike env via `pixi run f5`) -- first-ever committed tasks.json, launch rows 43->44 incl. E703, everyday presets provably never touch .vscode (PIT-64/65/66 filed). P-batch: PolyOrch rust-bindings stops re-concatenating the lane prefix (four sites; bare standalone names frozen; fused names single-prefixed app/consumer/clean/cxx-regen proven), tutorials :98 + :167-168, three examples comment blocks, and the loader-active status.md verification row re-copied from LIVE target lists; PolyOrch suite RECORDED offline 80/0/26 / e2e 102/0/4 at parity. Ledger: .omo/plans/2026-10-14-stale-{audit,fixes}.md. Commits: V a0c4489..8687d42 (14 incl. deps+memory), PolyOrch cf5f1f9/34dae27, gitlink advanced (all still unpublished -- gitee blocker).

## Reuse-audit round (executed 2026-10-15, adjudications 13-17, all five cashed)

13+17 (run buttons -> supplier machine): three hand sites swapped to polyorch_example_run (WORKING_DIRECTORY + GUI splat shape), scripts/run-gui.sh retired with reference sweep, d45efd1 -- path required a supplier side-quest (17=R1c: GUI form set()+re-expansion split the probe at ';' -- inline COMMAND quoting now load-bearing + t-example-run LIVE-exec leg with negative proof; PolyOrch PIT-65; v0.2.2 e7f61ab). 15+16 (Qt): SDK two-generation discovery (Qt6 -> Qt>=5.15, canonical VISIAENGINE_QT_* outputs, generation-honest STATUS), E703 folded into visiaengine_add_example (one compile guard from the REAL 5.15.15 build: VE_MOUSE_POS; wheel unguarded -- position() exists since 5.0, measured; zero moc stance kept), qt5-spike validation env (5.12 explicitly out of contract per owner directive), smoke-qt's T-H6 path-drift fossil fixed en route -- both generations build AND run real windows (e51fae8/ed140a3/9d49915). 14 (rlib): supplier v0.3.0 LIB kind (D42: cargo --lib restriction, platform-invariant lib<name>.rlib naming proven from rustc output.rs, INTERFACE handle + POLYORCH_RUST_LIB, default dispatch untouched; aec3278/b078e51/d96713c) + host Crates lane collapsed onto it (ba72ac4): 283 addressable target names byte-preserved, ctest 44 identical, bare-PATH build wrappers retired (PIT-14 leak class killed at the lane, --unset lines visible in generated ninja), 9 rlibs cold-built 23.6s through the face, full ci 345 tests green. Acceptance-phrasing lesson: "help diff = 0" is the wrong pin when a face stamps outputs -- the honest criterion is addressable-NAME-SET identity (worker caught it, ticket author = orchestrator). C23 first runs held (dev-visiaengine work, ff main, suite-on-main, HEAD back to dev); inner a246861 (PIT-65 memory commit) had in fact PUSHED (ls-remote re-check after a swallowed-output timeout; PIT-69 filed for the phantom claims in the closing report -- c3c5355/39-backlog were fabrications, real backlog 2); outer pin followed to a246861 once publication was proven, closing the M-PolyOrch drift.

## Headless-page-gate round (executed 2026-10-16, owner-adjudicated chain: download / heartbeat / option-4 beacon)

gate-web joins the gate family (scripts/gate-web.sh, 155 lines: ~65 bash + embedded stdlib python gate server on :8018; pixi task + ci step wired, honest self-SKIP). Wall count 43/43, engine fail-marks + Uncaught legs live, heartbeat via anchor-img + 8-frame beacon (VE_GATE_FRAMES/VE_GATE_MIN_FRAMES coupling commented both sides). Chromium 131 CLI-only (npmmirror playwright-cache path; conda-forge has NO chromium package, measured), WebGPU over swiftshader with the D26 flag combo; D-Bus proxy scrub (PIT-70). C0 duty: orchestrator's independent FIRST run went RED (cold wall race) -> root-caused to a bounded 3-roll re-jury (verdict unsoftened) -> 2x green + `pixi run gate-web` green; negative ledger (a)-(e) all observed red-in-channel (glue-throw, port-BUSY, no-manifest SKIP, boot-throw ✗, beacon-removal). Numbers: beacon 618-1353 ms, full run ~4 s warm. D26 records the five rejected browser channels with evidence. PolyOrch gitlink advanced to 5945f2a (owner sync, CMP0174 guards, c5d4d1b -- publication ls-remote-proven per C23). Plan: .omo/plans/2026-10-16-headless-page-gate.md. ## Serve-https lane (executed 2026-10-16, owner project-approval after the LAN secure-context report)

Host-side TLS sibling of the wasm serve lane: scripts/wasm-serve-https-gen.sh (idempotent mkcert CA+leaf, SANs localhost/127.0.0.1/192.168.122.177/10.144.0.10, NSS step degrades honestly -- no certutil on this host) + scripts/wasm-serve-https.py (32-line stdlib ssl static lane :8443, guards: missing dir/cert/BUSY) + pixi tasks serve-https-gen/serve-https + .gitignore .certs/. Measured: https 200 on LAN IPs, wall 43/43 + beacon-over-https GREEN with the UNtrusted cert = secure-context confirmed (chrome 131, --ignore-certificate-errors route empirically backed); four negatives red. Cross-browser remote live-fire awaits the owner's GPU machine (T3). PIT-73 (setsid $! vs port holder) + PIT-74 (orchestrator spec-confabulation retracted before damage) filed. Commits c7fdeb4.

Closeout same day (59df128/aec496d): chromium pin lives IN the gate (exact-match --version drift red / honest SKIP absent / opt-in sha-verified provisioning) because conda-forge provably carries no chromium; anchor-beacon heartbeat rolled to all 6 live pages with per-page READY namespaces (beacons 581-975 ms measured, E401 beacon-cut reddens only its own leg).

## Dual-serve round baseline (2026-10-19, owner-adjudicated: one button, http+https)

- owner request: the `visiaengine-wasm-serve` button must start BOTH servers and its
  `[n/m]` build-line must name both URLs. Root lesson: supplier's serve function bakes
  its Ninja DESC at add_custom_target time and is C23-frozen, so the host registers the
  button itself (VisiaEngineWasm.cmake, assembly recipe mirrored from
  PolyOrchRustHelpers.cmake) -- `COMMENT` written at registration carries both URLs.
- behavior: `.certs/` present (serve-https-gen once) = dual serve, http :8017 (dev +
  gate-web replay) + https :8443 (LAN secure-context WebGPU, self-signed cert, SANs
  cover 192.168.122.177 / 10.144.0.10); no certs = supplier's http-only form verbatim.
  The pixi `serve-https` standalone task stays as the non-button alternative.
- measured: DESC line in build.ninja carries `...http://127.0.0.1:8017 +
  https://<LAN-IP>:8443...`; generated script py_compile OK; live run http 200 + https
  200 (real TLS handshake); verify processes cleaned, port handed back to the IDE.
- new lessons: PIT-75 (DESC baked at registration; register, not patch) / PIT-76
  (file(WRITE) multi-line parse error + stray-`]` smuggling; py_compile the artifact)
  / PIT-77 (port-bound verification must return the port; setsid children survive the
  launcher kill) + edit-safety 39/40/41.
- open: commit awaiting user instruction; gate-docs has no anchor on the serve DESC
  text (checked: no gate greps it); supplier retirement condition recorded inline
  (adopt when PolyOrch grows TLS_PORT/TLS_CERT/TLS_KEY args post-unfreeze).

## Review-fix band baseline (2026-10-19, 5-agent review FAILED -> three blocking fixes landed)

- origin: the /review-work 5-lane round (goal/QA/quality/security/context) on the wasm band;
  3 FAIL 1 PASS 1 INCONCLUSIVE; the QA lane's live probes caught what the green ci could not.
- fixes (3 commits): E811 fixture = twoprim.glb (the "two-primitive" claim was a
  1-primitive poser) / mount_gltf framing fit (E202 family law, gltf leg -- the QA
  "wasm pick dead" report was THIS engine bug: hierarchy.glb off-frustum, 0 pick hits;
  oblique-depression camera per the E201 pitch law) / E813 knife sign [0,0,-1,v]
  (keeps=n*p+d>=0) + pick-target sweeping for the de-centered consumers (E701/E811_c/
  E811cpp) + gate-web consumes the ?selftest beacon (E401 asserts RED+GREEN, 165 cells).
- disproven en route: the matrix-transpose hypothesis (gltf crate is column-major per
  the glTF spec, verified from crate source; the transpose was reverted same-session).
- honest note: the QA lane's "wasm32-specific" framing was wrong -- the bug was engine-
  level; the wasm surface just had no fit and no page-level pick probe to see it.
- new gates: gltf_node_transform_pick_preserved / clip_knife_sign_semantics (both
  RED-proven) / gate-web E401 pick-surface assertion (RED-proven by its first run).
- baseline: pixi run ci rc=0 (13 segments); ctest headless lane green; gate-web OK with
  the pick assertion; worktree clean; 3 commits this band.
- remaining: user hard-refresh retest of E401/E811/E813 (staged artifacts good);
  group-offset pick leg landed in the same session as the review's blocking #3.

## Web wall expansion band baseline (2026-10-23, live pages 6 -> 26)

- six sessions of wall expansion: E302/E306/E502/E814/E505/E812/E402/E403/
  E305/E901/E304/E301/E203/E506/E509/E511/E508/E510/E501 live, plus the
  E302/E306 C-twin cross-references (E815/E816 stay native by family).
- bridges added: setMorphTime/clearMorph (REND-41), loadEnvHdrBytes (bytes
  entry + wasm leaf), setProjection/getProjection (N2.1 seam + query),
  pickPoint (pick_geo now returns the full surface point; pick_hit engine
  face), setPostEffects (flat kind/p1/p2 chain: bloom/outline/tonemap/ssao/
  haze; post field wired through BOTH Frame sites AND the wasm-only
  new_canvas constructor -- the N4 band's web-check catch repeating), setEdl,
  setShadow (light-dir -> CameraRig construction, E501 family).
- gate hardening: gate-web PAGES + summary derive from wall.json (the
  hardcoded list drifted twice); tag-balance check per live page (PIT-82
  breakage probe red-proven naming E812).
- lessons: PIT-81 (veLoop onFrame takes NO arguments; ad-hoc chrome probes
  need the D-Bus scrub -- empty dom + no console = probe-side), PIT-82
  (unclosed <script type="module"> = the page NEVER evaluates, zero console
  evidence; node --check cannot catch it -- sed range falls through to EOF;
  tag balance grep is the gate), C21 re-sighting (quoted 'a;b' in the copy
  foreach is ONE string; assemble failed loudly), bindgen glue does not
  track lib.rs (rm -rf web/pkg-web forces the replay).
- baseline: pixi run ci rc=0; web-check + MIRROR 3/3; gate-web 26 live
  pages all beacon green (396-2219ms, E501's shadow pass the slowest);
  43 wall items = ctest = registry; worktree clean; 14 commits ahead.
- remaining native-only: E204 (load_pcl bytes needed), E205 (text labels
  need the C text port semantics), E206 (load_tile_file bytes needed),
  E303 (render_view_rects bridge, the largest face), E503 (strokes bridge
  absent -- addPoints covers dots only), E504 (alpha hot-update semantics
  via addMesh rebuild is possible but the native page teaches the material
  hot path), E601/E70x/E801/E802/E810 (bench/window/CLI families, non-web),
  E815/E816 (C twins cross-referenced), E817 (loadRasterDir bytes needed).

## Web wall expansion addendum (2026-10-23 same band, live pages 26 -> 29)

- three bytes-bridge pages: E204 (loadPclBytes was already on the face --
  zero new bridge), E206 (MemSource in io-tiles + load_tile_bytes on the
  engine; mount_tile_layer tail carved out of load_mvt_dir and shared),
  E817 (load_raster_bytes single-tile quad tail + loadRasterBytes; the
  band-T same-domain pair: raster_over underlay + vector overlay, z10).
- wall now 29/43 live (67%). Bridge face 41 -> 51 across the whole band.
- a botched extraction (mount_tile_layer) produced duplicate mount_layer
  blocks and an unclosed fn -- reverted to HEAD and redone with window-based
  string surgery; the compiler spans named every defect. Lesson: large
  multi-site extractions go through revert-and-redo, not incremental patches.
- remaining native-only (14): E205 (C text-port semantics), E303
  (render_view_rects bridge, the largest face), E503 (strokes bridge),
  E504 (material hot-path semantics), E601/E70x/E801/E802/E810 (non-web
  families), E815/E816 (C twins cross-referenced).
- 17 commits ahead at close; worktree clean; ports returned after every
  verification run.

## 2026-10-30: the docs book line closes (S1-S8 -> D46 -> polish)

- The mdBook site is English-only (D46): the PO translation pipeline was removed
  after five batches reached ~86% calibrated coverage; the 513-string lineage is
  preserved on the local branch `backup/pre-squash` (delete only on explicit
  owner instruction).
- Structure polish landed (plan 2026-10-30-book-structure-polish, expert-revised):
  gate-prose enforces heading length/style and paragraph/list-item length; the
  22 real findings were fixed information-preservingly.
- The sidebar folds: SDD contracts (9) and the reference library (37) nest under
  draft parents; gate-docs 13 now checks indented SUMMARY entries.
- Pending: push (4 commits), backup branch deletion decision, optional C24
  subject fixes on two PUSHED commits (force-push required -- owner's call).

## repo-audit Full Scan — 2026-10-30 (first Full run)

- **Local (24 items / 10 domains)**: mechanical health green across scripts (0 syntax
  failures, 0 mixed EOL, 0 orphans after verification), naming (0 case collisions,
  task style clean), cli (USAGE sh/bat parity), tests (registry<->examples complete),
  mcp (3/3 pins match), book vendor+licenses complete; gates green this session.
- **Findings**: (1) CRITICAL/domain-5: decisions.md carries two D45s and two D46s
  (numbering-uniqueness defect; fix = manual renumber AFTER the parallel D53 session
  closes -- same file); (2) HIGH/domain-6: toc(16) != SUMMARY(54) chapter counts,
  attributed to the parallel D53 WIP mid-flight, re-verify after it lands;
  (3) LOW/domain-2: conventions reserved-slot notation (C2-C8/C10-C14 documented in
  the header) defeats naive contiguity probes -- future scanners must read the header.
- **External scan**: P2 sammcj/agentic-coding#rust-engineer (6.3/10, Apache-2.0,
  SAFE-with-warn: the author parks it under skills_disabled/; overlaps our own
  rust rules -- skip unless a generic-Rust companion is wanted). P2 mattpocock/skills
  (7/10, 51k stars: adopt the skill-structure discipline, specific skills are
  TS-domain -> rewrite-to-fit). Rejected: skillhub (enterprise registry, mismatch),
  skillfile (skill manager, we are our own), jcode (harness, not a skill).
- **Next scan**: first check whether findings 1-2 were fixed by the D53 session.

## repo-audit Quick Scan — 2026-10-30 (closure re-check)

- Full-scan finding 1 (D45/D46 duplicates): **CLOSED** -- renumbered to D54/D55 in
  3696c26, uniqueness re-verified (no dups, no holes, max=D55).
- Full-scan finding 2 (toc 16 != SUMMARY 54): **RECLASSIFIED as a probe defect,
  not a repo defect** -- the probe counted only [[chapter]] entries (16) against
  all SUMMARY lines (54 = 52 entries + 2 draft parents). Correct accounting:
  [[chapter]] 16 + [[group]] glob expansion 36 = 52 == SUMMARY 52. Match.
- Quick sweep: 10/10 domains green (frontmatter, numbering, mcp pins, script
  syntax, cli parity, registry spot-check, naming). Three probe artifacts
  corrected for future runs: (a) conventions "holes" C2-C8/C10-C14 are documented
  reserved slots; (b) pitfalls PIT-27/63 gaps are documented intentional (header
  note); (c) the structure probe must compare hidden dirs on both sides and
  exclude generated dirs (node_modules et al); AGENTS.md tree rows all resolve.
- No new findings. Next scan: fresh cycle.
