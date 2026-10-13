//! C ABI face (CAPI-01/02/03/05): handle table, fence, thread affinity, error strings, input entry.
//! Style [FFI-R:FC-5]: `#[unsafe(no_mangle)] pub extern "C"` safe signatures,
//! pointer/argument validation done inside the function body (within the fence); call sites carry no unsafe duty.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic::AssertUnwindSafe;
use std::thread::ThreadId;

use crate::engine::Engine;

pub const VE_OK: i32 = 0;

// ===== Return codes and kind (Rust single source, the C header/TS are both mirrors [FFI-R:CS-M1/BS discipline]) =====
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
/// Illegal kind probe value (>5 unregistered = not this engine's event, not consumed)
pub const KIND_NO_SUCH: u32 = u32::MAX;
pub const MISS: u64 = u64::MAX;

/// The single crossing-structure (CAPI-05/BS-1): struct_size as first field = self-describing evolution anchor.
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

// ===== Handle table (CAPI-01: slot base 1, generation advances on recycle) =====
struct Slot {
    generation: u32,
    engine: Option<Engine>,
    owner: Option<ThreadId>,
}
/// Slot-table access platform abstraction: native = Mutex (engine types being Send holds on native);
/// wasm = thread_local (the wgpu web backend's Rc is non-Send -- the browser single-thread world landed as-is,
/// batch 7 J1 compile-time proof [FFI-R:v13-J1-1]).
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
    Live(usize), // table index
    Arg,
    State,
}

/// Handle decode + generation + owner check (error string written per CAPI-03 to the calling thread's TLS).
fn gate(ve: u64) -> Gate {
    let (slot, generation) = dec(ve);
    if slot == 0 {
        return Gate::Arg; // slot 0 is never legal (ve==0 included)
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

// ===== Error strings (thread-bound TLS; written on <0 branches only [CAPI-03 write standard]) =====
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
            // contract: the previously returned string is valid until this write -- what is freed here is precisely the already-voided old string
            unsafe { drop(std::ffi::CString::from_raw(old.cast_mut())) };
        }
    });
}

/// Single fence macro (CAPI-02): all 17 entries + the implementation bodies route through it; no panic escapes, diagnostics written.
macro_rules! capi_guard {
    ($body:expr, $err_val:expr) => {{
        match std::panic::catch_unwind(AssertUnwindSafe(|| $body)) {
            Ok(v) => v,
            Err(p) => {
                // diagnostic upgrade: take the panic payload (&str/String) into last_error, unknown falls back to the original sentence
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

/// CAPI-17: event-push entry types and id domain (clause body = docs/sdd/capi.md CAPI-17).
pub type VeEventCb = Option<unsafe extern "C" fn(*mut std::ffi::c_void, u32, u64, u64)>;
pub const VE_EVT_LOAD_PROGRESS: u32 = 1;
pub const VE_EVT_LOAD_ERROR: u32 = 2;

// ===== C ABI entry face =====

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_abi_version() -> u32 {
    (1 << 16) | 16 // major 1 minor 16 (V2.2: +CAPI-42/43 node transforms; >>16==1 unchanged)
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_create_headless(w: u32, h: u32) -> u64 {
    capi_guard!(
        {
            if w == 0 || h == 0 {
                set_err(format!("create size {w}x{h} invalid (use >0; -5 family)"));
                return 0; // the u64 face fails with 0 only (diagnostics via last_error, NULL semantics: no handle reads TLS)
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
                    }); // slot 0 permanently reserved
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

/// released/foreign/stale re-entry is always -1 (double destroy not swallowed); non-owner = -2.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_destroy(ve: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    with_table(|t| t[i].engine = None); // slot recycle (gen kept, re-entry -1)
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
                            "attach x11 requires display_ptr (Xlib cannot build a surface without a display)"
                                .to_string(),
                        );
                        return VE_ERR_ARG;
                    }
                    use raw_window_handle::{
                        AppKitWindowHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle,
                        XlibDisplayHandle, XlibWindowHandle,
                    };
                    // NonNull::new's Option is the non-zero guarantee (the former filter(is_null) belonged to the
                    // useless_ptr_null_checks lint domain, removed)
                    let nn = |p: u64| std::ptr::NonNull::new(p as *mut std::ffi::c_void);
                    // ? forbidden (the fence macro's closure returns i32) -- construct via Option combinators
                    let built: Option<(Option<RawDisplayHandle>, RawWindowHandle)> = match kind {
                        0 => nn(display_ptr).map(|d| {
                            (
                                Some(RawDisplayHandle::Xlib(XlibDisplayHandle::new(Some(d), 0))),
                                // rwh new(c_ulong) -- target widens (x86_64=64/wasm32=32),
                                // as _ infers per platform; XID value range is lossless at 32 bits
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

/// Locked table execution (the Live index comes from the gate; the engine vanishing mid-way = Err propagated to the calling entry's sentinel branch).
fn with_engine<T>(
    idx: usize,
    f: impl FnOnce(&mut Engine) -> Result<T, String>,
) -> Result<T, String> {
    with_table(|t| match t[idx].engine.as_mut() {
        Some(e) => f(e),
        None => Err("engine released mid-call".to_string()),
    })
}

/// Shared load body (CAPI-04/05): null path = -1; IO/parse failure = -3; success = 0 (size violations live in the -5 family).
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
                    // CAPI-17: failure-exit event (a = return code in signed form; b = 0)
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

/// The sole input entry (CAPI-05): struct_size validation + kind convention; 1 = consumed, 0 = not this engine's event.
// [FFI-R:FC-5] style ruling = safe signature (call sites carry no unsafe duty), pointer dereference validated inside the fence;
// this clippy lint targets the general scenario of "public safe fn dereferences", and the FFI boundary is a settled exception (the allow here = recording the ruling, not evading the lint).
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
                            Some(None) | None => 0, // unregistered kind / vanished in a race: not consumed
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

/// [FFI-R:FC-5] same as on_input: safe signature with in-body validation (allow = FFI boundary ruling record).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_readback(ve: u64, buf: *mut u8, len: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    // no frame to read is a hard precondition (before the buffer magnitude check, CAPI-04)
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

/// Exception-set member: reads the calling thread's TLS (no handle validation); no error = empty static string.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_last_error(_ve: u64) -> *const c_char {
    // the fence's meaning for pointer returns is limited (reading TLS has no panic surface), still wrapped uniformly
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
                _ => MISS, // the u64 face has no error-code family: every anomaly = the miss sentinel (CAPI-01/08)
            }
        },
        MISS
    )
}

/// Entity handle encoding (CAPI-08): slot+gen literal, no engine-side base-1 bias (**slot0gen0 = 0 is legal**,
/// the asymmetry with the engine-side 0 sentinel is a deliberate CAPI-01 division; the host must not judge entity-handle validity by 0,
/// validity = the enumeration domain / pick output scheme); miss = UINT64_MAX.
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

/// CAPI-10 shared argument parsing: null key / non-UTF-8 -> None (caller returns -1; the error string is already written).
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

/// CAPI-10: f64 attribute read. 1 = hit (out written) / 0 = absent (out untouched, absent != zero value) / <0 = error.
/// entity = the shape the host obtained from pick/entity_at (the key carries the generation, an old-generation handle never collides into a new generation).
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

/// CAPI-11: str attribute read. Writes NUL-terminated into buf[cap]; 1 = hit / 0 = absent / -5 = cap insufficient (zero partial write, no length-probe sub-mode) / -1 = null pointer.
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

/// CAPI-10: bool attribute read (out = 0/1); same convention as attr_f64.
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

/// CAPI-36 (N2.4): typed property diff-update — f64 column.
/// Typed-reject: writing a different type to an existing column (override or
/// source) returns VE_ERR_ARG with a type-conflict message; a missing column
/// is created (first write types it). Unknown entity = VE_ERR_ARG.
/// Zero render-face effect (attributes are host/query data).
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // NULL-guarded; attr-face precedent
pub extern "C" fn visiaengine_update_entity_attr_f64(
    ve: u64,
    entity: u64,
    name: *const c_char,
    v: f64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if name.is_null() {
                        return VE_ERR_ARG;
                    }
                    let name = match unsafe { CStr::from_ptr(name) }.to_str() {
                        Ok(n) => n,
                        Err(_) => return VE_ERR_ARG,
                    };
                    match with_engine(i, |e| e.update_entity_props_f64(entity, name, v)) {
                        Ok(()) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-37 (N2.4): typed property diff-update — str column (UTF-8 required).
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // NULL-guarded (load_mvt_dir precedent)
pub extern "C" fn visiaengine_update_entity_attr_str(
    ve: u64,
    entity: u64,
    name: *const c_char,
    v: *const c_char,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if name.is_null() || v.is_null() {
                        return VE_ERR_ARG;
                    }
                    let name = match unsafe { CStr::from_ptr(name) }.to_str() {
                        Ok(n) => n,
                        Err(_) => return VE_ERR_ARG,
                    };
                    let val = { unsafe { CStr::from_ptr(v) }.to_string_lossy().into_owned() };
                    match with_engine(i, |e| e.update_entity_props_str(entity, name, val)) {
                        Ok(()) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-38 (N2.4): typed property diff-update — bool column (1/0).
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // NULL-guarded; attr-face precedent
pub extern "C" fn visiaengine_update_entity_attr_bool(
    ve: u64,
    entity: u64,
    name: *const c_char,
    v: i32,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if name.is_null() {
                        return VE_ERR_ARG;
                    }
                    let name = match unsafe { CStr::from_ptr(name) }.to_str() {
                        Ok(n) => n,
                        Err(_) => return VE_ERR_ARG,
                    };
                    match with_engine(i, |e| e.update_entity_props_bool(entity, name, v != 0)) {
                        Ok(()) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-02 behavior probe (for Rust-side integration tests; not extern, stays off the ABI face / nm whitelist).
pub mod test_util {
    use super::*;
    #[must_use]
    pub fn guard_panic_probe() -> i32 {
        capi_guard!(panic!("injected for CAPI-02"), VE_ERR_PANIC)
    }
}

// -- B1 data band (CAPI-13..16): visibility / query / procedural add-delete; gate/with_engine/capi_guard single-source form --

/// CAPI-15 value structure (struct_size forward-compat gate = same scheme as VeInput; pointers read during the call only, releasable by the host on return).
#[repr(C)]
pub struct VeMeshDesc {
    pub struct_size: usize,
    pub positions: *const f32, // [x,y,z] × n_positions
    pub normals: *const f32,   // may be NULL = engine synthesizes +Z
    pub indices: *const u32,
    pub n_positions: u64,
    pub n_indices: u64,
    pub base_color: *const f32, // 4-tuple
    pub origin: *const f64,     // 3-tuple
    // N6 (REND-45): GGX PBR values (NOT pointers -- struct_size forward-compat gate governs).
    pub roughness: f32, // GGX perceptual roughness; 0=host-said-nothing → engine default 1.0
    pub metalness: f32, // metallic factor; 0 = dielectric
}

/// CAPI-13: owner thread; strict visible in {0,1}; unknown shape / out-of-range value = -1 (VE_ERR_ARG), idempotent repeat = 0.
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

/// CAPI-14: owner thread (the last_error exception set is not among these -- pure table read); 1/0/unknown = -1.
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

/// CAPI-15: owner thread; 0 = success writing *out_entity (shape may = 0, CAPI-01 scheme),
/// degenerate / NULL / struct_size gate / bad out = -1 with out zero-partial-write (same discipline as attr_str).
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
                    // 0.0 roughness = legacy zero-init hosts said nothing → map to
                    // the dielectric default 1.0 (0.0 roughness is not a legal
                    // GGX input a host can set deliberately: use 1e-4 minimum).
                    let roughness = if d.roughness <= 0.0 { 1.0 } else { d.roughness };
                    let metalness = d.metalness;
                    match with_engine(i, move |e| {
                        e.add_mesh(
                            positions, normals, indices, base_color, origin, roughness, metalness,
                        )
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

/// CAPI-16: owner thread; success = 0; unknown shape / old handle re-entry = -1 (same scheme as double destroy).
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

/// CAPI-17: register/replace/unregister (NULL) the event callback. native-only (wasm32 has no C ABI face,
/// JS closures go through the visiaengine-wasm bridge set_event_fn; set_event_anchor is also native cfg).
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

/// CAPI-18 point element (the C mirror of the 32B PointMark Pod; built element by element with new() engine-side, no bytemuck translation face).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VePointMark {
    pub pos: [f32; 3],
    pub radius_px: f32,
    pub color: [f32; 3],
}

/// CAPI-18 value structure (struct_size forward-compat gate follows the VeMeshDesc scheme).
#[repr(C)]
pub struct VePointsDesc {
    pub struct_size: usize,
    pub marks: *const VePointMark,
    pub count: u64,
}

/// CAPI-19 report structure (the C mirror of io-points PclReport; struct_size forward-compat gate same scheme).
#[repr(C)]
pub struct VePclReport {
    pub struct_size: usize,
    pub dropped_non_finite: u32,
    pub dropped_out_of_domain: u32,
    pub dropped_unsupported: u32,
    pub truncated_points: u32,
    pub kept: u64,
}

/// CAPI-19 load policy.
pub const VE_PCL_FASTFAIL: u32 = 0;
pub const VE_PCL_LENIENT: u32 = 1;

/// CAPI-19: point-cloud file load. RED stub = gates all real, behavior always -3 (the success road must be red).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_load_pcl(
    ve: u64,
    path: *const std::os::raw::c_char,
    policy: u32,
    out_entity: *mut u64,
    out_report: *mut VePclReport,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if path.is_null()
                        || out_entity.is_null()
                        || out_report.is_null()
                        || !matches!(policy, VE_PCL_FASTFAIL | VE_PCL_LENIENT)
                    {
                        set_err(
                            "load_pcl: null out / policy out of range (zero partial write)"
                                .to_string(),
                        );
                        return VE_ERR_ARG;
                    }
                    // SAFETY: FFI boundary -- the struct_size forward-compat gate goes first (an old host's view of the new engine form is unchanged)
                    let want = unsafe { &*out_report };
                    if want.struct_size < std::mem::size_of::<VePclReport>() {
                        set_err("load_pcl: report struct_size too old".to_string());
                        return VE_ERR_ARG;
                    }
                    let cstr = unsafe { std::ffi::CStr::from_ptr(path) };
                    let p = cstr.to_string_lossy().into_owned();
                    let r = with_engine(i, |e| e.load_pcl(&p, policy == VE_PCL_LENIENT));
                    match r {
                        Ok((bits, rep)) => {
                            // SAFETY: non-null verified; the single write point on success (upper faces untouched on failure paths)
                            unsafe {
                                *out_entity = bits;
                                *out_report = VePclReport {
                                    struct_size: std::mem::size_of::<VePclReport>(),
                                    dropped_non_finite: rep.dropped_non_finite,
                                    dropped_out_of_domain: rep.dropped_out_of_domain,
                                    dropped_unsupported: rep.dropped_unsupported,
                                    truncated_points: rep.truncated_points,
                                    kept: rep.kept,
                                };
                            }
                            0
                        }
                        Err(msg) => {
                            set_err(format!("load_pcl '{p}': {msg}"));
                            // CAPI-17: load-failure-exit event (the progress face fires at the mount tail terminal)
                            let _ = with_engine(i, |e| {
                                e.emit(VE_EVT_LOAD_ERROR, VE_ERR_IO as i64 as u64, 0);
                                Ok::<(), String>(())
                            });
                            VE_ERR_IO
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-18: point-cloud passthrough load (raw dereference inside the fence = the settled FFI-boundary exception, the allow family records the ruling).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_add_points(
    ve: u64,
    desc: *const VePointsDesc,
    out_entity: *mut u64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if desc.is_null() || out_entity.is_null() {
                        set_err("add_points: null desc/out".to_string());
                        return VE_ERR_ARG;
                    }
                    // SAFETY: FFI boundary -- non-null verified, the struct read by value (releasable by the host on return)
                    let d = unsafe { &*desc };
                    if d.struct_size < std::mem::size_of::<VePointsDesc>()
                        || d.count == 0
                        || d.marks.is_null()
                    {
                        set_err("add_points: degenerate desc (zero commit)".to_string());
                        return VE_ERR_ARG;
                    }
                    // SAFETY: marks[0..count] guaranteed valid by the host (same contract as VeMeshDesc)
                    let raw = unsafe { std::slice::from_raw_parts(d.marks, d.count as usize) };
                    match with_engine(i, |e| e.add_points(raw)) {
                        Ok(bits) => {
                            // SAFETY: out_entity non-null verified; the single write point on success
                            unsafe { *out_entity = bits };
                            0
                        }
                        Err(msg) => {
                            set_err(format!("add_points: {msg}"));
                            VE_ERR_IO
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

// -- B2 clip band (CAPI-20): world cut planes set/get; gate/with_engine/capi_guard single-source form --

/// CAPI-20 value structure (32B Pod, coefficient form [n, d]; d = -<n, anchor>, normalization on the engine side).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VeClipPlane {
    pub nx: f64,
    pub ny: f64,
    pub nz: f64,
    pub d: f64,
}

/// CAPI-20: set the clip planes (owner thread, same as the visibility entry). n in [0,4], **n=0 = the only clear form**;
/// n>4 reject-before-read (bounds check before dereference); NULL and n>0 / degenerate (zero normal / non-finite) = -1 + error string.
/// Semantics = the normal points to the kept side, `dot(n,P)+d >= 0` keeps (on-plane = kept); multiple planes AND.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_set_clips(ve: u64, planes: *const VeClipPlane, n: usize) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if n > 4 {
                        set_err("set_clips: n>MAX(4)".to_string());
                        return VE_ERR_ARG;
                    }
                    if n == 0 {
                        return match with_engine(i, |e| e.set_clips(&[])) {
                            Ok(()) => VE_OK,
                            Err(msg) => {
                                set_err(msg);
                                VE_ERR_ARG
                            }
                        };
                    }
                    if planes.is_null() {
                        set_err("set_clips: null planes with n>0".to_string());
                        return VE_ERR_ARG;
                    }
                    // SAFETY: planes[0..n] guaranteed valid by the host (same contract as VeMeshDesc); the n<=4 bound is gated above
                    let raw = unsafe { std::slice::from_raw_parts(planes, n) };
                    let arr: Vec<[f64; 4]> = raw.iter().map(|p| [p.nx, p.ny, p.nz, p.d]).collect();
                    match with_engine(i, |e| e.set_clips(&arr)) {
                        Ok(()) => VE_OK,
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

/// CAPI-20: readback. **Return >=0 = current plane count** (errors exclusive to negative codes, the return value does not serve two masters);
/// buf NULL = count only; cap truncation = write min(cap,count) while returning the true count (n=3, cap=2 -> return 3 write 2);
/// cap=0 with buf!=NULL = a legal pure-count road.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_get_clips(ve: u64, buf: *mut VeClipPlane, cap: usize) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => match with_engine(i, |e| Ok(e.clips())) {
                    Ok(cs) => {
                        let m = if buf.is_null() { 0 } else { cap.min(cs.len()) };
                        if m > 0 {
                            // SAFETY: buf[0..m] guaranteed valid by the host (m<=cap<=a validated array;
                            // m=0 builds no slice -- NULL+0 would trip from_raw_parts UB beforehand [debug proof])
                            unsafe {
                                let dst = std::slice::from_raw_parts_mut(buf, m);
                                for (slot, pl) in dst.iter_mut().zip(&cs) {
                                    *slot = VeClipPlane {
                                        nx: pl[0],
                                        ny: pl[1],
                                        nz: pl[2],
                                        d: pl[3],
                                    };
                                }
                            }
                        }
                        cs.len() as i32
                    }
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

/// CAPI-22 value structure (struct_size forward-compat gate = same scheme as VeMeshDesc; pos = world f64 [D7],
/// color = host sRGB face [CORE-16], text = NUL-terminated UTF-8).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VeLabelSpec {
    pub struct_size: usize,
    pub pos: [f64; 3],
    pub color: [f32; 4],
    pub size_px: f32,
    pub text: *const std::ffi::c_char,
}

/// CAPI-21: load/replace the label font (owner thread; NULL/0/non-TTF = -1 + error string, a success leaves the existing font untouched).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_load_font(ve: u64, data: *const u8, len: usize) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if data.is_null() || len == 0 {
                        set_err("load_font: null/empty".to_string());
                        return VE_ERR_ARG;
                    }
                    // SAFETY: data[0..len] guaranteed valid by the host (same byte contract as load_*)
                    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
                    match with_engine(i, |e| e.set_font(bytes)) {
                        Ok(()) => VE_OK,
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

/// CAPI-22: single world-anchored label (owner thread; no font = rejected [timing gate]; NULL spec / bad out / struct_size
/// gate / empty text / size out of domain / bad UTF-8 = -1 zero commit; success = 0 writing *out (shape may be 0 = the 15 scheme, same system as CAPI-01)).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_add_label(
    ve: u64,
    spec: *const VeLabelSpec,
    out_entity: *mut u64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if spec.is_null() {
                        set_err("add_label: null spec".to_string());
                        return VE_ERR_ARG;
                    }
                    // SAFETY: spec non-null verified; the struct_size gate precedes field reads [FFI-R:CS-4]
                    let hdr = unsafe { *spec };
                    if hdr.struct_size < std::mem::size_of::<VeLabelSpec>() {
                        set_err("add_label: struct_size too small".to_string());
                        return VE_ERR_ARG;
                    }
                    if hdr.text.is_null() {
                        set_err("add_label: null text".to_string());
                        return VE_ERR_ARG;
                    }
                    if out_entity.is_null() {
                        set_err("add_label: null out_entity".to_string());
                        return VE_ERR_ARG;
                    }
                    // SAFETY: the text pointer is non-null and NUL-terminated (host contract, same as the path entry family)
                    let text = match unsafe { std::ffi::CStr::from_ptr(hdr.text) }.to_str() {
                        Ok(t) => t,
                        Err(_) => {
                            set_err("add_label: invalid utf-8".to_string());
                            return VE_ERR_ARG;
                        }
                    };
                    let finite = hdr.pos.iter().all(|v| v.is_finite())
                        && hdr.color.iter().all(|v| (0.0..=1.0).contains(v));
                    if !finite || !hdr.size_px.is_finite() {
                        set_err("add_label: non-finite spec".to_string());
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| e.add_label(hdr.pos, text, hdr.color, hdr.size_px)) {
                        Ok(bits) => {
                            // SAFETY: out_entity non-null verified; the single write point on success
                            unsafe { *out_entity = bits };
                            VE_OK
                        }
                        Err(msg) => {
                            set_err(format!("add_label: {msg}"));
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

// -- 5a flight band (CAPI-23/24): wall-clock thruster thin leaf (layering = plan T1) --

/// CAPI-23 value structure (struct_size forward-compat gate = same scheme as VeMeshDesc; pose six components + fov,
/// **near/far stay out of pose = always the from values** [Momus-A1/PIT-5 linked]).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VeCameraPose {
    pub struct_size: usize,
    pub target: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub dist: f64,
    pub zoom: f64,
    pub fov: f64,
}

/// CAPI-23: takeoff/reroute (same family as the owner-thread gate). 0 = takeoff (dur_ms=0 = teleport also 0);
/// NULL / struct_size gate / out of domain = -1 + error string. Re-entry while flying is legal (from = current sampled pose).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_fly_to(ve: u64, pose: *const VeCameraPose, dur_ms: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if pose.is_null() {
                        set_err("fly_to: null pose".to_string());
                        return VE_ERR_ARG;
                    }
                    // SAFETY: non-null verified; 64B Pod direct read (host guarantees [addr,addr+size) valid)
                    let p = unsafe { *pose };
                    if p.struct_size < std::mem::size_of::<VeCameraPose>() {
                        set_err("fly_to: struct_size too small".to_string());
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| {
                        e.fly_to(p.target, p.yaw, p.pitch, p.dist, p.zoom, p.fov, dur_ms)
                    }) {
                        Ok(()) => VE_OK,
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

/// CAPI-24: flight state. Returns **1 = done (the merged idle/done) / 0 = flying**;
/// out_t01 written only while flying [0,1) (the done path writes zero = B1 scheme), NULL = state only (same system as get_clips).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_fly_state(ve: u64, out_t01: *mut f64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    match with_engine(i, |e| Ok((e.fly_state_done(), e.fly_progress()))) {
                        Ok((true, _)) => 1,
                        Ok((false, Some(t))) => {
                            if !out_t01.is_null() {
                                // SAFETY: non-null judged beforehand; single f64 write (the flying road only)
                                unsafe { *out_t01 = t };
                            }
                            0
                        }
                        Ok((false, None)) => 1, // theoretically unreachable (thruster atomicity), conservatively attributed to done
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

// -- 5b second wave (CAPI-25..27): minimap / navigation / pose readback --

/// CAPI-25 value structure (normalized rect; NULL = disable and clear).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct VeMapView {
    pub struct_size: usize,
    pub fx: f32,
    pub fy: f32,
    pub fw: f32,
    pub fh: f32,
    pub zoom: f64,
}

/// CAPI-25: enable/disable the minimap (NULL = off; domain rejection = -1 zero side effects; auto-follows resize).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_set_map(ve: u64, cfg: *const VeMapView) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if cfg.is_null() {
                        return match with_engine(i, |e| e.set_map(None, 0.0)) {
                            Ok(()) => VE_OK,
                            Err(msg) => {
                                set_err(msg);
                                VE_ERR_ARG
                            }
                        };
                    }
                    // SAFETY: non-null verified; Pod direct read (host guarantees [addr,size) valid)
                    let m = unsafe { *cfg };
                    if m.struct_size < std::mem::size_of::<VeMapView>() {
                        set_err("set_map: struct_size too small".to_string());
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| e.set_map(Some((m.fx, m.fy, m.fw, m.fh)), m.zoom)) {
                        Ok(()) => VE_OK,
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

/// CAPI-26: minimap click navigation (outside region / no map = rejected with no covert reroute; entity hit first, ground fallback).
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_navigate_click(ve: u64, px: f32, py: f32, dur_ms: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if !px.is_finite() || !py.is_finite() {
                        set_err("navigate_click: non-finite px/py".to_string());
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| e.navigate_click(px, py, dur_ms)) {
                        Ok(()) => VE_OK,
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

/// CAPI-27: main camera pose readback (out.struct_size pre-stamped by the host = the gate; NULL = rejected).
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_get_camera(ve: u64, out: *mut VeCameraPose) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if out.is_null() {
                        set_err("get_camera: null out".to_string());
                        return VE_ERR_ARG;
                    }
                    // SAFETY: non-null verified; read the gate field before writing (the host pre-stamps struct_size)
                    let hdr = unsafe { *out };
                    if hdr.struct_size < std::mem::size_of::<VeCameraPose>() {
                        set_err("get_camera: struct_size too small".to_string());
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| Ok(e.camera_pose())) {
                        Ok(rig) => {
                            // SAFETY: non-null + gate passed; all fields written at once (Pod has no Drop)
                            unsafe {
                                *out = VeCameraPose {
                                    struct_size: hdr.struct_size,
                                    target: rig.target,
                                    yaw: rig.yaw,
                                    pitch: rig.pitch,
                                    dist: rig.dist,
                                    zoom: rig.zoom,
                                    fov: rig.fov_y,
                                };
                            }
                            VE_OK
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

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn visiaengine_load_mvt_dir(
    ve: u64,
    path: *const std::os::raw::c_char,
    z: u32,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if path.is_null() || z > 30 {
                        return VE_ERR_ARG;
                    }
                    let path = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
                        Ok(p) => p,
                        Err(_) => return VE_ERR_ARG,
                    };
                    match with_engine(i, |e| e.load_mvt_dir(path, z as u8)) {
                        Ok(n) => n as i32,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-42 (V2.2): write a node-local transform (column-major f64 x16).
/// 0 = ok; negative = error (NULL / non-finite / unknown entity).
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // NULL-guarded (attr-face precedent)
pub extern "C" fn visiaengine_set_node_transform(ve: u64, entity: u64, m: *const f64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if m.is_null() {
                        return VE_ERR_ARG;
                    }
                    let mut mat = [[0.0f64; 4]; 4];
                    unsafe {
                        for (r, row) in mat.iter_mut().enumerate() {
                            for (c, v) in row.iter_mut().enumerate() {
                                *v = *m.add(r * 4 + c);
                            }
                        }
                    }
                    match with_engine(i, |e| e.set_node_transform(entity, mat)) {
                        Ok(()) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-43 (V2.2): read back a node-local transform (column-major f64 x16).
/// 0 = ok; negative = error (NULL / unknown entity).
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // NULL-guarded (get_clips precedent)
pub extern "C" fn visiaengine_get_node_transform(ve: u64, entity: u64, out: *mut f64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if out.is_null() {
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| e.get_node_transform(entity)) {
                        Ok(mat) => {
                            unsafe {
                                for (r, row) in mat.iter().enumerate() {
                                    for (c, v) in row.iter().enumerate() {
                                        *out.add(r * 4 + c) = *v;
                                    }
                                }
                            }
                            0
                        }
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-41 (I band): load an .hdr environment (RGBE equirect), project to
/// SH-9 irradiance, install as global env (replaces legacy constant ambient;
/// demo-grade — PMREM/LUT = separate ticket). 0 = ok, negative = error.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // NULL-guarded (load_font precedent)
pub extern "C" fn visiaengine_load_env_hdr(ve: u64, path: *const std::os::raw::c_char) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if path.is_null() {
                        return VE_ERR_ARG;
                    }
                    let path = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
                        Ok(p) => p,
                        Err(_) => return VE_ERR_ARG,
                    };
                    match with_engine(i, |e| e.load_env_hdr(path)) {
                        Ok(()) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-39 (R band): mount a RASTER tile directory (PNG; flat ground quads,
/// NOT terrain/drape — N1.2 wording discipline). Returns tile count.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // NULL-guarded (load_mvt_dir precedent)
pub extern "C" fn visiaengine_load_raster_dir(
    ve: u64,
    path: *const std::os::raw::c_char,
    z: u32,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if path.is_null() || z > 30 {
                        return VE_ERR_ARG;
                    }
                    let path = match unsafe { std::ffi::CStr::from_ptr(path) }.to_str() {
                        Ok(p) => p,
                        Err(_) => return VE_ERR_ARG,
                    };
                    match with_engine(i, |e| e.load_raster_dir(path, z as u8)) {
                        Ok(n) => n as i32,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-40 (R band): raster view bbox (3857 meters, finite, min<max).
/// Returns the mounted-tiles-visible count in the requested view.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_set_raster_view(
    ve: u64,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    match with_engine(i, |e| e.set_raster_view(min_x, min_y, max_x, max_y)) {
                        Ok(n) => n as i32,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_set_tile_view(
    ve: u64,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    match with_engine(i, |e| e.set_tile_view(min_x, min_y, max_x, max_y)) {
                        Ok(visible) => visible as i32,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-35 (N1.4): mount an HTTP tile source (discovery-driven — no
/// directory listing; the tile set is discovered by set_tile_view viewport
/// enumeration and pumped per render frame). empty url / z>30 = reject.
/// Returns 0 = mounted (count-semantics fork vs CAPI-28 documented in the
/// clause: HTTP has no directory to scan).
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)] // NULL-guarded before deref (PIT-26); load_mvt_dir precedent
pub extern "C" fn visiaengine_set_tile_source_http(ve: u64, url: *const c_char, z: u8) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    // N1.4: NULL+0 must not form a slice (PIT-26) — read as CStr first.
                    let url_str = if url.is_null() {
                        String::new()
                    } else {
                        unsafe { CStr::from_ptr(url) }
                            .to_string_lossy()
                            .into_owned()
                    };
                    match with_engine(i, |e| e.load_mvt_http(&url_str, z)) {
                        Ok(_) => 0,
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}
// ── Scene tree (CAPI-30..34) ────────────────────────────────────────────────

/// CAPI-30: Create a group node. `name` may be NULL (anonymous).
/// out_entity: written on VE_OK; caller must pre-allocate 8 bytes.
/// Returns VE_OK (0) or neg error code. out_entity=0 is a legal slot0gen0 group.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn visiaengine_create_group(
    ve: u64,
    name: *const std::os::raw::c_char,
    out_entity: *mut u64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if out_entity.is_null() {
                        return VE_ERR_ARG;
                    }
                    let n = if name.is_null() {
                        None
                    } else {
                        match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
                            Ok(s) => Some(s),
                            Err(_) => {
                                set_err("name not valid UTF-8".into());
                                return VE_ERR_ARG;
                            }
                        }
                    };
                    match with_engine(i, |e| e.create_group(n)) {
                        Ok(bits) => {
                            unsafe {
                                *out_entity = bits;
                            }
                            VE_OK
                        }
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-31: Reparent. parent=0 = detach to root. 0=ok; neg=err.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_set_parent(ve: u64, child: u64, parent: u64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => match with_engine(i, |e| e.set_parent(child, parent)) {
                    Ok(()) => 0,
                    Err(msg) => {
                        set_err(msg);
                        VE_ERR_ARG
                    }
                },
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-32: Get parent handle. 0=root/no parent/err.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_get_parent(ve: u64, entity: u64) -> u64 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => with_engine(i, |e| e.get_parent(entity)).unwrap_or(0),
                _ => 0,
            }
        },
        0u64
    )
}

/// CAPI-33: Set group offset (world metres, f64×3). 0=ok; neg=err.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
pub extern "C" fn visiaengine_set_group_offset(
    ve: u64,
    group: u64,
    dx: f64,
    dy: f64,
    dz: f64,
) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => match with_engine(i, |e| e.set_group_offset(group, dx, dy, dz)) {
                    Ok(()) => 0,
                    Err(msg) => {
                        set_err(msg);
                        VE_ERR_ARG
                    }
                },
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}

/// CAPI-34: Read group own offset. out=write [dx,dy,dz]. 0=ok; neg=err.
#[cfg_attr(not(target_arch = "wasm32"), unsafe(no_mangle))]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn visiaengine_get_group_offset(ve: u64, group: u64, out: *mut f64) -> i32 {
    capi_guard!(
        {
            match gate(ve) {
                Gate::Live(i) => {
                    if out.is_null() {
                        return VE_ERR_ARG;
                    }
                    match with_engine(i, |e| e.get_group_offset(group)) {
                        Ok(o) => {
                            // SAFETY: out not null; caller guarantees ≥3×f64 capacity
                            unsafe {
                                *out = o[0];
                                *out.add(1) = o[1];
                                *out.add(2) = o[2];
                            }
                            0
                        }
                        Err(msg) => {
                            set_err(msg);
                            VE_ERR_ARG
                        }
                    }
                }
                Gate::Arg => VE_ERR_ARG,
                Gate::State => VE_ERR_STATE,
            }
        },
        VE_ERR_PANIC
    )
}
