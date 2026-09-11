# visiaengine-capi 行为契约（CAPI-NN）

C ABI 面条款。**位布局与编码规则正本住本档（C 头不外露 [FFI-R:CS-R1]）**。

## CAPI-01: 句柄编码与世代（slot 基 1）
引擎句柄=`u64 (slot<<32 | generation)`；**slot 自 1 起分配，slot 0 永不占用**（0=失败/无效专用哨兵）；destroy 槽回收（**released 句柄再入含 destroy 本身一律 -1**——双销毁=宿主 bug 信号不吞没）、槽位复用世代前进（stale=旧句柄进门→`VE_ERR_ARG`）；表外 slot（foreign）同 -1。全部带 ve 入口校验先行于状态机校验。实体句柄同形**不同空间**（引擎侧 slot 基 1 偏置不适用于实体：slot0gen0=0 合法；miss/越界哨兵=`UINT64_MAX`，与引擎侧 0 哨兵不对称是有意分工）。abi_version=`(major<<16)|minor`，v0=0x00010000（宿主校验 `>>16==1`）。

## CAPI-02: panic 栅栏（VE_ERR_PANIC 不外溢）
全部 14 extern 入口体经 `capi_guard` 单宏（=catch_unwind(AssertUnwindSafe)+TLS 写诊断+返回 -4）；grep 门：`#[unsafe(no_mangle)]` 计数==14 且 `pub unsafe extern` 签名零命中（安全签名+内部校验，[FFI-R:FC-5]）。栅栏后同 handle 后续入口行为不受污染。

## CAPI-03: 线程亲和与错误串协议
create 记录 owner ThreadId；**例外集={abi_version, last_error}** 外全部入口非 owner 调用→`VE_ERR_STATE`（destroy 跨线程**不销毁**）并在调用方线程 TLS 记含"thread"诊断。错误串协议：`last_error(ve)` 读**调用线程 TLS**，线程绑定、**至该线程下次错误写入前有效**；**仅返回值 <0 分支可写 TLS——任何返回 0/正值入口（含 destroy 幂等 0、on_input 未消费 0）不得触碰**（SDL 警告面：串禁作分支判据）。

## CAPI-06: attach 与窗口生命周期（I3 实装）
`attach(ve, win, display, kind)`：kind 0=x11（**display 必填**——Xlib 无 display 不可构面）1=win32（display 槽=hinstance，0=自动）2=appkit；非法 kind/win=0/必需槽空 → `VE_ERR_ARG`。**原子性**：构面/配置失败保全原 headless 目标（半途不换面，错误走 -2+last_error）。**宿主义务**：窗口与 display 必须存活至 swapchain 析构（destroy 即释放；跨重启持久化句柄被 CAPI-01 世代门拒）。resize=下帧自动重配；Outdated/Lost 帧内消化；Timeout/Occluded=跳帧不计错（Filament beginFrame 语义的 wgpu 对应物，D8 触发器①就此兑现）。管线按 surface caps 格式惰性建（Rgba8Unorm 优先，Bgra* 合法——X11 面格式多样，I3 smoke 实锤）。

## CAPI-05: 输入映射与错误归因（前段，I1 生效子集）
`VeInput.struct_size` 过小→`VE_ERR_ARG`（-1 入口参数家族；-5 专属缓冲/维度运行时量值——归因表）。kind 口径：PTR_DOWN 消费=1、其后 MOVE=orbit 消费=1、未按下的 MOVE=0（非本引擎事件不消费）、WHEEL=zoom 乘性（透视/正交共享 [E3D:B6]）、未登记 kind no-op=0。输入→相机=引擎策略非事件透传。

## CAPI-04: headless 出图字节链
`create_headless→load_*→render→readback` 全链经 C ABI：装载成功=0 且 entity_count 增长、render=0、readback 缓冲不足→-5/空指针→-1、像素回读非背景。viewport(w,h) 0 维→-5 族（w==0 归 -5 量值口径，attach kind 越界归 -1 参数口径）。pick 命中句柄 ∈ {entity_at(i)}（代际稳定），装载件的世界几何同一来源（REND-23/24 复用）。
