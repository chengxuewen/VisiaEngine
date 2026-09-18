/* E701 · 绑定镜像·C headless —— canonical 10 行嵌入样板（文档件，[FFI-R:EP-附]；
 * 跑不通即 API 未完成 [E3D:D7]）。
 * 构建与运行见 scripts/gate-abi.sh；亦可直接：
 *   x86_64-conda-linux-gnu-cc -I include demo_headless.c -L target/debug \
 *     -lvisiaengine -o target/demo_headless && LD_LIBRARY_PATH=target/debug ./target/demo_headless */
#include <stddef.h>
#include <stdio.h>

#include "visiaengine.h"

int main(int argc, char **argv) {
    if (visiaengine_abi_version() >> 16 != 1) { puts("FAIL abi major"); return 1; } /* ABI 门 [FFI-R:v13-FEAS-2] */
    uint64_t ve = visiaengine_create_headless(160, 120);
    if (!ve) { puts("FAIL create"); return 1; }
    if (visiaengine_load_gltf(ve, argc > 1 ? argv[1] : "resources/data/twoprim.glb") != VE_OK) {
        printf("FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    for (int i = 0; i < 3; ++i) {
        if (visiaengine_render(ve) != VE_OK) { puts("FAIL render"); return 1; }
    }
    static unsigned char buf[160 * 120 * 4];
    if (visiaengine_readback(ve, buf, sizeof buf) != VE_OK) { puts("FAIL readback"); return 1; }
    size_t mid = (size_t)((60 * 160 + 80) * 4);
    unsigned lum = (unsigned)(buf[mid] + buf[mid + 1] + buf[mid + 2]);
    if (lum <= 40) { printf("FAIL dark center lum=%u\n", lum); return 1; }
    uint64_t hit = visiaengine_pick(ve, 80.0f, 60.0f);
    if (hit == VE_MISS) { puts("FAIL pick miss on lit pixel"); return 1; }
    /* input 段（I4）：DOWN→MOVE=orbit、WHEEL=zoom（消费性断言 1） */
    VeInput in_ = { sizeof(VeInput), KIND_PTR_DOWN, 30.0f, 30.0f, 0.0f, 1u, 0u };
    if (visiaengine_on_input(ve, &in_) != 1) { puts("FAIL down unconsumed"); return 1; }
    in_.kind = KIND_PTR_MOVE; in_.px = 60.0f; in_.py = 40.0f;
    if (visiaengine_on_input(ve, &in_) != 1) { puts("FAIL move unconsumed"); return 1; }
    in_.kind = KIND_PTR_UP;
    (void)visiaengine_on_input(ve, &in_);
    in_.kind = KIND_WHEEL; in_.wheel = 1.0f;
    if (visiaengine_on_input(ve, &in_) != 1) { puts("FAIL wheel unconsumed"); return 1; }
    if (visiaengine_render(ve) != VE_OK) { puts("FAIL render after input"); return 1; }
    if (visiaengine_readback(ve, buf, sizeof buf) != VE_OK) { puts("FAIL readback2"); return 1; }
    in_.struct_size = 1; /* 演进锚探针：过小必拒 */
    if (visiaengine_on_input(ve, &in_) != VE_ERR_ARG) { puts("FAIL struct_size gate"); return 1; }
    /* 属性读段（CAPI-10/11）：geo 加性装载→按 entity 句柄读列；
       故意读缺键 height=0 且 out 不动=缺失≠零值的活广告 */
    int32_t base = visiaengine_entity_count(ve);
    if (visiaengine_load_geojson(ve, "resources/data/park.geojson") != VE_OK) {
        printf("FAIL geo load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    uint64_t feat = visiaengine_entity_at(ve, (uint32_t)base);
    char nm[64];
    if (visiaengine_attr_str(ve, feat, "name", nm, sizeof nm) != 1) { puts("FAIL attr name"); return 1; }
    double op = -1.0;
    if (visiaengine_attr_f64(ve, feat, "fill-opacity", &op) != 1 || op < 0.79 || op > 0.81) {
        puts("FAIL attr opacity");
        return 1;
    }
    double ghost = -1.0;
    if (visiaengine_attr_f64(ve, feat, "height", &ghost) != 0 || ghost != -1.0) {
        puts("FAIL missing-not-zero breach");
        return 1;
    }
    printf("OK attrs name=%s opacity=%.2f missing-intact\n", nm, op);
    /* 文字闭环（CAPI-21/22）：注字体→世界锚标签→渲染；时序门活广告 */
    FILE *ff = fopen("resources/data/DejaVuSans.ttf", "rb");
    if (!ff) { puts("FAIL font open"); return 1; }
    static unsigned char fbuf[1024 * 1024];
    size_t flen = fread(fbuf, 1, sizeof fbuf, ff);
    fclose(ff);
    VeLabelSpec lspec = {0};
    lspec.struct_size = sizeof lspec;
    lspec.pos[0] = 0.0; lspec.pos[1] = 0.0; lspec.pos[2] = 1.0;
    lspec.color[0] = lspec.color[1] = lspec.color[2] = 1.0f; lspec.color[3] = 1.0f;
    lspec.size_px = 20.0f;
    lspec.text = "E701";
    uint64_t lab = 0;
    if (visiaengine_add_label(ve, &lspec, &lab) != VE_ERR_ARG) {
        puts("FAIL label-before-font gate"); return 1;
    }
    if (flen == 0 || flen == sizeof fbuf || visiaengine_load_font(ve, fbuf, flen) != VE_OK) {
        puts("FAIL load_font"); return 1;
    }
    if (visiaengine_add_label(ve, &lspec, &lab) != VE_OK) {
        printf("FAIL add_label: %s\n", visiaengine_last_error(ve)); return 1;
    }
    if (visiaengine_render(ve) != VE_OK) { puts("FAIL render labels"); return 1; }
    puts("OK font+label");
    visiaengine_destroy(ve);
    puts("OK capi headless input+pick+attrs");
    return 0;
}
