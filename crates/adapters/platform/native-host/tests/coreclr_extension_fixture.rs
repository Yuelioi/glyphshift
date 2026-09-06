#![cfg(all(windows, target_arch = "x86_64"))]

use glyphshift_adapter_native_abi::{
    NativeDecisionV1, NativeRuntimeHostV1, DECISION_TEXT_REPLACE, STATUS_OK,
};
use glyphshift_adapter_native_host::{LoadedNativeAdapter, NativeHostError};
use glyphshift_adapter_sdk::{AdapterDescriptor, AdapterVersion};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement};
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static GENERATION: AtomicU64 = AtomicU64::new(1);
static OBSERVATIONS: AtomicU64 = AtomicU64::new(0);

#[link(name = "ole32")]
extern "system" {
    fn CoTaskMemFree(memory: *mut c_void);
}

extern "C" fn decide(
    _context: *mut c_void,
    source: *const u16,
    source_len: u32,
    output: *mut u16,
    capacity: u32,
    _font: *mut u16,
    _font_capacity: u32,
) -> NativeDecisionV1 {
    OBSERVATIONS.fetch_add(1, Ordering::Relaxed);
    let generation = GENERATION.load(Ordering::Acquire);
    let mut result = NativeDecisionV1 {
        status: STATUS_OK,
        generation,
        decision_bits: 0,
        text_len: 0,
        font_len: 0,
    };
    let source = unsafe { std::slice::from_raw_parts(source, source_len as usize) };
    if source != "Source text".encode_utf16().collect::<Vec<_>>() {
        return result;
    }
    let translation = if generation == 1 {
        "第一代译文"
    } else {
        "第二代译文"
    };
    let units = translation.encode_utf16().collect::<Vec<_>>();
    if units.len() <= capacity as usize {
        unsafe { std::ptr::copy_nonoverlapping(units.as_ptr(), output, units.len()) };
        result.decision_bits = DECISION_TEXT_REPLACE;
        result.text_len = units.len() as u32;
    }
    result
}

extern "C" fn characters(_context: *mut c_void, _output: *mut u16, _capacity: u32) -> u32 {
    0
}

#[test]
#[ignore = "requires the explicit synthetic CoreCLR fixture build; no real target attachment"]
fn coreclr_fixture_uses_authoritative_native_abi_and_loader() {
    let path = PathBuf::from(
        std::env::var_os("GLYPHSHIFT_CORECLR_FIXTURE_DLL")
            .expect("explicit fixture artifact supplied by its test runner"),
    );
    let expected = AdapterDescriptor::new(
        AdapterId::new("windows.coreclr.synthetic-text"),
        AdapterVersion::new(1, 0, 0),
        ApplyModel::InlineRender,
        Placement::TargetProcess,
        [Feature::TextObserve, Feature::TextReplace],
    )
    .with_platforms(["windows"])
    .with_architectures(["x86_64"]);
    let adapter =
        unsafe { LoadedNativeAdapter::load(&path, &expected) }.expect("V1 loader accepts fixture");
    assert_eq!(
        adapter.negotiate([Feature::TextReplace], [Feature::TextObserve]),
        Err(NativeHostError::UnauthorizedFeature)
    );
    let library = unsafe { libloading::Library::new(&path) }.expect("fixture library");
    let translate = unsafe {
        library.get::<unsafe extern "C" fn(*const u16, i32) -> *mut u16>(
            b"glyphshift_coreclr_translate\0",
        )
    }
    .expect("managed bridge export");
    let render = |text: &str| {
        let mut units = text.encode_utf16().collect::<Vec<_>>();
        let length = units.len() as i32;
        units.push(0);
        let output = unsafe { translate(units.as_ptr(), length) };
        if output.is_null() {
            return text.to_owned();
        }
        let mut result = Vec::new();
        for index in 0..=65536 {
            let unit = unsafe { *output.add(index) };
            if unit == 0 {
                break;
            }
            result.push(unit);
        }
        unsafe { CoTaskMemFree(output.cast()) };
        String::from_utf16(&result).expect("UTF-16 result")
    };
    let host = Box::leak(Box::new(NativeRuntimeHostV1 {
        struct_size: std::mem::size_of::<NativeRuntimeHostV1>() as u32,
        context: Box::into_raw(Box::new(0_u8)).cast(),
        decide_utf16: decide,
        source_characters_utf16: characters,
    }));
    let features = [Feature::TextObserve, Feature::TextReplace];
    assert_eq!(
        adapter.activate(host, features, features).unwrap(),
        features
    );
    assert_eq!(render("Source text"), "第一代译文");
    assert_eq!(render("Unmapped"), "Unmapped");
    GENERATION.store(2, Ordering::Release);
    adapter.request_refresh();
    assert_eq!(render("Source text"), "第二代译文");
    adapter.deactivate().unwrap();
    let stopped_count = OBSERVATIONS.load(Ordering::Relaxed);
    assert_eq!(render("Source text"), "Source text");
    assert_eq!(OBSERVATIONS.load(Ordering::Relaxed), stopped_count);
    // This test never attaches a CLR profiler. There are no rewritten methods or
    // profiler callbacks holding the fixture, so ordinary Library drops are safe.
}
