/* demo_x11.c —— attach 真窗口 demo（I3/smoke-x11；文档件：宿主集成骨架）。 */
#include <X11/Xlib.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "visiaengine.h"

static void xsync(Display *d) { XSync(d, False); }

int main(int argc, char **argv) {
    Display *dpy = XOpenDisplay(NULL);
    if (!dpy) { puts("FAIL no display"); return 1; }
    int scr = DefaultScreen(dpy);
    Window win = XCreateSimpleWindow(dpy, RootWindow(dpy, scr), 10, 10, 320, 240, 1,
                                     BlackPixel(dpy, scr), WhitePixel(dpy, scr));
    XStoreName(dpy, win, "visiaengine attach demo");
    XMapWindow(dpy, win);
    xsync(dpy);

    uint64_t ve = visiaengine_create_headless(320, 240);
    if (!ve) { puts("FAIL create"); return 1; }
    if (visiaengine_load_gltf(ve, argc > 1 ? argv[1] : "resources/data/twoprim.glb") != VE_OK) {
        printf("FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    /* attach：xid + Display* 双槽（x11 display 必填，CAPI-06） */
    if (visiaengine_attach(ve, (uint64_t)win, (uint64_t)dpy, 0) != VE_OK) {
        printf("FAIL attach: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    for (int i = 0; i < 3; ++i) {
        if (visiaengine_render(ve) != VE_OK) {
            printf("FAIL render %d: %s\n", i, visiaengine_last_error(ve));
            return 1;
        }
        xsync(dpy);
        usleep(80000); /* 80ms：present 与 X 服务端 flush 交叠 */
    }
    if (visiaengine_destroy(ve) != VE_OK) { puts("FAIL destroy"); return 1; }
    XCloseDisplay(dpy);
    puts("OK capi x11");
    return 0;
}
