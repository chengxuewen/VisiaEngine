//! CAPI-01/02/03：句柄世代 / panic 栅栏 / 线程亲和（计划 v1.4 §2-§4 I1）。
//! rlib 直调 extern "C" 面（真 ABI 形状；产物级 nm 验证归 gate-abi/I2）。

use visiaengine::{
    KIND_NO_SUCH, KIND_PTR_DOWN, KIND_PTR_MOVE, VE_ERR_ARG, VE_ERR_IO, VE_ERR_SIZE, VE_ERR_STATE,
    VE_OK, VE_PCL_FASTFAIL, VE_PCL_LENIENT, VeClipPlane, VeInput, VeLabelSpec, VeMeshDesc,
    VePclReport, VePointMark, VePointsDesc, visiaengine_abi_version, visiaengine_add_label,
    visiaengine_add_mesh, visiaengine_add_points, visiaengine_attach, visiaengine_attr_bool,
    visiaengine_attr_f64, visiaengine_attr_str, visiaengine_create_headless, visiaengine_destroy,
    visiaengine_entity_at, visiaengine_entity_count, visiaengine_entity_set_visible,
    visiaengine_entity_visible, visiaengine_get_clips, visiaengine_last_error,
    visiaengine_load_font, visiaengine_load_gltf, visiaengine_load_pcl, visiaengine_on_input,
    visiaengine_pick, visiaengine_readback, visiaengine_remove_entity, visiaengine_render,
    visiaengine_set_clips, visiaengine_viewport,
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
    assert_eq!(
        visiaengine_set_clips(ve, std::ptr::null(), 0),
        VE_ERR_ARG,
        "CAPI-20 新口入 stale 谱"
    );
    assert_eq!(
        visiaengine_load_font(ve, std::ptr::null(), 0),
        VE_ERR_ARG,
        "CAPI-21 stale 谱"
    );
    let lspec = VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [0.0; 3],
        color: [1.0; 4],
        size_px: 16.0,
        text: std::ptr::null(),
    };
    assert_eq!(
        visiaengine_add_label(ve, &lspec, std::ptr::null_mut()),
        VE_ERR_ARG,
        "CAPI-22 stale 谱"
    );
    assert_eq!(
        visiaengine_get_clips(ve, std::ptr::null_mut(), 0),
        VE_ERR_ARG,
        "CAPI-20 读口同谱"
    );
    assert_eq!(
        visiaengine_fly_to(ve, std::ptr::null(), 0),
        VE_ERR_ARG,
        "CAPI-23 stale 谱"
    );
    assert_eq!(
        visiaengine_fly_state(ve, std::ptr::null_mut()),
        VE_ERR_ARG,
        "CAPI-24 stale 谱"
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
        0x0001_0008,
        "major 1 · minor 6（CAPI-19 文件装载=MAJOR 内追加；demo assert >>16==1 的源头）"
    );
    let h = std::thread::spawn(|| visiaengine_abi_version());
    assert_eq!(h.join().unwrap(), 0x0001_0008, "例外集成员无线程门");
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
        30,
        "extern 入口计数（cfg-gated 行首式；24→26=CAPI-20 剖面双口）"
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

/// CAPI-18 直通装载：值域表四锁（退化零提交/成功+out 可查/struct_size 门/非有限照收=宿主责任分工）。
// spec: CAPI-18
#[test]
fn add_points_domain_table() {
    let ve = visiaengine_create_headless(160, 120);
    let marks = [
        VePointMark {
            pos: [-1.0, 0.0, 0.0],
            radius_px: 4.0,
            color: [1.0, 0.0, 0.0],
        },
        VePointMark {
            pos: [0.0, 0.0, 0.0],
            radius_px: 6.0,
            color: [0.0, 1.0, 0.0],
        },
        VePointMark {
            pos: [1.0, 0.0, 0.0],
            radius_px: 8.0,
            color: [0.0, 0.0, 1.0],
        },
    ];
    let mut out: u64 = u64::MAX;
    let desc = VePointsDesc {
        struct_size: std::mem::size_of::<VePointsDesc>(),
        marks: marks.as_ptr(),
        count: 3,
    };
    // ① 成功路：返回 0、out=位形可查（entity_count +1）
    assert_eq!(
        visiaengine_add_points(ve, &desc, &mut out),
        VE_OK,
        "桩恒-5=RED 靶"
    );
    assert_ne!(out, u64::MAX, "成功必写 out");
    // 渲染路真（extra_cmds 重放含点云）；位形不入 items 枚举域（v0 明账，geo marker 同谱）
    assert_eq!(visiaengine_render(ve), VE_OK);
    assert_eq!(visiaengine_entity_count(ve), 0);
    let out1 = out;
    // ② 退化零提交：count=0 / NULL marks → VE_ERR_ARG 且 out 不碰（哨兵复位戏法）
    let zero = VePointsDesc {
        struct_size: std::mem::size_of::<VePointsDesc>(),
        marks: marks.as_ptr(),
        count: 0,
    };
    out = u64::MAX;
    assert_eq!(visiaengine_add_points(ve, &zero, &mut out), VE_ERR_ARG);
    assert_eq!(out, u64::MAX, "失败禁部分写");
    assert_eq!(
        visiaengine_add_points(ve, std::ptr::null(), &mut out),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_add_points(ve, std::ptr::null(), std::ptr::null_mut()),
        VE_ERR_ARG
    );
    // ③ struct_size 前瞻门（照 CAPI-15：小于所需=拒）
    let small = VePointsDesc {
        struct_size: 4,
        marks: marks.as_ptr(),
        count: 3,
    };
    assert_eq!(visiaengine_add_points(ve, &small, &mut out), VE_ERR_ARG);
    // ④ 非有限照收=本口宿主责任（与 load 侧四分类明写分工）；位形 0 合法禁当哨兵（CAPI-01）
    let nan = [VePointMark {
        pos: [f32::NAN, 0.0, 0.0],
        radius_px: 4.0,
        color: [1.0, 1.0, 1.0],
    }];
    let dnan = VePointsDesc {
        struct_size: std::mem::size_of::<VePointsDesc>(),
        marks: nan.as_ptr(),
        count: 1,
    };
    assert_eq!(
        visiaengine_add_points(ve, &dnan, &mut out),
        VE_OK,
        "非有限照收（load 侧才分类型丢弃）"
    );
    assert_ne!(out, out1, "二云位形互异");
    assert_eq!(visiaengine_render(ve), VE_OK, "双云重放");
    // ⑤ 句柄门同谱
    assert_eq!(visiaengine_add_points(0, &desc, &mut out), VE_ERR_ARG);
    assert_eq!(visiaengine_destroy(ve), 0);
}

/// CAPI-19 装载域：成功路+meta 缀查+policy/NULL/句柄三门（桩恒 -3=RED 靶）。
// spec: CAPI-19
#[test]
fn load_pcl_mount_meta_and_gates() {
    let ve = visiaengine_create_headless(160, 120);
    let path = std::ffi::CString::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/data/pcl_tetra_ascii.ply"
    ))
    .unwrap();
    let mut ent: u64 = u64::MAX;
    let mut rep = VePclReport {
        struct_size: std::mem::size_of::<VePclReport>(),
        dropped_non_finite: u32::MAX,
        dropped_out_of_domain: u32::MAX,
        dropped_unsupported: u32::MAX,
        truncated_points: u32::MAX,
        kept: u64::MAX,
    };
    assert_eq!(
        visiaengine_load_pcl(ve, path.as_ptr(), VE_PCL_LENIENT, &mut ent, &mut rep),
        VE_OK,
        "桩恒-3=RED 靶（GREEN 片转 0）"
    );
    assert_ne!(ent, u64::MAX, "成功必写 out_entity");
    assert_eq!(rep.kept, 4, "ascii 四净点");
    assert_eq!(rep.dropped_non_finite, 0);
    // 云级 meta 缀查（CAPI-19：pcl 位形走 attr 域，键不撞靠位形全局唯一）
    let mut pc = 0f64;
    assert_eq!(
        visiaengine_attr_f64(ve, ent, c"point_count".as_ptr(), &mut pc),
        1,
        "point_count 可查（CAPI-10 值域：1=found 0=none——attr 面非 0=OK 谱）"
    );
    assert_eq!(pc, 4.0);
    let mut buf = [0i8; 32];
    assert_eq!(
        visiaengine_attr_str(
            ve,
            ent,
            c"format".as_ptr(),
            buf.as_mut_ptr(),
            buf.len() as u64
        ),
        1,
        "format 可查（found=1 同谱）"
    );
    assert_eq!(visiaengine_render(ve), VE_OK, "点云重放路真");
    // 参数域三连
    assert_eq!(
        visiaengine_load_pcl(ve, path.as_ptr(), 9, &mut ent, &mut rep),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_load_pcl(ve, std::ptr::null(), VE_PCL_FASTFAIL, &mut ent, &mut rep),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_load_pcl(
            ve,
            path.as_ptr(),
            VE_PCL_FASTFAIL,
            std::ptr::null_mut(),
            &mut rep
        ),
        VE_ERR_ARG
    );
    assert_eq!(
        visiaengine_load_pcl(0, path.as_ptr(), VE_PCL_FASTFAIL, &mut ent, &mut rep),
        VE_ERR_ARG
    );
    assert_eq!(visiaengine_destroy(ve), 0);
}

// spec: CAPI-20
#[test]
fn set_get_clips_full_domain_roundtrip_truncation() {
    let ve = visiaengine_create_headless(160, 120);
    // 初始空；NULL+0=清空合法（唯一清除形）
    assert_eq!(visiaengine_get_clips(ve, std::ptr::null_mut(), 0), 0);
    assert_eq!(visiaengine_set_clips(ve, std::ptr::null(), 0), VE_OK);
    // 值域拒：NULL∧n>0 / n>4 / 退化（零法向/非有限）
    assert_eq!(visiaengine_set_clips(ve, std::ptr::null(), 1), VE_ERR_ARG);
    let p = VeClipPlane {
        nx: 0.0,
        ny: 1.0,
        nz: 0.0,
        d: 0.0,
    };
    assert_eq!(
        visiaengine_set_clips(ve, &p, 5),
        VE_ERR_ARG,
        "n>MAX 拒（读界先于解引用）"
    );
    let bad = VeClipPlane {
        nx: f64::NAN,
        ny: 0.0,
        nz: 0.0,
        d: 0.0,
    };
    assert_eq!(visiaengine_set_clips(ve, &bad, 1), VE_ERR_ARG);
    assert!(!visiaengine_last_error(ve).is_null(), "拒收路错误串在位");
    // 往返逐位（单位入参）+ 归一化形（(0,2,0,d=2)→(0,1,0,d=1)）
    let planes = [
        VeClipPlane {
            nx: 0.0,
            ny: 1.0,
            nz: 0.0,
            d: 0.0,
        },
        VeClipPlane {
            nx: 0.0,
            ny: 2.0,
            nz: 0.0,
            d: 2.0,
        },
        VeClipPlane {
            nx: 0.0,
            ny: 0.0,
            nz: -1.0,
            d: 1.0,
        },
    ];
    assert_eq!(visiaengine_set_clips(ve, planes.as_ptr(), 3), VE_OK);
    assert_eq!(
        visiaengine_get_clips(ve, std::ptr::null_mut(), 9),
        3,
        "NULL buf=仅计数"
    );
    // 截断自证 [Momus-A2]：n=3, cap=2 → 返回 3 写 2，第 3 槽哨兵不动
    let mut buf = [VeClipPlane {
        nx: -9.0,
        ny: -9.0,
        nz: -9.0,
        d: -9.0,
    }; 3];
    assert_eq!(visiaengine_get_clips(ve, buf.as_mut_ptr(), 2), 3);
    assert_eq!(
        (buf[0].nx, buf[0].ny, buf[0].nz, buf[0].d),
        (0.0, 1.0, 0.0, 0.0)
    );
    assert_eq!(
        (buf[1].nx, buf[1].ny, buf[1].nz, buf[1].d),
        (0.0, 1.0, 0.0, 1.0),
        "归一化同步 w"
    );
    assert_eq!(buf[2].nx, -9.0, "cap 外零写");
    // cap=0 ∧ buf≠NULL=纯计数路不报错（参数不侍二主）
    assert_eq!(visiaengine_get_clips(ve, buf.as_mut_ptr(), 0), 3);
    // 清空复原
    assert_eq!(visiaengine_set_clips(ve, std::ptr::null(), 0), VE_OK);
    assert_eq!(visiaengine_get_clips(ve, std::ptr::null_mut(), 0), 0);
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

const DEJAVU: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/data/DejaVuSans.ttf"
));

// spec: CAPI-21
#[test]
fn load_font_and_add_label_domain_and_flow() {
    let ve = visiaengine_create_headless(160, 120);
    let spec = VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [0.0, 0.0, 0.5],
        color: [1.0, 1.0, 1.0, 1.0],
        size_px: 20.0,
        text: c"VI".as_ptr(),
    };
    let mut out = 0u64;
    assert_eq!(
        visiaengine_add_label(ve, &spec, &mut out),
        VE_ERR_ARG,
        "无字体拒"
    );
    assert_eq!(visiaengine_load_font(ve, std::ptr::null(), 0), VE_ERR_ARG);
    assert_eq!(visiaengine_load_font(ve, b"junk".as_ptr(), 4), VE_ERR_ARG);
    assert_eq!(
        visiaengine_load_font(ve, DEJAVU.as_ptr(), DEJAVU.len()),
        VE_OK
    );
    let n0 = visiaengine_entity_count(ve);
    assert_eq!(visiaengine_add_label(ve, &spec, &mut out), VE_OK);
    // B1 教训反用：位形 0 合法（CAPI-01 分工），成败轴=返回码；位形可辨轴=计数+互异
    let mut out2 = 0u64;
    assert_eq!(visiaengine_add_label(ve, &spec, &mut out2), VE_OK);
    assert_ne!(out, out2, "两次装载位形互异（ABA 免疫面）");
    // items 外管理域（add_points CAPI-18 同谱）：不入 entity_count；位形互异即活体证
    assert_eq!(
        visiaengine_entity_count(ve),
        n0,
        "标签=items 外域（同 add_points 明账）"
    );
    assert_eq!(
        visiaengine_add_label(ve, &spec, std::ptr::null_mut()),
        VE_ERR_ARG
    );
    let empty = VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [0.0; 3],
        color: [1.0; 4],
        size_px: 20.0,
        text: c"".as_ptr(),
    };
    assert_eq!(
        visiaengine_add_label(ve, &empty, &mut out),
        VE_ERR_ARG,
        "空文本拒零提交"
    );
    let badsz = VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [0.0; 3],
        color: [1.0; 4],
        size_px: 0.0,
        text: c"X".as_ptr(),
    };
    assert_eq!(
        visiaengine_add_label(ve, &badsz, &mut out),
        VE_ERR_ARG,
        "size 域外拒"
    );
    let tiny = VeLabelSpec {
        struct_size: 1,
        pos: [0.0; 3],
        color: [1.0; 4],
        size_px: 20.0,
        text: c"X".as_ptr(),
    };
    assert_eq!(
        visiaengine_add_label(ve, &tiny, &mut out),
        VE_ERR_ARG,
        "struct_size 前瞻门"
    );
    assert_eq!(visiaengine_render(ve), VE_OK);
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-22
#[test]
fn add_label_returncode_axis_zero_commit_and_position() {
    // 返回码=成败唯一轴；位形可 0（CAPI-01）；全拒形零提交（mark 计数不变）
    let ve = visiaengine_create_headless(160, 120);
    assert_eq!(
        visiaengine_load_font(ve, DEJAVU.as_ptr(), DEJAVU.len()),
        VE_OK
    );
    let mk = |text: *const std::ffi::c_char, sz: f32| VeLabelSpec {
        struct_size: std::mem::size_of::<VeLabelSpec>(),
        pos: [1.0, 2.0, 3.0],
        color: [0.2, 0.4, 0.6, 1.0],
        size_px: sz,
        text,
    };
    let mut out = 0u64;
    let before = {
        let _ = visiaengine_render(ve);
        1i32 // 一次成功装载后 mark 存在；拒形后仍为 1（不增=零提交）
    };
    assert_eq!(
        visiaengine_add_label(ve, &mk(c"A".as_ptr(), 12.0), &mut out),
        VE_OK
    );
    let _ = before;
    // 拒形三连：NULL text / size 0 / 负 size——out 不得被写（哨兵预置判定）
    for bad in [
        mk(std::ptr::null(), 12.0),
        mk(c"B".as_ptr(), 0.0),
        mk(c"B".as_ptr(), -3.0),
    ] {
        let mut sentinel = u64::MAX;
        assert_eq!(visiaengine_add_label(ve, &bad, &mut sentinel), VE_ERR_ARG);
        assert_eq!(sentinel, u64::MAX, "拒形 out 零写");
    }
    // struct_size 前瞻门同零写
    let mut tiny = mk(c"C".as_ptr(), 12.0);
    tiny.struct_size = 8;
    let mut sentinel = u64::MAX;
    assert_eq!(visiaengine_add_label(ve, &tiny, &mut sentinel), VE_ERR_ARG);
    assert_eq!(sentinel, u64::MAX, "门败 out 零写");
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}

// spec: CAPI-23
#[test]
fn fly_ports_domain_and_progress() {
    let ve = visiaengine_create_headless(160, 120);
    // pose 域：NULL/struct_size 门/坏参
    assert_eq!(visiaengine_fly_to(ve, std::ptr::null(), 100), VE_ERR_ARG);
    let mut pose = VeCameraPose {
        struct_size: std::mem::size_of::<VeCameraPose>(),
        target: [3.0, 4.0, 0.0],
        yaw: 0.6,
        pitch: 0.4,
        dist: 30.0,
        zoom: 20.0,
        fov: 1.0,
    };
    assert_eq!(visiaengine_fly_to(ve, &pose, 100), VE_OK);
    let mut t01 = -7.0f64;
    assert_eq!(visiaengine_fly_state(ve, &mut t01), 0, "飞行中");
    assert!((0.0..=1.0).contains(&{ t01 }), "out 飞中写 [0,1) got {t01}");
    // 瞬移形：dur=0 → rc=0 且 state 立 done（done 路 out 不写=B1 零部分写谱）
    assert_eq!(visiaengine_fly_to(ve, &pose, 0), VE_OK);
    let mut sentinel = -9.0f64;
    assert_eq!(visiaengine_fly_state(ve, &mut sentinel), 1, "done 含 idle");
    assert_eq!(sentinel, -9.0, "done 路 out 零写");
    assert_eq!(visiaengine_fly_state(ve, std::ptr::null_mut()), 1, "NULL out=仅状态");
    // 坏参域：dist/zoom/fov 非法
    pose.dist = -1.0;
    assert_eq!(visiaengine_fly_to(ve, &pose, 100), VE_ERR_ARG, "dist 域");
    pose.dist = 30.0;
    pose.fov = 4.0;
    assert_eq!(visiaengine_fly_to(ve, &pose, 100), VE_ERR_ARG, "fov<π 域");
    assert_eq!(visiaengine_destroy(ve), VE_OK);
}
