#!/bin/sh
# 交互 run 步骤启动包装：IDE/远程 shell 常无 DISPLAY（本仓实测：vscode-server 进程树无显示变量）。
# 规则：三变量全空 且 本机存在 :0 X socket → 回退 DISPLAY=:0（本仓桌面形态）；否则原样 exec，
# 让 winit/SDL 报清晰错误。仅 cargo-run_* 与本地手跑使用——CI/ctest 绝不经过本包装。
if [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ] && [ -z "${WAYLAND_SOCKET:-}" ] && [ -S /tmp/.X11-unix/X0 ]; then
    DISPLAY=:0; export DISPLAY
    echo "run-gui: 环境无显示变量，回退 DISPLAY=:0（本机 X socket 探测）" >&2
fi
exec "$@"
