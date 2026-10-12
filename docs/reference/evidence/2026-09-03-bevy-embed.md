# Bevy-as-default-backend research memo (for merging into the C1 decision table)

Method note: all evidence live-verified over the network on 2026-09-03 (GitHub API / bevy.org / docs.rs / crates.io / MDN BCD). `raw.githubusercontent.com` is blocked on this machine, so everything goes through the `api.github.com` raw route — no impact on the conclusions. No cargo/rustc on this machine, so binary sizes could not be measured (see Q4).

**Version facts (anchored first)**: latest release **v0.19.1 (2026-08-13)**, 0.19.0 = 2026-06-18; main = **0.20.0-dev**, edition 2024, MSRV 1.96.
- https://github.com/bevyengine/bevy/releases.atom (fetched 2026-09-03)
- https://github.com/bevyengine/bevy/blob/main/Cargo.toml (API fetched 2026-09-03)
- https://crates.io/api/v1/crates/bevy → max_version 0.19.1

---

## Q1 API stability record

**Breaking cadence (full tag census via the GitHub releases API)**: 0.x minor = a breaking version. 2024: 0.13(Feb)/0.14(Jul)/0.15(Nov) = 3 times; 2025: 0.16(Apr)/0.17(Sep) = 2 times; 2026 through September: 0.18(Jan)/0.19(Jun) = 2 times. → **about 2–3 breaking minors per year**, the last 18 months slowing to one every ~6 months.
- URL: https://api.github.com/repos/bevyengine/bevy/releases (fetched 2026-09-03)

**Official position (verbatim)**:
- README WARNING: "breaking changes … released approximately once every 3 months … We provide migration guides, but we can't guarantee migrations will always be easy" / "MSRV generally close to the latest stable release of Rust" — https://github.com/bevyengine/bevy#readme (fetched 2026-09-03)
- Migration-guide preface: "Bevy is still in the \"experimentation phase\", which means each release has its fair share of breaking changes." — https://bevy.org/learn/migration-guides/introduction/ (fetched 2026-09-03)
- Among the sitemap's 1374 URLs there is **no 1.0 roadmap / API-stability policy article** (search terms 1.0/roadmap/stability/vision all came up empty) -> no official 1.0 timeline commitment.

**Breakage magnitude of one version**: the 0.18->0.19 migration guide contains **120 H3/H4 subsections** (the vast majority breaking changes); render-related: `RenderSystems::ManageViews` renamed, `shadow_pass` split per_view/shared, Camera TextureFormat rework, Post Processing Split, new crate `bevy_material`, old `bevy_scene` renamed `bevy_world_serialization`, `bevy_reflect` reorganized, the 2d/3d/ui features no longer imply each other. — https://bevy.org/learn/migration-guides/0-18-to-0-19/ (fetched 2026-09-03)

**cargo features/module strategy**: 0.20-dev reorganizes the API surface into feature sets (`default_app`/`common_api`/`2d_api`/`3d_api`/`ui_api`, the comments explicitly state "2d_api … does not include a render backend… unless you are using a custom renderer") — the feature->crate mapping is itself public API, and **it gets reshuffled every release** (the 0.19 migration includes a "bevy_window … features moved to alternate feature" entry).

**Empirical cost of pinning one version long-term**: pinning is technically feasible (crates.io retains all 63 versions; 0.19.1's own MSRV=1.95, buildable indefinitely), but **ecosystem lockstep** is the main cost. Case: the SpawnForge engine project's issue records the bevy 0.18.1->0.19 migration being "**HARD-BLOCKED since 2026-07-02**" because `bevy_rapier*` declared `bevy ^0.18.1`; once unblocked it required **five-party coordination in one PR** across bevy+rapier+hanabi+panorbit+the local gizmo fork — https://github.com/Tristan578/project-forge/issues/8887 (fetched 2026-09-03)

---

## Q2 can bevy_render be used apart from App/ECS

**Conclusion: it cannot, and after 0.19 the coupling is deeper.**
1. **The RenderGraph was deleted**: #22144 "Replace `RenderGraph` with systems" (opened 2025-12-15, **milestone 0.19**) — render passes are now systems in ECS `Schedule`s, a top-level `RenderGraph` schedule + the `camera_driver` system driving per-camera schedules; new `CurrentView`/`ViewQuery`/`RenderContext` (system param)/`SystemBuffer`. The author's stated motivation: "Schedule has basically become co-extensive with what the RenderGraph API is doing…iterate on a more ECS based approach". **The `crates/bevy_render/src/` at the v0.19.0 tag no longer has a `render_graph/` directory** (git tree measured). — https://github.com/bevyengine/bevy/issues/22144
2. **The 0.19.1 public API shape** (docs.rs): re-exports `ExtractSchedule`, `MainWorld`; modules `extract_component/extract_instances/extract_plugin/extract_resource`, `sync_component`, `sync_world`, `pipelined_rendering` — the dataflow is the "main world -> extract -> render world" ECS copy pipeline; there is no functional "given device+surface+scene, draw a frame" entry. — https://docs.rs/bevy_render/0.19.1/bevy_render/ (fetched 2026-09-03)
3. The official docs' own rendering-architecture description (The Bevy Renderer -> render-architecture chapter) stresses layering and the freedom to "bypass Bevy's abstractions", but that book's Renderer chapter is annotated **"This page is hidden because an ancestor is hidden" — the rendering-architecture chapter is in an unfinished state** (after the 0.19 rewrite the docs never caught up). — https://bevy.org/learn/book/the-renderer/render-architecture/

→ For the decision table: "reuse bevy_render" behind the `visia-render` trait = swallowing the whole ECS+App+SubApp+asset pipeline, stripping it out is not feasible; the viable minimal shape is Q3's App-disable-winit.

---

## Q3 manually driving Bevy inside an external event loop

**An official precedent exists, but it is extremely young and covers only headless/offscreen**:
- `examples/app/externally_driven_headless_renderer.rs`: "pumping the update loop manually" — `DefaultPlugins.disable::<WinitPlugin>()` + `WindowPlugin{primary_window:None, exit_condition:DontExit}` + `app.finish(); app.cleanup();` + `std::mem::take(app.sub_apps_mut())`, per-frame manual `update()` + a manual `RenderDevice::wgpu_device().poll(PollType::Wait)` (**a blocking poll, an inter-frame sync point**). First commit **#22551, 2026-01-18** — the pattern has been officially endorsed for under 8 months. — https://github.com/bevyengine/bevy/blob/main/examples/app/externally_driven_headless_renderer.rs (incl. commit-history API, fetched 2026-09-03)
- **Battle scars (this route was still bleeding in 2026)**: #24178 "headless_renderer: copy render target after RenderQueue has been submitted" (fix merged 2026-05); #24927 "Headless app (`WgpuSettings{backends:None}`) panics on despawn of any render-synced entity: PendingSyncEntity does not exist" (closed 2026-07-09) — **a render-synced world lifecycle bug only got fixed in July 2026**.
- **Rendering inside a host window (Qt widget / C# container surface): no first-party API or issue found at all**. Bevy's window creation lives in bevy_winit (inside the `default_platform` feature); issue searches for external window / parent window / embed in another engine / window handle all failed to surface any accepted capability request.
- The closest "engine inside a host" precedent **godot-bevy** (bytemeadow/godot-bevy, 540★, pushed 2026-09-02, v0.11.0): **deliberately does not use the bevy renderer** — README: "bring Bevy's powerful ECS to Godot, …leveraging Godot's excellent editor and **rendering capabilities**", the `#[bevy_app]` macro + `bevy = {version="0.18", default-features=false}` + on-demand plugins (v0.8+ explicit selling point "Smaller binaries"). → The flagship precedent proves ECS-logic reuse is feasible, **renderer reuse has been pulled off by nobody**. — https://github.com/bytemeadow/godot-bevy (fetched 2026-09-03)
- The opposite-direction `bevy_child_window` (not-elm, v0.2.1, last updated 2025-04-28): embeds other windows inside a Bevy window, not our direction; it shows the window-embedding primitives are all community DIY. — https://crates.io/api/v1/crates/bevy_child_window

---

## Q4 Size

**Controlled comparison data (bevy minimal 3D app vs bare wgpu triangle, native, three platforms): no credible public measurement found -> UNCERTAIN**. This machine has no rust toolchain and cannot fill the gap (`which cargo rustc` empty, measured 2026-09-03). Citable indirect evidence:
- A real wasm data point: jlg.io's homepage bevy background app, PR #550 (2026-07-03) self-reported "**13 MB wasm, ~3.4 MB gzipped over the wire**". — https://github.com/jgeschwendt/jlg.io/pull/550
- Officially acknowledged size burden: #20337 open (2025-07-30) "Reflection results in significant increases to compile times and **binary size**" — bevy_reflect in 0.19 is still **non-disableable** (no `reflect` feature flag). — https://github.com/bevyengine/bevy/issues/20337
- Official trimming means exist but are routine: feature pruning (`--no-default-features` + `2d`/`3d`/`ui`/`default_app`), the release-builds chapter (opt-level=z/lto=fat/codegen-units=1/strip/wasm-opt), the compiling-less-code chapter ("Keeping every feature enabled will result in … a bloated project binary size"). — https://bevy.org/learn/book/releasing-projects/release-builds/
- Bare wgpu triangle size: no first-hand public number to cite -> UNCERTAIN (the community often says "~1-2MB", no citable source was found in this research).

→ The verdict for the ≤10MB target can only read: the only measured wasm sample (13MB unstripped, 3.4MB gzipped) already hugs/breaches the line; the native side awaits a same-profile `--features 3d` vs in-house-wgpu control experiment once you have a toolchain — this is an **open measurement item** to be marked in §4.

---

## Q5 camera switching / custom passes / 100k-scale instances

- **ortho<->perspective runtime switching = a first-class pattern**: the official `projection_zoom.rs` example's `switch_projection` mutates the same entity's `Projection` enum in place (`Projection::Orthographic(_) ⇄ Projection::Perspective(_)`), no rebuild, no plugin reload; also `3d/orthographic.rs` (the official comment names "isometric-look in games or **CAD applications**"), `camera/custom_projection.rs` (custom projection trait), `pan_orbit_camera_ortho.rs`. **The pattern holds; the switch animation interpolation is on you (the API gives the jump only)**. — https://github.com/bevyengine/bevy/blob/main/examples/camera/projection_zoom.rs (source fetched 2026-09-03)
- **Custom passes**: the correct hook after 0.19 = the top-level `RenderGraph` schedule (#22144 verbatim "This top level schedule provides an extension point for apps that may want to do custom rendering, or non-camera rendering") + phases inserted in the per-camera schedule; the examples `custom_render_phase`/`custom_phase_item`/`custom_post_processing`/`render_depth_to_texture` all exist. **The cost: the hook shape was just fully replaced in 0.19** (the old Node-style graph API is gone), so all old tutorials/third-party render plugins are migration debt.
- **Heavy instancing**: the `automatic_instancing.rs` sample ships 256×256 = **65,536 cubes in one mesh one material automatically collapsed to 1 draw call** (the example's header comment: "should be only a single draw call", verifiable in a graphics debugger); `MeshTag` carries per-instance external data; `GpuComponentArrayBuffer` (#24922, enhanced in 2026-07 to store arbitrary per-instance data); the 0.19 theme "Render Bigger Scenes Faster" + 0.20-dev is merging an extra-large-scene example (#25371 "Add the Zero-Day large-scene example", 2026-08-11). **100k-scale tiles/lines are inside the GPU-side design envelope; no public benchmark of 100k *entity* CPU extract/sync overhead found -> UNCERTAIN** (which is exactly the GIS/twin scene shape, so §4 should list a "100k entity extract frame budget" as a must-test risk of the bevy option). Line drawing: no evidence of a built-in wide-area polyline wireframe solution in bevy (gizmos are debug-only) — GIM/road-line data needs a self-built mesh-generation layer; whoever you choose you write it, so it is not a differentiator.

---

## Q6 C ABI / FFI precedents

**No project found shipping a bevy renderer behind a stable C ABI** (GitHub repo search bevy+language:C no valid hits; the bevy ffi/c-api/capi combos 0 hits; all 65 results of crates.io q=bevy ffi are audio/ECS bridges). The four closest all bypass rendering:
| project | shape | who renders | status (fetched 2026-09-03) |
|---|---|---|---|
| bytemeadow/godot-bevy 540★ | gdext in-process, `#[bevy_app]`, default-features=false | **Godot** | pushed 2026-09-02, active |
| KBVE uniti 18dl | csbindgen-generated C#<->Rust, game logic only | Unity | v0.1.0, 2026-05-05 |
| matthunz/bevy_mod_ffi 59dl | guest/host FFI utilities | — | v0.2.0, 2026-01-07, low adoption |
| bevy_python_ffi 2.9k dl | Python<->bevy app control | bevy itself (the window is still bevy's) | exists, but not a C-ABI SDK shape |

→ "The bevy renderer hidden behind `extern \"C\"`" **has no success precedent and no failure precedent — it is a complete blank**; stacked on Q3's "no first-party path to a host-window surface", the SDK-ization risk concentrates in an unvalidated combination.

---

## Q7 wasm / WebGPU (the 2026 state)

- Capability since 0.11 (the 2023-05-17 official announcement "Bevy + WebGPU", @cart: bevy natively runs WebGPU/wgpu, live examples shipped); `webgl2` is still in the 0.20-dev `default_platform` = **the WebGL2 backstop is maintained in parallel**. — https://bevy.org/news/bevy-webgpu/
- Browser baseline (MDN BCD `api/GPU`, repo pushed 2026-09-03 same-day snapshot): Chrome 113+, **Firefox 141+, Safari 26+** -> WebGPU is a stable feature in all three mainstream browsers.
- **wasm has no multithreading**: the tracking issue #4078 (opened 2022-03, 89 reactions) is **still open as of 2026-09** -> wasm builds run all ECS extract single-threaded, big-scene frame budgets feel direct pressure.
- 0.19 settled a half of the old web debt: "Cancellable Web Tasks" — on wasm Task drop did not cancel, silently leaking work (PR #21795, switching to the new web-task crate); the same version's breaking entry "Dropping Tasks in Web Builds". **Before 0.19 the web task semantics diverging from native was a fact of record**.
- Open question: #22545 open (2026-01-16) "Rapidly growing number of WebGPU objects causing stuttering" — the WebGPU backend still has an object-growth stutter on file.
- **0.20-dev is swapping the shader language**: #25088 "Migrate to WESL" closed **2026-08-05** (the main example shaders are already `.wesl`) — the next release's custom material/shader interface changes another round.

---

## Three hard conclusions directly usable in §4

1. **Stability**: 2–3 breaking minors per year, 120 breakages in one version, no 1.0 commitment, and lockstep ecosystem migrations have a real case (#8887 hard-block) -> a "long-term API stability promise" can only be built on **a self-made C ABI layer fully isolating bevy types**, and you must accept pinning + forgoing the plugin-upgrade dividend + non-prunable reflection.
2. **Main loop**: manual driving has an official example but it was born 2026-01 and was still fixing P0 panics in 2026-07; **rendering to a host-window surface has zero first-party/third-party precedent** (godot-bevy dodges the question via Godot's renderer) -> the "must not hijack the host main loop" clause is **yet to be invented** under the Bevy option and a default property under the in-house wgpu option.
3. **Capability surface**: dual-projection camera switching / custom phases / 65k->single-draw-call instancing are all officially example-backed (not a fight); size and the 100k-entity CPU overhead **lack data and are must-test items** — §4 should require a 2-hour spike (once the toolchain is available): the `bevy --no-default-features --features 3d` minimal app's release sizes on three platforms vs bare wgpu, plus the extract frame time at 100k entities.

No repository file was modified.
