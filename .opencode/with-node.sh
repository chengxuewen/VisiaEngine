#!/usr/bin/env bash
# with-node.sh — MCP 启动包装：解析 node 运行时后透传执行 "$@"。
# 顺序：仓内 pixi env（D5 单源）→ 用户级 pixi env → PATH。
# 附带把 node 目录前插 PATH——桥脚本子进程调用 npm/npx 依赖之（绝对路径 node 直启做不到这点）。
set -u
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# pixi 本体注入（openspace 等桥脚本子进程 execSync('pixi ...') 依赖之）
if ! command -v pixi >/dev/null 2>&1 && [ -x "$HOME/.pixi/bin/pixi" ]; then
    export PATH="$HOME/.pixi/bin:$PATH"
fi

for cand in "$DIR/../.pixi/envs/default/bin" "$HOME/.pixi/envs/default/bin"; do
    if [ -x "$cand/node" ]; then
        export PATH="$cand:$PATH"
        exec "$@"
    fi
done

if command -v node >/dev/null 2>&1; then
    exec "$@"
fi

echo "with-node.sh: 未找到 node 运行时（先执行 pixi install 或 source pixi.sh）" >&2
exit 127
