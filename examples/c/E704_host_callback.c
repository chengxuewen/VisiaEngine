/* E704 · 绑定镜像·C 事件回调 —— CAPI-17 推送口验收例：进度/错误/摘除三态全走。
 * 构建与运行：ctest example_E704_host_callback（gate/cmake-smoke 路）；手跑：
 *   cc -I bindings/c/visiaengine-capi/include examples/c/E704_host_callback.c \
 *      -L target/debug -lvisiaengine -o target/e704 && LD_LIBRARY_PATH=target/debug ./target/e704 */
#include <stdint.h>
#include <stdio.h>

#include "visiaengine.h"

/* 同步语义=调用线程触发（条款体声明的 C 侧活证）：普通静态量即可，无原子义务。 */
static uint64_t g_done_last;
static uint64_t g_prog;
static uint64_t g_error;
static uint64_t g_last_b;

static void on_event(void *user, uint32_t event, uint64_t a, uint64_t b) {
    (void)user;
    if (event == VE_EVT_LOAD_PROGRESS) {
        if (++g_prog == 1) { g_done_last = 0; }
        if (a <= g_done_last) { puts("FAIL progress 非单调"); }
        g_done_last = a;
        g_last_b = b;
    } else if (event == VE_EVT_LOAD_ERROR) {
        ++g_error;
        if ((int64_t)a != VE_ERR_IO) { printf("FAIL error code=%llu\n", (unsigned long long)a); }
    } else {
        printf("FAIL unknown event %u\n", event);
    }
}

int main(void) {
    if (visiaengine_abi_version() >> 16 != 1) { puts("FAIL abi major"); return 1; }
    uint64_t ve = visiaengine_create_headless(160, 120);
    if (!ve) { puts("FAIL create"); return 1; }
    if (visiaengine_set_event_callback(ve, on_event, NULL) != 0) { puts("FAIL set"); return 1; }
    /* ① 进度态：park.geojson=5 feature，终事件必 done==total */
    if (visiaengine_load_geojson(ve, "resources/data/park.geojson") != VE_OK) {
        printf("FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    if (g_prog < 5 || g_done_last != g_last_b) {
        printf("FAIL 进度面 prog=%llu terminal=(%llu,%llu)\n",
               (unsigned long long)g_prog, (unsigned long long)g_done_last,
               (unsigned long long)g_last_b);
        return 1;
    }
    /* ② 错误态：缺失文件=返回值与 LOAD_ERROR 同刻 */
    if (visiaengine_load_geojson(ve, "resources/data/does-not-exist.geojson") == VE_OK) {
        puts("FAIL 缺失文件竟成功");
        return 1;
    }
    if (g_error != 1) { printf("FAIL 无错误事件 error=%llu\n", (unsigned long long)g_error); return 1; }
    /* ③ 摘除态：NULL 再注册后装载=进度静默 */
    const uint64_t snapshot = g_prog;
    if (visiaengine_set_event_callback(ve, NULL, NULL) != 0) { puts("FAIL null set"); return 1; }
    if (visiaengine_load_geojson(ve, "resources/data/park.geojson") != VE_OK) { puts("FAIL reload"); return 1; }
    if (g_prog != snapshot) { puts("FAIL 摘除后仍触发"); return 1; }
    if (visiaengine_render(ve) != VE_OK) { puts("FAIL render"); return 1; }
    printf("OK e704 events prog=%llu error=%llu\n", (unsigned long long)g_prog,
           (unsigned long long)g_error);
    return visiaengine_destroy(ve) == 0 ? 0 : 1;
}
