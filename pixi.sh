#!/usr/bin/env bash
# pixi.sh — 在当前 shell 激活 VisiaEngine pixi 环境
# 用法: source pixi.sh      （不要用 bash pixi.sh——子 shell 退出即失活）
# 未装环境先跑: bash bootstrap.sh

_vdir="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd -P)"

_vpixi=""
if command -v pixi >/dev/null 2>&1; then
    _vpixi="$(command -v pixi)"
elif [ -x "$HOME/.pixi/bin/pixi" ]; then
    _vpixi="$HOME/.pixi/bin/pixi"
fi
if [ -z "$_vpixi" ]; then
    echo "未找到 pixi — 先执行: bash bootstrap.sh" >&2
    unset _vdir
    return 1
fi

# TTY 守卫（归档 H1 教训）: 非交互（管道抓取/脚本消费）不打印横幅
if [ -t 1 ]; then
    echo "激活 VisiaEngine pixi 环境 ($_vpixi)..."
fi
eval "$("$_vpixi" shell-hook --manifest-path "$_vdir/pixi.toml" --shell bash)"
if [ -t 1 ]; then
    echo ""
    echo "环境已激活。常用:"
    echo "  pixi run verify   — 工具链冒烟"
    echo "  pixi run lint     — clippy -D warnings（workspace 落地后）"
    echo "  pixi task list    — 全部任务"
    echo "退出激活: exit 或关闭本 shell"
fi
unset _vdir _vpixi
