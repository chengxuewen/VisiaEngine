/* E815_fly_camera · SDK 消费·C 相机飞行（CAPI-23/24 活体门，⑤a）。
 * 无头证据身份（E813/E814 同制）：墙钟推进器经 render() 驱动——进度单调/到达落位/
 * 输入 cancel/瞬移形/值域拒，终端 OK 行秒退。 */
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
    /* 值域拒：NULL pose / struct_size 门 / dist 域 */
    double t01 = -7.0;
    if (visiaengine_fly_to(ve, NULL, 100) != VE_ERR_ARG) return fail("null pose 未拒");
    if (visiaengine_fly_state(ve, &t01) != 1) return fail("idle 须 done");
    if (t01 != -7.0) return fail("done 路 out 零写被破");
    VeCameraPose tiny = pose;
    tiny.struct_size = 8;
    if (visiaengine_fly_to(ve, &tiny, 100) != VE_ERR_ARG) return fail("struct_size 门");
    VeCameraPose badd = pose;
    badd.dist = -2.0;
    if (visiaengine_fly_to(ve, &badd, 100) != VE_ERR_ARG) return fail("dist 域");
    /* 瞬移形：dur=0 立即落位（读不到 pose——以再飞 done 位姿经 fly_state 恒 1 侧证 + render 通） */
    if (visiaengine_fly_to(ve, &pose, 0) != VE_OK) return fail("teleport");
    if (visiaengine_fly_state(ve, NULL) != 1) return fail("teleport 后未 done");
    if (visiaengine_render(ve) != VE_OK) return fail("render teleported");
    /* 真飞行 300ms：起飞→进度单调采样→到达 done */
    if (visiaengine_fly_to(ve, &pose, 300) != VE_OK) return fail("takeoff");
    double prev = -1.0;
    int samples = 0;
    long deadline = 1200;
    for (long waited = 0; waited < deadline; waited += 20) {
        if (visiaengine_render(ve) != VE_OK) return fail("render flying");
        double t = -9.0;
        int32_t st = visiaengine_fly_state(ve, &t);
        if (st == 0) {
            if (t < prev) return fail("进度倒走");
            prev = t;
            samples++;
            nap_ms(20);
        } else if (st == 1) {
            break;
        } else {
            return fail("fly_state 码外");
        }
    }
    if (samples < 2) return fail("采样不足（时钟未走）");
    if (visiaengine_fly_state(ve, NULL) != 1) return fail("未在时限内到达");
    /* 输入 cancel：长飞行中途 pointer-down → done 且可再飞 */
    VeCameraPose back = pose;
    back.target[0] = -6.0;
    if (visiaengine_fly_to(ve, &back, 4000) != VE_OK) return fail("refly");
    VeInput in = {0};
    in.struct_size = sizeof in;
    in.kind = 2; /* PTR_DOWN */
    if (visiaengine_on_input(ve, &in) != 1) return fail("输入未消费");
    if (visiaengine_fly_state(ve, NULL) != 1) return fail("cancel 未生效");
    if (visiaengine_fly_to(ve, &back, 0) != VE_OK) return fail("cancel 后不可再飞");
    if (visiaengine_destroy(ve) != VE_OK) return fail("destroy");
    printf("OK e815 fly samples=%d progress_max=%.2f gate=域拒+cancel\n", samples, prev);
    return 0;
}
