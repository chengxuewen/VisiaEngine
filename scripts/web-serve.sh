#!/usr/bin/env bash
# Retired shim (T-H5.4): the demo lane lives in the build graph now.
# Build the tree, then: cmake --build cmake-build --target visiaengine-wasm-serve
echo "web-serve.sh is retired: run the visiaengine-wasm-serve button instead"
echo "  pixi run cmake --build cmake-build --target visiaengine-wasm-serve" >&2
exit 2
