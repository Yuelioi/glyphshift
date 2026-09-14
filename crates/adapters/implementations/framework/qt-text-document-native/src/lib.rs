//! Qt 6.8.3 MSVC x64 `QTextDocument::setHtml` retained-text Adapter.
//!
//! The hook accepts plain text and conservative HTML with one visible literal text node. It
//! records the document's source and last Glyphshift write, and marshals publication updates /
//! restoration back to the owning Qt GUI thread before touching retained document state.

use glyphshift_adapter_native_abi::*;
use glyphshift_adapter_qt_text_document::{
    DocumentText, HtmlTextTemplate, ADAPTER_ID, MAX_TEXT_UNITS,
};
use retour::GenericDetour;
use std::collections::HashMap;
use std::ffi::{c_void, CStr};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{HWND, LPARAM};
use windows_sys::Win32::System::LibraryLoader::{
    GetModuleHandleExW, GetModuleHandleW, GetProcAddress, GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
    GET_MODULE_HANDLE_EX_FLAG_PIN,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, EnumWindows, GetClassNameW, GetWindowThreadProcessId, PostMessageW,
    SetWindowsHookExW, UnhookWindowsHookEx, MSG, WH_GETMESSAGE, WM_NULL,
};

type Object = *mut c_void;
type Setter = unsafe extern "system" fn(Object, Object);
type Destructor = unsafe extern "system" fn(Object);
type Getter = unsafe extern "system" fn(Object, Object) -> Object;
type QStringCtor = unsafe extern "system" fn(Object, *const u16, i64) -> Object;
type QStringSize = unsafe extern "system" fn(Object) -> i64;
type QStringUtf16 = unsafe extern "system" fn(Object) -> *const u16;
type QVersion = unsafe extern "system" fn() -> *const i8;

const FEATURES: u64 = FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE;
const SUPPORTED_QT_VERSION: &[u8] = b"6.8.3";
const MARKER: usize = 0x4753_5144;

const SET_HTML: &[u8] = b"?setHtml@QTextDocument@@QEAAXAEBVQString@@@Z\0";
const TO_PLAIN_TEXT: &[u8] = b"?toPlainText@QTextDocument@@QEBA?AVQString@@XZ\0";
const DOCUMENT_DTOR: &[u8] = b"??1QTextDocument@@UEAA@XZ\0";
const QSTRING_CTOR: &[u8] = b"??0QString@@QEAA@PEBVQChar@@_J@Z\0";
const QSTRING_DTOR: &[u8] = b"??1QString@@QEAA@XZ\0";
const QSTRING_SIZE: &[u8] = b"?size@QString@@QEBA_JXZ\0";
const QSTRING_UTF16: &[u8] = b"?utf16@QString@@QEBAPEBGXZ\0";

static ACTIVE: AtomicU64 = AtomicU64::new(0);
static GUI_THREAD: AtomicU32 = AtomicU32::new(0);
static NEXT_REQUEST: AtomicUsize = AtomicUsize::new(0);
static PENDING_REQUEST: AtomicUsize = AtomicUsize::new(0);
static HOST: Mutex<Option<Host>> = Mutex::new(None);
static HOOKS: OnceLock<Hooks> = OnceLock::new();
static RECORDS: OnceLock<Mutex<Records>> = OnceLock::new();
static REQUEST_HOOK: OnceLock<Mutex<Option<RequestHook>>> = OnceLock::new();
static REQUEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static COMPLETED: OnceLock<(Mutex<usize>, Condvar)> = OnceLock::new();

#[derive(Clone, Copy)]
struct Host {
    context: usize,
    decide: DecideUtf16V1,
}

struct Qt {
    ctor: QStringCtor,
    dtor: Destructor,
    size: QStringSize,
    utf16: QStringUtf16,
    to_plain_text: Getter,
}

struct Hooks {
    qt: Qt,
    set_html: GenericDetour<Setter>,
    destroy: GenericDetour<Destructor>,
}

#[derive(Clone)]
struct Record {
    serial: u64,
    text: DocumentText,
    template: HtmlTextTemplate,
}

#[derive(Default)]
struct Records {
    next: u64,
    documents: HashMap<usize, Record>,
}

struct RequestHook {
    thread_id: u32,
    generation: usize,
    handle: isize,
}

#[repr(C, align(8))]
struct Storage([usize; 4]);

fn records() -> &'static Mutex<Records> {
    RECORDS.get_or_init(|| Mutex::new(Records::default()))
}

fn completed() -> &'static (Mutex<usize>, Condvar) {
    COMPLETED.get_or_init(|| (Mutex::new(0), Condvar::new()))
}

fn supported_qt_version(version: &[u8]) -> bool {
    version == SUPPORTED_QT_VERSION
}

impl Qt {
    unsafe fn string(&self, value: Object) -> Option<String> {
        if value.is_null() {
            return None;
        }
        let length = usize::try_from((self.size)(value)).ok()?;
        if length > MAX_TEXT_UNITS {
            return None;
        }
        if length == 0 {
            return Some(String::new());
        }
        let units = (self.utf16)(value);
        if units.is_null() {
            return None;
        }
        String::from_utf16(std::slice::from_raw_parts(units, length)).ok()
    }

    unsafe fn read_document(&self, document: Object) -> Option<String> {
        let mut value = Storage([0; 4]);
        let value = std::ptr::from_mut(&mut value).cast();
        (self.to_plain_text)(document, value);
        let result = self.string(value);
        (self.dtor)(value);
        result
    }

    unsafe fn with_string<R>(&self, text: &str, call: impl FnOnce(Object) -> R) -> Option<R> {
        let units = text.encode_utf16().collect::<Vec<_>>();
        if units.is_empty() || units.len() > MAX_TEXT_UNITS {
            return None;
        }
        let mut value = Storage([0; 4]);
        let value = std::ptr::from_mut(&mut value).cast();
        (self.ctor)(value, units.as_ptr(), units.len() as i64);
        let result = call(value);
        (self.dtor)(value);
        Some(result)
    }
}

fn replacement(source: &str) -> Option<String> {
    let active = ACTIVE.load(Ordering::Acquire);
    if active == 0 {
        return None;
    }
    let host = (*HOST.lock().ok()?)?;
    let units = source.encode_utf16().collect::<Vec<_>>();
    let mut output = vec![0_u16; MAX_TEXT_UNITS];
    let decision = (host.decide)(
        host.context as Object,
        units.as_ptr(),
        units.len() as u32,
        output.as_mut_ptr(),
        output.len() as u32,
        std::ptr::null_mut(),
        0,
    );
    let current = ACTIVE.load(Ordering::Acquire);
    if decision.status != STATUS_OK
        || decision.text_len as usize > output.len()
        || current & FEATURE_TEXT_REPLACE == 0
        || decision.decision_bits & DECISION_TEXT_REPLACE == 0
    {
        return None;
    }
    output.truncate(decision.text_len as usize);
    let replacement = String::from_utf16(&output).ok()?;
    (!replacement.trim().is_empty() && replacement != source).then_some(replacement)
}

fn store_host_source(document: Object, template: HtmlTextTemplate) -> Option<u64> {
    let mut state = records().lock().ok()?;
    state.next = state.next.wrapping_add(1);
    let serial = state.next;
    let source = template.source().to_owned();
    state.documents.insert(
        document as usize,
        Record {
            serial,
            text: DocumentText::new(source),
            template,
        },
    );
    Some(serial)
}

unsafe extern "system" fn host_set_html(document: Object, value: Object) {
    let Some(hooks) = HOOKS.get() else {
        return;
    };
    if GetCurrentThreadId() != GUI_THREAD.load(Ordering::Acquire) {
        return hooks.set_html.call(document, value);
    }
    let replacement_call = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let raw = hooks.qt.string(value)?;
        let template = HtmlTextTemplate::parse(&raw);
        if ACTIVE.load(Ordering::Acquire) == 0 || template.is_none() {
            if let Ok(mut state) = records().lock() {
                state.documents.remove(&(document as usize));
            }
            return None;
        }
        let template = template?;
        let source = template.source().to_owned();
        let serial = store_host_source(document, template.clone())?;
        let desired = replacement(&source)?;
        let rendered = template.render(&desired)?;
        Some((serial, source, desired, rendered))
    }))
    .ok()
    .flatten();

    if let Some((serial, source, desired, rendered)) = replacement_call {
        if hooks
            .qt
            .with_string(&rendered, |replacement| {
                hooks.set_html.call(document, replacement);
            })
            .is_some()
        {
            if let Ok(mut state) = records().lock() {
                if let Some(record) = state.documents.get_mut(&(document as usize)) {
                    if record.serial == serial && record.text.source() == source {
                        record.text.mark_written(desired);
                    }
                }
            }
            return;
        }
    }
    hooks.set_html.call(document, value);
}

unsafe extern "system" fn host_destroy(document: Object) {
    let Some(hooks) = HOOKS.get() else {
        return;
    };
    if let Ok(mut state) = records().lock() {
        state.documents.remove(&(document as usize));
    }
    hooks.destroy.call(document);
}

unsafe fn write_document(
    hooks: &Hooks,
    document: usize,
    serial: u64,
    source: &str,
    visible_value: &str,
    html_value: &str,
    restore: bool,
) {
    let accepted = records().lock().ok().is_some_and(|state| {
        let Some(record) = state.documents.get(&document) else {
            return false;
        };
        record.serial == serial && record.text.source() == source
    });
    if !accepted {
        return;
    }
    if hooks
        .qt
        .with_string(html_value, |text| {
            hooks.set_html.call(document as Object, text)
        })
        .is_none()
    {
        return;
    }
    if let Ok(mut state) = records().lock() {
        if let Some(record) = state.documents.get_mut(&document) {
            if record.serial == serial && record.text.source() == source {
                if restore {
                    record.text.clear_written();
                } else {
                    record.text.mark_written(visible_value.to_owned());
                }
            }
        }
    }
}

unsafe fn refresh_documents() {
    let Some(hooks) = HOOKS.get() else {
        return;
    };
    let active = ACTIVE.load(Ordering::Acquire) != 0;
    let pending = records()
        .lock()
        .map(|state| {
            state
                .documents
                .iter()
                .map(|(object, record)| (*object, record.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    for (document, snapshot) in pending {
        let current = hooks.qt.read_document(document as Object);
        let Some(current) = current else {
            continue;
        };
        let fresh = records()
            .lock()
            .ok()
            .and_then(|state| state.documents.get(&document).cloned());
        let Some(fresh) = fresh.filter(|record| record.serial == snapshot.serial) else {
            continue;
        };
        let source = fresh.text.source().to_owned();
        let ours = fresh.text.restore(&current).is_some();
        if current != source && !ours {
            if let Ok(mut state) = records().lock() {
                state.documents.remove(&document);
            }
            continue;
        }
        if active {
            if let Some(desired) = replacement(&source) {
                if desired != current {
                    if let Some(rendered) = fresh.template.render(&desired) {
                        write_document(
                            hooks,
                            document,
                            fresh.serial,
                            &source,
                            &desired,
                            &rendered,
                            false,
                        );
                    } else if ours {
                        let original = fresh.template.original_html().to_owned();
                        write_document(
                            hooks,
                            document,
                            fresh.serial,
                            &source,
                            &source,
                            &original,
                            true,
                        );
                    }
                }
            } else if ours {
                let original = fresh.template.original_html().to_owned();
                write_document(
                    hooks,
                    document,
                    fresh.serial,
                    &source,
                    &source,
                    &original,
                    true,
                );
            }
        } else if ours {
            let original = fresh.template.original_html().to_owned();
            write_document(
                hooks,
                document,
                fresh.serial,
                &source,
                &source,
                &original,
                true,
            );
        }
    }
    if !active {
        if let Ok(mut state) = records().lock() {
            state.documents.clear();
        }
    }
}

unsafe extern "system" fn find_qt_window(window: HWND, data: LPARAM) -> i32 {
    let out = &mut *(data as *mut (u32, usize));
    let mut process = 0;
    let thread = GetWindowThreadProcessId(window, &mut process);
    let mut name = [0_u16; 64];
    let length = GetClassNameW(window, name.as_mut_ptr(), name.len() as i32);
    if process == GetCurrentProcessId()
        && thread != 0
        && length >= 2
        && name[..2] == ['Q' as u16, 't' as u16]
    {
        *out = (thread, window as usize);
        return 0;
    }
    1
}

unsafe fn locate_gui_window() -> Option<(u32, HWND)> {
    let mut found = (0_u32, 0_usize);
    EnumWindows(
        Some(find_qt_window),
        std::ptr::from_mut(&mut found) as LPARAM,
    );
    (found.0 != 0 && found.1 != 0).then_some((found.0, found.1 as HWND))
}

fn take_request_hook(thread_id: u32, generation: usize) -> Option<RequestHook> {
    REQUEST_HOOK
        .get()
        .and_then(|hook| hook.lock().ok())
        .and_then(|mut hook| {
            let current = hook.take()?;
            if current.thread_id == thread_id && current.generation == generation {
                Some(current)
            } else {
                *hook = Some(current);
                None
            }
        })
}

unsafe fn cancel_request(generation: usize) {
    if let Some(slot) = REQUEST_HOOK.get() {
        if let Ok(mut slot) = slot.lock() {
            let matches = slot
                .as_ref()
                .is_some_and(|hook| hook.generation == generation);
            if matches {
                if let Some(hook) = slot.take() {
                    UnhookWindowsHookEx(hook.handle as *mut c_void);
                }
            }
        }
    }
    let _ = PENDING_REQUEST.compare_exchange(generation, 0, Ordering::AcqRel, Ordering::Acquire);
}

unsafe extern "system" fn request_hook(code: i32, wparam: usize, lparam: isize) -> isize {
    if code >= 0 && lparam != 0 {
        let message = &*(lparam as *const MSG);
        let generation = message.lParam as usize;
        let thread_id = GetCurrentThreadId();
        if message.message == WM_NULL && message.wParam == MARKER {
            if let Some(hook) = take_request_hook(thread_id, generation) {
                UnhookWindowsHookEx(hook.handle as *mut c_void);
                if PENDING_REQUEST
                    .compare_exchange(generation, 0, Ordering::AcqRel, Ordering::Acquire)
                    .is_ok()
                {
                    let _ = std::panic::catch_unwind(|| refresh_documents());
                    let (lock, signal) = completed();
                    if let Ok(mut done) = lock.lock() {
                        *done = generation;
                        signal.notify_all();
                    }
                }
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

unsafe fn request_gui_work() -> Option<usize> {
    let _guard = REQUEST_LOCK.get_or_init(|| Mutex::new(())).lock().ok()?;
    let (thread_id, window) = locate_gui_window()?;
    if thread_id != GUI_THREAD.load(Ordering::Acquire) {
        return None;
    }
    let mut generation = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
    if generation == 0 {
        generation = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
    }
    PENDING_REQUEST.store(generation, Ordering::Release);

    let slot = REQUEST_HOOK.get_or_init(|| Mutex::new(None));
    let previous = slot.lock().ok()?.take();
    if let Some(previous) = previous {
        UnhookWindowsHookEx(previous.handle as *mut c_void);
    }
    let handle = SetWindowsHookExW(
        WH_GETMESSAGE,
        Some(request_hook),
        std::ptr::null_mut(),
        thread_id,
    );
    if handle.is_null() {
        let _ =
            PENDING_REQUEST.compare_exchange(generation, 0, Ordering::AcqRel, Ordering::Acquire);
        return None;
    }
    *slot.lock().ok()? = Some(RequestHook {
        thread_id,
        generation,
        handle: handle as isize,
    });
    if PostMessageW(window, WM_NULL, MARKER, generation as isize) == 0 {
        cancel_request(generation);
        return None;
    }
    Some(generation)
}

unsafe fn module(name: &str) -> Result<isize, ()> {
    let wide = name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let handle = GetModuleHandleW(wide.as_ptr());
    (!handle.is_null()).then_some(handle as isize).ok_or(())
}

unsafe fn resolve<T: Copy>(module: isize, symbol: &'static [u8]) -> Result<T, ()> {
    let function = GetProcAddress(module as _, symbol.as_ptr().cast()).ok_or(())?;
    Ok(std::mem::transmute_copy(&function))
}

unsafe fn initialize() -> Result<(), ()> {
    if HOOKS.get().is_none() {
        let core = module("Qt6Core.dll")?;
        let gui = module("Qt6Gui.dll")?;
        let version: QVersion = resolve(core, b"qVersion\0")?;
        let version = CStr::from_ptr(version()).to_bytes();
        if !supported_qt_version(version) {
            return Err(());
        }
        let qt = Qt {
            ctor: resolve(core, QSTRING_CTOR)?,
            dtor: resolve(core, QSTRING_DTOR)?,
            size: resolve(core, QSTRING_SIZE)?,
            utf16: resolve(core, QSTRING_UTF16)?,
            to_plain_text: resolve(gui, TO_PLAIN_TEXT)?,
        };
        let set_html =
            GenericDetour::new(resolve::<Setter>(gui, SET_HTML)?, host_set_html as Setter)
                .map_err(|_| ())?;
        let destroy = GenericDetour::new(
            resolve::<Destructor>(gui, DOCUMENT_DTOR)?,
            host_destroy as Destructor,
        )
        .map_err(|_| ())?;
        let mut own = std::ptr::null_mut();
        if GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            glyphshift_adapter_entry_v1 as *const () as *const u16,
            &mut own,
        ) == 0
        {
            return Err(());
        }
        HOOKS
            .set(Hooks {
                qt,
                set_html,
                destroy,
            })
            .map_err(|_| ())?;
    }
    let hooks = HOOKS.get().ok_or(())?;
    if !hooks.set_html.is_enabled() {
        hooks.set_html.enable().map_err(|_| ())?;
    }
    if !hooks.destroy.is_enabled() {
        hooks.destroy.enable().map_err(|_| ())?;
    }
    let (thread, _) = locate_gui_window().ok_or(())?;
    GUI_THREAD.store(thread, Ordering::Release);
    Ok(())
}

extern "C" fn negotiate_features(requested: u64, granted: u64) -> NativeNegotiationV1 {
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
    let failure = |status| NativeNegotiationV1 {
        status,
        active_feature_bits: 0,
    };
    if host.is_null()
        || unsafe { (*host).struct_size } != std::mem::size_of::<NativeRuntimeHostV1>() as u32
    {
        return failure(STATUS_INVALID_HOST);
    }
    let result = negotiate_features(requested, granted);
    if result.status != STATUS_OK {
        return result;
    }
    let initialized = std::panic::catch_unwind(AssertUnwindSafe(|| unsafe { initialize() }));
    if !matches!(initialized, Ok(Ok(()))) {
        return failure(STATUS_ACTIVATION_FAILED);
    }
    let mut current = match HOST.lock() {
        Ok(host) => host,
        Err(_) => return failure(STATUS_INVALID_HOST),
    };
    *current = Some(Host {
        context: unsafe { (*host).context } as usize,
        decide: unsafe { (*host).decide_utf16 },
    });
    drop(current);
    ACTIVE.store(result.active_feature_bits, Ordering::Release);
    result
}

extern "C" fn deactivate() -> i32 {
    ACTIVE.store(0, Ordering::Release);
    if HOOKS.get().is_none() {
        return STATUS_OK;
    }
    let Some(generation) = std::panic::catch_unwind(|| unsafe { request_gui_work() })
        .ok()
        .flatten()
    else {
        return STATUS_ACTIVATION_FAILED;
    };
    let deadline = Instant::now() + Duration::from_secs(2);
    let (lock, signal) = completed();
    let Ok(mut done) = lock.lock() else {
        unsafe { cancel_request(generation) };
        return STATUS_ACTIVATION_FAILED;
    };
    while *done < generation {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            unsafe { cancel_request(generation) };
            return STATUS_ACTIVATION_FAILED;
        };
        let Ok((next, timeout)) = signal.wait_timeout(done, remaining) else {
            unsafe { cancel_request(generation) };
            return STATUS_ACTIVATION_FAILED;
        };
        done = next;
        if timeout.timed_out() && *done < generation {
            unsafe { cancel_request(generation) };
            return STATUS_ACTIVATION_FAILED;
        }
    }
    if let Ok(mut host) = HOST.lock() {
        *host = None;
    }
    STATUS_OK
}

extern "C" fn request_refresh() {
    let _ = std::panic::catch_unwind(|| unsafe { request_gui_work() });
}

#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: std::mem::size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::retained_target(
            ADAPTER_ID,
            (1, 0, 0),
            FEATURES,
            PLATFORM_WINDOWS,
            ARCH_X86_64,
        ),
        negotiate_features,
        activate,
        deactivate,
        request_refresh,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_verified_qt_683_profile_is_accepted() {
        assert!(supported_qt_version(b"6.8.3"));
        assert!(!supported_qt_version(b"6.8.2"));
        assert!(!supported_qt_version(b"6.9.0"));
        assert!(!supported_qt_version(b"6.11.1"));
    }
}
