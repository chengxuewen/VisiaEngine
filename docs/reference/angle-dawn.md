# ANGLE / Dawn — the two titans of translation layers

**Snapshot 2026-09-03** (shields measured): google/angle 4.1k★, last-commit ≈2026-09-01 (active, follows Chromium); google/dawn 1.1k★, pushed the same day. BSD-3-Clause (angle). **No maintained ANGLE binding on crates.io** (`angle` crate 0.5.0, dead since 2022).

## Profile
- **ANGLE**: GLES/EGL **front-end** -> D3D11/D3D11on12/Vulkan/Metal/GL multi-backend translation layer. Its sole reason to exist = supplying stable GLES on platforms that do not offer a conformant one (Windows->D3D11, AMD/Intel Linux->GL, ANGLE-on-Vulkan on Chrome OS/Android). It is **the most widely deployed graphics translation layer on the planet** (the WebGL of Chrome/Edge/Firefox is ANGLE; the Flutter Windows engine is confirmed to link ANGLE (BUILD.gn evidence); **Qt6 refuted** — QRhi's D3D11 is a native backend, qtbase 3rdparty carries no angle, only the Qt5-era `-opengl es2 -angle` was the translation-layer usage).
- **Dawn**: Google's WebGPU native (C++), same API family as wgpu, different implementation. Comparison value: the engineering shape of Chromium's "platform-difference absorption layer".

## For VisiaEngine
- **Core proposition (verified, original hypothesis overturned)**: wgpu v30 **has native GL/GLES backends** (Windows GL3.3+ defaults to WGL / Linux-Android GLES3.0+, downlevel tier) and **ships its own ANGLE build options** (Windows `cfg(windows_angle)`, macOS `angle` feature, README 📐 note) -> "integrating ANGLE standalone" is a pseudo-need: the legacy cost drops from "build a second GLES render backend in-house" to "a build switch + QA matrix".
- **Adopt**: the backend capability matrix + workaround mechanism (ANGLE's per-device workaround table for buggy GL drivers) = the best industrial template for SDK compatibility-matrix management; its "feature level" probing -> quality tiering (maps to the whitepaper's compatibility-tier language).
- **Avoid**: ① the ops tax of GN/ninja C++ build chains inside a cargo project (precompiled distribution / size ~2-4 MB *UNCERTAIN*); ② the performance tax of the D3D11 translation layer (buffer upload, no async compute) — on heavy 3D scenes use it only as a backstop, not the main road; ③ do not conflate ANGLE-on-Vulkan and ANGLE-on-D3D11 into one solution (they fix opposite device problems: the former "has Vulkan, no good GLES", the latter "has nothing but D3D11").
- **XP verdict (closed)**: the ANGLE D3D9 backend was removed from main in 2026-04~07 (commit-level evidence) + Rust MSVC officially requires Win10+ (rustc book verified the same day) -> XP is unsolvable, removed from the commitment text. Win7 = unofficial best-effort (technically reachable: wgpu-GL/ANGLE-D3D11, but pinned to an old toolchain); matrix in [legacy-platforms.md](legacy-platforms.md).

## Status
✅ research complete: all evidence in [2026-09-03-angle-integration.md](evidence/2026-09-03-angle-integration.md) (grep.app commit-level; incl. the Qt6 refutation, the finding that ANGLE-on-Vulkan is a red herring for Android, and the §3.3/§4 suggested wording).

## Sources
img.shields.io (2026-09-03); crates.io; to be cited: the ANGLE research memo.
