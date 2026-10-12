# Legacy and embedded-platform support matrix — the wgpu/ANGLE/toolchain three-way cross

**Snapshot 2026-09-03**. Evidence chain: the wgpu v30 README (locally fetched copy) + the rustc book platform pages (fetched directly the same day) + `evidence/2026-09-03-angle-integration` (ANGLE backends/removal history, grep.app code evidence) + the Wikipedia Mali architecture table (same day). Where this file overlaps the angle memo it does not re-expand, only giving verdicts.

## Core corrections (versus the intuitive assumptions before the research)
1. **wgpu has native GL/GLES backends** (the README's first sentence + the platform table): Windows 🆗 GL 3.3+ (default WGL, `cfg(windows_angle)` switches to ANGLE); Linux/Android 🆗 **GLES 3.0+**; Web 🆗 WebGL2. "wgpu has no GL path" ❌.
2. **ANGLE is already a build option of wgpu**, not a second backend requiring independent integration. The "cargo add angle" kind of need does not exist (the maintained `angle` crate is dead, but the upstream wgpu route is open).
3. **ANGLE's D3D9 backend has been removed from main** (the 2026-04~07 commit series, angle memo §2) -> **XP via ANGLE is unsolvable**; and **the Rust MSVC targets officially require Win10+** (rustc book original text, verified the same day) -> XP/Win7 break at the toolchain layer already.
4. **Qt6 does not use ANGLE** (refuted in its research: Qt6 RHI = native D3D11/D3D12/Vulkan/Metal/GL; qtbase 3rdparty has no angle). Flutter Windows = a real ANGLE user (BUILD.gn link evidence).
5. **RK3588 GPU = Mali-G610 = Valhall 3rd gen** (Wikipedia variant table) -> the open-source Vulkan path is **Panvk**, not Panthor (the latter starts at G615/Immortalis).

## Platform × path matrix (verdict edition)

| platform | feasible render path | verdict |
|------|-------------|------|
| Win10/11 | wgpu DX12 or Vulkan (first-class); GL backstop | ✅ the T1 main battlefield |
| Win7 SP1 | technically wgpu-GL(ANGLE-D3D11, FL10_0) is reachable; **the official Rust MSVC line has broken** (requires pinning an old rustc/GNU target + a self-built pipeline) | ⚠ **unofficial best-effort tier**, no commitment unless a customer explicitly pays for it |
| XP/Vista | Rust has dropped target support since 1.27; ANGLE D3D9 deleted; Chromium stopped 2016 | ❌ **impossible**, removed from any commitment text |
| Android API 24+ | system GLES (CDD-mandated) or Vulkan (new SoCs) -> the wgpu GL/Vulkan dual route, ANGLE not needed | ✅ the mobile main battlefield (API24 covers ~96.6%, aligned with Flutter's floor) |
| Android API 21–25 | GLES 3.0 subset + wgpu downlevel (GL backend) | 🆗 runs, feature-subset degradation; the 3.4% share per product decision |
| RK3588 (G610/Valhall) | **two**: ① the Rockchip/ARM closed-source libmali blob (Android GKI/vendor BSP 5.10, Vulkan 1.2+, but the blob is tightly coupled to kernel/distro) ② mainline **Panvk** (Mesa 24.x+ Valhall support, maturity rapidly evolving in 2025-26, *version details UNCERTAIN*) | ✅ viable, but **shipped binaries lock one driver stack** (pick ① follow the vendor, pick ② follow the distro), halve the QA matrix or costs double |
| RK3566/8-class G52 (Bifrost) | libmali or Panvk (Bifrost support matured earlier) / panfrost GLES | ✅ same strategy |
| Web (host browser) | wgpu->WebGPU (Chrome/Edge 113+, Safari 26, Firefox 141, full spectrum); the webgl feature backstops old GLSL browsers | ✅ the Web wing holds; **but Chrome 119+ is unavailable on Win7** -> the Web route does not rescue legacy Windows |

## Implications for the final backend ruling and the whitepaper
1. Whitepaper §3.3 can be honestly upgraded to: **"Vulkan/Metal/DX12 first-class + GLES 3.0+/GL 3.3+ degradation tier + WebGL2 backstop"** — stronger than the original "adapts Vulkan, Metal, DX12 and WebGPU" and fully backed by upstream documentation.
2. Add a qualifier to "cross-platform consistency": **what is consistent is the API surface and the feature-tier semantics, not identical visual quality/framerate** (downlevel = a feature subset; rerun's WebGL compatibility tier is the isomorphic precedent).
3. XP disappears from all texts; Win7 = "best-effort without an official commitment" (if sales lands a big Win7 order, that is a business question of enterprise plugins + custom builds, not an architecture question).
4. The Android floor is set at **API 24** (aligned with the Flutter ecosystem, covering 96.6%); old devices rely on the wgpu GL downlevel, not ANGLE.
5. RK3588's real cost lies in **driver-stack choice and the QA matrix** (blob vs mainline distro — pick one after fixing the customer persona first), not in API feasibility.

## Sources
local `wgpu_README.md` (trunk, fetched 2026-09-03); the doc.rust-lang.org windows-msvc page (same day); `evidence/2026-09-03-angle-integration` §2-§9 (incl. grep.app/commit evidence); en.wikipedia.org/wiki/Mali_(processor) (same day, rev 1369060869); the docs.rs wgpu platforms/angle route existence (README internal links). *UNCERTAIN: the exact Mesa version line for Panvk Valhall (gitlab anti-scraping blocked first-hand data); the concrete Vulkan version matrix of the libmali blob.*
