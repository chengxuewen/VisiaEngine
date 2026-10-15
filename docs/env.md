# VisiaEngine development-environment guide (pixi single source, D5)

## Three steps on a new machine

```bash
bash bootstrap.sh     # 1. first init: auto-installs pixi (pinned 0.78.0) -> solves & installs every dependency -> smoke
source pixi.sh        # 2. daily activation into the shell (cargo/rustc/cargo-deny/rust-analyzer all in place)
pixi run verify       # 3. quick self-check anytime (works without activation: pixi run single-command mode)
```

Windows: `bootstrap.bat` -> `pixi.bat` (sub-shell form). **Note: the .bat counterparts are untested on Windows; treat them as best-effort until the first machine runs green**; if conda rust causes too much friction on Windows, the escape hatch = per-machine rustup+MSVC (see D5 clause ①; the repo configuration does not fork).

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
