#!/usr/bin/env bash
# J3 人检服务器：装配 demo 目录（pkg + 数据 + 页）→ http://127.0.0.1:8017
# 用法：bash scripts/web-serve.sh（先决：web-build.sh 已跑）。Ctrl-C 结束。
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
D=target/web/demo
mkdir -p "$D"
cp web/demo/index.html "$D/"
cp -r target/web/pkg-web "$D/pkg"
cp resources/data/park.geojson "$D/"
echo "▶ http://127.0.0.1:8017 （T3 人检清单见页面；退出 Ctrl-C）"
cd "$D" && python3 -m http.server 8017
