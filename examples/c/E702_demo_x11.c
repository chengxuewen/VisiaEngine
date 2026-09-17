/* E702 · 绑定镜像·C X11 —— attach 真窗口双模例（I3/smoke-x11；宿主集成骨架；
 * 跑不通即 API 未完成 [E3D:D7]）。
 * 双模（E801/E703 同制，argv 单源 C15）：
 *   无参                = 常驻人验窗（Expose 重绘，关窗/WM_DELETE 退出）
 *   --frames N [path]   = 自动快退（ctest 注册形/smoke-x11）；path 缺省=twoprim.glb */
#include <X11/Xatom.h>
#include <X11/Xlib.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "visiaengine.h"

static void xsync(Display *d) { XSync(d, False); }

static int render_or_fail(uint64_t ve, Display *d, int i) {
    if (visiaengine_render(ve) != VE_OK) {
        printf("FAIL render %d: %s\n", i, visiaengine_last_error(ve));
        return 1;
    }
    xsync(d);
    return 0;
}

int main(int argc, char **argv) {
    int frames = 0; /* 0=常驻人验形（约定：run 步零参=一直显示） */
    const char *path = "resources/data/twoprim.glb";
    for (int i = 1; i < argc; ++i) {
        if (strcmp(argv[i], "--frames") == 0 && i + 1 < argc) {
            frames = atoi(argv[++i]);
        } else {
            path = argv[i];
        }
    }
    Display *dpy = XOpenDisplay(NULL);
    if (!dpy) { puts("FAIL no display"); return 1; }
    int scr = DefaultScreen(dpy);
    Window win = XCreateSimpleWindow(dpy, RootWindow(dpy, scr), 10, 10, 320, 240, 1,
                                     BlackPixel(dpy, scr), WhitePixel(dpy, scr));
    XStoreName(dpy, win, "visiaengine attach demo (E702)");
    XSelectInput(dpy, win, ExposureMask);
    XMapWindow(dpy, win);
    xsync(dpy);

    uint64_t ve = visiaengine_create_headless(320, 240);
    if (!ve) { puts("FAIL create"); return 1; }
    if (visiaengine_load_gltf(ve, path) != VE_OK) {
        printf("FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    /* attach：xid + Display* 双槽（x11 display 必填，CAPI-06） */
    if (visiaengine_attach(ve, (uint64_t)win, (uint64_t)dpy, 0) != VE_OK) {
        printf("FAIL attach: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    if (frames > 0) {
        for (int i = 0; i < frames; ++i) {
            if (render_or_fail(ve, dpy, i)) return 1;
            usleep(80000); /* 80ms：present 与 X 服务端 flush 交叠 */
        }
        if (visiaengine_destroy(ve) != VE_OK) { puts("FAIL destroy"); return 1; }
        XCloseDisplay(dpy);
        puts("OK capi x11");
        return 0;
    }
    /* 常驻形：WM_DELETE 握手 + Expose 重绘（attach 骨架例保持最小，不做 resize 联动——E801 面） */
    Atom wm = XInternAtom(dpy, "WM_DELETE_WINDOW", False);
    XSetWMProtocols(dpy, win, &wm, 1);
    puts("E702 常驻：关窗退出（自动形=--frames N）");
    for (XEvent ev;;) {
        XNextEvent(dpy, &ev);
        if (ev.type == Expose && ev.xexpose.count == 0) {
            if (render_or_fail(ve, dpy, -1)) return 1;
        } else if (ev.type == ClientMessage &&
                   (Atom)ev.xclient.data.l[0] == wm) {
            break;
        }
    }
    if (visiaengine_destroy(ve) != VE_OK) { puts("FAIL destroy"); return 1; }
    XCloseDisplay(dpy);
    puts("OK capi x11 (interactive)");
    return 0;
}
