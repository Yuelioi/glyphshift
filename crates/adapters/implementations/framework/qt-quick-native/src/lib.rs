//! Qt Quick labels, retained and restored on their owning GUI thread.
//! Native callbacks remain pinned until process exit; deactivation drains and restores.
mod qt;
use glyphshift_adapter_native_abi::*;
use glyphshift_adapter_qt_quick::{LabelText, ADAPTER_ID, MAX_TEXT_UNITS};
use qt::{Destructor, Object, Qt, Setter};
use retour::GenericDetour;
use std::{
    cell::Cell,
    collections::HashMap,
    sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering},
    sync::{Condvar, Mutex, OnceLock},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::*,
    System::{LibraryLoader::*, Threading::*},
    UI::WindowsAndMessaging::*,
};

const FEATURES: u64 = FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE;
const MARKER: usize = 0x4753_5154;
static ACTIVE: AtomicU64 = AtomicU64::new(0);
static GUI_THREAD: AtomicU32 = AtomicU32::new(0);
static GUI_WINDOW: AtomicUsize = AtomicUsize::new(0);
static TIMER: AtomicUsize = AtomicUsize::new(0);
static REQUEST: AtomicU64 = AtomicU64::new(0);
static COMPLETED: OnceLock<(Mutex<u64>, Condvar)> = OnceLock::new();
static INITIALIZE: Mutex<()> = Mutex::new(());
static HOST: Mutex<Option<Host>> = Mutex::new(None);
static HOOKS: OnceLock<Hooks> = OnceLock::new();
static RECORDS: OnceLock<Mutex<Records>> = OnceLock::new();
thread_local! {static PUMPING:Cell<bool>=const {Cell::new(false)};}

#[derive(Clone, Copy)]
struct Host {
    context: usize,
    decide: DecideUtf16V1,
}
type Dispatch = unsafe extern "system" fn(*const MSG) -> isize;
struct Hooks {
    dispatch: GenericDetour<Dispatch>,
    qt: Qt,
    set: GenericDetour<Setter>,
    destroy: GenericDetour<Destructor>,
}
#[derive(Clone)]
struct Record {
    serial: u64,
    text: LabelText,
}
#[derive(Default)]
struct Records {
    next: u64,
    labels: HashMap<usize, Record>,
}
fn records() -> &'static Mutex<Records> {
    RECORDS.get_or_init(|| Mutex::new(Records::default()))
}
fn completed() -> &'static (Mutex<u64>, Condvar) {
    COMPLETED.get_or_init(|| (Mutex::new(0), Condvar::new()))
}

unsafe extern "system" fn host_set(object: Object, value: Object) {
    let Some(hooks) = HOOKS.get() else {
        return;
    };
    if GetCurrentThreadId() == GUI_THREAD.load(Ordering::Acquire) {
        if let Some(source) = hooks.qt.string(value) {
            if let Ok(mut state) = records().lock() {
                if let Some(record) = state.labels.get_mut(&(object as usize)) {
                    record.text.host_write(source);
                }
            }
        }
    }
    hooks.set.call(object, value);
}
unsafe extern "system" fn host_destroy(object: Object) {
    let Some(hooks) = HOOKS.get() else {
        return;
    };
    if let Ok(mut state) = records().lock() {
        state.labels.remove(&(object as usize));
    }
    hooks.destroy.call(object);
}

fn replacement(source: &str) -> Option<String> {
    if ACTIVE.load(Ordering::Acquire) == 0 {
        return None;
    }
    let host = (*HOST.lock().ok()?)?;
    let source = source.encode_utf16().collect::<Vec<_>>();
    let mut output = vec![0u16; MAX_TEXT_UNITS];
    let decision = (host.decide)(
        host.context as Object,
        source.as_ptr(),
        source.len() as u32,
        output.as_mut_ptr(),
        output.len() as u32,
        std::ptr::null_mut(),
        0,
    );
    if decision.status != STATUS_OK
        || decision.text_len as usize > output.len()
        || ACTIVE.load(Ordering::Acquire) & FEATURE_TEXT_REPLACE == 0
        || decision.decision_bits & DECISION_TEXT_REPLACE == 0
    {
        return None;
    }
    output.truncate(decision.text_len as usize);
    if output.is_empty() {
        return None;
    }
    String::from_utf16(&output).ok()
}

/// Validate the incarnation again after calling host code. Qt signal handlers may
/// destroy a later item in the snapshot or assign new application text during a write.
unsafe fn write(
    hooks: &Hooks,
    object: usize,
    serial: u64,
    expected_source: &str,
    value: &str,
    restore: bool,
) {
    let accepted = records().lock().ok().is_some_and(|mut state| {
        let Some(record) = state.labels.get_mut(&object) else {
            return false;
        };
        if record.serial != serial || record.text.source() != expected_source {
            return false;
        }
        if restore {
            record.text.clear_written();
        } else {
            record.text.mark_written(value.to_owned());
        }
        true
    });
    if accepted {
        // Calling the trampoline skips only our own setter hook. Reentrant setters
        // invoked by the application's signals still go through host_set.
        hooks.qt.write(object as Object, value, |object, text| {
            hooks.set.call(object, text)
        });
    }
}
unsafe fn update_labels(hooks: &Hooks) {
    let labels = hooks.qt.labels();
    let mut pending = Vec::new();
    if let Ok(mut state) = records().lock() {
        for label in labels {
            state.next = state.next.wrapping_add(1);
            let serial = state.next;
            let record = state.labels.entry(label.object).or_insert_with(|| Record {
                serial,
                text: LabelText::new(label.source.clone()),
            });
            record.text.observe(&label.source);
            pending.push((label, record.serial));
        }
    }
    for (label, serial) in pending {
        if ACTIVE.load(Ordering::Acquire) == 0 {
            break;
        }
        let record = records()
            .lock()
            .ok()
            .and_then(|state| state.labels.get(&label.object).cloned());
        let Some(record) = record.filter(|r| r.serial == serial) else {
            continue;
        };
        let desired = if label.eligible {
            std::panic::catch_unwind(|| replacement(record.text.source()))
                .ok()
                .flatten()
        } else {
            None
        };
        if let Some(desired) = desired {
            if desired != label.source {
                write(
                    hooks,
                    label.object,
                    serial,
                    record.text.source(),
                    &desired,
                    false,
                );
            }
        } else if let Some(source) = record.text.restore(&label.source) {
            write(
                hooks,
                label.object,
                serial,
                record.text.source(),
                source,
                true,
            );
        }
    }
}
unsafe fn restore_labels(hooks: &Hooks) {
    let pending = records()
        .lock()
        .map(|state| {
            state
                .labels
                .iter()
                .map(|(p, r)| (*p, r.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for (object, record) in pending {
        let alive = records().lock().ok().is_some_and(|state| {
            state
                .labels
                .get(&object)
                .is_some_and(|r| r.serial == record.serial)
        });
        if !alive {
            continue;
        }
        if let Some(current) = hooks.qt.read(object as Object) {
            // Re-read state after callbacks and restore only our own last write.
            let fresh = records()
                .lock()
                .ok()
                .and_then(|state| state.labels.get(&object).cloned());
            if let Some(fresh) = fresh.filter(|r| r.serial == record.serial) {
                if let Some(source) = fresh.text.restore(&current) {
                    write(
                        hooks,
                        object,
                        record.serial,
                        fresh.text.source(),
                        source,
                        true,
                    );
                }
            }
        }
    }
    if let Ok(mut state) = records().lock() {
        state.labels.clear();
    }
}
unsafe fn pump() {
    if GetCurrentThreadId() != GUI_THREAD.load(Ordering::Acquire) {
        return;
    }
    PUMPING.with(|busy| {
        if busy.replace(true) {
            return;
        }
        let epoch = REQUEST.load(Ordering::Acquire);
        let result = std::panic::catch_unwind(|| {
            let Some(hooks) = HOOKS.get() else {
                return;
            };
            if ACTIVE.load(Ordering::Acquire) == 0 {
                let timer = TIMER.swap(0, Ordering::AcqRel);
                if timer != 0 {
                    KillTimer(std::ptr::null_mut(), timer);
                }
                restore_labels(hooks);
            } else {
                if TIMER.load(Ordering::Acquire) == 0 {
                    TIMER.store(
                        SetTimer(std::ptr::null_mut(), 0, 250, Some(tick)),
                        Ordering::Release,
                    );
                }
                update_labels(hooks);
            }
        });
        if result.is_ok() {
            let (lock, signal) = completed();
            if let Ok(mut done) = lock.lock() {
                *done = epoch;
                signal.notify_all();
            }
        }
        busy.set(false);
    });
}
unsafe extern "system" fn tick(_: HWND, _: u32, _: usize, _: u32) {
    pump();
}
unsafe extern "system" fn dispatch_message(message: *const MSG) -> isize {
    let Some(hooks) = HOOKS.get() else {
        return 0;
    };
    let marked = !message.is_null() && (*message).message == WM_NULL && (*message).wParam == MARKER;
    let result = hooks.dispatch.call(message);
    if marked {
        pump();
    }
    result
}
unsafe extern "system" fn find_window(window: HWND, data: LPARAM) -> i32 {
    let out = &mut *(data as *mut (u32, usize));
    let mut process = 0;
    let thread = GetWindowThreadProcessId(window, &mut process);
    let mut name = [0u16; 64];
    let size = GetClassNameW(window, name.as_mut_ptr(), 64);
    if process == GetCurrentProcessId() && thread != 0 && size >= 2 && name[..2] == [81, 116] {
        *out = (thread, window as usize);
        return 0;
    }
    1
}
unsafe fn initialize() -> Result<(), ()> {
    let _lock = INITIALIZE.lock().map_err(|_| ())?;
    if HOOKS.get().is_none() {
        let qt = Qt::resolve()?;
        let user = GetModuleHandleW(
            "user32.dll"
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>()
                .as_ptr(),
        );
        let original: Dispatch = std::mem::transmute(
            GetProcAddress(user, c"DispatchMessageW".as_ptr().cast()).ok_or(())?,
        );
        let dispatch =
            GenericDetour::new(original, dispatch_message as Dispatch).map_err(|_| ())?;
        let set = GenericDetour::new(qt.setter, host_set as Setter).map_err(|_| ())?;
        let destroy =
            GenericDetour::new(qt.destructor, host_destroy as Destructor).map_err(|_| ())?;
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
                set,
                destroy,
                dispatch,
            })
            .map_err(|_| ())?;
    }
    let hooks = HOOKS.get().ok_or(())?;
    if !hooks.set.is_enabled() {
        hooks.set.enable().map_err(|_| ())?;
    }
    if !hooks.destroy.is_enabled() {
        hooks.destroy.enable().map_err(|_| ())?;
    }
    if GUI_THREAD.load(Ordering::Acquire) == 0 {
        let mut found = (0u32, 0usize);
        EnumWindows(Some(find_window), std::ptr::from_mut(&mut found) as LPARAM);
        if found.0 == 0 {
            return Err(());
        }
        GUI_THREAD.store(found.0, Ordering::Release);
        GUI_WINDOW.store(found.1, Ordering::Release);
    }
    if !hooks.dispatch.is_enabled() {
        hooks.dispatch.enable().map_err(|_| ())?;
    }
    Ok(())
}
fn request() -> Option<u64> {
    let epoch = REQUEST.fetch_add(1, Ordering::AcqRel) + 1;
    unsafe {
        if GetCurrentThreadId() == GUI_THREAD.load(Ordering::Acquire) {
            pump();
            return Some(epoch);
        }
        // A popup may have disappeared since activation. Pick another native Qt
        // window from the same process, never a cached application title/handle.
        let mut found = (0u32, 0usize);
        EnumWindows(Some(find_window), std::ptr::from_mut(&mut found) as LPARAM);
        if found.0 != GUI_THREAD.load(Ordering::Acquire) || found.1 == 0 {
            return None;
        }
        GUI_WINDOW.store(found.1, Ordering::Release);
        (PostMessageW(found.1 as HWND, WM_NULL, MARKER, 0) != 0).then_some(epoch)
    }
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
    if unsafe { initialize() }.is_err() {
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
    if request().is_none() {
        ACTIVE.store(0, Ordering::Release);
        return failure(STATUS_ACTIVATION_FAILED);
    }
    result
}
extern "C" fn deactivate() -> i32 {
    ACTIVE.store(0, Ordering::Release);
    if HOOKS.get().is_none() {
        return STATUS_OK;
    }
    let Some(epoch) = request() else {
        return STATUS_ACTIVATION_FAILED;
    };
    let deadline = Instant::now() + Duration::from_secs(2);
    let (lock, signal) = completed();
    let Ok(mut done) = lock.lock() else {
        return STATUS_ACTIVATION_FAILED;
    };
    while *done < epoch {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return STATUS_ACTIVATION_FAILED;
        };
        let Ok((next, timeout)) = signal.wait_timeout(done, remaining) else {
            return STATUS_ACTIVATION_FAILED;
        };
        done = next;
        if timeout.timed_out() && *done < epoch {
            return STATUS_ACTIVATION_FAILED;
        }
    }
    if let Ok(mut host) = HOST.lock() {
        *host = None;
    }
    STATUS_OK
}
extern "C" fn request_refresh() {
    let _ = request();
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
