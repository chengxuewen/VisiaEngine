#!/usr/bin/env bash
# bootstrap.sh — VisiaEngine 开发环境首次初始化（幂等，可重复执行）
# 用法: bash bootstrap.sh
# D5: 环境全由 pixi 管理（conda-forge 单源，rust 工具链含在内）。本机永不引入 rustup。
# 注: 不使用 set -e——脚本设计为可被 source 而不污染用户 shell 的 errexit 状态，错误均显式处理

set -uo pipefail

BOOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"
PIXI_PIN="0.78.0"   # D5 条款④: 版本钉死（CI 同步钉此值）

ver_ge() { [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -n1)" = "$2" ]; }

pixi_version() { "$1" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -n1; }

# --- [1/3] 定位或安装 pixi ---
PIXI=""
if command -v pixi >/dev/null 2>&1; then
    PIXI="$(command -v pixi)"
elif [ -x "$HOME/.pixi/bin/pixi" ]; then
    PIXI="$HOME/.pixi/bin/pixi"
fi

if [ -n "$PIXI" ]; then
    have="$(pixi_version "$PIXI")"
    if ver_ge "${have:-0}" "$PIXI_PIN"; then
        echo "[1/3] pixi ${have} 已就绪 (${PIXI})"
    else
        echo "[1/3] 现存 pixi ${have} 低于钉死版本 ${PIXI_PIN}，改装至 ~/.pixi ..."
        PIXI=""
    fi
fi

if [ -z "$PIXI" ]; then
    echo "[1/3] 预检网络: conda-forge 可达性..."
    if ! curl -sI --max-time 10 https://conda.anaconda.org/conda-forge/noarch/repodata.json >/dev/null 2>&1; then
        echo "ERROR: 无法连通 conda-forge。配置镜像后重试（见 docs/env.md 镜像节）" >&2
        exit 1
    fi
    tmp_inst="$(mktemp)"
    if ! curl -fsSL --proto '=https' --tlsv1.2 "https://pixi.sh/install.sh" -o "$tmp_inst"; then
        echo "ERROR: pixi 安装器下载失败" >&2
        rm -f "$tmp_inst"
        exit 1
    fi
    # 落盘后执行（可审计），非 curl|sh 直管；版本断言在下方兜底
    PIXI_VERSION="v${PIXI_PIN}" bash "$tmp_inst"
    rc=$?
    rm -f "$tmp_inst"
    [ "$rc" -eq 0 ] || { echo "ERROR: pixi 安装器执行失败 (rc=$rc)" >&2; exit 1; }
    PIXI="$HOME/.pixi/bin/pixi"
fi

got="$(pixi_version "$PIXI")"
if ! ver_ge "${got:-0}" "$PIXI_PIN"; then
    echo "ERROR: pixi 版本断言失败: 得到 '${got:-无}'，要求 >= ${PIXI_PIN}" >&2
    exit 1
fi

# --- [2/3] 依赖求解与安装 ---
echo "[2/3] pixi install（conda-forge 首次求解含下载，可能数分钟）..."
if ! "$PIXI" install --manifest-path "$BOOT_DIR/pixi.toml"; then
    echo "      首次求解失败 → 重生成 lock 后重试一次（锁漂移恢复，归档实证模式）"
    "$PIXI" update --manifest-path "$BOOT_DIR/pixi.toml" || { echo "ERROR: pixi update 失败" >&2; exit 1; }
    "$PIXI" install --manifest-path "$BOOT_DIR/pixi.toml" || { echo "ERROR: pixi install 失败" >&2; exit 1; }
fi

# --- [3/3] 冒烟 ---
echo "[3/3] 冒烟: pixi run verify ..."
"$PIXI" run --manifest-path "$BOOT_DIR/pixi.toml" verify || { echo "ERROR: verify 失败" >&2; exit 1; }

echo ""
echo "=============== VisiaEngine 环境就绪 ==============="
echo "  日常激活:  source pixi.sh"
echo "  单命令:    pixi run <task>     （全部任务: pixi task list）"
echo "  指南:      docs/env.md"
echo "===================================================="
