# PROJECT KNOWLEDGE BASE

**Generated:** 2026-09-03 (batch 5 correction 2026-09-14) | **Branch:** main | commit / worktree state per live `git log` / `git status`

## OVERVIEW

VisiaEngine — a multi-dimensional spatial visualization engine: a unified 2D/2.5D/3D rendering pipeline for GIS / digital twin / autonomous-driving simulation / BIM display, embedded as an SDK (C API FFI) into Qt/Flutter/C#/Web, Open Core model. Stack settled 2026-09-03 in whitepaper v0.1.0: **Rust core + wgpu rendering** (D4 final ruling: use wgpu directly with the from-scratch pipeline `visiaengine-render-wgpu`, not Bevy). **Batches 0-5 wrapped (2026-09-14)**: nine-crate workspace, **152** SDD clauses (spec-trace bidirectional lock), ctest unified example registry (31 entries) + four gates (style/trace/abi/docs) + cmake-smoke three states three anchors; golden runs on real hardware with no SKIP; agent configuration was ported from the predecessor project MediaServo (Rust WebRTC, different stack — do not confuse) and neutralized.

## STRUCTURE

```
./
├── Cargo.toml/lock   # workspace (members=crates/*+examples/rs+the two FFI crates in bindings, per S1/S2); deny.toml licenses/bans
├── crates/           # 7 pure-core crates: core→render (trait+IR/camera/rebase)→render-wgpu (wgpu backend/pipeline/offscreen) + io-gltf + io-points (PLY point cloud, IO-* family) + io-text (fontdue raster annotation, IO-07..09) + geo (GeoJSON→3857→tessellation→style/GPU expansion-slice output GEO-24)
├── examples/         # examples by language: rs=cargo example tutorial series ×12 ([[example]]-registered shell package); c=C examples (E701/702; E8xx+template from S4 on); cpp/qt open with S4; single run surface = ctest/pixi smoke
├── bindings/         # implementation surface in one nest: c/visiaengine-capi (C ABI 41 entry points + handwritten header + CAPI clause tests) · c/probe · cpp/include (S4 hpp) · qt/widget.hpp · js/rust/visiaengine-wasm (CAPI-09 mirror) + js/demo
├── docs/sdd/         # behavioral contract clauses (CORE/REND/WGPU-NN, bidirectionally traced to tests via // spec: — scripts/spec-trace.sh)
├── docs/tutorials.md # E-numbered tutorial index (filename = header comment = index three-way lock = scripts/gate-docs.sh)
├── .github/workflows # ci.yml on standby (activates on GitHub mirror day; local equivalent = pixi run ci + same greps)
├── .agents/          # project memory + rules + skills (see its AGENTS.md)
├── .opencode/        # opencode.json (instructions/MCP/LSP assembly points) + init-mcp-*.mjs / init-lsp-wrap.mjs bridge scripts
├── .omo/omo.jsonc    # agent model tiering / team config; .gitignore excludes .omo/*, only omo.jsonc is tracked
├── .refinfo/         # ⚠ full MediaServo archive (21M, gitignored, local-only) — read-only, never edit
├── docs/             # whitepaper.md (primary fact source for positioning) + architecture.md (v0.1 design baseline, D4-aligned) + reference/ (reference library + evidence/ snapshots)
├── pixi.toml/lock    # D5 environment single source (conda-forge fully locked incl. rust toolchain); bootstrap.{sh,bat} first start / pixi.{sh,bat} activation
├── LICENSE-MIT / LICENSE-APACHE   # dual-license originals (irrevocability promise in README)
├── SKILL.md          # project skill registry (23 items)
└── README.md         # whitepaper digest edition
```

Engineering layer: Cargo workspace (7 crates; scale per live `cargo test --workspace`) + pixi environment (D5) + SDD clauses + CI on standby + dual license. Phase 1 MVP closed (batch 2 host embedding / batch 7 Web / batch 4 rendering hardening / batch 5 documentation rehoming); batch 4g/4h and the Alpha tier (Qt widget, npm/pip packaging, true PBR) await the go order.

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| What context each turn loads | `.opencode/opencode.json` → `instructions` (exactly 14 entries) | a new entry = fixed per-turn token cost; prove "needed every turn" before adding |
| Project status/conventions/decisions/pitfalls | `.agents/memorys/` | template state, accumulated from zero; numbering system in its AGENTS.md |
| Language rules | `.agents/rules/<lang>/` | rust/{coding-style,hooks} already in instructions (16 entries) |
| Skills | `.agents/skills/*/SKILL.md` | 23 total; frontmatter description = routing trigger surface |
| Any history of the predecessor MediaServo | `.refinfo/MediaServo/` | decision history / pitfall history / full rules — for verification only; never cite its numbering into this repo |

## CODE MAP

`Scene/EntityId`(core/src/scene.rs: slab + generation + dirty flags, 100k entities ~12ms spike measured) → `RenderBackend/MeshDesc/CameraRig`(render/src/{contract,camera}.rs: object-safe, depth variant locked [0,1] see PIT-5) → `MeshCore/headless/offscreen`(render-wgpu/src/: real mesh pipeline + cube golden) + `load_gltf`(io-gltf/src/lib.rs: gltf from_slice+util::Iter, GLB-only). Example domain (since v1.3): examples/rs ×12 cargo examples ([[example]]-registered shell package, ctest forwards through the shell) + examples/{c,cpp,qt} native bodies (E70x/E80x). Dependencies flow one way: core←{render, io-gltf}←render-wgpu (dev merged).

## CONVENTIONS

- C1 (comparison-table format) / C9 (deposit lessons immediately) / C14 (subagent artifacts must be verified) live in `.agents/memorys/conventions.md`, loaded every turn via instructions — not restated here.
- `Cargo.lock` must be committed and other build constraints sit in `rules/common/constraints.md` (inherited from the predecessor; re-audit if a non-Rust stack is ever chosen).
- remote is gitee VisiaEngine — the initial commit push was authorized (2026-09-03); pushes and further commits still wait for explicit user instruction each time.

## ANTI-PATTERNS (THIS PROJECT)

- **NEVER** edit `.refinfo/` (read-only archive) or `git add` it.
- **NEVER** cite MediaServo's D-nnn/PIT-nn/C-nn numbering in new documents — the new repo has no corresponding entries; a dangling reference is a hallucination source. After config changes run the grep gates below.
- **NEVER** scaffold / create a Cargo workspace before an explicit user ruling — stack and backend settled ≠ authorization to build (commit/landing actions always wait for user instruction). Primary facts on positioning and stack: `.agents/memorys/decisions.md` D2/D4 + the whitepaper.
- **Subagent dispatch limits** (proven this session): fast-tier model aliases fail at the gateway (`Model not exist`) → explore/librarian/quick classes die. Available: `deep`/`unspecified-high`/`visual-engineering`/`writing` (premium).
- **NEVER** run `cargo fmt` indiscriminately (workspace format-drift history) — single files via `rustfmt --edition <ed> <file>`.

## UNIQUE STYLES

- Configuration-as-product: the `.agents` three layers (memorys = volatile facts / rules = constant constraints / skills = on-demand depth) and the layering discipline of the opencode.json assembly points are this repo's core invariant.

## COMMANDS

```bash
# config hygiene gate (run after touching .agents/.opencode; expectation: only memorys archive pointers / historical entries hit)
grep -rniE 'mediaservo|audemsp|mediasoup|omsp|msrtc|oxmgr' .agents .opencode --include='*.md' --include='*.json' | grep -v '.refinfo'
grep -rnE 'PIT-[0-9]+|D1[0-9][0-9]|D2[0-9][0-9]' .agents/rules .agents/skills
python3 -m json.tool .opencode/opencode.json >/dev/null && echo json-OK
```
bash bootstrap.sh    # first-time environment init (idempotent; measured 0.24s on second run; pixi pinned 0.78.0)
source pixi.sh       # daily activation (or one-shot pixi run <task>)
pixi run ci          # fmt+lint+check+test+audit aggregate (start-of-work gate)
pixi run <check|build|test|lint|fmt|audit|verify>   # single task
source pixi.sh && pixi run pack-check   # installed-tree hygiene (manifest / NEEDED self-sufficiency / no build paths; RUNPATH reported only, cleaning belongs to the publish band)
pixi run keys-probe     # XTEST input probe (private Xvfb display; real key/wheel/drag injection, red if the picture does not move; not in the ci chain)
bash scripts/spec-trace.sh    # SDD↔test bidirectional tracing

## NOTES

- `.sisyphus` is reserved at `.gitignore` L92 (plans stay out of the repo); `www/` is NOT reserved — the project is unbuilt, do not assume the directory exists.
- MCP (after the 2026-09-03 fix round): nodejs lives in the pixi default env, all local bridges launch via `bash .opencode/with-node.sh`; local-github still needs the `GITHUB_TOKEN` environment variable (environment key, not env); local-playwright/postgres/websearch/openspace disabled by default, enable on demand (openspace additionally needs a python tree + LLM key).
- Rust-stack backfill points (unblock when due): instructions += `rules/rust/{coding-style,hooks}.md`; `rules/rust/testing.md` L38 placeholder calibration; docker.md kept as on-demand reference.
