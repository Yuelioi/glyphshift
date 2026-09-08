//! CatSystem2 MSVC x86 UTF-8 and classic ANSI retained text objects.
#[cfg(all(windows, target_arch = "x86"))]
mod platform;
pub mod shape;
#[cfg(any(test, all(windows, target_arch = "x86")))]
mod text;
use glyphshift_adapter_native_abi::*;
use std::sync::Mutex;

pub const ADAPTER_ID: &str = "windows.catsystem2.utf8-text";
const FEATURES: u64 = FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE;
static CONTROL: Mutex<()> = Mutex::new(());

extern "C" fn negotiate(requested: u64, granted: u64) -> NativeNegotiationV1 {
    let status = if requested & !FEATURES != 0 {
        STATUS_UNSUPPORTED_FEATURE
    } else if requested & !granted != 0 {
        STATUS_UNAUTHORIZED_FEATURE
    } else {
        STATUS_OK
    };
    NativeNegotiationV1 {
        status,
        active_feature_bits: if status == STATUS_OK { requested } else { 0 },
    }
}
extern "C" fn activate(
    host: *const NativeRuntimeHostV1,
    requested: u64,
    granted: u64,
) -> NativeNegotiationV1 {
    let result = negotiate(requested, granted);
    if result.status != STATUS_OK {
        return result;
    }
    let fail = |status| NativeNegotiationV1 {
        status,
        active_feature_bits: 0,
    };
    if host.is_null() || unsafe { (*host).struct_size } != size_of::<NativeRuntimeHostV1>() as u32 {
        return fail(STATUS_INVALID_HOST);
    }
    let Ok(_control) = CONTROL.lock() else {
        return fail(STATUS_ACTIVATION_FAILED);
    };
    #[cfg(all(windows, target_arch = "x86"))]
    if std::panic::catch_unwind(|| platform::activate(unsafe { *host }, requested))
        .is_ok_and(|r| r.is_ok())
    {
        return result;
    }
    fail(STATUS_ACTIVATION_FAILED)
}
extern "C" fn deactivate() -> i32 {
    let Ok(_control) = CONTROL.lock() else {
        return STATUS_ACTIVATION_FAILED;
    };
    #[cfg(all(windows, target_arch = "x86"))]
    if platform::deactivate().is_err() {
        return STATUS_ACTIVATION_FAILED;
    }
    STATUS_OK
}
extern "C" fn refresh() {
    #[cfg(all(windows, target_arch = "x86"))]
    platform::refresh();
}
#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::retained_target(
            ADAPTER_ID,
            (1, 0, 0),
            FEATURES,
            PLATFORM_WINDOWS,
            ARCH_X86,
        ),
        negotiate_features: negotiate,
        activate,
        deactivate,
        request_refresh: refresh,
    }
}
