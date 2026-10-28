# VisiaEngine development-environment guide (pixi single source, D5)

## Three steps on a new machine

```bash
bash bootstrap.sh     # 1. first init: auto-installs pixi (pinned 0.78.0) -> solves & installs every dependency -> smoke
source pixi.sh        # 2. daily activation into the shell (cargo/rustc/cargo-deny/rust-analyzer all in place)
./visiaengine.sh verify   # 3. quick self-check anytime (no activation needed: one command per invocation)
```

Windows: `bootstrap.bat` -> `pixi.bat` (sub-shell form). **Note: the .bat counterparts are untested on Windows; treat them as best-effort until the first machine runs green**; if conda rust causes too much friction on Windows, the escape hatch = per-machine rustup+MSVC (see D5 clause ①; the repo configuration does not fork).

## Entry points (do not confuse them)

| Script | How to run it | What it is for |
|---|---|---|
| `bootstrap.sh` / `.bat` | execute | one-time first init (pixi + dependencies + smoke) |
| `pixi.sh` / `.bat` / `.ps1` | **`source`** / sub-shell | activate this shell so `cargo`, `rustc`, `node` resolve |
| `visiaengine.sh` / `.bat` | execute | task front door: `./visiaengine.sh <task> [args...]` |

`./visiaengine.sh <task>` is a **pure passthrough** to `pixi run <task>` — it holds no command table, so it cannot drift from `pixi.toml`. `-h` prints a few common examples (exit 0); no arguments prints usage (exit 2); a missing `pixi` exits 127.

Who uses which form:

| Audience | Form | Why |
|---|---|---|
| Humans, daily work | `./visiaengine.sh <task>` | shorter, discoverable via `-h`, and daily use keeps the wrapper exercised |
| Agents, skills, CI | `pixi run <task>` | no intermediate layer, OS-neutral, does not assume the wrapper exists |

Both forms are always equivalent; `pixi run <task>` is the fallback whenever the wrapper is unavailable (Windows without the `.bat`, a stripped-down container, etc.).

## GUI apps / MCP hosts need `pixi` on their inherited PATH (O5)

`bootstrap.sh` / `bootstrap.bat` only fix the *terminal* environment. Desktop apps (CodeBuddy, opencode, ...) are launched from the desktop and **do not source shell rc files**, so a `pixi` that lives only in `~/.pixi/bin` (Linux) is invisible to them — MCP servers then fail with `Executable not found in $PATH: "pixi"`.

Fix once per machine, on the PATH the GUI inherits:
- **Linux/macOS**: `ln -s "$HOME/.pixi/bin/pixi" "$HOME/.local/bin/pixi"` (user-level, no sudo; `~/.local/bin` is on the ambient PATH). If the desktop session does not include `~/.local/bin`, use `/usr/local/bin/pixi` (needs sudo) or the desktop session env.
- **Windows**: the pixi installer adds `%USERPROFILE%\.pixi\bin` to the *user* PATH env var, which GUI apps inherit — verify with `where pixi` in a fresh shell.

Verify: `command -v pixi` in a shell that did **not** `source pixi.sh` must resolve. `pixi run gate-mcp` probes under this sanitized PATH, so the regression is caught in CI.

The generated project-scope MCP file (root `.mcp.json`) uses `scripts/mcp-launch.sh` / `mcp-launch.bat` instead, which resolve pixi themselves and are immune to the spawn PATH / cwd.

### MCP lifecycle: install, index, status

All local MCP maintenance goes through one task (`scripts/mcp-cli.py`):

```bash
./visiaengine.sh mcp setup    # install/repair every server at the pinned version (idempotent, --save-exact)
./visiaengine.sh mcp index    # first run: build .codegraph; later runs: full rebuild
./visiaengine.sh mcp sync     # incremental sync (the watcher covers normal editing)
./visiaengine.sh mcp status   # pin vs installed versions + index state (read-only)
```

- Pins live in `scripts/mcp-servers.json` (single source); `package.json`, `package-lock.json` and `node_modules/` are gitignored local echoes.
- `.codegraph/` is a machine-local cache (gitignored) and is **not** built in CI -- build it once per machine.
- The MCP host spawns `scripts/mcp-launch.sh`: a missing binary is installed on the spot, but a **pin drift fails fast** with `run: pixi run mcp setup` instead of silently reinstalling (an install would write to stdout, which is the JSON-RPC channel).

### Serena MCP (semantic/LSP server, complements codegraph)

Runs in the isolated pixi environment `serena` (Python 3.13 + pypi `serena-agent`, pinned in `pixi.lock`; `rust-analyzer` + `clang-tools` provide the language servers). The registry entry is `kind: "pixi"` in `scripts/mcp-servers.json`: install is declarative (`pixi install -e serena`), `gate-mcp` probes it via `<env>/bin/serena --version` and **never starts the server** (it would spawn language servers), and `pixi run mcp status` reports it next to codegraph.

```bash
./visiaengine.sh mcp status    # codegraph (node_modules) + serena (pixi env), one line each
pixi run -e serena serena --version   # raw version
```

- `.serena/`: `project.yml` is **tracked** (reviewed config: language servers, backend, tool trimming); `cache/`, `memories/` and `project.local.yml` are **ignored** — Serena memories must NOT become a second memory store (single source: `.agents/memorys`, D41).
- Tool surface: the default agent interface exposes **2 MCP tools** (`initial_instructions`, `serena_repl`) — same minimal-surface philosophy as codegraph; trimming is available via `excluded_tools` / `--mode` in `project.yml`.
- Context: `--context claude-code` (`ide-assistant` is deprecated in Serena 2.1.0).
- C++ coverage needs `compile_commands.json` (CMake presets export it, but only inside a build tree — configure once and link it at the repo root, see the gitignored `compile_commands.json`); without it clangd falls back to guessed flags.

## Environment composition

| piece | package (conda-forge) | notes |
|----|------|------|
| Rust toolchain | `rust` (measured install 1.98.0) | rustc/cargo/**rustfmt/clippy are provided by its main package** — `rustfmt`/`clippy` are not standalone package names |
| audit | `cargo-deny` 0.20.2 | `pixi run audit` |
| IDE | `rust-analyzer` + `rust-src` | the editor connects directly to `.pixi/envs/default/bin/` |
| helpers | `ripgrep` `ccache` `pkg-config` | — |
| wasm (frozen) | `rust-std-wasm32-unknown-unknown` `wasm-bindgen-cli` | on Web-SDK delivery day, uncomment in pixi.toml and use |

Task surface (`pixi task list`): `check/build/test/lint/fmt/fmt-fix/audit/verify`. **cargo-class tasks are guaranteed to fail before the Cargo workspace lands (an interface contract, by design)**.

## Version and lock discipline (D5)

- The single source of versions = `pixi.toml` + `pixi.lock` (**there is no rust-toolchain.toml, and rustup is not installed on this machine**).
- `pixi.lock` must be committed; CI uses `pixi install --frozen`; upgrading the toolchain = a dedicated `pixi update`, commit only after reviewing the lock diff.
- pixi itself pinned 0.78.0; the pin lives in two places -- CI (`.github/workflows/ci.yml`, setup-pixi) and the upstream PolyOrch `pixi-init` default (the bootstrap scripts inherit it); upgrading pixi = bump both in sync.
- Any new dependency goes through `pixi add <pkg>` (maintains the lock automatically); hand-editing the dependency section and forgetting to relock is forbidden.

## Domestic mirrors (personal-machine configuration, **not committed**)

`~/.pixi/config.toml`:

```toml
[mirrors]
"https://conda.anaconda.org/conda-forge" = [
  "https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/conda-forge",
]
```

(the schema is authoritative per pixi's official docs -> advanced -> mirror.)

## Troubleshooting

| symptom | handling |
|------|------|
| bootstrap pre-check reports "cannot reach conda-forge" | configure the mirror section above; or in a proxied environment export `https_proxy` and retry |
| solving fails pointing at one package on one platform | handle at package granularity (IDE-class may be moved off that platform); record the conclusion as an addendum to decisions.md D5; **do not fall back to rustup** |
| `--frozen` reports lock drift | intentional change -> `pixi update` and commit the new lock; unintentional -> `git checkout pixi.lock` |
| disk usage | `.pixi/` measured 1.7 GB (incl. the full toolchain); `pixi clean` clears the cache, re-run bootstrap to restore |
| uninstall | `rm -rf .pixi/ ~/.pixi` + `git rm` the configuration-layer files; zero system residue (pixi is self-contained) |

## This machine's measured baseline (2026-09-03, linux-64)

First-run solve+install **37s** (after the cache warms) · idempotent second run **0.24s** · `pixi run verify` six-piece suite fully green · five-platform lock entries linux×2:57 / osx×2:26 / win-64:42.

## CMake preset build trees (T-A0' 2026-10-10)

The CMake build trees live in `cmake-build*/` (VSCode picks a preset and goes). Mind the difference from the old `target/cmake-rust` shared incremental surface: now **the cargo cache is isolated per build tree** (PolyOrch defaults `.cargo-target/` inside each cmake-build*) — each preset's first build compiles cargo fully once; what that buys is build-directory self-sufficiency (deleting the tree = deleting everything) and parallel presets. The save-frame thumbnail pipeline owns `build/gallery/assets/` (wall thumbnail source), direct cargo calls own `target/` exclusively.
The committed `.vscode` debug lists are refreshed by `pixi run f5` — the `debug` preset is the only tree that sees every lane (Qt on + the debug knob); no other configure touches them.

## Qt support matrix (R2, 2026-10-15)

`-DVISIAENGINE_QT=ON` resolves two generations, Qt6 preferred then Qt5: **Qt 6.x** (pixi `qt-spike` env, the CI lane — proven by `xvfb-run -a pixi run -e qt-spike smoke-qt`) and **Qt >= 5.15** (pixi `qt5-spike` validation env, conda-forge `qt 5.15.*`; validated live at 5.15.15 — discovery, E703 real-window run and the ctest leg all green). **Qt 5.12 and older are excluded from the contract** (owner directive 2026-10-15): the `find_package(Qt5 5.15)` version arg enforces the floor and the widget header carries exactly one version guard (`QMouseEvent::position()` on Qt6 vs `pos()` on Qt5 — the only compile-measured divergence).

Consumers never spell a generation name: the SDK publishes the canonical `VISIAENGINE_QT_*` variables (FOUND / MAJOR / VERSION / TARGET / LIBDIR / PLUGINSDIR — the last two probed per packaging shape, conda and Debian layouts both resolve) and the example family consumes them inside `visiaengine_add_example(... QT)`. The `qt5-spike` env is validation-only: no solve-group, no ci chain.

## Skill naming convention (future additions)

- Pattern: `<domain>-<activity>`, 2–4 lowercase hyphenated words (e.g. `rust-engineering`, `shipping-and-launch`).
- Avoid names already owned by global plugins / marketplaces (`test-driven-development`, `debugging`, `frontend`, `git-master`); pick a specific name instead.
- Do not rename existing skills for style alone — renaming breaks cross-references (`skill-router`, `doc-audit`, `AGENTS.md` SKILL DIRECTORY) and muscle memory.
- A colon namespace (`plugin:skill`) is a plugin-loader feature; it cannot be encoded in a directory name (illegal on Windows) and is not used here.

## Agent config sync: one entry (`pixi run agents`)

`pixi run agents sync|check|status` is the unified entry. It **delegates** to
`scripts/agent-links.py` and `scripts/skills-links.py`:

- `sync` — regenerate every generated host layer (rules / mcp / memory / hub indexes / skills links).
- `check` — same as `sync`'s verification; exits non-zero on drift.
- `status` — read-only, always exits 0; shows both reports and the worst code.

The older `agent-sync`, `gate-agents`, and `gate-skills` tasks remain valid aliases; `ci`
keeps the two gates separate (so the CONSTRAINTS gate list is unchanged). `visiaengine.sh
agents <sub>` works through the passthrough.
