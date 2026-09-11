/* visiaengine.h —— 手写人审骨架（I0）；14 入口签名照 .omo/plans/visiaengine-host-embed.md v1.4 §2，
 * I1 起逐函数填充：每函数一行用途 + 一行 threadsafety（[FFI-R:CS-M2]）。 */
#ifndef VISIAENGINE_H
#define VISIAENGINE_H

#include <stdint.h>
#include <stddef.h>

/* 句柄为不透明值：仅 0=无效/失败、UINT64_MAX=未命中哨兵有公共语义；
 * 禁拆解、禁跨进程/跨重启持久化（编码与世代规则住 CAPI-01 合同，不外露位布局
 * [FFI-R:CS-R1]）。 */
typedef uint64_t VeEngine;

/* I1+: 14 入口 + VeInput struct + 返回码宏 + kind/错误码常量镜像 */

#endif /* VISIAENGINE_H */
