//! Native SideFX CV `CV_PaintBuffer::textWrappedInBox` package.

use glyphshift_adapter_native_abi::{
    DecideUtf16V1, NativeAdapterApiV1, NativeAdapterDescriptorV1, NativeDecisionV1,
    NativeNegotiationV1, NativeRuntimeHostV1, ARCH_X86_64, DECISION_TEXT_REPLACE,
    FEATURE_TEXT_OBSERVE, FEATURE_TEXT_REPLACE, PLATFORM_WINDOWS, STATUS_ACTIVATION_FAILED,
    STATUS_INVALID_HOST, STATUS_OK, STATUS_UNAUTHORIZED_FEATURE, STATUS_UNSUPPORTED_FEATURE,
};
use glyphshift_adapter_sidefx_cv_paint_buffer::{ADAPTER_ID, MAX_TEXT_BYTES};
use retour::GenericDetour;
use std::cell::Cell;
use std::ffi::{c_char, c_void, CStr, CString};
use std::mem;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{OnceLock, RwLock};
use windows::core::{w, PCSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

const SUPPORTED_FEATURES: u64 = FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE;
const MAX_TEXT_UNITS: usize = 64 * 1024;
const TEXT_WRAPPED_IN_BOX_SYMBOL: &[u8] = b"?textWrappedInBox@CV_PaintBuffer@@QEAAXAEBV?$UT_Rect@VUT_DimRectImpl@@@@W4CV_HorizontalAlignment@@W4CV_VerticalAlignment@@PEBDPEAVUT_Color@@PEANPEAV2@@Z\0";

type FnTextWrappedInBox = unsafe extern "C" fn(
    *mut c_void,
    *const c_void,
    i32,
    i32,
    *const c_char,
    *mut c_void,
    *mut f64,
    *mut c_void,
);

#[derive(Clone, Copy)]
struct HostBridge {
    context: usize,
    decide_utf16: DecideUtf16V1,
}

struct Hooks {
    text_wrapped_in_box: GenericDetour<FnTextWrappedInBox>,
}

struct DecisionBuffers {
    decision: NativeDecisionV1,
    text: Vec<u16>,
}

static ACTIVE_FEATURES: AtomicU64 = AtomicU64::new(0);
static HOST: OnceLock<RwLock<HostBridge>> = OnceLock::new();
static HOOKS: OnceLock<Hooks> = OnceLock::new();

thread_local! {
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
}

struct CallbackGuard;

impl CallbackGuard {
    fn enter() -> Option<Self> {
        IN_CALLBACK.with(|active| (!active.replace(true)).then_some(Self))
    }
}

impl Drop for CallbackGuard {
    fn drop(&mut self) {
        IN_CALLBACK.with(|active| active.set(false));
    }
}

extern "C" fn negotiate_features(requested: u64, granted: u64) -> NativeNegotiationV1 {
    if requested & !SUPPORTED_FEATURES != 0 {
        return negotiation_error(STATUS_UNSUPPORTED_FEATURE);
    }
    if requested & !granted != 0 {
        return negotiation_error(STATUS_UNAUTHORIZED_FEATURE);
    }
    NativeNegotiationV1 {
        status: STATUS_OK,
        active_feature_bits: requested,
    }
}

const fn negotiation_error(status: i32) -> NativeNegotiationV1 {
    NativeNegotiationV1 {
        status,
        active_feature_bits: 0,
    }
}

extern "C" fn activate(
    host: *const NativeRuntimeHostV1,
    requested: u64,
    granted: u64,
) -> NativeNegotiationV1 {
    if host.is_null()
        || unsafe { (*host).struct_size } != mem::size_of::<NativeRuntimeHostV1>() as u32
    {
        return negotiation_error(STATUS_INVALID_HOST);
    }
    let negotiated = negotiate_features(requested, granted);
    if negotiated.status != STATUS_OK {
        return negotiated;
    }
    let host = unsafe { *host };
    let bridge = HostBridge {
        context: host.context as usize,
        decide_utf16: host.decide_utf16,
    };
    let host_ready = if let Some(current) = HOST.get() {
        current.write().map(|mut current| *current = bridge).is_ok()
    } else {
        HOST.set(RwLock::new(bridge)).is_ok()
    };
    if !host_ready || unsafe { install_hooks() }.is_err() {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }
    ACTIVE_FEATURES.store(negotiated.active_feature_bits, Ordering::Release);
    negotiated
}

extern "C" fn deactivate() -> i32 {
    ACTIVE_FEATURES.store(0, Ordering::Release);
    STATUS_OK
}

fn decide(source: &str) -> Option<DecisionBuffers> {
    let host = *HOST.get()?.read().ok()?;
    let source = source.encode_utf16().collect::<Vec<_>>();
    if source.is_empty() || source.len() > MAX_TEXT_UNITS {
        return None;
    }
    let mut text = vec![0_u16; MAX_TEXT_UNITS];
    let mut font = [];
    let decision = (host.decide_utf16)(
        host.context as *mut c_void,
        source.as_ptr(),
        source.len() as u32,
        text.as_mut_ptr(),
        text.len() as u32,
        font.as_mut_ptr(),
        0,
    );
    if decision.status != STATUS_OK || decision.text_len as usize > text.len() {
        return None;
    }
    text.truncate(decision.text_len as usize);
    Some(DecisionBuffers { decision, text })
}

fn replacement_for(source: &str) -> Option<CString> {
    let decision = std::panic::catch_unwind(|| decide(source)).ok().flatten()?;
    if ACTIVE_FEATURES.load(Ordering::Acquire) & FEATURE_TEXT_REPLACE == 0
        || decision.decision.decision_bits & DECISION_TEXT_REPLACE == 0
        || decision.text.is_empty()
    {
        return None;
    }
    let replacement = String::from_utf16(&decision.text).ok()?;
    if replacement.is_empty() || replacement.len() > MAX_TEXT_BYTES {
        return None;
    }
    CString::new(replacement).ok()
}

unsafe fn read_source(text: *const c_char) -> Option<String> {
    if text.is_null() {
        return None;
    }
    let bytes = CStr::from_ptr(text).to_bytes();
    if bytes.is_empty() || bytes.len() > MAX_TEXT_BYTES {
        return None;
    }
    std::str::from_utf8(bytes).ok().map(ToOwned::to_owned)
}

unsafe extern "C" fn text_wrapped_in_box_detour(
    this: *mut c_void,
    rect: *const c_void,
    horizontal: i32,
    vertical: i32,
    text: *const c_char,
    color: *mut c_void,
    scale: *mut f64,
    bounds: *mut c_void,
) {
    let Some(hooks) = HOOKS.get() else {
        return;
    };
    let original = &hooks.text_wrapped_in_box;
    if ACTIVE_FEATURES.load(Ordering::Acquire) == 0 {
        return original.call(this, rect, horizontal, vertical, text, color, scale, bounds);
    }
    let Some(_guard) = CallbackGuard::enter() else {
        return original.call(this, rect, horizontal, vertical, text, color, scale, bounds);
    };
    let Some(source) = read_source(text) else {
        return original.call(this, rect, horizontal, vertical, text, color, scale, bounds);
    };
    let Some(replacement) = replacement_for(&source) else {
        return original.call(this, rect, horizontal, vertical, text, color, scale, bounds);
    };
    original.call(
        this,
        rect,
        horizontal,
        vertical,
        replacement.as_ptr(),
        color,
        scale,
        bounds,
    );
}

unsafe fn resolve<T: Copy>(module: HMODULE, symbol: &'static [u8]) -> Option<T> {
    let address = GetProcAddress(module, PCSTR(symbol.as_ptr()));
    address.map(|address| mem::transmute_copy::<_, T>(&address))
}

unsafe fn install_hooks() -> Result<(), ()> {
    if HOOKS.get().is_some() {
        return Ok(());
    }
    let module = GetModuleHandleW(w!("libCV.dll")).map_err(|_| ())?;
    let target = resolve(module, TEXT_WRAPPED_IN_BOX_SYMBOL).ok_or(())?;
    let hooks = Hooks {
        text_wrapped_in_box: GenericDetour::new(
            target,
            text_wrapped_in_box_detour as FnTextWrappedInBox,
        )
        .map_err(|_| ())?,
    };
    HOOKS.set(hooks).map_err(|_| ())?;
    let hooks = HOOKS.get().ok_or(())?;
    hooks.text_wrapped_in_box.enable().map_err(|_| ())?;
    Ok(())
}

#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: mem::size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::inline_target(
            ADAPTER_ID,
            (1, 0, 0),
            SUPPORTED_FEATURES,
            PLATFORM_WINDOWS,
            ARCH_X86_64,
        ),
        negotiate_features,
        activate,
        deactivate,
        request_refresh: glyphshift_adapter_native_abi::request_refresh_noop,
    }
}
