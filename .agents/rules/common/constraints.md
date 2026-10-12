# Development Constraints

> This is a stub file. Docker & network constraints split to a topic file for on-demand loading.
>
> - [docker.md](docker.md) — Docker & network constraints (volume mounts, UDP port ranges, proxy pitfalls)

## Git Commit Rules

### Cargo.lock Must Be Committed
**ALWAYS** commit `Cargo.lock` along with dependency changes. This file tracks exact dependency versions and must be in sync with `Cargo.toml`.

Common mistake: Forgetting to `git add Cargo.lock` after `Cargo.toml` changes. This causes build failures for other developers.

**Checklist before committing:**
- [ ] `Cargo.toml` changes committed
- [ ] `Cargo.lock` changes committed (if dependencies changed)
- [ ] `git status` shows clean working tree

## pixi environment-management discipline (D5)
**Constraint**: the development environment is managed uniformly by pixi (conda-forge single source), the rust toolchain included; the rustup main path is not introduced.
- `pixi.lock` must be committed to the repository just like `Cargo.lock`; CI uses `pixi install --frozen`, and the pixi version is pinned in the workflow.
- Cross-compilation for embedded targets (vendor devices of the RK3588/Jetson class) always goes through Docker/vendor toolchains; conda rust does host builds only (conda rust injects linker priority above `.cargo/config.toml` and is untameable in cross scenarios).
- Windows is the escape-hatch scenario: standalone rustup+MSVC is allowed, but repository configuration must not fork for it.
- cargo tools with no conda package are obtained via cargo-binstall, with the version pinned in the bootstrap task.

**Check command**: `grep -rn "rustup" pixi.toml bootstrap.sh 2>/dev/null || true` (allowed only inside Windows escape-hatch comments)
