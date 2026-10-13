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
- **Decision**: brand name = **VisiaEngine / 维视引擎** (PascalCase, docs/UI/formal address); technical prefix = **`visiaengine-`** (lowercase full brand name: crate package names / lib underscore names / workspace paths / CI and gate references, full chain). C ABI layer: symbols `visiaengine_*`, header `visiaengine.h`, library `libvisiaengine.{so,dylib,dll}` (capi slice delivers it). Environment-variable prefix reserved: `VISIAENGINE_`. Layering exception: **product names keep their own form** (the editor "Visia Studio" is not a technical namespace).
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
