# 2026-09-16 SDL3×capi attach spike evidence (snapshot day, never updated)

> S0 exit artifact (.omo/plans/bindings-restructure-cpp-examples.md v1.3 §S0). The throwaway code lives in /tmp (not in the repo);
> this file records environment facts and the path conclusions, for the S4 window example (E801) and the packaging round to cite.

## Conclusion (both states fully green)

| state | command essentials | result |
|---|---|---|
| real window @:0 | `DISPLAY=:0 SDL_VIDEO_DRIVER=x11 <spike>` | `OK sdl3 attach` rc=0 (passed on the first try) |
| Xvfb :77 | default-environment Xvfb + `SDL_VIDEO_DRIVER=x11` | same, rc=0 (**the xkb tree must be fixed first, see pit 2**) |

Path chain: `SDL_CreateWindow → SDL_GetWindowProperties → SDL_GetNumberProperty(SDL_PROP_WINDOW_X11_WINDOW_NUMBER)`
`→ xid + SDL_GetPointerProperty(SDL_PROP_WINDOW_X11_DISPLAY_POINTER) → Display*`
→ **the existing `visiaengine_attach(ve, xid, (uintptr_t)dpy, kind=0)` two-slot contract is directly usable (CAPI-06, isomorphic with the E702 self-held Xlib form)**,
the protocol sequence is item-for-item the same as E702: create_headless(320,240)→load_gltf(twoprim.glb)→attach→render×3 (SDL_PumpEvents+Delay80 between frames)→destroy.
Rendering runs on lavapipe (the WARNING lines are the normal-state note of wgpu software Vulkan, consistent with the golden family).

## Environment facts

1. **sdl3 3.4.16** (`sdl3-3.4.16-h5330f5c_0.conda`, conda-forge) run-deps are a full C stack:
   libstdcxx/libgcc, xorg-libx{11,xext,xfixes,xi,xrandr,xrender,xscrnsaver,xtst,xcursor,xau,xdmcp}, 
   wayland, libxkbcommon, xkeyboard-config, libgl/libegl, dbus, libvulkan-loader, libdrm, libusb, liburing, libunwind.
   **Does not pull the rust toolchain** (G-12 number: the package body is pure, zero entanglement with the cargo environment). zlib/libpng licenses.
2. **Pit (PIT-19, filed): pixi file clobber makes the conda Xvfb always blow up XKB** — `xorg-xvfb-server` ships
   `share/X11/xkb/compiled/.keep` conflicting with the `xkeyboard-config` tree -> pixi 0.78 **redirects the latter's whole tree** to
   `share/xkeyboard-config-2/`, leaving the canonical `share/X11/xkb/` empty -> `Xvfb` at startup immediately
   `XKB: Failed to compile keymap` / `Failed to activate virtual core keyboard`.
   Local fix (the environment is local state, replay after rebuild): `cp -a .pixi/envs/default/share/xkeyboard-config-2/. .pixi/envs/default/share/X11/xkb/`.
   Same for the host-spike environment (historically smoke-x11 never walked the Xvfb branch = this machine always has :0, so it stayed hidden).
   CI surface: the apt system Xvfb is immune; the B2 sub-state account is revised accordingly = **prefer `command -v Xvfb` (system), the conda fallback must ship the tree-fix command with it, neither available = SKIP note, no false red**.
3. `SDL_VIDEO_DRIVER=x11` must be set explicitly (the wayland default backend does not provide the SDL_PROP_WINDOW_X11_* property surface; the attach contract is v0=X11-only, risk ③ holds as expected).
4. Dependency registered: `pixi.toml` [feature.tools.dependencies] `sdl3 >=3.4,<4` (all platforms);
   [feature.tools.target.linux-64.dependencies] `xorg-xvfb-server >=21.1.24,<22` (display sub-state local fallback); pixi.lock follows.

## Verdict

S0 Go — the S4 window example goes with SDL3 as planned, **no retreat to a Qt fallback needed**. E801's header note at birth must carry
the `SDL_VIDEO_DRIVER=x11` lock + a DISPLAY-absent = 77 pre-check (native self-inspection, no stub).
