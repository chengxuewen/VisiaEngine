/* E801_sdl_window -- SDK consumption, C+SDL3 real window (the 8x band; formalized from the S0 spike road).
 *  Dual mode: no args = interactive window (after the first frame SDL_WaitEvent until close, resize drives viewport -- human check face);
 *  `--frames N` = draw N frames then exit (the CI/xvfb sub-state contract).
 *  Contract: the SDL_VIDEO_DRIVER=x11 lock (attach v0 = X11-only); missing DISPLAY = exit 77 (the ctest SKIP convention,
 *  the native self-check has no stub; an IDE without DISPLAY takes visiaengine-example-E801_sdl_window-run's run-gui :0 fallback).
 *  If it does not run, the API is unfinished [E3D:D7]. */
#include <SDL3/SDL.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "visiaengine.h"

int main(int argc, char **argv) {
    int frames = 0; /* 0 = interactive */
    for (int i = 1; i + 1 < argc; i++) {
        if (strcmp(argv[i], "--frames") == 0) frames = atoi(argv[++i]);
    }
    if (getenv("DISPLAY") == NULL || getenv("DISPLAY")[0] == '\0') {
        fputs("E801: no DISPLAY (the IDE takes the visiaengine-example-E801_sdl_window-run step's run-gui fallback; CI takes xvfb; ctest records Skipped)\n",
              stderr);
        return 77;
    }
    setenv("SDL_VIDEO_DRIVER", "x11", 1); /* the attach contract v0 = X11-only, immune to wayland-default machines */

    if (!SDL_Init(SDL_INIT_VIDEO)) { printf("E801 FAIL sdl init: %s\n", SDL_GetError()); return 1; }
    SDL_Window *win = SDL_CreateWindow("VisiaEngine E801 · SDL3 real window (close to exit)", 320, 240, 0);
    if (!win) { printf("E801 FAIL window: %s\n", SDL_GetError()); return 1; }

    SDL_PropertiesID props = SDL_GetWindowProperties(win);
    uint64_t xid = (uint64_t)SDL_GetNumberProperty(props, SDL_PROP_WINDOW_X11_WINDOW_NUMBER, 0);
    void *dpy    = SDL_GetPointerProperty(props, SDL_PROP_WINDOW_X11_DISPLAY_POINTER, NULL);
    if (!xid || !dpy) { puts("E801 FAIL no x11 props (confirm SDL_VIDEO_DRIVER=x11)"); return 1; }

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
            if (!SDL_WaitEvent(&e)) break; /* event-driven zero spin; X disconnect = exit */
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
