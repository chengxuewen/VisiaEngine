# VisiaEngine Pitfalls & Gotchas

> Every PIT is the five-part form; symptom / root cause / solution / verification are all mandatory (consolidation duty per C9). Lessons repeated ≥2 times or costing >3 rounds of localization are promoted in sync into an executable rule in `rules/common/edit-safety.md`.

> Numbering note: PIT-27 and PIT-63 were never allocated (verified 2026-10-29); the gaps are intentional.

## Format example

## PIT-{n}: <one-line title> (YYYY-MM-DD)
- **Symptom**: <the observable, including raw error text / observable evidence>
- **Root cause**: <why it happened, not a restatement of the surface>
- **Solution**: <the correct approach, minimal executable steps>
- **Verification**: <a check command that nails the fix>

<!-- Real entries append below. Generic toolchain lessons (edit duplicate insertion, pkill self-kill, set -e traps) are already consolidated in rules/common/edit-safety.md — do not repeat them here. The predecessor MediaServo pitfall history is in the .refinfo archive. -->

## PIT-1: "Pre-set facts" in research prompts pollute deliverables — must be labeled as unverified assumptions (2026-09-03)
- **Symptom**: the ANGLE research prompt stated "wgpu has no native GL backend (verify)" — had an agent inherited it unchecked, the whole legacy plan would misjudge as "must self-build a GLES renderer" (a cost off by one backend's workload). In fact the wgpu v30 README: native GL/GLES backends + built-in ANGLE via `cfg(windows_angle)`. In the same period two self-written docs propagated "Qt6 uses ANGLE" as fact (later falsified: Qt6 RHI is native D3D11).
- **Root cause**: "my speculation" and "the point to verify" were mixed into one sentence; and industry assertions already committed to docs did not all carry same-day citations.
- **Solution**: in research briefs, every preset gets its own "⚠ assumption pending proof" line; forbidden as embedded fact clauses; in-document industry assertions must carry "snapshot date + source" or an *UNCERTAIN* marker (this repo's docs/reference template already contains this discipline; this time it was executed — all three errors were caught by the review line).
- **Verification**: before delivery, grep the docs for "Qt6.*ANGLE|no native GL"-style sentences against the falsification entries in `evidence/2026-09-03-angle-integration`; compare memo conclusions verbatim against the primary READMEs.

## PIT-2: .gitignore negation-order bug + check-ignore -q exit-code misleading (2026-09-03)
- **Symptom**: `!.omo/omo.jsonc` sat before `.omo/*`, so `git add .omo/omo.jsonc` reported "ignored"; and before the fix `git check-ignore -q` returned 0 for paths hitting a negation rule, easily misread as "still ignored / already ignored". The comment docs (AGENTS.md "only omo.jsonc is committed") were a long-standing unfunded promise, unnoticed.
- **Root cause**: gitignore semantics = later matching rules override earlier ones; the directory exclusion `dir/*` must be written **before** the negation `!dir/file`; `check-ignore` default mode's exit-code semantics for paths "matched by an excluding rule but contradicting the final ignore state" do not match intuition (the final matched line shown by -v is the source of truth).
- **Solution**: reorder to `.omo/*` → `!.omo/omo.jsonc`; validate with `git check-ignore -v` (shows the final hit line) and **whether the actual `git add` succeeds** — don't trust the `-q` exit code.
- **Verification**: the hit line of `git check-ignore -v .omo/omo.jsonc` must be the negation rule; `git ls-files .omo/` has output. Any newly added `!` negation rule gets a real add rehearsal before commit.

## PIT-3: wgpu v30 upgrade breakage trio — flags default / Color semantics / draw migration (2026-09-03, S3/S4 measured)
- **Symptom**: ① under the debug profile `cargo test` panics on lavapipe with `Unable to load cmd_begin_debug_utils_label_ext` (ash, panic-in-cannot-unwind); ② clear color passed as `0.05*255` expected dark blue, got pure white (255,255,255); ③ `RenderPassColorAttachment` compile errors for missing `depth_slice`, `InstanceDescriptor` has no `Default`, `enumerate_adapters` became async, `encoder.set_pipeline/draw` gone, `multiview`→`multiview_mask`, `create_slice`→`slice`, `map_async` changed to (mode, bounds, cb) callback style, `Maintain`→`PollType::wait_indefinitely()`. Also: pruning winit `default-features=false` accidentally removed `rwh_06` → `Window: HasWindowHandle` unmet, create_surface type error.
- **Root cause**: ① `InstanceFlags::default()=from_build_config()` — debug builds automatically include VALIDATION → force-load VK_EXT_debug_utils, which soft renderers / old loaders lack the symbol for; ② v30 `Color` component semantics 0-255→0-1 (out-of-range clamped to white); ③ v30's major-breakage alignment with the WebGPU spec (the anticipated cost of the quarterly-pin discipline).
- **Solution**: explicit `flags = InstanceFlags::empty()` in `create_instance` (diagnostic validation stays in a dedicated slice); pass clear colors in 0-1; winit features explicitly include `rwh_06`; API differences judged against **local registry sources** (`~/.cargo/registry/src/.../wgpu-30.0.1`, faster and truer than docs.rs).
- **Verification**: `pixi run ci` green + the three offscreen golden tests green on real hardware; **any wgpu version-upgrade day = first grep this entry + re-run the breakage checklist**; experimental calibration beats doc acceptance (the Color semantics was confirmed by experiment).

## PIT-4: cargo integration-test CWD = crate dir + Iter construction probing trips library debug_assert (2026-09-03, G1)
- **Symptom**: ① tests referencing repo-root `testdata/x.glb` by relative path all NotFound (running `pixi run cargo test` by hand the CWD = workspace root, and filtered unit runs likewise — but **cargo launches integration-test binaries with the crate root as CWD**, the two run modes differ in path semantics and produced false-green/false-red combinations); ② probing types in sequence with `Iter::<T>::new` (thought returning Option made it safe) panics outright in debug builds (library-internal `debug_assert_eq!(size_of::<T>(), accessor.size())` fires first).
- **Root cause**: ① cargo's run directory for tests/ is the package directory, not the executing shell's; ② beyond the Option-returning surface the library also carries a debug_assert — "can return an Option" ≠ "can be probed safely".
- **Solution**: ① fixture paths always joined from `env!("CARGO_MANIFEST_DIR")`; ② before reading an accessor, pre-check `data_type()/dimensions()` and dispatch exactly — mismatched types never enter the constructor.
- **Verification**: `grep -rn '"testdata/' crates/*/tests/` must return 0 hits (only via the fixture() helper); degradation chains may only be match-dispatched.
- **Also**: `mkdir -p` the tests/ directory redirect target first (third same-family omission this round — mechanical precondition, not after-the-fact repair).

## PIT-5: wgpu projection depth must be [0,1] — a GL-convention matrix silently culls the whole frame (2026-09-03, G2 golden)
- **Symptom**: HeadlessBackend drew a cube; the vp matrix verified within ±1 point by point, **zero verification errors**, the triangle old path stayed green — but the cube simply never appeared (golden asserted center = clear color). Only the binary diagnostic (hand-fed identity projection nearby) pinned the issue to the depth domain.
- **Root cause**: the wgpu clip-space depth is **[0,1]** (WebGPU's native convention), not GL's [-1,1]. Ortho/persp built by the GL formula produce negative z_ndc up close → hardware culls everything. A wrong depth range raises **no error anywhere in the pipeline** — geometry dies silently.
- **Solution**: build/verify all projection matrices against the [0,1] variant (glam's old `*_rh_gl` are [-1,1]; `perspective_rh`/hand-written ortho are the wgpu way); camera.rs's header comment now locks the variant declaration, and REND-11/12 use **behavioral assertions** (near→0/far→1) instead of copying formulas.
- **Verification**: the offscreen golden family's existence is itself prevention — after any matrix/pipeline change, `pixi run cargo test -p visiaengine-render-wgpu` cube tests must be truly green (not the SKIP path; verification location recorded honestly). Same family: `x+(y-x)*t` at t=1 is not exactly y (lerp endpoint fast path, REND-16); clippy's partial-order-negation ban resolved with the correct `partial_cmp`.

## PIT-6: MCP bridge's bare-PATH dependency + opencode config-key silent ignore (2026-09-03, MCP repair round)
- **Symptom**: three MCPs reported `Executable not found in $PATH: node/npx`; the local-github token was never injected (no error, purely silent).
- **Root cause**: ① the bridge config assumed node on PATH while this machine follows the D5 discipline and installs no system node — nodejs simply wasn't registered in the environment; ② the config key was written as `env` while the opencode schema (McpLocalConfig) names it `environment`; unknown keys raise no error, just ignored; ③ the bridge's child processes further need `npm`/`pixi` — launching node by absolute path doesn't save that layer.
- **Solution**: nodejs joined into the pixi default environment (single source unbroken); the `with-node.sh` wrapper uniformly injects the node directory + ~/.pixi/bin then exec-passes through; config key corrected.
- **Verification**: `bash .opencode/with-node.sh node --version` prints a version + bridge child survives the timeout; **any opencode config field change is validated against the https://opencode.ai/config.json schema, never from memory**; config changes need an opencode restart to hot-load (running sessions use the old config).

## PIT-7: cargo-deny audit fetches live over network = false-failure source of local network jitter (2026-09-11)
- **Symptom**: `pixi run ci` green then red intermittently; when red, all tests/lint/clippy were green — only the audit segment exited 1.
- **Root cause**: `cargo deny check` (advisories part) **git-fetches the RustSec advisory-db live, every run**; this machine's TLS egress to github.com (gnutls_handshake) fails intermittently → fetch failure = red, wholly unrelated to code/dependency changes.
- **Solution**: before treating "audit red" as real, `grep 'failed to fetch advisory'` in the log; network-shaped failures clear on re-run; only a real advisory prints RUSTSEC-ids. GitHub CI runners fetch reliably; on this machine the symptom wobbles with network conditions.
- **Verification**: `pixi run cargo deny check` must give consistent results across two consecutive runs before any dependency-surface conclusion; inconsistency = network layer.

## PIT-8: first-version T2 pixel assertions must fail — geometric coverage/channel-multiply chains unmeasured (2026-09-12, batch 4ab M2/M3 two rounds)
- **Symptom**: M2 texture_render first version: the sample window (8,8) all black (off the quad) + single-pixel checkerboard assertions ground to orange-gray by linear-filtering blend bands; M3 capi first version: the `B<20` predicate was a tautology-false (baseColorFactor's B=0.5 multiplied by the red texel → magenta) and with ±1 quad coverage at 21px, the red family's core had only 20 px.
- **Root cause**: at RED time the assertions were written by uv/projection intuition instead of first measuring "the quad's screen bbox" and "texel×base×shade channel products". A pixel test's predicate is the **intersection of three multiply chains** — any chain guessed wrong = false red.
- **Solution**: first run a throwaway dump probe (stepwise-print R/G values or bbox stats per region), derive predicate thresholds and windows from measured values; delete the probe, rewrite assertions as **region-family statistics** (count≥N, two families coexisting) rather than single pixels; size fixtures so texels ≥16px screen-wide (filter blend bands swallow purity cores <8px).
- **Verification**: before a new pixel assertion's first run, inspect the numeric band via `-- --nocapture`; the planning stage's [four-not-installed / honest surface] forbids masking broken chains by threshold tuning — predicates must keep discriminative power (the broken-uv chain → some family identically 0 invariant; M3's red=20/purple=304 is exactly the evidence shape of an unconnected chain).

## PIT-9: L2 smoke lanes not in the local ci chain = false-green window for window-format regressions (2026-09-14, 4c seven-lane first blowup)
- **Symptom**: three window examples, even after I3 surface-format alignment, still handed `render_view` directly (internally hardwired to Rgba8Unorm) while x11's actual config was Bgra8UnormSrgb → wgpu validation panic; the bug survived three T1 rounds of all-green (the ci chain excludes smoke).
- **Root cause**: the local gate = `pixi run ci` (fmt/lint/test/audit/gates); the smoke-* seven lanes = a manual segment — nobody ran them before their mirror-day activation; the "six smoke ✓" baseline claim = a snapshot of the last manual run date, not a standing assertion.
- **Solution**: ① any round touching render paths must run all seven lanes pre-merge (DISPLAY=:0 directly — don't wrap in timeout; timeout doesn't resolve the shell-function/alias version of pixi; use absolute paths or run bare); ② example/attach consumption formats pass `config.format` straight through to render_view_format — never route through hardwired convenience ports.
- **Verification**: `for t in smoke-*; do pixi run $t; done` (true execution ≠ compilation, the E3D:D2 discipline); after mirror-day ci.yml L2 wiring, clause ① becomes a machine gate.

**Source**: first blowup found by the full re-run at 4c K3's smoke-bench-tune hookup.

## PIT-10: scene human-check "the foreground should be lit ground" = geometric assumption (2026-09-14, batch 5 E901)
- **Symptom**: E901 twin_city's first version had the entire foreground in half-penumbra (obs 0.58-0.63 vs expected 0.86), diagnosed as a shadow-math bug → three probe rounds (dir/slope/None controls) all said "the parameters aren't wrong".
- **Root cause**: the scene is legal — 13m buildings × low sun (ld horizontal component 0.5/0.79≈0.63) → shadow length ≈10m; the shadows of a dense 16×16 grid meet head-to-tail, so **the foreground simply is the shadowed zone**. "Lit ground belongs here" was screen intuition, not a light-geometry derivation.
- **Solution**: when pixel assertions mismatch, do the light-source geometry computation first (shadow length = Σh·|l.xy|/l.z against occupancy) before parameter bisection; scene-demo shadows use **family counts** (here dark = shadow-ground + building-dark-face families), no positional point assertions.
- **Verification**: every new render example's human-check image gets a `look_at`-style visual re-review (this time it surfaced the real defect = the base map suppressed by the city → fixed with a right offset); passing pure numeric assertions ≠ the picture is right.

## PIT-11: judging an API ledger by name without reading the implementation = a whole plan premise collapses (2026-09-14, Qt round v1.0 REJECT)
- **Symptom**: the plan asserted "14 entry points, zero resize channel, need a 15th symbol CAPI-10"; the Momus review read the implementation and overturned it — `visiaengine_viewport(w,h)` (the 14th entry) IS the full resize channel (ffi.rs:441 rejects zero dims → engine.rs:405 w/h + backend.resize + Window sw.resize). Working from the false premise = one redundant symbol + three-lock rework.
- **Root cause**: evidence-gathering grepped only the **entry-point name set**; `viewport` was read literally as "viewport query" — name ≠ meaning; building a capability ledger correctly = reading the implementation's call chain, not function names.
- **Solution**: any "capability missing / new entry needed" assertion must first grep the capability's **verb implementations** (resize/reconfigure/set_size words) + read one call chain from entry to effect before concluding.
- **Verification**: plan §0 anchors must carry file:line down to implementation bodies (not just symbol tables); Momus's accountable review being called out for "citations measured" is exactly this entry's machine surface.

## PIT-12: conda-forge Qt6 twin pits — package-name orthography and QX11Application absence (2026-09-14, Qt round Q2)
- **Symptom**: ① installing `qt-main` yields **Qt5** (5.15.15, lib/cmake only has Qt5*); ② after switching to `qt6-main`, Qt6Config/xcb plugins are all present, but the `<QNativeInterface/QX11Application>` header **does not exist** (x11extras not packaged) — the public-API route for widgets to obtain Display* is beheaded.
- **Root cause**: conda-forge Qt package names split historically (qt-main = the Qt5 legacy name; Qt6 = the qt6-main prefix form); second case of the "pixi search fuzzy hit" package-name pit family (first case xorg-xvfb-server). QX11Application belongs to the qtx11extras module, which qt6-main's pruning excludes.
- **Solution**: the host self-holds its Xlib `Display*` (the XOpenDisplay/XSync/CloseDisplay three-call surface) + the winId() xid **shared across X connections** (the attach contract, demo_x11 proved it same-machine); pin QT_QPA_PLATFORM_PLUGIN_PATH explicitly to `$CONDA_PREFIX/lib/qt6/plugins`.
- **Verification**: `ls $CONDA_PREFIX/lib/cmake/Qt6/Qt6Config.cmake && find $CONDA_PREFIX/include -name QX11Application` (the latter should be empty → triggers the Xlib route); `pixi list -e qt-spike | grep qt6-main` confirms it is not qt-main.

## PIT-13: handwritten C ABI header missing extern "C" guard — all-green C tests mask certain C++ breakage (2026-09-14, CMake layer C2)
- **Symptom**: `visiaengine.h` had zero `extern "C"` wrapping; the C demos (E701/702) and the 14-entry nm gate were green for three rounds; the C++ consumer (ctest probe consume_capi.cpp) failed its first link with `undefined reference to 'visiaengine_abi_version()'` (C++ mangling).
- **Root cause**: structural bias between the consumer test surface and the ABI declaration language — a C header tested only by C = self-proving circle; C++/Rust FFI is the real host-language distribution (Qt/Flutter/C# all compile through C++ TUs).
- **Solution**: add the `#ifdef __cplusplus extern "C" {` guard at the top of handwritten headers; the consumer probe matrix must include **≥1 C++ TU** (here = the cmake/probe ctest resident); the acceptance anchor for new C ABI surfaces upgrades from "the C demo runs" to "C+C++ both run".
- **Verification**: `grep -c 'extern "C"' crates/visiaengine-capi/include/visiaengine.h` ≥2; `pixi run ctest --test-dir target/cmake-bare` 1/1.

## PIT-14: environment-state pollutes gate assertions — the activated-shell PATH makes "forced modes" pass falsely + wrapped report lines defeat whole-sentence greps (2026-09-14, C1 review/C3)
- **Symptom**: ① `-DRUST_SDK=SYSTEM` inside a pixi-activated shell let find_program hit the conda tree's cargo, judging [system] and passing — the "force system" semantics cannot be true in that environment, yet raised no error; ② the escape-hatch FATAL assertion grep of the whole sentence "no capi artifact" missed — cmake wrapped the message across two lines "no capi\nartifact".
- **Root cause**: ① the resolver's lookup domain = the current environment, and the environment's PATH/PREFIX is polluted by activation state — "force X" without explicitly excluding non-X sources is nominal only; ② terminal/log line-wrapping is destructive for whole-string matching.
- **Solution**: ① forced modes get **dual anti-pollution filters** (prefix comparison of find_program results against CONDA_PREFIX/.pixi, same check for Qt6_DIR; a hit = reject with the message naming it); ② gate assertions use **wrap-stable short phrases** ("no capi" class), and negative-path assertions accept both rc and message conditions.
- **Verification**: `pixi run bash scripts/cmake-smoke.sh` (both negative-path messages on the assertion column); the SYSTEM rejection message must appear in the local activated state.

## PIT-15: enable_testing() after add_subdirectory = that directory's add_test silently dropped (2026-09-15, S4 real-machine proof)
- **Symptom**: platform/qt's example_E703 add_test, once registered, never appeared in `ctest -N`, and platform/qt/CTestTestfile.cmake was never produced; zero errors, zero hints.
- **Root cause**: the root's enable_testing() sat after add_subdirectory(platform/qt) (the C2 probe happened to be after it so the issue stayed hidden); CMake warns nothing about this ordering problem.
- **Solution**: enable_testing() must precede **all** add_subdirectory calls (moved up + nailed with a comment).
- **Verification**: `ctest --test-dir target/qt-build -N | grep -c "Test #"` == registration count (13); `ls target/qt-build/platform/qt/CTestTestfile.cmake` exists.

## PIT-16: two "scope/shape" pits — enable_language inside function() doesn't propagate; preset fields defer to the shipped schema (2026-09-15)
- **Symptom**: ① after enable_language(C) inside function(), the function's add_executable(.c) reported "can not determine linker language"; ② four consecutive guesses of the CMakePresets testPreset label-filter field were all rejected (excludeLabels/labelExclude/exclude.label array/filter.exclude.labels).
- **Root cause**: ① enable_language's compiler variables land in the function scope — lost on return — **language enablement must happen at file/directory scope**; ② presets v6 has no label filtering; the v7 correct shape = `filter.exclude.label` (a **string regex**); memory and search are both unreliable.
- **Solution**: ① hoist enable_language(C) to the .cmake file top level under the if(VISIAENGINE_EXAMPLES) block; ② read the offline canonical source `$CMAKE_HOME/share/cmake-*/Help/manual/presets/schema.yaml` (or exclude-properties.rst beside it); with egress down (context7/webfetch both failed) this path costs 0.
- **Verification**: configure passes + `ctest --preset qt-pixi` count = 7 after excluding display; grep schema.yaml for "``label``".

## PIT-17: cargo harness threads share the pid — tmp-path uniqueness needs pid+nanoseconds (2026-09-15, S1 flicker)
- **Symptom**: attr_three_types FAILED on first run, passed when run alone/re-run (flaky appearance); 3 tests with the same tmp prefix ran in parallel.
- **Root cause**: `format!("ve-attr-{tag}-{}", process::id())` is identical across all threads **within one test binary** → multithreaded parallel `fs::write` (truncate-then-write is not atomic) trample each other and readers see half-empty files.
- **Solution**: uniqueness = pid + SystemTime nanoseconds dual-suffixed (same form in both the engine inline and attr_ffi_spec); or a per-test dedicated tag.
- **Verification**: `for i in $(seq 6); do cargo test -p visiaengine-capi | grep -c 'test result: ok'; done` all equal (6×6 measured green).

## PIT-18: validating through `pixi run` wrapping ≠ validating the user surface — the IDE's direct build call blew up on rustc-missing (2026-09-15, user-proven)
- **Symptom**: everything verified green "13/13" via `pixi run cmake --build` + ctest; then the user's VS Code (CMake Tools calling /usr/bin/cmake directly, no pixi PATH) built E301 → `error: could not execute process rustc -vV (never executed)`.
- **Root cause**: conda cargo finds its sibling rustc via PATH; `pixi run` activation happens to put rustc into PATH = every step green. The runtime environment of build-time COMMANDs and the configure-time resolution are two different things — **when the verification channel coincides with the proven channel, all environment-difference blind spots are masked** (the build-surface projection of verification-honesty; the inverse twin of the old "un-activated shell directly calling conda cargo, false rustc-missing" case).
- **Solution**: prefix every cargo call site with `${CMAKE_COMMAND} -E env "PATH=<cargo's directory>:$ENV{PATH}"` (the cargo-step + examples-step; SYSTEM's true-system pieces are harmless); the IDE user surface = the bare environment, so every build gate must be retested on bare PATH.
- **Verification**: `env PATH=/usr/bin:/bin cmake --build target/cmake-bare --target visiaengine-examples-step` passes; now hardened as cmake-smoke's fourth-state dual target (guard regression lock).

## PIT-19: pixi file clobber makes the conda-built Xvfb always die on XKB (2026-09-16, S0 spike proof)
- **Symptom**: after `pixi add`, starting `.pixi/envs/default/bin/Xvfb :77 …` in the default environment immediately fails with
  `XKB: Failed to compile keymap` → `Failed to activate virtual core keyboard: 2`;
  the `share/X11/xkb/` directory keeps only `compiled/`; rules/keycodes/symbols all vanished.
- **Root cause**: the `xorg-xvfb-server` package ships `share/X11/xkb/compiled/.keep` conflicting with `xkeyboard-config`'s tree paths → pixi 0.78's clobber handling **redirects the whole xkeyboard-config tree** to
  `share/xkeyboard-config-2/` (conda-meta shows the `__clobbers__/xkeyboard-config/share/X11/xkb` marker), and Xvfb looks for its keymaps at the compile-time canonical location. Same in the host-spike environment; historically undetected = local smoke-x11 always used the :0 branch.
- **Solution**: the environment is this machine's state; after rebuild, replay the merge once:
  `cp -a .pixi/envs/<env>/share/xkeyboard-config-2/. .pixi/envs/<env>/share/X11/xkb/`.
  Display-test sub-state priority = system `command -v Xvfb` (apt CI immune) → conda fallback (must carry the tree-replay fix) → neither = SKIP note.
- **Verification**: `Xvfb :77 -screen 0 1024x768x24` alive after 2s checked by `kill -0`; the SDL3×attach spike green under `DISPLAY=:77`.
  Anti-recurrence check: `ls .pixi/envs/default/share/X11/xkb | grep -q rules || cp -a …xkeyboard-config-2/. …X11/xkb/`.

## PIT-20: the "ghost API" family — interface/parameter names written from memory, ≥3 cases (established 2026-09-16)
- **Symptom**: plans/code cited nonexistent external interfaces: `corrosion_add_test_crate` (upstream #13, never implemented since 2018), the `CORROSION_CARGO` user cache (removed since 0.3), `crates_only` (real name CRATES), "conda has no corrosion package" (it has 0.6.1) — each time the execution hit the wall or silently detoured.
- **Root cause**: treating "I remember it exists" as "I verified the name".
- **Solution**: every external API handle (function/parameter/package name/file path) gets its canonical source captured at writing time (docs/source grep/API actual test), cited with a verbatim fragment; no anchor = marked [UNCERTAIN] and not allowed into the plan body.
- **Verification**: this lesson's source automates itself: context7/source/`pixi search` triple-checks before planning already rescued 4 errors across two Momus rounds; residual errors = sentences that went unchecked.

## PIT-21: edit-tool insert-style rewrites leave "old-new coexisting" residue + grep assertions' BRE false greens (2026-09-16, quadruple proof)
- **Symptom**: a batch replace in one file consumed only the first anchor line, so the replaced block's "second old line" survived → zombie lines ×4 (IDE terminal-state double line / gate double line / swallowed if-line leaving a dangling || / double example anchors); same family: `grep 'a|b'` without -E kills the alternation = always-zero matches = a "zero-out assertion" permanently true.
- **Root cause**: insert-style rewrites without a declared end range; assertion predicates using bare | in BRE.
- **Solution**: ① insert-style rewrites must consume the whole old range via `pos+end`, then `sort|uniq -d` for duplicates + old-name grep=0; ② alternation assertions always `grep -E`/`grep -c`, first prove "can hit a positive", then assert zero; ③ the guard's "fix it if it errors later" phrasing = conditional-branch suspense — switch to "pre-set at birth" to kill the branch.
- **Verification**: `bash -n` + rerun with identical parameters in the same band; S3/S4 each hit once, all closed in-band (nothing overnight = passing; the lesson is pattern-hardened in edit-safety #18).

## PIT-22: "verification channel ≠ user channel" environment blind spot, second sighting — the DISPLAY family (2026-09-16, user-proven)
- **Symptom**: all real-window verifications (S0 spike / smoke-qt / cargo-run spot checks) ran green in the agent's tool shell with `export DISPLAY=:0`; when the user clicked cargo-run_* inside VS Code (the vscode-server process tree), winit reported `neither WAYLAND_DISPLAY nor DISPLAY is set`, rc=1.
- **Root cause**: the generalization of PIT-18 — the tool shell and the user's IDE shell are two environments; a manual export = secretly adding to the verification channel supplies the user doesn't have (the PATH family back then was conda activation; this time a display variable). Window-class assertions' green counts only when taken in the **user's environment form**.
- **Solution**: ① environment-sensitive steps (GUI/clipboard/audio/network-proxy surfaces) always go through "launcher wrapper + probe fallback + stderr announcement of which path was taken" (the run-gui.sh form); ② before delivering window features, re-run once in the **un-spiked** shell (`env -u DISPLAY` / `env PATH=/usr/bin:/bin`); ③ the IDE-side promise surface (test panel/Run buttons) is T3 — if machine-unreachable, say so plainly and wait for human verification.
- **Verification**: `env -u DISPLAY -u WAYLAND_DISPLAY cmake --build target/cmake-bare --target cargo-run_E501_shadow_demo` survives to timeout(124) = the fallback chain works; bare cargo run (unwrapped) must still error clearly = the sentinel surface not broken.

## PIT-23: three "tautology/swallow" shapes in CMake/ctest parameters and assertions (2026-09-16, the E801 hang case)
- **Symptom**: ① a `cmake_parse_arguments` oneValue keyword receiving `"--frames;3"` took only `--frames` (the list flattened to two elements in ${ARGN}; the single-value port ate the first and dropped the rest) → the ctest example fell into interactive mode and cmake-smoke hung 15 minutes; ② `ctest -R <no-match>` prints "No tests were found" and **exits 0**, keeping forwarding shells always green; ③ (PIT-21 already recorded the bare `|` BRE always-false form; mutual same-family reference).
- **Root cause**: the common pattern of the three shapes = "looks like it's passing/asserting, actually silently dropping/always passing" — CMake list semantics and ctest's empty-set philosophy both run counter to intuition.
- **Solution**: ① all list parameters go through the multivalue port (`cmake_parse_arguments(prefix options oneValue multiValue)`, the fourth slot); generated files are reviewed against what `CTestTestfile.cmake` actually does; ② ctest wrappers must carry a "≥1 selected" assertion (`--show-only` pre-count or grepping the output `out of [1-9]`, the smoke-rs.sh precedent) + `--timeout` always mounted (hang = red, not a black hole); ③ any "=0 means green" gate first self-proves with one positive hit / one negative trigger.
- **Verification**: `grep -o 'E801_sdl_window" "[^)]*' target/*/examples/c/CTestTestfile.cmake` shows both `"--frames" "3"` positions; injecting a nonexistent -R forwarded once must go red.

## PIT-24: winding dumb-mine in shared geometry — cull-off makes winding a dark observation read only by picking (2026-09-17, the E402 band)
- **Symptom**: `unit_box_mesh`'s three consumers (instances/bench/twin_city) rendered perfectly, goldens all green;
  the moment the E402 pick chain first consumed it, the center ray always missed — a straight-down ray missing the box top, a bottom-up ray hitting falsely.
- **Root cause**: the birth winding was **globally reversed** against the declared normal (geometry cross = −nrm). The GPU path left PrimitiveState cull unset (wgpu default None) → both faces drawn → zero render symptoms; `pick_meshes`' front-face rule (REND-23) reads exactly the winding — the same geometry means opposite things on the two consumption paths. Anything that "reuses a render piece into picking / gpu-skinning / a cull-enabled pipeline" is this mine's fuse.
- **Solution**: index-mirror correction `[b,b+1,b+2][b,b+2,b+3]`→`[b,b+2,b+1][b,b+3,b+2]` (one edit, every consumer benefits; pixel-identical under cull-off = constructive proof, 162 passed zero-regression, hard evidence). Shared geometry definitions must state the **winding = outward-normal CCW** contract (now written into the golden training notes).
- **Verification**: any new shared geometry, at birth, runs a "front ray must hit / back ray must not" bidirectional assertion (E402's gate in the corner-canary form); `grep -n 'CCW' crates/visiaengine-render-wgpu/src/offscreen.rs` keeps the contract line present.

## PIT-25: toolchain silent disconnection — package-name drift + absent-continue tautology (2026-09-17, dug out in the point-cloud band G3)
- **Symptom**: `bash scripts/bench.sh` perpetually printed `BENCH ✓/exit 0`, but resources/bench/*.json had not updated since the S1 migration across two rounds (timestamps stuck at 9-14).
- **Root cause**: double silence — ① the spec's `-p visiaengine-render-wgpu --example E601…` errored in cargo after the examples moved to `examples/rs` (package name examples); ② the script only `continue`d on "no RESULT line" without setting an error; the pipeline + `|| true` swallowed rc (same family as edit-safety #19) → the dead chain was masked by an always-true green.
- **Solution**: package name fixed to `-p examples`; absent-counting `MISSING>0 → exit 1` (toolchain faults separated from performance regressions: the latter prints red text but keeps exit 0 as the observation tier); the new bench_pcl joined the same chain.
- **Verification**: `bash scripts/bench.sh` with all three json timestamps same-day + deliberately `sed` the package name wrong once, must exit 1 (positive/negative self-proof, the PIT-21 discipline).

## PIT-26: from_raw_parts(_mut)(NULL, 0) is UB — the debug precheck nails it (2026-09-17)
- **Symptom**: capi get_clips' `buf=NULL` pure-count path panicked in debug tests with "unsafe precondition(s) violated: slice::from_raw_parts_mut requires … non-null" (silent in release).
- **Root cause**: recent Rust core versions added debug prechecks to from_raw_parts (null/alignment/isize::MAX); `m=0` still counts as an illegal pointer source for constructing an empty slice. The existing add_points/add_mesh didn't trip it because the NULL check returns early **before** slice construction.
- **Solution**: guard `if m > 0 {}` before construction (sidesteps both NULL∧any length and any∧0 length); or is_null early-return, then construct.
- **Verification**: the NULL-buf segment of the ffi_spec `set_get_clips_full_domain_roundtrip_truncation` + `cargo test -p visiaengine-capi` (debug tier blows on regression).
- **Forbidden**: any FFI port unconditionally calling from_raw_parts on a possibly-NULL buf (even with length 0).

## PIT-28: label centering's px offset may only enter the dx domain — the anchor is world coordinates (2026-09-18)
- **Symptom**: in E205 the three labels' gold sub-line never rendered (hist had no yellow tones); white/cyan present but offset abnormally.
- **Root cause**: `anchor_x = cx - pen/2` subtracted the **pixel** pen from the **world** anchor (-4−50 world meters = off-screen). The canonical capi `add_label` form puts all center alignment in the dx(px) component: `dx = top_left_px - pen/2`, anchor keeps its world position untouched.
- **Solution**: the anchor takes world coordinates only; all px semantics (including alignment offsets) live in the metrics/dx domain (vs uniformly multiplied by px_scale).
- **Verification**: triple cross-check (capi engine.rs / hpp forwarding / the E205 example) — grep `pen / 2.0` must appear only inside the LabelMark's 4th parameter group; appearing in the 1st group = red.

## PIT-29: orbit constructed with degrees = perspective eating radians — three examples shipped with the bug (2026-09-18)
- **Symptom**: E303 top view OK but the main view's ground occupied only 51%/composition odd; E302 headless bright=96 (barely clearing the 40 threshold); E501's window surface carried the same mine (nobody looked closely).
- **Root cause**: `CameraRig::perspective`/look_at default `FRAC_PI_3` = **radian** convention; `orbit(...)` call sites once passed 46/55/60 (as degrees) → tan(23 rad) negative values flip/crush the projection. lavapipe reports no error = silently broken picture.
- **Solution**: all radians (0.802_851=46°, FRAC_PI_3=60°, 0.959_931=55°); the `orbit` doc nails the unit and records the priors. Post-fix: E302 bright 96→1987, E501 dark=19642 exact, E303 composition valid.
- **Verification**: grep the example surface for `orbit\(.*[0-9]\.[0-9], [0-9]{2}\.0` — degree-magnitude values (fov slots >7) = red; window-example pixel gates (the E302/E303 headless assertion family) run green.
- **Lesson**: a window surface the pixel gate never looked at is an unverified surface — when T3 human verification is unreachable, headless structural assertions must cover the window example's same frame shape (this band's double-insurance precedent).

## PIT-88: an ortho camera's only zoom definition = half-width; the dist field is decoration (2026-09-24, E206 third fix, second sighting) — renumbered from a duplicate PIT-29, then to PIT-88 after a collision with the Trae launcher PIT-86 (both 2026-10-29)
- **Symptom**: wheel zoom "does nothing" — both rig.zoom and rig.dist updated while the picture didn't budge.
- **Root cause**: dual stacking. ① `look_at()` constructs with default zoom=1.0 while the rendered ortho half-width consumed a hardcoded tile_w constant (rig.zoom was never read); ② under orthographic projection dist does not affect picture size (only perspective gets "farther = smaller") — changing dist = no-op.
- **Solution**: wheel → rig.zoom (the only ortho zoom semantics); the rendered half-width consumes rig.zoom; the clamp domain recomputed to the scene's scale (a 3857-meter world = 8km..500km, not E204's meter-scale 2..60).
- **Verification**: `grep -n 'rig.zoom' examples/rs/E206_tile_viewer.rs` (consumption point present); the wheel actually changes the picture.
- **Forbidden**: ortho examples applying zoom/zoom_scale onto dist; assuming zoom is a usable view width after look_at construction.

## PIT-30: cutting the window/interaction surface of window examples because "interaction = later Phase" = the C15 dual-mode breach trio (2026-09-24, E206, three measured cases)
- **Symptom**: user ran cargo-run_E206: ① exited immediately with no args (the window was a println stub) → ② drag/wheel did nothing (Redraw rebuilt the camera from hardcoded constants each frame; the fields events mutate aren't in the struct / aren't consumed) → ③ single-tile content too sparse ("minimally runnable" ≠ "demoable").
- **Root cause**: the delivery band treated "static/minimal" as the closing standard, while the dual-mode discipline (C15)'s zero-arg window = **resident + interactive** is an independent acceptance surface — smoke/ctest green cannot save red human verification; and "connect in Phase 1" was used as license to cut implementations instead of a phasing plan.
- **Solution**: window examples born with a real window + real interaction (copy the E204 skeleton, ~150 lines); the interactive camera = a persistent TileApp rig field (Redraw consumes self.rig; per-frame rebuilding forbidden); demo content to the "demoable" standard (3×3 neighborhood + tile-seam outlines), not the "runnable" standard (single tile).
- **Verification**: an Xvfb liveness probe (killed by timeout 22s = resident) + human check that drag/wheel actually take effect.
- **Forbidden**: a window example's no-arg path printing println then Ok(()); constructing a hardcoded rig inside the Redraw path overwriting interaction state; writing interaction promises into comments as delivered.

## PIT-31: ffi_spec integration tests' cwd = the crate directory (2026-09-24)
- **Symptom**: the capi's new port load_mvt_dir integration test returned -1 (VE_ERR_ARG) where 9 was expected; the same test was also red locally via `cargo test -p visiaengine-capi --test ffi_spec`.
- **Root cause**: when cargo runs integration tests (the tests/ directory), CWD = `bindings/c/visiaengine-capi/` (crate root), so the relative path `resources/data/tiles` resolves to `bindings/c/visiaengine-capi/resources/data/tiles` (nonexistent). FileSource path parameters are runtime strings, not compile-time macros.
- **Solution**: inside tests, climb from `env!("CARGO_MANIFEST_DIR")` to the repo root uniformly: `let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR")); while !p.join("Cargo.lock").exists() { p.pop(); }` (this pattern already exists in ffi_spec.rs — copy it).
- **Verification**: `cargo test -p visiaengine-capi --test ffi_spec` 19/19 passed.
- **Forbidden**: relative paths `resources/...` in tests pointing at repo-root fixtures (cwd is indeterminate).

## PIT-32: ureq's full dependency tree drags getrandom onto the wasm32 target (2026-09-24)
- **Symptom**: after adding `ureq` (a native-only HTTP source) to io-tiles, compilation for wasm32-unknown-unknown failed: `the wasm*-unknown-unknown targets are not supported by getrandom, you may need to enable the "js" feature` — the wasm surface shouldn't have ureq at all, but an unconditional Cargo.toml dependency = wasm compiles it too.
- **Root cause**: ureq → rustls → ring → getrandom. getrandom 0.2/0.3 on wasm32-unknown-unknown needs an explicit `js` feature; the rustls tree doesn't enable it, io-tiles' wasm consumers neither → the compile_error! macro fires at compile time.
- **Solution**: `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` quarantines ureq to the native section; wasm gets a typed stub for `HttpSource` (`load()` returns `Err(Unsupported)`). Real wasm HTTP needs host JS fetch + a bytes-injection port (Phase 2 wasm demo band).
- **Verification**: `pixi run web-check rc=0` (MIRROR 3/3).
- **Forbidden**: adding ungated dependencies containing getrandom/ring/rustls to any crate visible to wasm.

## PIT-33: depth lifecycle after a full-screen pass and wgpu30's use-domain rules (2026-09-28, the EDL band)
- **Symptom**: EDL post-processing needed to sample the main pass's color+depth inside the same encoder; using the swapchain view directly as a sample source (no TEXTURE_BINDING) or "writing color while sampling color" within one pass blew up.
- **Root cause**: wgpu 30's use-domain rules: ① an attachment and RESOURCE in the same pass are mutually exclusive (color as both write target and sample source is illegal); ② swapchain views lack TEXTURE_BINDING; ③ depth defaults to StoreOp::Discard so later passes can't read it.
- **Solution**: an intermediate-texture cache (key = (w,h,format), rebuilt on resize) takes the main pass's output + depth switched to Store; the EDL pass stands alone (samples color+depth, writes the final view); the OFF path skips the whole segment = bit-for-bit zero regression. The multi-viewport path was exempted this band (recorded under clause WGPU-33).
- **Verification**: `cargo test -p visiaengine-render-wgpu --test edl` (OFF re-render byte-identical + ON pixel delta + OFF-after-ON equal again).

## PIT-34: LRU/state double ledgers desync — state() lies "Done" while the bytes were already evicted (2026-09-28, revision #3)
- **Symptom**: pump() marked Done after cache.insert; after LRU evicted the bytes, state() still reported Done → decoded() None → the batch-rebuild path silently continued = a permanent rendering hole, and begin() skips Done and never re-arms.
- **Root cause**: two ledgers (the pump_state map + the LRU bytes) with no consistency maintenance; Done is a stable state and nobody re-checks whether the bytes still exist.
- **Solution**: an explicit `evict_done(ids)` (clears all three ledgers: state + bytes + decoded) + a read-only probe `decoded_cached(id)`; the consumption loop (the engine render loop) reconciles every frame on `state()==Done && !decoded_cached()` → evict_done, re-arm. Single-source-of-truth principle: when you find a double ledger, pair it with a reconciliation function.
- **Verification**: `cargo test -p visiaengine-io-tiles --test tile_pump lru_eviction`; the engine side is consumed by the batch-rebuild path (decoded==None must not render).

## PIT-35: env proxies also eat loopback at the library layer — ureq honors http_proxy and 502s 127.0.0.1 (2026-09-28)
- **Symptom**: E206 --source pumped 0/9 Done; the same URL got 200 direct with curl, 502 through the env proxy (10.144.0.1:7897).
- **Root cause**: the existing lesson only covered curl (edit-safety's "curl must use --noproxy"); HTTP client libraries (ureq 3.x) default Proxy::try_from_env and suffer the same — fixing the caller's curl can't save the library.
- **Solution**: **fix at the source**: inside HttpSource::load, `uri_host_is_loopback(root)` → proxy=None, else try_from_env; the 5s global timeout set at the same point. Library-layer network code always ships loopback-no-proxy + timeout; host environments carry no blame.
- **Verification**: `cargo run --example E206_tile_viewer -p examples -- --source http://127.0.0.1:8932/data/tiles` (against a python http.server) → 9/9 Done.

## PIT-36: python module-level definition order — TEMPLATE not yet defined when main() runs (2026-09-28, the gallery band)
- **Symptom**: in gen_gallery.py the DETAIL_PAGE template appended at the bottom sits after `if __name__ == "__main__":`, and pixi run gallery throws NameError.
- **Root cause**: append-style editing put the constant behind the entry guard; modules execute top-down, so TEMPLATE was unbound when main() called it.
- **Solution**: constants/templates always above the entry guard; after append edits, first run `python3 -c "import ast; ast.parse(open(f).read())"`, then run.
- **Verification**: `python3 -c "import ast; ast.parse(open('scripts/gen_gallery.py').read())" && pixi run gallery`

## PIT-37: example refactor dropped origin — static-table refactors silently zero per-marker data (2026-09-28, the E305 window surface)
- **Symptom**: after the E305 window-surface refactor (build table + group separation), the cyan pixel gate read 2 vs threshold 20 — all 24 trajectory markers drew at the origin.
- **Root cause**: in build(), `let _o = anim_origin(...)` computed the origin but the table stored only the (mesh,material) pair; group() uniformly set origin:[0;3] — the data chain broke between "computed" and "used".
- **Solution**: tables carry data (`Vec<((id,id), origin)>`); group() consumes per item; **the pixel gate caught it on the spot** (exactly why gates exist — the Nth cashing-in).
- **Verification**: `cargo run --example E305_entity_anim -p examples -- --frames 24` (the cyan>20 assertion).

## PIT-38: an if inside a winit match arm = the collapsible_match clippy rejection — use a match guard (2026-09-28)
- **Symptom**: the window example's Esc handling `WindowEvent::KeyboardInput{..} => { if pressed && key==Esc {exit} }` triggered clippy collapsible_match — three reds in a row.
- **Root cause**: an if as the first statement of a match arm has the merge form = match guard (`arm if cond =>`); clippy 1.98 suggests it for readability and rejects outright.
- **Solution**: copy the guard form (`KeyboardInput{..} if state==Pressed && key==Escape => exit`) + keep the `_ => {}` catch-all arm. Attached: winit examples **must always ship the full imports list when created fresh** (ApplicationHandler/WindowEvent/EventLoop/Window/WindowId — E304/E305 each missed one once, two rounds wasted).
- **Verification**: `cargo clippy --example <name> -p examples --all-targets -- -D warnings`.

## PIT-39: window examples' "first frame" had no machine guard — three black screens, all caught by hand (2026-09-29, the E305/E508/E505 family)
- **Symptom**: three window examples successively black/empty (E305 no loop, E508 window with no keys, E505 no redraw) while cargo test/ctest/ci all stayed green — every one was caught only by user human verification.
- **Root cause**: the machine surface (the assert path) and the human surface (the window path) have completely different lifecycles — nothing in CI asserts "does the swapchain have a first frame" for window examples; Wait mode + not requesting redraw on resumed = permanently black.
- **Solution**: scripts/window-probe.sh (run the example with zero args → Xvfb screenshot → brightness-ratio assertion) + window-probe-all.sh batch (the 20 resident window examples) + **screenshot before killing the app** (the root screenshot depends on the window being alive — the first version's inverted order captured full black). Self-exiting examples skip (the goldens guard those).
- **Verification**: `bash scripts/window-probe-all.sh` (all 20 ✓); breakage probe executed (commenting out the first-frame request → red rc=1).

## PIT-40: registry disp/args must come in pairs — two half-missing cases (2026-09-28/29)
- **Symptom**: E403 missed _args (a display example staying resident with zero args → ctest hung 10 minutes); E508 missed _args (a headless example mis-registered → died on DISPLAY); both only surfaced under ci/ctest.
- **Root cause**: in the cmake registry, _disp_X ON and _args_X "--frames;N" are one semantic pair (disp = window form, args = automation form); registering half = form mismatch on the example surface.
- **Solution**: new-example registration writes both lines in the same commit; verification = `ctest -LE display -N | grep <name>` selected, and `--frames` present in argv (a smoke run passing proves it). Gate-ification candidate: extend R9's bidirectional check to args.
- **Verification**: `grep -A2 "_disp_${NAME}" cmake/VisiaEngineBindings.cmake | grep _args` (the pair is present).

## PIT-41: multi-block patch scripts = a truncation minefield — four hits this round (2026-09-30, the io-gltf wipeout)
- **Symptom**: when a multi-patch python script died mid-way on assert/NameError, the already-written content varied (some blocks landed, some didn't); worst case io-gltf lib.rs emptied to 0 bytes (the open(p,'w') truncation moment hit exactly).
- **Root cause**: "read→replace→write" mixed with "write at the end"; multiple patches in one script sharing an outer `s` variable die half-new-half-old with the state indeterminate; `open(p,'w')` truncates an instant before write but when the variable chain breaks the outcome is unpredictable.
- **Solution**: ① each patch function body does its own independent read→write (no shared outer s); ② replace results stored in `ns` before the write; ③ changes spanning more than 2 blocks are **split into multiple tool calls** (1-2 patches each); ④ grep immediately after any patch to verify the key field is present.
- **Verification**: `wc -l` before/after comparison + `grep -c` key anchors present + cargo check.
- **Blocking condition**: >2 patches in one script without splitting; continuing after a patch without grep verification.

## PIT-42: WGSL uniform layout triple mine — closures/let-if/array stride (2026-09-30, the I band triple)
- **Symptom**: adding the env_sh section to mesh.wgsl collapsed three times: ① the `|i: u32| -> ...` closure syntax was rejected by naga (WGSL has no closures); ② the `let x = if cond {...}` expression form was rejected (WGSL has no let-if; needs var + statements); ③ the `array<f32,27>` uniform validation was rejected (uniform address-space array strides must be 16-aligned).
- **Root cause**: Rust intuitions translated directly into WGSL — all three are forms Rust has and WGSL doesn't; the stride rule is a hard constraint of GPU uniform layout (vec3 = 16B, array<f32,N> also needs 16 alignment).
- **Solution**: closures → module-level functions; let-if → `var x = ...; if cond { x = ...; }`; data arrays → `array<vec4<f32>,N>` (RGB+pad) or hand-arranged 16B alignment. After extending the View block tail, the total size must be a multiple of 16.
- **Verification**: after changes, first `cargo run --example <nearest>` to watch shader parsing/validation errors (lavapipe messages report exact lines) before discussing pixel assertions.
- **Blocking condition**: new WGSL uniform sections bound to binding sizes without the 16-alignment check.

## PIT-43: flag-offset mines = a probe's duty, not review's (2026-09-30, I band second hit)
- **Symptom**: the env_on flag was written at float 92 (a remnant of the old 27-float layout) while after the vec4 layout its true position = float 100 — sh-diff=0 pretended it worked; none of the three reviewers found it; a probe caught it in one pass.
- **Root cause**: after CPU/GPU sides evolved separately, the offset table drifted; review reading code can't spot "92 vs 100" numerical misalignment.
- **Solution**: CPU/GPU shared layout sections write **the same byte-offset table into both comments** ("flag=float 100=byte 400", dual-sided anchors); validation predicates for layout changes must include **end-to-end value deltas** (sh-diff > threshold); testing "doesn't crash" alone is forbidden.
- **Verification**: an end-to-end pixel-delta assertion present (the E507 env probe form).
- **Blocking condition**: delivering a new CPU→GPU section without an end-to-end value-delta assertion.

## PIT-44: the web channel is an independent surface outside ci + roster scripts read artifacts without freshness (2026-10-08, the V2.2 mine surfaced only two bands later)
- **Symptom**: `pixi run web-check` reported `the trait bound Vec<f64>: RefMutFromWasmAbi is not satisfied`, rooted in
  the wasm bridge `get_node_transform(&mut self, entity, out: &mut Vec<f64>)` written in the V2.2 band.
  That band had recorded "WEB MIRROR 3/3" in the baseline, and the mine lived on until the next band detonated it.
- **Root cause**: three stacked. ① `pixi.toml`'s `ci` = 11 segments, **not including web-check** (`web-check` is a standalone task) —
  no "WEB ✓" wording is backed by ci. ② the host channel (`cargo test -p visiaengine-wasm`, target=host) **does not compile
  the wasm32 codegen path**: wasm-bindgen only demands `RefMutFromWasmAbi` on wasm32 targets,
  so the signature is green on host and red on the real target (the compile-surface variant of PIT-22 "verification channel ≠ user channel").
  ③ `scripts/web-mirror.mjs` asserts against **build artifacts** under `target/web/` (d.ts rosters/value pairs),
  with no freshness check — with yesterday's artifacts and uncompilable sources, it still printed MIRROR 3/3.
- **Solution**: ① fix the signature per repo precedent (same shape as `getCameraPose -> Vec<f64>`): wasm has no out-parameter channel,
  so the C surface's "negative code + out untouched" maps to **empty-array returns** on the wasm side; ② add a **staleness guard** to the roster script (source-directory max-mtime
  newer than the d.ts → red, the report naming the directory and the remedy command), directly walling off the "report green off stale artifacts" class;
  ③ claim discipline: **"WEB ✓" may only come from same-terminal `pixi run web-check` output** (the web-surface projection of #17).
- **Verification**: breakage probe executed — after `touch bindings/wasm/visiaengine-wasm/src/lib.rs`, running the roster script alone
  must go red (report = "artifact stale: … run pixi run web-check first", rc=1); after re-running web-check it re-greens at MIRROR 3/3.
  `grep -c "stale artifact" scripts/web-mirror.mjs` ≥1.
- **Forbidden**: using host `cargo test -p visiaengine-wasm` results to stand in for wasm32 usability;
  writing WEB wording in baselines/commit messages without running `pixi run web-check`.

## PIT-45: "window looks lit" is not "window looks right" — two false-evidence channels (2026-10-08)
- **Symptoms**: E510's and E511's resident windows passed every automated gate (first-frame
  lit-ratio 82.3%, pixel-delta assertions, ctest, Xvfb display group) yet both rendered the
  wrong thing: E510 showed yellow discs ~1/4 frame height swallowing the road ribbon; E511
  showed a flat plate that "swung" about the wrong axis (it was built in the x-y plane in a
  Z-up world and rotated about Y — a floor panel, not a gate).
- **Root causes** (three, all independent):
  1. `px_world_scale: 1.0` in a window/assertion lane means "1 pixel = 1 world unit". Point
     radii and stroke widths are declared in **pixels** (`PointMark.radius_px`, GEO-24), so a
     6 px marker became a 6-world-unit disc. The geo pipeline computes the scale from the
     camera; hand-written frames must too.
  2. A hardcoded clear-colour triple in a pixel assertion (`[16,21,29]`) missed every pixel
     (actual `[15,20,28]`), so "geometry px" counted 153600/153600 = the assertion was
     constant-true and had been for a whole band.
  3. Lit-ratio and pixel-delta are *differential* measures: neither has any notion of whether
     the subject resembles what the example claims to show.
- **Fixes**:
  1. derive `px_world_scale = 2·dist·tan(fov_y/2)/height` from the rig (helper in both examples);
  2. self-calibrate the clear reference from the render's own corner pixel instead of a literal;
  3. added a determinism canary (same pose renders byte-identically) so a pixel-delta claim
     cannot be propped up by nondeterministic background;
  4. review the actual picture: `xwd -> PNG` and look (or measure per-band delta). Machine
     harness used: XTEST key/wheel/drag injection + screenshot diff (drives the "only a human
     can press this" cases; angle/title echo read back through the window manager).
- **Verification**: after fixes — E511 diff 1222 -> 5848 (measured, threshold 3500) and the
  picture reads as two hinged leaves between posts on a ground plane; E510 geometry count
  dropped from the vacuous 153600 to 16457 and the ribbon/markers are legible; harness reports
  E506/E507/E510/E511 all OK with Esc exiting each through the real X input path.
- **Prohibited**: hand-writing `px_world_scale: 1.0` in a frame that draws px-sized points or
  strokes; hardcoding a clear-colour literal in a pixel assertion; treating lit-ratio as
  content verification for an example whose claim is about *what* is drawn.

**Pointer (band K, 2026-10-08)**: the hand-built harness this entry describes is now
a repo tool -- `pixi run keys-probe` (scripts/keys-probe.py + scripts/xinject.c).
Rebuild it by hand only if the tool is skipped for a missing dependency.

## PIT-46: a stale ledger line generated a wrong plan (2026-10-08)
- **Symptoms**: the recommended next-step order handed to the user led with "integrate the tile
  scheduler into the render loop" as the highest-value item. That work had already shipped
  (N1.4 / CAPI-35): the engine pumps the layer every frame.
- **Root cause**: the item was copied from a residual-ledger line in `status.md` without re-deriving it
  from code. Ledger lines are cheap to write and never announce when they become false; the
  install-tree band's own residual list had inherited one from the P1 band verbatim.
- **Fix**: before a ledger item becomes a plan item, open its anchor. Here that took one grep and
  produced the *real* gap instead: the capi engine keeps a **single** layer slot
  (`engine.rs:1196/1991/2021` all assign `self.tiles = Some(..)`, no `None`), so a raster
  basemap and vector tiles are mutually exclusive. Band T of the new plan is that, not scheduling.
- **Verification**: `grep -n "self.tiles" bindings/c/visiaengine-capi/src/engine.rs` before
  claiming anything about tile-layer capability; `sed -n '765,805p'` before claiming the pump is
  missing. Cross-check any residual/todo line the same way when promoting it.
- **Prohibited**: turning a ledger line into a plan item without re-measuring it; "we should
  integrate X" where X exists.

## PIT-47: wiring a gate into CI without asking "who provides its tooling" — a permanently silent SKIP green (2026-10-08)
- **Symptom**: band K wrote `keys-probe` / `window-probe-all.sh` into `.github/workflows/ci.yml`;
  all green locally — but on the image runner these two lines **never execute any assertion**:
  they `SKIP exit 0` when `xwd` is missing, and CI just prints green. A step existing ≠ a step running.
- **Root cause**: two layers. ① wrong dependency provenance: `xwd` belongs to the **system package x11-apps**
  (`dpkg -S /usr/bin/xwd`, measured), not the pixi environment, while the workflow's apt only installed `mesa-vulkan-drivers xvfb`;
  `xorg-libxtst`/XTest.h, conversely, is in pixi.lock (so that part carried no risk — judging correctly required checking provenance item by item).
  ② the gate convention itself is "missing tool = SKIP exit 0" (so clones don't false-red); inside CI that convention becomes
  "missing tool = never runs, still passes" — i.e., the CI-wiring variant of PIT-25 (absent-true).
- **Solution**: ① add `x11-apps` to apt with a comment marking it non-optional (protecting it from future "dependency tidying" deletions);
  ② give the probe a `--strict`: the SKIP flips to exit 1 and **names the missing piece** — CI uses `--strict`, local defaults may still skip;
  ③ look tools up on PATH (the old code hardcoded `/usr/bin/xwd`; other container layouts would neither find it nor say why);
  ④ before wiring, **actually run that step once locally** (`window-probe-all.sh` had landed for an entire session without being run;
  this entry's re-run: rc=0 / 20 resident windows / E509 lit=7500/7500 100.0% vs the 5.0% threshold).
- **Verification**:
  ```bash
  # branch reachability (--strict's meaning is precisely that it can be triggered)
  python3 - <<'PY'   # make only xwd missing; inject the missing-tool branch of main()
  #   --strict --all -> rc=1 and the report "missing toolchain: xwd"
  #   default        -> SKIP rc=0
  PY
  dpkg -S "$(command -v xwd)"                    # expect it names the system package; don't assume pixi
  grep -n "x11-apps\|--strict" .github/workflows/ci.yml
  bash scripts/window-probe-all.sh; echo rc=$?   # newly wired gates must run locally for real first
  ```
- **Forbidden**: wiring a SKIP-semantics gate into CI without giving it a "missing tool = red" calling form;
  skipping the local real run before committing a CI change.

## PIT-48: the probe/injector's own invocation errors swallowed by `>/dev/null 2>&1`, misjudged as "the object under test doesn't respond" (2026-10-08, band V4)
- **Symptom**: the interaction tri-state gate reported `idle==drag==wheel` for two examples in a row, printing "✗ input eaten";
  meanwhile `pixi run keys-probe` itself was green 4/4. The second hand-built `gcc scripts/xinject.c -lX11 -lXtst`
  then reported `-lXtst: not found` — because `keys-probe.py:build_injector()` had long added
  `-L$ENV/lib -Wl,-rpath,$ENV/lib`, and the artifact `target/keys-probe/xinject` already existed.
- **Root cause**: two layers. ① xinject's contract is **positional arguments** `xinject <display> <title-substr> <action> [args]`;
  out of habit I wrote `xinject -display :69 -drag ...`, so parameter parsing failed and exited; ② I sent all stdout/stderr
  to `/dev/null`, so "the gate never ran" and "it ran but the picture didn't change" were **identical in output**. The assertion only looked at the latter.
- **Solution**: ① reuse the tool's own constructor/artifact instead of re-handbuilding the compile command (what was missing was `-L`, not knowledge I should learn);
  ② harness calls uniformly go to `> /tmp/x.log 2>&1` keeping the report, and **assert harness rc==0 first**, only then inspect picture hashes;
  ③ the gate's self-proof: before judging red, run `xinject <disp> <needle> name` to prove it actually found the window.
- **Verification**:
  ```bash
  target/keys-probe/xinject :70 NOPE_NO_SUCH_WINDOW name >/tmp/h.log 2>&1; echo "rc=$?"  # expect non-zero
  target/keys-probe/xinject :70 E501 name            # expect NAME=... echo, proving the needle hit
  grep -n "2>/dev/null" scripts/keys-probe.py        # expect: no report-swallowing forms on the harness side
  ```
- **Forbidden**: asserting "input had no effect / picture unchanged" while harness output was discarded or its rc unchecked.

## PIT-49: Translation batches can silently delete code lines; line-count diff is not a semantic check (2026-10-12)
- **Symptom**: E1 (facade i18n batch) removed three code lines from `scripts/window-probe.sh`:
  the app launch (`DISPLAY=... "$BIN" & APPID=$! sleep 6`) and the final `exit $rc`.
  The probe then exited 0 unconditionally (the `&&/||` echo tail swallows the verdict) =
  the black-screen machine gate was vacuous for two rounds before the T-D era caught it.
  bash -n passes, gate-docs passes -- nothing in the E1 closure loop ran the script's semantics.
- **Root cause**: whole-file manual translation with no structural diff between old and new
  content; comment-line churn hides a deleted code line at the same position.
- **Solution**: restore launch block + `exit $rc`; audit every E1/E2/E3-touched script for
  exit-line count vs the pre-translation parent commit (`git show <parent>:<f>` signature
  diff). E5 added `gate-i18n.sh` (CJK scan) -- structural loss is its blind spot; the
  exit-line audit is the completing check.
- **Verification**: `bash scripts/window-probe.sh E101_clear` SKIPs or asserts per contract;
  `diff <(git show <E1^>:f | grep -E '^\s*(exit|return)\b') <(grep -E '^\s*(exit|return)\b' f)` empty.
- **Forbidden**: committing a translation batch without a code-line delta audit
  (non-comment line count before vs after, per file).

## PIT-50: `git add -A` while a parallel agent works in the same tree sweeps the agent's staged work (2026-10-12)
- **Symptom**: the E2 commit (`git add docs/`) landed 96 files belonging to the concurrently
  running E3 agent -- E3 had self-staged part of its work; the E2 message under-described the content.
- **Root cause**: the index is shared state; any agent staging + any actor committing = cross-commit bleed.
- **Solution**: soft-reset + re-add by exact path spec before E3 finished; E3 residuals were intact
  (per-file whole writes, no partial corruption). Standing rule: the orchestrator commits only
  exact path specs it owns; parallel translation agents are told not to stage at all (E3 was, E4 wasn't).
- **Verification**: `git show --name-only <commit>` count matches the message's claim, or fix before push.
- **Forbidden**: `-A`/`.` as path spec while any background task edits the tree; agent-instruction
  omission of the "no git add" clause.

## PIT-51: The "T-C contract" reading kept stale handle names after the crate homecoming (2026-10-12, user-caught)
- **Symptom**: after T-H3.5 moved the crate to crates/visiaengine-ffi, the IDE tree showed FOLDER "crates/visiaengine-ffi" containing buttons still named visiaengine-capi-shared/-static/-rusttest -- FOLDER and target names disagreed.
- **Root cause**: the T-H2 plan's rename table was row-granular; the handle name hiding in polyorch_rust_build's TARGET kwarg was never named as a row, and the "T-C contract" justification was accepted without asking what the contract actually was. The real contract is the visiaengine::c ALIAS layer (zero external consumers of the handle name; scripts use -p visiaengine-ffi since step 3) -- an internal name has no contract standing.
- **Solution**: internal names follow their home. Rename the handle + derived verbs (visiaengine-ffi, -ffi-shared/-static/-shared-build/-static-build, -ffi-rusttest) and repoint the VISIAENGINE_ARTIFACT probe; keep only genuinely-consumed names (the alias, the lib name, clause IDs).
- **Verification**: `git grep -n "visiaengine-capi" cmake/ scripts/` empty (frozen memory docs excepted); `git grep -nE 'visiaengine-(c|cpp|qt)(-sources)?$|visiaengine::(c|cpp|qt)\b' cmake/` shows the four-lane family only.
- **Forbidden**: keeping any internal target name whose prefix disagrees with its FOLDER/disk home on the grounds of an unnamed "contract" -- name the consumer, or the name follows the home.

## PIT-52: Planned-rename inventories are stale on arrival; only a fresh git grep plus real execution catches the whole class (2026-10-12, three live catches in one round)
- **Symptom**: three classes the plan's static inventory missed, each caught by a different mechanism: (a) CARGO_MANIFEST_DIR/../../../ fixture depth -- invisible to string greps for the crate name, broke rusttest compile at execution; (b) the umbrella rename left 3 body lines (include/link/rpath) using the old real name -- configure red immediately; (c) the wasm path-dep example literal in the plan was arithmetically wrong (../visiaengine-ffi vs the correct ../../../crates/visiaengine-ffi).
- **Root cause**: an inventory taken at planning time is a photograph; the tree moves between plan and execution (T-H3 landed between inventory and execution, shifting counts 31->55 and depths). Also: grep for a NAME cannot see RELATIVE-DEPTH references, and plan-table literals are hand-arithmetic.
- **Solution**: command-driven sweeps (run `git grep` fresh at execution time, classify hits machine-bound/auto-follow/docs/frozen) + every rename step carries its own REAL verification legs (configure, cargo metadata, rusttest, install-tree) -- each of the three catches came from a leg actually running, not from review. Fix relative-depth references by re-deriving depth from the new home, not by editing the old literal.
- **Verification**: `cargo check -p <moved-crate>` + the moved crate's rusttest button green; `git grep -c "<old-literal>"` = 0 on machine-bound files; install-tree leg path-independent (assert by installed-relative names).
- **Forbidden**: executing a planned rename by table lookup without re-running the sweep commands; treating "verified inventory" in a plan as a substitute for execution-time re-derivation.

## PIT-53: the INTERNAL metadata-source cache outlives its keys (2026-10-13)
- **Symptom**: a supplier fix to source-mount logic showed zero effect after
  reconfigure; File API showed 0/29 example nodes mounted despite the data
  provably present in cargo metadata (isolated CMake repro: hits=1).
- **Root cause**: `_polyorch_rust_metadata_sources` caches per
  (manifest, crate) as `CACHE INTERNAL` inside the build tree --
  entries persist across configures keyed by OLD paths (pre-rename
  manifests, even the retired bindings/js tree). An EMPTY cached value
  short-circuits before the new code ever runs.
- **Solution**: wipe the build tree's stale entries (delete
  `polyorch_meta_srcs` lines from CMakeCache.txt or nuke the build dir)
  after any change to the mount/metadata logic; reconfigure.
- **Verification**:
  `grep -c polyorch_meta_srcs cmake-build/CMakeCache.txt` before/after;
  the File API query (codemodel -> target -> sources) must list the
  expected .rs files.
- **Forbidden**: concluding "the fix does not work" from one reconfigure
  while a keyed INTERNAL cache survives from earlier runs.

## PIT-54: a MID-argument quote in execute_process lands in the child cache as a literal (2026-10-13, three debug rounds)
- **Symptom**: `-DPOLYORCH_CMAKE_DIR="${dir}"` inside execute_process(COMMAND ...) --
  the child configure's include() failed on a path whose first segment was wrapped in
  literal quote characters; three rounds misdiagnosed (trailing slash, then heredoc
  escaping) before the raw argv was trusted.
- **Root cause**: CMake-language quoting keeps quotes appearing MID-argument in the
  resulting token; the child cmake takes everything after `=` verbatim, quotes included.
  A direct SHELL invocation does not reproduce it (the shell strips quotes) -- so a
  shell repro is NOT a repro of an execute_process argv path.
- **Solution**: quote the WHOLE -D argument: `"-DPOLYORCH_CMAKE_DIR=${dir}"`.
- **Verification**: the include() line of the generated child CMakeLists prints the
  expanded value; a `"` inside it = this pitfall.
- **Forbidden**: diagnosing this from a shell repro; editing the child file's $-escaping
  while the parent's -D value is the broken side.

## PIT-55: test-scaffold CMakeLists trio -- CASE reset, bracket-arg escaping, python-chain quote-eating (2026-10-13/14, six recurrences)
- **Symptom**: (a) a generated child CMakeLists containing `set(CASE "")` reported
  zero branch hits despite -DCASE passed (three cases: t-wasm-serve, t-option, and the
  matrix design round); (b) `$\{POLYORCH_CMAKE_DIR\}` written "to escape" for the child
  became a LITERAL in the file and broke include() (bracket arguments expand NOTHING --
  write ${VAR} plainly inside [=[ ]=]); (c) python-patch chains ate the `\"` escaping
  of CMake string content ("Syntax Warning ... Argument not separated" from a
  de-escaped regex literal).
- **Root cause**: three independent quoting layers (cmake -P script / file(WRITE
  bracket-arg) / python string replace) each have DIFFERENT escape semantics; the
  intuition of one layer applied to another produces silent literals, not errors.
- **Solution**: inside bracket-arg templates write variables plainly (no expansion
  happens anyway); NEVER seed a -D-influenced variable with set() in the template;
  for CMake strings containing JSON quotes prefer the write tool over multi-layer
  python replace.
- **Verification**: `sed -n '3p' <generated child CMakeLists> | cat -A` -- visible
  backslashes/quotes where none belong; zero-hit branch = grep the template for a
  `set(CASE` line.
- **Forbidden**: re-escaping the same content a second time because round one "didn't
  take" -- confirm which layer mangled it first.

## PIT-56: renaming one creation point INTO the name of another arms a latent CMP0002 (2026-10-13, exploded in the debug preset a full round later)
- **Symptom**: T-H5.5 renamed rs/cargo-run buttons to visiaengine-example-<E#>-run;
  configure green under default. The DEBUG preset (gate ON) then died on CMP0002:
  V's manual button and polyorch_rust_run's internal `<TARGET>-run` had become the
  SAME name. One month of three-way distinct names had hidden the two creation points.
- **Root cause**: a rename sweep checks the renamed strings; it does not ask "does the
  NEW name already have a creator somewhere (a supplier face that also emits one)".
  The AFTER snapshot ran one preset -- the second creator lived behind a gate the
  snapshot never opened.
- **Solution**: rename target names must enumerate ALL creators of the destination
  name (grep the supplier face for add_custom_target/${T}- patterns), and the AFTER
  verification must run EVERY gate polarity the lane supports.
- **Verification**: configure under BOTH presets green; `--target help` diff before/
  after under each polarity, not just the default one.
- **Forbidden**: closing a rename round after one preset's snapshot.

## PIT-57: a gated generator hook short-circuits the WHOLE surface -- switching presets FREEZES stale rows, it does not empty them (2026-10-14, measured)
- **Symptom**: expectation (from the launch.json empty-block report): a default-preset
  (gate OFF) reconfigure empties the debug region. Reality after the dual-state fix:
  the DEFER hook registers only when a face gate is ON -- gate OFF runs no generator
  at all, the launch.json keeps its previous debug-era rows UNCHANGED (usable F5 by
  accident, or silently pointing at renamed artifacts).
- **Root cause**: "writes empty region" and "does not write at all" are two different
  failure shapes; the hook architecture gives the second. The Momus one-line STATUS
  (empty-over-nonempty) is UNREACHABLE in pure single-polarity worlds -- retained
  only for the mixed-face case (one face ON, another switched OFF).
- **Solution**: document "preset switch does not clear" as the architecture fact;
  stale rows on renames die at the next gate-ON configure (rows are regenerated from
  current targets, never from the file).
- **Verification**: gate-off configure leaves .vscode mtime unchanged; gate-on
  configure regenerates rows fully (rename round proof: 29 rows reappeared with the
  new names).
- **Forbidden**: reasoning "empty block overwrote the file" from the first
  observation -- check the file content before attributing the mechanism.

## PIT-58: a test leg that re-types the shipped string tests the copy, not the product (2026-10-14, live user failure)
- **Symptom**: E306 run button died in winit ("neither WAYLAND_DISPLAY nor
  WAYLAND_SOCKET nor DISPLAY is set") and then in sh syntax errors, while
  t-rust-run-gui/t-example-run were all GREEN: the probe string shipped to
  build.ninja carried literal `$${DISPLAY:-}`, leaked `]==]` bracket-close
  garbage, and had its newlines flattened (`&&[ -z`).
- **Root cause**: three stacked blind spots -- (1) the `$$`+REGEX collapse
  inherited from the quoted era NEVER matched (CMake regex `\$\$` semantics);
  (2) the MULTI-LINE command string is newline-flattened by the NINJA
  generator, invisible under the Makefile generator the unit legs run on;
  (3) the live leg hand-copied the script text into execute_process instead
  of executing the shipped function's output -- so it proved the copy.
- **Solution**: bracket argument expands nothing (single $ verbatim) + ONE
  line, semicolon-separated (portable across generators) + the test consumes
  `_<...>_probe_script` OUTPUT directly (absence-pin on `$$` added).
- **Verification**:
  `grep -n "_polyorch_gui_probe_script" tests/cases/t-rust-run-gui.cmake`
  (the live leg must call the shipped helper, not re-type the text); the
  double-dollar absence pin fires on any regression.
- **Forbidden**: proving a shipped string by re-typing it in the leg;
  Makefile-only shape pins for command text meant to run under Ninja.

## PIT-59: a ';' inside an embedded -c script silently splits into argv (2026-10-14, serve button never bound)
- **Symptom**: the serve button "completed 0" without a listener: python ran only the first segment `import http.server,sys` of the -c one-liner and exited clean -- no FAILED line, no traceback.
- **Root cause**: the code string passed through an unquoted COMMAND list expansion: CMake splits on ';' FIRST, so each segment became its own argv entry. The C21 corruption class wearing the "shell one-liner" hat.
- **Solution**: shipped shell bodies live in a generated file (file(WRITE script)) invoked by path -- zero quoting ladders; where a string must survive, '@'-join and collapse.
- **Verification**: `grep -m1 "COMMAND" build.ninja | grep ";"` on the command text -- a ';' outside CMake's own separators = split happened; live proof = port actually listens.
- **Forbidden**: trusting a "completed 0" for a daemon-shaped command; one-liners with semicolons through list expansion.

## PIT-60: post-hoc set_property(DEPENDS) never reaches the Ninja graph -- constructor only (2026-10-14)
- **Symptom**: polyorch_rust_wasm_serve's header promised "stale glue cannot be served", but a fresh tree served an empty pkg/: build.ninja showed only the module mediator edge.
- **Root cause**: the promise was written as a COMMENT, never wired; the first repair attempt used set_property(APPEND PROPERTY DEPENDS) after target creation -- silently invisible to the generator (measured: the edge line never changed).
- **Solution**: file-level deps of custom targets go through the add_custom_target(DEPENDS ...) constructor, full stop. Read the generated build.ninja as the acceptance evidence, not the source comment.
- **Verification**: `grep "wasm-serve: CUSTOM" build.ninja` shows the flavor .js outputs among the edge's deps.
- **Forbidden**: accepting "the property was set" as proof a dependency edge exists.

## PIT-61: curl-200 is not a browser -- page surfaces need execution checks (2026-10-14, hub DOM suicide)
- **Symptom**: every wall probe answered 200 (files, MIME, wasm) yet the user saw an empty sidebar + black iframe: render() wiped side.innerHTML and then re-appended #q -- already removed from the DOM -> appendChild(null) threw before anything rendered.
- **Root cause**: static-asset verification (curl) proves delivery, never JS execution; all pages were authored in the same session that never opened one.
- **Solution**: page-class work ships with (a) node --check on every inline module (write the script to a .mjs -- --input-type+--check is not a thing) and (b) ONE browser pass on the checklist before declaring done; honest escalation: a headless (playwright) leg is the real net, filed as a proposal, not built.
- **Verification**: `grep -c '"id"' wall.json` vs user-visible entry count; the syntax sweep command lives in the T-H11 commit history.
- **Forbidden**: reporting "wall verified live" off curl probes alone.

## PIT-62: shared-target attributes rewritten per registration drift by call ORDER; an opt-in with a default name is opt-out (2026-10-14)
- **Symptom**: the tree-wide polyorch-rust-all aggregate materialized inside the host's examples/rs FOLDER, and existed in trees that never asked for it (user catch: “is this abnormal?”).
- **Root cause**: every polyorch_rust_build call re-stamped the aggregate's FOLDER (last-writer-wins: the button "moved" to whoever registered last) and the "opt-in" knob carried a default name, so the opt was never required.
- **Solution**: first-writer pins (create-and-fold once; later calls only add_dependencies); opt-in knobs have NO default -- unnamed means unbuilt. Applies to any shared aggregate/generator hook attribute.
- **Verification**: t-rust-agg-optin (no-knob absence pin + folder-pin probe); `--target help` under a host that never set the knob shows zero aggregate.
- **Forbidden**: stamping shared-target properties outside the creation branch; "optional" features that appear anyway.

## PIT-64: a head-truncated wiring grep created a phantom gap -- the call already existed (2026-10-15, stale-fixes round)
- **Symptom**: audit concluded "E703 example is missing its native-debug wiring"; the walk-through adjudicated "add the call"; the plan-review pass found `examples/qt/CMakeLists.txt:32-36` ALREADY contains the guard+call (landed cd2ea51). Executing the adjudication as written would have duplicated a registration.
- **Root cause**: the original evidence was a `grep ... | head -12` whose 12 lines were all consumed by earlier matches; the wiring lines sat below the cut. Both the auditor and the orchestrator trusted the truncated view.
- **Solution**: wiring-presence greps must run UNTRUNCATED over the file (or with `-c` first), and any "missing X" adjudication gets one full-file `grep -n X <file>` re-check before dispatch. The plan-review caught it because the reviewer read the real file instead of the digest.
- **Verification**: before writing "missing/absent" findings into any plan: `grep -n '<exact callee>' <file>` full output in hand.
- **Forbidden**: concluding absence from a truncated grep; treating an adjudication premise as immune to a later full-file read.

## PIT-65: a trailing comment on a .gitignore line voids the rule -- tasks.json never landed since T-E (2026-10-15)
- **Symptom**: `!.vscode/tasks.json   # T-E: commit the managed region` looked like an un-ignore rule; git treats the whole string after `tasks.json` as part of the pattern (gitignore `#` is a comment ONLY at line start) -- the negation matched nothing, and tasks.json stayed ignored for its entire lifetime.
- **Root cause**: no gate checked the TRACKED state of the trio; the IDE audit read the DISK file (the generator writes it regardless of git status) and reported "all three committed" from disk evidence.
- **Solution**: comments go on their own line; contracts about committed files are proved with `git ls-files`, not by reading the file off disk. Fixed by `27cbbfc` (first tasks.json commit followed in b6e0e7f).
- **Verification**: `git ls-files .vscode/` lists exactly launch.json settings.json tasks.json.
- **Forbidden**: `pattern   # comment` shapes in .gitignore; asserting tracked-ness from disk presence.

## PIT-66: two configure presets must not share one binaryDir; a refresh tree must see EVERY lane (2026-10-15)
- **Symptom**: `debug` preset (Qt ON + debug knob) inherited `cmake-build` from `default`; one `pixi run f5` stamped `VISIAENGINE_QT=ON` into the everyday cache, and since default carries no Qt key, every later `--preset default` configure FATALed. Separately: giving the everyday preset the refresh knob made the committed .vscode ping-pong -- default sees 43 lanes, debug sees 44; last configurator wins, rows flip.
- **Root cause**: preset cacheVariables apply only to ABSENT keys -- a stamp from a sibling preset survives as a booby trap; and the .vscode generator REWRITES its whole managed region from what the current configure registered -- a refresh tree that cannot see a lane actively deletes that lane's row.
- **Solution**: one binaryDir per preset (`cmake-build-<name>`, repo convention); only the FULL-FEATURE tree (Qt + knob, run in the env that carries Qt) owns the refresh duty -- `pixi run f5` (debug preset, own `cmake-build-debug`); knob-less everyday configures provably leave `.vscode` untouched (cmp-verified).
- **Verification**: `python3 -m json.tool CMakePresets.json` + each preset's binaryDir distinct; `pixi run cmake --preset default` after `f5` → `cmp` byte-identical.
- **Forbidden**: sharing a cache across presets with asymmetric feature gates; handing region-rewrite duty to a tree that cannot see every lane.

## PIT-67: audit dispatch pinned the WRONG repo's layout -- the auditor self-attested its drift via HEAD (2026-10-15, team audit round)
- **Symptom**: the docs-and-generated team member reported on the ./PolyOrch submodule (its "HEAD 3cbc155", greet-cargo/rust-basic surface) instead of the host VisiaEngine docs the assignment needed.
- **Root cause**: the dispatch prompt's SCOPE file list was copied from the supplier's AGENTS layout (whitepaper/tutorials/modules tree) while cwd was the host; contradictory anchors resolved silently -- the worker obeyed the file list, not the intent.
- **Solution**: first line of any cross-repo dispatch = the absolute path of the target repo; on report arrival, check the self-attested scope (HEAD hash / path prefixes) BEFORE folding findings in. Rescoping by continuation message (same session, context kept) cost one round-trip and still produced the supplier findings as a bonus.
- **Verification**: `grep -m1 'HEAD' <report>` names the repo the worker actually read.
- **Forbidden**: relative-only scope lists in multi-repo workspaces; trusting dispatch intent over dispatch anchors.

## PIT-68: synthesis-lead member was dead weight in a read-only audit fanout (2026-10-15, one-line capture)
- All member reports arrive at the orchestrator INLINE as peer messages; a "dedup+rank+summarize" lead only sees what the orchestrator already holds -- plain parallel task() fanout saves the slot and the unread-message traffic. Use teams when members must message EACH OTHER; audits don't.

## PIT-69: closing reports from memory fabricate facts -- hashes, backlogs, phantom fixes (2026-10-15, self-caught same session)
- **Symptom**: the reuse-round closing message asserted (a) a fix commit `c3c5355` that does not exist (no T-H12-line error existed to fix), (b) "V backlog 39" (truth: 2), (c) remote tips quoted from a garbled remembered string instead of a live read. Caught only when the user's follow-up question forced fresh `cat-file`/`rev-list`/`ls-remote` runs.
- **Root cause**: composition-from-recollection after long serial-worker rounds: real events + plausible details get blended into claims; pushes whose output was swallowed by shell timeouts got "concluded" both ways (earlier "blocked by network stall" was also WRONG -- refs had landed).
- **Solution**: every hash/count/tip in a report must come from a command executed IN THE SAME MESSAGE's preceding tool call; swallowed push output => re-derive from `ls-remote` before declaring blocked or done.
- **Verification**: the report's numbers traceable to pasted outputs; `git cat-file -t <every quoted hash>` resolves.
- **Forbidden**: quoting SHAs/backlogs from session memory; declaring push state without ls-remote; the recapitulating close-out without a fresh evidence block.

## PIT-70: D-Bus system-proxy config poisons every chromium loopback fetch (2026-10-16, gate-web round)
- **Symptom**: chromium headless hung on ALL http pages (dump returned placeholders, screenshots black, "Page load timed out"); file:// pages were fine. A raw socket listener proved chrome opened the TCP connection and sent ZERO bytes.
- **Root cause**: chromium reads the desktop system-proxy over the session D-Bus. Scrubbing http_proxy env, --no-proxy-server and --proxy-bypass-list all failed -- the proxy came from the bus, not the env.
- **Solution**: launch chromium with the session bus removed: `env -u DBUS_SESSION_BUS_ADDRESS` (baked into gate-web.sh as CHROME_ENV).
- **Verification**: one small served page dumps instantly after the scrub; the whole gate runs in ~4 s.
- **Prevention**: when a browser-ish tool dies ONLY on loopback but file:// works, suspect the desktop services bus before blaming the server -- and re-test this class with the smallest possible page.

## PIT-71: --dump-dom fires AT the load event; CLI snapshots cannot see post-load GPU boot (2026-10-16, gate-web round)
- **Symptom**: the data-frame heartbeat never appeared in ANY dump (measured 20+ attempts, both headless forms, ~0.56-0.63 s exits); boot completes 1-3 s after load under swiftshader.
- **Root cause**: Chrome 131 headless does NOT hold the load event for pending top-level await (spec-correct page semantics, unimplemented in the CLI lifecycle); --timeout is only a cap, virtual-time dumps before promises resolve.
- **Solution**: make the page's OWN loader hold the line: a parse-time `<img src="/ve-gate-ready">` subresource whose response the gate server holds until the post-8-frame beacon arrives -> load (and dump) lands after boot; server-side READY log is the race-free proof channel. Harmless 404 outside the gate server.
- **Verification**: green line carries `beacon <ms>, frames=8+`; negative proof (e) -- deleting the beacon line alone -- turns the leg RED while all other legs stay clean (load-bearing proven).
- **Prevention**: to observe post-load async state from a CLI with no driver layer, hold load with a subresource, do not add a remote-control dependency.

## PIT-72: restore-cp from a stale snapshot silently reverted an accepted mutation (2026-10-16, gate-web round)
- **Symptom**: right after a "restore" in the negative-proof chain, the gate went RED impossibly (no beacon, empty server-side READY log, implausibly fast legs) -- the page had been reverted to its pre-anchor shape.
- **Root cause**: the restore snapshot was taken BEFORE a later accepted page mutation; every accepted mutation invalidates all earlier restore snapshots -- the cp restored a legitimate-looking stale state.
- **Solution**: retake the snapshot immediately after every accepted mutation; verify canonical state by grep + cmp BEFORE trusting any subsequent green/red.
- **Verification**: this round re-snapshotted the anchor state, re-ran the full ledger, and only then recorded the final green; post-state cmp clean.
- **Prevention**: in mutate/restore loops a snapshot is only valid until the next accepted edit -- stale-snapshot restores are the dangerous kind of "byte-identical".

## PIT-73: with setsid, $! is NOT the server -- the port holder is (2026-10-16, serve-https round)
- **Symptom**: `setsid python3 server & echo $!` recorded PID N; group-kill of N left the listener alive on :8443 (real holder N+4 -- setsid double-forks when the caller is not already a leader).
- **Root cause**: trusting $! as the daemon PID under setsid; the recorded id is the intermediate parent.
- **Solution**: after start, resolve the actual holder via `ss -tlnp` (pid=...) and kill by THAT; keep both ids recorded.
- **Verification**: post-run `ss -tln | grep <port>` empty -- was re-proven live in the task-wiring test (holder killed, port clean).
- **Prevention**: any lane that backgrounds a server under setsid must include a port-holder check in its cleanup path, not just a PID echo.

## PIT-74: orchestrator "corrected" a working diagnostic with a spec-confabulated diagnosis; user evidence won (2026-10-16)
- **Symptom**: remote LAN machine showed the wall's ✗ insecure-context line ("not localhost/HTTPS, WebGPU locked by the browser" — the page's own copy, translated here); the orchestrator declared the diagnostic itself buggy ("WebGPU is NOT gated on http; only mediaDevices is") and queued a fix to common.js.
- **Root cause**: model-memory overrode both the W3C spec (WebGPU requires a secure context; navigator.gpu is simply absent on insecure origins) and the user's direct measurement; the "fix" would have corrupted a correct product string.
- **Solution**: the queued fix was cancelled before any edit; the observed ✗ was the true behavior; the serve-https lane (c7fdeb4) is the spec-correct remedy, and the host-side beacon-over-https-untrusted-cert test empirically confirmed the secure-context boundary.
- **Verification**: the page message matches spec; gate/web lane tests green.
- **Prevention**: before declaring an observed diagnostic "wrong", cite the primary spec sentence or design a measurement -- chat-memory is not evidence (same family as the repo's "Do not invent facts").

## PIT-75: Ninja DESC of a custom target is baked at add_custom_target time; post-hoc COMMENT writes never reach it (2026-10-19, dual-serve round)
- **Symptom**: after the host-side overlay called set_target_properties(COMMENT ...) on visiaengine-wasm-serve, the build log STILL printed the supplier's old `[2/3] ... http://127.0.0.1:8017` description; the new dual-URL text appeared nowhere in build.ninja.
- **Root cause**: the Ninja generator emits `DESC = <COMMENT>` into build.ninja during add_custom_target() registration; COMMENT is read once at target creation and never re-read. Verified by two minimal probes (with/without COMMAND): set_target_properties(... COMMENT "new desc") after creation leaves `DESC = old desc` in build.ninja in both.
- **Solution**: to change a visible build-step description, the COMMENT must be correct AT add_custom_target time. When the text lives in a frozen supplier function (C23), the host registers the target itself (mirroring the recipe) instead of trying to patch a registered target.
- **Verification**: `grep "DESC = " cmake-build/build.ninja | grep serve` shows the intended text; or run the /tmp probe pair (add_custom_target + post-hoc set_target_properties, grep "new desc" build.ninja = 0).
- **Forbidden**: assuming CMake target properties are live/views that regenerate ninja output; describing a "fixed" DESC without grepping build.ninja.

## PIT-76: file(WRITE) with a multi-line quoted argument is a CMake parse error, and my edit-tool writes kept smuggling in a stray `]` (2026-10-19, dual-serve round)
- **Symptom**: `Parse error. Function missing ending ")"` at the file(WRITE) call; after switching to string(JOIN), the generated python failed with `unmatched ']'` (`];    pass` on one line) -- twice across two write attempts.
- **Root cause**: (1) CMake quoted arguments do not span lines inside a command invocation the way I wrote them -- the parser dies before any code runs; (2) the edit tool wrote the line list with a `]` on the `"except KeyboardInterrupt:"` line (list-close glued to a content line), and since string(JOIN) receives the list via ${_py_lines}, the bracket rode into the generated script as literal text.
- **Solution**: generate multi-line text via set(list of single-line strings) + string(JOIN "\n" ...) + file(WRITE "${f}" "${body}\n"); after ANY such edit, grep the cmake file for `\]` outside real list closes AND py_compile the generated artifact before declaring success.
- **Verification**: `python3 -m py_compile <generated.py>` (the artifact, not the generator); `grep -n '\]' cmake/VisiaEngineWasm.cmake` eyeball for glued brackets.
- **Forbidden**: multi-line quoted args in file(WRITE); trusting generator-side syntax while skipping artifact-side compile.

## PIT-77: smoke-testing a foreground serve target from the orchestrator shell kills the IDE's own run -- one port, one owner (2026-10-19, dual-serve round)
- **Symptom**: my background verification of visiaengine-wasm-serve bound :8017; the user's IDE run of the same target then failed with `OSError: [Errno 98] Address already in use` -- twice (once from a leftover serve, once from my own detached probe).
- **Root cause**: the serve target is a FOREGROUND dev server by design; any orchestrator-side verification run occupies the same port the IDE button wants. setsid/nohup kept the python child alive after the parent cmake was killed (PIT-73 family: kill the port holder by PID, not the launcher).
- **Solution**: when verifying a serve/port-bound target from the agent shell, ALWAYS clean up by port holder PID (`ss -tlnp 'sport = :PORT'` -> kill pid) before ending the turn, and never leave a detached instance running. State it in the delivery message so the user knows the port is theirs.
- **Verification**: `ss -tln 'sport = :8017'` empty at hand-off; user IDE run binds successfully.
- **Forbidden**: leaving detached serve processes running after verification; killing the launcher PID only and assuming the child died.

## PIT-78: "pick dead on wasm" was an ENGINE bug -- a surface the pixel gate never probes can hide a total defect (2026-10-19, review-fix round)
- **Symptom**: E811 on wasm picked 0/307200 px while the same fixture picked 462 hits native; the 5-lane review's QA read it as "wasm32-specific glTF node-transform breakage".
- **Root cause**: mount_gltf never fit a camera (only geo/tiles paths did) -- node-transformed fixtures sat off-frustum of the initial rig on EVERY backend. The wasm framing was wrong twice over (no fit + no page-level pick probe), so the wasm surface hid an engine-level defect and mis-attributed it to the platform.
- **Solution**: fix the engine (mount_gltf framing fit, oblique depression per the E201 pitch law) and give every interactive page a machine-readable probe (?selftest rides the gate beacon; gate-web asserts RED+GREEN).
- **Verification**: gltf_node_transform_pick_preserved (RED-proven) + gate-web E401 pick assertion.
- **Forbidden**: reading a backend-specific diagnosis off a surface that shares one engine; calling a lane "verified" when its pick/interaction surface has no probe.

## PIT-79: transposing a matrix to "fix" a mystery -- verify the producer's contract from its SOURCE before touching conventions (2026-10-19, same round)
- **Symptom**: hierarchy.glb picked 0 hits; a transpose of mat4_f64 was tried, GLTF-02/08 spec gates went red, and the "fix" fight consumed several rounds before being reverted.
- **Root cause**: the transposed form fixed a symptom (translation landing in the wrong slot) that actually belonged to the missing mount fit; the gltf crate's Transform::matrix() is COLUMN-major per glTF spec -- verified only later by reading the crate source ("4x4 transformation matrix in column-major order").
- **Solution**: read the producing crate's source/doc FIRST (grep the vendored registry source), re-derive, then edit; when a spec gate contradicts your change, treat that as evidence AGAINST the change, not a stale gate.
- **Verification**: git log 34931f9 (revert documented in the commit message); gltf_spec 9/9 green at the verbatim widening.
- **Forbidden**: "the gate must be stale" reasoning; convention-level rewrites to rescue a hypothesis.

## PIT-80: fit-changed framing breaks every "pick the center pixel" assumption downstream (2026-10-19, same round)
- **Symptom**: after mount_gltf's fit landed, hide_filters/render_spec/E701/E811_c/E811cpp failed on "center must hit" -- and E701's lit-scan also mistook the clear color [13,18,25] (sums to 56 > 40) for content.
- **Root cause**: a framing change is an invisible API break for consumers that hard-coded screen coordinates; the clear color sits above common luminance floors.
- **Solution**: sweep for the first pick hit (or walk lit pixels until pick agrees, skipping the clear color explicitly); gates assert existence ("some pixel lit", "some hit"), not a specific pixel.
- **Verification**: cargo test -p visiaengine-ffi (10 files green); ctest headless lane green; ctest -R example_E811_entity_hide_c/cpp green.
- **Forbidden**: new consumers asserting a fixed screen point after any camera/framing change; luminance thresholds without a background-color exclusion.

## PIT-81: veLoop's onFrame callback receives NO arguments; ad-hoc harness chrome runs need the D-Bus scrub (2026-10-23, E305 band)
- **Symptom 1**: E305's drone never moved and the page threw `Uncaught TypeError: Cannot read properties of undefined (reading '0')` at the Catmull-Rom call site -- `moveDrone(now)` received `now === undefined` because `common.js` veLoop invokes `onFrame()` bare.
- **Root cause 1**: the callback contract is "page takes its own clock" (deliberate: the rAF timestamp belongs to the closure that owns the loop); assuming an rAF-timestamp parameter is writing against an unverified signature (ghost-API family).
- **Solution 1**: `const moveDrone = () => { const now = performance.now(); ... }`. Rule: every new page reads common.js's actual signatures before consuming shared scaffolding.
- **Symptom 2**: an ad-hoc `chrome --dump-dom http://127.0.0.1:...` run returned an EMPTY dom (40 bytes) with zero console lines, mimicking a dead page.
- **Root cause 2**: the shell's proxy/D-Bus environment; gate-web.sh already documents the measured fix (`env -u DBUS_SESSION_BUS_ADDRESS`, "every http page hangs, file:// unaffected") -- the probe bypassed the gate's own environment.
- **Solution 2**: any manual chrome probe copies gate-web's environment (`env -u DBUS_SESSION_BUS_ADDRESS`) or just runs the gate; empty-dom + no-console = probe-side, not page-side.
- **Verification**: gate-web E305 beacon 947ms; grep CONSOLE lines in the gated dom shows only the WebGPU-experimental INFO.
- **Forbidden**: consuming shared scaffolding without reading its signatures; hand-rolled chrome probes without the gate's env scrub.

## PIT-82: an unclosed <script type="module"> tag silently disables the whole page -- and my own degenerate write caused it (2026-10-23, E901 band)
- **Symptom**: E901 failed gate with "no beacon" while showing ZERO console output; the status line never left its initial text. Every manual probe (node --check, ESM check, screenshots on a bare server) looked healthy or inconclusive.
- **Root cause**: two stacked defects. (1) The degenerate write that authored the page ate the closing `</script></body></html>` tail -- an unclosed module block swallows the rest of the document and is NEVER evaluated, silently (no SyntaxError reaches the console). (2) node --check on the sed-extracted body cannot catch this: the sed range `/<script type="module">/,/<\/script>/p` falls through to EOF when no close exists, so the "JS-OK" check validated JS that the browser never ran.
- **Solution**: structural self-check before any page gate: `grep -c "<script" ` must equal `grep -c "</script>"` (and `</html>` must be the last line). In-page stage probes (`st.textContent = 'stage...'` after each await) localize a hang on the next gate run.
- **Verification**: after the close landed, the SAME file immediately progressed from "no beacon" to "engine reported a fail mark" (evaluation began) and then to beacon green once the normals-tiling bug (addMesh rejects normals shorter than positions -- tile the zero block per merged tower) was fixed.
- **Forbidden**: trusting node --check as page-level proof (it checks text, not the document); diagnosing "page dead" without a tag-balance grep first.

## PIT-83: wasm-bindgen glue does not track lib.rs -- a stale pkg passes host checks and ships without the new bridge (2026-10-23, bytes-bridge band)
- **Symptom**: after adding a wasm-bindgen leaf (e.g. loadTileBytes), `cargo check --target wasm32` passed, web-check passed, but the served `pkg/visiaengine_wasm.js` did not contain the new export -- the page died calling an undefined function.
- **Root cause**: the js-build target's ninja graph judged the bindgen step up-to-date (the wasm artifact was rebuilt, but the glue generation was not re-run; "Re-running CMake" consumed the build round). Stale glue is invisible to every host-side check because only the page actually calls the export.
- **Solution**: force the replay whenever lib.rs changed under bindings/wasm: `rm -rf cmake-build/web/pkg-web && cmake --build ... --target visiaengine-wasm-js-build`, then grep the generated js for the new `js_name` BEFORE any page-level verification (expect >=2 hits: export + d.ts shape).
- **Verification**: `grep -c "<BridgeName>" cmake-build/web/pkg-web/visiaengine_wasm.js` >= 2 after the forced replay; gate-web page beacon green.
- **Forbidden**: trusting web-check/cargo-check as evidence a new bridge shipped; building pages against a bridge before grepping the generated glue.

## PIT-84: generated hub regions silently duplicate their END marker every sync -- and the gate's "first END" check hid it (2026-10-28, agent-sync)

- **Symptom**: after many `agent-sync` runs, `AGENTS.md` carried 38 `END:agents-memory` markers (and 2 BEGIN); the skills region and `CONSTRAINTS.md` grew the same way, yet every `gate-agents` run stayed green.
- **Root cause**: the REWRITE path in `memory-aggregate`/`skills-index`/`constraints-index` computed `after = END + hub_text.split(END, 1)[1]` and then ALSO appended `END`, so each rewrite re-added one END. The gate's `_current_region` uses `split(BEGIN, 1)[1].split(END, 1)[0]` (first BEGIN / first END), so the region still compared equal and the stray markers were invisible.
- **Solution**: one idempotent rewrite: `before = split(BEGIN, 1)[0]`, `after = split(END, 1)[1]`, `new_text = before + BEGIN + "\n" + block + END + after` (single END, no re-prepend). Rebuilt the hubs from their hand-written head (truncate at the first BEGIN, then `agent-sync` re-appends one clean region).
- **Verification**: two consecutive `agent-sync` runs leave `END:agents-memory` / `END:skills-directory` / `END:ci-gates` at exactly 1 each; `gate-agents` green. (The second `BEGIN:agents-memory` is a literal mention inside the D34 prose, not a marker.)
- **Forbidden**: a generated-region rewrite that re-adds BEGIN/END; trusting a "gate green" check that only reads the first occurrence of a marker.

## PIT-85: a `.gitignore` inline comment silently disables the rule it annotates (2026-10-29, .gitignore reorg)

- **Symptom**: the cleanup draft carried lines such as `*.flc              # xemacs temporary files` and `Testing/                        # ctest output`. They read fine in review, but `git check-ignore` refused to match them — the whole line, spaces and `#` included, is the pattern.
- **Root cause**: `.gitignore` has no inline-comment syntax; only a `#` at the START of a line starts a comment. A trailing `# ...` becomes pattern text, so the intended path is never ignored, and nothing goes red because no other check references those rules. Seven lines were affected by the first draft.
- **Solution**: every explanatory note sits on its own comment line above the pattern. The syntax is proven in an isolated scratch repo (`git init` under `.cache/`): the inline form returns rc=1 (no match); moving the comment to its own line returns rc=0 (match).
- **Verification**: `grep -nE '^[^#].*[[:space:]]#' .gitignore` is empty; the 22-case `git check-ignore --no-index` matrix passes; `git status --ignored` before/after shows no diff.
- **Forbidden**: annotating a pattern on the same line; reviewing ignore files by eye without a parser probe; trusting "it looks right" for configuration syntax.

## PIT-86: the MCP launcher was cwd-robust but its REFERENCE was not -- Trae spawns outside the repo root (2026-10-29, Trae onboarding)

- **Symptom**: after `agents sync` produced `.trae/mcp.json`, Trae failed every server with bash's localized "No such file or directory" (ENOENT) for `scripts/mcp-launch.sh` — while the same file worked in CodeBuddy and the script existed, was executable, and resolved its own repo root from `BASH_SOURCE`.
- **Root cause**: `mcp-launch.sh` is cwd-independent *inside* itself, but the **argv that points at it** was repo-relative (`bash scripts/mcp-launch.sh <task>`). That only works for hosts whose MCP spawn cwd is the workspace root; Trae's is not. The design note in `mcp-links.py` ("so they work regardless of the spawn PATH / cwd") covered the script body and was silently read as covering the reference too.
- **Solution**: `mcp_launcher_absolute` target flag — a **gitignored** host file (`.trae/mcp.json`) may carry a machine-local absolute launcher path; a **tracked** one (root `.mcp.json`) may not, so it keeps the relative form. The asymmetry is legitimate precisely because adjacency is an ownership fact, not a portability preference.
- **Verification**: reproduced the exact message with `cd /tmp && bash scripts/mcp-launch.sh mcp-codegraph`; the absolute form returned valid MCP `initialize` results from `/tmp` for both `local-codegraph` (codegraph 1.6.0) and `local-serena` (Serena 2.1.0); root `.mcp.json` `git diff` stays empty; `gate-agents` green.
- **Forbidden**: assuming every host spawns MCP servers with cwd = workspace root; treating "the wrapper is path-independent" and "the reference to the wrapper is path-independent" as the same claim.

## PIT-87: a 0-byte case-variant `AGENTS.md` inside a rules folder becomes a generated rule, silently (2026-10-29, Trae onboarding)

- **Symptom**: `.agents/rules/common/` gained a 0-byte `AGENTS.md` next to the canonical `agents.md`, and CodeBuddy's generated tree grew to 41 rules. Every gate stayed green.
- **Root cause**: the generator mirrors canonical faithfully, so a stray `.md` at the expected depth IS a rule — that is the design, and it means garbage in the source is indistinguishable from a real rule once generated. The gate compares generated-vs-canonical, not canonical-vs-intent, so it cannot see the difference.
- **Hazard beyond this incident**: `agents.md` and `AGENTS.md` are the SAME file on a case-insensitive filesystem (Windows, default macOS). A case variant in the source tree is a checkout collision waiting for the next Windows clone, not just extra noise.
- **Solution**: delete the variant; confirm the generated tree pruned back to 40 (`[ MDCDEL] codebuddy common-AGENTS`). No code guard added: an empty-rule filter would hide the symptom, not the cause (the file must not exist), and it cannot fix a case collision anyway. Prevent with review discipline: never create a file in `.agents/rules/<lang>/` whose name differs from an existing one only by case, and read the `GATE-RULES ✓ (N rule files …)` count when it changes.
- **Verification**: `ls .agents/rules/common/` shows 13 canonical files and no uppercase variant; `ls -d .codebuddy/rules/*/ | wc -l` = 40; `git status` clean of untracked files under `.agents/`.

## PIT-88: msgid bytes are the gettext lookup key -- rewriting them silently de-translates entries (2026-10-30, book translation batches)

- **Symptom**: a translation sat in `book/po/zh-CN.po` (`msgstr` filled, msgfmt green), but the built zh page still rendered the English sentence. No error anywhere.
- **Root cause**: the mdbook gettext preprocessor matches `msgstr` to `msgid` by exact bytes. A hand "canonicalization" pass that rewrote every entry onto one physical line changed the byte form of entries containing escaped quotes (`\"`), so the lookup missed and the string silently fell back to English.
- **Solution**: msgid lines are immutable -- only `msgstr` may be rewritten. When migrating translations across a POT refresh, key by the UNESCAPED msgid text and re-emit the msgid exactly as the reference generator (msginit/msgmerge) wrote it. Verify a distinctive phrase with `grep` on the built HTML, not on the PO.
- **Verification**: grep the built page for a distinctive translated phrase (e.g. the zh tagline) -- the count changes only when the page actually changes; `pixi run book-i18n status` numbers move in lockstep.
- **Forbidden**: treating a PO file as plain text; "normalize the whole catalog" scripts; trusting `msgfmt --check` (it validates format, not lookup success).

## PIT-89: gettext reconstruction re-parses "N. Title" headings as ordered lists (2026-10-30, silent-fallback class)

- **Symptom**: translations for every numbered heading ("1. Problem and Opportunity") always fell back, regardless of correctness; 15 strings formed a stable failure class across five batches.
- **Root cause**: the preprocessor rebuilds markdown from the translated string and compares event streams. A heading whose text starts with `1. ` re-parses as an ordered list, so the source and translation event streams can never agree. Second bug in the same class: emitters that forgot to escape REAL newlines produced multi-line `msgstr` values polib rejects (all bash-comment entries died this way).
- **Solution**: rename English headings to `## §N. Title` (section sign is not a list marker, and matches the existing §-cross-reference convention); append explicit translations for the new msgids. Any PO emitter escapes `\n` like any other metacharacter.
- **Verification**: grep the built whitepaper page for a translated §-heading (the section-sign form) -- count >= 1; the harness fallback class count drops to the documented residue.
- **Forbidden**: heading text starting with `N. `; PO emitters that escape only `"` and `\`.

## PIT-90: whole-book greedy validation conflates failures -- use a one-page harness (2026-10-30, translation infrastructure)

- **Symptom**: the greedy apply-one-rebuild-whole-book loop dropped 30 of 109 strings, including single-word table cells, and re-runs dropped a different set -- drops looked random and unfalsifiable.
- **Root cause**: one bad string fails the entire book build, so every failure is attributed to the most recent candidate; whole-book builds also take minutes, capping batch size.
- **Solution**: a per-entry harness: build a one-page book with a one-entry PO; verdict `ok` (Chinese marker in output) / `struct` (build fails) / `fallback` (builds, still English). ~5 s per string, and the two failure modes separate cleanly -- `fallback` is a lookup problem, `struct` is a markdown-reconstruction problem; they have different fixes.
- **Verification**: harness verdicts predicted the fallback classes of the following three batches exactly (numbered headings, quoted strings, fence comments).
- **Forbidden**: validating translation batches against whole-book builds; treating "the build passed" as "the translation landed".

## PIT-91: prose scanners -- sibling bullets are distinct items, indented continuations are not (2026-10-30, gate-prose structure rules)

- **Symptom**: the new paragraph-length check reported a 1110-char "list item" that was actually four sibling `-` bullets concatenated, then an 832-char "paragraph" that was actually the soft-wrapped continuation of one bullet.
- **Root cause**: two symmetrical bugs: consecutive bullets sharing the same marker never flushed the accumulator (marker equality used as identity), and unmarked indented continuation lines flushed it instead of joining.
- **Solution**: every marker line (`-`/`*`/`N.`) starts a NEW item unconditionally; only an UNMARKED line whose raw form starts with whitespace continues the current item. Tables and fences reset both accumulators.
- **Verification**: gate-prose findings matched the manually counted real blocks (README 2990-char paragraph, tutorials trio, Qt matrix); zero false concatenations after the fix.
- **Forbidden**: using the marker character as item identity; stripping indentation before continuation detection.

## PIT-92: indented SUMMARY entries escaped the book chapter existence check (2026-10-30, sidebar folding)

- **Symptom**: after nesting the SDD contracts and the reference library under draft parent rows, gate-docs' chapter count silently dropped 52 -> 7 -- 45 entries were no longer existence-checked.
- **Root cause**: the check greps `^- \[...\](path)` -- anchored at line start, so the indented children of a folded section match nothing.
- **Solution**: `grep -oE '^[[:space:]]*- \[...\](...)'`. Re-verified the count returns to 52 and the negative path (a ghost entry) still turns red.
- **Verification**: `pixi run gate-docs` prints `book: 52 chapters`; removing a target file goes red.
- **Forbidden**: line-start-anchored greps over generated outlines that can gain indentation; trusting a count drop as "fewer chapters" instead of "fewer checks".

## PIT-93: docs/*.md are CRLF -- python string surgery with '\n' silently misses (2026-10-30, structure polish)

- **Symptom**: three consecutive edit scripts reported success but the target text never changed; a later block replacement glued the following bullets onto one physical line.
- **Root cause**: the docs tree is CRLF. `t.replace(old_with_newline)` misses every pattern spanning a line break (`\r\n` in the file); `t.index('\n\n')` never finds a blank line; re-emitting a multi-line block with bare `\n` glued what the blank line used to separate.
- **Solution**: detect `'\r\n' in raw` first, normalize to `\n` for processing, restore on write; or operate line-wise and re-join with the file's own ending. After every scripted edit, `git diff` must show exactly the intended hunks.
- **Verification**: `git diff --stat` hunk count matches the number of intended edits; grep finds the new text.
- **Forbidden**: multi-line python replace on repo text files without a line-ending check; re-emitting blocks with hardcoded `\n`.

## PIT-94: history surgery -- backup branch first, synchronous rebase, fresh origin/main (2026-10-30, 24->5 commit squash)

- **Symptom**: a reword-rebase driven by a backgrounded editor script raced and produced a mangled stack; a "24 unpushed commits" premise turned out stale because the user had pushed mid-session (origin/main was ahead of the local assumption), so a soft-reset targeted the wrong base.
- **Root cause**: three compounding process errors: rebase + sleep-coordinated background scripts; no backup ref before rewriting; origin state assumed instead of fetched.
- **Solution**: the checklist that finally worked: `git branch backup/<name> <tip>` BEFORE touching anything; `git fetch` + read `origin/main` immediately before the reset; `git reset --soft origin/main` then file-grouped re-commits (deterministic, no conflicts); verify with `git diff <old-tip> HEAD` == the intended delta only; synchronous `git rebase -i` with a GIT_SEQUENCE_EDITOR sed, `--amend` and `--continue` in one `&&` chain.
- **Verification**: `git diff <backup> HEAD` shows only the intended follow-up change; `git status` clean; `git log origin/main..HEAD` shows the intended thematic stack.
- **Forbidden**: backgrounding any rebase step; rewriting history without a backup ref; trusting an "unpushed count" from earlier in the session.
