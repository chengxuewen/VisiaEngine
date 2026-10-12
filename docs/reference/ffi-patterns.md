# FFI/C-API binding pattern study — the team's ffi-best-practices evidence file

**Snapshot 2026-09-11** (team, 4 lanes × 28 first-hand sources: header files actually pulled to the local /tmp session archive + ~/.cargo/registry sources + dissection of the `.refinfo/MediaServo/bindings` own deliverables; line numbers are real file line numbers). **Purpose**: the register of the `[FFI-R:xx]` reference numbers used in the batch-2/batch-7 plans (guarding against dangling references). The pattern knowledge is non-GPL (all MIT/Apache/BSD/public specs + own MediaServo).

## Conclusion in one sentence
The batch-2 §2 fourteen-function surface converges in shape with the three most mature peers (**Sokol/Filament/webgpu standard headers**) with zero structural errors; the four vacuum holes that neither Momus round nor this research caught earlier are now nailed down by this research and landed in v1.3 (struct_size / overwrite assertions / gate tool names / re-attach semantics).

## Reference-number register

### CS = capi-surface (9 sources: SDL3/Vulkan/godot-headers/llama/sqlite/sokol/webgpu-headers/zlib/ggml)
| no. | conclusion | evidence |
|---|---|---|
| CS-M1 | VeInput evolution policy missing -> adopt the "memset zero-init + tail append, 0 = old behavior" SQLite iVersion light tier; reject the Vulkan pNext chain (v0 single-structure K8) | sqlite.h.in:855; vulkan_core sType at 1157 sites (heavy) |
| CS-M3 | two last_error assertions enter tests: a successful call does not overwrite (SDL's original "do not branch on the error string") + the invalidation predicate (SQLite's original "may be overwritten by subsequent calls") | sdl_error.h:31/132; sqlite.h.in:4245 |
| CS-M2 | a `\threadsafety` header-comment line per function (the SDL3 convention, in the same slot as the "one-line purpose") | sdl_error.h:151 |
| CS-R1 | **the handle bit layout is not written into the C header** (sokol dares to write because it source-distributes; a binary SDK writing it = inviting disassembly-and-persistence; Vulkan never documents bit layouts); write only "the value is opaque, only 0/UINT64_MAX carry meaning" | sokol_gfx.h:2131 comment vs vk_platform.h silence |
| CS-2 | u64 slot+gen generational handles: sokol as a direct precedent (16+16 bits, slot 0 reserved = isomorphic with our base 1) + Vulkan's 32-bit-platform mandates u64 constant-width precedent; **generational handles are a minority practice but effective** (nobody exposes them outside sokol) | sokol_gfx.h:2131-2147; vk_platform.h:26-58 |
| CS-4 | the error protocol's code+string dual-pull entry = the same school as SQLite/SDL (don't be led astray by webgpu's `Success=1`: its Success≠0 convention) | webgpu.h:207 |
| CS-5 | the packed u32 version is the same family as Vulkan/SDL; ELF .symver (zlib.map) = an Alpha optional, not a v0 gap | vk_platform.h:62; zlib.map |
| CS-9 | pull model: sokol sg_commit/webgpu present/SDL/llama are all host-driven; SDL3's push layer (SDL_AppIterate) is exactly the "pull core + push framework" two-layer precedent = isomorphic with our frame_requested deferred evaluation | sdl_init.h:89-133 |
| CS-10 | the VeInput flat struct is the same family as SDL_Event/WndProc (a fixed-length superset is lazier and enough versus a dispatching union) | sdl_events.h:1100 |

### FC = rust-ffi-craft (Rust->cdylib engineering; first-hand = [MS] `.refinfo/MediaServo/bindings` actual deliverables + wgpu-native fetched sources)
| no. | conclusion | evidence |
|---|---|---|
| FC-M1 | **gate-abi/demo build tool names**: this machine has no bare nm/gcc (orchestrator recheck ✓), conda pieces carry prefixes; fix = the I0 host feature extends binutils+gcc, or use prefixed tool names | `which nm`=1; `x86_64-conda-linux-gnu-nm` ✓ |
| FC-M2 | the input thread/paint thread may differ -> the header nails the "host marshals to the owner thread" obligation sentence | Qt QWindow thread semantics |
| FC-M3 | 14 barriers collapse into the single macro `capi_guard(\|\|…)`; the demo's first line asserts abi_version (the ABI gate welded into the test) | [MS] 12 extern × 13 catch_unwind |
| FC-1 | the hand-written header sets the tone: wgpu-native (685 functions) is hand-written, the upstream philosophy "do not edit generated files"; the cbindgen reversal threshold = 40 functions | wgpu-native README; cbindgen README |
| FC-2 | musl/embedded defaults to crt-static making cdylib unproducible (needs `-C target-feature=-crt-static`) -> docker.md note ✓ | cargo-c troubleshooting |
| FC-3 | a cdylib without LTO does not leak rust_eh_personality (the [MS] living .so proves it) — nm==14 is preservable; no version-script (GNU-ld single script + macOS asymmetry) | [MS] build.rs/.so |
| FC-4 | the owner-tid check school, empirically (the [MS] last_error thread_local); v0 single-threaded + `!Send` documented, Mutex<Slab> governs slots only, the engine is never shared across threads | [MS] c/src:109 |
| FC-5 | style finalized: `#[unsafe(no_mangle)] pub extern "C"` + safe signature with internal validation — validation runs inside the function body, returning an error code instead of caller-site UB ([MS] implements exactly this 12×) | [MS] c/src throughout |
| FC-6 | the canonical link line + soname deferral has grounds (the [MS] build.rs `-Wl,-soname` precedent is tabbed to the release round); abidiff is in the conda library (libabigail 2.7) but the 14-function surface does not warrant it | [MS] build.rs |

### BS = binding-stack (7 houses' layering: llama/DuckDB/SQLite/MapLibre/godot/napi-rs/sql.js+duckdb-wasm)
| no. | conclusion | evidence |
|---|---|---|
| **BS-1** | **headline adoption: the VeInput `struct_size` first field** + library validation (MediaServo's blood-tears written into its own header verbatim; godot method-info uses the same device). The EP-8 conflict ruling: the EP trigger "dynamically distributed binary" **is already the now** (the .so that gate-abi verifies is exactly a dynamic-distribution artifact; sokol has lived ten years without a size field on its source-recompile contract) | [MS] header comments; gdextension_interface.h |
| BS-2 | the compatibility-direction promise, one line into CAPI-07 (godot's wording: old header -> new .so runs, never the reverse, only additions within MAJOR) | godot docs |
| BS-3 | wasm-pack is **not dead** (repo pushed 2026-08, this research corrects the task premise); the reason for using the CLI directly changes to "no conda package + fewer moving parts" | npm/github registry |
| BS-4 | CAPI-09 mirror linkage (v13 review correction): fixed numeric literals forbidden across arch (wasm32 size_t=4) — the Rust half carries two assertions `struct_size=1->-1`, `real sizeof->≠-1`; the node half's typeof bigint becomes a **static grep of the generated** `.d.ts` containing `pick(...): bigint` (node has no GPU instance, cannot construct one, a runtime typeof is infeasible, [FFI-R:v13-FEAS-1/7]) | [MS] lib.rs:257 rejection-assert precedent |
| BS-5 | route A final review: one crate with two faces is not mainstream (mainstream = one repo per language), but the **invariant = one kernel + a pure-marshaling thin face**, and a single-vendor SDK uses dual faces + a mirror guard cheaper and is immune to MapLibre-style drift; **the emscripten refusal gains language-match law** (sql.js = compiled C, duckdb-wasm = compiled C++, wgpu = Rust -> bindgen — all three fit) | the three wasm projects' READMEs |
| BS-6 | the four ownership-mode catalog (M1 borrowed/M2 free-fn/M3 caller-alloc/M4 by-value) — ours is the M1+M3 combination = the dual school of SQLite+MediaServo; **the M2 free-fn ban is maintained** (the number-one C memory-bug farm) | duckdb.h / sqlite / [MS] |
| BS-7 | "what kills bindings is ABI churn, not age": llama-cpp-node died (repo 404/stale since 2024) + node-sqlite3 DEPRECATED (the native faction stopped maintaining -> the platform built-in absorbed it, node:sqlite) — another confirmation of CAPI-07's freeze; the node face stops at abiVersion+constants (no WebGPU, no napi face built) = the right answer | each repo's live state |
| BS-8 | the proc-address registry (godot/DuckDB plugin-style) not adopted: 14 symbols + static linking + single vendor; if a third-party .so ever loads our ABI in future, the C API is the anchor (the DuckDB 1.2 precedent, the shape is already right) | the gdextension mechanism |

### EP = embed-peer (7 same-class embedding surfaces: wgpu-native/Filament/bgfx/sokol/raylib/Flutter/UaaL)
| no. | conclusion | evidence |
|---|---|---|
| EP-1 | attach(ve,win,display,kind) three scalars **CONFIRM** (webgpu.h SurfaceSource carries the same field set; the kind enum = the bgfx NATIVE_WINDOW_TYPE precedent); the RECONSIDER text surface -> **CAPI-06 pins: when kind=win32, display_ptr=hinstance (0=automatic)** | webgpu.h:3180-3350; bgfx.h:784 |
| EP-2 | the u64 handle double-confirmed (sokol 16+16+slot0 reserved); RECONSIDER = a typed struct wrapper for compile-time misuse prevention — **default: not added** (the if(!ve) idiom is already ruled), listed as a D8 informed trade-off | sokol_gfx.h:2125-2150 |
| EP-3 | input struct CONFIRM (sapp_event, same flat family); missing frame_count timestamps = the D8 trigger ② (supplied when camera inertia arrives) | sokol_app.h:1560 |
| EP-4 | the pull model, three precedents CONFIRM (Filament do-loop/bgfx_frame/sg_commit; the Unity UaaL counter-example = self-spinning + full-screen lock proves exactly the opposite side); the beginFrame()->bool frame-skip negotiation port = D8 trigger ① | Engine.h:107; uaal README |
| EP-5 | single-threaded owner CONFIRM and ours is stricter (bgfx secretly spawns a render thread = the counter-example; the Flutter platform-thread assertion = the direct precedent) | embedder.h discipline |
| EP-6 | headless = same facade CONFIRM (Filament createSwapChain(w,h) / bgfx nwh=NULL, dual precedents) | SwapChain.h:164 |
| EP-7 | the viewport() single entry CONFIRM (bgfx_reset packages present policy; our v0 exposes none of it = correct, re-adjudicate at the real-device round) | bgfx.h:2510 |
| EP-8 | the struct_size trigger argument (conflicts with BS-1, see BS-1 for the ruling: the trigger is already met -> adopt) | — |
| EP-9 | **re-attach semantics have no precedent to copy across the six houses** -> CAPI-06 must nail "repeated attach = reconfigure after releasing the old target", otherwise the implementation side keeps a race | scan of the seven houses |
| EP-10 | the Qt demo's three puzzle pieces nailed first-hand: `QWidget::winId()`->XId ✓ `QNativeInterface::QX11Application::display()` "for use with Xlib" ✓ (display_ptr uses Qt's directly); QQuickRenderControl is only needed on the QML compositing route, not the widget route (the qt-3d.md verdict maintained) | three pages of the Qt 6.11 docs |
| EP-appendix | the demo is the documentation (the canonical 10-line embedding boilerplate = the Filament loop in C, all READMEs do the same) — the I2/I3 demo code is written to the standard of "documentation pieces directly paste-able into the README" | the various READMEs |

## v13-final-review supplemental rows (second evidence batch, the v1.4 citation register)
| no. | conclusion | evidence |
|---|---|---|
| v13-CONS-1 | the reviewer's "13≠14" was a **false report** (their own line-number list proves 14 items) — the orchestrator recount + the feasibility independent count both give 14, rejected; reviewer artifacts must likewise be back-verified under C14 | sed count of the code block = 14 |
| v13-FEAS-2 | the abi assert `>>16==0` self-contradicts v0=0x00010000(major=1), coverage #6 independently confirms -> **==1** | two reviews, double evidence |
| v13-FEAS-4 | the canonical link line died three deaths: pixi 0.78 hard-parse error on `$$` / bare cc-nm absent / `-lvisiaengine` missing `[lib] name` -> the zero-$ + conda-prefixed + LD_LIBRARY_PATH variant | pixi run measured |
| FC-6 | spec-trace is not in the local ci chain (only in the standby ci.yml L33) -> the `gate-trace` task enters ci depends-on, localized; the regex is actually **three** alternations (2 in the tags line, missing one = a silent empty set) | ci.yml/pixi.toml live state |
| env-W4 | conditional-segment precedents = zero (ci.yml L35-42 all unconditional) — the smoke-web/smoke-x11 conditions = step-level shell gates, hand-written | ci.yml |
| v13-LEDG-* | ledger five spots: ready text / graveyard backfill / README 43->83 honesty debt / status doubly stale | ledger-auditor table |

## Landing summary (v1.3->v1.4)
Batch 2: VeInput+struct_size / the CAPI-06 double pin / the CAPI-07 double promise / I1 tests+3 / style+tool names finalized / D8 triggers ×4 / threadsafety header comments. Batch 7: wasm-pack wording / CAPI-09 sizeof linkage / (route A, the emscripten refusal, the no-free discipline all gain 7-house endorsement, zero changes). Cross-cutting: the docker.md musl note.
**The four lanes overturned none of the core design** (the 14 signatures, the pull model, the single entry, the generational handles, the no-callback, the thread affinity, the thin facade single source — all CONFIRM, by name and evidence); net gain four holes "missed by both Momus rounds and the team review round" — the empirical proof of the research's marginal value.
