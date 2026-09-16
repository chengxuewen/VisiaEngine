# 2026-09-16 SDL3×capi attach spike 证据（快照日，永不更新）

> S0 出口件（.omo/plans/bindings-restructure-cpp-examples.md v1.3 §S0）。throwaway 代码在 /tmp（不进仓）；
> 本档记录环境事实与通路结论，供 S4 窗口例（E801）与打包轮引用。

## 结论（两态全绿）

| 态 | 命令要点 | 结果 |
|---|---|---|
| 真窗 @:0 | `DISPLAY=:0 SDL_VIDEO_DRIVER=x11 <spike>` | `OK sdl3 attach` rc=0（首试即过）|
| Xvfb :77 | default 环境 Xvfb + `SDL_VIDEO_DRIVER=x11` | 同上 rc=0（**须先修 xkb 树，见坑 2**）|

通路链：`SDL_CreateWindow → SDL_GetWindowProperties → SDL_GetNumberProperty(SDL_PROP_WINDOW_X11_WINDOW_NUMBER)
→ xid + SDL_GetPointerProperty(SDL_PROP_WINDOW_X11_DISPLAY_POINTER) → Display*`
→ **现 `visiaengine_attach(ve, xid, (uintptr_t)dpy, kind=0)` 双槽合同直接可用（CAPI-06，与 E702 Xlib 自持形同构）**，
协议序列逐件同 E702：create_headless(320,240)→load_gltf(twoprim.glb)→attach→render×3（帧间 SDL_PumpEvents+Delay80）→destroy。
渲染走 lavapipe（WARNING 行系 wgpu 软件 Vulkan 常态注记，与 golden 族一致）。

## 环境事实

1. **sdl3 3.4.16**（`sdl3-3.4.16-h5330f5c_0.conda`，conda-forge）run-deps 全 C 栈：
   libstdcxx/libgcc、xorg-libx{11,xext,xfixes,xi,xrandr,xrender,xscrnsaver,xtst,xcursor,xau,xdmcp}、
   wayland、libxkbcommon、xkeyboard-config、libgl/libegl、dbus、libvulkan-loader、libdrm、libusb、liburing、libunwind。
   **不拉 rust 工具链**（G-12 出数：包体纯净，与 cargo 环境零纠缠）。zlib/libpng 许可。
2. **坑（PIT-19，已入档）：pixi 文件 clobber 使 conda 版 Xvfb 恒炸 XKB**——`xorg-xvfb-server` 携
   `share/X11/xkb/compiled/.keep` 与 `xkeyboard-config` 树冲突 → pixi 0.78 将后者**整树重定向**
   `share/xkeyboard-config-2/`，正位 `share/X11/xkb/` 空 → `Xvfb` 起服即
   `XKB: Failed to compile keymap` / `Failed to activate virtual core keyboard`。
   本机解法（环境为本机态，重建后重放）：`cp -a .pixi/envs/default/share/xkeyboard-config-2/. .pixi/envs/default/share/X11/xkb/`。
   host-spike 环境同款（历史上 smoke-x11 从未走过 Xvfb 分支=本机恒有 :0，故未暴露）。
   CI 面：apt 系统 Xvfb 免疫；B2 子态口径据此修订=**优先 `command -v Xvfb`（系统），conda 兜底需随附修树命令，皆不可得=SKIP note 不误红**。
3. `SDL_VIDEO_DRIVER=x11` 必须显式设（wayland 默认后端不提供 SDL_PROP_WINDOW_X11_* 属性面；attach 合同 v0=X11-only，风险③按预期成立）。
4. 依赖入册：`pixi.toml` [feature.tools.dependencies] `sdl3 >=3.4,<4`（全平台）；
   [feature.tools.target.linux-64.dependencies] `xorg-xvfb-server >=21.1.24,<22`（display 子态本地兜底）；pixi.lock 已随。

## 判定

S0 Go——S4 窗口例按原计划走 SDL3，**无需退 Qt fallback**。E801 出生时头注须含
`SDL_VIDEO_DRIVER=x11` 锁 + DISPLAY 缺=77 预检（原生自查，无 stub）。
