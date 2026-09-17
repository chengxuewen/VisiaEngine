//! C ABI 面（CAPI-01/02/03/05）：句柄表、栅栏、线程亲和、错误串、输入口。
//! 风格 [FFI-R:FC-5]：`#[unsafe(no_mangle)] pub extern "C"` 安全签名，
//! 指针/参数校验在函数体内做（栅栏内），调用点无 unsafe 义务。

use std::os::raw::c_char;
use std::panic::AssertUnwindSafe;
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
/// 槽表访问平台抽象：native=Mutex（引擎类型 Send 于 native 成立）；
/// wasm=thread_local（wgpu web 后端 Rc 非 Send——浏览器单线程世界如实落地，
/// 批 7 J1 编译期实锤 [FFI-R:v13-J1-1]）。
mod slots {
    use super::Slot;

    #[cfg(not(target_arch = "wasm32"))]
    mod imp {
        use super::Slot;
        use std::sync::Mutex;
        static TABLE: Mutex<Vec<Slot>> = Mutex::new(Vec::new());
        pub fn with_table<R>(f: impl FnOnce(&mut Vec<Slot>) -> R) -> R {
            let mut g = TABLE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            f(&mut g)
        }
    }
    #[cfg(target_arch = "wasm32")]
    mod imp {
        use super::Slot;
        use std::cell::RefCell;
        thread_local! {
            static TABLE: RefCell<Vec<Slot>> = const { RefCell::new(Vec::new()) };
        }
        pub fn with_table<R>(f: impl FnOnce(&mut Vec<Slot>) -> R) -> R {
            TABLE.with(|c| f(&mut c.borrow_mut()))
        }
    }
    pub use imp::with_table;
}
use slots::with_table;

const fn enc(slot: u32, generation: u32) -> u64 {
    ((slot as u64) << 32) | (generation as u64)
}
#[allow(clippy::cast_possible_truncation)]
const fn dec(ve: u64) -> (u32, u32) {
    ((ve >> 32) as u32, ve as u32)
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
    let st = with_table(|t| match t.get(slot as usize) {
        Some(s) if s.generation == generation && s.engine.is_some() => {
            u8::from(s.owner == Some(std::thread::current().id()))
        }
        _ => 2u8,
    });
    match st {
        1 => Gate::Live(slot as usize),
        0 => {
            set_err(format!(
                "non-owner thread call on handle {ve:#x} (thread affinity: CAPI-03)"
            ));
            Gate::State
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

/// 栅栏单宏（CAPI-02）：全 17 入口 + 实现体统一经此，panic 不外溢、写诊断。
macro_rules! capi_guard {
    ($body:expr, $err_val:expr) => {{
        match std::panic::catch_unwind(AssertUnwindSafe(|| $body)) {
            Ok(v) => v,
            Err(p) => {
                // 诊断升级：取 panic payload（&str/String）入 last_error，未知兜底原句
                let msg = p
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_string())
                    .or_else(|| p.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "panic caught at FFI boundary".to_string());
                set_err(format!("panic at FFI boundary: {msg}"));
                $err_val
            }
        }
    }};
}

/// CAPI-17：事件推送口类型与 id 域（条款体=docs/sdd/capi.md CAPI-17）。
pub type VeEventCb = Option<unsafe extern "C" fn(*mut std::ffi::c_void, u32, u64, u64)>;
pub const VE_EVT_LOAD_PROGRESS: u32 = 1;
pub const VE_EVT_LOAD_ERROR: u32 = 2;

// ===== C ABI 入口面 =====

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_abi_version() -> u32 {
    (1 << 16) | 3 // major 1 · minor 3（CAPI-17 事件口=MAJOR 内追加；>>16==1 校验面不变）
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
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
            with_table(|t| {
                if t.is_empty() {
                    t.push(Slot {
                        generation: 0,
                        engine: None,
                        owner: None,
                    }); // slot 0 永久占位
                }
                let owner = Some(std::thread::current().id());
                let idx = match t
                    .iter()
                    .position(|s| s.engine.is_none() && s.generation > 0)
                {
                    Some(i) => {
                        t[i].generation += 1;
                        t[i].engine = Some(engine);
                        t[i].owner = owner;
                        i
                    }
                    None => {
                        t.push(Slot {
                            generation: 1,
                            engine: Some(engine),
                            owner,
                        });
                        t.len() - 1
                    }
                };
                enc(idx as u32, t[idx].generation)
            })
        },
        0u64
    )
}

/// released/foreign/stale 再入一律 -1（双销毁不吞没）；非 owner=-2。
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_destroy(ve: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    with_table(|t| t[i].engine = None); // 槽回收（gen 保持，再入 -1）
                    0
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_attach(ve: u64, win_ptr: u64, display_ptr: u64, kind: i32) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if !(0..=2).contains(&kind) || win_ptr == 0 {
                        set_err(format!(
                            "attach kind {kind}/win {win_ptr:#x} rejected (0=x11 1=win32 2=cocoa)"
                        ));
                        return VE_ERR_ARG;
                    }
                    if kind == 0 && display_ptr == 0 {
                        set_err(
                            "attach x11 requires display_ptr (Xlib 无 display 不可构面)"
                                .to_string(),
                        );
                        return VE_ERR_ARG;
                    }
                    use raw_window_handle::{
                        AppKitWindowHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle,
                        XlibDisplayHandle, XlibWindowHandle,
                    };
                    // NonNull::new 的 Option 即非零担保（原 filter(is_null) 属
                    // useless_ptr_null_checks lint 域，删）
                    let nn = |p: u64| std::ptr::NonNull::new(p as *mut std::ffi::c_void);
                    // ? 禁入（栅栏宏闭包返回 i32）——Option combinator 式构造
                    let built: Option<(Option<RawDisplayHandle>, RawWindowHandle)> = match kind {
                        0 => nn(display_ptr).map(|d| {
                            (
                                Some(RawDisplayHandle::Xlib(XlibDisplayHandle::new(Some(d), 0))),
                                // rwh new(c_ulong)——target 变宽（x86_64=64/wasm32=32），
                                // as _ 平台自推；XID 值域 32 位无损
                                RawWindowHandle::Xlib(XlibWindowHandle::new(win_ptr as _)),
                            )
                        }),
                        1 => std::num::NonZeroIsize::new(win_ptr as isize)
                            .map(|w| (None, RawWindowHandle::Win32(Win32WindowHandle::new(w)))),
                        2 => nn(win_ptr)
                            .map(|v| (None, RawWindowHandle::AppKit(AppKitWindowHandle::new(v)))),
                        _ => None,
                    };
                    let Some((dh, wh)) = built else {
                        set_err("attach: null/unsupported handle slot".to_string());
                        return VE_ERR_ARG;
                    };
                    let r = with_engine(i, |e| unsafe { e.attach_raw(dh, wh) });
                    match r {
                        Ok(()) => VE_OK,
                        Err(msg) => {
                            set_err(format!("attach: {msg}"));
                            VE_ERR_STATE
                        }
                    }
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
    with_table(|t| match t[idx].engine.as_mut() {
        Some(e) => f(e),
        None => Err("engine released mid-call".to_string()),
    })
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
                    // CAPI-17：失败出口事件（a=返回码 signed 形；b=0）
                    let _ = with_engine(i, |e| {
                        e.emit(VE_EVT_LOAD_ERROR, VE_ERR_IO as u64, 0);
                        Ok::<(), String>(())
                    });
                    VE_ERR_IO
                }
            }
        }
        Gate::State => VE_ERR_STATE,
        Gate::Arg => VE_ERR_ARG,
    }
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_load_gltf(ve: u64, path: *const c_char) -> i32 {
    capi_guard!(load_impl(ve, path, "load_gltf"), VE_ERR_PANIC)
}
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_load_geojson(ve: u64, path: *const c_char) -> i32 {
    capi_guard!(load_impl(ve, path, "load_geojson"), VE_ERR_PANIC)
}

/// 唯一输入口（CAPI-05）：struct_size 校验 + kind 口径；1=消费 0=非本引擎事件。
// [FFI-R:FC-5] 风格裁决=安全签名（调用点无 unsafe 义务），指针解引用在栅栏内校验；
// clippy 该 lint 针对"公开 safe fn 解引用"的一般场景，FFI 边界是既定例外（allow 在此=记录裁决而非绕lint）。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
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
                    with_table(|t| {
                        match t[i]
                            .engine
                            .as_mut()
                            .map(|e| e.apply_input(inp.kind, inp.px, inp.py, inp.wheel))
                        {
                            Some(Some(consumed)) => i32::from(consumed),
                            Some(None) | None => 0, // 未登记 kind / 竞态消失：不消费
                        }
                    })
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
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

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_last_error(_ve: u64) -> *const c_char {
    // 栅栏对指针返回意义有限（读 TLS 不 panic 面），仍统一裹持
    capi_guard!(
        ERRPTR.with(std::cell::Cell::get),
        std::ptr::addr_of!(EMPTY).cast()
    )
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_entity_count(ve: u64) -> i32 {
    capi_guard!(
        match gate(ve) {
            Gate::Live(i) => {
                with_table(|t| t[i].engine.as_ref().map_or(0, Engine::entity_count)) as i32
            }
            Gate::State => VE_ERR_STATE,
            Gate::Arg => VE_ERR_ARG,
        },
        VE_ERR_PANIC
    )
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
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

/// 实体句柄编码（CAPI-08）：slot+gen 直译，无引擎侧基 1 偏置（**slot0gen0=0 合法**，
/// 与引擎侧 0 哨兵的不对称是 CAPI-01 有意分工；宿主禁按 0 判实体句柄有效性，
/// 有效性=枚举域/pick 出口谱）；miss=UINT64_MAX。
pub fn enc_entity(id: visiaengine_core::EntityId) -> u64 {
    enc(id.slot(), id.generation())
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
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

/// CAPI-10 共用入参解析：null key/非 UTF-8 → None（调用方返 -1；错误串已写）。
fn parse_attr_key<'a>(key: *const c_char) -> Option<&'a str> {
    if key.is_null() {
        set_err("attr: null key".to_string());
        return None;
    }
    match unsafe { std::ffi::CStr::from_ptr(key) }.to_str() {
        Ok(s) => Some(s),
        Err(_) => {
            set_err("attr: key not utf-8".to_string());
            None
        }
    }
}

/// CAPI-10：f64 属性读。1=命中（out 写）/ 0=缺失（out 不动，缺失≠零值）/ <0=错误。
/// entity=宿主自 pick/entity_at 所得位形（键含代际，旧代句柄永不撞新代行）。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_attr_f64(
    ve: u64,
    entity: u64,
    key: *const c_char,
    out: *mut f64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    let Some(k) = parse_attr_key(key) else {
                        return VE_ERR_ARG;
                    };
                    if out.is_null() {
                        set_err("attr_f64: null out".to_string());
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| Ok(e.attr_f64(entity, k))) {
                        Ok(Some(v)) => {
                            unsafe { *out = v };
                            1
                        }
                        Ok(None) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_STATE
                        }
                    }
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-11：str 属性读。NUL 终止写 buf[cap]；1=命中 / 0=缺失 / -5=cap 不足（零部分写，无探长子模式）/ -1=空指针。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_attr_str(
    ve: u64,
    entity: u64,
    key: *const c_char,
    buf: *mut c_char,
    cap: u64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    let Some(k) = parse_attr_key(key) else {
                        return VE_ERR_ARG;
                    };
                    if buf.is_null() {
                        set_err("attr_str: null buf".to_string());
                        return VE_ERR_ARG;
                    }
                    let got = with_engine(i, |e| Ok(e.attr_str(entity, k).map(str::to_owned)));
                    match got {
                        Ok(None) => 0,
                        Ok(Some(s)) => {
                            let need = s.len() + 1; // NUL
                            if cap < need as u64 {
                                set_err(format!("attr_str: cap {cap} < {need}"));
                                return VE_ERR_SIZE;
                            }
                            let dst =
                                unsafe { std::slice::from_raw_parts_mut(buf as *mut u8, need) };
                            dst[..s.len()].copy_from_slice(s.as_bytes());
                            dst[s.len()] = 0;
                            1
                        }
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_STATE
                        }
                    }
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-10：bool 属性读（out=0/1）；口径同 attr_f64。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_attr_bool(
    ve: u64,
    entity: u64,
    key: *const c_char,
    out: *mut i32,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    let Some(k) = parse_attr_key(key) else {
                        return VE_ERR_ARG;
                    };
                    if out.is_null() {
                        set_err("attr_bool: null out".to_string());
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| Ok(e.attr_bool(entity, k))) {
                        Ok(Some(v)) => {
                            unsafe { *out = i32::from(v) };
                            1
                        }
                        Ok(None) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_STATE
                        }
                    }
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
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

// ── B1 数据带（CAPI-13..16）：显隐/查询/程序化增删；gate/with_engine/capi_guard 单源形 ═─

/// CAPI-15 值结构（struct_size 前瞻门=VeInput 同谱；指针仅调用期读取，返回后宿主即可释放）。
#[repr(C)]
pub struct VeMeshDesc {
    pub struct_size: usize,
    pub positions: *const f32, // [x,y,z] × n_positions
    pub normals: *const f32,   // 可 NULL=引擎合成 +Z
    pub indices: *const u32,
    pub n_positions: u64,
    pub n_indices: u64,
    pub base_color: *const f32, // 4 元组
    pub origin: *const f64,     // 3 元组
}

/// CAPI-13：owner 线程；严格 visible∈{0,1}；未知位形/越值=-1（VE_ERR_ARG），幂等重复=0。
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_entity_set_visible(ve: u64, entity: u64, visible: i32) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if !matches!(visible, 0 | 1) {
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| e.set_visible(entity, visible == 1)) {
                        Ok(()) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-14：owner 线程（last_error 例外集不在此列——纯读表）；1/0/未知=-1。
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_entity_visible(ve: u64, entity: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => match with_engine(i, |e| Ok(e.is_visible(entity))) {
                    Ok(Some(true)) => 1,
                    Ok(Some(false)) => 0,
                    Ok(None) => VE_ERR_ARG,
                    Err(msg) => {
                        set_err(msg);
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

/// CAPI-15：owner 线程；0=成功写 *out_entity（位形可=0，CAPI-01 谱），
/// 退化/NULL/struct_size 门/坏 out=-1 且 out 零部分写（attr_str 纪律同谱）。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_add_mesh(
    ve: u64,
    desc: *const VeMeshDesc,
    out_entity: *mut u64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if desc.is_null() {
                        set_err("add_mesh: null desc".to_string());
                        return VE_ERR_ARG;
                    }
                    if out_entity.is_null() {
                        set_err("add_mesh: null out_entity".to_string());
                        return VE_ERR_ARG;
                    }
                    let d = unsafe { &*desc };
                    if d.struct_size < std::mem::size_of::<VeMeshDesc>() {
                        set_err(format!(
                            "add_mesh: struct_size {} < {}",
                            d.struct_size,
                            std::mem::size_of::<VeMeshDesc>()
                        ));
                        return VE_ERR_ARG;
                    }
                    if d.positions.is_null()
                        || d.indices.is_null()
                        || d.base_color.is_null()
                        || d.origin.is_null()
                    {
                        set_err("add_mesh: required field pointer null".to_string());
                        return VE_ERR_ARG;
                    }
                    let (np, ni) = (d.n_positions as usize, d.n_indices as usize);
                    let positions =
                        unsafe { std::slice::from_raw_parts(d.positions as *const [f32; 3], np) };
                    let indices = unsafe { std::slice::from_raw_parts(d.indices, ni) };
                    let normals = if d.normals.is_null() {
                        None
                    } else {
                        Some(unsafe {
                            std::slice::from_raw_parts(d.normals as *const [f32; 3], np)
                        })
                    };
                    let base_color = unsafe { *(d.base_color as *const [f32; 4]) };
                    let origin = unsafe { *(d.origin as *const [f64; 3]) };
                    match with_engine(i, move |e| {
                        e.add_mesh(positions, normals, indices, base_color, origin)
                    }) {
                        Ok(h) => {
                            unsafe { *out_entity = h };
                            0
                        }
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-16：owner 线程；成功=0；未知位形/旧句柄再入=-1（双销毁同谱）。
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_remove_entity(ve: u64, entity: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => match with_engine(i, |e| e.remove_entity(entity)) {
                    Ok(()) => 0,
                    Err(msg) => {
                        set_err(msg);
                        VE_ERR_ARG
                    }
                },
                Gate::State => VE_ERR_STATE,
                Gate::Arg => VE_ERR_ARG,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-17：注册/替换/摘除（NULL）事件回调。native 独占（wasm32 无 C ABI 面，
/// JS 闭包走 visiaengine-wasm 桥 set_event_fn；set_event_anchor 亦 native cfg）。
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[cfg(not(target_arch = "wasm32"))]
pub extern "C" fn visiaengine_set_event_callback(
    ve: u64,
    cb: VeEventCb,
    user: *mut std::ffi::c_void,
) -> i32 {
    let anchor = cb.map_or(0, |f| f as *const () as usize);
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => match with_engine(i, |e| {
                    e.set_event_anchor(anchor, user as usize);
                    Ok::<(), String>(())
                }) {
                    Ok(()) => 0,
                    Err(msg) => {
                        set_err(msg);
                        VE_ERR_STATE
                    }
                },
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}
