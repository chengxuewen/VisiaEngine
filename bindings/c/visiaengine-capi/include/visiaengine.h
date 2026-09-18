/* visiaengine.h —— 手写人审骨架（I0）；14 入口签名照 .omo/plans/visiaengine-host-embed.md v1.4 §2，
 * I1 起逐函数填充：每函数一行用途 + 一行 threadsafety（[FFI-R:CS-M2]）。 */
#ifndef VISIAENGINE_H
#define VISIAENGINE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {   /* C++ 宿主护栏（Qt/Flutter 嵌入面；批 C2 ctest 探针实证缺口） */
#endif

/* 句柄为不透明值：仅 0=无效/失败、UINT64_MAX=未命中哨兵有公共语义；
 * 禁拆解、禁跨进程/跨重启持久化（编码与世代规则住 CAPI-01 合同，不外露位布局
 * [FFI-R:CS-R1]）。 */
typedef uint64_t VeEngine;

typedef struct { size_t struct_size; uint32_t kind;
                 float px, py, wheel; uint32_t button, mods; } VeInput;

#define VE_OK            0
#define VE_ERR_ARG      (-1)   /* 句柄无效/stale/foreign/空指针/struct_size 过小/kind 越界 */
#define VE_ERR_STATE    (-2)   /* 未 attach/非 owner 线程（线程亲和：入口仅 owner 线程可调） */
#define VE_ERR_IO       (-3)
#define VE_ERR_PANIC    (-4)   /* 栅栏兜底；诊断读 last_error */
#define VE_ERR_SIZE     (-5)   /* 缓冲/维度运行时量值 */
#define KIND_PTR_MOVE 1u
#define KIND_PTR_DOWN 2u
#define KIND_PTR_UP   3u
#define KIND_WHEEL    4u
#define KIND_KEY      5u
#define VE_MISS       UINT64_MAX  /* pick/entity_at 未命中与越界哨兵 */

/* 	hreadsafety 每入口：owner 线程；create 记录属主。例外={abi_version, last_error}。
 * 宿主义务：输入线程≠paint 线程时（Qt 类）转送 owner 线程。 */
uint32_t   visiaengine_abi_version(void);                       /* (major<<16)|minor；v0 major=1 */
uint64_t   visiaengine_create_headless(uint32_t w, uint32_t h); /* 0=失败（诊断读 last_error） */
int32_t    visiaengine_destroy(uint64_t ve);                    /* released 再入 -1（双销毁不吞没） */
int32_t    visiaengine_attach(uint64_t ve, uint64_t win, uint64_t display_or_hinstance, int32_t kind); /* 0=x11 1=win32 2=cocoa；重复 attach=旧目标释放后重配 */
int32_t    visiaengine_load_gltf(uint64_t ve, const char *path);
int32_t    visiaengine_load_geojson(uint64_t ve, const char *path);
int32_t    visiaengine_on_input(uint64_t ve, const VeInput *in); /* 唯一输入口；1=消费 0=非本引擎事件 */
int32_t    visiaengine_render(uint64_t ve);                     /* 拉模型一帧（宿主 rAF/paint 时机自决） */
int32_t    visiaengine_readback(uint64_t ve, uint8_t *buf, uint64_t len);
int32_t    visiaengine_viewport(uint64_t ve, uint32_t w, uint32_t h);
const char *visiaengine_last_error(uint64_t ve);                /* 线程绑定；至该线程下次错误写入前有效；禁作分支判据 */
int32_t    visiaengine_entity_count(uint64_t ve);
uint64_t   visiaengine_pick(uint64_t ve, float px, float py);   /* 实体句柄（异空间）；VE_MISS=未命中 */
uint64_t   visiaengine_entity_at(uint64_t ve, uint32_t index);
/* 属性读（CAPI-10/11）：entity=pick/entity_at 出口位形；返回 1=命中 0=缺失(out/缓冲不动) <0=错误。
   str: NUL 终止写 buf[cap]，cap 不足=-5 零部分写（无探长模式），宿主习惯 256B。 */
int32_t    visiaengine_attr_f64(uint64_t ve, uint64_t entity, const char *key, double *out);
int32_t    visiaengine_attr_str(uint64_t ve, uint64_t entity, const char *key, char *buf, uint64_t cap);
int32_t    visiaengine_attr_bool(uint64_t ve, uint64_t entity, const char *key, int32_t *out);

/* ── B1 数据带（CAPI-13..16，abi minor=2）──
   显隐：strict visible∈{0,1}；隐藏件 render/pick 不可见但枚举域不变（图层开关谱）。 */
int32_t visiaengine_entity_set_visible(uint64_t ve, uint64_t entity, int32_t visible);
int32_t visiaengine_entity_visible(uint64_t ve, uint64_t entity); /* 1/0/-1 */

/* 程序化加网格（CAPI-15）：struct_size 前瞻门；指针仅调用期读取；
   normals 可 NULL=引擎合成 +Z。返回码制：0=成功写 *out_entity（位形可=0，
   宿主禁按 0 判有效性——枚举域/pick 出口才是存在性判据，CAPI-01 分工），
   <0=失败零部分写（attr_str 纪律同谱）。 */
typedef struct VeMeshDesc {
    size_t    struct_size;
    const float    *positions;   /* [x,y,z] × n_positions */
    const float    *normals;     /* 可 NULL；否则同长度 */
    const uint32_t *indices;
    uint64_t  n_positions;
    uint64_t  n_indices;
    const float    *base_color;  /* 4 元组，sRGB/CSS 惯例值（后端咽喉转线性，CORE-16） */
    const double   *origin;      /* 3 元组（D7 远坐标语义） */
} VeMeshDesc;
int32_t  visiaengine_add_mesh(uint64_t ve, const VeMeshDesc *desc, uint64_t *out_entity);
/* 删除（CAPI-16）：items/属性/显隐三面清理，槽位代际 +1；旧句柄再入=-1 双销毁同谱。 */
int32_t  visiaengine_remove_entity(uint64_t ve, uint64_t entity);
/* 事件推送（CAPI-17）：注册/替换/NULL 摘除；load_* 循环内逐要素**同步**触发
   PROGRESS(a=done b=total，单调且终态 done==total)；失败出口与返回值同刻触发
   ERROR(a=返回码 signed 形 b=0)。触发线程=调用线程（异步管线落地后=loader 线程，
   id/序不变——前向兼容声明）。回调内再入 visiaengine_* = 未定义（禁）。 */
typedef enum { VE_EVT_LOAD_PROGRESS = 1, VE_EVT_LOAD_ERROR = 2 } VeEvent;
typedef void (*VeEventCb)(void *user, uint32_t event, uint64_t a, uint64_t b);
int32_t  visiaengine_set_event_callback(uint64_t ve, VeEventCb cb, void *user);
/* 点云直通（CAPI-18）：单实体单 DrawPoints；非有限照收=宿主责任（load 侧才四分类
   丢弃）；管理域=items 外（geo 纯 marker 同谱 v0 明账）。色=sRGB 宿主面（CORE-16）。 */
typedef struct {
    float pos[3];      /* 宿主系局部坐标（origin=0） */
    float radius_px;   /* 屏幕像素（REND-30 语义） */
    float color[3];    /* sRGB/CSS 惯例 */
} VePointMark;
typedef struct {
    size_t             struct_size; /* 前瞻门=sizeof(VePointsDesc) */
    const VePointMark *marks;
    uint64_t           count;
} VePointsDesc;
int32_t  visiaengine_add_points(uint64_t ve, const VePointsDesc *desc, uint64_t *out_entity);
/* 点云文件装载（CAPI-19）：v0=PLY(ascii/binary_le)；四类脏数据分型见 IO-03（列/元素级
   unsupported 不触发 FastFail——点级丢弃/截断才整拒）；云=单实体，meta（point_count/
   format/bbox 8 列）走 attr_* 缀查；容量门 4M 声明先拒（IO-05）。 */
typedef enum { VE_PCL_FASTFAIL = 0, VE_PCL_LENIENT = 1 } VePclPolicy;
typedef struct {
    size_t   struct_size;
    uint32_t dropped_non_finite;
    uint32_t dropped_out_of_domain;
    uint32_t dropped_unsupported; /* 列/元素级计数，不触发 FastFail（IO-03 落点澄清） */
    uint32_t truncated_points;
    uint64_t kept;
} VePclReport;
int32_t  visiaengine_load_pcl(uint64_t ve, const char *path, VePclPolicy policy,
                              uint64_t *out_entity, VePclReport *out_report);
/* 剖面裁切（CAPI-20）：世界系数面 [n,d]（法向指保留侧，dot(n,P)+d≥0 保留，面上=
   保留；多面 AND）。归一化引擎侧做（读回恒单位形）；n=0=唯一清空形；n>4 先拒后读。
   render 管线 fs discard（含 shadow caster 同裁）+ pick 命中点重试过滤同帧一致。 */
typedef struct { double nx, ny, nz, d; } VeClipPlane;
int32_t  visiaengine_set_clips(uint64_t ve, const VeClipPlane *planes, size_t n);
/* 读回：返回≥0=当前面数（错误=负码专属）；buf NULL=仅计数；cap 截断=写 min 而返回真数。 */
int32_t  visiaengine_get_clips(uint64_t ve, VeClipPlane *buf, size_t cap);
/* 文字标注（CAPI-21/22，档①引擎自管）：load_font 注入 TTF/OTF 字节（替换式；无默认
   字体=文字管线休眠）。add_label 世界锚 f64 [D7]、色=宿主 sRGB/CSS 面（CORE-16 咽喉转
   线性）、text=NUL 终止 UTF-8；成功 rc=0 写 *out_entity（位形 0 合法=CAPI-01，成败看
   rc=CAPI-15 谱）。管理域=items 外（add_points CAPI-18 同谱）：不入 remove/enumeration，
   明账 v0。未载字体=add_label 显式拒（时序门；休眠语义只属 GEO-25 样式域）。 */
typedef struct {
    size_t             struct_size; /* 前瞻门 = sizeof(VeLabelSpec) */
    double             pos[3];
    float              color[4];    /* sRGB/CSS */
    float              size_px;
    const char        *text;        /* UTF-8, NUL 终止 */
} VeLabelSpec;
int32_t  visiaengine_load_font(uint64_t ve, const uint8_t *data, size_t len);
int32_t  visiaengine_add_label(uint64_t ve, const VeLabelSpec *spec, uint64_t *out_entity);

#ifdef __cplusplus
}  /* extern "C" */
#endif

#endif /* VISIAENGINE_H */
