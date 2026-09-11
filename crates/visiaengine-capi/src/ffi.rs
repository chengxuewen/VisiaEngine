//! C ABI 面（CAPI-01/02/03/05）：句柄表、栅栏、线程亲和、错误串、输入口。
//! 风格 [FFI-R:FC-5]：`#[unsafe(no_mangle)] pub extern "C"` 安全签名，
//! 指针/参数校验在函数体内做（栅栏内），调用点无 unsafe 义务。

use std::os::raw::c_char;
use std::panic::AssertUnwindSafe;
use std::sync::Mutex;
use std::thread::ThreadId;

use crate::engine::Engine;

pub const VE_OK: i32 = 0;

// ===== 返回码与 kind（Rust 单源，C 头/TS 皆镜像 [FFI-R:CS-M1/BS 纪律]）=====
pub const VE_ERR_ARG: i32 = -1;
pub const VE_ERR_STATE: i32 = -2;
pub const VE_ERR_IO: i32 = -3;
pub const VE_ERR_PANIC: i32 = -4;
pub const VE_ERR_SIZE: i32 = -5;
pub const KIND_PTR_MOVE: u32 = 1;
pub const KIND_PTR_DOWN: u32 = 2;
pub const KIND_PTR_UP: u32 = 3;
pub const KIND_WHEEL: u32 = 4;
pub const KIND_KEY: u32 = 5;
/// 非法 kind 探测值（>5 未登记=非本引擎事件，不消费）
pub const KIND_NO_SUCH: u32 = u32::MAX;
pub const MISS: u64 = u64::MAX;

/// 唯一过界结构体（CAPI-05/BS-1）：struct_size 首字段=self-describing 演进锚。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VeInput {
    pub struct_size: usize,
    pub kind: u32,
    pub px: f32,
    pub py: f32,
    pub wheel: f32,
    pub button: u32,
    pub mods: u32,
}

// ===== 句柄表（CAPI-01：slot 基 1，世代回收前进）=====
struct Slot {
    generation: u32,
    engine: Option<Engine>,
    owner: Option<ThreadId>,
}
static TABLE: Mutex<Vec<Slot>> = Mutex::new(Vec::new());

const fn enc(slot: u32, generation: u32) -> u64 {
    ((slot as u64) << 32) | (generation as u64)
}
#[allow(clippy::cast_possible_truncation)]
const fn dec(ve: u64) -> (u32, u32) {
    ((ve >> 32) as u32, ve as u32)
}

fn table_lock() -> std::sync::MutexGuard<'static, Vec<Slot>> {
    TABLE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

enum Gate {
    Live(usize), // 表索引
    Arg,
    State,
}

/// 句柄解码 + 世代 + owner 校验（错误串按 CAPI-03 写调用线程 TLS）。
fn gate(ve: u64) -> Gate {
    let (slot, generation) = dec(ve);
    if slot == 0 {
        return Gate::Arg; // slot 0 永不合法（含 ve==0）
    }
    let t = table_lock();
    match t.get(slot as usize) {
        Some(s) if s.generation == generation && s.engine.is_some() => {
            if s.owner == Some(std::thread::current().id()) {
                Gate::Live(slot as usize)
            } else {
                drop(t);
                set_err(format!(
                    "non-owner thread call on handle {ve:#x} (thread affinity: CAPI-03)"
                ));
                Gate::State
            }
        }
        _ => Gate::Arg,
    }
}

// ===== 错误串（线程绑定 TLS；仅 <0 分支写 [CAPI-03 写口径]）=====
static EMPTY: [u8; 1] = [0];
thread_local! {
    static ERRPTR: std::cell::Cell<*const c_char> = const { std::cell::Cell::new(EMPTY.as_ptr().cast()) };
}
fn set_err(msg: String) {
    let fresh = std::ffi::CString::new(msg)
        .map(std::ffi::CString::into_raw)
        .unwrap_or(std::ptr::null_mut());
    ERRPTR.with(|p| {
        let old = p.replace(fresh.cast_const());
        if !old.is_null() && !std::ptr::eq(old, EMPTY.as_ptr().cast()) {
            // 契约：上次返回串至本次写前有效——此处释放的正是"已失效"的旧串
            unsafe { drop(std::ffi::CString::from_raw(old.cast_mut())) };
        }
    });
}

/// 栅栏单宏（CAPI-02）：全 14 入口 + 实现体统一经此，panic 不外溢、写诊断。
macro_rules! capi_guard {
    ($body:expr, $err_val:expr) => {{
        match std::panic::catch_unwind(AssertUnwindSafe(|| $body)) {
            Ok(v) => v,
            Err(_) => {
                set_err("panic caught at FFI boundary".to_string());
                $err_val
            }
        }
    }};
}

// ===== 14 入口 =====

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_abi_version() -> u32 {
    1 << 16 // v0 = major 1（demo/宿主校验 >>16==1）
}

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_create_headless(w: u32, h: u32) -> u64 {
    capi_guard!(
        {
            if w == 0 || h == 0 {
                set_err(format!("create size {w}x{h} invalid (use >0; -5 family)"));
                return 0; // u64 面失败仅 0（诊断走 last_error(NULL 语义：无句柄读 TLS)
            }
            let Some(engine) = Engine::new_headless(w, h) else {
                set_err("create: no adapter/device available".to_string());
                return 0;
            };
            let mut t = table_lock();
            if t.is_empty() {
                t.push(Slot {
                    generation: 0,
                    engine: None,
                    owner: None,
                }); // slot 0 永久占位
            }
            let idx = match t
                .iter()
                .position(|s| s.engine.is_none() && s.generation > 0)
            {
                Some(i) => {
                    t[i].generation += 1;
                    t[i].engine = Some(engine);
                    t[i].owner = Some(std::thread::current().id());
                    i
                }
                None => {
                    t.push(Slot {
                        generation: 1,
                        engine: Some(engine),
                        owner: Some(std::thread::current().id()),
                    });
                    t.len() - 1
                }
            };
            enc(idx as u32, t[idx].generation)
        },
        0u64
    )
}

/// released/foreign/stale 再入一律 -1（双销毁不吞没）；非 owner=-2。
#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_destroy(ve: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    let mut t = table_lock();
                    t[i].engine = None; // 槽回收（generation 保持，再入 -1；复用才 +1）
                    0
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_attach(ve: u64, win_ptr: u64, display_ptr: u64, kind: i32) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(_) => {
                    if !(0..=2).contains(&kind) || win_ptr == 0 {
                        set_err(format!(
                            "attach kind {kind}/win {win_ptr:#x} rejected (0=x11 1=win32 hinstance? 2=cocoa)"
                        ));
                        return VE_ERR_ARG;
                    }
                    let _ = display_ptr;
                    set_err("surface wiring lands in I3".to_string());
                    VE_ERR_STATE
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

/// 锁表执行（Live 索引来自 gate；引擎中途消失=Err 传播给调用入口的哨兵分支）。
fn with_engine<T>(
    idx: usize,
    f: impl FnOnce(&mut Engine) -> Result<T, String>,
) -> Result<T, String> {
    let mut t = table_lock();
    match t[idx].engine.as_mut() {
        Some(e) => f(e),
        None => Err("engine released mid-call".to_string()),
    }
}

/// 共用装载体（CAPI-04/05）：path null=-1；IO/解析失败=-3；成功=0（尺寸违规在 -5 族）。
fn load_impl(ve: u64, path: *const c_char, label: &str) -> i32 {
    match gate(ve) {
        Gate::Live(i) => {
            if path.is_null() {
                set_err(format!("{label}: null path"));
                return VE_ERR_ARG;
            }
            let cstr = unsafe { std::ffi::CStr::from_ptr(path) };
            let p = cstr.to_string_lossy().into_owned();
            let r = with_engine(i, |e| {
                if label == "load_gltf" {
                    e.load_gltf(&p)
                } else {
                    e.load_geojson(&p)
                }
            });
            match r {
                Ok(_) => VE_OK,
                Err(msg) => {
                    set_err(format!("{label} '{p}': {msg}"));
                    VE_ERR_IO
                }
            }
        }
        Gate::State => VE_ERR_STATE,
        Gate::Arg => VE_ERR_ARG,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_load_gltf(ve: u64, path: *const c_char) -> i32 {
    capi_guard!(load_impl(ve, path, "load_gltf"), VE_ERR_PANIC)
}
#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_load_geojson(ve: u64, path: *const c_char) -> i32 {
    capi_guard!(load_impl(ve, path, "load_geojson"), VE_ERR_PANIC)
}

/// 唯一输入口（CAPI-05）：struct_size 校验 + kind 口径；1=消费 0=非本引擎事件。
// [FFI-R:FC-5] 风格裁决=安全签名（调用点无 unsafe 义务），指针解引用在栅栏内校验；
// clippy 该 lint 针对"公开 safe fn 解引用"的一般场景，FFI 边界是既定例外（allow 在此=记录裁决而非绕lint）。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_on_input(ve: u64, input: *const VeInput) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if input.is_null() {
                        set_err("on_input: null VeInput".to_string());
                        return VE_ERR_ARG;
                    }
                    let inp = unsafe { *input };
                    if inp.struct_size < std::mem::size_of::<VeInput>() {
                        set_err(format!(
                            "on_input: struct_size {} < {}",
                            inp.struct_size,
                            std::mem::size_of::<VeInput>()
                        ));
                        return VE_ERR_ARG;
                    }
                    let mut t = table_lock();
                    match t[i]
                        .engine
                        .as_mut()
                        .map(|e| e.apply_input(inp.kind, inp.px, inp.py, inp.wheel))
                    {
                        Some(Some(consumed)) => i32::from(consumed),
                        Some(None) | None => 0, // 未登记 kind / 竞态消失：不消费
                    }
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_render(ve: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => match with_engine(i, |e| e.render()) {
                    Ok(()) => VE_OK,
                    Err(msg) => {
                        set_err(format!("render: {msg}"));
                        VE_ERR_STATE
                    }
                },
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

/// [FFI-R:FC-5] 同 on_input：安全签名体内校验（allow=FFI 边界裁决记录）。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_readback(ve: u64, buf: *mut u8, len: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    // 无帧可读是硬前置（先于缓冲量值检查，CAPI-04）
                    let need = match with_engine(i, |e| {
                        e.frame_bytes()
                            .ok_or_else(|| "no frame; call render".to_string())
                    }) {
                        Ok(n) => n,
                        Err(msg) => {
                            set_err(format!("readback: {msg}"));
                            return VE_ERR_STATE;
                        }
                    };
                    if buf.is_null() || (len as usize) < need {
                        set_err(format!("readback buffer {len} < need {need}"));
                        return VE_ERR_SIZE;
                    }
                    let dest = unsafe { std::slice::from_raw_parts_mut(buf, need) };
                    let _ = with_engine(i, |e| {
                        e.copy_frame(dest);
                        Ok(0)
                    });
                    VE_OK
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_viewport(ve: u64, w: u32, h: u32) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if w == 0 || h == 0 {
                        set_err(format!("viewport {w}x{h}: zero dimension (-5 family)"));
                        return VE_ERR_SIZE;
                    }
                    let _ = with_engine(i, |e| {
                        e.resize(w, h);
                        Ok(0)
                    });
                    VE_OK
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

/// 例外集成员：读调用线程 TLS（无句柄校验）；无错=空串静态。
#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_last_error(_ve: u64) -> *const c_char {
    // 栅栏对指针返回意义有限（读 TLS 不 panic 面），仍统一裹持
    capi_guard!(
        ERRPTR.with(std::cell::Cell::get),
        std::ptr::addr_of!(EMPTY).cast()
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_entity_count(ve: u64) -> i32 {
    capi_guard!(
        match gate(ve) {
            Gate::Live(i) => table_lock()[i]
                .engine
                .as_ref()
                .map_or(0, Engine::entity_count) as i32,
            Gate::State => VE_ERR_STATE,
            Gate::Arg => VE_ERR_ARG,
        },
        VE_ERR_PANIC
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_pick(ve: u64, px: f32, py: f32) -> u64 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    with_engine(i, |e| Ok(e.pick(px, py).map_or(MISS, enc_entity))).unwrap_or(MISS)
                }
                _ => MISS, // u64 面无错误码族：一切异常=未命中哨兵（CAPI-01/08）
            }
        },
        MISS
    )
}

/// 实体句柄编码（CAPI-08）：slot+gen 直译，无引擎侧基 1 偏置；miss=UINT64_MAX。
fn enc_entity(id: visiaengine_core::EntityId) -> u64 {
    enc(id.slot(), id.generation())
}

#[unsafe(no_mangle)]
pub extern "C" fn visiaengine_entity_at(ve: u64, index: u32) -> u64 {
    capi_guard!(
        match gate(ve) {
            Gate::Live(i) =>
                with_engine(i, |e| { Ok(e.entity_at(index).map_or(MISS, enc_entity)) })
                    .unwrap_or(MISS),
            _ => MISS,
        },
        MISS
    )
}

/// CAPI-02 行为探针（Rust 侧集成测试用；非 extern，不进 ABI 面/nm 白名单）。
pub mod test_util {
    use super::*;
    #[must_use]
    pub fn guard_panic_probe() -> i32 {
        capi_guard!(panic!("injected for CAPI-02"), VE_ERR_PANIC)
    }
}
