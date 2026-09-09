//! Native Qt 5 MSVC x86/x64 and Qt 6 MSVC x64 `QPainter::drawText` package.

mod symbols;

use glyphshift_adapter_native_abi::{
    DecideUtf16V1, NativeAdapterApiV1, NativeAdapterDescriptorV1, NativeDecisionV1,
    NativeNegotiationV1, NativeRuntimeHostV1, ARCH_X86, ARCH_X86_64, DECISION_TEXT_REPLACE,
    FEATURE_TEXT_OBSERVE, FEATURE_TEXT_REPLACE, PLATFORM_WINDOWS, STATUS_ACTIVATION_FAILED,
    STATUS_INVALID_HOST, STATUS_OK, STATUS_UNAUTHORIZED_FEATURE, STATUS_UNSUPPORTED_FEATURE,
};
use glyphshift_adapter_native_abi::{NativeTextEventV1, NativeTextHostBinding, NativeTextHostV1};
use glyphshift_adapter_qt_painter::ADAPTER_ID;
use retour::GenericDetour;
use std::cell::Cell;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};
use windows::core::{w, PCSTR};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

const SUPPORTED_FEATURES: u64 = FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE;
const MAX_TEXT_UNITS: usize = 16 * 1024;

const DRAW_POINT_SYMBOL: &[u8] = b"?drawText@QPainter@@QEAAXAEBVQPointF@@AEBVQString@@HH@Z\0";
const DRAW_RECT_SYMBOL: &[u8] = b"?drawText@QPainter@@QEAAXAEBVQRect@@HAEBVQString@@PEAV2@@Z\0";
const DRAW_RECT_OPTION_SYMBOL: &[u8] =
    b"?drawText@QPainter@@QEAAXAEBVQRectF@@AEBVQString@@AEBVQTextOption@@@Z\0";
const DRAW_RECT_F_SYMBOL: &[u8] = b"?drawText@QPainter@@QEAAXAEBVQRectF@@HAEBVQString@@PEAV2@@Z\0";
const QSTRING_UTF16_SYMBOL: &[u8] = b"?utf16@QString@@QEBAPEBGXZ\0";
const QSTRING_DTOR_SYMBOL: &[u8] = b"??1QString@@QEAA@XZ\0";
const QSTRING5_CTOR_SYMBOL: &[u8] = b"??0QString@@QEAA@PEBVQChar@@H@Z\0";
const QSTRING5_SIZE_SYMBOL: &[u8] = b"?size@QString@@QEBAHXZ\0";
const QSTRING6_CTOR_SYMBOL: &[u8] = b"??0QString@@QEAA@PEBVQChar@@_J@Z\0";
const QSTRING6_SIZE_SYMBOL: &[u8] = b"?size@QString@@QEBA_JXZ\0";
const QAPPLICATION_ALL_WIDGETS_SYMBOL: &[u8] =
    b"?allWidgets@QApplication@@SA?AV?$QList@PEAVQWidget@@@@XZ\0";
const QWIDGET_FIND_SYMBOL: &[u8] = b"?find@QWidget@@SAPEAV1@_K@Z\0";
const QWIDGET_REPAINT_SYMBOL: &[u8] = b"?repaint@QWidget@@QEAAXXZ\0";
const QLIST_DATA_DISPOSE_SYMBOL: &[u8] = b"?dispose@QListData@@SAXPEAUData@1@@Z\0";
const QARRAY_DATA_DEALLOCATE_SYMBOL: &[u8] = b"?deallocate@QArrayData@@SAXPEAU1@_J1@Z\0";
const MAX_WIDGETS: usize = 100_000;
const REFRESH_MESSAGE_WPARAM: usize = 0x4753_5257;

type RawProc = unsafe extern "system" fn() -> isize;
use member::*;
mod member;
type FnQWidgetFind = unsafe extern "C" fn(usize) -> *mut c_void;
type FnQApplicationAllWidgets = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
type FnQListDataDispose = unsafe extern "C" fn(*mut c_void);
type FnQArrayDataDeallocate = unsafe extern "C" fn(*mut c_void, isize, isize);

#[derive(Clone, Copy)]
struct HostBridge {
    context: usize,
    decide_utf16: DecideUtf16V1,
    text_host: Option<NativeTextHostBinding>,
}

#[derive(Clone, Copy)]
enum QStringApi {
    Qt5 {
        ctor: FnQString5Ctor,
        dtor: FnQStringDtor,
        size: FnQString5Size,
        utf16: FnQStringUtf16,
    },
    Qt6 {
        ctor: FnQString6Ctor,
        dtor: FnQStringDtor,
        size: FnQString6Size,
        utf16: FnQStringUtf16,
    },
}

impl QStringApi {
    unsafe fn read(self, value: *const c_void) -> Option<String> {
        if value.is_null() {
            return None;
        }
        let (length, units) = match self {
            Self::Qt5 { size, utf16, .. } => {
                let length = usize::try_from(size(value)).ok()?;
                (length, utf16(value))
            }
            Self::Qt6 { size, utf16, .. } => {
                let length = usize::try_from(size(value)).ok()?;
                (length, utf16(value))
            }
        };
        if length == 0 || length > MAX_TEXT_UNITS || units.is_null() {
            return None;
        }
        String::from_utf16(std::slice::from_raw_parts(units, length)).ok()
    }

    unsafe fn with_temporary<R>(
        self,
        units: &[u16],
        call: impl FnOnce(*const c_void) -> R,
    ) -> Option<R> {
        if units.is_empty() || units.len() > MAX_TEXT_UNITS {
            return None;
        }
        match self {
            Self::Qt5 { ctor, dtor, .. } => {
                let length = i32::try_from(units.len()).ok()?;
                let mut storage = MaybeUninit::<Qt5StringStorage>::uninit();
                let object = storage.as_mut_ptr().cast::<c_void>();
                ctor(object, units.as_ptr(), length);
                let guard = QStringGuard { object, dtor };
                let result = call(object);
                drop(guard);
                Some(result)
            }
            Self::Qt6 { ctor, dtor, .. } => {
                let length = isize::try_from(units.len()).ok()?;
                let mut storage = MaybeUninit::<Qt6StringStorage>::uninit();
                let object = storage.as_mut_ptr().cast::<c_void>();
                ctor(object, units.as_ptr(), length);
                let guard = QStringGuard { object, dtor };
                let result = call(object);
                drop(guard);
                Some(result)
            }
        }
    }
}

#[repr(C, align(8))]
struct Qt5StringStorage([u8; 64]);

#[repr(C, align(8))]
struct Qt6StringStorage([u8; 64]);

struct QStringGuard {
    object: *mut c_void,
    dtor: FnQStringDtor,
}

impl Drop for QStringGuard {
    fn drop(&mut self) {
        unsafe { (self.dtor)(self.object) };
    }
}

#[repr(C)]
struct Qt5WidgetListData {
    ref_count: AtomicI32,
    allocation: i32,
    begin: i32,
    end: i32,
}

#[repr(C)]
struct Qt6ArrayData {
    ref_count: AtomicI32,
    flags: u32,
    allocation: isize,
}

#[repr(C)]
struct Qt6WidgetList {
    data: *mut Qt6ArrayData,
    widgets: *mut *mut c_void,
    size: isize,
}

#[derive(Clone, Copy)]
enum WidgetListRelease {
    Qt5(FnQListDataDispose),
    Qt6(FnQArrayDataDeallocate),
}

struct WidgetRefreshHooks {
    all_widgets: FnQApplicationAllWidgets,
    find_widget: FnQWidgetFind,
    repaint: FnQWidgetRepaint,
    release: WidgetListRelease,
}

impl WidgetRefreshHooks {
    unsafe fn repaint_all(&self) {
        match self.release {
            WidgetListRelease::Qt5(dispose) => {
                let mut list = std::ptr::null_mut::<Qt5WidgetListData>();
                (self.all_widgets)(std::ptr::from_mut(&mut list).cast());
                if list.is_null() {
                    return;
                }
                let header = &*list;
                let begin = header.begin;
                let end = header.end;
                let allocation = header.allocation;
                if begin >= 0 && end >= begin && end <= allocation {
                    let count = usize::try_from(end - begin).unwrap_or(MAX_WIDGETS + 1);
                    if count <= MAX_WIDGETS {
                        let widgets = list
                            .cast::<u8>()
                            .add(std::mem::size_of::<Qt5WidgetListData>())
                            as *const *mut c_void;
                        for index in begin..end {
                            let widget = *widgets.add(index as usize);
                            if !widget.is_null() {
                                (self.repaint)(widget);
                            }
                        }
                    }
                }
                if release_ref(&header.ref_count) {
                    dispose(list.cast());
                }
            }
            WidgetListRelease::Qt6(deallocate) => {
                let mut list = Qt6WidgetList {
                    data: std::ptr::null_mut(),
                    widgets: std::ptr::null_mut(),
                    size: 0,
                };
                (self.all_widgets)(std::ptr::from_mut(&mut list).cast());
                if list.size >= 0 {
                    let count = usize::try_from(list.size).unwrap_or(MAX_WIDGETS + 1);
                    if count <= MAX_WIDGETS && (count == 0 || !list.widgets.is_null()) {
                        for index in 0..count {
                            let widget = *list.widgets.add(index);
                            if !widget.is_null() {
                                (self.repaint)(widget);
                            }
                        }
                    }
                }
                if !list.data.is_null() && release_ref(&(*list.data).ref_count) {
                    deallocate(list.data.cast(), 8, 8);
                }
            }
        }
    }
}

fn release_ref(ref_count: &AtomicI32) -> bool {
    let current = ref_count.load(Ordering::Acquire);
    current > 0 && ref_count.fetch_sub(1, Ordering::AcqRel) == 1
}

struct QtHooks {
    strings: QStringApi,
    point: GenericDetour<FnDrawPoint>,
    rect: GenericDetour<FnDrawRect>,
    rect_option: GenericDetour<FnDrawRectOption>,
    rect_f: GenericDetour<FnDrawRect>,
    widget_refresh: Option<WidgetRefreshHooks>,
}

struct DecisionBuffers {
    decision: NativeDecisionV1,
    text: Vec<u16>,
}

static ACTIVE_FEATURES: AtomicU64 = AtomicU64::new(0);
static NEXT_REFRESH_GENERATION: AtomicUsize = AtomicUsize::new(0);
static PENDING_REFRESH_GENERATION: AtomicUsize = AtomicUsize::new(0);
static HOST: OnceLock<RwLock<HostBridge>> = OnceLock::new();
static HOOKS: OnceLock<QtHooks> = OnceLock::new();
static REFRESH_REQUEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static WINDOW_HOOKS: OnceLock<Mutex<Vec<WindowThreadHook>>> = OnceLock::new();

struct WindowThreadHook {
    thread_id: u32,
    handle: isize,
    generation: usize,
    windows: Vec<usize>,
}

thread_local! {
    static IN_CALLBACK: Cell<bool> = const { Cell::new(false) };
}

struct CallbackGuard;

impl CallbackGuard {
    fn enter() -> Option<Self> {
        IN_CALLBACK.with(|active| (!active.replace(true)).then(|| Self))
    }
}

impl Drop for CallbackGuard {
    fn drop(&mut self) {
        IN_CALLBACK.with(|active| active.set(false));
    }
}

extern "C" fn negotiate_features(requested: u64, granted: u64) -> NativeNegotiationV1 {
    if requested & !SUPPORTED_FEATURES != 0 {
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

fn negotiation_error(status: i32) -> NativeNegotiationV1 {
    NativeNegotiationV1 {
        status,
        active_feature_bits: 0,
    }
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
    if !host_ready || unsafe { install_hooks() }.is_err() {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }
    ACTIVE_FEATURES.store(negotiated.active_feature_bits, Ordering::Release);
    negotiated
}

extern "C" fn deactivate() -> i32 {
    ACTIVE_FEATURES.store(0, Ordering::Release);
    STATUS_OK
}

fn decide(source: &str) -> Option<DecisionBuffers> {
    let host = *HOST.get()?.read().ok()?;
    let source = source.encode_utf16().collect::<Vec<_>>();
    let mut text = vec![0_u16; MAX_TEXT_UNITS];
    let mut font = [];
    let decision = if let Some(extended) = host.text_host {
        extended.decide(
            &NativeTextEventV1::complete_draw(&source),
            &mut text,
            &mut font,
        )
    } else {
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

fn replacement_for(source: &str) -> Option<Vec<u16>> {
    let decision = std::panic::catch_unwind(|| decide(source)).ok().flatten()?;
    let active = ACTIVE_FEATURES.load(Ordering::Acquire);
    (active & FEATURE_TEXT_REPLACE != 0
        && decision.decision.decision_bits & DECISION_TEXT_REPLACE != 0
        && !decision.text.is_empty())
    .then_some(decision.text)
}

fn drawn_point_text(text: &str, from: i32, length: i32) -> Option<String> {
    let from = usize::try_from(from).ok()?;
    let units = text.encode_utf16().collect::<Vec<_>>();
    if from > units.len() || length < -1 {
        return None;
    }
    let end = if length == -1 {
        units.len()
    } else {
        from.checked_add(usize::try_from(length).ok()?)?
            .min(units.len())
    };
    if from == end {
        return None;
    }
    String::from_utf16(&units[from..end]).ok()
}

unsafe fn draw_rect_with(
    hooks: &QtHooks,
    text: *const c_void,
    original: impl FnOnce(),
    replacement_call: impl FnOnce(*const c_void),
) {
    if ACTIVE_FEATURES.load(Ordering::Acquire) == 0 {
        return original();
    }
    let Some(_guard) = CallbackGuard::enter() else {
        return original();
    };
    let Some(source) = hooks.strings.read(text) else {
        return original();
    };
    let _scope = text_scope();
    let Some(replacement) = replacement_for(&source) else {
        return original();
    };
    if hooks
        .strings
        .with_temporary(&replacement, replacement_call)
        .is_none()
    {
        original();
    }
}

unsafe fn resolve(module: HMODULE, symbol: &'static [u8]) -> Result<RawProc, ()> {
    GetProcAddress(module, PCSTR(symbol.as_ptr())).ok_or(())
}

unsafe fn loaded_qt_modules() -> Result<(u8, HMODULE, HMODULE), ()> {
    let qt5 = GetModuleHandleW(w!("Qt5Gui.dll"))
        .ok()
        .zip(GetModuleHandleW(w!("Qt5Core.dll")).ok());
    let qt6 = GetModuleHandleW(w!("Qt6Gui.dll"))
        .ok()
        .zip(GetModuleHandleW(w!("Qt6Core.dll")).ok());
    match (qt5, qt6) {
        (Some((gui, core)), None) => Ok((5, gui, core)),
        (None, Some((gui, core))) if cfg!(target_arch = "x86_64") => Ok((6, gui, core)),
        _ => Err(()),
    }
}

extern "C" fn request_refresh() {
    let _ = std::panic::catch_unwind(|| unsafe { request_widget_refresh() });
}

unsafe fn request_widget_refresh() {
    use windows_sys::Win32::Foundation::{HWND, LPARAM};
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetWindowThreadProcessId, PostMessageW, WM_NULL,
    };

    struct WindowThread {
        thread_id: u32,
        windows: Vec<usize>,
    }

    struct RefreshState {
        process_id: u32,
        threads: Vec<WindowThread>,
    }

    if HOOKS
        .get()
        .and_then(|hooks| hooks.widget_refresh.as_ref())
        .is_none()
    {
        return;
    }
    // Runtime lifecycle callbacks run on a control thread, while every QWidget API must stay on
    // the Qt GUI thread. A short-lived native message hook marshals one marked message to each
    // Qt window thread without calling into Qt from here.
    let Ok(_request_guard) = REFRESH_REQUEST_LOCK.get_or_init(|| Mutex::new(())).lock() else {
        return;
    };
    let mut generation = NEXT_REFRESH_GENERATION
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(1);
    if generation == 0 {
        generation = NEXT_REFRESH_GENERATION
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
    }
    PENDING_REFRESH_GENERATION.store(generation, Ordering::Release);

    unsafe extern "system" fn refresh_window(window: HWND, state: LPARAM) -> i32 {
        let state = &mut *(state as *mut RefreshState);
        let mut process_id = 0;
        let thread_id = GetWindowThreadProcessId(window, &mut process_id);
        let mut class_name = [0_u16; 64];
        let class_length = GetClassNameW(window, class_name.as_mut_ptr(), class_name.len() as i32);
        // Qt's Windows platform windows use a versioned `Qt...` class. Filtering here keeps the
        // later QWidget lookup off unrelated process threads that merely own native windows.
        let is_qt_window = class_length >= 2 && class_name[..2] == ['Q' as u16, 't' as u16];
        if process_id == state.process_id && thread_id != 0 && is_qt_window {
            if let Some(thread) = state
                .threads
                .iter_mut()
                .find(|thread| thread.thread_id == thread_id)
            {
                thread.windows.push(window as usize);
            } else {
                state.threads.push(WindowThread {
                    thread_id,
                    windows: vec![window as usize],
                });
            }
        }
        1
    }

    let mut state = RefreshState {
        process_id: GetCurrentProcessId(),
        threads: Vec::new(),
    };
    EnumWindows(
        Some(refresh_window),
        std::ptr::from_mut(&mut state) as LPARAM,
    );
    for thread in state.threads {
        let thread_id = thread.thread_id;
        let marker_window = thread.windows[0] as HWND;
        if install_window_thread_hook(thread_id, thread.windows, generation)
            && PostMessageW(
                marker_window,
                WM_NULL,
                REFRESH_MESSAGE_WPARAM,
                generation as isize,
            ) == 0
        {
            remove_window_thread_hook(thread_id, generation);
        }
    }
}

unsafe fn install_window_thread_hook(
    thread_id: u32,
    windows: Vec<usize>,
    generation: usize,
) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetWindowsHookExW, UnhookWindowsHookEx, WH_GETMESSAGE,
    };

    if thread_id == 0 {
        return false;
    }
    let hooks = WINDOW_HOOKS.get_or_init(|| Mutex::new(Vec::new()));
    let Ok(mut registered) = hooks.lock() else {
        return false;
    };
    let previous = registered
        .iter()
        .position(|hook| hook.thread_id == thread_id)
        .map(|index| registered.swap_remove(index));
    drop(registered);
    if let Some(previous) = previous {
        if previous.generation == generation {
            let Ok(mut registered) = hooks.lock() else {
                return false;
            };
            registered.push(previous);
            return true;
        }
        UnhookWindowsHookEx(previous.handle as *mut c_void);
    }
    let hook = SetWindowsHookExW(
        WH_GETMESSAGE,
        Some(refresh_window_hook),
        std::ptr::null_mut(),
        thread_id,
    );
    if hook.is_null() {
        return false;
    }
    let Ok(mut registered) = hooks.lock() else {
        UnhookWindowsHookEx(hook);
        return false;
    };
    registered.push(WindowThreadHook {
        thread_id,
        handle: hook as isize,
        generation,
        windows,
    });
    true
}

fn take_window_thread_hook(thread_id: u32, generation: usize) -> Option<WindowThreadHook> {
    WINDOW_HOOKS
        .get()
        .and_then(|hooks| hooks.lock().ok())
        .and_then(|mut hooks| {
            let index = hooks
                .iter()
                .position(|hook| hook.thread_id == thread_id && hook.generation == generation)?;
            Some(hooks.swap_remove(index))
        })
}

unsafe fn remove_window_thread_hook(thread_id: u32, generation: usize) {
    use windows_sys::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx;

    if let Some(hook) = take_window_thread_hook(thread_id, generation) {
        UnhookWindowsHookEx(hook.handle as *mut c_void);
    }
}

unsafe extern "system" fn refresh_window_hook(code: i32, wparam: usize, lparam: isize) -> isize {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{CallNextHookEx, MSG, WM_NULL};

    if code >= 0 && lparam != 0 {
        let message = &*(lparam as *const MSG);
        let generation = message.lParam as usize;
        let thread_id = GetCurrentThreadId();
        if message.message == WM_NULL && message.wParam == REFRESH_MESSAGE_WPARAM {
            if let Some(hook) = take_window_thread_hook(thread_id, generation) {
                use windows_sys::Win32::UI::WindowsAndMessaging::UnhookWindowsHookEx;

                // WH_GETMESSAGE invokes this callback on the window-owning Qt GUI thread.
                UnhookWindowsHookEx(hook.handle as *mut c_void);
                if PENDING_REFRESH_GENERATION.load(Ordering::Acquire) == generation {
                    if let Some(refresh) =
                        HOOKS.get().and_then(|hooks| hooks.widget_refresh.as_ref())
                    {
                        let owns_widget = hook
                            .windows
                            .iter()
                            .any(|window| !(refresh.find_widget)(*window).is_null());
                        if owns_widget
                            && PENDING_REFRESH_GENERATION
                                .compare_exchange(
                                    generation,
                                    0,
                                    Ordering::AcqRel,
                                    Ordering::Acquire,
                                )
                                .is_ok()
                        {
                            refresh.repaint_all();
                        }
                    }
                }
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

unsafe fn build_widget_refresh_hooks(
    major: u8,
    core: HMODULE,
    namespace: symbols::Namespace,
) -> Result<Option<WidgetRefreshHooks>, ()> {
    let resolve = |module, symbol| resolve(module, namespace.symbol(symbol)?);
    let widgets = match major {
        5 => GetModuleHandleW(w!("Qt5Widgets.dll")),
        6 => GetModuleHandleW(w!("Qt6Widgets.dll")),
        _ => return Err(()),
    };
    let Ok(widgets) = widgets else {
        return Ok(None);
    };
    let Ok(all_widgets) = resolve(widgets, QAPPLICATION_ALL_WIDGETS_SYMBOL) else {
        return Ok(None);
    };
    let Ok(repaint) = resolve(widgets, QWIDGET_REPAINT_SYMBOL) else {
        return Ok(None);
    };
    let Ok(find_widget) = resolve(widgets, QWIDGET_FIND_SYMBOL) else {
        return Ok(None);
    };
    let all_widgets = std::mem::transmute::<RawProc, FnQApplicationAllWidgets>(all_widgets);
    let repaint = std::mem::transmute::<RawProc, FnQWidgetRepaint>(repaint);
    let find_widget = std::mem::transmute::<RawProc, FnQWidgetFind>(find_widget);
    let release = match major {
        5 => {
            let Ok(dispose) = resolve(core, QLIST_DATA_DISPOSE_SYMBOL) else {
                return Ok(None);
            };
            WidgetListRelease::Qt5(std::mem::transmute::<RawProc, FnQListDataDispose>(dispose))
        }
        6 => {
            let Ok(deallocate) = resolve(core, QARRAY_DATA_DEALLOCATE_SYMBOL) else {
                return Ok(None);
            };
            WidgetListRelease::Qt6(std::mem::transmute::<RawProc, FnQArrayDataDeallocate>(
                deallocate,
            ))
        }
        _ => return Err(()),
    };
    Ok(Some(WidgetRefreshHooks {
        all_widgets,
        find_widget,
        repaint,
        release,
    }))
}

unsafe fn build_hooks() -> Result<QtHooks, ()> {
    let (major, gui, core) = loaded_qt_modules()?;
    let namespace = symbols::Namespace::detect(major, |symbol| resolve(core, symbol).is_ok())?;
    let resolve = |module, symbol| resolve(module, namespace.symbol(symbol)?);
    let strings = match major {
        5 => QStringApi::Qt5 {
            ctor: std::mem::transmute::<RawProc, FnQString5Ctor>(resolve(
                core,
                QSTRING5_CTOR_SYMBOL,
            )?),
            dtor: std::mem::transmute::<RawProc, FnQStringDtor>(resolve(
                core,
                QSTRING_DTOR_SYMBOL,
            )?),
            size: std::mem::transmute::<RawProc, FnQString5Size>(resolve(
                core,
                QSTRING5_SIZE_SYMBOL,
            )?),
            utf16: std::mem::transmute::<RawProc, FnQStringUtf16>(resolve(
                core,
                QSTRING_UTF16_SYMBOL,
            )?),
        },
        6 => QStringApi::Qt6 {
            ctor: std::mem::transmute::<RawProc, FnQString6Ctor>(resolve(
                core,
                QSTRING6_CTOR_SYMBOL,
            )?),
            dtor: std::mem::transmute::<RawProc, FnQStringDtor>(resolve(
                core,
                QSTRING_DTOR_SYMBOL,
            )?),
            size: std::mem::transmute::<RawProc, FnQString6Size>(resolve(
                core,
                QSTRING6_SIZE_SYMBOL,
            )?),
            utf16: std::mem::transmute::<RawProc, FnQStringUtf16>(resolve(
                core,
                QSTRING_UTF16_SYMBOL,
            )?),
        },
        _ => return Err(()),
    };
    let point_target =
        std::mem::transmute::<RawProc, FnDrawPoint>(resolve(gui, DRAW_POINT_SYMBOL)?);
    let rect_target = std::mem::transmute::<RawProc, FnDrawRect>(resolve(gui, DRAW_RECT_SYMBOL)?);
    let rect_option_target =
        std::mem::transmute::<RawProc, FnDrawRectOption>(resolve(gui, DRAW_RECT_OPTION_SYMBOL)?);
    let rect_f_target =
        std::mem::transmute::<RawProc, FnDrawRect>(resolve(gui, DRAW_RECT_F_SYMBOL)?);
    let widget_refresh = build_widget_refresh_hooks(major, core, namespace)?;
    Ok(QtHooks {
        strings,
        point: GenericDetour::new(point_target, draw_point_detour).map_err(|_| ())?,
        rect: GenericDetour::new(rect_target, draw_rect_detour).map_err(|_| ())?,
        rect_option: GenericDetour::new(rect_option_target, draw_rect_option_detour)
            .map_err(|_| ())?,
        rect_f: GenericDetour::new(rect_f_target, draw_rect_f_detour).map_err(|_| ())?,
        widget_refresh,
    })
}

unsafe fn install_hooks() -> Result<(), ()> {
    if HOOKS.get().is_none() {
        HOOKS.set(build_hooks()?).map_err(|_| ())?;
    }
    let hooks = HOOKS.get().ok_or(())?;
    if !hooks.point.is_enabled() {
        hooks.point.enable().map_err(|_| ())?;
    }
    if !hooks.rect.is_enabled() {
        hooks.rect.enable().map_err(|_| ())?;
    }
    if !hooks.rect_option.is_enabled() {
        hooks.rect_option.enable().map_err(|_| ())?;
    }
    if !hooks.rect_f.is_enabled() {
        hooks.rect_f.enable().map_err(|_| ())?;
    }
    Ok(())
}

#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: std::mem::size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::inline_target(
            ADAPTER_ID,
            (1, 0, 0),
            SUPPORTED_FEATURES,
            PLATFORM_WINDOWS,
            ARCH_X86 | ARCH_X86_64,
        ),
        negotiate_features,
        activate,
        deactivate,
        request_refresh,
    }
}

static TEXT_HOST: std::sync::Mutex<Option<NativeTextHostBinding>> = std::sync::Mutex::new(None);
/// Optional source-evidence binding; the V1 host and legacy rendering path stay valid.
///
/// # Safety
/// A non-null host must point to a readable V1 extension whose context and callbacks
/// remain valid until every Adapter callback has finished.
#[no_mangle]
pub unsafe extern "C" fn glyphshift_adapter_bind_text_host_v1(host: *const NativeTextHostV1) -> i32 {
    let value = if host.is_null() {
        None
    } else {
        let Some(value) = NativeTextHostBinding::new(unsafe { *host }) else {
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

fn text_scope() -> Option<glyphshift_adapter_native_abi::NativeTextScope> {
    let binding = HOST.get()?.read().ok()?.text_host?;
    binding.enter_scope()
}
