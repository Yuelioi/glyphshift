#![cfg(all(windows, target_arch = "x86_64"))]

use glyphshift_adapter_native_abi::*;
use glyphshift_adapter_native_host::LoadedNativeAdapter;
use glyphshift_domain::Feature;
use std::{
    ffi::{c_char, c_void, CStr},
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};

static GENERATION: AtomicU32 = AtomicU32::new(1);

extern "C" fn decide(
    _: *mut c_void,
    source: *const u16,
    length: u32,
    output: *mut u16,
    capacity: u32,
    _: *mut u16,
    _: u32,
) -> NativeDecisionV1 {
    let source = unsafe { std::slice::from_raw_parts(source, length as usize) };
    let generation = GENERATION.load(Ordering::Acquire);
    let translation = if generation == 1 {
        "打开"
    } else {
        "再次打开"
    };
    let text = translation.encode_utf16().collect::<Vec<_>>();
    let open = "Open".encode_utf16().collect::<Vec<_>>();
    let replace = source == open && text.len() <= capacity as usize;
    if replace {
        unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), output, text.len()) };
    }
    NativeDecisionV1 {
        status: STATUS_OK,
        generation: generation.into(),
        decision_bits: if replace { DECISION_TEXT_REPLACE } else { 0 },
        text_len: if replace { text.len() as u32 } else { 0 },
        font_len: 0,
    }
}

extern "C" fn characters(_: *mut c_void, _: *mut u16, _: u32) -> u32 {
    0
}

#[test]
#[ignore = "requires the opt-in MSVC SideFX CV ABI fixture"]
fn sidefx_cv_text_wrapped_in_box_replaces_updates_and_restores() {
    let root =
        PathBuf::from(std::env::var_os("GLYPHSHIFT_FRAMEWORK_ABI_ROOT").expect("fixture root"));
    let profile = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    unsafe {
        let library = Box::leak(Box::new(
            libloading::Library::new(root.join("libCV.dll")).expect("load synthetic libCV"),
        ));
        let draw: libloading::Symbol<unsafe extern "C" fn() -> *const c_char> =
            library.get(b"fixture_draw\0").unwrap();
        let render = || {
            CStr::from_ptr(draw())
                .to_str()
                .expect("fixture UTF-8")
                .to_owned()
        };
        let package = LoadedNativeAdapter::load(
            &profile.join("glyphshift_adapter_sidefx_cv_paint_buffer_native.dll"),
            &glyphshift_adapter_sidefx_cv_paint_buffer::descriptor(),
        )
        .expect("load SideFX CV package");
        let host = Box::leak(Box::new(NativeRuntimeHostV1 {
            struct_size: size_of::<NativeRuntimeHostV1>() as u32,
            context: std::ptr::null_mut(),
            decide_utf16: decide,
            source_characters_utf16: characters,
        }));

        assert_eq!(render(), "Open");
        for generation in 1..=2 {
            GENERATION.store(generation, Ordering::Release);
            package
                .activate(
                    host,
                    [Feature::TextObserve, Feature::TextReplace],
                    [Feature::TextObserve, Feature::TextReplace],
                )
                .expect("activate SideFX CV Adapter");
            assert_eq!(
                render(),
                if generation == 1 {
                    "打开"
                } else {
                    "再次打开"
                }
            );
            package.deactivate().expect("deactivate SideFX CV Adapter");
            assert_eq!(render(), "Open");
        }
    }
}
