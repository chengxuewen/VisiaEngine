# VisiaEngine 开发环境指南（pixi 单源，D5）

## 新机器三步

```bash
bash bootstrap.sh     # 1. 首次初始化：自动装 pixi(钉 0.78.0) → 求解安装全部依赖 → 冒烟
source pixi.sh        # 2. 日常激活进 shell（cargo/rustc/cargo-deny/rust-analyzer 全部就位）
pixi run verify       # 3. 随手自检（不激活也可：pixi run 单命令模式）
```

Windows：`bootstrap.bat` → `pixi.bat`（子 shell 形态）。**注：.bat 对等文件未经 Windows 实测，首台机器跑通前视为 best-effort**；若 conda rust 在 Windows 摩擦超标，逃生舱 = 单机 rustup+MSVC（见 D5 条款①，仓配置不分叉）。

## 环境构成

| 件 | 包（conda-forge） | 说明 |
|----|------|------|
| Rust 工具链 | `rust`（实测安装 1.98.0） | rustc/cargo/**rustfmt/clippy 由其主包提供**——`rustfmt`/`clippy` 非独立包名 |
| 审计 | `cargo-deny` 0.20.2 | `pixi run audit` |
| IDE | `rust-analyzer` + `rust-src` | 编辑器直连 `.pixi/envs/default/bin/` |
| 辅助 | `ripgrep` `ccache` `pkg-config` | — |
| wasm（冻结） | `rust-std-wasm32-unknown-unknown` `wasm-bindgen-cli` | Web SDK 到货日解开 pixi.toml 注释即用 |

任务面（`pixi task list`）：`check/build/test/lint/fmt/fmt-fix/audit/verify`。**cargo 类任务在 Cargo workspace 落地前运行必失败（接口合同，属预期）**。

## 版本与锁纪律（D5）

- 版本唯一来源 = `pixi.toml` + `pixi.lock`（**不存在 rust-toolchain.toml，本机不装 rustup**）。
- `pixi.lock` 必入库；CI 用 `pixi install --frozen`；升级工具链 = 专项 `pixi update`，评审 lock diff 后提交。
- pixi 本体钉 0.78.0（bootstrap 与 CI 同值）；升级 pixi = 两处同步改。
- 新增任何依赖走 `pixi add <pkg>`（自动维护 lock），禁止手编依赖段后忘记重锁。

## 国内镜像（个人机器配置，**不入库**）

`~/.pixi/config.toml`：

```toml
[mirrors]
"https://conda.anaconda.org/conda-forge" = [
  "https://mirrors.tuna.tsinghua.edu.cn/anaconda/cloud/conda-forge",
]
```

（schema 以 pixi 官方 docs → advanced → mirror 为准。）

## 故障排查

| 症状 | 处置 |
|------|------|
| bootstrap 预检报"无法连通 conda-forge" | 配上方镜像节；或代理环境导出 `https_proxy` 后重试 |
| 求解失败且指向某包某平台 | 按包粒度处理（IDE 类可移出该平台），结论记入 decisions.md D5 追加条款；**不回退 rustup** |
| `--frozen` 报 lock 漂移 | 有意变更→`pixi update` 提交新 lock；无意变更→`git checkout pixi.lock` |
| 磁盘占用 | `.pixi/` 实测 1.7GB（含全套工具链）；`pixi clean` 清缓存，重跑 bootstrap 即复原 |
| 卸载 | `rm -rf .pixi/ ~/.pixi` + `git rm` 配置层文件；系统零残留（pixi 自含） |

## 本机实测基线（2026-09-03，linux-64）

首跑 solve+安装 **37s**（缓存热后）· 幂等二跑 **0.24s** · `pixi run verify` 六件套全绿 · 五平台 lock 条目 linux×2:57 / osx×2:26 / win-64:42。
