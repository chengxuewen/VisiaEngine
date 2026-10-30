# VisiaEngine Architecture Decision Record

> Records decisions that affect interfaces, data flow, dependency direction, or build shape. Each entry carries a number, date, decision content, rationale, and impact. Option-comparison presentation follows C1 (pros-cons / source / impact / recommendation, four elements). Overturned decisions are marked SUPERSEDED with the new number; entries are never deleted.

## Format example

## D<N>: <decision title> (YYYY-MM-DD)
- **Background**: <why this decision was needed>
- **Decision**: <what was chosen, what was dropped>
- **Rationale**: <key reasoning, including sources/referenced practices>
- **Impact**: <modules, conventions, and follow-on development affected>

<!-- Real entries append below. The predecessor MediaServo decision history is in the .refinfo archive. -->

## D1: Ported-config neutralization (MediaServo → VisiaEngine, fresh start) (2026-09-03)
- **Background**: the repo's .agents/.opencode/.omo were ported wholesale from the completed project MediaServo (Rust WebRTC); 3437 lines of old-project memory + 14 hard-referencing skills would mislead the new project.
- **Decision**: position as a brand-new project → memory zeroed to templates (keeping only the three general methodology entries C1/C9/C14, original numbering retained to avoid dangling references); the 14 skills neutralized in place, not deleted; rules spot-generalized + platform.md deleted; instructions dropped docker/platform/rust (stack undecided → no preload of language rules).
- **Rationale**: the old project's history is fully archived at .refinfo/MediaServo (gitignored, on this machine); overwrite/delete loses nothing.
- **Impact**: the config loaded each turn no longer contains MediaServo architecture; once the stack is chosen, backfill at the placeholders (see status.md next steps). Verification: dual grep gates passed.

## D2: Tech stack and product form settled (whitepaper v0.1.0 first draft) (2026-09-03)

> **Update**: this entry's impact ④ — the render-backend final ruling — has been superseded by **D4** (wgpu direct use, self-built pipeline adopted).
- **Background**: the user submitted the《VisiaEngine whitepaper》first draft, positioning a multi-dimensional spatial visualization engine (unified 2D/2.5D/3D viewport, for GIS / digital twin / AV simulation / BIM), explicitly not a game engine.
- **Decision**: Rust core (`visia-core` scene graph / coordinate systems / spatial index + `visia-render` abstraction); rendering on wgpu (Vulkan/Metal/DX12/WebGPU), whitepaper tentatively Bevy as default backend (`visia-render-bevy`); the C API is the sole stable FFI boundary, shipping SDK form without hijacking the host main loop; Open Core: the core stays MIT/Apache-2.0 forever, commercial product = Visia Studio editor + enterprise plugins/support; startup size target ≤10MB.
- **Rationale**: fills the gap between game engines (heavy, weak GIS) and GIS/web libraries (hard to embed in native desktop, 2D/3D on different cores); benchmark precedents: Cesium (unified viewport but JS-first), MapLibre (map-only), Three.js (no spatial-data layer).
- **Impact**: ① stack = settled, rules/rust backfill into instructions unlocks, pending execution; ② crate naming visia-* settled, wholly unrelated to the predecessor MediaServo (only the toolchain config was ported); ③ whitepaper §3.2 "native ODR/OSC parsing" contradicts §6.2 "Beta plugin" in commitment level — must be unified before release; ④ **the Bevy binding approach is a load-bearing undecided decision** (bevy_render is deeply ECS-coupled; "decouple ECS, use its renderer" is practically near-infeasible); ⑤ the whitepaper's full text currently exists only in the session — its landing location awaits adjudication.

## D3: Whitepaper v0.1.0 committed with wording revisions (2026-09-03)
- **Decision**: the user's whitepaper first draft, after review and edits, was committed to docs/whitepaper.md + a condensed README.md. Revision scope (user-approved): ① §4 dropped the "Bevy default backend" promise, replaced with "wgpu base + two backend candidates pending final ruling", architecture-diagram backend box renamed visia-render-<backend>; ② §3.2/§5 ODR/OSC downgraded from "native parsing" to "official plugin (Beta)", aligned with the roadmap; ⑤ §1.1's critique of Cesium rewritten precisely (cross-dimensional scene semantics differ, native-embedding cost), avoiding factual errors; ⑥ date changed to 2026-09, license gained the "released versions are irrevocable, no retroactive closing" promise, Visia etymology revised to the honest "vision-family coinage". Not revised: §3.3's 10MB figure and projection-range wording (not in the approval list; to be re-checked together with the D4 backend ruling).
- **Rationale**: the whitepaper carries externally stable promises; anything conflicting with engineering reality/facts gets fixed before commit; the backend ruling is undecided and must not be pre-locked by the docs.
- **Impact**: the whitepaper becomes the project's first-hand positioning source (AGENTS.md already points to it); after D4 (backend ruling), §4 needs a second backfill.

## D4: Render-backend final ruling = wgpu direct use, self-built pipeline (Option B) (2026-09-03)
- **Background**: whitepaper §4 had tentatively "default backend Bevy"; D2 impact ④ held the final ruling. Four research lanes delivered (evidence/2026-09-03-wgpu-direct / -bevy-embed / -angle-integration / legacy-platforms.md, all live-cited 2026-09-03).
- **Decision**: **Option B adopted** — self-built render pipeline directly on wgpu; default-backend crate renamed **`visia-render-wgpu`**; `visia-render-bevy` not started. Option A (reuse the Bevy stack) rejected; Option C (A first, then B) rejected (the costliest layer written twice + C ABI leakage).
- **Rationale** (each Option-A rejection point has evidence): bevy_render is inseparable from App/ECS and 0.19 deepened it (RenderGraph removal measured); rendering inside a host window has zero precedent (godot-bevy abandoning the renderer is circumstantial evidence); no 1.0 promise + 120 breaking changes per release + ecosystem lockstep cases — conflicts with "stable C API SDK"; wgpu direct keeps main loop / size / compat tier fully self-held. Known B costs: frame graph / pipeline cache / GPU culling / 3D Tiles streaming — none has a mature piece (permanently self-held); MVP 15-27 person-month composite estimate.
- **legacy ruled together**: XP withdrawn from all external text (ANGLE D3D9 already removed + Rust MSVC = Win10 floor); Win7 = unofficial best-effort (not in promises); Android floor API 24 (system GLES/Vulkan dual path, ANGLE irrelevant); RK3588 (Valhall G610) = Panvk or vendor libmali blob, **released binaries lock one driver stack**. §3.3 wording upgraded: first-class = Vulkan/Metal/DX12/WebGPU, degraded tiers = GL 3.3+/GLES 3.0+/WebGL2 ("what is consistent is the API surface and tier semantics, not identical image quality").
- **Impact**: whitepaper §3.3/§4.1/§4.2/§6.2 backfilled (same day); README architecture row updated; AGENTS.md CODE MAP planned crate changes; MVP-period self-risk items: 100k-entity frame budget, wgpu quarterly breaking-change pin strategy, cosmic-text CJK atlas quality.
- **SUPERSEDED statement**: this decision supersedes D2's "whitepaper tentatively Bevy default backend (pending final ruling)" wording; D2's remaining content (stack = Rust / SDK form / Open Core) stays valid.

## D5: Development environment fully managed by pixi, rust toolchain included (2026-09-03)
- **Decision**: the development environment is managed uniformly by pixi (conda-forge single source); **the rust toolchain is also provided by conda-forge**; the rustup main path is not adopted. The only prerequisite on a new machine = `source bootstrap.sh` (auto-installs pixi → everything comes from the lock).
- **Rationale**: measured conda-forge Rust family is complete (rust 1.98 tracks monthly + rustfmt/clippy/rust-src/rust-analyzer + `rust-std-wasm32-unknown-unknown`/android + cargo-deny/wasm-bindgen-cli/cargo-binstall); single lockfile, mature domestic conda mirrors, one mental model. The predecessor project's daily main development ran entirely on conda rust; its pitfalls concentrated in cross/vendor-stack scenarios (not things conda rust couldn't install). The "pixi shell + rustup toolchain" hybrid (C) was evaluated and demoted after its wasm/cross-std arguments were overturned by probing.
- **Clauses (boundaries)**:
  1. Windows escape hatch: win-64 enters platforms; if win conda rust friction exceeds the bar, per-machine rustup+MSVC overrides without polluting the mainline.
  2. Embedded cross (vendor devices of the RK3588/Jetson class) always goes Docker/vendor toolchains; conda rust does host builds only — the boundary-handling of predecessor PIT-85 (conda rust injects `CARGO_TARGET_*_LINKER` priority above `.cargo/config.toml`).
  3. cargo tools with no conda package (tarpaulin/cbindgen etc.) are obtained as prebuilt binaries via cargo-binstall (conda has the package).
  4. CI pins pixi-version; lock drift is gated with `--frozen`.
  5. `pixi.lock` must be committed with the same discipline as `Cargo.lock`.
- **Impact**: on the work-start day, generate pixi.toml / bootstrap.{sh,bat} / pixi.{sh,bat} / docs/env.md; `rules/common/constraints.md` gains the environment discipline addendum; rustup never appears on the machine (except clause ①'s escape scenario).

## D6: Technical namespace unified to visiaengine-* (2026-09-03, user ruling)
- **Decision**: brand name = **VisiaEngine** (PascalCase, docs/UI/formal address); technical prefix = **`visiaengine-`** (lowercase full brand name: crate package names / lib underscore names / workspace paths / CI and gate references, full chain). C ABI layer: symbols `visiaengine_*`, header `visiaengine.h`, library `libvisiaengine.{so,dylib,dll}` (capi slice delivers it). Environment-variable prefix reserved: `VISIAENGINE_`. Layering exception: **product names keep their own form** (the editor "Visia Studio" is not a technical namespace).
- **Revision surface**: crate-name wording in D4's historical entry stays as historical fact, not rewritten; from this entry forward, all present-tense docs use the new names. SDD clause numbers (CORE/REND/WGPU-*) are independent of crate naming — zero change.
- **Rationale**: ① "visia" is not the official product address; the full name is trinity (repo name = brand = prefix) with search monopoly and zero ambiguity; ② measured obstacle-free — both candidate namespaces free on crates.io, GitHub shows zero C-symbol use of `visia_`; ③ the end-state lesson of the predecessor's D209→D221 double full-rename tax: get it right once = brand lowercase full name; ④ the original counter-proposal "C ABI short-prefix dual track" was overturned on user challenge (the C prefix can be changed in sync; long prefixes like gdk_pixbuf_* have C-idiom precedent).
- **Impact**: one atomic rename commit (3 crate dirs + package names + references + CI + present docs, committed only after `pixi run ci` and the three golden tests all green); the historical surface (old entries in decisions/pitfalls, docs/reference snapshots, commit history) is not rewritten. If the brand ever evolves again, the rename-tax cost of relapse is backstopped by the "settle it before release" discipline.

## D7: P1 adjudication — RTC adopts the layered origin scheme (viewer-origin rebase, 2026-09-03 user ruling)
- **Decision**: the coordinate-precision mechanism is fixed as a three-layer union: ① core world = **f64 global** (ECEF/projected-domain numbers, semantic ownership on the io data-source side, orthogonal to this issue); ② vertex storage = **local f32** (relative to model/tile-level origin; at local ≤64km f32 keeps mm); ③ draw-uniform translation = **f64 `origin−eye` then drop to f32** (per frame / on camera motion threshold, v0 every frame). CesiumJS global-absolute vertices + high/low split are not adopted (that compensates its legacy baggage); shader f64 not adopted (banned on WebGPU); per-frame vertex rewrite not adopted (big-model VBO churn = death).
- **Rationale**: four independent convergences (3D Tiles tile transform / MapLibre tile-local+u_matrix / deck.gl layer origin / UE5 LWC — whose per-component specialization was deprecated and reverted in 5.0, the cautionary tale against "over-layered origin"); Cesium `czm_translateRelativeToEye.glsl` source-proves viewer-origin rebase semantics; this scheme's pure-f32 upload surface → **the Web SDK route needs zero extra work**; incidentally makes explicit the G2 leftover `model f64→f32` large-coordinate step-size debt (1e7 m level step ≈ 1 m).
- **Impact**: implementation timing = **GeoJSON slice** (glTF demo coordinates are small, current path compatible, zero change); core SDD will add CORE-11/12 rebase-behavior assertions at that time; tile-split granularity (when map-level sub-mm needs arise) = io-geo decides, engine mechanism unchanged; for extreme remote-precision needs, a Cesium-style high/low path is reserved as the upgrade route (not implemented by default). architecture.md ④/open-points table synchronized and closed.
- **Evidence**: `docs/reference/evidence/2026-09-03-rtc-hierarchy.md`
- **Erratum added (2026-09-03 planning round)**: "core SDD adds CORE-11/12" was mis-attributed — the rebase combination math lives in render (core has no matrices/wgpu semantics); actual numbers = REND-19/20; the CORE namespace gains nothing on this chain.

## D8: Host embedding surface spec (approved 2026-09-11 as final; pre-registered 2026-09-03)
- **Source**: Easy3D embedding-domain research [E3D:C1/C2/C5] (docs/reference/easy3d.md); its Qt integration relies on a QApplication::notify private hack, the ImGui version loses mouse coordinates, GLFW pointers leaked into the public ABI, bindings were auto-scanned wholesale — all four pits are counter-evidence for us.
- **Pre-registered content**: ① engine zero-event API: hosts inject uniformly via `visiaengine_on_input(ev)`, outbound only `frame_requested`/`redraw_needed` callbacks; never require host subclassing or event polling; ② the rendering contract surface (render trait + IR) must carry no window/surface backend types; the rwh handle appears only at the capi boundary (Easy3D's renderer module with zero glfw references proves this purity attainable); ③ C ABI first version = a 6-line demo proving the minimal set (create → ready to render, zero multi-stage init), unbound items go through the graveyard process; GLOB-style full-set binding scans forbidden.
- **Finalization addendum (batch 2, I0-I4 all cleared, 2026-09-11)**: ① handle = u64 (slot<<32|gen), slot base 1, C header exposes zero bit-layout; ② thread exception set = {abi_version,last_error}, destroy rejects cross-thread destruction, re-entering a released handle returns -1 (double destroy not swallowed); ③ **frame_requested/set_callbacks rejected** (pull model ve_render + host rAF/paint decides for itself; the SDL3 two-tier precedent on record; re-evaluation trigger = Qt animation round); ④ attach atomicity (on failure the headless current target is preserved) + x11 display mandatory / win32 hinstance 0 = automatic; ⑤ **the whole wasm-side C ABI is cfg'd out** (no_mangle and the bindgen export table are mutually exclusive under wasm-ld, measured) — JS = the separate crate visiaengine-wasm, zero erosion of the C symbol graph.
- **Verification anchors**: gate-abi (14-symbol whitelist + demo compile-run) · SMOKE-X11 'OK capi x11' real window three frames · 'OK capi headless input+pick' · CAPI-01..09 all fifteen clauses carry tests.

## D9: P2 adjudication — style spec adopts the MapLibre v8 paint-alias subset (2026-09-03, user ruling, Option B)
- **Decision**: on top of the existing six simplestyle keys, accept **static paint-layout key aliases** of the MapLibre v8 style spec: `fill-color`→fill, `line-color`→stroke, `line-width`→stroke-width, `circle-color`→marker-color, `circle-radius`→marker-radius (`fill-opacity` keeps its name). v8's expressions / zoom stops / source-layer / data-driven are **not adopted** (that belongs to the Visia Studio commercial surface + full-spec round). simplestyle primary keys take precedence, aliases as fallback.
- **Rationale**: off-the-shelf GIS style assets are mostly v8 format (MapTiler/OSM thisdir ecosystems); the alias mapping is ~30 LoC and takes the whole compatibility surface, lowering the SDK-embedding barrier; the full v8 expression engine costs 3-4 weeks and erodes the commercial layer (Open Core boundary), so we take the minimal-compatibility cross-section that is the "static alias subset".
- **Reference**: C1 four-option comparison (A keep / B alias subset / C full / D custom), Option B = docs/reference/maplibre.md live-benchmark ecosystem compatibility + three-js.md library-form precedent; D rejected (home-grown = zero ecosystem + hidden lock-in).
- **Impact**: geo::style gains an alias table (GEO-19); value formats reuse GEO-12 color parsing (#hex/rgb()); the style system's commercial surface (Visia Studio expressions/layer orchestration) is clearly lifted toward the undecided P4 neighborhood. The later 3b scalar→LUT builds on this style surface.

## D10: Web/wasm binding = route A (dual-surface single-source) (2026-09-11 J1-J3 landed, final)
- **Pre-registered content**: ① browser surface = the capi crate's `wasm` feature; js.rs and ffi.rs are two leaves sharing engine.rs (single crate, dual surfaces — not per-repo: lazier under full single-vendor control and immune to MapLibre-style "same name, different core"); ② thin-surface discipline: js.rs limited to marshaling/lifecycle/constant re-export/error translation; semantics reside uniquely in engine.rs; constants flow from Rust pub const single source into .d.ts and the C header (mirrors the CAPI-09 machine guard); ③ the no-free-functions discipline is kept (borrowed + caller-alloc two forms, SQLite/[MS] two schools both evidenced); ④ the emscripten route rejected (language-match law: C/C++ core→emscripten, Rust core→wasm-bindgen; three cases all conform); ⑤ u64≡BigInt, async factories (web GPU interaction has no synchronous path, wgpu's 30 Futures evidenced).
- **Finalization addendum (J1-J3)**: ① js-surface frozen list = class VisiaEngine (fromCanvas async factory / flat input / pick→bigint) + 11 constant getters + abiVersion static; the .d.ts is the lock file (guarded by web-mirror.mjs compile-surface assertions); ② **revision surface (rejected routes recorded)**: wasm-pack not dead but D5 uses its CLI directly without a package; the `visiaengine_input_struct_size()` export rejected (the 14-symbol surface); playwright CI automation belongs to M2 (timebox abandonment on this machine, logged); ③ size record **raw 733KB/gz 294KB** (the whole wgpu web family; the web half of the ≤10MB target hits the bar early); ④ final isolation shape: capi (native C ABI) + visiaengine-wasm (bindgen), two crates, **single source = Engine**, CAPI-09 machine reconciliation. Evidence chain docs/reference/ffi-patterns.md [FFI-R:BS-5/6/7, v13-FEAS-2/7].


**D8 trigger-① re-evaluation (2026-09-14, Qt round Q3 real-window evidence)**: the frame_requested frame-skip negotiation port — the QTimer 60Hz pull model hit a no-frame-rate pain over 6 frames on V1's real window; **maintain suspension**; next trigger = video/decoded-frame-aligned hosts (revisit only when the pull model can't follow an external clock).

## D11: Qt-round demo build integration = thin self-managed CMake (Option B) (2026-09-14)

**Decision**: the host consumption surface of the Qt widget round uses the iceoryx2-shaped thin self-managed CMake (~60 lines): `add_custom_target` cargo step + `RUST_BUILD_ARTIFACT_PATH` escape hatch + INTERFACE umbrella `visiaengine::capi` (BUILD/INSTALL dual genex) + in-tree fake-Config trick (find_package consumption code identical in-tree/post-install); the `visiaengineConfig.cmake.in` template is written but install tree/.pc/SOVERSION are not done. Option A (g++ script) and Option C (straight corrosion) dropped.
**Rationale**: the demo must be consumed by a real host via cmake — write once now vs rewrite+revalidate twice at the packaging round; the find_package smoke is naturally carried by the demo (the predecessor's headline lesson: the cmake package was never compiled). Basis = team research, four-option comparison (docs/reference/cmake-integration-patterns.md, iceoryx2 :153-205/:38-78/install.cmake:17-21, slint :16-26 and its corrosion pitfalls :427-466, evidenced).
**Impact**: Qt round +1 file-level increment; capi Cargo.toml gains `links = "visiaengine_c"` in passing (duplicate static-link foolproof).
**Landing continuation (same day)**: Option B's landing surface = the CMake engineering layer (.omo/plans/visiaengine-cmake-project.md C1-C3 landed; the Qt round v1.3 onward is a pure consumer); the links account is not opened (cargo hard-validates build-script absence) and is still covered by the gate-abi nm set.
**Re-evaluation trigger (packaging-round agenda)**: convert to corrosion or keep thin self-managed — trigger = a second C++ consumer / Windows matrix / real install-tree need; corrosion's promotion requires first opening the D5 exception account "conda has no package → cargo-binstall/GitHub-mirror vendor" (FetchContent network at configure time conflicts with the offline discipline). The umbrella interface `visiaengine::capi` is the stable contract (T-H2 2026-10-12: renamed `visiaengine::c` per the iceoryx2-c/zenoh-c consumer-language corpus; the contract class is unchanged) — swapping the kernel under it changes the consumption surface zero.

## D12: CMake example-entry engineering = stub launcher + contract table + ctest single-source (2026-09-15 landed, final)
- **Background**: the 11 E-series examples are opaque to CMake; IDEs have no run entry; two contracts are badly broken — CWD data paths (7 items) and DISPLAY (5 items).
- **Decision**: A = launcher stub (~40 lines of single C++ source) carrying the real IDE Run body: chdir to repo root + display-family DISPLAY precheck exit 77 + argv passthrough; registration = contract table (glob⇄table dual validation, missing item = hard configure error); ctest example_* and the targets share one source. B (custom targets + ctest only) dropped — CLion has no Run button.
- **Rationale**: CLion Run only recognizes regular executable targets (imported/custom produce no run config); Easy3D evidence "name = directory = title" needs zero maintenance (probe A file:line); 77 = the ctest SKIP native convention (SKIP_RETURN_CODE needs ≥3.24). G8 revision: the manual-cc route in gate-abi/smoke-x11 = a pure-C-consumer compilability gate property, not a redundant parallel — kept.
- **Impact**: new-example duty in three places (source file on disk / contract-table registration / tutorials.md index); the examples family needs CMake ≥3.24; the third lock surface extends to the cmake entry (configure is the gate; no separate gate-docs⑥).

## D13: ABI attribute-read trio 14→17 (adjudication chain closed, 2026-09-15)
- **Background**: the host-facing attribute query was missing (capi had 14 entry points, zero read ports, probe-proven); the gap table recommended an "AttrSet attribute example".
- **Decision**: user direction approved (the issue was presented with "the 15th port" wording) → the plan explicitly laid out the capacity delta → C-1 final = **3 typed ports 14→17**; C-2 str has no probe-length (cap insufficient = -5, zero partial write, 256B doc convention); C-3 the E701 appended section does not open a new E number; the standalone point-cloud example is **skipped** (overlaps E202 with no dedicated data source).
- **Rationale**: the three-type mirror closes CORE-12; the host learns it all at once; a single variant-type port is ugly on the C side (type double-entendre); key = enc_entity pose pass-through (zero decode, ABA-immune = generation inside the key).
- **Impact**: D6's "14 entry points" lock formally rewritten (CAPI-01/02 clause revision sentences + the twelve-item dependent-surface checklist already cleared); the precedent for future port expansion = **lay the capacity delta out before asking for approval**; the wasm surface keeps mirroring constants (functions are not mirrored, status quo); the `attr_of` kept-surface reserves an isomorphic shape for batch 7's js attribute port.

## D14: Root examples directory law + unified c+ example surface + stub system retired (supersedes D12, 2026-09-16 v1.3 landed)
- **Background**: the user judged the CMake engineering messy (three complaints: directories/targets/examples); two red-team rounds with 27+10 evidenced items: the stub launcher = the root of "the examples are fake", and corrosion's IDE import of examples has zero mainstream precedent (slint/iceoryx2 keep examples in the cargo world) while carrying its own eight taxes (cargo-build_* fake names / always-dirty #624 / dual caches / conda first-crab / no CTest #13 open since 2018).
- **Decision**: ① launcher + 189-line contract table + old ctest example_* all abolished; ② root `examples/{rs,c,cpp,qt}/` layered by language; Rust examples keep **cargo example semantics** (explicit [[example]] blocks, not bin-ified — the v1.1 deadlock that bins can't eat dev-deps dissolves with it); ③ Rust examples enter the IDE via **c+**: self-written `cargo-build_*/cargo-run_*` steps (R10 adopted corrosion's naming grammar, zero dependency) + the unified ctest `example_<stem>` list; the nine pixi smoke entries demoted to forwarding shells (argv single source = R9 registry); ④ the corrosion question is parked at the packaging round (its adoption day = the bulk retirement day of the self-written steps, same-name tripwire).
- **Rationale**: mainstream shapes + a measured tax bill; the c+ experience is 90% at a cost of 19 lines, while the b-route's signboard self-discounts by its price list. The item-by-item adjudication ran entirely under user eyes (including four "speak plainly" rounds).
- **Impact**: cmake-smoke tri-state tri-anchor (bare-PATH configure probe + dual build anchors — B1 "don't let the guard get pruned away too"); D11's re-evaluation refreshed in sync (conda has a corrosion package = the original "no package" criterion was wrong; this round still drops it for the tax table); RA Run demoted to an auxiliary track.

## D15: bindings as one nest + SDL3 window host + E8xx/twin numbering law (2026-09-16 v1.3 landed)
- **Background**: the MediaServo shape was adopted with the ruling "everything in one nest"; window examples need a cross-platform host piece; the new E8xx band needed a numbering law.
- **Decision**: ① `crates/` = the pure core, 5 libraries; capi → `bindings/c/visiaengine-capi` (src + handwritten header + tests cohabiting, probe at the same level), wasm → `bindings/js/rust/` (measured path references = 0; members + one path line); `bindings/cpp` (hpp) / `bindings/qt` (widget.hpp) / `bindings/js/demo`; the directory law in one sentence: examples/ = teaching examples, bindings/ = implementation + headers + acceptance + packaging shell. ② Host trinity: headless main force (readback assertions) + **SDL3** (conda 3.4.16, xid property = the attach contract's isomorph, S0 dual-state evidenced) + a dedicated Qt demo band; no FetchContent fallback (D5 single source). ③ C/C++ SDK examples = **the E8xx band**; twin shared-number `.c/.cpp` separated (target names suffixed `_c/_cpp`); asset escape hatch = directory name = file stem. ④ hpp opens no SDD account (widget.hpp precedent; the three shells' semantics live in header comments, the living anchor is CAPI-nn; trigger = when shell growth carries new behavior, open cpp.md + extend spec-trace's dual regex, 30 minutes, no interest). ⑤ install tree = S-c β: this round only a fail-loud guard; tree + out-of-tree rehearsal = hard debt of the packaging round (template nailed at §7).
- **Rationale**: six mainstream repos measured (iceoryx2/slint/SDL3/Vulkan-Hpp/Sascha/filament) + the red-team G-group gaps all absorbed; examples centralization = 6/6 mainstream agreement; the dichotomy "tests co-locate with bindings, examples centralize".
- **Impact**: example gate = a ctest-enumerable single source; the toolchain batch (compile_commands/.clang-format/.vscode presets always + .gitignore L56 negation deadlock fixed); PIT-19 (pixi clobber → conda Xvfb XKB death) booked.

## D16: Point-cloud band scope ruling (2026-09-17, hyperplan three adversarial rounds + user "follow the recommendation")
- **Scope**: M0 ramp exposure + M1 add_points pass-through + M2 io-points(PLY ascii/bin_le) in one band; **LAS = second band** (criteria: AV/surveying clients' lead demo list, no evidence on this machine; the las crate 0.11.1 MIT verified, may enter); the whole picking family deferred (trigger = interaction demand ∧ bench >16ms); normals/simplification/EDL/per-point queries all non-goals.
- **Naming**: new prefix `IO-*` (io-* family law = naming follows the crate), spec-trace's **third-check machine lock** in the same commit (a prefix outside the whitelist = red; positive/negative self-proofs on record) — reducing "missed prefix, both regex sides silently blind" to a gating problem.
- **Data model**: cloud = single entity, single DrawPoints; origin = f64 bbox center + local f32 (isomorphic to mount_geo, D7 adds zero new math); cloud-level meta via attr_* prefixed queries (zero new query ports); per-point attributes/query ports defer with the picking family.
- **Capacity**: cap = 4M (declaration rejects first, no half-acceptance); release measured 1M = upload 127ms / frame 914ms (lavapipe, bench_pcl json on record); the number-change window = one (constant + clause + test, three places in one commit).
- **Process ledger**: value wrung out by the red team — "S-tier pure-CPU preshading = not deliverable" merged into M; "ray_aabb reuse of cloud-level pick" was beaten down by four lanes and deferred; bench.sh's always-true green (PIT-25) was incidentally root-fixed by G3.

## D17: Cross-section clipping band, eight adjudication points (2026-09-17, B2 planning-round probe adversarials + user "full band per recommendation")
- **a route**: fs discard single path — wgpu 30's hardware CLIP_DISTANCES verified to exist (docs.rs) but
  the DX12 backend is not in the list + feature-gate needs dual-path branching; re-evaluation trigger = discard edge quality genuinely hurts (customer jaggedness complaints).
- **b face count**: ≤4 AND (single-face / corner-cut / three-axis box tiers); six-face box / OR-union (three.js's two-group shape) = non-goal;
  raising to 6 touches only MAX_PLANES in one place (fixed-layout Pod lock).
- **c caps**: not done (hollow interiors after cutting are honestly documented); stencil cap trigger = BIM interior demo list.
- **d precision**: world coefficients f64 (n points to the kept side + on-plane = kept closed interval), shader uses the model-space identity
  `n_m=Mᵀn, d_m=d+n·(o+t_M)` (multiply in f64, then drop to f32) — two self-caught root fixes during implementation (Mᵀ / translation column);
  lesson: under column-major order the "normal times matrix" direction must be derived via the identity, never by intuition.
- **e clipped family**: every drawing family (mesh/inst/stroke/point, per-pixel) — "clip the world, not the types".
- **f shadows**: casters discard identically (a cut-away building leaves no whole shadow; three.js's clipShadows defaulting false
  is a legacy habit, not a reason).
- **g picking**: hit points on the negative side of keeps → excluded, retry loop (what is visible after cutting must be pickable); capi pick = positions ×
  IDENTITY is existing fact, noted as "two frames merged"; not incidentally widened into the origin domain (independent gap, separate entry).
- **h surfaces**: E813 C single example, headless living gate; interactive clip sliders booked (E901 key control = triggered by demo demand).
- **Bonus record**: the View-block extension scheme (origin + transform already in the view_block signature) made this band
  **cost zero new bindings/bindings-groups/buffers** — a full construction layer cheaper than the probe's independent binding9 estimate;
  the "check the data path before choosing the mount point" discipline (think-before-act), evidenced.

## D18: S2 text surface band, eight adjudication points (2026-09-18, probes A/B adversarial + user "tier ① engine self-managed · full band")
- **a tier/scheme**: tier ① minimum-usable + scheme I (engine self-managed fontdue rasterization) — the user chose explicitly among three;
  not host-self-managed (SDK value halved), not the standard tier (swash touches R2). The MSDF route was rejected (Rust ecosystem ≈ 0, msdf-font 1★).
- **b dependency lock shape**: fontdue `default-features=false, features=["hashbrown"]` —
  disables SIMD (wasm traps #25/#72) but keeps hashbrown (under no_std, std/hashbrown is either-or [docs measured;
  the probe report missed it; N0 smoke corrected it]); ttf-parser RUSTSEC-2026-0192 deny-ignore
  (unmaintained, not a vulnerability; attack surface = host-injected own assets + panic fence; ab_glyph depends identically = switching is a no-op;
  three re-evaluation triggers written into the deny comments).
- **c new crate location**: `crates/visiaengine-io-text` (the 9th crate, pure CPU: face/cache/layout);
  GPU state lives in render-wgpu; interface isolation keeps a swash upgrade non-breaking.
- **d namespace**: reuse the IO prefix (IO-07..09, already in the whitelist = the third check unchanged), not a new TEXT-.
- **e label coordinates**: world-anchored + constant screen size (the ortho exact path's assert_eq locks pixel-equal across zoom);
  color = linear-domain pass-through (sRGB chokepoint label table = core::srgb_to_linear, chokepoint six); center alignment lands in the dx domain
  (PIT-28: pen is px and cannot enter the anchor's world coordinates). Always-on-top: depth write=false/Always + command order last.
- **f atlas**: R8 single slot 512² + shelf packing + 1px gutter (sampling-bleed defense); when full, full-clear rebake with transparency degradation
  ([ponytail] per-glyph LRU pending profile); set_glyph_atlas is a wholesale replace (io-text dirty-driven).
- **g clipping**: labels go through WGPU-21's clipped() same predicate (anchor in model-space, REND-33 table domain);
  casters don't project (no projected surface → don't mount ②).
- **h dual-port mirror**: CAPI-21 load_font + CAPI-22 add_label (26→28, abi minor = 7 not 8);
  the three-surface mirror covers the full band (header POD direct use / hpp thin forwarding / wasm loadFont+addLabel→bigint+MISS);
  the management domain is outside items (same catalog as add_points, out of entity_count/remove).
- **R2 size war result**: wasm gz 429401→490967 = **+14.3% (passes inside the 15% threshold, zero margin, honestly recorded)**;
  the probe's +6% estimate was optimistic (reality = font stack + shader + label chain). Re-evaluation triggers: real wasm production, or
  re-judge if CJK embedding is ever needed (fallback = feature-gate the font rasterizer to native only, same shape as R2's texture fallback).

## D19: ⑤a camera-flight band ruling (2026-09-18, dual-probe adversarial + Momus OKAY 0B/3A + user "full band")
- **Scope knife**: this band = flyTo pure flight; **multi-viewport split-screen = its own band** (four-pit list into non-goals: the depth_cache singleton upgraded to Map /
  scissor unproved / input px routing / dual px_world_scale — both probes agreed; mixing bands = drags the verification cadence down). WGPU-26 reserved.
- **Layering settled (forced = the right answer)**: the render crate sees zero std::time — the pure function fly_sample(from,to,t,easing) lives in render;
  the wall-clock advancer lives in engine. **The wasm runtime landmine is actively defused**: std::time::Instant is unavailable on wasm32-unknown-unknown
  (flyTo would trip the capi_guard -4 fake-death) → a now_ms() shim in two forms (native = Instant process anchor / wasm = js_sys::Date::now)
  — the web is the main stage; compiling ≠ running; written before stepping on it.
- **Easing**: Easing{Linear,CubicInOut(smoothstep),CubicOut}; the capi surface fixes CubicInOut (adding a parameter = YAGNI).
- **Interruption = MapLibre A-school**: pointer/wheel (kind 1/2/4) cancel on the first line; keys (5) do not interrupt (test counter-evidence on record).
- **done/idle merged single source**: fly_state returning 1 = not flying (after cancel = 1, can fly again); out_t01 is written only while flying,
  done writes zero (B1 zero-partial-write catalog); wasm splits flyState+flyProgress(done=1.0) to spare JS double-value encoding.
- **near/far do not fly** (pose carries no depth domain = always current values, PIT-5/REND-14 dependent [Momus-A1]);
  **rerouted from = current sample** to preserve continuity [A3]; the yaw shortest arc lives in the fly_sample construction piece — **mix_rig untouched**
  = REND-15/16's existing assertions zero-regress (wrap comparison disproven via the linear midpoint, doubly locked).
- dur_ms = 0 = teleport form (not an error value domain); altitude arcs / Van Wijk = triggered by an AV fly-through ticket.

## D20: ⑤b multi-viewport band ruling (2026-09-18, dual probes + Momus OKAY 0B/2A → fixed in + user "full band")
- **Scope knife**: this band = split-screen pipeline + living example (pure rs); capi multi-viewport ports / click-to-fly = wave two (CAPI-25 reserved);
  the shared viewer library = its own entry (four examples E501/E901/E302/E303 isomorphic, the ≥3 threshold has broken, the account transfers to D21 candidates).
- **clear semantics settled on this machine** (current lock at the adjudication gate): wgpu30 LoadOp::Clear covers **the whole attachment**
  (the AllClear form's later region swallows the earlier region, redA=0; consistent with the Vulkan render-area conclusion) ⇒ **first pass Clear
  (full-extent base color = gutter color, zero new mechanism) + subsequent passes Load** is the only safe shape; the AllClear enum slot = probes/teaching.
- **Depth must clear every pass** (the live landmine E303 currently caught, overwriting the probe's expectation): Loading depth between passes takes the main view's
  opaque ground depth and swallows the small window's whole-area occlusion (nonsky_map=0, live); re-clearing inside the scissor fence has
  zero side effects and no depth consumer between passes ⇒ Clear+Discard every pass (saving Store bandwidth).
- **rect mount point = does not touch Frame** (ViewportRect is a standalone type passed as a render-call parameter; the 44th fallback wave is exempt);
  full-rect emits no set_* = the old path's bit-for-bit canary lock; FOV/px are both per-Frame independent (two projections compose autonomously).
- **casters do one shared-map trip** (light-space is independent of the main camera); small window = top-right square (a long strip = a composition mistake, measured).
- **fov unit landmine eradicated** (a real bug in the stock): perspective consumes radians (look_at defaults π/3); E501/E302
  once passed 46/55/60 as degrees = projection distortion (E302's window bright 96→1987 proves it was once broken); the orbit doc nails the unit;
  lesson = a window surface the pixel gate never looked at is an unverified surface (this band's volume of the T3 human-check + headless-assertion double insurance).

## D21: ④ translucency band ruling (2026-09-18, dual probes + Momus OKAY 0B/3A fixed in + user "full band")
- **Three-zero discipline**: zero new ports (alpha already rides the add_mesh four-tuple path; the hot-change port set_material_alpha is booked with triggers) /
  zero dependencies / zero IR fields (transparency decision = material alpha<1, sorted on the backend side table, WGPU-28 orchestration).
- **V0 blend domain settled**: this machine's wgpu30 = linear domain (checkerboard gate 151 vs encode-domain prediction 83, decided on a single value,
  consistent with the Vulkan spec; the prediction's arithmetic first version missed the Lambert shade coefficient — PIT-8 "predictions carry all chain factors" extended note).
- **Translucent pipeline = Variant value-extension trio** (FlatT/TexturedT/InstancedT; bgl/layout identical, only the pipeline state differs;
  lazily created, never prebuilt); blend = the SRC_ALPHA standard form (same equation as Labels), depthWrite=false, compare stays Less.
- **Two-pass orchestration**: no translucency and no labels = the old single path untouched (golden fidelity preserved by construction); pass2 = painter's on view depth descending
  (key origin + M translation column, f64 per D7 law; same-key submission order stable) + Labels always-on-top at the end; **pass2 depth Load**
  inherits the opaque occlusion (distinct from ⑤b's across-viewport clear-depth shape — the two Load/Clear semantics each in their rightful domain).
- **E504 bidirectional semantic lock**: the water film "attenuates contrast" rather than "destroys contrast" (a=1.0 control 52 / a=0.5 measured 15,
  assertion band (10,30)) — a decidable form for translucency correctness.
- **v0 residual declarations into clauses**: single-mesh self-overlap cannot be ordered (RTT/OIT in a later band) · translucent pieces cast fully solid shadows (three.js chose the same)
  · material a==1 ∧ texel a<1 goes the old road (no dark semantics) · per-instance alpha needs its own layout ruling.

## D22: ⑤b wave-two minimap navigation band ruling (2026-09-18, dual probes + Momus OKAY 0B/2A → fixed in + user approval)
- **Option C: main + fixed secondary viewport + engine-internal hit-test routing** (A = host dispatch / B = VeInput field additions, both judged out).
  Probe B nailed `struct_size`'s strict `<` forward-compat gate = adding a field to VeInput breaks every old host —
  hosts report only screen px; inside/outside determination and dual-projection routing live in the engine (VeInput untouched).
- **MapView = proportional-rect form** (fx/fy/fw/fh∈[0,1]+zoom); resize auto-follows, zero host duty;
  the minimap rig = a top-view follower derived from the main target (zero sync port; two-rig drift radius = construct 0).
- **[ruling e] navigate_click swaps only the target, preserving dist/fov/yaw/pitch**: a minimap click = a pan intent,
  not a re-composition (E816 f64 bit-exact assertion lock). Two-stage hit = entities first (pick_geo top view) →
  ground z=0 fallback (REND-37 ray_ground); outside the region / no map = reject (no silent rerouting, C15 value-domain table first).
- **Routing order: minimap region first** (visual top = input top, same reasoning as ImGui focus); kind1..4
  pointer/wheel inside the minimap region = consumed no-ops (v0 has no map-drag semantics; navigation goes through the explicit CAPI-26 port).
- **canary construction guarantee**: map=None ⇒ the old single-frame path byte-for-byte (E816 leg 1 + leg 5 dual-form evidence; leg 5's lesson
  = baselines must be re-captured after flight; post-write self-check, one root fix).
- Ledger: 152 clauses · 33 entry points minor=9 · ctest 31 (E816 five-segment living gate measured 1596g/756r/target
  (3.579,3.714,0)) · wasm bridge 4 names (setMap/clearMap/navigateClick/getCameraPose;
  clear=NULL of the three C ports splits into two web laws) · R2 zero new dependencies.

## D23: Install-tree band (packaging-round v1.3) — user-adjudicated one by one (2026-10-08)

**Context**: `.omo/plans/packaging-round.md` v1.2 (2026-09-20, Momus x2) planned
install+npm+pip+corrosion as one band. User cut it to install-tree-only, then
walked the open points with the adjudication-walkthrough protocol (one card per
message). Momus re-reviewed the resulting v1.3 section and returned OKAY.

**Decisions (all user-adopted, letter = the card's recommended option unless noted)**

| Card | Question | Result |
|------|----------|--------|
| 1 | what the tree contains | **A+**: cdylib + three public headers + licenses + package config. **No static archive** (release `.a` measures 67 MB vs the 7.25 MB cdylib, zero in-repo consumers, and shipping it implies owning the system-library list). Upgrade cost when a consumer appears: ~10 lines + a list derivable by linking once. |
| 2 | static-lib refusal challenged by user | measured rather than argued: linking a C consumer against the `.a` with only the cdylib's own `NEEDED` set (`-ldl -lpthread -lm -lgcc_s`) **works on Linux** -- so "no hidden autolink table" was withdrawn; the decision survived on the two remaining legs (size + zero consumers). |
| 3 | SONAME | **A**: emit it from `bindings/c/visiaengine-capi/build.rs` (`cargo:rustc-cdylib-link-arg`), the wasmer shape. Voids v1.2's `IMPORTED_NO_SONAME TRUE`. First `build.rs` in this repo; wasm32 emits nothing; mac/win are declared gaps, no silent `else`. |
| 4 | package config form | **A**: `configure_package_config_file` template, imported targets created consumer-side from `PACKAGE_PREFIX_DIR`. `install(EXPORT)` is unusable here (measured CMake errors), and `_IMPORT_PREFIX` does not exist in a hand-written config (measured: expands empty -> `/lib/...`). |
| 5 | Qt widget header | **A**: ship it, declare nothing. Probed: with only our include dirs it fails at `visiaengine_widget.hpp:7` on `QtGlobal`, and the default env has no Qt headers -- so the tree ships the file and the consumer brings Qt/X11 (what `examples/qt` already does). |
| 6 | README drift (40 vs measured 43) | **C**: fix the numbers **and** lock both halves -- gallery card count in `gate-docs` check 7, ctest registry count in `cmake-smoke` (each lives in the tool that already holds the true value). Both locks seen red by probe before shipping. |
| 7 | P3 execution order | **A**: commit the finished small thing (skill port) separately, then review, then code. |
| 8 | where the record lives | **B**: one `D23` entry written at the approval point (the plan file itself is gitignored, so the decision must be in the committed ledger to survive a fresh clone). |
| 9 | P3 scope after new measurement | **A** (user chose against my written recommendation of patchelf-in-band): RUNPATH cleaning moved to the publish band's assembly step. Root cause traced: `.pixi/.../pkgconfig/tinfo.pc` in this env still names a *sibling* checkout prefix (env is a relocated copy), and a `-sys` build script feeds its `Libs:` line to the linker; any machine bakes its own env path, and install(CODE) cleaning would push patchelf onto every installer. |

**Consequences**
* Promised now: `find_package(visiaengine)` / `pkg-config` consume an installed
  tree; consumers record the bare `libvisiaengine.so`; the tree survives a move
  (probed by reinstall-and-rebuild).
* Gates added: `gate-abi` SONAME assertion (expected name derived live from
  `[lib] name`), `cmake-smoke` positive state 4 (install rc=0 + 9-item manifest +
  `.pc` text checks -- the old "install must fail" assertion inverted),
  `gate-pack.sh`/`pack-check` (manifest, `NEEDED` subset of system libs, no
  build-machine paths). `ci` stays eleven segments: `pack-check` is independent.
* Declared gaps (not silent): staticlib, npm/pip, win/mac install forms,
  publish-time RUNPATH cleaning, relocatable `.pc` beyond pcfiledir form.
* Incidental root-cause found by this band's wasm channel run: the V2.2 bridge
  `get_node_transform(&mut Vec<f64>)` never compiled for wasm32; `ci` carries no
  web segment and `web-mirror.mjs` had no freshness check, hence PIT-44 and the
  staleness guard.

## D24: Shared-code value counts in **sites**, not in lines; triggers get archived by measured coverage (2026-10-08, band V4)
- **Ruling**: after V4 landed `orbit_drag`/`zoom_dist`, the net line change was **zero** (example side 4×(−9), viewer.rs +36).
  Its value is the feel constants (0.006 sensitivity / 0.0012 wheel coefficient / LineDelta×40) and the "seed on first motion only" sentinel protocol
  converging from 4 sites to 1 — while previously **no mechanism at all** kept those 4 copies in sync.
- **Ruled together**: whether a trigger holds is decided by measurement, **overriding the archived step list in the plan**. The plan originally said "step 1 wires in
  E510/E511/E206-type representatives"; the field fingerprint + verbatim comparison proved those are exactly the semantically **different** family (E510 wheel = phase;
  E508/E509/E510/E511/E206 carry their own spherical solving; E402/E403's drag arm includes hover-pick/box-select course text; E306 pan + right-click tilt;
  E303 per-viewport follow). What is truly byte-identical is E201/E501/E504/E901 (E201's `.take()` and the copy are behavior-equivalent on a
  `Copy` Option). Hence in the end only two free functions were shared; no `Orbit` type was built.
- **Reusable criterion**: if a constant's "differs per scenario" comes from **scene scale** (clamp bounds) → parameterize; if it comes from
  **feel/protocol** (sensitivity, sentinel protocol) → the constant belongs in the shared piece; keeping it a parameter is forbidden (a parameter nobody varies = decoration).
- **Impact**: later shared-code proposals are evaluated as "site count × presence of a sync mechanism"; the line-count yardstick means something only at genuine
  convergence (V3's −1,376). This entry pairs with C18: C18 governs **what must not be shared**, D24 governs **whether sharing pays**.

## D25: The queue trinity — guardrails / capability / publish (2026-10-09)

- **Background**: the next-band-queue's six cards were adjudicated one by one (adjudication-walkthrough protocol, C1 four columns per card).
  The trinity = **guardrails** (automated checks over existing surfaces) / **capability** (new language or new channel surfaces) / **publish** (distribution and artifacts).
  Their questions differ: guardrails ask "can the predicate go red", capability asks "is there a real consumer", publish asks "can it be evidenced on this machine".
- **Rulings (six cards, one sentence each)**:
  1. keys-probe permanence = **A** (into the harness keeping `--all`/`--strict`, but **not into** `pixi run ci`;
     ci.yml already calls it separately with `--strict`).
  2. npm = **A**; pip is **not packaging but a new capability surface** — the measured `bindings/` holds only the c / cpp / js / qt four forms,
     no py, nothing to package.
  3. site = **A, don't do**: the whitepaper measurement contains no site promise (grep zero hits); mdbook keeps its batch-5 rejection verdict.
  4. census first + predicate first = **A**: predicates go into the document **before** any count. The root cause of this session's six counting-drifts was each time
     "count first, define later".
  5. publish band = **C**: only artifacts evidenceable on this machine (CPack tarball); the cross-platform matrix is parked per
     `.agents/rules/common/constraints.md` (cross-compilation belongs to Docker/vendor toolchains, not this band).
  6. C-surface shared bootstrap = **A, close**: measured, the `visiaengine_create_headless` bootstrap shape is **15 lines / 13 files**
     (predicate `grep -rn 'visiaengine_create[a-z_]*\('`; two rounds each wrong once — 14 was a C-only line count, 15 initially an occurrence count that happened to match;
     E817 re-mounted creates twice more, so 3 lines sit in one file); the drift is in **wording, not behavior**; `template.c`/`template.cpp`
     are already in the repo; those three host-call lines are exactly the **course text** of these examples (C18: shared pieces must not eat claims).
- **Reusable criterion (extending D24)**: the value of shared-code and guardrail work = **site count × absence of a sync mechanism**, never counted in lines.
  Card 6 is the positive case: sites are plenty (15 lines), but the sync mechanism **is present** (handwritten header + template skeletons + the example surface itself),
  so net value is zero — close it. Card 2's pip is the mirror case: site count **zero**, so it isn't even packaging debt.
- **Ledger frozen**: 199 clauses / 49 C ports / 44 examples (= ctest 44 = gallery 44 cards) / 12 packages / 12 ci segments.
  Everything **unchanged** except the last. The only count this band moved is **ci segments 11→12** (pack-check joined the chain, deliberately);
  both prose counters (`docs/gap-closure-roadmap.md`'s Gates row and `.agents/memorys/status.md`'s
  ledger row) were updated in `10c7dc7` **in the same commit** — this is the repo's only count with no machine lock guarding it, so the discipline = whoever moves it
  updates the wording in that very commit.

## D26: gate-web browser channel = chromium CLI + anchor-beacon; five alternatives rejected on measurement (2026-10-16)

The headless page gate observes the wall through the bare chromium CLI (no driver layer).
Rejected, each with its evidence: **Lightpanda** (no rendering surface: no canvas/WebGPU/compositor -- the gate's load-bearing legs are exactly what it omits); **playwright as driver** (extra npm remote + self-managed cache outside the lock; kept ONLY as a download channel after conda-forge proved to have no chromium package -- the binary was fetched from the npmmirror playwright CDN and invoked as a CLI); **CDP observer** (python/node stdlib have no websocket client: ~80-line hand-rolled frame protocol or a new dependency -- against the no-new-gadgets ruling); **newer-chromium bet** (unproven); **scope-reduction v1** (legs 1+2 only would stay GREEN while the engine hangs silently -- a never-reds-when-broken shape is decoration per C0). Adopted: WebGPU combo `--no-sandbox --enable-unsafe-webgpu --use-angle=swiftshader --enable-unsafe-swiftshader --enable-features=Vulkan --use-vulkan=swiftshader` (measured 2/2 on Chromium 131; the Vulkan pair is required on Linux, sandbox starves the GPU process in this container); heartbeat observation = parse-time anchor `<img src="/ve-gate-ready">` whose response the gate's own stdlib server holds until the post-8-frame beacon arrives, so the load event (and the dump) lands after boot. Binary at ~/.cache/ms-playwright/chromium-1148/chrome-linux/chrome (cache layout reused). Sources: this repository's spike + gate rounds 2026-10-16 (plan .omo/plans/2026-10-16-headless-page-gate.md).

Revision (same day): the lock-side half-fix CLOSED as an in-gate pin -- conda-forge carries no chromium (api.anaconda.org/package/conda-forge/chromium returns empty JSON; pixi solve no-candidates; both measured), so no feature+lock path can exist. The gate holds VE_CHROME_VERSION/URL/SHA256 as the single-source pin with an exact-match --version drift check (hard FAIL; the live `--version` trailing-space catch is od-verified in-file) and opt-in VE_GATE_PROVISION=1 sha256-verified provisioning (never auto-download; absent => the honest SKIP stands). Commits 59df128 (pin) + aec496d (heartbeat rolled out to all 6 live pages, per-page anchor/beacon namespaces; per-page fault isolation proven: E401 beacon cut reddens ONLY E401's leg).

Addendum (2026-10-16, serve-https lane, c7fdeb4): LAN remote WebGPU is a secure-context lock, not a GPU lock (WebGPU requires secure contexts; navigator.gpu absent on http non-loopback origins). Host-side TLS lane added beside the gate: scripts/wasm-serve-https{,-gen} (stdlib ssl; mkcert 1.4.4 via pixi global; leaf SANs = localhost/127.0.0.1/both measured LAN IPs). Measured ruling data: https with an UNtrusted cert counts as a secure context on chrome 131 (wall 43/43 + E101 beacon green over https) -- so remote browsers work with zero client setup past the cert warning, and rootCA install is the no-warning nicety. flag-lane stays Chromium-only, tunnel-lane per-machine: TLS is the cross-browser root cure.

## D27: project skills have one source; host activation layers are generated links, enforced by `gate-skills` (2026-10-28)

`.agents/skills/` is the only copy. Host layers (`.codebuddy/skills/`, with `.claude/skills/` pre-declared as disabled) are **generated** by `scripts/skills-links.py sync` and checked by `scripts/skills-links.py check`, which runs in the ci chain as `gate-skills`. Reasons, in the order they were measured: hand-built links had silently disappeared while the docs still claimed them (the plan's first draft asserted a link layer that `ls` proved absent) -- a hand-made artifact with no gate is indistinguishable from a working one; and CodeBuddy's discovery of `.codebuddy/skills/` is undocumented, so the layer must be cheap to rebuild and cheap to verify. Shape: `check` separates `missing` / `dangling` / `wrong-target` (absolute, cross-name) / `exists` (real dir squatting the name) / `extra-link` / `extra-dir` -- a coarser set would let a stale or misdirected link pass as green. Guard: if the target directory is itself a symlink or resolves inside `.agents/skills`, the harness exits **2** and writes nothing, because in that state the backup/prune steps would rename or delete the single source (reachable via one `ln -s ../../.agents/skills .codebuddy/skills`). Windows: `os.symlink(..., target_is_directory=True)` and only `winerror 1314` triggers the `--copy` fallback (hashed link-map); junction was rejected because `mklink /J` stores an absolute target, which the same classifier calls `wrong-target`. Plan: `.omo/plans/2026-10-28-agent-skills-single-source.md`.

## D28: Agent config unification = "single source `.agents/` + read-through-first + one rules translation + CodeBuddy/WorkBuddy first then Qoder/Trae/ZCode + memory aggregation into AGENTS.md + subagent slot reserved" (2026-10-28)

Cross-host adaptation architecture adjudication, adjudicated one by one via the adjudication-walkthrough protocol: (1) mapping strategy = read-through-first (hosts that can read `.agents/` do so directly; only those that cannot get a generated layer); (2) rules = single-source md + frontmatter, with the generator translating the dialect; (3) rollout order = CodeBuddy/WorkBuddy first (to harden the generation pattern, reusing this round's `.codebuddy/skills`), then Qoder/Trae/ZCode (cheap read-through); (4) memory = concise aggregation into root `AGENTS.md`, with deep records kept as on-demand references; (5) the subagent layer is left as a reserved slot this round (no project agents in this repo; opencode only recognizes global plugins). Full plan: `.omo/plans/2026-10-28-agent-config-unification.md`. Discipline reuses the skills layer: write the single source once, host directories are generated artifacts, `gate-agents` prevents drift, and the guard exits 2 when the target resolves back into `.agents/`.

## D29: Serena deferred from single-source MCP — this environment lacks clangd and an installer (2026-10-28)

Feasibility probe of running Serena under opencode: the environment probe shows `clangd` is missing (apt candidate 1:10.0, needs system-level install + approval), `rust-analyzer` is present, and `uv`/`uvx`/`npx`/`pip3` are all absent from PATH (the pixi default env also has no pip/uv — only `~/.pixi/bin/pixi` and system `python3`). Conclusion: Serena's **C++ side (this engine's primary language) cannot work**, and even with an installer added, Serena still needs a resident clangd + rust-analyzer dual daemon, at non-trivial cost. Contrast with `local-codegraph`, already configured, tree-sitter based, no LSP daemon, read-only, lightweight — confirming "codegraph first, Serena on demand". Serena is deferred until clangd is formally managed (clangd is worth installing for the repo anyway, but that is a separate infrastructure decision needing approval). Serena was not written into `.agents/mcp.json`.

## D30: All MCP local stdio, commands unified as `pixi run mcp-<name>`, needs a bootstrap mechanism (2026-10-28)

MCP-layer resolution (discussion + environment probe): (1) all MCP servers move to local stdio; in canonical `.agents/mcp.json` every command is `["pixi","run","mcp-<name>"]` — local probe confirmed `uvx`/`npx`/`pip3` are all absent from PATH, pixi is the only installer; and the command string is identical across hosts, so even opencode need not translate the command body — the translator only moves the `mcpServers` shape. (2) `websearch` (opencode's only remote entry, disabled) is not brought into the single source; remote types are not force-localized. (3) credential-bearing servers (local-postgres etc.) go through env, not committed files; keyless local servers (local-codegraph/local-github/local-playwright) enter `.agents/mcp.json`. (4) clangd/rust-analyzer are brought under pixi (conda-forge), resolving D29's clangd gap; Serena still deferred pending clangd in the pixi env. (5) a bootstrap is needed: `.agents/mcp.json` only describes launch and cannot express the three runtime facts "install package / pre-index graph.db / LSP probe" — adopt a hybrid: pixi `[dependencies]`+`[pypi-dependencies]` declare clean packages, `[tasks.mcp-<name>]` is a thin launch layer, and `scripts/mcp-bootstrap.py` (invoked by `pixi run mcp-setup`) does pre-indexing + LSP probe + repo-local wrapper handling; `scripts/gate-mcp.py` enters ci to verify each server can start. Design: `.omo/plans/2026-10-28-agent-config-unification.md` "MCP layer special design".

## D31: MCP single-source second-round adjudication (2026-10-28, user-adjudicated one by one) — scope / abstraction / credentials

Continuing the D28/D29/D30 MCP single-source discussion, the three P0 team-review items adjudicated one by one:
1. **MCP single-source scope = minimal set (A)**: `.agents/mcp.json` first batch contains only `local-codegraph` + `local-github` (currently enabled and keyless); `local-playwright`/`local-postgres`/`local-openspace` stay disabled, out of the first batch, appended drop-in on demand later. Reason: a disabled server is usually disabled for a reason; pulling it into the single source unverified = freezing an assumption into a spec (same discipline as D29/D30 deferring Serena).
2. **harness abstraction = per-layer small generators + shared base (B)**: extract `scripts/agent_common.py` (guard/classify/dry-run shared), with `rules-links.py`/`mcp-links.py` reusing it — not over-generalizing into a single `agent-links.py`. Reason: the four layers' actions are essentially different (symlink/copy/translate/aggregate); forcing them into a generic loop would be over-abstraction (team review #13).
3. **credential shape = two-file merge (B)**: commit `.agents/mcp.json` (keyless) + gitignored `.agents/mcp.local.json` (real env for postgres etc.) + `.agents/mcp.local.json.example` template, merged at runtime/generation time, keys never enter git. Reason: structure visible to all, keys not leaked, uniform across hosts, the common MCP-ecosystem practice (team review #14). Even with no keys in the first batch, the shape is established first; adding postgres later is a config-level operation.
All three written into the corresponding sections of `.omo/plans/2026-10-28-agent-config-unification.md`.

## D32: Stage 0 start — codegraph confirmed an npm package (review #17 verified + implemented) (2026-10-28)
MCP single-source stage 0 underway. Step one was review #17's "verify package source": reading `.opencode/init-mcp-codegraph.mjs` confirmed **`codegraph` is the npm package `@colbymchenry/codegraph` (not pypi)**, launched via `npm install -D` then `codegraph serve --mcp`. Implemented: (1) `.agents/mcp.json` (minimal set local-codegraph+local-github, all commands `pixi run mcp-<name>`, adjudication 1 A); (2) `.agents/mcp.local.json.example` + `.gitignore` gains `.agents/mcp.local.json` (adjudication 3 B); (3) `scripts/mcp-serve.mjs` generic version-pinned npm launcher (not `npx -y`, implementing review #16); (4) `pixi.toml` gains `[feature.mcp]` + tasks (mcp-codegraph/mcp-github/mcp-setup/gate-mcp), default env mounts the `mcp` feature. ⚠️ the codegraph/github version pins in `mcp.toml` (0.9.0 / 0.6.0) are starting values, to be locked after `npm view` confirmation (acceptance #8). serena still deferred (pending clangd).

## D33: MCP real-launch acceptance (acceptance #8, 2026-10-28) — version pins and env name corrected by live probe

Stage 0 ran `pixi run mcp-codegraph` / `mcp-github` for real acceptance, catching and fixing two genuine bugs:
- **codegraph version pin**: the original `0.9.0` pin was flagged by npm as **Broken install (requires 0.9.1+)**; changed to **1.6.0** (the version opencode's wrapper installed as latest, proven installable and startable). The launcher `scripts/mcp-serve.mjs` previously used `existsSync` to skip reinstall, which would bypass the version pin (#4 hole) — changed to verify installed version == pinned version, reinstall if mismatch.
- **github server env name**: the live probe reads **`GITHUB_PERSONAL_ACCESS_TOKEN`**, not `GITHUB_TOKEN`; canonical `mcp.json` and `mcp.local.json.example` are corrected (canonical uses `"GITHUB_PERSONAL_ACCESS_TOKEN": "{env:GITHUB_TOKEN}"` to rename-inject the user-side GITHUB_TOKEN). With the correct env name, `GitHub MCP Server running on stdio`, exit 0.
- ⚠ **`@modelcontextprotocol/server-github@0.6.0` is deprecated (npm: no longer supported) and `npm audit` reports 3 high vulnerabilities** — this repo only uses it for a local MCP demo, not blocking for now; security / least-privilege (team review #8) to be patched before stage 1 enables credential-bearing servers.
- during the codegraph upgrade, npm hit `ENOTEMPTY` (leftover platform dir from the previous broken 0.9.0) — clearing `node_modules/@colbymchenry` then a clean install of 1.6.0 from scratch (exit 0). A clean clone has no such issue.

## D34: Stage 1 implemented and verified (2026-10-28) — per-layer small generators + shared base + gate in ci

Stage 1 landed per adjudication 2-B (per-layer small generators + shared base) and fully passes:
- `scripts/agent_common.py`: extracts skills-links' guard/classify/link/backup/exit codes (0/1/2) for reuse across layers.
- `scripts/rules-links.py` (rules layer symlink/copy transport, 81 md files, hierarchy preserved), `scripts/mcp-links.py` (mcpServers written directly for codebuddy/cursor; opencode converted to named keys with **non-destructive merge** preserving websearch/playwright/postgres), `scripts/memory-aggregate.py` (4 memory md files aggregated into root `AGENTS.md` marker block `<!-- BEGIN:agents-memory -->…END -->`, hand-written content isolated) — all dispatched by the thin `scripts/agent-links.py` scheduler (importlib loads hyphenated filenames by path, preserving plan-doc naming).
- `scripts/mcp-servers.json`: **single source** for launcher params (pkg/version/bin/args); `mcp-serve.mjs` gains `--from/--name` support; pixi `mcp-*` tasks and `gate-mcp.py`/`mcp-bootstrap.py` share it → removes version-pin duplication.
- `scripts/gate-mcp.py`: invokes each server's launcher directly (node is in the env), **bypassing the nested `pixi run`** (review #18's PATH hazard empirically triggered: pixi absent from PATH inside the env); missing credentials degrade to SKIP (yellow) not red.
- `scripts/mcp-bootstrap.py`: idempotently ensures codegraph is installed; index strategy deferred (#6).
- `pixi.toml`: `gate-agents`+`agent-sync` tasks; the `ci` chain now includes `gate-agents`+`gate-mcp`.
Verification: `pixi run agent-sync` writes → `gate-agents` all green (rules 81 / mcp 2×2 / memory 4); `gate-mcp` green (codegraph OK, github SKIP on missing credential); `opencode.json` valid after merge and has a `.bak` backup.
Deferred (non-blocking): rules frontmatter dialect normalization needs the #11 mapping table first (this script only transports, does not translate); codegraph index increment (#6); `@modelcontextprotocol/server-github@0.6.0` deprecated + 3 high vulns (D33).

## D35: Host discovery behavior empirically verified (review #15/#18, 2026-10-28) — verification caught two dead files

Following "verify before claim", a minimal verification of CodeBuddy's discovery behavior (web_fetch official docs + local `~/.codebuddy/mcp.json` evidence) yielded:
- **skills**: `.codebuddy/skills/` is read — already landed and ci-verified, ✅.
- **mcp filename**: `~/.codebuddy/mcp.json` = `{"mcpServers":{}}` was the **wrong evidence base** — that is the *deprecated* user-level file, and it says nothing about the project level. Official docs (`docs/cli/mcp`) give the real layout: USER `~/.codebuddy/.mcp.json` (recommended) > `~/.codebuddy/mcp.json` (deprecated) > `~/.codebuddy.json`; **PROJECT `<repo>/.mcp.json` (recommended) > `<repo>/mcp.json` (deprecated)**; prefixes from the same file do not merge — only the first existing one is read. Nothing is read from `.codebuddy/mcp.json`, so the D35 target was a **dead file even though `gate-agents` was green** (the gate compared generator output against generator output — green but meaningless). **Correction, same day**: `agent-targets.json` now points at `.mcp.json` (repo root, commit-tracked, launcher path repo-relative so the file is machine-independent); the fix was forced by direct evidence — once the file existed the IDE panel parsed it and rejected the entries with `Server type must be one of: 'stdio','sse','streamableHttp'`. Two shape facts from that error: the IDE **requires** `type` (CLI docs claim it can be inferred from `command` — panel is stricter), and env references use `${VAR}` (opencode's `{env:VAR}` is not expanded by CodeBuddy; `mcp-links.py` translates on the opencode side). Third shape fact, same class of bug: `command` must be a **STRING** with the rest in `args` — the canonical file previously carried opencode's array form (`command: ["pixi","run",...]`), which crashed the panel with a JS `ir.replace is not a function`; `_host_entry()` now rewrites launcher hosts to `command`/`args` and `_expected_opencode_mcp()` reassembles the array. Same failure mode is reported upstream (`Tencent/teamai-cli#1185`: project servers written to `.codebuddy/mcp.json` never load, no error). Consequence of the tracked file: `.agents/mcp.local.json` must **not** carry secrets any more (its values are merged into the tracked `.mcp.json`); the token comes from the environment the host inherits.
- **rules shape**: docs are conclusive — CodeBuddy rules are **not** arbitrary `.md` under `.codebuddy/rules/`, but **one subfolder per rule + a `RULE.mdc` file inside it** (frontmatter with `description/alwaysApply/enabled/updatedAt/provider`). The 81 flat `.md` symlinks currently produced by `rules-links.py` are **not read at all** → this confirms the #11 frontmatter/shape translation work is **necessary, not speculative**. And `.agents/rules/` is an org-wide language-generic set (includes php/swift/perl etc. absent from this repo), pushing it to a Rust-engine IDE is noise → the translation must filter by project language (rust/c/cpp/web/typescript/python).
- **CODEBUDDY.md**: docs say a project-root `CODEBUDDY.md` is loaded in full (independent of the rules system). This repo has no such file; memory aggregation currently only writes root `AGENTS.md` (read by opencode/claude/cursor). Whether CodeBuddy reads `AGENTS.md` remains to be verified — a potential third discovery item.
Conclusion: before verification, `gate-agents` all-green was "green but false" (rules/mcp were once dead files); the discovery verification was well worth it. Follow-up: mcp fixed; rules need the #11 translation (RULE.mdc + language filtering); CODEBUDDY.md TBD.

## D36: Rules layer #11 translation designed & adjudicated (2026-10-28)

The rules dialect-translation (#11) is now designed and user-adjudicated one by one (7 points), resolving the D35 finding that CodeBuddy cannot read the 81 flat `.md` symlinks `rules-links.py` produces — it needs `.codebuddy/rules/<name>/RULE.mdc` with frontmatter (`description/alwaysApply/enabled/updatedAt`).

All seven adjudications chose option A:
1. **Language whitelist** = `["common","rust","cpp","web","typescript","python"]` → 40 effective rules; the 8 noise langs (csharp/dart/golang/java/kotlin/perl/php/swift, 40 rules) excluded (this repo is a Rust engine; org-wide set is noise per D35). Note: no `c` dir exists (only `cpp`).
2. **`alwaysApply: true`** — force-load all 40; `globs` field undocumented, not used.
3. **`provider` field omitted** — CodeBuddy docs list it but give no value; do not fabricate.
4. **`updatedAt`** = stable hash of canonical content (not wall-clock now) so `gate-agents` content-compare stays stable across runs.
5. **RULE.mdc body = canonical content verbatim after stripping any existing frontmatter block** — self-review found 20/40 whitelist files (rust/cpp/typescript/python) carry their own `paths:` frontmatter; naive prepend would produce double frontmatter (broken file). Their `paths` globs are dropped (`alwaysApply: true`; CodeBuddy `globs` undocumented — canonical `paths` remains the future mapping source). `description` = first H1 found anywhere in the file (web files carry it on line 3, after an extends-note), fallback `<lang>-<topic>` slug.
6. **Folder naming** = flat `<lang>-<topic>/RULE.mdc` (e.g. `rust-coding-style/RULE.mdc`), matching D35's verified one-level shape; nested `<lang>/<topic>/` not proven to be recursed.
7. **Cursor left `enabled: false`** — its discovery behavior unverified; the `mdc` generator is host-agnostic, so Cursor can adopt later after its own discovery check.

Implementation (not yet coded): add `rules_mode: "mdc"` + `langs` whitelist to the codebuddy target in `scripts/agent-targets.json`; extend `scripts/rules-links.py` with an `mdc` branch (copy+normalize, prune stale `.md` symlinks/old folders, content-compare check); keep `links` mode for other hosts. `gate-agents` rules check becomes content-compare for mdc targets (true-green). Generated `.codebuddy/rules/**/RULE.mdc` is gitignored (`.codebuddy` L110); edits to generated files are overwritten on sync — single source stays `.agents/rules/`. Design recorded in `.omo/plans/2026-10-28-agent-config-unification.md` (§Rules layer #11 translation).

**Status (2026-10-28): implemented & verified.** `scripts/agent-targets.json` codebuddy gains `rules_mode: "mdc"` + `langs: ["common","rust","cpp","web","typescript","python"]`; `scripts/rules-links.py` adds the `mdc` branch (strip canonical frontmatter, filter by `langs`, synthesize `RULE.mdc`, prune stale). `pixi run agent-sync` yields 40 `RULE.mdc` folders; the old 81 `.md` symlinks pruned to zero; `pixi run gate-agents` green with content-compare (true-green, not the D35 false-green). Review-finding fixes applied: canonical `paths:` frontmatter stripped (not duplicated), `description` = first H1 anywhere (web H1 on line 3), `updatedAt` = sha256 first 12 hex.

## D37: Memory layer confirmed consumed by CodeBuddy — root AGENTS.md IS read (no CODEBUDDY.md needed) (2026-10-28)

Resolves the D35 open item ("CODEBUDDY.md / does CodeBuddy read AGENTS.md — TBD") via official-doc evidence (CodeBuddy Rules doc, translated): "For backward compatibility, when the project root has AGENTS.md but not CODEBUDDY.md, CodeBuddy automatically loads the full content of AGENTS.md into the conversation context."

Conclusion:
- CodeBuddy loads root `AGENTS.md` in full **iff** `CODEBUDDY.md` is absent. This repo has no `CODEBUDDY.md`, so the memory region in `AGENTS.md` (produced by `memory-aggregate.py`) IS consumed by CodeBuddy — the memory layer is real, not a dead layer. No `CODEBUDDY.md` generator is needed.
- **Do NOT add `CODEBUDDY.md`**: it takes precedence and suppresses `AGENTS.md`, which opencode/claude/cursor also read. Adding it would fork the memory source per host and break the single-source model. Keep one `AGENTS.md`.
- Observed side-effect: root `AGENTS.md` is ~320 KB (the memory region aggregates conventions/decisions/pitfalls/status in full). Full-context injection every session is a large token cost and contradicts the plan's "concise aggregation, deep records on-demand" intent — a separate token-budget optimisation is deferred (not fixed in this pass).
- `.agents/` direct support: docs do not mention it; not relied upon (per-host layers are generated instead).

## D38: C17 English scope clarified — git-tracked prose only; functional CJK exempt (2026-10-28)

User clarification: the English mandate (C17) applies to **git-tracked files only**; gitignored working docs (e.g. `.omo/plans/*`, ignored at `.gitignore` L107 — `git ls-files .omo/plans/` = 0) are out of scope and may remain in the working language.

Within tracked files, the mandate covers **prose/artifacts we author and maintain** — `.agents/memorys/*.md`, root `AGENTS.md`, and our `scripts/*.py` — all now 0 CJK. It does NOT mean stripping **functional CJK data**:
- `.agents/skills/*/SKILL.md` — Chinese trigger phrases / protocol text (the skills must respond to Chinese commands; translating the triggers disables them).
- `examples/**` — CJK-label demos (E505_cjk_labels etc. exist to demonstrate CJK rendering).
- `crates/**` — pre-existing Chinese comments / test strings (fixture-class).
These are functional data (same class as C20's fixture exception) and are intentionally kept.

Rationale: C17's purpose (grep / CI anchoring / external readability) targets the memory hub and tooling, not trigger keywords or demo fixtures; translating them breaks functionality for zero benefit.

Side note: `.omo/plans/2026-10-28-agent-config-unification.md` was translated to English before this clarification; it is gitignored so the translation was not required, but it is kept (harmless).

**Refinement (2026-10-28, user-ruled):** skill prose → English, Chinese trigger keywords kept. Scope: `context-engineering`, `skill-router`, `adjudication-walkthrough`, `lesson-review` (the four with Chinese prose); the four single-trigger skills (`security-hardening`/`performance-optimization`/`ecosystem-scan`/`code-simplification`) keep their one Chinese trigger word as functional data. So the earlier "entire-skill Chinese exempt" is narrowed to "triggers kept, prose translated". See `.omo/plans/2026-10-28-agent-skills-rules-optimization.md` §7.

## D39: MCP cross-platform launch fix — adjudicated (2026-10-28, user-adjudicated one by one)

Fixes the verified failure where opencode (and CodeBuddy) MCP servers using `["pixi","run","mcp-<name>"]` die with "Executable not found in $PATH: pixi" (pixi lives in `~/.pixi/bin`, not on the GUI host's inherited PATH). Plan: `.omo/plans/2026-10-28-mcp-crossplatform-launch.md`. Three adjudications, all A:
1. **Fix combination** = O2 immediate hotfix (route opencode's two entries through the existing tracked `.opencode/with-node.sh`, which injects `~/.pixi/bin`) + O3 target (add a cross-platform launcher pair `scripts/mcp-launch.{sh,bat}` generated per host/OS) + O5 fallback (bootstrap/docs ensure pixi is on the GUI-inherited PATH: Linux `/usr/local/bin` symlink or session env; Windows via the pixi installer's user PATH) + harden `gate-mcp` to probe the real generated command under a sanitized PATH (kills the false-green).
2. **Tracked `.opencode/opencode.json` stays OS-neutral** (`["pixi","run",...]`), relying on O5; O2 is interim only. OS-specific launcher commands go only into gitignored host layers (`.codebuddy/mcp.json`).
3. **Windows = best-effort, deferred** (plan #7): ship `scripts/mcp-launch.bat` marked "authored on Linux, not yet exercised on real Windows", per the project's existing convention (`pixi.bat`/`pixi.ps1`/`bootstrap.bat` carry the same note). No Windows machine available for real verification.

Root cause class is the same as D35/D37: a "green but host-broken" false-green caused by the gate not reproducing the host's real spawn environment.

**Status (2026-10-28): implemented & verified.** O5: a user-level `~/.local/bin/pixi -> ~/.pixi/bin/pixi` symlink (no sudo) makes `pixi` resolve on the GUI-inherited PATH; opencode.json stays `["pixi","run",...]`. O3: added `scripts/mcp-launch.{sh,bat}`; `mcp-links.py` wraps commands for `mcp_launcher` targets (`.codebuddy/mcp.json` now uses the absolute launcher) while opencode stays OS-neutral. Gate: `gate-mcp.py` probes under a sanitized PATH (drops `~/.pixi/bin`) and goes red if pixi is not resolvable there. Verified: `agent-sync` + `gate-mcp` + `gate-agents` green; launcher direct run exits 0. Windows `.bat` remains best-effort (#7).

## D40: Agent skills/rules optimization — ecosystem-scan adjudicated (2026-10-28, user-adjudicated one by one)

Ran `ecosystem-scan` Full mode (local deep audit + two external indexes: VoltAgent/awesome-agent-skills, addyosmani/agent-skills). Report + plan: `.omo/plans/2026-10-28-agent-skills-rules-optimization.md`.

Local: 23 skills (all carry name+description frontmatter), 14 language rule dirs / 81 md, memory C25/D39 contiguous, single-source generation + gates in place. Issues: `review-hardcode` is a merged alias still in the directory; `AGENTS.md` has no SKILL DIRECTORY; 8 noise language rule dirs; rules frontmatter in three shapes; `AGENTS.md` ~316 KB full aggregation.

Seven adjudications (all A except #7 = B):
1. Rust engineering gap → new `rust-engineering` skill (reuse `rules/rust/*` as references).
2. Observability gap → new `observability-and-instrumentation` skill.
3. Debugging gap → new `debugging-and-error-recovery` skill.
4. General review gap → new `code-review-and-quality` skill.
5. Quality-gate single source + anti-gaming → new `constraint-driven-development` skill + root `CONSTRAINTS.md`.
6. Skill inventory → generate a compact SKILL DIRECTORY into the `AGENTS.md` memory region (marker section, gate-asserted).
7. `review-hardcode` → keep as an explicit deprecated alias (description marks it deprecated → `security-hardening`), not removed.

P2 (deferred): 6-section (Rationalizations / Red Flags / Verification) audit of existing skills, version-aware MSRV matrix, `repo-triage` router, ADR/shipping/deprecation skills, prune noise-language rules, trim the ~316 KB `AGENTS.md` aggregation.

**Status (2026-10-28): batch 0 done.** `book-to-skill` SKILL.md trimmed 616→486 lines by moving 3 self-contained sections to `references/` (`repl-access.md`, `update-fold-in.md`, `master-skill-template.md`); the skills link layer is a whole-directory symlink, so the references are reachable; `gate-agents` green. The planned "translate the 4 skills' prose" turned out a **no-op**: after re-inspection their Chinese is entirely trigger keywords / routing tables / protocol labels (functional), so it was kept per the "keep triggers" rule (CJK 22/12/6/3 unchanged).

## D41: AGENTS.md memory region became a navigable digest (2026-10-28)

`memory-aggregate.py` now emits a **digest** — one bullet per `## ` entry in each memory file plus a `- full text: .agents/memorys/<file>.md` pointer — instead of the full text of the four memory files. Result: `AGENTS.md` **334 KB → ~40 KB** (memory region 314 KB → ~22 KB, a 93% cut) while the full text stays in `.agents/memorys/*.md` (read on demand). The digest is deterministic, so `gate-agents` content-compare still passes.

Rationale: D28/D37 already specified "concise aggregation into root AGENTS.md, deep records kept as on-demand references", but the aggregator dumped full text, injecting ~314 KB every session. Trade-off accepted: the hub now carries titles, not full rule bodies — the digest header tells the agent to read the source file before acting on a rule it is unsure about.

## D46: New host onboarding deferred; Cursor needs a flat .mdc layout when adopted (2026-10-28)

Cursor/Qoder/Trae/ZCode are **not installed** on this machine, so no host adapter was enabled — adding one unverified is the D35 dead-file trap. Verified from Cursor's official docs: project rules are `.cursor/rules/**/*.mdc` (flat `.mdc` files; a plain `.md` is ignored), frontmatter is `description`/`globs`/`alwaysApply`, and **`globs` is officially supported**; MCP is `.cursor/mcp.json` with `type: "stdio"` required for stdio servers.

Decision (user-adjudicated): **defer**. Do not implement the flat `.mdc` layout or enable the target until Cursor is actually installed/used. When it is, the work is: a flat `<lang>-<topic>.mdc` layout variant, map canonical `paths` to `globs`, and emit `type: "stdio"` in the MCP entry. Qoder/Trae/ZCode remain unverified and are not added.

**Correction (2026-10-28, audit):** the "not installed" premise is wrong for Qoder — QoderCN 1.106.3 is installed on this machine (`~/.qoder-cn/`, project `.mcp.json` is loaded by it). The actual blocker is the unverified project-level rules path/shape: no `.qoder/` directory exists and the official Rules page returns 404. A runtime probe is scheduled (`.omo/plans/2026-10-28-agent-audit-followups.md` S5); the deferral itself (no adapter enabled) still stands. Cursor/Trae/ZCode remain unverified / not in use.

## D43: ast-grep ported — CLI dependency + ast-grep-rewrites skill (2026-10-28)

`ast-grep-cli ==0.51.0` (official PyPI distribution by the ast-grep author, MIT) added to the default pixi env `[pypi-dependencies]`; verified `ast-grep 0.51.0` runs. New skill `.agents/skills/ast-grep-rewrites/` (six sections): dry-run the pattern first, preview with `-r`, apply with `--update-all`, then run the gates; regex matching is the failure mode it replaces. This **supersedes** the 2026-10-28 deferral.

Verified: read-only probe found 264 `$X.unwrap()` sites; SKILL DIRECTORY 37; `pixi run agents check` green. Skill name per the naming convention (upstream `ast-grep` would collide with oh-my-openagent).

Pre-existing note (unrelated to this pypi change, `Cargo.lock` untouched): `pixi run audit` advisories FAIL on `rustls-webpki 0.103.15` / `webpki-roots 1.0.9` (RUSTSEC-2026-0382, distrusted emSign intermediates); RustSec fix = `cargo update -p rustls-webpki` (>=0.103.16) — tracked as its own dependency-security change.
## D42: MCP lifecycle CLI (`pixi run mcp`) — install/index/status consolidated (2026-10-28)

Plan: `.omo/plans/2026-10-28-mcp-lifecycle-cli.md` (four adjudications + three re-adjudications + an executability review; Chinese working doc in gitignored `.omo/` per D38).

Landed:
- **`scripts/mcp-cli.py`** — single lifecycle entry: `setup` (install/repair at the pin, `npm install -D --save-exact`), `index` (init on first run, full rebuild afterwards), `sync` (incremental), `status` (read-only; `Not initialized` is not a failure — `codegraph status` exits 0 there). Reached as `pixi run mcp <sub>`: pixi 0.78 forwards extra args to the task command (measured with a probe arg).
- **`pixi.toml`** — one global `mcp` task (global, not `[feature.mcp.tasks]`, because the mcp feature is default-env-only while `nodejs` is a base dependency). `mcp-setup` retired and `scripts/mcp-bootstrap.py` deleted: one implementation, no second entry name.
- **`scripts/mcp-serve.mjs`** — a **missing** binary still installs on the spot (GUI first click, clean clone and `gate-mcp` all depend on it), but npm's stdout is captured and re-emitted on **stderr** (stdout IS the JSON-RPC channel); a **pin drift no longer reinstalls** — it exits 1 with `run: pixi run mcp setup`. Rationale: "move the install out of the launcher entirely" was rejected in review because it would break both first-run paths.
- **npm quirk** — switching versions of a package with platform sub-packages leaves a non-empty staging dir and then fails `ENOTEMPTY` forever. `setup` moves those staging dirs aside (`_unblock`) instead of bulk-deleting a 1,300-file tree.
- **Index** — `.codegraph/` is built once per machine: 323 files / 3,874 nodes / 12.96 MB (rust 195, python 54, c 28, js 24, cpp 16, ts 4, yaml 2), i.e. ignore rules are honored and `.refinfo` (~11 GB), `target/` and `.pixi/` are not crawled. Rebuild 3.2 s, `sync` 0.4 s; gitignored, never built in CI.
- **codebuddy MCP generation re-enabled** — root `.mcp.json` is a generation product again (the "generation disabled until path/shape verified" note was obsolete: path, `type` and `command` shape were all verified against the live panel). `scripts/mcp-file-guard.py` hardened accordingly: server names read from the single source, plus shape assertions (`type` in stdio/sse/streamableHttp, `command` a non-empty STRING, `args` a list of strings, `${VAR}` env form, referenced launcher exists).
- **`docs/env.md`** — lifecycle commands documented next to the O5 PATH note; the stale `.codebuddy/mcp.json` reference corrected.

Open: removing `local-github` (no token, no use case while the remote is gitee, package deprecated with 3 high) lands as S5; when the GitHub mirror turns on, re-select against the official implementation rather than reviving `@modelcontextprotocol/server-github@0.6.0`.

**Status (2026-10-28): S1-S5 landed, gates green.** `pixi run mcp {setup,index,sync,status}` works; `.codegraph/` built (323 files / 3,874 nodes / 12.96 MB); `mcp-github` task and `mcp-bootstrap.py` retired, so the task surface is `mcp` / `mcp-codegraph` / `gate-mcp`; `gate-mcp` now `1 green, 0 skipped`; `GATE-AGENTS` reports `GATE-MCP-FILE ✓ .mcp.json valid (1 server(s), shapes ok, matches single source)`.

Removal caveat (S5): the opencode conversion in `mcp-links.py` is **non-destructive** -- it rewrites the servers it knows about and preserves every other `mcp{}` key (that is how `local-playwright`/`local-postgres`/`local-openspace` survive). Consequence: deleting a server from the single source does **not** delete it from `.opencode/opencode.json`; the stale key must be pruned by hand (done for `local-github`). `.mcp.json`, by contrast, is a full generation product, so it follows the source automatically. `@modelcontextprotocol/server-github` may remain in the gitignored `node_modules/` until the next `npm install` prunes it -- harmless, and no longer referenced anywhere.

## D45: Serena MCP integrated as a pixi-env server behind the unified registry (2026-10-28)

Plan: `.omo/plans/2026-10-28-serena-mcp-integration.md` (four adjudications + team review with 2 independent reviewers + an executability re-check; Chinese working doc in gitignored `.omo/`).

Landed:
- **pixi**: new `serena` environment (own solve-group, NOT in base; `qt5-spike` precedent): python 3.13 + pypi `serena-agent==2.1.0` + `rust-analyzer` + `clang-tools` (linux-64 only, target-scoped after review P0-5 -- aarch64/win-64 have no clangd evidence in the lock). Task `mcp-serena` uses `--transport stdio --context claude-code --language-backend LSP --project "$PIXI_PROJECT_ROOT"` (PIXI_PROJECT_ROOT measured inside tasks; `ide-assistant` context is deprecated in Serena 2.1.0, warning observed and fixed).
- **Registry** (`scripts/mcp-servers.json`): `kind` (`npm` | `pixi`) + `pixi_env`/`task`/`probe`. The kind MUST NOT leak into `.agents/mcp.json` (team review P0-3: `_host_entry` copies the entry verbatim and the guard does not check extra keys); the registry field is `pixi_env`, not `env` (the generation layer's `env` is an env-var dict, team review P0-2).
- **mcp-links**: `_host_entry` forwards EVERYTHING after `run` to the launcher (canonical may carry `-e <env>`); codegraph's generated bytes unchanged.
- **gate-mcp**: probe dispatch by kind. npm = start via launcher (as before); pixi = run the env binary `<repo>/.pixi/envs/<pixi_env>/bin/<probe>` -- nesting `pixi run` is impossible inside a task (review #18) and starting Serena would spawn language servers. A missing env is an explicit `skipped-env` counted in the summary (never folded into green), and `TOKEN_HINT` applies to npm only.
- **mcp-cli**: `setup`/`status` dispatch by kind; pixi version parsing keeps only the semver (Serena prints `2.1.0-<git describe>-dirty`).
- **`.serena/`**: `project.yml` tracked (verified no absolute paths); `cache` / `project.local.yml` / `memories` ignored at repo level -- Serena memories must NOT become a second memory store (D41). Serena's own `.gitignore` covers only cache + local overrides.
- **Tool surface measured**: default agent interface exposes 2 MCP tools (`initial_instructions`, `serena_repl`) with 53 internal operations behind the REPL -- token parity with codegraph's single tool; trimming via `excluded_tools`/`--mode` stays available.

分工 (vs codegraph): codegraph answers "where / who calls whom / blast radius" from the pre-built tree-sitter graph (1 tool, no LSP); Serena adds compiler-grade symbol editing and references via LSP (rust-analyzer present, clangd via clang-tools). Where does it fail: C++ needs `compile_commands.json` (presets export ON, but a build tree must be configured first) and the Serena `ls_specific_settings` wiring is still unverified.

Open: full project index pending (verified so far: rust LSP cache created in-repo); C++ symbol query after wiring; Windows unverified; Serena `probe-proj` registry leftover can be removed via `serena project remove`.

## D44: local-playwright added via the MCP runbook; 0.0.83 publish is broken (2026-10-28)

- `@playwright/mcp@0.0.83` (npm latest) depends on playwright-core `1.64.0-alpha-1790635538000` whose tarball is broken as published: `lib/` contains only LICENSE files (utilsBundle.js missing) -> the server cannot start at all. ALL @playwright/mcp releases pin ALPHA playwright-core builds (they track upstream dailies), so "avoid alpha" is not a strategy; version selection is.
- Pinned `@playwright/mcp@0.0.82`: setup's drift detection reinstalled it and the probe passed. gate-mcp now 3 green.
- The runbook (docs/env.md "Adding an MCP server") caught this at steps 7-8: the probe red is exactly why the manual-acceptance step exists; `mcp status` alone had shown "ok" (package installed) -- recorded as a known limitation of automated gating.
- Legacy `.opencode/init-mcp-playwright.mjs` is superseded by the pinned package (kept on disk until proven). opencode's legacy disabled `local-playwright` entry is overwritten by the enabled canonical entry (name match, non-destructive merge).
- Browsers are a tool-call-time runtime dependency: the probe passes without them; first real use may need `pixi run npx playwright install`.

## D47: Memory digest compresses to the host budget (2026-10-28)

D41 made `AGENTS.md` a navigable digest, but the digest still exceeded the observed per-turn injection ceiling: the previous 329,047-byte hub was cut at line 391 / 29,941 bytes, so everything after the first ~30,000 characters never reached the model. Measured 2026-10-28 before this change: hub 42,621 chars (pitfalls 10.5k, status 5.0k, skills directory 5.1k).

Decision (user-adjudicated): keep the digest shape and compress per file — pitfalls keep the latest 15 entries, status the latest 6 band sections, and the skill directory carries a ~45-character purpose; each window ends with a pointer to the full file. The skills region moves BEFORE the memory region because hosts truncate the tail. `GATE-HUB` asserts both `wc -m` and `wc -c` <= 30,000 (warn at 28,500, 95% of the limit).

This amends D41: "one bullet per entry" now applies to conventions and decisions only; pitfalls and status are windowed. Trade-off accepted: older entries leave the digest but stay one Read away in `.agents/memorys/*.md`. Rationale: a hub that cannot be read in full is not a hub.

## D48: `.gitignore` reorganized by ownership; build outputs root-anchored (2026-10-29, user-adjudicated one by one)

The file had grown by append (136 lines, 12 implicit groups, `.pixi` duplicated, `.cache` under "IDE", `Testing/` under "reference clones", `.sisyphus`/`.omo` under the activation-trio comment). Two decisions were adjudicated:
- D1 = **B**: build outputs (`build`, `target`, `out`, `dist`, `log`, `install`) are root-anchored (`/build/` etc.); measured: only `./build` exists, no nested same-name directories.
- D2 = **A**: legacy `stash`/`nodejs` rules are kept but root-anchored, with a legacy note (no current user; grep found no directory references).

Result: 12 explicit sections (noise -> legacy toolchains -> IDE -> build -> node -> python -> pixi -> secrets -> agent layers -> local notes -> wrapper -> clones), `.pixi` deduped, five `cmake-build-*` collapsed to `cmake-build*/`, `/CMakeUserPresets.json` added, and `.serena/*` root anchoring restored. Verification: 22/22 ignore matrix, ignored list no-diff, zero tracked-but-ignored files. The inline-comment trap found during review is recorded as PIT-85.

## D49: CodeBuddy rule budget — two always-on rules plus described on-demand rules (2026-10-29)

CodeBuddy injected all 40 generated rules with `alwaysApply: true` into every request — 139,613 characters, ~14x the IDE's 10k warning. Two changes landed:

- **Always-on whitelist** (`always_on_rules` in `agent-targets.json`, honored by `_mdc_content`): only `common-constraints` + `common-security` stay `alwaysApply: true`; the other 38 flip to `false`. Always-on payload 139,613 -> **3,048** characters (-98%); 2 true / 38 false.
- **Rule descriptions** (`rule_descriptions`, same config): 12 high-frequency non-core rules (common coding-style / testing / edit-safety / performance / code-review; rust coding-style / testing / security / hooks; cpp, typescript, web coding-style) get a hand-written "Use when ..." description instead of their H1 title — with `alwaysApply: false`, the description is what selects the rule.

`edit-safety` was deliberately **not** trimmed (483 lines / 45 KB): it left the always-on budget when it flipped to `false`, so the 10k warning is resolved without cutting content (adjudicated C).

Verification: 2 true / 38 false; always-on stays 3,048 chars; a second `agent-sync` produces zero `MDCMK` (idempotent); `gate-agents` green. Open item: whether the host actually pulls in `alwaysApply: false` rules by description is unverified — if it does not, the canonical files remain readable on demand at `.agents/rules/<lang>/`.

## D50: Trae host onboarding — four adjudications (2026-10-29, user-adjudicated one by one)

D46 deferred Trae on the premise "not installed / not verified". That premise expired: a live Trae session reads the root `AGENTS.md` as always-applied workspace rules and natively discovers all project skills under `.agents/skills/` (37 routed, zero link layer). The two native surfaces mean Trae needs no skills target at all — only MCP, rules, and a shadow guard.

Design (analysis only at time of writing; nothing landed yet):

- **Adj 1 = B, rules payload**: `.trae/rules/` gets the two always-on rules (`common-constraints` + `common-security`, matching D49's CodeBuddy whitelist) plus ONE router card that indexes the rest of `.agents/rules/` and says when to read which. Trae project rules are plain markdown with no frontmatter/`alwaysApply` switch, so "which files exist" is the entire budget decision. The router card is canonical content under `.agents/rules/` and flows through the existing verbatim transfer — no new hand-maintained layer. Rejected: full 12-file mirror (would replay the opencode always-on cost problem on a new host).
- **Adj 2 = A, MCP write semantics**: `.trae/mcp.json` is rewritten wholesale, byte-compared, no merge branch. `mcpServers` shape already transfers with zero code change (`_expected_mcp_file`); the OpenCode-only non-destructive merge stays OpenCode-only. Panel edits are transient and must land in `.agents/mcp.json` / `mcp.local.json` to count. Rationale: preserving byte-equality is what keeps `gate-agents` true-green and keeps D27/D31 ("generated layers are never hand-edited") intact.
- **Adj 3 = A, git boundary**: `.trae/` joins `.codebuddy/` and `.qoder/` as a fully ignored activation directory (not added to `generated_tracked`). The repo's existing split stands: contract-layer products (`AGENTS.md`, `CONSTRAINTS.md`, `.mcp.json`, `opencode.json`, `SKILL.md`) are committed; activation-layer products are ignored. Consequence: the `check` path needs an optional-target branch so a machine without Trae stays green (~10 lines).
- **Adj 4 = A, shadow guard placement**: the "`.trae/skills/` must not exist" rule goes into `skills-links.py` / `gate-skills`, reusing the existing exit-2 guard semantics (same failure class as a real directory squatting a link slot), rather than a new script + pixi task + `ci` dependency. Reason: one gate name to remember, and `CONSTRAINTS.md` does not drift.

Open items: (1) whether Trae injects every `.trae/rules/*.md` per turn or only directory-scoped ones is unmeasured — measure before fixing the router card's size; (2) whether Trae rewrites `.trae/mcp.json` from its panel behind the agent's back needs a live probe before Adj 2's rigid overwrite is trusted; (3) `bootstrap.sh` does not run `agent-sync`, so a fresh clone has no generated layer for any host — pre-existing gap, not introduced here.

**Revision (2026-10-29, R1-R3 confirmed = A each).** A review pass on the plan found four blocking defects in the wording above and corrected three of them:

- **R1 (amends Adj 2's comparison, not its write semantics)**: `.trae/mcp.json` is IDE-managed (the sandbox lists the path as read-only while the file does not yet exist), so the panel will re-indent / re-key it. `check` therefore compares **parsed structure**, not bytes; writing stays wholesale-overwrite with no merge. Rationale: `.strip()` in the existing byte comparison already conceded that whitespace is not content, and a gate that reddens on formatting churn stops being read.
- **R2 (amends Adj 1's placement)**: the router card is NOT canonical prose under `.agents/rules/`. `_canonical_rels` filters by `langs` only and has no per-file whitelist, so a card mentioning `.trae/rules/` would be mirrored into CodeBuddy as a 41st rule. `rules-links.py` synthesizes the card instead, from the actual `.agents/rules` tree — the same pattern as the mdc frontmatter synthesis.
- **R3 (amends Adj 4's severity)**: the `.trae/skills` shadow guard reports a **finding (exit 1)**, not a guard (exit 2). Exit 2 means "writing here would corrupt the single source, so nothing is written"; we never write into `.trae/skills`, and escalating would let one local skill file abort every host's sync.

Unchanged by the review: Adj 1 = B (two always-on rules + one router), Adj 3 = A (`.trae/` fully gitignored, not in `generated_tracked`), Adj 4 = A (guard lives in `skills-links.py` / `gate-skills`, no new gate).

Two review findings remain to be fixed in the implementation, both pre-existing rather than Trae-specific: `gate-agents` is not fresh-clone-safe (a missing layer reads as drift, and `ci` runs no `agent-sync` before it — `.codebuddy` has the same defect today), and `rules-links.py:297` hardcodes `copy_mode=False` with no copy fallback, so the rules layer would fail on Windows.

**Landed (2026-10-29).** Four slices, each verified before the next:

1. **MCP.** `trae` target in `agent-targets.json`. `mcp-links.py` gained two per-target flags: `optional` (an absent gitignored layer is a SKIP, not a finding) and `structured_compare` (parse-then-compare, so the IDE re-indenting the file is not "drift"). A third flag, `mcp_launcher_absolute`, was forced by a live failure: Trae does not spawn MCP servers with cwd = repo root, so the repo-relative `bash scripts/mcp-launch.sh` died with ENOENT even though the launcher resolves its own root (PIT-86). `.trae/` is gitignored, so a machine-local absolute path is legal there; root `.mcp.json` stays relative because it is tracked.
2. **First-run safety.** The review's blocking item was fixed WITHOUT adding `agent-sync` to the `ci` chain — that would make a gate write files and would inject a sync step into `CONSTRAINTS.md`'s derived gate list. Instead "absent gitignored layer = SKIP" was generalized to every host (`codebuddy` rules/mcp/skills too). The resulting contract: **a layer that exists but does not match is red; a layer that is absent is skipped**; committed generated artifacts are still compared byte-for-byte.
3. **Rules.** New `rules_mode: "copy"` in `rules-links.py` driven by an explicit `rules_files` whitelist (because `langs` means "language folders", not "these exact files"), flattened to one directory so every rule stays project-wide, frontmatter stripped, cross-rule links flattened to siblings. The router card is synthesized from the live tree — it is not canonical prose (R2) — so removing a canonical topic removes its entry automatically. Trae confirmed it reads the folder: file reads return a "Rules relevant to this file" section carrying all three rules verbatim.
4. **Shadow guard.** `forbidden_dirs` in `skills-targets.json`, checked in `skills-links.py`: a `SKILL.md` under `.trae/skills` is a finding (exit 1, R3), and `sync` still runs for the other hosts.

Also fixed en route: `bootstrap.sh` now runs `agent-sync` as step 4/4 (advisory), `.agents/rules/common/agents.md` documents the generated-layer map for every host, and root `AGENTS.md` lists `.trae/` among generated layers. Two incidental findings were filed: PIT-86 (launcher reference cwd) and PIT-87 (a 0-byte case-variant `AGENTS.md` in a rules folder silently became a generated rule).

Still open after landing: whether Trae injects all three `.trae/rules/*.md` every turn or only directory-scoped ones is unmeasured (it decides how large the router card may grow), Trae's panel has not been observed rewriting `.trae/mcp.json` (byte-equal after the first sync, but the panel was never saved), and `bootstrap.sh` previously left every host layer absent on a fresh clone — now covered by step 4/4.

## D51: `.gitignore` gaps closed — pre-declared hosts, local env files, pycache (2026-10-29, user-adjudicated one by one)

Audit of the post-D48 ignore set found no over-ignores (0 tracked-but-ignored, 0 ignored files in source/docs/resources, sensitive paths covered) and four preventive gaps. Adjudicated A/A/A/A:
- D1: `.claude/` and `.cursor/` are ignored now, alongside `.codebuddy/`/`.qoder/`/`.trae/`; both targets are pre-declared (disabled) in `agent-targets.json`.
- D2: `.env`, `.env.local`, `.env.*.local` are ignored with `!.env.example` kept committable.
- D3: `__pycache__/` is ignored as a directory (`*.pyc` alone misses non-pyc leftovers).
- D4: `*.log` is deliberately NOT ignored — ctest logs are already covered by `Testing/`, and a global rule would hide future sample-log fixtures.

Verification: 29-case `git check-ignore --no-index` matrix passes (22 existing + 7 new), `git ls-files -ci --exclude-standard` is empty, inline-comment grep is empty, `GATE-AGENTS` green. Plan: `.omo/plans/2026-10-29-gitignore-gaps.md`.

Note: the same audit round resolved a concurrent numbering collision — the renumbered duplicate PIT-29 entry became PIT-88 because the Trae onboarding round had independently taken PIT-86.

## D52: Repo-wide review fixes — six batches, with the debt named (2026-10-29)

A read-only review of the whole repo (engineering, scripts, build, config, gates) produced 6 Optional + 7 Nit findings; six batches landed:

- **B1 `ebb65c4`**: `mcp-serve.mjs` spawns npm with an argument array (no shell) while still capturing stdout and re-emitting it on stderr — stdout is the JSON-RPC channel. The dead `@modelcontextprotocol/server-github` entry lived only in the gitignored, generated `package.json`, so removing it is a local clean-up with no repo diff.
- **B2 `ef207c3`**: rules copy mode prunes like the mdc path (symlink/dir/file) and skips pruning when a whitelist entry is unusable; `check` classifies nested dirs as `subdir` and ignores `.bak-*` backups (a pre-existing false positive that would redden after any prune). codebuddy is no longer `optional`, and CI runs `pixi run agents sync` before `pixi run ci`, so a missing primary layer is a real defect instead of a silent SKIP.
- **B3 `96fe1ec`**: `cmake-smoke` and `check-promises --self-test` restore their moved files via traps on EXIT/INT/TERM.
- **B4 `77363bd`**: workspace member de-duplicated; `deny.toml` header refreshed and `MPL-2.0` removed from the allow list (zero MPL crates today, so an MPL dep now fails the gate for adjudication); the pre-commit hook stages generated artefacts line by line.
- **B5 `49ebd8e`**: `[workspace.lints.rust] unsafe_op_in_unsafe_fn = "deny"` inherited by all 12 packages (0 hits); four pre-existing clippy hits fixed in ffi; qt6-main/binaryen/gxx_linux-64/gcc/binutils range-pinned from the lock; a new `msrv` environment pins rust 1.98.0 and `cargo check --workspace` passes there (48.7 s), wired into CI.
- **B6 `b3eeed8`**: two gate scripts gained `pipefail`; `docs/env.md` names directory ownership (cmake-build* trees, build/gallery/assets thumbnails, target/ cargo); `SECURITY.md` and `CONTRIBUTING.md` added; the standby CI workflow records that its actions must be SHA-pinned before enabling.

**Named debt**: `clippy::undocumented_unsafe_blocks` is deliberately NOT enabled — 37 legacy FFI blocks lack `// SAFETY:`, and `-D warnings` overrides per-crate `[lints]` (measured: `-W` and per-crate `allow` both lose), so the lint cannot be scoped. The root `Cargo.toml` note records it; enable workspace-wide once backfilled. **Skipped as designed**: wasm-serve main guard, `bootstrap.bat` partial-submodule condition, Trae's bash dependency. **Environment limits** (identical before and after): full `cmake-smoke` is blocked by the sandbox bulk-delete guard, and `gate-web` needs a serve on :8018.

**Follow-up (same day)**: the decisions digest joined the window (`WINDOW["decisions"] = 25`) — 52 entries had pushed the hub to 28.6 KB, leaving only ~1.4 KB before the 30 KB hard cap. `AGENTS.md` fell to 25.7 KB and `GATE-HUB` went quiet. Full text stays in this file; the window is deterministic, so the gate compare still passes.

## D45: human-facing docs site -- mdBook + codegraph graph + lazy mermaid (2026-10-29)

Plan: `.omo/plans/2026-10-28-repo-book.md` (14 sections: four adjudication rounds, a team review, a re-review; Chinese working doc in gitignored `.omo/`).

- **Stack**: mdBook 0.5.4 in a new `[feature.book]` together with markdownlint-cli and GNU gettext tools; `cargo-binstall` is target-scoped to linux-64 (conda-forge ships no aarch64 build). The only planned non-pixi install is `mdbook-i18n-helpers` (S5), pinned per the cargo-binstall rule.
- **Layout**: `docs/` is the single source; `book/` holds only the site shell (`book.toml`, `toc.toml`, and the graph/mermaid assets); `scripts/book-build.py` assembles `build/book-src/` and runs `mdbook build -d www/book/<lang>`; `www/` is a new gitignored output territory, keeping `build/` reserved for the wasm wall.
- **Graph page**: `graph.json` is generated by reading `.codegraph/codegraph.db` directly -- the codegraph SDK exposes only per-seed subgraph queries, not a whole-graph export. After excluding tooling/site/output trees the payload is ~2.1k nodes / ~7.8k edges / ~300 KB. The page is an iframe over a cytoscape app (search, 1-2 hop neighbourhood, click-through to the exact repo file#L).
- **Diagrams**: mermaid fences rendered client-side by a 1 KB loader that pulls the vendored bundle only on pages that need it (mermaid 11.17.2 UMD, 3.5 MB; v12 ships 5.3 MB plus 550 hash-named chunks). No CDN, so the site stays offline-safe like the wasm wall.
- **Bug caught by the build**: the markdown chapter and the static app both resolved to `web/graph.html`; the app moved to `web/graph-view.html`, and gate-docs ⑬ now asserts the chapter embeds it.
- **Gates**: `gate-docs` ⑬ (SUMMARY entries resolve; graph chapter embeds the app) SKIPs when `build/` is absent, like cmake-smoke; `pixi run book-check` asserts index/graph.json/app/vendor and will grow a browser layer once chromium is available.
- **Language boundary**: book sources stay English (gate-i18n scans .toml/.js/.md, so `toc.toml` titles are English); Chinese arrives through PO catalogs in S5. `visibility = "internal"` marks pages that `--public` removes from both the SUMMARY and the artifact.
- **Status**: S1-S4 landed; S5 (PO/i18n), S6 (language switch + browser checks), S7 (gate-prose: type rules, evidence anchors, banned words), S8 (book-serve/book-graph tasks, footer provenance) remain.

## D46: the book is English-only; browser-side translation covers other languages (2026-10-30)

Revision of D45's bilingual plan. The PO pipeline (mdbook-i18n-helpers, book/po/,
BOOK_LANG builds, the language switch and the coverage ratchet) is removed: measured
over five batches it cost a re-pot/re-merge/re-translate tax on every English docs
edit, and its hardest tail (escaped characters, fenced-block comments, quoted strings)
fought the preprocessor's markdown reconstruction. The reader-side alternative --
Edge/Chrome built-in translate, or Immersive Translate -- covers 100% of the content
(including the 27 SDD pages that were never in translation scope) with zero
per-release maintenance.

- **Kept**: the whole mdBook site, the codegraph graph, lazy mermaid, book-serve,
  book-graph and the provenance footer are language-agnostic and unchanged.
- **Kept in history**: book/po/*.po|pot (513 translated strings, 86% of translatable
  prose) stay recoverable from git if a published Chinese tree is ever wanted again.
- **Marker rename**: toc.toml's `translated = true` becomes `polish = true` -- the
  attribute now scopes gate-prose's banned-word list, not a translation promise.
- **Simplified**: book-build emits www/book/en only (no generated per-language book
  dirs, no langs.json/root.html landing page); book-check drops the bilingual
  assertions; the cargo-binstall and gettext dependencies leave the book feature.

## D53: the docs site moves to `build/book`; `www/` retires (2026-10-30, user-adjudicated U4)

Supersedes D45's layout half. Its rationale -- "keeping `build/` reserved for the wasm
wall" -- was already false when written: the wall is served from
`cmake-build-*/web-demo` (gate-web.sh:34) and merely *consumes*
`build/gallery/assets`. Measured tenants of `build/` before this ruling:
`book-src/` (the assembled book source), `gallery/`, `i18n-src/`, `i18n-check/`.

- **Decision A**: `book-build` writes `build/book/<lang>` (today `en/`), `build/book/api/`
  and the root redirect; `build/book-src/` stays the assembly dir. One tool, one root.
- **Why not B (`www/book`)**: `www` had exactly one tenant and three hardcoded constant
  references (book-build/book-check/book-serve) -- a path alias, not an output territory.
  Keeping it froze "one tool spans two derived roots", the only real defect in the status
  quo. The "zero gitignore exception" argument for B was inverted: `/www/` had been ignored
  since D45, so A *removes* a rule instead of adding one.
- **Why not C (`out/book`)**: `out/` has no consumer anywhere (scripts / CMake / tasks:
  zero writers). Giving an empty ignored slot its first meaning would force any future
  deploy work to negotiate ownership with the book artifact.
- **Guard**: `OUT_DIR` must stay a subdirectory of `build/` -- book-build's stale-language
  prune deletes any *direct child* of `OUT_DIR` carrying `index.html`, so pointing it at a
  bare `build` would sweep `gallery/`. Inline comment added at the constant.
- **Deliberately unchanged**: `-d` still passes per target (the root redirect and `api/`
  are siblings of `en/`); the artifact a CI pages job would take is `build/book`; no
  command name changes. Touch list: three scripts, gate-docs ④ skip prefix, pixi.toml,
  book/book.toml, docs/env.md, AGENTS.md, .gitignore.
