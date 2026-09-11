# visiaengine-capi 行为契约（CAPI-NN）

C ABI 面条款。**位布局与编码规则正本住本档（C 头不外露 [FFI-R:CS-R1]）**。

## CAPI-01: 句柄编码与世代（slot 基 1）
引擎句柄=`u64 (slot<<32 | generation)`；**slot 自 1 起分配，slot 0 永不占用**（0=失败/无效专用哨兵）；destroy 槽回收、世代前进（stale=destroy 后复用槽位以旧句柄进门→`VE_ERR_ARG`）；表外 slot（foreign）同 -1。全部带 ve 入口校验先行于状态机校验。实体句柄同形**不同空间**（引擎侧 slot 基 1 偏置不适用于实体：slot0gen0=0 合法；miss/越界哨兵=`UINT64_MAX`，与引擎侧 0 哨兵不对称是有意分工）。abi_version=`(major<<16)|minor`，v0=0x00010000（宿主校验 `>>16==1`）。

## CAPI-02: panic 栅栏（VE_ERR_PANIC 不外溢）
全部 14 extern 入口体经 `capi_guard` 单宏（=catch_unwind(AssertUnwindSafe)+TLS 写诊断+返回 -4）；grep 门：`#[unsafe(no_mangle)]` 计数==14 且 `pub unsafe extern` 签名零命中（安全签名+内部校验，[FFI-R:FC-5]）。栅栏后同 handle 后续入口行为不受污染。

## CAPI-03: 线程亲和与错误串协议
create 记录 owner ThreadId；**例外集={abi_version, last_error}** 外全部入口非 owner 调用→`VE_ERR_STATE`（destroy 跨线程**不销毁**）并在调用方线程 TLS 记含"thread"诊断。错误串协议：`last_error(ve)` 读**调用线程 TLS**，线程绑定、**至该线程下次错误写入前有效**；**仅返回值 <0 分支可写 TLS——任何返回 0/正值入口（含 destroy 幂等 0、on_input 未消费 0）不得触碰**（SDL 警告面：串禁作分支判据）。

## CAPI-05: 输入映射与错误归因（前段，I1 生效子集）
`VeInput.struct_size` 过小→`VE_ERR_ARG`（-1 入口参数家族；-5 专属缓冲/维度运行时量值——归因表）。kind 口径：PTR_DOWN 消费=1、其后 MOVE=orbit 消费=1、未按下的 MOVE=0（非本引擎事件不消费）、WHEEL=zoom 乘性（透视/正交共享 [E3D:B6]）、未登记 kind no-op=0。输入→相机=引擎策略非事件透传。
