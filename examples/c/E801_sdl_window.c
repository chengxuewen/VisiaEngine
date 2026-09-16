/* E801_sdl_window —— SDK 消费·C+SDL3 窗口例（8x 带首婴；S0 spike 通路正式化）。
 * 合同：SDL_VIDEO_DRIVER=x11 锁（attach v0=X11-only，S0 evidence）；DISPLAY 缺=exit 77
 * （ctest SKIP 约定，原生自查无 stub）；协议=create→load→attach(xid,display,0)→render×3→destroy。
 * 跑不通即 API 未完成 [E3D:D7]。 */
#include <SDL3/SDL.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "visiaengine.h"

int main(void) {
    if (getenv("DISPLAY") == NULL || getenv("DISPLAY")[0] == '\0') {
        fputs("E801: DISPLAY 未设置（桌面直跑/CI 走 xvfb；ctest 记 Skipped）\n", stderr);
        return 77;
    }
    setenv("SDL_VIDEO_DRIVER", "x11", 1);  /* attach 合同=v0 X11-only，wayland 默认机免疫 */

    if (!SDL_Init(SDL_INIT_VIDEO)) { printf("E801 FAIL sdl init: %s\n", SDL_GetError()); return 1; }
    SDL_Window *win = SDL_CreateWindow("E801_sdl_window", 320, 240, 0);
    if (!win) { printf("E801 FAIL window: %s\n", SDL_GetError()); return 1; }

    SDL_PropertiesID props = SDL_GetWindowProperties(win);
    uint64_t xid = (uint64_t)SDL_GetNumberProperty(props, SDL_PROP_WINDOW_X11_WINDOW_NUMBER, 0);
    void *dpy    = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_X11_DISPLAY_POINTER, NULL);
    if (!xid || !dpy) { puts("E801 FAIL no x11 props"); return 1; }

    VeEngine ve = visiaengine_create_headless(320, 240);
    if (!ve) { puts("E801 FAIL create"); return 1; }
    if (visiaengine_load_gltf(ve, "resources/data/twoprim.glb") != VE_OK) {
        printf("E801 FAIL load: %s\n", visiaengine_last_error(ve)); return 1;
    }
    if (visiaengine_attach(ve, xid, (uint64_t)(uintptr_t)dpy, 0) != VE_OK) {
        printf("E801 FAIL attach: %s\n", visiaengine_last_error(ve)); return 1;
    }
    for (int i = 0; i < 3; ++i) {
        if (visiaengine_render(ve) != VE_OK) {
            printf("E801 FAIL render %d: %s\n", i, visiaengine_last_error(ve)); return 1;
        }
        SDL_PumpEvents();
        SDL_Delay(80);
    }
    if (visiaengine_destroy(ve) != VE_OK) { puts("E801 FAIL destroy"); return 1; }
    SDL_Quit();
    puts("OK E801_sdl_window x11present=3");
    return 0;
}
