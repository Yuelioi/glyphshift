#![cfg(windows)]

use glyphshift_adapter_native_abi::{
    NativeDecisionV1, NativeRuntimeHostV1, NativeTextEventV1, NativeTextEventV2, NativeTextHostV1,
    DECISION_TEXT_REPLACE, STATUS_OK, TEXT_EVENT_METADATA_CONTEXT, TEXT_EVENT_METADATA_VERSION_V1,
    TEXT_HOST_VERSION_V1,
};
use glyphshift_adapter_native_host::LoadedNativeAdapter;
use glyphshift_domain::Feature;
use std::ffi::{c_char, c_void, CString};
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use windows::core::{PCSTR, PCWSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LoadLibraryExW, LOAD_WITH_ALTERED_SEARCH_PATH,
};

const TRANSLATE_SYMBOL: &[u8] = b"?translate@QCoreApplication@@SA?AVQString@@PEBD00H@Z\0";
const QSTRING_DTOR_SYMBOL: &[u8] = b"??1QString@@QEAA@XZ\0";
const QSTRING_EQUALS_UTF8_SYMBOL: &[u8] = b"??8QString@@QEBA_NPEBD@Z\0";

type RawProc = unsafe extern "system" fn() -> isize;
type Translate = unsafe extern "system" fn(
    result: *mut c_void,
    context: *const c_char,
    source_text: *const c_char,
    disambiguation: *const c_char,
    n: i32,
) -> *mut c_void;
type QStringDtor = unsafe extern "system" fn(*mut c_void);
type QStringEqualsUtf8 = unsafe extern "system" fn(*const c_void, *const c_char) -> bool;

#[repr(C, align(16))]
struct OpaqueQString([u8; 64]);

static MODE: AtomicU8 = AtomicU8::new(1);
static TEXT_HOST_CALLS: AtomicUsize = AtomicUsize::new(0);
static CONTEXT_MATCHES: AtomicUsize = AtomicUsize::new(0);

fn native_package() -> PathBuf {
    let executable = std::env::current_exe().expect("current test executable");
    let profile = executable
        .parent()
        .and_then(|deps| deps.parent())
        .expect("Cargo profile directory");
    profile.join("glyphshift_adapter_qt_translation_native.dll")
}

unsafe fn load_module(path: &Path) -> HMODULE {
    let wide = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    LoadLibraryExW(PCWSTR(wide.as_ptr()), None, LOAD_WITH_ALTERED_SEARCH_PATH)
        .expect("load configured Qt Core module")
}

unsafe fn resolve(module: HMODULE, symbol: &[u8]) -> RawProc {
    std::mem::transmute(
        GetProcAddress(module, PCSTR(symbol.as_ptr())).expect("resolve required Qt export"),
    )
}

extern "C" fn decide(
    _context: *mut c_void,
    source: *const u16,
    source_len: u32,
    text_out: *mut u16,
    text_capacity: u32,
    _font_out: *mut u16,
    _font_capacity: u32,
) -> NativeDecisionV1 {
    let source = unsafe { std::slice::from_raw_parts(source, source_len as usize) };
    let source = String::from_utf16(source).expect("Adapter must provide valid UTF-16");
    let replacement = match MODE.load(Ordering::Acquire) {
        1 if source == "Open" => "First translation",
        2 if source == "Open" => "Second translation",
        3 if source == "Profile: %1" => "配置：%1",
        4 if source == "Profile: %1" => "配置",
        _ => "",
    }
    .encode_utf16()
    .collect::<Vec<_>>();
    assert!(replacement.len() <= text_capacity as usize);
    unsafe {
        std::ptr::copy_nonoverlapping(replacement.as_ptr(), text_out, replacement.len());
    }
    NativeDecisionV1 {
        status: STATUS_OK,
        generation: u64::from(MODE.load(Ordering::Acquire)),
        decision_bits: if replacement.is_empty() {
            0
        } else {
            DECISION_TEXT_REPLACE
        },
        text_len: replacement.len() as u32,
        font_len: 0,
    }
}

extern "C" fn source_characters(_context: *mut c_void, _output: *mut u16, _capacity: u32) -> u32 {
    0
}

extern "C" fn decide_text(
    _context: *mut c_void,
    event: *const NativeTextEventV1,
    text_out: *mut u16,
    text_capacity: u32,
    _font_out: *mut u16,
    _font_capacity: u32,
) -> NativeDecisionV1 {
    let event = unsafe { &*event };
    TEXT_HOST_CALLS.fetch_add(1, Ordering::AcqRel);
    if event.struct_size == std::mem::size_of::<NativeTextEventV2>() as u32 {
        let event = unsafe { &*(event as *const NativeTextEventV1).cast::<NativeTextEventV2>() };
        if event.metadata_version == TEXT_EVENT_METADATA_VERSION_V1
            && event.metadata_flags & TEXT_EVENT_METADATA_CONTEXT != 0
        {
            let context =
                unsafe { std::slice::from_raw_parts(event.context, event.context_len as usize) };
            if String::from_utf16_lossy(context) == "Contract" {
                CONTEXT_MATCHES.fetch_add(1, Ordering::AcqRel);
            }
        }
    }
    let source = unsafe { std::slice::from_raw_parts(event.source, event.source_len as usize) };
    let source = String::from_utf16_lossy(source);
    let replacement = match MODE.load(Ordering::Acquire) {
        1 if source == "Open" => "First translation",
        2 if source == "Open" => "Second translation",
        3 if source == "Profile: %1" => "配置：%1",
        4 if source == "Profile: %1" => "配置",
        _ => "",
    }
    .encode_utf16()
    .collect::<Vec<_>>();
    if replacement.len() <= text_capacity as usize {
        unsafe {
            std::ptr::copy_nonoverlapping(replacement.as_ptr(), text_out, replacement.len());
        }
    }
    NativeDecisionV1 {
        status: STATUS_OK,
        generation: u64::from(MODE.load(Ordering::Acquire)),
        decision_bits: if replacement.is_empty() {
            0
        } else {
            DECISION_TEXT_REPLACE
        },
        text_len: replacement.len() as u32,
        font_len: 0,
    }
}

fn host() -> &'static NativeRuntimeHostV1 {
    Box::leak(Box::new(NativeRuntimeHostV1 {
        struct_size: std::mem::size_of::<NativeRuntimeHostV1>() as u32,
        context: std::ptr::null_mut(),
        decide_utf16: decide,
        source_characters_utf16: source_characters,
    }))
}

fn text_host(host: &'static NativeRuntimeHostV1) -> &'static NativeTextHostV1 {
    Box::leak(Box::new(NativeTextHostV1 {
        struct_size: std::mem::size_of::<NativeTextHostV1>() as u32,
        version: TEXT_HOST_VERSION_V1,
        context: host.context,
        decide_text,
        enter_scope: None,
        leave_scope: None,
    }))
}

unsafe fn translated_equals(
    translate: Translate,
    destroy: QStringDtor,
    equals: QStringEqualsUtf8,
    context: &str,
    source: &str,
    expected: &str,
) -> bool {
    let context = CString::new(context).unwrap();
    let source = CString::new(source).unwrap();
    let expected = CString::new(expected).unwrap();
    let mut result = MaybeUninit::<OpaqueQString>::uninit();
    let result = result.as_mut_ptr().cast::<c_void>();
    translate(
        result,
        context.as_ptr(),
        source.as_ptr(),
        std::ptr::null(),
        -1,
    );
    let matches = equals(result, expected.as_ptr());
    destroy(result);
    matches
}

#[test]
#[ignore = "requires GLYPHSHIFT_QT_TRANSLATION_CORE_DLL pointing at a verified Qt 6.10.3 MSVC x64 module"]
fn qt_translation_real_qt_module_preserves_placeholders_and_recovers_after_disable() {
    let core_path = PathBuf::from(
        std::env::var_os("GLYPHSHIFT_QT_TRANSLATION_CORE_DLL")
            .expect("configured Qt translation Core module"),
    );
    let core = unsafe { load_module(&core_path) };
    let translate =
        unsafe { std::mem::transmute::<RawProc, Translate>(resolve(core, TRANSLATE_SYMBOL)) };
    let destroy =
        unsafe { std::mem::transmute::<RawProc, QStringDtor>(resolve(core, QSTRING_DTOR_SYMBOL)) };
    let equals = unsafe {
        std::mem::transmute::<RawProc, QStringEqualsUtf8>(resolve(core, QSTRING_EQUALS_UTF8_SYMBOL))
    };

    let package = unsafe {
        LoadedNativeAdapter::load(
            &native_package(),
            &glyphshift_adapter_qt_translation::descriptor(),
        )
        .expect("load verified Qt translation package")
    };
    let host = host();
    package
        .activate_with_text_host(
            host,
            Some(text_host(host)),
            [Feature::TextObserve, Feature::TextReplace],
            [Feature::TextObserve, Feature::TextReplace],
        )
        .expect("activate Qt translation adapter");

    TEXT_HOST_CALLS.store(0, Ordering::Release);
    CONTEXT_MATCHES.store(0, Ordering::Release);
    MODE.store(1, Ordering::Release);
    assert!(unsafe {
        translated_equals(
            translate,
            destroy,
            equals,
            "Contract",
            "Open",
            "First translation",
        )
    });
    MODE.store(2, Ordering::Release);
    assert!(unsafe {
        translated_equals(
            translate,
            destroy,
            equals,
            "Contract",
            "Open",
            "Second translation",
        )
    });
    assert!(TEXT_HOST_CALLS.load(Ordering::Acquire) >= 2);
    assert!(CONTEXT_MATCHES.load(Ordering::Acquire) >= 2);

    MODE.store(3, Ordering::Release);
    assert!(unsafe {
        translated_equals(
            translate,
            destroy,
            equals,
            "MainStatusBar",
            "Profile: %1",
            "配置：%1",
        )
    });
    MODE.store(4, Ordering::Release);
    assert!(unsafe {
        translated_equals(
            translate,
            destroy,
            equals,
            "MainStatusBar",
            "Profile: %1",
            "Profile: %1",
        )
    });

    package
        .deactivate()
        .expect("deactivate Qt translation adapter");
    assert!(unsafe { translated_equals(translate, destroy, equals, "Contract", "Open", "Open") });
}
