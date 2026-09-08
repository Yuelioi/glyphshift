//! VGUI complete localization queries and validated standard Label refresh.
pub mod panel_abi;
pub mod query_abi;
pub mod refresh_abi;

#[cfg(all(windows, target_arch = "x86"))]
mod memory;
#[cfg(all(windows, target_arch = "x86"))]
mod platform;
#[cfg(all(windows, target_arch = "x86"))]
mod refresh;
#[cfg(all(windows, target_arch = "x86"))]
mod returned_text;

use glyphshift_adapter_native_abi::*;
use std::sync::Mutex;

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
    let negotiated = negotiate(requested, granted);
    if negotiated.status != STATUS_OK {
        return negotiated;
    }
    let failure = |status| NativeNegotiationV1 {
        status,
        active_feature_bits: 0,
    };
    if host.is_null() || unsafe { (*host).struct_size } != size_of::<NativeRuntimeHostV1>() as u32 {
        return failure(STATUS_INVALID_HOST);
    }
    let Ok(_control) = CONTROL.lock() else {
        return failure(STATUS_ACTIVATION_FAILED);
    };
    #[cfg(all(windows, target_arch = "x86"))]
    if std::panic::catch_unwind(|| platform::activate(unsafe { *host }, requested))
        .is_ok_and(|result| result.is_ok())
    {
        return negotiated;
    }
    failure(STATUS_ACTIVATION_FAILED)
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

extern "C" fn request_refresh() {
    #[cfg(all(windows, target_arch = "x86"))]
    refresh::request();
}

#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::retained_target(
            "windows.vgui.localize-query",
            (1, 0, 0),
            FEATURES,
            PLATFORM_WINDOWS,
            ARCH_X86,
        ),
        negotiate_features: negotiate,
        activate,
        deactivate,
        request_refresh,
    }
}
