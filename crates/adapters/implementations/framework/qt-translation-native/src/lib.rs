//! Qt 6.10.3 MSVC x64 `QCoreApplication::translate` adapter.
//!
//! MSVC x64 lowers the `QString` return value to a hidden first argument. The verified ABI is:
//! `translate(QString* result, context, sourceText, disambiguation, n) -> QString*`.
//! Replacement is performed by substituting only the source pointer passed to the original Qt
//! function. The detour never mutates a `QString` and calls the original exactly once.
//! Exact Qt 5 research profiles remain available only behind the explicit `research-qt5` Cargo
//! feature so normal production builds cannot expose an unpromoted Qt 5 profile.

use glyphshift_adapter_native_abi::{
    DecideUtf16V1, NativeAdapterApiV1, NativeAdapterDescriptorV1, NativeDecisionV1,
    NativeNegotiationV1, NativeRuntimeHostV1, NativeTextEventV1, NativeTextEventV2,
    NativeTextHostBinding, NativeTextHostV1, ARCH_X86_64, DECISION_TEXT_REPLACE,
    FEATURE_TEXT_OBSERVE, FEATURE_TEXT_REPLACE, MAX_TEXT_EVENT_CONTEXT_UNITS, PLATFORM_WINDOWS,
    STATUS_ACTIVATION_FAILED, STATUS_INVALID_HOST, STATUS_OK, STATUS_UNAUTHORIZED_FEATURE,
    STATUS_UNSUPPORTED_FEATURE, TEXT_EVENT_METADATA_CONTEXT, TEXT_EVENT_METADATA_DISAMBIGUATION,
    TEXT_EVENT_METADATA_PLURAL_N, TEXT_EVENT_METADATA_VERSION_V1, TEXT_EVENT_OBSERVE,
    TEXT_EVENT_RETAINED,
};
use glyphshift_adapter_qt_translation::ADAPTER_ID;
use retour::GenericDetour;
use std::cell::Cell;
use std::ffi::{c_char, c_void, CStr, CString};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};
use windows::core::{s, w, PCSTR};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

#[cfg(feature = "research-qt5")]
const SUPPORTED_QT5_VERSIONS: &[&[u8]] = &[b"5.15.13", b"5.15.18"];
const SUPPORTED_QT6_VERSION: &[u8] = b"6.10.3";
const MAX_TEXT_UNITS: usize = 16 * 1024;
const MAX_PARAMETER_BYTES: usize = 64 * 1024;
const TRANSLATE_SYMBOL: &[u8] = b"?translate@QCoreApplication@@SA?AVQString@@PEBD00H@Z\0";
const QCORE_INSTANCE_SYMBOL: &[u8] = b"?instance@QCoreApplication@@SAPEAV1@XZ\0";
const QCORE_POST_EVENT_SYMBOL: &[u8] =
    b"?postEvent@QCoreApplication@@SAXPEAVQObject@@PEAVQEvent@@H@Z\0";
const QEVENT_CTOR_SYMBOL: &[u8] = b"??0QEvent@@QEAA@W4Type@0@@Z\0";
const QEVENT_DTOR_SYMBOL: &[u8] = b"??1QEvent@@UEAA@XZ\0";
const QEVENT_CLONE_SYMBOL: &[u8] = b"?clone@QEvent@@UEBAPEAV1@XZ\0";
#[cfg(feature = "research-qt5")]
const QMALLOC_SYMBOL: &[u8] = b"?qMalloc@@YAPEAX_K@Z\0";
const QEVENT_LANGUAGE_CHANGE: i32 = 89;

type QVersion = unsafe extern "system" fn() -> *const c_char;
type Translate = unsafe extern "system" fn(
    result: *mut c_void,
    context: *const c_char,
    source_text: *const c_char,
    disambiguation: *const c_char,
    n: i32,
) -> *mut c_void;
type QCoreInstance = unsafe extern "system" fn() -> *mut c_void;
type QCorePostEvent = unsafe extern "system" fn(*mut c_void, *mut c_void, i32);
type QEventCtor = unsafe extern "system" fn(*mut c_void, i32) -> *mut c_void;
type QEventDtor = unsafe extern "system" fn(*mut c_void);
type QEventClone = unsafe extern "system" fn(*const c_void) -> *mut c_void;
#[cfg(feature = "research-qt5")]
type QMalloc = unsafe extern "system" fn(usize) -> *mut c_void;

#[repr(C, align(16))]
struct OpaqueQEvent([u8; 64]);

#[derive(Clone, Copy)]
struct HostBridge {
    context: usize,
    decide_utf16: DecideUtf16V1,
    text_host: Option<NativeTextHostBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TranslationCall {
    context: Option<String>,
    source: String,
    disambiguation: Option<String>,
    n: i32,
}

struct EncodedTranslationEvent {
    event: NativeTextEventV2,
    _source: Vec<u16>,
    _context: Option<Vec<u16>>,
    _disambiguation: Option<Vec<u16>>,
}

struct DecisionBuffers {
    decision: NativeDecisionV1,
    text: Vec<u16>,
}

#[derive(Clone, Copy)]
enum RefreshBridge {
    CloneOwned {
        instance: QCoreInstance,
        post_event: QCorePostEvent,
        event_ctor: QEventCtor,
        event_dtor: QEventDtor,
        event_clone: QEventClone,
    },
    #[cfg(feature = "research-qt5")]
    Qt5MallocOwned {
        instance: QCoreInstance,
        post_event: QCorePostEvent,
        event_ctor: QEventCtor,
        q_malloc: QMalloc,
    },
}

impl RefreshBridge {
    unsafe fn post_language_change(self) {
        match self {
            Self::CloneOwned {
                instance,
                post_event,
                event_ctor,
                event_dtor,
                event_clone,
            } => {
                let receiver = instance();
                if receiver.is_null() {
                    return;
                }
                let mut event = MaybeUninit::<OpaqueQEvent>::uninit();
                let event = event.as_mut_ptr().cast::<c_void>();
                event_ctor(event, QEVENT_LANGUAGE_CHANGE);
                let owned = event_clone(event);
                event_dtor(event);
                if !owned.is_null() {
                    post_event(receiver, owned, 0);
                }
            }
            #[cfg(feature = "research-qt5")]
            Self::Qt5MallocOwned {
                instance,
                post_event,
                event_ctor,
                q_malloc,
            } => {
                let receiver = instance();
                if receiver.is_null() {
                    return;
                }
                let event = q_malloc(std::mem::size_of::<OpaqueQEvent>());
                if event.is_null() {
                    return;
                }
                event_ctor(event, QEVENT_LANGUAGE_CHANGE);
                post_event(receiver, event, 0);
            }
        }
    }
}

static ACTIVE_FEATURES: AtomicU64 = AtomicU64::new(0);
static HOST: OnceLock<RwLock<HostBridge>> = OnceLock::new();
static HOOK: OnceLock<GenericDetour<Translate>> = OnceLock::new();
static REFRESH: OnceLock<RefreshBridge> = OnceLock::new();
static TEXT_HOST: Mutex<Option<NativeTextHostBinding>> = Mutex::new(None);

thread_local! {
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
}

struct CallbackGuard;

impl CallbackGuard {
    fn enter() -> Option<Self> {
        IN_CALLBACK.with(|active| (!active.replace(true)).then_some(Self))
    }
}

impl Drop for CallbackGuard {
    fn drop(&mut self) {
        IN_CALLBACK.with(|active| active.set(false));
    }
}

fn supported_qt_profile(major: u8, version: &[u8]) -> bool {
    if major == 6 && version == SUPPORTED_QT6_VERSION {
        return true;
    }
    #[cfg(feature = "research-qt5")]
    if major == 5
        && SUPPORTED_QT5_VERSIONS
            .iter()
            .any(|supported| version == *supported)
    {
        return true;
    }
    false
}

fn supports_immediate_refresh(major: u8, version: &[u8]) -> bool {
    if major == 6 && version == SUPPORTED_QT6_VERSION {
        return true;
    }
    #[cfg(feature = "research-qt5")]
    if major == 5 && version == b"5.15.13" {
        return true;
    }
    false
}

fn negotiation_error(status: i32) -> NativeNegotiationV1 {
    NativeNegotiationV1 {
        status,
        active_feature_bits: 0,
    }
}

extern "C" fn negotiate_features(requested: u64, granted: u64) -> NativeNegotiationV1 {
    let supported = FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE;
    if requested & !supported != 0 {
        return negotiation_error(STATUS_UNSUPPORTED_FEATURE);
    }
    if requested & !granted != 0 {
        return negotiation_error(STATUS_UNAUTHORIZED_FEATURE);
    }
    NativeNegotiationV1 {
        status: STATUS_OK,
        active_feature_bits: requested,
    }
}

unsafe fn read_required_utf8(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let bytes = CStr::from_ptr(value).to_bytes();
    if bytes.len() > MAX_PARAMETER_BYTES {
        return None;
    }
    std::str::from_utf8(bytes).ok().map(ToOwned::to_owned)
}

unsafe fn read_optional_utf8(value: *const c_char) -> Option<Option<String>> {
    if value.is_null() {
        return Some(None);
    }
    read_required_utf8(value).map(Some)
}

unsafe fn translation_call(
    context: *const c_char,
    source_text: *const c_char,
    disambiguation: *const c_char,
    n: i32,
) -> Option<TranslationCall> {
    let call = TranslationCall {
        context: read_optional_utf8(context)?,
        source: read_required_utf8(source_text)?,
        disambiguation: read_optional_utf8(disambiguation)?,
        n,
    };
    let units = call.source.encode_utf16().count();
    let metadata_bounded = call
        .context
        .as_ref()
        .is_none_or(|value| value.encode_utf16().count() <= MAX_TEXT_EVENT_CONTEXT_UNITS)
        && call
            .disambiguation
            .as_ref()
            .is_none_or(|value| value.encode_utf16().count() <= MAX_TEXT_EVENT_CONTEXT_UNITS);
    (units != 0 && units <= MAX_TEXT_UNITS && !call.source.trim().is_empty() && metadata_bounded)
        .then_some(call)
}

fn encode_translation_event(call: &TranslationCall, kind: u32) -> EncodedTranslationEvent {
    let source = call.source.encode_utf16().collect::<Vec<_>>();
    let context = call
        .context
        .as_ref()
        .map(|value| value.encode_utf16().collect::<Vec<_>>());
    let disambiguation = call
        .disambiguation
        .as_ref()
        .map(|value| value.encode_utf16().collect::<Vec<_>>());
    let mut metadata_flags = 0;
    if context.is_some() {
        metadata_flags |= TEXT_EVENT_METADATA_CONTEXT;
    }
    if disambiguation.is_some() {
        metadata_flags |= TEXT_EVENT_METADATA_DISAMBIGUATION;
    }
    if call.n >= 0 {
        metadata_flags |= TEXT_EVENT_METADATA_PLURAL_N;
    }
    let event = NativeTextEventV2 {
        base: NativeTextEventV1 {
            struct_size: std::mem::size_of::<NativeTextEventV2>() as u32,
            kind,
            surface: 0,
            run: 0,
            epoch: 0,
            ordinal: 0,
            source: source.as_ptr(),
            source_len: source.len() as u32,
        },
        metadata_version: TEXT_EVENT_METADATA_VERSION_V1,
        metadata_flags,
        context: context
            .as_ref()
            .map_or(std::ptr::null(), |value| value.as_ptr()),
        context_len: context.as_ref().map_or(0, |value| value.len() as u32),
        disambiguation: disambiguation
            .as_ref()
            .map_or(std::ptr::null(), |value| value.as_ptr()),
        disambiguation_len: disambiguation
            .as_ref()
            .map_or(0, |value| value.len() as u32),
        plural_n: if call.n >= 0 { call.n } else { -1 },
        reserved: 0,
    };
    EncodedTranslationEvent {
        event,
        _source: source,
        _context: context,
        _disambiguation: disambiguation,
    }
}

fn observe(call: &TranslationCall) {
    let Some(host) = HOST
        .get()
        .and_then(|host| host.read().ok())
        .map(|host| *host)
    else {
        return;
    };
    let mut text = vec![0_u16; MAX_TEXT_UNITS];
    let mut font = [];
    if let Some(text_host) = host.text_host {
        let event = encode_translation_event(call, TEXT_EVENT_OBSERVE);
        let _ = text_host.decide_v2(&event.event, &mut text, &mut font);
    } else {
        let source = call.source.encode_utf16().collect::<Vec<_>>();
        let _ = (host.decide_utf16)(
            host.context as *mut c_void,
            source.as_ptr(),
            source.len() as u32,
            text.as_mut_ptr(),
            text.len() as u32,
            font.as_mut_ptr(),
            0,
        );
    }
}

fn legacy_source_only_replacement_safe(call: &TranslationCall) -> bool {
    call.n == -1
}

fn decide(call: &TranslationCall) -> Option<DecisionBuffers> {
    let host = HOST.get()?.read().ok().map(|host| *host)?;
    let mut text = vec![0_u16; MAX_TEXT_UNITS];
    let mut font = [];
    let decision = if let Some(text_host) = host.text_host {
        let event = encode_translation_event(call, TEXT_EVENT_RETAINED);
        text_host.decide_v2(&event.event, &mut text, &mut font)
    } else {
        // Dictionary identity is source-only, so context/disambiguation do not block a legacy
        // source lookup. Plural calls still fail open because they need an explicit plural model.
        if !legacy_source_only_replacement_safe(call) {
            observe(call);
            return None;
        }
        let source = call.source.encode_utf16().collect::<Vec<_>>();
        (host.decide_utf16)(
            host.context as *mut c_void,
            source.as_ptr(),
            source.len() as u32,
            text.as_mut_ptr(),
            text.len() as u32,
            font.as_mut_ptr(),
            0,
        )
    };
    if decision.status != STATUS_OK || decision.text_len as usize > text.len() {
        return None;
    }
    text.truncate(decision.text_len as usize);
    Some(DecisionBuffers { decision, text })
}

fn replacement_for(call: &TranslationCall) -> Option<CString> {
    let decision = decide(call)?;
    if ACTIVE_FEATURES.load(Ordering::Acquire) & FEATURE_TEXT_REPLACE == 0
        || decision.decision.decision_bits & DECISION_TEXT_REPLACE == 0
        || decision.text.is_empty()
    {
        return None;
    }
    let replacement = String::from_utf16(&decision.text).ok()?;
    (!replacement.is_empty() && qt_placeholders_compatible(&call.source, &replacement))
        .then(|| CString::new(replacement).ok())
        .flatten()
}

fn qt_placeholders_compatible(source: &str, replacement: &str) -> bool {
    fn placeholders(value: &str) -> Vec<&str> {
        let bytes = value.as_bytes();
        let mut placeholders = Vec::new();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] != b'%' || index + 1 >= bytes.len() {
                index += 1;
                continue;
            }
            if bytes[index + 1] == b'%' {
                index += 2;
                continue;
            }
            let start = index;
            index += 1;
            if index < bytes.len() && bytes[index] == b'L' {
                index += 1;
            }
            if index < bytes.len() && bytes[index] == b'n' {
                index += 1;
                placeholders.push(&value[start..index]);
                continue;
            }
            if index < bytes.len() && matches!(bytes[index], b'1'..=b'9') {
                index += 1;
                if index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                placeholders.push(&value[start..index]);
                continue;
            }
        }
        placeholders.sort_unstable();
        placeholders
    }
    placeholders(source) == placeholders(replacement)
}

unsafe extern "system" fn translate_detour(
    result: *mut c_void,
    context: *const c_char,
    source_text: *const c_char,
    disambiguation: *const c_char,
    n: i32,
) -> *mut c_void {
    let Some(original) = HOOK.get() else {
        return result;
    };
    let active = ACTIVE_FEATURES.load(Ordering::Acquire);
    if active == 0 {
        return original.call(result, context, source_text, disambiguation, n);
    }
    let Some(_guard) = CallbackGuard::enter() else {
        return original.call(result, context, source_text, disambiguation, n);
    };
    let replacement = std::panic::catch_unwind(|| {
        let call = translation_call(context, source_text, disambiguation, n)?;
        if active & FEATURE_TEXT_REPLACE != 0 {
            replacement_for(&call)
        } else {
            if active & FEATURE_TEXT_OBSERVE != 0 {
                observe(&call);
            }
            None
        }
    })
    .ok()
    .flatten();
    let effective_source = replacement
        .as_ref()
        .map_or(source_text, |replacement| replacement.as_ptr());
    original.call(result, context, effective_source, disambiguation, n)
}

unsafe fn install_hook() -> Result<(), ()> {
    let qt5 = GetModuleHandleW(w!("Qt5Core.dll")).ok();
    let qt6 = GetModuleHandleW(w!("Qt6Core.dll")).ok();
    let (core, major) = match (qt5, qt6) {
        (Some(core), None) => (core, 5),
        (None, Some(core)) => (core, 6),
        _ => return Err(()),
    };
    let q_version: QVersion = std::mem::transmute(GetProcAddress(core, s!("qVersion")).ok_or(())?);
    let version = CStr::from_ptr(q_version()).to_bytes();
    if !supported_qt_profile(major, version) {
        return Err(());
    }
    if supports_immediate_refresh(major, version) {
        let instance: QCoreInstance = std::mem::transmute(
            GetProcAddress(core, PCSTR(QCORE_INSTANCE_SYMBOL.as_ptr())).ok_or(())?,
        );
        let post_event: QCorePostEvent = std::mem::transmute(
            GetProcAddress(core, PCSTR(QCORE_POST_EVENT_SYMBOL.as_ptr())).ok_or(())?,
        );
        let event_ctor: QEventCtor = std::mem::transmute(
            GetProcAddress(core, PCSTR(QEVENT_CTOR_SYMBOL.as_ptr())).ok_or(())?,
        );
        let refresh = if major == 6 {
            RefreshBridge::CloneOwned {
                instance,
                post_event,
                event_ctor,
                event_dtor: std::mem::transmute(
                    GetProcAddress(core, PCSTR(QEVENT_DTOR_SYMBOL.as_ptr())).ok_or(())?,
                ),
                event_clone: std::mem::transmute(
                    GetProcAddress(core, PCSTR(QEVENT_CLONE_SYMBOL.as_ptr())).ok_or(())?,
                ),
            }
        } else {
            #[cfg(feature = "research-qt5")]
            {
                if major != 5 || version != b"5.15.13" {
                    return Err(());
                }
                RefreshBridge::Qt5MallocOwned {
                    instance,
                    post_event,
                    event_ctor,
                    q_malloc: std::mem::transmute(
                        GetProcAddress(core, PCSTR(QMALLOC_SYMBOL.as_ptr())).ok_or(())?,
                    ),
                }
            }
            #[cfg(not(feature = "research-qt5"))]
            {
                return Err(());
            }
        };
        REFRESH.set(refresh).map_err(|_| ())?;
    }
    let target = GetProcAddress(core, PCSTR(TRANSLATE_SYMBOL.as_ptr())).ok_or(())?;
    let target: Translate = std::mem::transmute(target);
    let detour = GenericDetour::<Translate>::new(target, translate_detour).map_err(|_| ())?;
    HOOK.set(detour).map_err(|_| ())?;
    HOOK.get().ok_or(())?.enable().map_err(|_| ())
}

extern "C" fn activate(
    host: *const NativeRuntimeHostV1,
    requested: u64,
    granted: u64,
) -> NativeNegotiationV1 {
    if host.is_null()
        || unsafe { (*host).struct_size } != std::mem::size_of::<NativeRuntimeHostV1>() as u32
    {
        return negotiation_error(STATUS_INVALID_HOST);
    }
    let negotiated = negotiate_features(requested, granted);
    if negotiated.status != STATUS_OK {
        return negotiated;
    }
    let host = unsafe { *host };
    let bridge = HostBridge {
        context: host.context as usize,
        decide_utf16: host.decide_utf16,
        text_host: TEXT_HOST
            .lock()
            .ok()
            .and_then(|binding| *binding)
            .filter(|binding| binding.matches(&host)),
    };
    let host_ready = if let Some(current) = HOST.get() {
        current.write().map(|mut current| *current = bridge).is_ok()
    } else {
        HOST.set(RwLock::new(bridge)).is_ok()
    };
    if !host_ready || (HOOK.get().is_none() && unsafe { install_hook() }.is_err()) {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }
    ACTIVE_FEATURES.store(negotiated.active_feature_bits, Ordering::Release);
    negotiated
}

extern "C" fn deactivate() -> i32 {
    ACTIVE_FEATURES.store(0, Ordering::Release);
    STATUS_OK
}

extern "C" fn request_refresh() {
    let Some(refresh) = REFRESH.get().copied() else {
        return;
    };
    let _ = std::panic::catch_unwind(|| unsafe { refresh.post_language_change() });
}

#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: std::mem::size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::inline_target(
            ADAPTER_ID,
            (1, 0, 0),
            FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE,
            PLATFORM_WINDOWS,
            ARCH_X86_64,
        ),
        negotiate_features,
        activate,
        deactivate,
        request_refresh,
    }
}

/// Optional text-evidence binding. It preserves the legacy V1 host while letting Runtime classify
/// this call as an observe event rather than a draw event.
///
/// # Safety
/// A non-null host must remain valid until all Adapter callbacks have finished.
#[no_mangle]
pub unsafe extern "C" fn glyphshift_adapter_bind_text_host_v1(
    host: *const NativeTextHostV1,
) -> i32 {
    let value = if host.is_null() {
        None
    } else {
        let Some(value) = NativeTextHostBinding::new(*host) else {
            return STATUS_INVALID_HOST;
        };
        Some(value)
    };
    match TEXT_HOST.lock() {
        Ok(mut binding) => {
            *binding = value;
            STATUS_OK
        }
        Err(_) => STATUS_INVALID_HOST,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn production_profile_accepts_only_verified_qt6() {
        assert!(supported_qt_profile(6, b"6.10.3"));
        assert_eq!(
            supported_qt_profile(5, b"5.15.18"),
            cfg!(feature = "research-qt5")
        );
        assert_eq!(
            supported_qt_profile(5, b"5.15.13"),
            cfg!(feature = "research-qt5")
        );
        assert!(!supported_qt_profile(5, b"5.15.17"));
        assert!(!supported_qt_profile(5, b"6.10.3"));
        assert!(!supported_qt_profile(6, b"5.15.18"));
        assert!(!supported_qt_profile(6, b"6.8.3"));
        assert!(!supported_qt_profile(6, b"6.10.1"));
    }

    #[cfg(feature = "research-qt5")]
    #[test]
    fn research_profile_can_explicitly_enable_verified_qt5() {
        assert!(supported_qt_profile(5, b"5.15.13"));
        assert!(supported_qt_profile(5, b"5.15.18"));
        assert!(supported_qt_profile(6, b"6.10.3"));
    }

    #[test]
    fn refresh_profiles_are_explicit_and_versioned() {
        assert_eq!(
            supports_immediate_refresh(5, b"5.15.13"),
            cfg!(feature = "research-qt5")
        );
        assert!(!supports_immediate_refresh(5, b"5.15.18"));
        assert!(supports_immediate_refresh(6, b"6.10.3"));
        assert!(!supports_immediate_refresh(6, b"6.10.1"));
    }

    #[test]
    fn translation_call_keeps_context_disambiguation_and_plural_count() {
        let context = CString::new("Application").unwrap();
        let source = CString::new("%n torrent(s) completed").unwrap();
        let disambiguation = CString::new("system notification").unwrap();
        let call = unsafe {
            translation_call(
                context.as_ptr(),
                source.as_ptr(),
                disambiguation.as_ptr(),
                3,
            )
        }
        .unwrap();
        assert_eq!(call.context.as_deref(), Some("Application"));
        assert_eq!(call.source, "%n torrent(s) completed");
        assert_eq!(call.disambiguation.as_deref(), Some("system notification"));
        assert_eq!(call.n, 3);

        let event = encode_translation_event(&call, TEXT_EVENT_RETAINED);
        assert_eq!(event.event.metadata_version, TEXT_EVENT_METADATA_VERSION_V1);
        assert_eq!(
            event.event.metadata_flags,
            TEXT_EVENT_METADATA_CONTEXT
                | TEXT_EVENT_METADATA_DISAMBIGUATION
                | TEXT_EVENT_METADATA_PLURAL_N
        );
        assert_eq!(event.event.plural_n, 3);
    }

    #[test]
    fn null_optional_disambiguation_is_preserved_and_bad_sources_fail_open() {
        let context = CString::new("MainWindow").unwrap();
        let source = CString::new("Download completed").unwrap();
        let call =
            unsafe { translation_call(context.as_ptr(), source.as_ptr(), std::ptr::null(), -1) }
                .unwrap();
        assert_eq!(call.disambiguation, None);
        assert_eq!(call.n, -1);

        let no_context =
            unsafe { translation_call(std::ptr::null(), source.as_ptr(), std::ptr::null(), -1) }
                .unwrap();
        assert_eq!(no_context.context, None);

        let whitespace = CString::new(" \t ").unwrap();
        assert!(unsafe {
            translation_call(context.as_ptr(), whitespace.as_ptr(), std::ptr::null(), -1)
        }
        .is_none());
        assert!(unsafe {
            translation_call(context.as_ptr(), std::ptr::null(), std::ptr::null(), -1)
        }
        .is_none());
    }

    #[test]
    fn legacy_source_only_host_accepts_non_plural_context_but_rejects_plural_calls() {
        let source = CString::new("Open").unwrap();
        let context = CString::new("MainMenu").unwrap();
        let disambiguation = CString::new("verb").unwrap();

        let source_only =
            unsafe { translation_call(std::ptr::null(), source.as_ptr(), std::ptr::null(), -1) }
                .unwrap();
        assert!(legacy_source_only_replacement_safe(&source_only));

        let contextual =
            unsafe { translation_call(context.as_ptr(), source.as_ptr(), std::ptr::null(), -1) }
                .unwrap();
        assert!(legacy_source_only_replacement_safe(&contextual));

        let disambiguated = unsafe {
            translation_call(
                std::ptr::null(),
                source.as_ptr(),
                disambiguation.as_ptr(),
                -1,
            )
        }
        .unwrap();
        assert!(legacy_source_only_replacement_safe(&disambiguated));

        let plural =
            unsafe { translation_call(std::ptr::null(), source.as_ptr(), std::ptr::null(), 2) }
                .unwrap();
        assert!(!legacy_source_only_replacement_safe(&plural));

        let invalid_negative_n =
            unsafe { translation_call(std::ptr::null(), source.as_ptr(), std::ptr::null(), -2) }
                .unwrap();
        assert!(!legacy_source_only_replacement_safe(&invalid_negative_n));
    }

    #[test]
    fn qt_placeholder_contract_allows_reordering_but_rejects_loss() {
        assert!(qt_placeholders_compatible(
            "%1 copied to %L2",
            "已将 %L2 复制为 %1"
        ));
        assert!(qt_placeholders_compatible(
            "%n file(s), %1 failed",
            "%1 失败，共 %n 个文件"
        ));
        assert!(!qt_placeholders_compatible("Open %1", "打开"));
        assert!(!qt_placeholders_compatible("%n file(s)", "%Ln 个文件"));
        assert!(qt_placeholders_compatible("Progress: 100%%", "进度：100%%"));
    }

    #[test]
    fn msvc_x64_symbol_is_the_verified_global_qt_export() {
        assert_eq!(
            TRANSLATE_SYMBOL,
            b"?translate@QCoreApplication@@SA?AVQString@@PEBD00H@Z\0"
        );
    }

    #[test]
    fn qt6_refresh_symbols_match_the_verified_public_exports() {
        assert_eq!(
            QCORE_INSTANCE_SYMBOL,
            b"?instance@QCoreApplication@@SAPEAV1@XZ\0"
        );
        assert_eq!(
            QCORE_POST_EVENT_SYMBOL,
            b"?postEvent@QCoreApplication@@SAXPEAVQObject@@PEAVQEvent@@H@Z\0"
        );
        assert_eq!(QEVENT_CTOR_SYMBOL, b"??0QEvent@@QEAA@W4Type@0@@Z\0");
        assert_eq!(QEVENT_DTOR_SYMBOL, b"??1QEvent@@UEAA@XZ\0");
        assert_eq!(QEVENT_CLONE_SYMBOL, b"?clone@QEvent@@UEBAPEAV1@XZ\0");
        assert_eq!(QEVENT_LANGUAGE_CHANGE, 89);
    }

    #[cfg(feature = "research-qt5")]
    #[test]
    fn qt5_refresh_uses_the_verified_qmalloc_export() {
        assert_eq!(QMALLOC_SYMBOL, b"?qMalloc@@YAPEAX_K@Z\0");
        assert!(supports_immediate_refresh(5, b"5.15.13"));
        assert!(!supports_immediate_refresh(5, b"5.15.18"));
    }

    #[test]
    fn observe_and_replace_are_the_only_supported_features() {
        let both = FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE;
        assert_eq!(negotiate_features(both, both).status, STATUS_OK);
        assert_eq!(
            negotiate_features(both, FEATURE_TEXT_OBSERVE).status,
            STATUS_UNAUTHORIZED_FEATURE
        );
        assert_eq!(
            negotiate_features(both | (1 << 63), u64::MAX).status,
            STATUS_UNSUPPORTED_FEATURE
        );
    }
}
