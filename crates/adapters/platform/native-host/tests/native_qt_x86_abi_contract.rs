#![cfg(windows)]
use glyphshift_adapter_native_abi::*;
use glyphshift_adapter_native_host::LoadedNativeAdapter;
use glyphshift_domain::Feature;
use std::{
    ffi::c_void,
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
    let text = if generation == 1 {
        "打开"
    } else {
        "再次打开"
    }
    .encode_utf16()
    .collect::<Vec<_>>();
    let replace = source == [79, 112, 101, 110] && text.len() <= capacity as usize;
    if replace {
        unsafe {
            std::ptr::copy_nonoverlapping(text.as_ptr(), output, text.len());
        }
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
#[ignore = "requires the opt-in MSVC synthetic Qt ABI fixture"]
fn qt5_msvc_members_replace_update_and_restore_in_both_architectures() {
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
        // Keep all libraries resident while the process contains their trampolines.
        let core = Box::leak(Box::new(
            libloading::Library::new(root.join("Qt5Core.dll")).unwrap(),
        ));
        let gui = Box::leak(Box::new(
            libloading::Library::new(root.join("Qt5Gui.dll")).unwrap(),
        ));
        let _ = core;
        let draw: libloading::Symbol<unsafe extern "C" fn(i32, *const u16, i32) -> *const u16> =
            gui.get(b"fixture_draw\0").unwrap();
        let render = |kind| {
            let text = "Open".encode_utf16().collect::<Vec<_>>();
            let result = draw(kind, text.as_ptr(), text.len() as i32);
            let length = (0..64)
                .find(|index| *result.add(*index) == 0)
                .expect("bounded result");
            String::from_utf16(std::slice::from_raw_parts(result, length)).unwrap()
        };
        let package = LoadedNativeAdapter::load(
            &profile.join("glyphshift_adapter_qt_painter_native.dll"),
            &glyphshift_adapter_qt_painter::descriptor(),
        )
        .unwrap();
        let host = Box::leak(Box::new(NativeRuntimeHostV1 {
            struct_size: size_of::<NativeRuntimeHostV1>() as u32,
            context: std::ptr::null_mut(),
            decide_utf16: decide,
            source_characters_utf16: characters,
        }));
        let extended = Box::leak(Box::new(glyphshift_adapter_native_abi::NativeTextHostV1 {
            struct_size: size_of::<glyphshift_adapter_native_abi::NativeTextHostV1>() as u32,
            version: glyphshift_adapter_native_abi::TEXT_HOST_VERSION_V1,
            context: host.context,
            decide_text: structured_decide,
            enter_scope: None,
            leave_scope: None,
        }));
        for kind in 0..4 {
            assert_eq!(render(kind), "Open");
        }
        for generation in 1..=3 {
            GENERATION.store(generation, Ordering::Release);
            package
                .activate_with_text_host(
                    host,
                    if generation == 2 {
                        Some(extended)
                    } else {
                        None
                    },
                    [Feature::TextObserve, Feature::TextReplace],
                    [Feature::TextObserve, Feature::TextReplace],
                )
                .unwrap();
            for _ in 0..100 {
                for kind in 0..4 {
                    assert_eq!(
                        render(kind),
                        if generation == 1 {
                            "打开"
                        } else {
                            "再次打开"
                        }
                    );
                }
            }
            assert_eq!(
                EXTENDED_CALLS.load(Ordering::Acquire),
                if generation == 1 { 0 } else { 400 }
            );
            package.deactivate().unwrap();
            for kind in 0..4 {
                assert_eq!(render(kind), "Open");
            }
        }
    }
}

static EXTENDED_CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
extern "C" fn structured_decide(
    context: *mut core::ffi::c_void,
    event: *const glyphshift_adapter_native_abi::NativeTextEventV1,
    text: *mut u16,
    text_capacity: u32,
    font: *mut u16,
    font_capacity: u32,
) -> NativeDecisionV1 {
    let event = unsafe { &*event };
    assert_eq!(event.kind, glyphshift_adapter_native_abi::TEXT_EVENT_DRAW);
    assert_eq!(
        event.struct_size,
        size_of::<glyphshift_adapter_native_abi::NativeTextEventV1>() as u32
    );
    EXTENDED_CALLS.fetch_add(1, Ordering::AcqRel);
    decide(
        context,
        event.source,
        event.source_len,
        text,
        text_capacity,
        font,
        font_capacity,
    )
}
