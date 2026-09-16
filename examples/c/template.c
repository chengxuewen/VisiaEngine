/* template.c —— 窗口例子·C 毛坯骨架（B8 裁决：骨架不是库——抄文件不继承，
 * 保 gate-abi 对例子的全 API 眼；新例出生 cp 本件改「主题区」，「骨架区」勿动）。
 * 复制后首行改写为 /* E8xx_name —— 标题 ... */ 保住三方锁（文件名=头注=索引）。 */
#include <SDL3/SDL.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "visiaengine.h"

/* ── 骨架区：开窗→xid→attach→present 循环→收尾（合同=SDL_VIDEO_DRIVER=x11 + 无 DISPLAY=77）── */
int main(void) {
    if (getenv("DISPLAY") == NULL || getenv("DISPLAY")[0] == '\0') {
        fputs("no DISPLAY（ctest Skipped 约定）\n", stderr);
        return 77;
    }
    setenv("SDL_VIDEO_DRIVER", "x11", 1);
    if (!SDL_Init(SDL_INIT_VIDEO)) { printf("sdl init: %s\n", SDL_GetError()); return 1; }
    SDL_Window *win = SDL_CreateWindow("E8xx_template", 320, 240, 0);
    if (!win) return 1;
    SDL_PropertiesID props = SDL_GetWindowProperties(win);
    uint64_t xid = (uint64_t)SDL_GetNumberProperty(props, SDL_PROP_WINDOW_X11_WINDOW_NUMBER, 0);
    void *dpy    = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_X11_DISPLAY_POINTER, NULL);
    if (!xid || !dpy) return 1;

    VeEngine ve = visiaengine_create_headless(320, 240);
    if (!ve) return 1;
    /* ── 主题区：装载数据 + attach + 帧循环（render 前完成输入注入）── */
    if (visiaengine_attach(ve, xid, (uint64_t)(uintptr_t)dpy, 0) != VE_OK) {
        printf("attach: %s\n", visiaengine_last_error(ve)); return 1;
    }
    for (int i = 0; i < 3; ++i) {
        if (visiaengine_render(ve) != VE_OK) { printf("render: %s\n", visiaengine_last_error(ve)); return 1; }
        SDL_PumpEvents();
        SDL_Delay(80);
    }
    /* ── 主题区 end ── */
    visiaengine_destroy(ve);
    SDL_Quit();
    puts("OK E8xx_template");
    return 0;
}
