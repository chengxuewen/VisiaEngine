//! CAPI-01/02/03：句柄世代 / panic 栅栏 / 线程亲和（计划 v1.4 §2-§4 I1）。
//! rlib 直调 extern "C" 面（真 ABI 形状；产物级 nm 验证归 gate-abi/I2）。

use visiaengine::{
    KIND_NO_SUCH, KIND_PTR_DOWN, KIND_PTR_MOVE, VE_ERR_ARG, VE_ERR_IO, VE_ERR_SIZE, VE_ERR_STATE,
    VE_OK, VeInput, VeMeshDesc, visiaengine_abi_version, visiaengine_add_mesh, visiaengine_attach,
    visiaengine_attr_bool, visiaengine_attr_f64, visiaengine_attr_str, visiaengine_create_headless,
    visiaengine_destroy, visiaengine_entity_at, visiaengine_entity_count,
    visiaengine_entity_set_visible, visiaengine_entity_visible, visiaengine_last_error,
    visiaengine_load_gltf, visiaengine_on_input, visiaengine_pick, visiaengine_readback,
    visiaengine_remove_entity, visiaengine_render, visiaengine_viewport,
};

/// 全入口对 stale/foreign 句柄必须 -1（17→22 谱随带扩，set_event_callback 门在 event_spec）（句柄校验先于状态校验；abi/last_error 无 ve 门）
/// （extern "C" fn 不 coerce 安全 fn 指针——显式枚举门表，[rustc ABI 指针类型]）
fn stale_doors(ve: u64) {
    assert_eq!(visiaengine_destroy(ve), VE_ERR_ARG);
    assert_eq!(visiaengine_attach(ve, 0, 0, 0), VE_ERR_ARG);
    assert_eq!(visiaengine_load_gltf(ve, std::ptr::null()), VE_ERR_ARG);
    assert_eq!(visiaengine_render(ve), VE_ERR_ARG);
    assert_eq!(
        visiaengine_readback(ve, std::ptr::null_mut(), 0),
        VE_ERR_ARG
    );
    assert_eq!(visiaengine_viewport(ve, 8, 8), VE_ERR_ARG);
    assert_eq!(
        visiaengine_attr_f64(ve, 0, std::ptr::null(), std::ptr::null_mut()),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_attr_str(ve, 0, std::ptr::null(), std::ptr::null_mut(), 0),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_attr_bool(ve, 0, std::ptr::null(), std::ptr::null_mut()),
        VE_ERR_ARG
    );
    assert_eq!(visiaengine_entity_set_visible(ve, 0, 1), VE_ERR_ARG);
    assert_eq!(visiaengine_entity_visible(ve, 0), VE_ERR_ARG);
    assert_eq!(
        visiaengine_add_mesh(ve, std::ptr::null(), std::ptr::null_mut()),
        VE_ERR_ARG,
        "stale 引擎门先于参数校验（返回码制，out 零写）"
    );
    assert_eq!(visiaengine_remove_entity(ve, 0), VE_ERR_ARG);
}

#[must_use]
fn input(kind: u32) -> VeInput {
    VeInput {
        struct_size: std::mem::size_of::<VeInput>(),
        kind,
        px: 0.0,
        py: 0.0,
        wheel: 0.0,
        button: 0,
        mods: 0,
    }
}
// 注：on_input/pick/entity_at/entity_count 校验签名不同形，单列断言（CAPI-01 下方）

// spec: CAPI-01
#[test]
fn handle_generation_slot_base1_stale_and_all_doors() {
    // slot 基 1：首个句柄非 0（0=失败哨兵专用）
    let ve = visiaengine_create_headless(64, 64);
    assert_ne!(ve, 0, "slot 基 1：首 create 不得编码为 0");
    // 全入口可用（非 -1/-2(线程)）：entity_count=0 空场景、pick miss 哨兵、on_input 消费性
    assert_eq!(visiaengine_entity_count(ve), 0);
    assert_eq!(
        visiaengine_pick(ve, 32.0, 32.0),
        u64::MAX,
        "空场景未命中=UINT64_MAX"
    );
    assert_eq!(
        visiaengine_entity_at(ve, 0),
        u64::MAX,
        "越界 index=UINT64_MAX"
    );
    // spec: CAPI-05
    // kind 非法（>5）→ 0 未消费（no-op 家族）；struct_size 过小 → -1（归因表 [FFI-R:CS-4]）
    assert_eq!(visiaengine_on_input(ve, &input(KIND_NO_SUCH)), 0);
    let tiny = input(KIND_PTR_MOVE); // 复制后改 size
    let mut tiny = tiny;
    tiny.struct_size = 1;
    assert_eq!(
        visiaengine_on_input(ve, &tiny),
        VE_ERR_ARG,
        "struct_size=1 必须拒"
    );
    // foreign（表外 slot）
    assert_eq!(
        visiaengine_viewport(ve, 0, 0),
        VE_ERR_SIZE,
        "维度违规归 -5（CAPI-05 归因表；RED 期笔误纠）"
    );
    assert_eq!(
        visiaengine_attach(ve, 0, 0, 9),
        VE_ERR_ARG,
        "kind 越界=入口参数 -1"
    );
    // destroy 幂等 → 0；stale 后全门 -1
    assert_eq!(visiaengine_destroy(ve), 0);
    // 语义修正（RED 期设计冲突裁决）：released 句柄再 destroy = -1——
    // 双重销毁是宿主 bug 信号，不做幂等吞没（原"重复=0"与 doors 全 -1 不可共存）
    assert_eq!(
        visiaengine_destroy(ve),
        VE_ERR_ARG,
        "双 destroy 必须报 -1（不吞宿主 bug）"
    );
    assert_eq!(visiaengine_entity_count(ve), VE_ERR_ARG, "stale 句柄必 -1");
    assert_eq!(
        visiaengine_pick(ve, 0.0, 0.0),
        u64::MAX,
        "stale pick=未命中哨兵"
    );
    assert_eq!(visiaengine_load_gltf(ve, std::ptr::null()), VE_ERR_ARG);
    stale_doors(ve); // 六门显式 -1（destroy 已含）
    // 重 create：新句柄 ≠ 旧（世代前进）；两活实例并存
    let ve2 = visiaengine_create_headless(64, 64);
    assert_ne!(ve2, ve);
    assert_ne!(ve2, 0);
    let ve3 = visiaengine_create_headless(320, 240);
    assert!(
        (ve2 & 0xFFFF_FFFF_0000_0000) != (ve3 & 0xFFFF_FFFF_0000_0000)
            || (ve2 >> 32) != (ve3 >> 32)
    );
    // I2 已实装：空场景 render=0（出图链行为断言归 CAPI-04/render_spec）
    assert_eq!(visiaengine_render(ve3), 0);
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
    assert_eq!(
        visiaengine_readback(ve, std::ptr::null_mut(), 0),
        VE_ERR_STATE,
        "I1 无设备 readback=-2 状态语义"
    );
    visiaengine_destroy(ve);
}

// spec: CAPI-03
#[test]
fn thread_affinity_owner_only_with_error_string() {
    let ve = visiaengine_create_headless(64, 64);
    let h = std::thread::spawn(move || {
        // 非 owner 线程：入口 -2；last_error=线程违规语
        assert_eq!(visiaengine_entity_count(ve), VE_ERR_STATE, "跨线程 -2");
        assert_eq!(
            visiaengine_destroy(ve),
            VE_ERR_STATE,
            "destroy 跨线程拒（不销毁）"
        );
        let s = unsafe { std::ffi::CStr::from_ptr(visiaengine_last_error(ve)).to_owned() };
        assert!(
            s.to_str().unwrap().contains("thread"),
            "非 owner 错误串含 thread 指认: {s:?}"
        );
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
    let read = || unsafe { std::ffi::CStr::from_ptr(visiaengine_last_error(ve)).to_owned() };
    let first = read();
    assert!(!first.to_str().unwrap().is_empty(), "错误分支必须写串");
    assert!(
        first.to_str().unwrap().contains("no-such-file"),
        "串要有诊断价值"
    );
    // 写口径：成功入口（返回 0/正值）不得触碰串
    assert_eq!(visiaengine_entity_count(ve), 0);
    // 引擎策略口径：单 MOVE（未按下）不消费=0；PTR_DOWN 后的 MOVE 消费=1（orbit）
    assert_eq!(
        visiaengine_on_input(ve, &input(KIND_PTR_MOVE)),
        0,
        "未按下的移动=非本引擎事件，不消费"
    );
    let _down = input(KIND_PTR_DOWN);
    assert_eq!(visiaengine_on_input(ve, &_down), 1, "按下消费");
    assert_eq!(
        visiaengine_on_input(ve, &input(KIND_PTR_MOVE)),
        1,
        "按下后移动=orbit 消费"
    );
    assert_eq!(
        read().to_str().unwrap(),
        first.to_str().unwrap(),
        "成功调用后串不变（SDL 警告面）"
    );
    // 失效谓词：下一次错误覆盖
    assert_eq!(visiaengine_load_gltf(ve, bad.as_ptr()), VE_ERR_IO);
    // 同路径同串则断"覆盖发生过"用不同路径
    let bad2 = std::ffi::CString::new("no-such-file2.glb").unwrap();
    assert_eq!(visiaengine_load_gltf(ve, bad2.as_ptr()), VE_ERR_IO);
    assert_ne!(
        read().to_str().unwrap(),
        first.to_str().unwrap(),
        "新错误必覆盖旧串"
    );
    visiaengine_destroy(ve);
}

// spec: CAPI-01
#[test]
fn abi_version_packed_and_never_thread_gated() {
    assert_eq!(
        visiaengine_abi_version(),
        0x0001_0003,
        "major 1 · minor 3（CAPI-17 事件口=MAJOR 内追加；demo assert >>16==1 的源头）"
    );
    let h = std::thread::spawn(|| visiaengine_abi_version());
    assert_eq!(h.join().unwrap(), 0x0001_0003, "例外集成员无线程门");
}

// spec: CAPI-02
#[test]
fn symbol_surface_grep_gate() {
    // 入口=17 且风格统一（安全签名内部校验）：文件自扫（[FFI-R:FC-5]）
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/ffi.rs")).unwrap();
    // v1.3 风格演进：C ABI 导出 native-only cfg_attr（wasm 零 no_mangle——与
    // wasm-bindgen 导出表冲突的根修，J1 链接期实锤）；grep 门随形
    assert_eq!(
        src.lines()
            .filter(
                |l| l.starts_with("#[cfg_attr(not(target_arch = \"wasm32\"), unsafe(no_mangle))]")
            )
            .count(),
        22,
        "extern 入口计数（cfg-gated 行首式）"
    );
    assert_eq!(
        src.matches("pub unsafe extern").count(),
        0,
        "禁 unsafe 签名（校验在体内）"
    );
    assert!(
        src.contains("visiaengine_") && !src.contains("cbindgen"),
        "手写头纪律"
    );
}

// ── B1 数据带四口（RED 先行；门表/abi/计数为联动面）──

fn mesh_desc(pos: &[[f32; 3]], nrm: &[[f32; 3]], idx: &[u32]) -> VeMeshDesc {
    VeMeshDesc {
        struct_size: std::mem::size_of::<VeMeshDesc>(),
        positions: pos.as_ptr().cast(),
        normals: nrm.as_ptr().cast(),
        indices: idx.as_ptr(),
        n_positions: pos.len() as u64,
        n_indices: idx.len() as u64,
        base_color: [1.0f32, 1.0, 1.0, 1.0].as_ptr(),
        origin: [0.0f64; 3].as_ptr(),
    }
}

fn load_twoprim(ve: u64) {
    let path = std::ffi::CString::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/data/twoprim.glb"
    ))
    .unwrap();
    assert_eq!(visiaengine_load_gltf(ve, path.as_ptr()), VE_OK);
}

// spec: CAPI-13
// spec: CAPI-14
#[test]
fn entity_visibility_ffi_roundtrip() {
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    load_twoprim(ve);
    let h0 = visiaengine_entity_at(ve, 0);
    assert_eq!(visiaengine_entity_visible(ve, h0), 1, "默认可见");
    assert_eq!(visiaengine_entity_set_visible(ve, h0, 0), VE_OK);
    assert_eq!(visiaengine_entity_visible(ve, h0), 0);
    assert_eq!(
        visiaengine_entity_set_visible(ve, h0, 2),
        VE_ERR_ARG,
        "严格 0/1 入参"
    );
    assert_eq!(visiaengine_entity_set_visible(ve, h0, -1), VE_ERR_ARG);
    assert_eq!(
        visiaengine_entity_set_visible(ve, 777, 1),
        VE_ERR_ARG,
        "未知位形"
    );
    assert_eq!(visiaengine_entity_visible(ve, 777), VE_ERR_ARG);
    assert_eq!(visiaengine_entity_set_visible(ve, h0, 1), VE_OK, "复原可见");
    assert_eq!(visiaengine_entity_count(ve), 2, "显隐不动枚举域");
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-15
#[test]
fn add_mesh_ffi_struct_contract() {
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    let pos = [
        [0f32, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let nrm = [[0f32, 0.0, 1.0]; 4];
    let idx = [0u32, 1, 2, 0, 2, 3];
    let mut out = u64::MAX;
    assert_eq!(
        visiaengine_add_mesh(ve, &mesh_desc(&pos, &nrm, &idx), &mut out),
        VE_OK
    );
    assert_eq!(
        out,
        visiaengine_entity_at(ve, 0),
        "位形=枚举域出口（可=0，禁按 0 判）"
    );
    assert_eq!(visiaengine_entity_count(ve), 1);
    let h_ok = out; // 首成位形另存（下用——out 即将被重置为哨兵戏法）
    out = u64::MAX; // 重置哨兵（上一成功写位形可=0，正是 CAPI-01 谱）
    assert_eq!(
        visiaengine_add_mesh(ve, std::ptr::null(), &mut out),
        VE_ERR_ARG,
        "NULL desc=-1 且 out 不动"
    );
    assert_eq!(out, u64::MAX, "失败路零部分写");
    let small = VeMeshDesc {
        struct_size: 4,
        ..mesh_desc(&pos, &nrm, &idx)
    };
    assert_eq!(
        visiaengine_add_mesh(ve, &small, &mut out),
        VE_ERR_ARG,
        "struct_size 门=拒（前瞻纪律）"
    );
    let oob = [0u32, 9, 2];
    assert_eq!(
        visiaengine_add_mesh(ve, &mesh_desc(&pos, &nrm, &oob), &mut out),
        VE_ERR_ARG,
        "索引越界=退化拒绝"
    );
    assert_eq!(
        visiaengine_add_mesh(ve, &mesh_desc(&pos, &nrm, &idx), std::ptr::null_mut()),
        VE_ERR_ARG,
        "坏 out 自守"
    );
    assert_eq!(visiaengine_render(ve), VE_OK, "加入件即入帧命令流");
    assert_eq!(visiaengine_remove_entity(ve, h_ok), VE_OK);
    assert_eq!(
        visiaengine_entity_count(ve),
        0,
        "退化/失败路零提交（计数全程可追溯）"
    );
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-16
#[test]
fn remove_entity_ffi_reentry_aba() {
    let ve = visiaengine_create_headless(160, 120);
    assert_ne!(ve, 0);
    load_twoprim(ve);
    let h0 = visiaengine_entity_at(ve, 0);
    assert_eq!(visiaengine_remove_entity(ve, h0), VE_OK);
    assert_eq!(visiaengine_entity_count(ve), 1);
    assert_eq!(
        visiaengine_remove_entity(ve, h0),
        VE_ERR_ARG,
        "旧句柄再入=双销毁同谱"
    );
    assert_eq!(visiaengine_entity_visible(ve, h0), VE_ERR_ARG);
    let pos = [
        [0f32, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];
    let nrm = [[0f32, 0.0, 1.0]; 4];
    let idx = [0u32, 1, 2, 0, 2, 3];
    let mut h_new = 0u64;
    assert_eq!(
        visiaengine_add_mesh(ve, &mesh_desc(&pos, &nrm, &idx), &mut h_new),
        VE_OK
    );
    assert_ne!(h_new, h0, "ABA：同槽新代际不撞旧位形");
    assert_eq!(
        visiaengine_remove_entity(ve, h0),
        VE_ERR_ARG,
        "新实体在场，旧句柄仍死"
    );
    assert_eq!(visiaengine_remove_entity(ve, h_new), VE_OK);
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}
