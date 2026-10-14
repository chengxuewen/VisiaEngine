# VisiaEngine Pitfalls & Gotchas

> Every PIT is the five-part form; symptom / root cause / solution / verification are all mandatory (consolidation duty per C9). Lessons repeated ≥2 times or costing >3 rounds of localization are promoted in sync into an executable rule in `rules/common/edit-safety.md`.

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

## PIT-29: an ortho camera's only zoom definition = half-width; the dist field is decoration (2026-09-24, E206 third fix, second sighting)
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
