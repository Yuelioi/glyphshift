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
#[ignore = "requires the opt-in MSVC C framework ABI fixtures"]
fn gtk_and_raylib_c_abis_replace_update_and_restore() {
    let root =
        PathBuf::from(std::env::var_os("GLYPHSHIFT_FRAMEWORK_ABI_ROOT").expect("fixture root"));
    let profile = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    // The fake raylib consumes synthetic bytes, not a machine font.
    std::env::set_var("GLYPHSHIFT_RAYLIB_FALLBACK_FONT", root.join("raylib.dll"));
    for (fixture, dll, descriptor) in [
        (
            "gtk3-fixture.dll",
            "glyphshift_adapter_gtk3_pango_native.dll",
            glyphshift_adapter_gtk3_pango::descriptor(),
        ),
        (
            "raylib.dll",
            "glyphshift_adapter_raylib_native.dll",
            glyphshift_adapter_raylib::descriptor(),
        ),
    ] {
        unsafe {
            let library = Box::leak(Box::new(
                libloading::Library::new(root.join(fixture)).unwrap(),
            ));
            let draw: libloading::Symbol<unsafe extern "C" fn(i32) -> *const std::ffi::c_char> =
                library.get(b"fixture_draw\0").unwrap();
            let render = |partial| {
                std::ffi::CStr::from_ptr(draw(partial))
                    .to_str()
                    .unwrap()
                    .to_owned()
            };
            let package = LoadedNativeAdapter::load(&profile.join(dll), &descriptor).unwrap();
            let host = Box::leak(Box::new(NativeRuntimeHostV1 {
                struct_size: size_of::<NativeRuntimeHostV1>() as u32,
                context: std::ptr::null_mut(),
                decide_utf16: decide,
                source_characters_utf16: characters,
            }));
            assert_eq!(render(0), "Open");
            for generation in 1..=2 {
                GENERATION.store(generation, Ordering::Release);
                package
                    .activate(
                        host,
                        [Feature::TextObserve, Feature::TextReplace],
                        [Feature::TextObserve, Feature::TextReplace],
                    )
                    .unwrap();
                for _ in 0..100 {
                    assert_eq!(
                        render(0),
                        if generation == 1 {
                            "打开"
                        } else {
                            "再次打开"
                        }
                    );
                }
                if fixture.starts_with("gtk") {
                    assert_eq!(render(1), "Open");
                }
                package.deactivate().unwrap();
                assert_eq!(render(0), "Open");
                if fixture == "raylib.dll" {
                    let fonts: libloading::Symbol<unsafe extern "C" fn() -> i32> =
                        library.get(b"fixture_fonts\0").unwrap();
                    assert_eq!(
                        fonts(),
                        0,
                        "retired font allocations reclaimed at frame boundary"
                    );
                }
            }
        }
    }
    std::env::remove_var("GLYPHSHIFT_RAYLIB_FALLBACK_FONT");
}
