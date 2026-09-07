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
#[ignore = "requires the opt-in synthetic Mono export and dispatch fixture"]
fn mono_native_exports_dispatch_replacement_and_restore_in_both_architectures() {
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
            libloading::Library::new(root.join("mono-2.0-bdwgc.dll")).unwrap(),
        ));
        let dispatch = *library
            .get::<unsafe extern "C" fn(*mut c_void)>(b"fixture_dispatch\0")
            .unwrap();
        let text = *library
            .get::<unsafe extern "C" fn(*mut u16, i32) -> i32>(b"fixture_text\0")
            .unwrap();
        let read = || {
            let mut output = [0; 64];
            let length = text(output.as_mut_ptr(), 64);
            assert!(length >= 0);
            String::from_utf16(&output[..length as usize]).unwrap()
        };
        struct Pump(
            std::sync::Arc<std::sync::atomic::AtomicBool>,
            Option<std::thread::JoinHandle<()>>,
        );
        impl Drop for Pump {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
                self.1.take().unwrap().join().unwrap();
            }
        }
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let active = running.clone();
        let _pump = Pump(
            running,
            Some(std::thread::spawn(move || {
                while active.load(Ordering::Acquire) {
                    dispatch(std::ptr::null_mut());
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            })),
        );
        let package = LoadedNativeAdapter::load(
            &profile.join("glyphshift_adapter_unity_mono_standard_ui_native.dll"),
            &glyphshift_adapter_unity_mono_standard_ui::descriptor(),
        )
        .unwrap();
        let host = Box::leak(Box::new(NativeRuntimeHostV1 {
            struct_size: size_of::<NativeRuntimeHostV1>() as u32,
            context: std::ptr::null_mut(),
            decide_utf16: decide,
            source_characters_utf16: characters,
        }));
        assert_eq!(read(), "Open");
        for generation in 1..=2 {
            GENERATION.store(generation, Ordering::Release);
            package
                .activate(
                    host,
                    [Feature::TextObserve, Feature::TextReplace],
                    [Feature::TextObserve, Feature::TextReplace],
                )
                .unwrap();
            let expected = if generation == 1 {
                "打开"
            } else {
                "再次打开"
            };
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            while read() != expected && std::time::Instant::now() < deadline {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            assert_eq!(read(), expected);
            package.deactivate().unwrap();
            assert_eq!(read(), "Open");
        }
    }
}
