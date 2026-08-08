#![cfg(windows)]

use glyphshift_adapter_native_abi::{
    NativeDecisionV1, NativeRuntimeHostV1, STATUS_ACTIVATION_FAILED, STATUS_OK,
};
use glyphshift_adapter_native_host::{LoadedNativeAdapter, NativeHostError};
use glyphshift_domain::Feature;
use std::ffi::c_void;
use std::path::PathBuf;

fn native_package() -> PathBuf {
    let executable = std::env::current_exe().expect("current test executable");
    let profile = executable
        .parent()
        .and_then(|deps| deps.parent())
        .expect("Cargo profile directory");
    profile.join("glyphshift_adapter_unity_mono_standard_ui_native.dll")
}

extern "C" fn decide(
    _context: *mut c_void,
    _source: *const u16,
    _source_len: u32,
    _text_out: *mut u16,
    _text_capacity: u32,
    _font_out: *mut u16,
    _font_capacity: u32,
) -> NativeDecisionV1 {
    NativeDecisionV1 {
        status: STATUS_OK,
        generation: 1,
        decision_bits: 0,
        text_len: 0,
        font_len: 0,
    }
}

extern "C" fn source_characters(_context: *mut c_void, _output: *mut u16, _capacity: u32) -> u32 {
    0
}

#[test]
fn unity_mono_standard_ui_package_loads_and_rejects_a_non_mono_process() {
    let package = unsafe {
        LoadedNativeAdapter::load(
            &native_package(),
            &glyphshift_adapter_unity_mono_standard_ui::descriptor(),
        )
        .expect("load verified Unity Mono Standard UI package")
    };
    let host = Box::leak(Box::new(NativeRuntimeHostV1 {
        struct_size: std::mem::size_of::<NativeRuntimeHostV1>() as u32,
        context: std::ptr::null_mut(),
        decide_utf16: decide,
        source_characters_utf16: source_characters,
    }));

    assert_eq!(
        package.activate(host, [Feature::TextReplace], [Feature::TextReplace],),
        Err(NativeHostError::PackageFailure(STATUS_ACTIVATION_FAILED))
    );
}
