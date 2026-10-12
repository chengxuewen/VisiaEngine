/* E815_fly_camera -- SDK consumption, C camera flight (the CAPI-23/24 living gate, 5a).
 *  Headless evidence identity (same system as E813/E814): the wall-clock thruster driven through render() -- progress monotonic / arrival landing /
 *  input cancel / teleport form / value-domain rejection, the terminal OK line exits in seconds. */
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

#include "visiaengine.h"

static int fail(const char *what) {
    printf("E815 FAIL %s\n", what);
    return 1;
}

static void nap_ms(long ms) {
    struct timespec ts = {ms / 1000, (ms % 1000) * 1000000L};
    nanosleep(&ts, NULL);
}

int main(void) {
    uint64_t ve = visiaengine_create_headless(160, 120);
    if (!ve) return fail("create");
    VeCameraPose pose = {0};
    pose.struct_size = sizeof pose;
    pose.target[0] = 6.0; pose.target[1] = 2.0;
    pose.yaw = 0.9; pose.pitch = 0.45; pose.dist = 34.0; pose.zoom = 18.0; pose.fov = 1.05;
    /* value-domain rejection: NULL pose / struct_size gate / dist domain */
    double t01 = -7.0;
    if (visiaengine_fly_to(ve, NULL, 100) != VE_ERR_ARG) return fail("null pose not rejected");
    if (visiaengine_fly_state(ve, &t01) != 1) return fail("idle must be done");
    if (t01 != -7.0) return fail("the done-path zero write to out was broken");
    VeCameraPose tiny = pose;
    tiny.struct_size = 8;
    if (visiaengine_fly_to(ve, &tiny, 100) != VE_ERR_ARG) return fail("struct_size gate");
    VeCameraPose badd = pose;
    badd.dist = -2.0;
    if (visiaengine_fly_to(ve, &badd, 100) != VE_ERR_ARG) return fail("dist domain");
    /* teleport form: dur = 0 lands immediately (the pose cannot be read -- side-proved via fly_state always 1 on the done pose of a re-fly + render passes) */
    if (visiaengine_fly_to(ve, &pose, 0) != VE_OK) return fail("teleport");
    if (visiaengine_fly_state(ve, NULL) != 1) return fail("not done after teleport");
    if (visiaengine_render(ve) != VE_OK) return fail("render teleported");
    /* real flight 300ms: takeoff -> monotonic progress sampling -> arrival done */
    if (visiaengine_fly_to(ve, &pose, 300) != VE_OK) return fail("takeoff");
    double prev = -1.0;
    int samples = 0;
    long deadline = 1200;
    for (long waited = 0; waited < deadline; waited += 20) {
        if (visiaengine_render(ve) != VE_OK) return fail("render flying");
        double t = -9.0;
        int32_t st = visiaengine_fly_state(ve, &t);
        if (st == 0) {
            if (t < prev) return fail("progress ran backwards");
            prev = t;
            samples++;
            nap_ms(20);
        } else if (st == 1) {
            break;
        } else {
            return fail("fly_state out-of-code");
        }
    }
    if (samples < 2) return fail("sampling insufficient (the clock did not run)");
    if (visiaengine_fly_state(ve, NULL) != 1) return fail("did not arrive within the time limit");
    /* input cancel: pointer-down mid long flight -> done and re-flyable */
    VeCameraPose back = pose;
    back.target[0] = -6.0;
    if (visiaengine_fly_to(ve, &back, 4000) != VE_OK) return fail("refly");
    VeInput in = {0};
    in.struct_size = sizeof in;
    in.kind = 2; /* PTR_DOWN */
    if (visiaengine_on_input(ve, &in) != 1) return fail("input not consumed");
    if (visiaengine_fly_state(ve, NULL) != 1) return fail("cancel took no effect");
    if (visiaengine_fly_to(ve, &back, 0) != VE_OK) return fail("cannot re-fly after cancel");
    if (visiaengine_destroy(ve) != VE_OK) return fail("destroy");
    printf("OK e815 fly samples=%d progress_max=%.2f gate=domain-reject+cancel\n", samples, prev);
    return 0;
}
