/* E801_sdl_window —— SDK 消费·C+SDL3 真窗（8x 带；S0 spike 通路正式化）。
 * 双模式：无参=交互窗（渲染首帧后 SDL_WaitEvent 至关窗，resize 联动 viewport——人检面）；
 * `--frames N`=画 N 帧即退（CI/xvfb 子态契约）。
 * 合同：SDL_VIDEO_DRIVER=x11 锁（attach v0=X11-only）；DISPLAY 缺=exit 77（ctest SKIP 约定，
 * 原生自查无 stub；IDE 无 DISPLAY 走 run_E801 的 run-gui :0 回退）。
 * 跑不通即 API 未完成 [E3D:D7]。 */
#include <SDL3/SDL.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "visiaengine.h"

int main(int argc, char **argv) {
    int frames = 0; /* 0 = 交互 */
    for (int i = 1; i + 1 < argc; i++) {
        if (strcmp(argv[i], "--frames") == 0) frames = atoi(argv[++i]);
    }
    if (getenv("DISPLAY") == NULL || getenv("DISPLAY")[0] == '\0') {
        fputs("E801: 无 DISPLAY（IDE 走 run_E801 步骤的 run-gui 回退；CI 走 xvfb；ctest 记 Skipped）\n",
              stderr);
        return 77;
    }
    setenv("SDL_VIDEO_DRIVER", "x11", 1); /* attach 合同 v0=X11-only，wayland 默认机免疫 */

    if (!SDL_Init(SDL_INIT_VIDEO)) { printf("E801 FAIL sdl init: %s\n", SDL_GetError()); return 1; }
    SDL_Window *win = SDL_CreateWindow("VisiaEngine E801 · SDL3 真窗（关窗退出）", 320, 240, 0);
    if (!win) { printf("E801 FAIL window: %s\n", SDL_GetError()); return 1; }

    SDL_PropertiesID props = SDL_GetWindowProperties(win);
    uint64_t xid = (uint64_t)SDL_GetNumberProperty(props, SDL_PROP_WINDOW_X11_WINDOW_NUMBER, 0);
    void *dpy    = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_X11_DISPLAY_POINTER, NULL);
    if (!xid || !dpy) { puts("E801 FAIL no x11 props（确认 SDL_VIDEO_DRIVER=x11）"); return 1; }

    VeEngine ve = visiaengine_create_headless(320, 240);
    if (!ve) { puts("E801 FAIL create"); return 1; }
    if (visiaengine_load_gltf(ve, "resources/data/twoprim.glb") != VE_OK) {
        printf("E801 FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    if (visiaengine_attach(ve, xid, (uint64_t)(uintptr_t)dpy, 0) != VE_OK) {
        printf("E801 FAIL attach: %s\n", visiaengine_last_error(ve));
        return 1;
    }

    if (frames > 0) {
        for (int i = 0; i < frames; ++i) {
            if (visiaengine_render(ve) != VE_OK) {
                printf("E801 FAIL render %d: %s\n", i, visiaengine_last_error(ve));
                return 1;
            }
            SDL_PumpEvents();
            SDL_Delay(60);
        }
    } else {
        if (visiaengine_render(ve) != VE_OK) {
            printf("E801 FAIL render: %s\n", visiaengine_last_error(ve));
            return 1;
        }
        SDL_Event e;
        for (;;) {
            if (!SDL_WaitEvent(&e)) break; /* 事件驱动零空转；X 断连=退出 */
            if (e.type == SDL_EVENT_QUIT) break;
            if (e.type == SDL_EVENT_WINDOW_RESIZED) {
                visiaengine_viewport(ve, (uint32_t)e.window.data1, (uint32_t)e.window.data2);
            }
            if (visiaengine_render(ve) != VE_OK) {
                printf("E801 FAIL render(ev): %s\n", visiaengine_last_error(ve));
                return 1;
            }
        }
    }
    if (visiaengine_destroy(ve) != VE_OK) { puts("E801 FAIL destroy"); return 1; }
    SDL_Quit();
    printf("OK E801_sdl_window mode=%s\n", frames > 0 ? "frames" : "interactive");
    return 0;
}
