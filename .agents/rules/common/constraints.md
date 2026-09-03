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

## pixi 环境管理纪律（D5）
**约束**：开发环境统一由 pixi（conda-forge 单源）管理，rust 工具链含在内；不引入 rustup 主路径。
- `pixi.lock` 与 `Cargo.lock` 同样**必入库**；CI 用 `pixi install --frozen`，pixi 版本在 workflow 中钉死。
- 嵌入式目标（RK3588/Jetson 类厂商设备）交叉编译**一律 Docker/厂商工具链**，conda rust 只做 host 编译（conda rust 注入链接器优先级高于 `.cargo/config.toml`，交叉场景不可驯服）。
- Windows 为逃生舱场景：单机 rustup+MSVC 允许，仓库配置不得为此分叉。
- conda 无包的 cargo 工具经 cargo-binstall 获取，并在 bootstrap 任务中固定版本。

**检查命令**：`grep -rn "rustup" pixi.toml bootstrap.sh 2>/dev/null || true`（仅允许出现在 Windows 逃生注释中）
