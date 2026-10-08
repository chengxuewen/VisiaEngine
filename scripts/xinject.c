/* xinject.c · dev-only XTEST injector (band K, 2026-10-08).
 *
 * Turns "only a human can press this key" into a machine assertion: focus a window
 * by title substring, then fake a key / wheel / drag through the real X input path
 * (XTEST extension), so winit sees exactly what a user produces. Driven by
 * scripts/keys-probe.py; see that file for the display/fixture policy.
 *
 * usage: xinject <display> <title-substr> <action> [args...]
 *   key <keysym>        e.g. key 4 / key Escape / key Prior
 *   wheel <dir> [reps]  dir>0 = up (button 4), else down (button 5)
 *   drag  <dx> <dy>     press at +40,+40 in window coords, 8 motion steps, release
 *   noop                focus only (a later delta can only come from the app clock)
 *   name                print NAME=<window title> and exit (state echo assertions)
 *
 * Two hard-won notes from the session that validated this harness:
 *   1) WM_NAME *and* _NET_WM_NAME are both queried. winit sets the UTF-8 property
 *      (_NET_WM_NAME); reading only the ICCCM legacy WM_NAME finds nothing.
 *   2) XTestGetDisplayRange is NOT exported by this libXtst build (conda-forge
 *      xorg-libxtst). Do not call it -- it links, then dies at load time.
 */
#define _GNU_SOURCE 1
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/extensions/XTest.h>

static int name_matches(Display *d, Window w, const char *needle) {
    Atom net = XInternAtom(d, "_NET_WM_NAME", False);
    Atom wm = XInternAtom(d, "WM_NAME", False);
    XTextProperty tp;
    int hit = 0;
    for (int which = 0; which < 2 && !hit; ++which) {
        if (XGetTextProperty(d, w, &tp, which ? wm : net) && tp.value) {
            hit = strstr((char *)tp.value, needle) != NULL;
            XFree(tp.value);
        }
    }
    return hit;
}

static Window find(Display *d, Window w, const char *needle, int depth) {
    if (name_matches(d, w, needle)) return w;
    if (depth > 6) return 0;
    Window r, p, *kids = NULL;
    unsigned n = 0;
    if (XQueryTree(d, w, &r, &p, &kids, &n)) {
        for (unsigned i = 0; i < n; ++i) {
            Window f = find(d, kids[i], needle, depth + 1);
            if (f) {
                XFree(kids);
                return f;
            }
        }
        XFree(kids);
    }
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 4) {
        fprintf(stderr, "usage: xinject DISPLAY needle action...\n");
        return 2;
    }
    Display *d = XOpenDisplay(argv[1]);
    if (!d) {
        fprintf(stderr, "cannot open display %s\n", argv[1]);
        return 3;
    }
    Window w = find(d, DefaultRootWindow(d), argv[2], 0);
    if (!w) {
        fprintf(stderr, "no window matching '%s'\n", argv[2]);
        return 4;
    }
    XSetInputFocus(d, w, RevertToParent, CurrentTime);
    XSync(d, False);
    usleep(120000);

    const char *act = argv[3];
    if (!strcmp(act, "key")) {
        KeySym k = XStringToKeysym(argv[4]);
        KeyCode c = XKeysymToKeycode(d, k);
        if (!c) {
            fprintf(stderr, "no keycode for %s\n", argv[4]);
            return 5;
        }
        XTestFakeKeyEvent(d, c, True, CurrentTime);
        XTestFakeKeyEvent(d, c, False, CurrentTime);
    } else if (!strcmp(act, "wheel")) {
        int dir = atoi(argv[4]), reps = argc > 5 ? atoi(argv[5]) : 3;
        unsigned btn = dir > 0 ? 4 : 5;
        for (int i = 0; i < reps; ++i) {
            XTestFakeButtonEvent(d, btn, True, CurrentTime);
            XTestFakeButtonEvent(d, btn, False, CurrentTime);
        }
    } else if (!strcmp(act, "drag")) {
        int dx = atoi(argv[4]), dy = atoi(argv[5]);
        int x0, y0, wx, wy;
        Window ch;
        XTranslateCoordinates(d, w, DefaultRootWindow(d), 0, 0, &x0, &y0, &ch);
        XTestFakeMotionEvent(d, 0, x0 + 40, y0 + 40, CurrentTime);
        XTestFakeButtonEvent(d, 1, True, CurrentTime);
        for (int i = 1; i <= 8; ++i)
            XTestFakeMotionEvent(d, 0, x0 + 40 + dx * i / 8, y0 + 40 + dy * i / 8, CurrentTime);
        XTestFakeButtonEvent(d, 1, False, CurrentTime);
    } else if (!strcmp(act, "noop")) {
        /* focus only: any later pixel delta came from the app's own clock */
    } else if (!strcmp(act, "name")) {
        XTextProperty tp;
        Atom net = XInternAtom(d, "_NET_WM_NAME", False);
        char *s = NULL;
        if (XGetTextProperty(d, w, &tp, net) && tp.value) s = strdup((char *)tp.value);
        if (!s && XGetWMName(d, w, &tp) && tp.value) s = strdup((char *)tp.value);
        printf("NAME=%s\n", s ? s : "(none)");
        free(s);
        XCloseDisplay(d);
        return 0;
    } else {
        fprintf(stderr, "unknown action '%s'\n", act);
        return 6;
    }
    XFlush(d);
    XSync(d, False);
    printf("xinject ok win=0x%lx act=%s\n", (unsigned long)w, act);
    XCloseDisplay(d);
    return 0;
}
