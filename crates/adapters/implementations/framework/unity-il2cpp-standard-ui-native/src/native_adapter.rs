use crate::diagnostics::trace;
use crate::font_substitution::FontWrite;
use crate::main_thread;
use crate::metadata::Il2CppStandardUiRuntime;
use crate::observer_loop::ObserverLoop;
use crate::runtime_gate::Il2CppRuntimeGate;
use glyphshift_adapter_native_abi::{
    DecideUtf16V1, NativeAdapterApiV1, NativeAdapterDescriptorV1, NativeNegotiationV1,
    NativeRuntimeHostV1, ARCH_X86_64, DECISION_FONT_SUBSTITUTE, DECISION_TEXT_REPLACE,
    FEATURE_FONT_SUBSTITUTE, FEATURE_TEXT_OBSERVE, FEATURE_TEXT_REPLACE, PLATFORM_WINDOWS,
    STATUS_ACTIVATION_FAILED, STATUS_INVALID_HOST, STATUS_OK, STATUS_UNAUTHORIZED_FEATURE,
    STATUS_UNSUPPORTED_FEATURE,
};
use glyphshift_adapter_unity_il2cpp_standard_ui::ADAPTER_ID;
use glyphshift_adapter_unity_standard_ui::{
    ManagedText, ObservedText, ObserverEvent, TextDecision, TextWrite, UnityStandardUiObserver,
    UnityStandardUiWriteback, MAX_TEXT_UNITS,
};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const SNAPSHOT_IDLE: u8 = 0;
const SNAPSHOT_COMPLETE: u8 = 1;
const SNAPSHOT_FAILED: u8 = 2;
const INITIAL_SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(3);
const MAIN_THREAD_DISPATCH_TIMEOUT: Duration = Duration::from_millis(750);
const RESTORE_TIMEOUT: Duration = Duration::from_secs(3);
const MUTATING_FEATURES: u64 = FEATURE_TEXT_REPLACE | FEATURE_FONT_SUBSTITUTE;
const SUPPORTED_FEATURES: u64 = FEATURE_TEXT_OBSERVE | MUTATING_FEATURES;

struct HostDecision {
    text: TextDecision,
    replacement: Option<Vec<u16>>,
    font: Option<Box<str>>,
}

#[derive(Clone, Copy)]
struct HostBridge {
    context: usize,
    decide_utf16: DecideUtf16V1,
}

impl HostBridge {
    fn decide(self, source: &str) -> Option<HostDecision> {
        let source = source.encode_utf16().collect::<Vec<_>>();
        if source.is_empty() || source.len() > MAX_TEXT_UNITS {
            return None;
        }
        let mut replacement = vec![0_u16; MAX_TEXT_UNITS];
        let mut font = vec![0_u16; 256];
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
        let replacement = if decision.decision_bits & DECISION_TEXT_REPLACE != 0 {
            let length = usize::try_from(decision.text_len).ok()?;
            if length > replacement.len() {
                return None;
            }
            replacement.truncate(length);
            Some(replacement)
        } else {
            None
        };
        let font = if decision.decision_bits & DECISION_FONT_SUBSTITUTE != 0 {
            let length = usize::try_from(decision.font_len).ok()?;
            if length == 0 || length > font.len() {
                return None;
            }
            font.truncate(length);
            let family = String::from_utf16(&font).ok()?;
            let family = family.trim();
            (!family.is_empty()).then(|| Box::<str>::from(family))
        } else {
            None
        };
        let text = replacement.as_ref().map_or_else(
            || TextDecision::keep(decision.generation),
            |replacement| TextDecision::replace_utf16(decision.generation, replacement.clone()),
        );
        Some(HostDecision {
            text,
            replacement,
            font,
        })
    }
}

struct ActiveSession {
    runtime: Il2CppStandardUiRuntime,
    observer: UnityStandardUiObserver,
    writeback: UnityStandardUiWriteback,
    host: HostBridge,
    active_feature_bits: u64,
    initial_snapshot: bool,
    pending_observations: Vec<ObservedText>,
    pending_writeback_snapshot: Option<Vec<ManagedText>>,
    writeback_started: bool,
    force_restore_all: bool,
}

#[derive(Default)]
struct SessionSlot {
    session: Option<ActiveSession>,
    in_flight: bool,
    deactivating: bool,
    deactivate_requested: bool,
}

static ACTIVE: AtomicBool = AtomicBool::new(false);
static COLLECTING: AtomicBool = AtomicBool::new(false);
static REFRESH_REQUESTED: AtomicBool = AtomicBool::new(false);
static SNAPSHOT_STATUS: AtomicU8 = AtomicU8::new(SNAPSHOT_IDLE);
static SESSION: OnceLock<Mutex<SessionSlot>> = OnceLock::new();
static OBSERVER_LOOP: OnceLock<Mutex<Option<ObserverLoop>>> = OnceLock::new();

fn sessions() -> &'static Mutex<SessionSlot> {
    SESSION.get_or_init(|| Mutex::new(SessionSlot::default()))
}

fn install_session(session: ActiveSession) -> bool {
    let Ok(mut slot) = sessions().lock() else {
        return false;
    };
    if slot.in_flight || slot.deactivating || slot.session.is_some() {
        return false;
    }
    slot.deactivate_requested = false;
    slot.session = Some(session);
    true
}

fn begin_session_transaction() -> Option<ActiveSession> {
    let mut slot = sessions().try_lock().ok()?;
    if slot.in_flight || slot.deactivating {
        return None;
    }
    let session = slot.session.take()?;
    slot.in_flight = true;
    Some(session)
}

fn begin_deactivation_transaction() -> Option<ActiveSession> {
    let mut slot = sessions().try_lock().ok()?;
    if slot.in_flight || !slot.deactivating {
        return None;
    }
    let session = slot.session.take()?;
    slot.in_flight = true;
    Some(session)
}

fn deactivate_requested() -> bool {
    sessions()
        .lock()
        .map(|slot| slot.deactivate_requested)
        .unwrap_or(true)
}

fn finish_session_transaction(session: Option<ActiveSession>) {
    if let Ok(mut slot) = sessions().lock() {
        slot.in_flight = false;
        slot.deactivate_requested = false;
        if slot.session.is_none() {
            slot.session = session;
        }
    }
}

fn cancel_deactivation() {
    if let Ok(mut slot) = sessions().lock() {
        slot.deactivating = false;
        slot.deactivate_requested = false;
    }
}

fn try_begin_deactivation(slot: &mut SessionSlot) -> bool {
    if slot.in_flight {
        slot.deactivate_requested = true;
        return false;
    }
    if slot.deactivating {
        return false;
    }
    slot.deactivating = true;
    true
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
    if negotiated.active_feature_bits & SUPPORTED_FEATURES == 0 {
        return negotiated;
    }
    if deactivate() != STATUS_OK {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }

    let require_writeback = negotiated.active_feature_bits & MUTATING_FEATURES != 0;
    let gate = match Il2CppRuntimeGate::inspect_current_process(require_writeback) {
        Ok(gate) => gate,
        Err(error) => {
            trace(&format!("gate.error.{error:?}"));
            return negotiation_error(STATUS_ACTIVATION_FAILED);
        }
    };
    trace("gate.ok");
    let runtime = match Il2CppStandardUiRuntime::resolve(
        gate,
        negotiated.active_feature_bits & FEATURE_FONT_SUBSTITUTE != 0,
    ) {
        Ok(runtime) => runtime,
        Err(error) => {
            trace(&format!("metadata.error.{error:?}"));
            return negotiation_error(STATUS_ACTIVATION_FAILED);
        }
    };
    trace("metadata.ok");
    if runtime.profile().is_empty() {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }
    let host = unsafe { *host };
    let session = ActiveSession {
        runtime,
        observer: UnityStandardUiObserver::default(),
        writeback: UnityStandardUiWriteback::default(),
        host: HostBridge {
            context: host.context as usize,
            decide_utf16: host.decide_utf16,
        },
        active_feature_bits: negotiated.active_feature_bits,
        initial_snapshot: false,
        pending_observations: Vec::new(),
        pending_writeback_snapshot: None,
        writeback_started: false,
        force_restore_all: false,
    };
    if !install_session(session) {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }

    if negotiated.active_feature_bits & MUTATING_FEATURES != 0
        && !main_thread::dispatch(probe_main_thread, MAIN_THREAD_DISPATCH_TIMEOUT)
    {
        trace("writeback.main-thread.unavailable");
        clear_session();
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    }

    SNAPSHOT_STATUS.store(SNAPSHOT_IDLE, Ordering::Release);
    REFRESH_REQUESTED.store(false, Ordering::Release);
    ACTIVE.store(true, Ordering::Release);
    let observer_loop = match ObserverLoop::start(collect_on_observer_thread) {
        Ok(observer_loop) => observer_loop,
        Err(error) => {
            trace(&format!("observer.error.{error:?}"));
            clear_session();
            return negotiation_error(STATUS_ACTIVATION_FAILED);
        }
    };
    trace("observer.ok");
    let observer_loops = OBSERVER_LOOP.get_or_init(|| Mutex::new(None));
    let Ok(mut current) = observer_loops.lock() else {
        observer_loop.stop();
        clear_session();
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    };
    *current = Some(observer_loop);
    drop(current);

    let started = Instant::now();
    while started.elapsed() < INITIAL_SNAPSHOT_TIMEOUT {
        match SNAPSHOT_STATUS.load(Ordering::Acquire) {
            SNAPSHOT_COMPLETE => {
                trace("snapshot.initial.ok");
                return negotiated;
            }
            SNAPSHOT_FAILED => break,
            _ => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    trace("snapshot.initial.failed");
    let _ = deactivate();
    negotiation_error(STATUS_ACTIVATION_FAILED)
}

extern "C" fn deactivate() -> i32 {
    let can_deactivate = sessions()
        .lock()
        .map(|mut slot| try_begin_deactivation(&mut slot))
        .unwrap_or(false);
    if !can_deactivate {
        trace("writeback.deactivate.busy");
        return STATUS_ACTIVATION_FAILED;
    }

    let needs_restore = sessions()
        .lock()
        .ok()
        .and_then(|slot| {
            slot.session.as_ref().map(|session| {
                session.active_feature_bits & MUTATING_FEATURES != 0
                    && (session.force_restore_all
                        || session.writeback.has_applied_replacements()
                        || session.runtime.has_font_substitutions())
            })
        })
        .unwrap_or(false);
    if needs_restore && !main_thread::dispatch(restore_on_main_thread, RESTORE_TIMEOUT) {
        trace("writeback.restore.failed");
        cancel_deactivation();
        return STATUS_ACTIVATION_FAILED;
    }

    // Only publish the inactive state after restoration succeeded. While the
    // deactivation gate is held, observer/main-thread transactions cannot
    // start, so a failed stop leaves the Adapter fully active and retryable.
    ACTIVE.store(false, Ordering::Release);
    REFRESH_REQUESTED.store(false, Ordering::Release);
    let observer_loop = OBSERVER_LOOP
        .get()
        .and_then(|observer_loop| observer_loop.lock().ok()?.take());
    if let Some(observer_loop) = observer_loop {
        observer_loop.stop();
    }
    clear_session();
    SNAPSHOT_STATUS.store(SNAPSHOT_IDLE, Ordering::Release);
    STATUS_OK
}

fn clear_session() {
    ACTIVE.store(false, Ordering::Release);
    REFRESH_REQUESTED.store(false, Ordering::Release);
    let session = if let Ok(mut slot) = sessions().lock() {
        if slot.in_flight {
            slot.deactivate_requested = true;
            None
        } else {
            slot.deactivating = false;
            slot.deactivate_requested = false;
            slot.session.take()
        }
    } else {
        None
    };
    if let Some(mut session) = session {
        let _ = session.observer.apply(ObserverEvent::Deactivate);
        let _ = session.writeback.deactivate();
        unsafe { session.runtime.release_object_handles() };
    }
}

fn probe_main_thread() -> bool {
    let Some(session) = begin_session_transaction() else {
        return false;
    };
    let result = unsafe { session.runtime.apply_writes_on_current_thread(&[]) }.is_ok();
    finish_session_transaction(Some(session));
    result
}

fn restore_on_main_thread() -> bool {
    let Some(mut session) = begin_deactivation_transaction() else {
        return false;
    };
    let mut restored_state = session.writeback.clone();
    let writes = if session.force_restore_all {
        restored_state.deactivate_all()
    } else {
        restored_state.deactivate()
    };
    let fonts_restored = unsafe { session.runtime.restore_fonts_on_current_thread() };
    let restored = match fonts_restored
        .map_err(|error| trace(&format!("font.restore.error.{error:?}")))
        .and_then(|_| {
            unsafe { session.runtime.apply_writes_on_current_thread(&writes) }
                .map_err(|error| trace(&format!("writeback.restore.error.{error:?}")))
        })
    {
        Ok(()) => {
            session.writeback = restored_state;
            session.writeback_started = false;
            session.force_restore_all = false;
            session.pending_writeback_snapshot = None;
            trace("writeback.restore.ok");
            true
        }
        Err(()) => {
            false
        }
    };
    finish_session_transaction(Some(session));
    restored
}

fn collect_on_observer_thread() {
    if !ACTIVE.load(Ordering::Acquire) || COLLECTING.swap(true, Ordering::AcqRel) {
        return;
    }
    let refresh_requested = REFRESH_REQUESTED.swap(false, Ordering::AcqRel);
    if SNAPSHOT_STATUS.load(Ordering::Acquire) == SNAPSHOT_IDLE {
        trace("snapshot.callback.enter");
    }
    struct CollectionGuard;
    impl Drop for CollectionGuard {
        fn drop(&mut self) {
            COLLECTING.store(false, Ordering::Release);
        }
    }
    let _guard = CollectionGuard;

    let collected = (|| {
        let mut session = begin_session_transaction()?;
        let snapshot = match unsafe { session.runtime.snapshot_on_attached_thread() } {
            Ok(snapshot) => snapshot,
            Err(error) => {
                trace(&format!("snapshot.runtime.error.{error:?}"));
                finish_session_transaction(Some(session));
                return None;
            }
        };
        let observer_snapshot = if session.active_feature_bits & FEATURE_TEXT_REPLACE != 0 {
            session.writeback.business_snapshot(&snapshot.texts)
        } else {
            snapshot.texts.clone()
        };
        if session.active_feature_bits & MUTATING_FEATURES != 0 {
            session.pending_writeback_snapshot = Some(snapshot.texts.clone());
        }
        let new_observations = match process_snapshot(
            &mut session.observer,
            &mut session.initial_snapshot,
            observer_snapshot,
        ) {
            Ok(observations) => observations,
            Err(()) => {
                trace("snapshot.content.empty");
                SNAPSHOT_STATUS.store(SNAPSHOT_FAILED, Ordering::Release);
                finish_session_transaction(Some(session));
                return None;
            }
        };
        for object_id in snapshot.collected {
            let _ = session.observer.apply(ObserverEvent::Collected(object_id));
            session.writeback.collected(object_id);
            session.runtime.mark_collected_font_object(object_id);
        }
        let mut observations = std::mem::take(&mut session.pending_observations);
        observations.extend(new_observations);

        // Keep transaction ownership across Host callbacks. The global mutex
        // is not held, but a re-entrant deactivate observes `in_flight` and
        // must return failure instead of claiming the Adapter is unload-safe
        // while this callback is still on the stack.
        let active_feature_bits = session.active_feature_bits;
        let rejected = if active_feature_bits & FEATURE_TEXT_OBSERVE != 0 {
            publish(session.host, observations)
        } else {
            Vec::new()
        };
        if !rejected.is_empty() && ACTIVE.load(Ordering::Acquire) {
            session.pending_observations = rejected;
        }
        let stop_requested = deactivate_requested();
        finish_session_transaction(Some(session));
        Some((active_feature_bits, stop_requested))
    })();

    if ACTIVE.load(Ordering::Acquire) {
        if let Some((active_feature_bits, stop_requested)) = collected {
            if stop_requested {
                trace("writeback.transaction.cancelled-by-stop");
                return;
            }
            let writeback_ok = active_feature_bits & MUTATING_FEATURES == 0
                || main_thread::dispatch(
                    pump_writeback_on_main_thread,
                    MAIN_THREAD_DISPATCH_TIMEOUT,
                );
            if writeback_ok {
                SNAPSHOT_STATUS.store(SNAPSHOT_COMPLETE, Ordering::Release);
            } else if SNAPSHOT_STATUS.load(Ordering::Acquire) == SNAPSHOT_IDLE {
                trace("writeback.initial.failed");
                SNAPSHOT_STATUS.store(SNAPSHOT_FAILED, Ordering::Release);
            } else {
                trace("writeback.refresh.deferred");
            }
        } else {
            if refresh_requested {
                let _ = main_thread::dispatch(
                    pump_writeback_on_main_thread,
                    MAIN_THREAD_DISPATCH_TIMEOUT,
                );
            }
            if SNAPSHOT_STATUS.load(Ordering::Acquire) == SNAPSHOT_IDLE {
                SNAPSHOT_STATUS.store(SNAPSHOT_FAILED, Ordering::Release);
            }
        }
    }
}

fn pump_writeback_on_main_thread() -> bool {
    let Some(mut session) = begin_session_transaction() else {
        return false;
    };
    if session.active_feature_bits & MUTATING_FEATURES == 0 {
        finish_session_transaction(Some(session));
        return true;
    }

    if session.force_restore_all {
        let mut recovered = session.writeback.clone();
        let restores = recovered.deactivate_all();
        let recovered_ok = match unsafe { session.runtime.restore_fonts_on_current_thread() }
            .map_err(|error| trace(&format!("font.recovery.error.{error:?}")))
            .and_then(|_| {
                unsafe { session.runtime.apply_writes_on_current_thread(&restores) }
                    .map_err(|error| trace(&format!("writeback.recovery.error.{error:?}")))
            })
        {
                Ok(()) => {
                    session.writeback = recovered;
                    session.writeback_started = false;
                    session.force_restore_all = false;
                    session.pending_writeback_snapshot = None;
                    trace("writeback.recovery.ok");
                    true
                }
                Err(()) => {
                    false
                }
            };
        let _ = recovered_ok;
        finish_session_transaction(Some(session));
        return false;
    }

    let snapshot = session.pending_writeback_snapshot.take();
    let text_replace_active = session.active_feature_bits & FEATURE_TEXT_REPLACE != 0;
    let Some((next, writes)) = prepare_writeback_transaction(
        &session.writeback,
        session.writeback_started,
        snapshot,
        |source| {
            session.host.decide(source).map(|decision| {
                if text_replace_active {
                    decision.text
                } else {
                    TextDecision::keep(decision.text.generation())
                }
            })
        },
    ) else {
        finish_session_transaction(Some(session));
        return false;
    };

    if deactivate_requested() {
        // The re-entrant stop attempt already returned failure, so keep Host
        // and Adapter lifecycle state aligned. Suppress writes that have not
        // started yet and let a later explicit deactivate retry normally.
        trace("writeback.transaction.cancelled-by-stop");
        finish_session_transaction(Some(session));
        return false;
    }

    let font_writes = prepare_font_writes(
        &next,
        session.host,
        session.active_feature_bits,
    );
    let fonts_applied = if session.active_feature_bits & FEATURE_FONT_SUBSTITUTE != 0 {
        unsafe { session.runtime.apply_fonts_on_current_thread(&font_writes) }
            .map_err(|error| trace(&format!("font.apply.error.{error:?}")))
            .is_ok()
    } else {
        true
    };
    let text_applied = fonts_applied
        && (!text_replace_active
            || unsafe { session.runtime.apply_writes_on_current_thread(&writes) }
                .map_err(|error| trace(&format!("writeback.apply.error.{error:?}")))
                .is_ok());

    let result = if text_applied {
            session.writeback = next;
            session.writeback_started = true;
            session.force_restore_all = false;
            true
        } else {
            let failed_state = next.clone();
            let mut reset_state = next;
            let restores = reset_state.deactivate_all();
            let fonts_restored = unsafe { session.runtime.restore_fonts_on_current_thread() }.is_ok();
            let texts_restored = unsafe { session.runtime.apply_writes_on_current_thread(&restores) }.is_ok();
            if fonts_restored && texts_restored {
                session.writeback = reset_state;
                session.writeback_started = false;
                session.force_restore_all = false;
            } else {
                // Preserve the complete latest-source state so a later stop can
                // still attempt to restore every object after a partial write.
                session.writeback = failed_state;
                session.writeback_started = true;
                session.force_restore_all = true;
            }
            false
    };
    finish_session_transaction(Some(session));
    result
}

fn prepare_writeback_transaction(
    current: &UnityStandardUiWriteback,
    started: bool,
    snapshot: Option<Vec<ManagedText>>,
    mut decide: impl FnMut(&str) -> Option<TextDecision>,
) -> Option<(UnityStandardUiWriteback, Vec<TextWrite>)> {
    if snapshot.is_none() && !started {
        return None;
    }

    let previous_generation = current.known_generation().unwrap_or(0);
    let mut next = current.clone();
    let mut writes = if !started {
        next.activate(snapshot.unwrap_or_default(), |source| {
            decide(source).unwrap_or_else(|| TextDecision::keep(previous_generation))
        })
    } else if let Some(snapshot) = snapshot {
        next.reconcile_snapshot(snapshot, |source| {
            decide(source).unwrap_or_else(|| TextDecision::keep(previous_generation))
        })
    } else {
        Vec::new()
    };

    let known_generation = next.known_generation().unwrap_or(previous_generation);
    if let Some(probe_source) = next.probe_source() {
        if let Some(probe) = decide(&probe_source) {
            if probe.generation() != known_generation
                || next.needs_generation_refresh(probe.generation())
            {
                writes.extend(next.refresh(|source| {
                    decide(source).unwrap_or_else(|| TextDecision::keep(known_generation))
                }));
            }
        }
    }
    Some((next, writes))
}

fn prepare_font_writes(
    state: &UnityStandardUiWriteback,
    host: HostBridge,
    active_feature_bits: u64,
) -> Vec<FontWrite> {
    if active_feature_bits & FEATURE_FONT_SUBSTITUTE == 0 {
        return Vec::new();
    }
    let text_replace_active = active_feature_bits & FEATURE_TEXT_REPLACE != 0;
    state
        .business_texts()
        .into_iter()
        .filter_map(|text| {
            let (object_id, kind, source) = text.into_decoded_parts()?;
            let decision = host.decide(&source)?;
            let family = decision.font?;
            let units = if text_replace_active {
                decision
                    .replacement
                    .unwrap_or_else(|| source.encode_utf16().collect())
            } else {
                source.encode_utf16().collect()
            };
            Some(FontWrite::new(object_id, kind, family, units))
        })
        .collect()
}

fn process_snapshot(
    observer: &mut UnityStandardUiObserver,
    initial_snapshot: &mut bool,
    snapshot: Vec<ManagedText>,
) -> Result<Vec<ObservedText>, ()> {
    let was_initial = !*initial_snapshot;
    let event = if !was_initial {
        ObserverEvent::RefreshSnapshot(snapshot)
    } else {
        if snapshot.is_empty() {
            return Err(());
        }
        *initial_snapshot = true;
        ObserverEvent::AttachSnapshot(snapshot)
    };
    let observations = observer.apply(event);
    if observations.is_empty() && was_initial {
        // An empty initial observation set means that the live objects did not
        // expose usable text. Empty refreshes are ordinary de-duplication.
        return Err(());
    }
    Ok(observations)
}

fn publish(host: HostBridge, observations: Vec<ObservedText>) -> Vec<ObservedText> {
    let mut rejected = Vec::new();
    for observation in observations {
        if host.decide(observation.source()).is_none() {
            rejected.push(observation);
        }
    }
    rejected
}

extern "C" fn request_refresh() {
    if ACTIVE.load(Ordering::Acquire) {
        // Native ABI refresh is an asynchronous control-thread request. The
        // observer loop consumes it and schedules any managed setter work on
        // the verified Unity GUI thread; never block or mutate Unity here.
        REFRESH_REQUESTED.store(true, Ordering::Release);
    }
}

#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: std::mem::size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::retained_target(
            ADAPTER_ID,
            (0, 1, 0),
            SUPPORTED_FEATURES,
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
    use glyphshift_adapter_native_abi::NativeDecisionV1;
    use glyphshift_adapter_unity_standard_ui::{ManagedObjectId, StandardUiKind};
    use std::sync::atomic::AtomicUsize;

    static DECISION_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn lifecycle_gate_keeps_failed_reentrant_stop_active_and_retryable() {
        let mut slot = SessionSlot {
            in_flight: true,
            ..SessionSlot::default()
        };

        assert!(!try_begin_deactivation(&mut slot));
        assert!(slot.in_flight);
        assert!(!slot.deactivating);
        assert!(slot.deactivate_requested);

        slot.in_flight = false;
        slot.deactivate_requested = false;
        assert!(try_begin_deactivation(&mut slot));
        assert!(slot.deactivating);
    }

    #[test]
    fn lifecycle_gate_rejects_overlapping_deactivation() {
        let mut slot = SessionSlot::default();
        assert!(try_begin_deactivation(&mut slot));
        assert!(!try_begin_deactivation(&mut slot));
        assert!(slot.deactivating);
        assert!(!slot.deactivate_requested);
    }

    extern "C" fn reject_once_then_accept(
        _context: *mut c_void,
        _source: *const u16,
        _source_len: u32,
        _text_out: *mut u16,
        _text_capacity: u32,
        _font_out: *mut u16,
        _font_capacity: u32,
    ) -> NativeDecisionV1 {
        let attempt = DECISION_ATTEMPTS.fetch_add(1, Ordering::AcqRel);
        NativeDecisionV1 {
            status: if attempt == 0 {
                STATUS_ACTIVATION_FAILED
            } else {
                STATUS_OK
            },
            generation: 0,
            decision_bits: 0,
            text_len: 0,
            font_len: 0,
        }
    }

    #[test]
    fn native_descriptor_and_negotiation_are_retained_standard_ui_windows_x64() {
        let api = glyphshift_adapter_entry_v1();
        assert_eq!(api.descriptor.adapter_id.as_str(), Ok(ADAPTER_ID));
        assert_eq!(api.descriptor.feature_bits, SUPPORTED_FEATURES);
        assert_eq!(api.descriptor.platform_bits, PLATFORM_WINDOWS);
        assert_eq!(api.descriptor.architecture_bits, ARCH_X86_64);
        assert_eq!(
            negotiate_features(FEATURE_TEXT_OBSERVE, FEATURE_TEXT_OBSERVE).status,
            STATUS_OK
        );
        assert_eq!(
            negotiate_features(FEATURE_TEXT_REPLACE, FEATURE_TEXT_REPLACE).status,
            STATUS_OK
        );
        assert_eq!(
            negotiate_features(
                FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE,
                FEATURE_TEXT_OBSERVE | FEATURE_TEXT_REPLACE
            )
            .status,
            STATUS_OK
        );
        assert_eq!(
            negotiate_features(FEATURE_TEXT_OBSERVE, 0).status,
            STATUS_UNAUTHORIZED_FEATURE
        );
        assert_eq!(
            negotiate_features(1 << 63, u64::MAX).status,
            STATUS_UNSUPPORTED_FEATURE
        );
    }

    #[test]
    fn synthetic_snapshot_callback_requires_initial_text_then_deduplicates_refreshes() {
        let mut observer = UnityStandardUiObserver::default();
        let mut initial = false;
        assert_eq!(
            process_snapshot(&mut observer, &mut initial, Vec::new()),
            Err(())
        );

        let first = ManagedText::text(
            ManagedObjectId::new(7),
            StandardUiKind::TextMeshPro,
            "Score: 0",
        );
        let observations = process_snapshot(&mut observer, &mut initial, vec![first.clone()])
            .expect("initial callback");
        assert_eq!(observations[0].source(), "Score: 0");

        assert!(process_snapshot(&mut observer, &mut initial, vec![first])
            .expect("refresh callback")
            .is_empty());
    }

    #[test]
    fn observations_rejected_during_host_activation_are_retryable() {
        DECISION_ATTEMPTS.store(0, Ordering::Release);
        let mut observer = UnityStandardUiObserver::default();
        let observations = observer.apply(ObserverEvent::AttachSnapshot(vec![ManagedText::text(
            ManagedObjectId::new(9),
            StandardUiKind::UGui,
            "Open",
        )]));
        let host = HostBridge {
            context: 0,
            decide_utf16: reject_once_then_accept,
        };

        let pending = publish(host, observations);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].source(), "Open");
        assert!(publish(host, pending).is_empty());
    }

    fn managed(id: u64, source: &str) -> ManagedText {
        ManagedText::text(
            ManagedObjectId::new(id),
            StandardUiKind::TextMeshPro,
            source,
        )
    }

    fn rendered(write: &TextWrite) -> String {
        String::from_utf16(write.units()).expect("valid UTF-16 write")
    }

    #[test]
    fn writeback_transaction_handles_initial_generation_refresh_and_dynamic_source() {
        let current = UnityStandardUiWriteback::default();
        let (generation_one, writes) = prepare_writeback_transaction(
            &current,
            false,
            Some(vec![managed(1, "Score: 0")]),
            |_| Some(TextDecision::replace_text(1, "分数：0")),
        )
        .expect("initial transaction");
        assert_eq!(writes.len(), 1);
        assert_eq!(rendered(&writes[0]), "分数：0");

        // The next snapshot contains our own visible translation. It must keep
        // the saved business source while request/generation probing advances
        // the Dictionary result to generation 2.
        let (generation_two, writes) = prepare_writeback_transaction(
            &generation_one,
            true,
            Some(vec![managed(1, "分数：0")]),
            |source| {
                assert_eq!(source, "Score: 0");
                Some(TextDecision::replace_text(2, "当前分数：0"))
            },
        )
        .expect("generation 2 transaction");
        assert_eq!(writes.len(), 1);
        assert_eq!(rendered(&writes[0]), "当前分数：0");

        let (dynamic, writes) = prepare_writeback_transaction(
            &generation_two,
            true,
            Some(vec![managed(1, "Score: 248")]),
            |source| {
                assert_eq!(source, "Score: 248");
                Some(TextDecision::replace_text(2, "当前分数：248"))
            },
        )
        .expect("dynamic source transaction");
        assert_eq!(writes.len(), 1);
        assert_eq!(rendered(&writes[0]), "当前分数：248");

        let mut restored = dynamic;
        let restores = restored.deactivate();
        assert_eq!(restores.len(), 1);
        assert_eq!(rendered(&restores[0]), "Score: 248");
    }

    #[test]
    fn observer_must_not_publish_the_adapters_own_retained_replacement() {
        let mut observer = UnityStandardUiObserver::default();
        let mut initial = false;
        let source_snapshot = vec![managed(1, "Shooter")];

        let observed = process_snapshot(&mut observer, &mut initial, source_snapshot.clone())
            .expect("initial observation");
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0].source(), "Shooter");

        let (writeback, writes) = prepare_writeback_transaction(
            &UnityStandardUiWriteback::default(),
            false,
            Some(source_snapshot),
            |_| Some(TextDecision::replace_text(1, "射击")),
        )
        .expect("initial writeback");
        assert_eq!(rendered(&writes[0]), "射击");

        let echoed_snapshot = writeback.business_snapshot(&[managed(1, "射击")]);
        let echoed = process_snapshot(&mut observer, &mut initial, echoed_snapshot)
            .expect("retained replacement snapshot");
        assert!(
            echoed.is_empty(),
            "adapter output must not be emitted as a new source: writeback={:?}",
            writeback.known_generation()
        );

        let dynamic_snapshot = writeback.business_snapshot(&[managed(1, "Burst")]);
        let dynamic = process_snapshot(&mut observer, &mut initial, dynamic_snapshot)
            .expect("dynamic business snapshot");
        assert_eq!(dynamic.len(), 1);
        assert_eq!(dynamic[0].source(), "Burst");
    }

    #[test]
    fn writeback_transaction_waits_for_a_snapshot_before_starting() {
        assert!(prepare_writeback_transaction(
            &UnityStandardUiWriteback::default(),
            false,
            None,
            |_| Some(TextDecision::keep(0)),
        )
        .is_none());
    }

    #[test]
    fn dynamic_source_generation_change_refreshes_other_retained_objects() {
        let current = UnityStandardUiWriteback::default();
        let (generation_one, writes) = prepare_writeback_transaction(
            &current,
            false,
            Some(vec![managed(1, "Score: 0"), managed(2, "Open")]),
            |source| {
                Some(match source {
                    "Score: 0" => TextDecision::replace_text(1, "分数：0"),
                    "Open" => TextDecision::replace_text(1, "打开"),
                    _ => TextDecision::keep(1),
                })
            },
        )
        .expect("initial transaction");
        assert_eq!(writes.len(), 2);

        let (_generation_two, writes) = prepare_writeback_transaction(
            &generation_one,
            true,
            Some(vec![managed(1, "Score: 248"), managed(2, "打开")]),
            |source| {
                Some(match source {
                    "Score: 248" => TextDecision::replace_text(2, "当前分数：248"),
                    "Open" => TextDecision::replace_text(2, "开启"),
                    _ => TextDecision::keep(2),
                })
            },
        )
        .expect("mixed generation transaction");

        let rendered = writes.iter().map(rendered).collect::<Vec<_>>();
        assert_eq!(rendered, vec!["当前分数：248", "开启"]);
    }
}
