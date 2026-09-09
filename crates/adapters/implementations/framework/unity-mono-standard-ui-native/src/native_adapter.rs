use crate::late_attach::{
    prepare_runtime, MonoApi, MonoGcHandle, PreparedRuntime, PreparedUiBinding,
};
use glyphshift_adapter_native_abi::{
    DecideUtf16V1, NativeAdapterApiV1, NativeAdapterDescriptorV1, NativeNegotiationV1,
    NativeRuntimeHostV1, ARCH_X86, ARCH_X86_64, DECISION_TEXT_REPLACE, FEATURE_TEXT_OBSERVE,
    FEATURE_TEXT_REPLACE, PLATFORM_WINDOWS, STATUS_ACTIVATION_FAILED, STATUS_INVALID_HOST,
    STATUS_OK, STATUS_UNAUTHORIZED_FEATURE, STATUS_UNSUPPORTED_FEATURE,
};
use glyphshift_adapter_unity_mono_standard_ui::{
    SetterOutcome, TextDecision, TextWrite, UnityMonoWriteback, ADAPTER_ID,
};
use glyphshift_adapter_unity_standard_ui::{
    ManagedObjectId, ManagedText, ObservedText, ObserverEvent, StandardUiKind,
    UnityStandardUiObserver, MAX_TEXT_UNITS,
};
use retour::GenericDetour;
use std::cell::Cell;
use std::collections::{BTreeSet, HashMap};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

const SWEEP_INTERVAL_FRAMES: u32 = 300;
const REFRESH_INTERVAL_FRAMES: u32 = 30;
const SNAPSHOT_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const RESTORE_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const SNAPSHOT_IDLE: u8 = 0;
const SNAPSHOT_PENDING: u8 = 1;
const SNAPSHOT_READY: u8 = 2;
const SNAPSHOT_REJECTED: u8 = 3;
const RESTORE_IDLE: u8 = 0;
const RESTORE_PENDING: u8 = 1;
const RESTORE_COMPLETE: u8 = 2;
const RESTORE_FAILED: u8 = 3;

type ManagedSetter = unsafe extern "C" fn(*mut c_void, *mut c_void);
type MainThreadDispatch = unsafe extern "C" fn(*mut c_void);

#[derive(Clone, Copy)]
struct HostBridge {
    context: usize,
    decide_utf16: DecideUtf16V1,
}

impl HostBridge {
    fn decide(self, source: &str) -> Option<TextDecision> {
        let source = source.encode_utf16().collect::<Vec<_>>();
        if source.is_empty() || source.len() > MAX_TEXT_UNITS {
            return None;
        }
        let mut replacement = vec![0_u16; MAX_TEXT_UNITS];
        let mut font = vec![0_u16; 63];
        let decision = (self.decide_utf16)(
            self.context as *mut c_void,
            source.as_ptr(),
            source.len() as u32,
            replacement.as_mut_ptr(),
            replacement.len() as u32,
            font.as_mut_ptr(),
            font.len() as u32,
        );
        if decision.status != STATUS_OK {
            return None;
        }
        if decision.decision_bits & DECISION_TEXT_REPLACE == 0 {
            return Some(TextDecision::keep(decision.generation));
        }
        let length = usize::try_from(decision.text_len).ok()?;
        if length > replacement.len() {
            return None;
        }
        replacement.truncate(length);
        Some(TextDecision::replace_utf16(
            decision.generation,
            replacement,
        ))
    }
}

#[derive(Default)]
struct RuntimeState {
    session: Option<Session>,
    observer: UnityStandardUiObserver,
    writeback: UnityMonoWriteback,
    active_feature_bits: u64,
}

struct Session {
    api: MonoApi,
    domain: usize,
    enumeration_method: usize,
    bindings: Vec<PreparedUiBinding>,
    handles: BTreeSet<MonoGcHandle>,
    pointer_cache: HashMap<usize, MonoGcHandle>,
    pending_observations: Vec<ObservedText>,
    snapshot_ready: bool,
    sweep_frame: u32,
    refresh_frame: u32,
    main_thread_id: Option<u32>,
    translations_applied: bool,
}

impl Session {
    fn new(prepared: PreparedRuntime) -> Self {
        Self {
            api: prepared.api,
            domain: prepared.domain,
            enumeration_method: prepared.enumeration_method,
            bindings: prepared.bindings,
            handles: BTreeSet::new(),
            pointer_cache: HashMap::new(),
            pending_observations: Vec::new(),
            snapshot_ready: false,
            sweep_frame: 0,
            refresh_frame: 0,
            main_thread_id: None,
            translations_applied: false,
        }
    }

    fn binding(&self, kind: StandardUiKind) -> Option<PreparedUiBinding> {
        self.bindings
            .iter()
            .copied()
            .find(|binding| binding.kind == kind)
    }

    unsafe fn initial_snapshot(&mut self) -> Result<Vec<ManagedText>, ()> {
        self.main_thread_id = Some(current_thread_id());
        let mut snapshot = Vec::new();
        for binding in self.bindings.clone() {
            let handles = unsafe {
                self.api
                    .enumerate_weak_handles(self.domain, binding.class, self.enumeration_method)
            }
            .map_err(|_| ())?;
            for handle in handles {
                let Some(pointer) = (unsafe { self.api.handle_target(handle) }) else {
                    unsafe { self.api.free_handle(handle) };
                    continue;
                };
                let Some(units) =
                    (unsafe { self.api.read_handle_field_utf16(handle, binding.field) })
                else {
                    unsafe { self.api.free_handle(handle) };
                    continue;
                };
                self.handles.insert(handle);
                self.pointer_cache.insert(pointer, handle);
                snapshot.push(ManagedText::utf16(
                    ManagedObjectId::new(handle as u64),
                    binding.kind,
                    units,
                ));
            }
        }
        self.snapshot_ready = true;
        Ok(snapshot)
    }

    unsafe fn setter_text(
        &mut self,
        kind: StandardUiKind,
        object: usize,
        string: usize,
    ) -> Option<ManagedText> {
        if !self.snapshot_ready || self.binding(kind).is_none() || object == 0 {
            return None;
        }
        let units = unsafe { self.api.copy_string_utf16(string) }?;
        let object_id = unsafe { self.object_id(object) }?;
        Some(ManagedText::utf16(object_id, kind, units))
    }

    unsafe fn object_id(&mut self, object: usize) -> Option<ManagedObjectId> {
        if let Some(handle) = self.pointer_cache.get(&object).copied() {
            if unsafe { self.api.handle_target(handle) } == Some(object) {
                return Some(ManagedObjectId::new(handle as u64));
            }
            self.pointer_cache.remove(&object);
        }

        for handle in self.handles.iter().copied() {
            if unsafe { self.api.handle_target(handle) } == Some(object) {
                self.pointer_cache.insert(object, handle);
                return Some(ManagedObjectId::new(handle as u64));
            }
        }

        let handle = unsafe { self.api.weak_handle_for_object(object) }?;
        self.handles.insert(handle);
        self.pointer_cache.insert(object, handle);
        Some(ManagedObjectId::new(handle as u64))
    }

    unsafe fn sweep_if_due(&mut self) -> Vec<ManagedObjectId> {
        self.sweep_frame = self.sweep_frame.wrapping_add(1);
        if !self.sweep_frame.is_multiple_of(SWEEP_INTERVAL_FRAMES) {
            return Vec::new();
        }

        let mut collected = Vec::new();
        let mut live_pointers = HashMap::with_capacity(self.handles.len());
        for handle in self.handles.iter().copied() {
            if let Some(pointer) = unsafe { self.api.handle_target(handle) } {
                live_pointers.insert(pointer, handle);
            } else {
                collected.push(handle);
            }
        }
        for handle in &collected {
            self.handles.remove(handle);
            unsafe { self.api.free_handle(*handle) };
        }
        self.pointer_cache = live_pointers;
        collected
            .into_iter()
            .map(|handle| ManagedObjectId::new(handle as u64))
            .collect()
    }

    fn refresh_due(&mut self) -> bool {
        self.refresh_frame = self.refresh_frame.wrapping_add(1);
        self.refresh_frame.is_multiple_of(REFRESH_INTERVAL_FRAMES)
    }

    fn is_main_thread(&self) -> bool {
        self.main_thread_id == Some(current_thread_id())
    }

    unsafe fn managed_string(&self, units: &[u16]) -> Option<ManagedStringHandle> {
        if !self.is_main_thread() {
            return None;
        }
        let handle = unsafe { self.api.new_string_handle_utf16(self.domain, units) }?;
        let target = unsafe { self.api.handle_target(handle) }?;
        Some(ManagedStringHandle {
            api: self.api,
            handle,
            target,
        })
    }

    unsafe fn apply_writes(&self, writes: &[TextWrite]) -> bool {
        if !self.is_main_thread() {
            return false;
        }
        for write in writes {
            let Ok(handle) = usize::try_from(write.object_id().value()) else {
                return false;
            };
            let Some(object) = (unsafe { self.api.handle_target(handle) }) else {
                continue;
            };
            let Some(string) = (unsafe { self.managed_string(write.units()) }) else {
                return false;
            };
            if unsafe { call_original_setter(write.kind(), object, string.target) }.is_err() {
                return false;
            }
        }
        true
    }

    unsafe fn release_handles(&mut self) {
        for handle in std::mem::take(&mut self.handles) {
            unsafe { self.api.free_handle(handle) };
        }
        self.pointer_cache.clear();
        self.pending_observations.clear();
        self.snapshot_ready = false;
        self.main_thread_id = None;
        self.translations_applied = false;
    }
}

struct ManagedStringHandle {
    api: MonoApi,
    handle: MonoGcHandle,
    target: usize,
}

impl Drop for ManagedStringHandle {
    fn drop(&mut self) {
        unsafe { self.api.free_handle(self.handle) };
    }
}

static ACTIVE: AtomicBool = AtomicBool::new(false);
static STOPPING: AtomicBool = AtomicBool::new(false);
static SNAPSHOT_STATUS: AtomicU8 = AtomicU8::new(SNAPSHOT_IDLE);
static RESTORE_STATUS: AtomicU8 = AtomicU8::new(RESTORE_IDLE);
static HOST: OnceLock<RwLock<HostBridge>> = OnceLock::new();
static STATE: OnceLock<Mutex<RuntimeState>> = OnceLock::new();
static DISPATCH_HOOK: OnceLock<GenericDetour<MainThreadDispatch>> = OnceLock::new();
static DISPATCH_TARGET: OnceLock<usize> = OnceLock::new();
static TMP_SETTER_HOOK: OnceLock<GenericDetour<ManagedSetter>> = OnceLock::new();
static TMP_SETTER_TARGET: OnceLock<usize> = OnceLock::new();
static UGUI_SETTER_HOOK: OnceLock<GenericDetour<ManagedSetter>> = OnceLock::new();
static UGUI_SETTER_TARGET: OnceLock<usize> = OnceLock::new();

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

const fn negotiation_error(status: i32) -> NativeNegotiationV1 {
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
    let state = STATE.get_or_init(|| Mutex::new(RuntimeState::default()));
    let Ok(mut state) = state.lock() else {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    };
    if state.session.is_some() {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }

    let prepared = match std::panic::catch_unwind(|| unsafe { prepare_runtime() }) {
        Ok(Ok(prepared)) => prepared,
        _ => return negotiation_error(STATUS_ACTIVATION_FAILED),
    };
    if unsafe { install_dispatch_hook(&prepared) }.is_err() {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }

    let bridge = HostBridge {
        context: host.context as usize,
        decide_utf16: host.decide_utf16,
    };
    let host_ready = if let Some(current) = HOST.get() {
        current.write().map(|mut current| *current = bridge).is_ok()
    } else {
        HOST.set(RwLock::new(bridge)).is_ok()
    };
    if !host_ready {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }
    let _ = state.observer.apply(ObserverEvent::Deactivate);
    let _ = state.writeback.deactivate();
    state.active_feature_bits = negotiated.active_feature_bits;
    state.session = Some(Session::new(prepared));
    STOPPING.store(false, Ordering::Release);
    RESTORE_STATUS.store(RESTORE_IDLE, Ordering::Release);
    SNAPSHOT_STATUS.store(SNAPSHOT_PENDING, Ordering::Release);
    ACTIVE.store(negotiated.active_feature_bits != 0, Ordering::Release);
    drop(state);

    let deadline = Instant::now() + SNAPSHOT_WAIT_TIMEOUT;
    while SNAPSHOT_STATUS.load(Ordering::Acquire) == SNAPSHOT_PENDING && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    if SNAPSHOT_STATUS.load(Ordering::Acquire) != SNAPSHOT_READY {
        let _ = deactivate();
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }
    if unsafe { install_setter_hooks() }.is_err() {
        let _ = deactivate();
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }
    negotiated
}

extern "C" fn deactivate() -> i32 {
    let Some(state) = STATE.get() else {
        ACTIVE.store(false, Ordering::Release);
        SNAPSHOT_STATUS.store(SNAPSHOT_IDLE, Ordering::Release);
        return STATUS_OK;
    };

    let Ok(state_guard) = state.lock() else {
        return STATUS_ACTIVATION_FAILED;
    };
    let needs_restore = state_guard.active_feature_bits & FEATURE_TEXT_REPLACE != 0
        && state_guard.writeback.has_applied_replacements();
    let restore_here = needs_restore
        && state_guard
            .session
            .as_ref()
            .is_some_and(Session::is_main_thread);
    drop(state_guard);

    let mut restore_ok = true;
    if needs_restore {
        STOPPING.store(true, Ordering::Release);
        RESTORE_STATUS.store(RESTORE_PENDING, Ordering::Release);
        if restore_here {
            let Some(_guard) = CallbackGuard::enter() else {
                return STATUS_ACTIVATION_FAILED;
            };
            if std::panic::catch_unwind(restore_main_thread).is_err() {
                RESTORE_STATUS.store(RESTORE_FAILED, Ordering::Release);
            }
        } else {
            let deadline = Instant::now() + RESTORE_WAIT_TIMEOUT;
            while RESTORE_STATUS.load(Ordering::Acquire) == RESTORE_PENDING
                && Instant::now() < deadline
            {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        let restore_status = RESTORE_STATUS.load(Ordering::Acquire);
        if restore_status == RESTORE_PENDING {
            ACTIVE.store(false, Ordering::Release);
            return STATUS_ACTIVATION_FAILED;
        }
        restore_ok = restore_status == RESTORE_COMPLETE;
    };

    ACTIVE.store(false, Ordering::Release);
    let Ok(mut state) = state.lock() else {
        return STATUS_ACTIVATION_FAILED;
    };
    let Some(mut session) = state.session.take() else {
        let _ = state.observer.apply(ObserverEvent::Deactivate);
        let _ = state.writeback.deactivate();
        state.active_feature_bits = 0;
        STOPPING.store(false, Ordering::Release);
        RESTORE_STATUS.store(RESTORE_IDLE, Ordering::Release);
        SNAPSHOT_STATUS.store(SNAPSHOT_IDLE, Ordering::Release);
        return if restore_ok {
            STATUS_OK
        } else {
            STATUS_ACTIVATION_FAILED
        };
    };
    let Ok(_thread) = (unsafe { session.api.attach_foreign_thread(session.domain) }) else {
        state.session = Some(session);
        return STATUS_ACTIVATION_FAILED;
    };
    unsafe { session.release_handles() };
    let _ = state.observer.apply(ObserverEvent::Deactivate);
    let _ = state.writeback.deactivate();
    state.active_feature_bits = 0;
    STOPPING.store(false, Ordering::Release);
    RESTORE_STATUS.store(RESTORE_IDLE, Ordering::Release);
    SNAPSHOT_STATUS.store(SNAPSHOT_IDLE, Ordering::Release);
    if restore_ok {
        STATUS_OK
    } else {
        STATUS_ACTIVATION_FAILED
    }
}

unsafe fn install_dispatch_hook(prepared: &PreparedRuntime) -> Result<(), ()> {
    if DISPATCH_HOOK.get().is_some() {
        if DISPATCH_TARGET.get().copied() != Some(prepared.dispatch_hook_address) {
            return Err(());
        }
    } else {
        let target: MainThreadDispatch =
            unsafe { std::mem::transmute(prepared.dispatch_hook_address) };
        let detour =
            unsafe { GenericDetour::new(target, main_thread_dispatch_detour) }.map_err(|_| ())?;
        DISPATCH_HOOK.set(detour).map_err(|_| ())?;
        DISPATCH_TARGET
            .set(prepared.dispatch_hook_address)
            .map_err(|_| ())?;
        unsafe { DISPATCH_HOOK.get().ok_or(())?.enable() }.map_err(|_| ())?;
    }

    Ok(())
}

unsafe fn install_setter_hooks() -> Result<(), ()> {
    let state = STATE.get().ok_or(())?;
    let state = state.lock().map_err(|_| ())?;
    let session = state.session.as_ref().ok_or(())?;
    for binding in &session.bindings {
        let (slot, target_slot, detour): (
            &OnceLock<GenericDetour<ManagedSetter>>,
            &OnceLock<usize>,
            ManagedSetter,
        ) = match binding.kind {
            StandardUiKind::TextMeshPro => {
                (&TMP_SETTER_HOOK, &TMP_SETTER_TARGET, tmp_setter_detour)
            }
            StandardUiKind::UGui => (&UGUI_SETTER_HOOK, &UGUI_SETTER_TARGET, ugui_setter_detour),
        };
        if slot.get().is_some() {
            if target_slot.get().copied() != Some(binding.hook_address) {
                return Err(());
            }
            continue;
        }
        let target: ManagedSetter = unsafe { std::mem::transmute(binding.hook_address) };
        let hook = unsafe { GenericDetour::new(target, detour) }.map_err(|_| ())?;
        slot.set(hook).map_err(|_| ())?;
        target_slot.set(binding.hook_address).map_err(|_| ())?;
        unsafe { slot.get().ok_or(())?.enable() }.map_err(|_| ())?;
    }
    Ok(())
}

unsafe fn call_original_setter(
    kind: StandardUiKind,
    object: usize,
    string: usize,
) -> Result<(), ()> {
    let hook = match kind {
        StandardUiKind::TextMeshPro => TMP_SETTER_HOOK.get(),
        StandardUiKind::UGui => UGUI_SETTER_HOOK.get(),
    }
    .ok_or(())?;
    unsafe { hook.call(object as *mut c_void, string as *mut c_void) };
    Ok(())
}

#[cfg(windows)]
fn current_thread_id() -> u32 {
    unsafe { windows::Win32::System::Threading::GetCurrentThreadId() }
}

#[cfg(not(windows))]
fn current_thread_id() -> u32 {
    1
}

unsafe extern "C" fn tmp_setter_detour(object: *mut c_void, string: *mut c_void) {
    unsafe { dispatch_setter(StandardUiKind::TextMeshPro, object, string) };
}

unsafe extern "C" fn ugui_setter_detour(object: *mut c_void, string: *mut c_void) {
    unsafe { dispatch_setter(StandardUiKind::UGui, object, string) };
}

unsafe fn dispatch_setter(kind: StandardUiKind, object: *mut c_void, string: *mut c_void) {
    let original = match kind {
        StandardUiKind::TextMeshPro => TMP_SETTER_HOOK.get(),
        StandardUiKind::UGui => UGUI_SETTER_HOOK.get(),
    };
    let Some(original) = original else {
        return;
    };
    if !ACTIVE.load(Ordering::Acquire) || STOPPING.load(Ordering::Acquire) {
        unsafe { original.call(object, string) };
        return;
    }
    let Some(_guard) = CallbackGuard::enter() else {
        unsafe { original.call(object, string) };
        return;
    };
    let processed = std::panic::catch_unwind(|| process_setter(kind, object, string))
        .ok()
        .flatten();
    if let Some(processed) = processed {
        publish(processed.observations);
        let argument = processed
            .replacement
            .as_ref()
            .map_or(string, |replacement| replacement.target as *mut c_void);
        unsafe { original.call(object, argument) };
    } else {
        unsafe { original.call(object, string) };
    }
}

struct ProcessedSetter {
    replacement: Option<ManagedStringHandle>,
    observations: Vec<ObservedText>,
}

fn process_setter(
    kind: StandardUiKind,
    object: *mut c_void,
    string: *mut c_void,
) -> Option<ProcessedSetter> {
    let host = HOST
        .get()
        .and_then(|host| host.read().ok())
        .map(|host| *host)?;
    let state = STATE.get()?;
    let mut state = state.lock().ok()?;
    let text = unsafe {
        state
            .session
            .as_mut()?
            .setter_text(kind, object as usize, string as usize)?
    };
    let mut observations = Vec::new();
    if state.active_feature_bits & FEATURE_TEXT_OBSERVE != 0
        && state.active_feature_bits & FEATURE_TEXT_REPLACE == 0
    {
        observations = state.observer.apply(ObserverEvent::Setter(text.clone()));
    }

    let replacement = if state.active_feature_bits & FEATURE_TEXT_REPLACE != 0 {
        let on_main_thread = state.session.as_ref()?.is_main_thread();
        let previous_generation = state.writeback.known_generation().unwrap_or(0);
        let outcome = state.writeback.intercept_setter(text, |source| {
            if on_main_thread {
                host.decide(source)
                    .unwrap_or_else(|| TextDecision::keep(previous_generation))
            } else {
                TextDecision::keep(previous_generation)
            }
        });
        if outcome.generation() != Some(previous_generation) {
            state.session.as_mut()?.translations_applied = false;
        }
        match outcome {
            SetterOutcome::Replace { units, .. } if on_main_thread => unsafe {
                state.session.as_ref()?.managed_string(&units)
            },
            SetterOutcome::Untracked
            | SetterOutcome::ForwardOriginal { .. }
            | SetterOutcome::Replace { .. } => None,
        }
    } else {
        None
    };

    Some(ProcessedSetter {
        replacement,
        observations,
    })
}

unsafe extern "C" fn main_thread_dispatch_detour(this: *mut c_void) {
    let Some(original) = DISPATCH_HOOK.get() else {
        return;
    };
    unsafe { original.call(this) };
    let restoring = RESTORE_STATUS.load(Ordering::Acquire) == RESTORE_PENDING;
    if !ACTIVE.load(Ordering::Acquire) && !restoring {
        return;
    }
    let Some(_guard) = CallbackGuard::enter() else {
        return;
    };
    if restoring {
        if std::panic::catch_unwind(restore_main_thread).is_err() {
            RESTORE_STATUS.store(RESTORE_FAILED, Ordering::Release);
        }
        return;
    }
    let observations = std::panic::catch_unwind(process_main_thread).ok().flatten();
    if let Some(observations) = observations {
        publish(observations);
    }
}

fn process_main_thread() -> Option<Vec<ObservedText>> {
    let state = STATE.get()?;
    let mut state = state.lock().ok()?;
    let active_features = state.active_feature_bits;
    let snapshot_ready = state.session.as_ref()?.snapshot_ready;
    if !snapshot_ready {
        let snapshot = match unsafe { state.session.as_mut()?.initial_snapshot() } {
            Ok(snapshot) => snapshot,
            Err(()) => {
                ACTIVE.store(false, Ordering::Release);
                SNAPSHOT_STATUS.store(SNAPSHOT_REJECTED, Ordering::Release);
                return None;
            }
        };
        if snapshot.is_empty() {
            ACTIVE.store(false, Ordering::Release);
            SNAPSHOT_STATUS.store(SNAPSHOT_REJECTED, Ordering::Release);
            return None;
        }
        if active_features & FEATURE_TEXT_REPLACE != 0 {
            let _ = state
                .writeback
                .activate(snapshot.clone(), |_| TextDecision::keep(0));
        }
        let observations = if active_features & FEATURE_TEXT_OBSERVE != 0
            && active_features & FEATURE_TEXT_REPLACE == 0
        {
            state
                .observer
                .apply(ObserverEvent::AttachSnapshot(snapshot))
        } else {
            Vec::new()
        };
        state
            .session
            .as_mut()?
            .pending_observations
            .extend(observations);
        SNAPSHOT_STATUS.store(SNAPSHOT_READY, Ordering::Release);
        return None;
    }

    let (mut observations, collected, refresh_due, translations_applied) = {
        let session = state.session.as_mut()?;
        (
            std::mem::take(&mut session.pending_observations),
            unsafe { session.sweep_if_due() },
            session.refresh_due(),
            session.translations_applied,
        )
    };
    for object_id in collected {
        if active_features & FEATURE_TEXT_OBSERVE != 0
            && active_features & FEATURE_TEXT_REPLACE == 0
        {
            observations.extend(state.observer.apply(ObserverEvent::Collected(object_id)));
        }
        if active_features & FEATURE_TEXT_REPLACE != 0 {
            state.writeback.collected(object_id);
        }
    }

    // The initial snapshot wakes the controller before it installs setter hooks.
    // Do not advance writeback generations until every original setter is callable.
    let setters_ready = state
        .session
        .as_ref()?
        .bindings
        .iter()
        .all(|binding| match binding.kind {
            StandardUiKind::TextMeshPro => TMP_SETTER_HOOK.get().is_some(),
            StandardUiKind::UGui => UGUI_SETTER_HOOK.get().is_some(),
        });
    if active_features & FEATURE_TEXT_REPLACE != 0
        && setters_ready
        && (!translations_applied || refresh_due)
    {
        let host = HOST
            .get()
            .and_then(|host| host.read().ok())
            .map(|host| *host)?;
        let known_generation = state.writeback.known_generation().unwrap_or(0);
        let probe = state.writeback.probe_source();
        let probe_decision = probe.as_deref().and_then(|source| host.decide(source));
        let should_refresh = !translations_applied
            || probe_decision
                .as_ref()
                .is_some_and(|decision| decision.generation() != known_generation);
        if probe.is_none() {
            state.session.as_mut()?.translations_applied = true;
        } else if probe_decision.is_some() && should_refresh {
            let writes = state.writeback.refresh(|source| {
                host.decide(source)
                    .unwrap_or_else(|| TextDecision::keep(known_generation))
            });
            let applied = unsafe { state.session.as_ref()?.apply_writes(&writes) };
            state.session.as_mut()?.translations_applied = applied;
        }
    }
    Some(observations)
}

fn restore_main_thread() {
    let restored = (|| {
        let state = STATE.get()?;
        let mut state = state.lock().ok()?;
        if !state.session.as_ref()?.is_main_thread() {
            return None;
        }
        let writes = state.writeback.deactivate();
        let restored = unsafe { state.session.as_ref()?.apply_writes(&writes) };
        if !restored {
            return None;
        }
        let mut session = state.session.take()?;
        unsafe { session.release_handles() };
        let _ = state.observer.apply(ObserverEvent::Deactivate);
        state.active_feature_bits = 0;
        ACTIVE.store(false, Ordering::Release);
        STOPPING.store(false, Ordering::Release);
        SNAPSHOT_STATUS.store(SNAPSHOT_IDLE, Ordering::Release);
        Some(())
    })()
    .is_some();
    RESTORE_STATUS.store(
        if restored {
            RESTORE_COMPLETE
        } else {
            RESTORE_FAILED
        },
        Ordering::Release,
    );
}

fn publish(observations: Vec<ObservedText>) {
    let Some(host) = HOST
        .get()
        .and_then(|host| host.read().ok())
        .map(|host| *host)
    else {
        return;
    };
    for observation in observations {
        let source = observation.source().encode_utf16().collect::<Vec<_>>();
        if source.is_empty() || source.len() > MAX_TEXT_UNITS {
            continue;
        }
        let mut text = vec![0_u16; MAX_TEXT_UNITS];
        let mut font = vec![0_u16; 63];
        let _ = (host.decide_utf16)(
            host.context as *mut c_void,
            source.as_ptr(),
            source.len() as u32,
            text.as_mut_ptr(),
            text.len() as u32,
            font.as_mut_ptr(),
            font.len() as u32,
        );
    }
}

#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: std::mem::size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::retained_target(
            ADAPTER_ID,
            (0, 1, 0),
            FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE,
            PLATFORM_WINDOWS,
            ARCH_X86 | ARCH_X86_64,
        ),
        negotiate_features,
        activate,
        deactivate,
        request_refresh: glyphshift_adapter_native_abi::request_refresh_noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_descriptor_is_retained_standard_ui_windows_x64() {
        let api = glyphshift_adapter_entry_v1();

        assert_eq!(api.descriptor.adapter_id.as_str(), Ok(ADAPTER_ID));
        assert_eq!(
            api.descriptor.feature_bits,
            FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE
        );
        assert_eq!(api.descriptor.platform_bits, PLATFORM_WINDOWS);
        assert_eq!(api.descriptor.architecture_bits, ARCH_X86 | ARCH_X86_64);
        assert_eq!(api.descriptor.version_major, 0);
        assert_eq!(api.descriptor.version_minor, 1);
        assert_eq!(api.descriptor.version_patch, 0);
    }

    #[test]
    fn native_negotiation_accepts_granted_observation_or_replacement() {
        assert_eq!(
            negotiate_features(FEATURE_TEXT_OBSERVE, FEATURE_TEXT_OBSERVE).status,
            STATUS_OK
        );
        assert_eq!(
            negotiate_features(FEATURE_TEXT_OBSERVE, 0).status,
            STATUS_UNAUTHORIZED_FEATURE
        );
        assert_eq!(
            negotiate_features(FEATURE_TEXT_REPLACE, FEATURE_TEXT_REPLACE).status,
            STATUS_OK
        );
        assert_eq!(
            negotiate_features(1 << 63, u64::MAX).status,
            STATUS_UNSUPPORTED_FEATURE
        );
    }
}
