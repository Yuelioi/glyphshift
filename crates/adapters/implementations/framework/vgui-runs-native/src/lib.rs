//! Deferred VGUI text runs. Only public drawing calls and IImage paint boundaries
//! are intercepted; no application's text-object fields are read or rewritten.
use glyphshift_adapter_native_abi::*;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Mutex,
};
#[cfg(all(windows, target_arch = "x86"))]
mod platform;
const ADAPTER_ID: &str = "windows.vgui.text-run";
const FEATURES: u64 = FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE;
static ACTIVE: AtomicU64 = AtomicU64::new(0);
static STOPPING: AtomicBool = AtomicBool::new(false);
static IN_FLIGHT: AtomicUsize = AtomicUsize::new(0);
/// Last reached activation stage: 10 interface, 30 paint boundaries, 40 publish,
/// 50 ready. Negative values identify rejected method contracts. No text is exposed.
#[export_name = "glyphshift_vgui_activation_stage_v1"]
pub static ACTIVATION_STAGE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);
static HOST: Mutex<Option<NativeTextHostBinding>> = Mutex::new(None);
/// Paint entries, qualified entries, buffered glyphs, empty/disabled ends,
/// opaque calls, missing glyph state, rejected adjacency, unscoped glyphs.
#[export_name = "glyphshift_vgui_paint_status_v1"]
pub static PAINT_STATUS: [std::sync::atomic::AtomicI32; 8] =
    [const { std::sync::atomic::AtomicI32::new(0) }; 8];
/// Last glyph refusal: disabled, unseen font/color, missing fresh position,
/// missing font/color/position values (successive bits).
#[export_name = "glyphshift_vgui_missing_state_v1"]
pub static MISSING_STATE: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

/// Bounded process-local counters for distinguishing decisions from native draw
/// acceptance. Fields are independent samples; no text or object pointers are kept.
#[repr(C)]
pub struct DrawStatus {
    host_errors: std::sync::atomic::AtomicI32,
    last_host_status: std::sync::atomic::AtomicI32,
    matched: std::sync::atomic::AtomicI32,
    measured: std::sync::atomic::AtomicI32,
    width: std::sync::atomic::AtomicI32,
    height: std::sync::atomic::AtomicI32,
    maximum: std::sync::atomic::AtomicI32,
    rejected: std::sync::atomic::AtomicI32,
    accepted: std::sync::atomic::AtomicI32,
    direct_draws: std::sync::atomic::AtomicI32,
    deferred_draws: std::sync::atomic::AtomicI32,
    x: std::sync::atomic::AtomicI32,
    y: std::sync::atomic::AtomicI32,
}
#[export_name = "glyphshift_vgui_draw_status_v1"]
pub static DRAW_STATUS: DrawStatus = DrawStatus {
    host_errors: std::sync::atomic::AtomicI32::new(0),
    last_host_status: std::sync::atomic::AtomicI32::new(0),
    matched: std::sync::atomic::AtomicI32::new(0),
    measured: std::sync::atomic::AtomicI32::new(0),
    width: std::sync::atomic::AtomicI32::new(0),
    height: std::sync::atomic::AtomicI32::new(0),
    maximum: std::sync::atomic::AtomicI32::new(0),
    rejected: std::sync::atomic::AtomicI32::new(0),
    accepted: std::sync::atomic::AtomicI32::new(0),
    direct_draws: std::sync::atomic::AtomicI32::new(0),
    deferred_draws: std::sync::atomic::AtomicI32::new(0),
    x: std::sync::atomic::AtomicI32::new(0),
    y: std::sync::atomic::AtomicI32::new(0),
};

#[no_mangle]
/// # Safety
/// Non-null hosts and their callbacks must remain valid until all drawing ends.
pub unsafe extern "C" fn glyphshift_adapter_bind_text_host_v1(
    host: *const NativeTextHostV1,
) -> i32 {
    if ACTIVE.load(Ordering::Acquire) != 0 {
        return STATUS_INVALID_HOST;
    }
    let binding = if host.is_null() {
        None
    } else {
        let host = unsafe { *host };
        if host.enter_scope.is_none() || host.leave_scope.is_none() {
            return STATUS_INVALID_HOST;
        }
        let Some(binding) = NativeTextHostBinding::new(host) else {
            return STATUS_INVALID_HOST;
        };
        Some(binding)
    };
    match HOST.lock() {
        Ok(mut slot) => {
            *slot = binding;
            STATUS_OK
        }
        Err(_) => STATUS_INVALID_HOST,
    }
}
extern "C" fn negotiate(requested: u64, granted: u64) -> NativeNegotiationV1 {
    NativeNegotiationV1 {
        status: if requested & !FEATURES != 0 {
            STATUS_UNSUPPORTED_FEATURE
        } else if requested & !granted != 0 {
            STATUS_UNAUTHORIZED_FEATURE
        } else {
            STATUS_OK
        },
        active_feature_bits: requested & granted & FEATURES,
    }
}
extern "C" fn activate(
    host: *const NativeRuntimeHostV1,
    requested: u64,
    granted: u64,
) -> NativeNegotiationV1 {
    ACTIVATION_STAGE.store(0, Ordering::Release);
    let result = negotiate(requested, granted);
    if result.status != STATUS_OK {
        return result;
    }
    if host.is_null()
        || unsafe { (*host).struct_size } != size_of::<NativeRuntimeHostV1>() as u32
        || !HOST
            .lock()
            .is_ok_and(|slot| slot.is_some_and(|binding| binding.matches(unsafe { &*host })))
    {
        return NativeNegotiationV1 {
            status: STATUS_INVALID_HOST,
            active_feature_bits: 0,
        };
    }
    #[cfg(all(windows, target_arch = "x86"))]
    if unsafe { platform::install() }.is_ok() {
        STOPPING.store(false, Ordering::Release);
        ACTIVE.store(result.active_feature_bits, Ordering::Release);
        return result;
    }
    NativeNegotiationV1 {
        status: STATUS_ACTIVATION_FAILED,
        active_feature_bits: 0,
    }
}
extern "C" fn deactivate() -> i32 {
    STOPPING.store(true, Ordering::Release);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while IN_FLIGHT.load(Ordering::Acquire) != 0 {
        if std::time::Instant::now() >= deadline {
            return STATUS_ACTIVATION_FAILED;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    ACTIVE.store(0, Ordering::Release);
    STATUS_OK
}
#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::inline_target(
            ADAPTER_ID,
            (1, 0, 0),
            FEATURES,
            PLATFORM_WINDOWS,
            ARCH_X86,
        ),
        negotiate_features: negotiate,
        activate,
        deactivate,
        request_refresh: request_refresh_noop,
    }
}
