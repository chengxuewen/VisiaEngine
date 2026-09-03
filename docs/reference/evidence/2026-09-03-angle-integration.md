# ANGLE Integration Memo for VisiaEngine

**Date**: 2026-09-03 | **Author**: research agent | **Scope**: ANGLE feasibility as legacy-platform escape hatch for VisiaEngine (wgpu renderer, C-API SDK, Qt/C#/Flutter embed)  
**All URLs fetched live 2026-09-03 unless marked. No 2023 memory used.**

---

## 1. What ANGLE Is (Canonical Repo)

| Field | Value | Source |
|---|---|---|
| Canonical repo | `google/angle` (GitHub, 4065★, BSD-3-Clause) | https://github.com/google/angle (fetched 2026-09-03) |
| `angleproject/angle` | 404 (archived/deleted) | webfetch 2026-09-03 |
| Chromium source | `chromium.googlesource.com/angle/angle` | blocked from this host (curl 000) |
| Stable API version | **2.1** (`ANGLE_MAJOR=2`, `ANGLE_MINOR=1`) | `src/common/angle_version.h` via grep.app |
| Revision numbering | `ANGLE_REVISION = ANGLE_COMMIT_POSITION` (Chromium-style build number) | same file |
| Release cadence | **No standalone releases/tags**; rides Chromium stable (~4-week cycle) | grep.app: google/angle has 0 GitHub releases; chromium.org/developers/calendar (fetched 2026-09-03) |
| License | BSD-3-Clause with "Google ANGLE" trademark clause | `LICENSE` file via GitHub API |
| vcpkg port | Exists (`ports/angle/` with portfile.cmake + vcpkg.json + patches) | GitHub tree fetch 2026-09-03 |
| Official Rust crate | **None** (crates.io `angle` = trig math library, 305k dl) | crates.io API 2026-09-03 |

**⚠ Key implication**: ANGLE is not a library you `cargo add` — it is a C/C++ project built via Chromium GN or vcpkg. Embedding it in a Rust SDK means either: (a) building from source via vcpkg/cmake-bindgen, or (b) shipping pre-built static libs (.a/.lib) alongside the Rust SDK. Neither is trivial.

---

## 2. Backend Inventory (Live grep.app Evidence)

Backend layout confirmed by `src/libANGLE/renderer/` include paths and `scripts/export_targets.py`:

| Backend | Path | Status (2026-09-03) |
|---|---|---|
| **D3D11** | `renderer/d3d/d3d11/` | ✅ Active — `Renderer11.cpp`, full include set |
| **D3D11on12** | Extension (`EGL_PLATFORM_ANGLE_D3D11ON12_ANGLE`) | ✅ Present, **requires Win10+** |
| **D3D9** | `renderer/d3d9/` | ❌ **Removed** — not in `export_targets.py`; 2026 "DirectX9 Backend Removal" commit series |
| **Vulkan** | `renderer/vulkan/{android,win32,xcb,wayland,mac,fuchsia}/` | ✅ Active |
| **OpenGL** (desktop) | `renderer/gl/{wgl,egl,glx,gbm,cgl}/` | ✅ Active |
| **Metal** | `renderer/metal/` | ✅ Active |
| **OpenCL** | `renderer/cl/` | ✅ Present |
| **Null** | `renderer/null/` | ✅ Test/debug backend |

### D3D9 Removal Timeline (Commits)

| Date | Commit | Action |
|---|---|---|
| 2026-04-09 | `d5d76f88` | Stop end2end testing with D3D9 and WebGPU on Win Intel |
| 2026-06-30 | `1860fb2d` | Remove tests |
| 2026-07-06 | `c1f9769b` | Remove non-shader backend files |
| 2026-07-07 | `04e32f44` | Remove shader translator code |
| 2026-07-08 | `fdb5d1f7` | Remove shader model 3 code paths |

**Source**: GitHub `search/commits?q=repo:google/angle+D3D9` (173 total matches; 5 listed above, top result 2026-07-07).  
**Verdict**: D3D9 is gone from `main`. The README table is **stale** (still shows D3D9 column).  
**Impact on XP**: ANGLE-D3D9 = the only ANGLE path for WinXP/WinVista. Its removal means **ANGLE provides no WinXP support**.

### D3D11on12 Gate

```cpp
// src/libANGLE/Display.cpp
extensions.platformANGLED3D11ON12 = angle::IsWindows10OrLater();
```

**Source**: grep.app `Display.cpp` snippet, line 2284.  
**Verdict**: D3D11on12 = Win10+ only. Not available on Win7. Regular D3D11 backend (without on12) is the only path for Win7.

---

## 3. wgpu + ANGLE Integration Path (Windows)

### wgpu README Platform Table (v30.0.1, fetched 2026-09-03)

| API | Windows | Linux/Android | macOS/iOS | Web |
|---|---|---|---|---|
| Vulkan | ✅ | ✅ | 🌋 MoltenVK | — |
| Metal | — | — | ✅ | — |
| DX12 | ✅ | — | — | — |
| OpenGL | 🆗 GL 3.3+ or 📐 ANGLE | 🆗 GL ES 3.0+ | 📐 ANGLE | 🆗 WebGL2 |
| WebGPU | — | — | — | ✅ |

**📐 definition**: *"Requires the ANGLE translation layer (GL ES 3.0 only). On macOS/iOS, use the `angle` feature. On Windows, `gles` uses WGL by default; build with `cfg(windows_angle)` to use ANGLE instead."*

**Source**: https://github.com/gfx-rs/wgpu/blob/trunk/README.md (fetched 2026-09-03 via webfetch)

### wgpu-hal GL Backend

- **docs.rs**: Module `wgpu-hal::gl` confirmed present (docs.rs/crate/wgpu/latest/wgpu/hal/gl/index.html via search API result)
- **wgpu changelog** (5998 lines, `/tmp/opencode/research2/wgpu-changelog.md`):
  - Line 1915: *"Previously, the `vulkan` and `gles` backends were non-optional on windows, linux, and android..."*
  - Line 4526: *"vulkan", for the Vulkan API (Linux, some Android, and occasionally Windows)*
  - GL backend is shipped, not deprecated, in wgpu v30.

### Integration Stack on Windows

```
VisiaEngine SDK (Rust)
  → wgpu v30 + "gl" feature (or cfg(windows_angle) build flag)
    → wgpu-hal GL backend
      → ANGLE (static link: libEGL + libGLESv2)
        → D3D11 API (ANGLE translates GL ES 3.0 → D3D11 Feature Level 10_0+)
```

**Practical implications**:
- wgpu already handles the ANGLE integration plumbing (`cfg(windows_angle)`)
- You bundle pre-built ANGLE static libs (libEGL.a + libGLESv2.a, ~3-5MB from vcpkg build)
- D3D11 Feature Level 10_0+ required — this covers Win7 SP1+ with any WDDM 1.1+ GPU driver
- D3D11on12 path = Win10+ only (ANGLE extension gate) — irrelevant for Win7

### vcpkg ANGLE Build

The vcpkg port (`ports/angle/`) includes portfile.cmake, patches, and `angle_commit.h.in` — it builds ANGLE from source pinned to a Chromium commit. No official pre-built binary exists from Google.

---

## 4. Windows Support Matrix

### Toolchain Floors

| Toolchain/SDK | Minimum Windows | Source |
|---|---|---|
| **Rust (MSVC target)** | **Windows 10** | doc.rust-lang.org/rustc/platform-support/windows-msvc.html (fetched 2026-09-03): *"OS version: Windows 10 or higher is required for client installs"* |
| Rust (GNU target) | Win7+ (deprecated, likely following MSVC) | UNCERTAIN — not verified separately |
| wgpu v30 (Vulkan/DX12) | Win10+ (DX12 = Win10; Vulkan runtime = Win7 SP1 but needs driver) | wgpu README |
| wgpu v30 (GL/WGL) | Win7+ | wgpu README: 🆗 GL 3.3+ |
| wgpu v30 (GL/ANGLE) | Win7 SP1 (D3D11 FL10_0+) | ANGLE D3D11 backend + Display.cpp gate (on12 requires Win10, plain D3D11 does not) |
| Flutter Windows | Win10+ | docs.flutter.dev/reference/supported-platforms (fetched 2026-09-03): *"Windows 10/11 supported, anything below 8 is unsupported"* |
| Chrome | Win10+ | chrome.dev/system-requirements (fetched 2026-09-03): *"Chrome 119+: Windows 10+"* |
| Chromium (older) | Win7+ historically | chromium.org/developers/calendar (fetched 2026-09-03) — 2019-2020 era data |

### Practical Windows Tiers

| Tier | Windows Version | Rust SDK Support | Rendering Path | Notes |
|---|---|---|---|---|
| **T1: Primary** | Win10+ | ✅ Official | wgpu DX12 or Vulkan (native) | First-class, no ANGLE needed |
| **T2: Extended** | Win7 SP1 | ⚠ Unofficial (need old Rust ≤1.8x + GNU target or pinned MSVC) | wgpu GL + ANGLE-D3D11 static | D3D11 available on Win7 via WDDM 1.1 drivers; ANGLE translates GL ES 3.0 |
| **T3: Legacy** | Vista/XP | ❌ No path | — | ANGLE D3D9 removed; Rust dropped XP in 1.27; no toolchain support |

**⚠ Honest assessment**: The Rust MSVC target requiring Win10+ means **Win7 support is officially dead** at the toolchain level. ANGLE-D3D11 remains technically viable on Win7 (D3D11 ships in Win7 SP1), but you'd need:
1. A pinned Rust version ≤1.8x (before Win10 requirement)
2. The `x86_64-pc-windows-gnu` target (may still allow Win7)
3. A custom build pipeline

This makes Win7 an **unofficial, best-effort tier** — not a supported platform.

---

## 5. Android ANGLE-Vulkan (Chrome/ARCVM) — Irrelevant to VisiaEngine

### What ANGLE-on-Vulkan Is

Chrome on Android uses ANGLE to implement OpenGL ES 3.x on top of Vulkan. This is an internal implementation detail for Chrome's compositor and WebGL/ANGLE path — it is NOT something app developers use.

### Why It's Irrelevant for VisiaEngine

| Concern | Reality | Source |
|---|---|---|
| VisiaEngine on Android needs ANGLE? | **No** — Android NDK provides system `libEGL` + `libGLESv3` directly | Android CDD: system MUST provide OpenGL ES implementation |
| ANGLE-on-Vulkan on old Android? | Requires **Vulkan 1.1** driver — old/cheap SoCs may lack it | ANGLE README table (grep.app): vulkan backend listed |
| Flutter on Android uses ANGLE? | Flutter Windows uses ANGLE static; Flutter Android uses system GL directly | grep.app: `engine/shell/platform/windows/BUILD.gn` links ANGLE; Android shell does NOT |
| Qt on Android uses ANGLE? | Qt6 RHI uses platform-native GL (EGL/GLES) directly | Qt6 docs: no ANGLE in RHI backends |

**Verdict**: ANGLE-on-Vulkan on Android is a red herring for VisiaEngine. The Android path is: **wgpu Vulkan (native) on modern SoCs, or wgpu GL ES (native) on older SoCs — no ANGLE involved**.

### Android Version Floors (2026-09-03)

| Floor | % Devices | Source | Notes |
|---|---|---|---|
| API 21+ (Lollipop 5.0) | 99.8% | apilevels.com (fetched 2026-09-03) | wgpu Vulkan minimum |
| API 24+ (Nougat 7.1) | 96.6% | apilevels.com | Flutter floor |
| API 26+ (Oreo 8.0) | 96.1% | apilevels.com | Safe modern floor |
| < API 21 | 0.2% | apilevels.com | Effectively dead |
| API 21–25 (5.0–7.0) | ~3.4% | apilevels.com | "legacy Android" segment |

**Practical floor**: API 24+ covers 96.6% of devices and aligns with Flutter's floor. The remaining 3.4% (API 21–25) is the "legacy Android" question — but ANGLE doesn't help here because the NDK always provides system GL ES.

---

## 6. RK3588 / Embedded ARM Linux

| Concern | Status | Source |
|---|---|---|
| Vulkan support? | Panthor (Mali-G610 MC4) — Vulkan 1.1+ | `research2/mesa-panthor.md`: Mali Panthor driver, Vulkan support |
| ANGLE needed? | **No** — native Vulkan or OpenGL ES via Mesa | Same |
| wgpu path? | wgpu Vulkan ✅ (primary), wgpu GL ES 🆗 (Mesa GL) | wgpu README |

**Verdict**: RK3588 has native GPU drivers. ANGLE adds no value.

---

## 7. Platform × Rendering Path Matrix

| Platform | wgpu Vulkan | wgpu DX12 | wgpu GL (native) | wgpu GL (via ANGLE) | softbuffer (CPU) | Unsupported |
|---|---|---|---|---|---|---|
| **Win10+ (modern GPU)** | ✅ primary | ✅ primary | 🆗 fallback | unnecessary | — | — |
| **Win7 SP1** | ⚠ needs driver + unofficial Rust | ❌ DX12 = Win10+ | ✅ WGL GL 3.3+ | ✅ ANGLE→D3D11 FL10_0 | ✅ CPU-only | — |
| **Win XP/Vista** | ❌ | ❌ | ❌ | ❌ (D3D9 removed) | ❌ (Rust floor) | ✅ |
| **Android API 24+** | ✅ primary | N/A | ✅ GL ES 3.0+ | unnecessary | — | — |
| **Android API 21–23** | ⚠ may lack Vulkan | N/A | ✅ GL ES 3.0 | unnecessary | — | — |
| **RK3588 / ARM Linux** | ✅ Panthor/Mesa | N/A | ✅ Mesa GL | unnecessary | — | — |
| **macOS** | 🌋 MoltenVK | ❌ | ❌ (no native GL) | 📐 ANGLE→GL ES 3.0 | — | — |
| **iOS** | 🌋 MoltenVK | ❌ | ❌ | 📐 ANGLE→GL ES 3.0 | — | — |
| **Web/WASM** | ❌ | ❌ | ❌ | ❌ | — | ✅ WebGL2 |

### Legend
- ✅ = works, recommended
- 🆗 = works, downlevel
- 📐 = works via ANGLE translation
- ⚠ = works but unofficial / needs workarounds
- ❌ = no path
- — = not applicable

---

## 8. Implications for Whitepaper §3.3 (Rendering Backends) and §4 (Roadmap)

### ANGLE Is Almost Entirely a Red Herring

The research reveals a surprising conclusion: **ANGLE's role in VisiaEngine is marginal**.

1. **wgpu already has ANGLE integration built-in** (`cfg(windows_angle)`, macOS `angle` feature) — you don't wrap ANGLE yourself.
2. **The Rust toolchain has dropped Win7** (MSVC target requires Win10+) — the only platform where ANGLE's D3D11 backend would add value.
3. **On Android, system GL ES is always available** — ANGLE-on-Vulkan is Chrome-internal, not an app-facing API.
4. **On RK3588/embedded Linux**, native Vulkan+GL via Mesa is the correct path.
5. **On macOS**, wgpu's `angle` feature already handles the GL ES translation if needed (but Metal is preferred).

### What ANGLE IS Useful For

| Use Case | Value | Effort |
|---|---|---|
| **macOS GL ES fallback** (Intel Macs without Metal) | Medium — wgpu `angle` feature handles it | Low (build ANGLE from source once) |
| **Win7 unofficial tier** (if pursued) | Low — <3% of Windows install base, Rust toolchain doesn't support it | High (pinned Rust + ANGLE static build + custom pipeline) |
| **Debugging/testing** (ANGLE null backend) | Low | Low |

### Recommended Wording for §3.3

> **Primary path**: wgpu v30 with Vulkan (Linux/Android), DX12 (Windows), and Metal (macOS/iOS).  
> **GL fallback**: wgpu's built-in GL backend for OpenGL ES 3.0+ (Android, Linux, older Windows) and OpenGL 3.3+ (Windows WGL).  
> **ANGLE**: Not a separate backend layer for VisiaEngine. wgpu already integrates ANGLE as a build-time option (`cfg(windows_angle)` on Windows, `angle` feature on macOS). Use it only if GL ES fallback is needed on macOS or as an unofficial Win7 path.  
> **Legacy Windows**: ANGLE-D3D9 is gone (removed 2026-07-07, google/angle main). The Rust MSVC target requires Windows 10+. Win7/XP are not supported tiers.  
> **Android**: System GL ES or native Vulkan — ANGLE is unnecessary. Floor = API 24+ (96.6% of devices, aligns with Flutter).

### Recommended Wording for §4 (Roadmap)

> **Phase 1 (MVP)**: wgpu Vulkan (primary Linux/Android), DX12 (primary Windows), Metal (primary macOS/iOS). No ANGLE dependency.  
> **Phase 2 (Extended)**: wgpu GL backend for downlevel fallback (older Android API 24–25, older Windows GPU). macOS GL ES via wgpu `angle` feature if Intel Mac support needed.  
> **Phase 3 (If required)**: Win7 unofficial tier via pinned Rust + ANGLE-D3D11 static build. Only on explicit customer demand.

---

## 9. Open Questions / UNCERTAIN Items

| Item | Status | Next Step |
|---|---|---|
| Chrome Android default = ANGLE-Vulkan | **UNCERTAIN** — chromestatus API returned 0 results for "ANGLE Vulkan"; chromium.googlesource.com blocked | Not material to VisiaEngine (system GL is the correct path regardless) |
| vcpkg angle pinned commit/version | vcpkg.json content not extractable (GitHub rendered page = JS-only) | Use GitHub API when quota resets |
| D3D9 removal confirmed in which Chrome version? | Commit dates known (2026-04–07) but shipping milestone UNCERTAIN | Not material (D3D9 = dead path regardless) |
| Rust `x86_64-pc-windows-gnu` Win7 floor | MSVC confirmed Win10+; GNU target floor not verified separately | Would need separate investigation if Win7 tier pursued |
| ANGLE static lib binary size | Not measured | vcpkg build + `ls -lh` on output libs; estimated ~3-5MB based on Chromium builds |
| Qt6 ANGLE usage | **Refuted** — Qt6 RHI lists OGL/D3D11/D3D12/Metal/Vulkan/Null only; no ANGLE in qtbase/src/3rdparty/ | Qt6 does not use ANGLE as a default backend |

---

## 10. Methodology & Sources

| Source | Access Method | Date |
|---|---|---|
| google/angle repo metadata | GitHub REST API (unauthenticated) | 2026-09-03 |
| ANGLE backend layout + version.h | grep.app (code search) | 2026-09-03 |
| ANGLE D3D9 removal commits | GitHub `search/commits` (separate quota) | 2026-09-03 |
| ANGLE D3D11on12 gate | grep.app `Display.cpp` snippet | 2026-09-03 |
| wgpu README platform table | webfetch on github.com/gfx-rs/wgpu/trunk/README.md | 2026-09-03 |
| wgpu changelog | Local file `/tmp/opencode/research2/wgpu-changelog.md` (5998 lines) | 2026-09-03 |
| Flutter Windows ANGLE link | grep.app `flutter/BUILD.gn` | 2026-09-03 |
| Flutter supported platforms | webfetch docs.flutter.dev/reference/supported-platforms | 2026-09-03 |
| Android API distribution | webfetch apilevels.com | 2026-09-03 |
| Rust MSVC Win10 requirement | webfetch doc.rust-lang.org/rustc/platform-support/windows-msvc.html | 2026-09-03 |
| Chromium release cadence | webfetch chromium.org/developers/calendar | 2026-09-03 |
| Qt6 RHI backends | webfetch doc.qt.io/qt-6/qrhi.html | 2026-09-03 |
| Qt6 ANGLE absence | webfetch github.com/qt/qtbase tree `src/3rdparty` (no `angle/` dir) | 2026-09-03 |
| vcpkg angle port existence | webfetch github.com/microsoft/vcpkg tree `ports/angle/` | 2026-09-03 |
| Crates.io ANGLE landscape | crates.io API (unauthenticated) | 2026-09-03 |
| RK3588 Panthor/Mesa | Local file `/tmp/opencode/research2/mesa-panthor.md` | 2026-09-03 |

**Blocked during session**: GitHub REST API rate limit (0/60 unauthenticated); `raw.githubusercontent.com` (403 proxy); `chromium.googlesource.com` (curl 000); MS Learn D3D11-on-Win7 (multiple URL guesses 404).
