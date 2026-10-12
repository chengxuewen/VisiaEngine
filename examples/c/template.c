/* template.c -- window example, C raw blank skeleton (B8 ruling: the skeleton is not a library -- copying a file inherits nothing,
 *  keeping gate-abi's full-API eye on examples; when a new example is born, cp this file and edit the "theme zone", never touch the "skeleton zone").
 *  after copying rewrite the first line as /* E8xx_name -- title ... */ to keep the triple lock (file name = header note = index). */
#include <SDL3/SDL.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "visiaengine.h"

/* -- skeleton zone: open window -> xid -> attach -> present loop -> teardown (contract = SDL_VIDEO_DRIVER=x11 + no DISPLAY = 77) -- */
int main(void) {
    if (getenv("DISPLAY") == NULL || getenv("DISPLAY")[0] == '\0') {
        fputs("no DISPLAY (ctest Skipped convention)\n", stderr);
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
    /* -- theme zone: load data + attach + frame loop (finish input injection before render) -- */
    if (visiaengine_attach(ve, xid, (uint64_t)(uintptr_t)dpy, 0) != VE_OK) {
        printf("attach: %s\n", visiaengine_last_error(ve)); return 1;
    }
    for (int i = 0; i < 3; ++i) {
        if (visiaengine_render(ve) != VE_OK) { printf("render: %s\n", visiaengine_last_error(ve)); return 1; }
        SDL_PumpEvents();
        SDL_Delay(80);
    }
    /* -- theme zone end -- */
    visiaengine_destroy(ve);
    SDL_Quit();
    puts("OK E8xx_template");
    return 0;
}
