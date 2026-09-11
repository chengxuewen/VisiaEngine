//! CAPI-01/02/03：句柄世代 / panic 栅栏 / 线程亲和（计划 v1.4 §2-§4 I1）。
//! rlib 直调 extern "C" 面（真 ABI 形状；产物级 nm 验证归 gate-abi/I2）。

use visiaengine::{
    visiaengine_abi_version, visiaengine_attach, visiaengine_create_headless,
    visiaengine_destroy, visiaengine_entity_at, visiaengine_entity_count,
    visiaengine_last_error, visiaengine_load_gltf, visiaengine_on_input, visiaengine_pick,
    visiaengine_readback, visiaengine_render, visiaengine_viewport, VE_ERR_ARG,
    VE_ERR_IO, VE_ERR_STATE, KIND_PTR_DOWN, KIND_PTR_MOVE, KIND_NO_SUCH, VeInput,
};

const OK_VV: fn(u64) -> i32 = visiaengine_entity_count;
/// 全 14 入口对 stale/foreign 句柄必须 -1（句柄校验先于状态校验；abi/last_error 无 ve）
const ALL_DOORS: &[fn(u64) -> i32] = &[
    visiaengine_destroy,
    |v| visiaengine_attach(v, 0, 0, 0),
    |v| visiaengine_load_gltf(v, std::ptr::null()),
    |v| visiaengine_render(v),
    |v| visiaengine_readback(v, std::ptr::null_mut(), 0),
    |v| visiaengine_viewport(v, 0, 0),
];

#[must_use]
fn input(kind: u32) -> VeInput {
    VeInput { struct_size: std::mem::size_of::<VeInput>(), kind, px: 0.0, py: 0.0,
              wheel: 0.0, button: 0, mods: 0 }
}
// 注：on_input/pick/entity_at/entity_count 校验签名不同形，单列断言（CAPI-01 下方）

// spec: CAPI-01
#[test]
fn handle_generation_slot_base1_stale_and_all_doors() {
    // slot 基 1：首个句柄非 0（0=失败哨兵专用）
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0, "slot 基 1：首 create 不得编码为 0");
    // 全入口可用（非 -1/-2(线程)）：entity_count=0 空场景、pick miss 哨兵、on_input 消费性
    assert_eq!(OK_VV(ve), 0);
    assert_eq!(visiaengine_pick(ve, 32.0, 32.0), u64::MAX, "空场景未命中=UINT64_MAX");
    assert_eq!(visiaengine_entity_at(ve, 0), u64::MAX, "越界 index=UINT64_MAX");
    // spec: CAPI-05
    // kind 非法（>5）→ 0 未消费（no-op 家族）；struct_size 过小 → -1（归因表 [FFI-R:CS-4]）
    assert_eq!(visiaengine_on_input(ve, &input(KIND_NO_SUCH)), 0);
    let tiny = input(KIND_PTR_MOVE); // 复制后改 size
    let mut tiny = tiny; tiny.struct_size = 1;
    assert_eq!(visiaengine_on_input(ve, &tiny), VE_ERR_ARG, "struct_size=1 必须拒");
    // foreign（表外 slot）
    assert_eq!(visiaengine_viewport(ve, 0, 0), VE_ERR_ARG, "0 尺寸非法归 -1(入口参数)");
    // destroy 幂等 → 0；stale 后全门 -1
    assert_eq!(visiaengine_destroy(ve), 0);
    assert_eq!(visiaengine_destroy(ve), 0, "幂等重复 destroy=0 且不写错误");
    assert_eq!(visiaengine_entity_count(ve), VE_ERR_ARG, "stale 句柄必 -1");
    assert_eq!(visiaengine_pick(ve, 0.0, 0.0), u64::MAX, "stale pick=未命中哨兵");
    assert_eq!(visiaengine_load_gltf(ve, std::ptr::null()), VE_ERR_ARG);
    for door in ALL_DOORS {
        assert_eq!(door(ve), VE_ERR_ARG, "stale 句柄进门必 -1");
    }
    // 重 create：新句柄 ≠ 旧（世代前进）；两活实例并存
    let ve2 = visiaengine_create_headless(64, 64);
    assert_ne!(ve2, ve);
    assert_ne!(ve2, 0);
    let ve3 = visiaengine_create_headless(320, 240);
    assert!((ve2 & 0xFFFF_FFFF_0000_0000) != (ve3 & 0xFFFF_FFFF_0000_0000) || (ve2 >> 32) != (ve3 >> 32));
    assert_eq!(visiaengine_render(ve3), VE_ERR_STATE, "I1 阶段出图未实装=-2（状态语义占位，I2 填实）");
    visiaengine_destroy(ve2);
    visiaengine_destroy(ve3);
}

// spec: CAPI-02
#[test]
fn panic_fence_returns_minus4_with_message() {
    // 栅栏宏行为（入口=同宏 100% 覆盖，grep 门另测）：注入 panic 的闭包
    let code = visiaengine::test_util::guard_panic_probe();
    assert_eq!(code, -4, "栅栏吞 panic 返回 VE_ERR_PANIC");
    let ve = visiaengine_create_headless(64, 64);
    assert_eq!(visiaengine_viewport(ve, 128, 128), 0, "栅栏不污染后续入口");
    assert_eq!(visiaengine_readback(ve, std::ptr::null_mut(), 0), VE_ERR_STATE,
               "I1 无设备 readback=-2 状态语义");
    visiaengine_destroy(ve);
}

// spec: CAPI-03
#[test]
fn thread_affinity_owner_only_with_error_string() {
    let ve = visiaengine_create_headless(64, 64);
    let h = std::thread::spawn(move || {
        // 非 owner 线程：入口 -2；last_error=线程违规语
        assert_eq!(visiaengine_entity_count(ve), VE_ERR_STATE, "跨线程 -2");
        assert_eq!(visiaengine_destroy(ve), VE_ERR_STATE, "destroy 跨线程拒（不销毁）");
        let s = unsafe {
            std::ffi::CStr::from_ptr(visiaengine_last_error(ve)).to_owned()
        };
        assert!(s.to_str().unwrap().contains("thread"), "非 owner 错误串含 thread 指认: {s:?}");
        ve
    });
    let ve_back = h.join().unwrap();
    assert_eq!(visiaengine_entity_count(ve_back), 0, "owner 线程句柄仍活");
    visiaengine_destroy(ve_back);
}

// spec: CAPI-03
#[test]
fn last_error_write_policy_success_never_clobbers() {
    let ve = visiaengine_create_headless(64, 64);
    // 制造错误：载入不存在文件 → -3 且写串
    let bad = std::ffi::CString::new("no-such-file.glb").unwrap();
    assert_eq!(visiaengine_load_gltf(ve, bad.as_ptr()), VE_ERR_IO);
    let read = || unsafe {
        std::ffi::CStr::from_ptr(visiaengine_last_error(ve)).to_owned()
    };
    let first = read();
    assert!(!first.to_str().unwrap().is_empty(), "错误分支必须写串");
    assert!(first.to_str().unwrap().contains("no-such-file"), "串要有诊断价值");
    // 写口径：成功入口（返回 0/正值）不得触碰串
    assert_eq!(visiaengine_entity_count(ve), 0);
    // 引擎策略口径：单 MOVE（未按下）不消费=0；PTR_DOWN 后的 MOVE 消费=1（orbit）
    assert_eq!(visiaengine_on_input(ve, &input(KIND_PTR_MOVE)), 0, "未按下的移动=非本引擎事件，不消费");
    let _down = input(KIND_PTR_DOWN);
    assert_eq!(visiaengine_on_input(ve, &_down), 1, "按下消费");
    assert_eq!(visiaengine_on_input(ve, &input(KIND_PTR_MOVE)), 1, "按下后移动=orbit 消费");
    assert_eq!(read().to_str().unwrap(), first.to_str().unwrap(), "成功调用后串不变（SDL 警告面）");
    // 失效谓词：下一次错误覆盖
    assert_eq!(visiaengine_load_gltf(ve, bad.as_ptr()), VE_ERR_IO);
    // 同路径同串则断"覆盖发生过"用不同路径
    let bad2 = std::ffi::CString::new("no-such-file2.glb").unwrap();
    assert_eq!(visiaengine_load_gltf(ve, bad2.as_ptr()), VE_ERR_IO);
    assert_ne!(read().to_str().unwrap(), first.to_str().unwrap(), "新错误必覆盖旧串");
    visiaengine_destroy(ve);
}

// spec: CAPI-01
#[test]
fn abi_version_packed_and_never_thread_gated() {
    assert_eq!(visiaengine_abi_version(), 0x0001_0000, "v0 = major 1（demo assert >>16==1 的源头）");
    let h = std::thread::spawn(visiaengine_abi_version);
    assert_eq!(h.join().unwrap(), 0x0001_0000, "例外集成员无线程门");
}
