use crate::diagnostics::trace;
use crate::metadata::Il2CppStandardUiRuntime;
use crate::observer_loop::ObserverLoop;
use crate::runtime_gate::Il2CppRuntimeGate;
use glyphshift_adapter_native_abi::{
    DecideUtf16V1, NativeAdapterApiV1, NativeAdapterDescriptorV1, NativeNegotiationV1,
    NativeRuntimeHostV1, ARCH_X86_64, FEATURE_TEXT_OBSERVE, PLATFORM_WINDOWS,
    STATUS_ACTIVATION_FAILED, STATUS_INVALID_HOST, STATUS_OK, STATUS_UNAUTHORIZED_FEATURE,
    STATUS_UNSUPPORTED_FEATURE,
};
use glyphshift_adapter_unity_il2cpp_standard_ui::ADAPTER_ID;
use glyphshift_adapter_unity_standard_ui::{
    ManagedText, ObservedText, ObserverEvent, UnityStandardUiObserver, MAX_TEXT_UNITS,
};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const SNAPSHOT_IDLE: u8 = 0;
const SNAPSHOT_COMPLETE: u8 = 1;
const SNAPSHOT_FAILED: u8 = 2;
const INITIAL_SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Copy)]
struct HostBridge {
    context: usize,
    decide_utf16: DecideUtf16V1,
}

struct ActiveSession {
    runtime: Il2CppStandardUiRuntime,
    observer: UnityStandardUiObserver,
    host: HostBridge,
    initial_snapshot: bool,
    pending_observations: Vec<ObservedText>,
}

static ACTIVE: AtomicBool = AtomicBool::new(false);
static COLLECTING: AtomicBool = AtomicBool::new(false);
static SNAPSHOT_STATUS: AtomicU8 = AtomicU8::new(SNAPSHOT_IDLE);
static SESSION: OnceLock<Mutex<Option<ActiveSession>>> = OnceLock::new();
static OBSERVER_LOOP: OnceLock<Mutex<Option<ObserverLoop>>> = OnceLock::new();

extern "C" fn negotiate_features(requested: u64, granted: u64) -> NativeNegotiationV1 {
    if requested & !FEATURE_TEXT_OBSERVE != 0 {
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
    if negotiated.active_feature_bits & FEATURE_TEXT_OBSERVE == 0 {
        return negotiated;
    }
    let _ = deactivate();

    let gate = match Il2CppRuntimeGate::inspect_current_process() {
        Ok(gate) => gate,
        Err(error) => {
            trace(&format!("gate.error.{error:?}"));
            return negotiation_error(STATUS_ACTIVATION_FAILED);
        }
    };
    trace("gate.ok");
    let runtime = match Il2CppStandardUiRuntime::resolve(gate) {
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
        host: HostBridge {
            context: host.context as usize,
            decide_utf16: host.decide_utf16,
        },
        initial_snapshot: false,
        pending_observations: Vec::new(),
    };
    let sessions = SESSION.get_or_init(|| Mutex::new(None));
    let Ok(mut current) = sessions.lock() else {
        return negotiation_error(STATUS_ACTIVATION_FAILED);
    };
    *current = Some(session);
    drop(current);

    SNAPSHOT_STATUS.store(SNAPSHOT_IDLE, Ordering::Release);
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
    ACTIVE.store(false, Ordering::Release);
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
    if let Some(sessions) = SESSION.get() {
        if let Ok(mut current) = sessions.lock() {
            if let Some(session) = current.as_mut() {
                let _ = session.observer.apply(ObserverEvent::Deactivate);
            }
            *current = None;
        }
    }
}

fn collect_on_observer_thread() {
    if !ACTIVE.load(Ordering::Acquire) || COLLECTING.swap(true, Ordering::AcqRel) {
        return;
    }
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
        let sessions = SESSION.get()?;
        let mut current = sessions.try_lock().ok()?;
        let session = current.as_mut()?;
        let snapshot = match unsafe { session.runtime.snapshot_on_attached_thread() } {
            Ok(snapshot) => snapshot,
            Err(error) => {
                trace(&format!("snapshot.runtime.error.{error:?}"));
                return None;
            }
        };
        let new_observations = match process_snapshot(
            &mut session.observer,
            &mut session.initial_snapshot,
            snapshot,
        ) {
            Ok(observations) => observations,
            Err(()) => {
                trace("snapshot.content.empty");
                SNAPSHOT_STATUS.store(SNAPSHOT_FAILED, Ordering::Release);
                return None;
            }
        };
        let mut observations = std::mem::take(&mut session.pending_observations);
        observations.extend(new_observations);
        Some((session.host, observations))
    })();

    if ACTIVE.load(Ordering::Acquire) {
        if let Some((host, observations)) = collected {
            let rejected = publish(host, observations);
            if !rejected.is_empty() && ACTIVE.load(Ordering::Acquire) {
                if let Some(sessions) = SESSION.get() {
                    if let Ok(mut current) = sessions.lock() {
                        if let Some(session) = current.as_mut() {
                            session.pending_observations = rejected;
                        }
                    }
                }
            }
            SNAPSHOT_STATUS.store(SNAPSHOT_COMPLETE, Ordering::Release);
        } else if SNAPSHOT_STATUS.load(Ordering::Acquire) == SNAPSHOT_IDLE {
            SNAPSHOT_STATUS.store(SNAPSHOT_FAILED, Ordering::Release);
        }
    }
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
        let source = observation.source().encode_utf16().collect::<Vec<_>>();
        if source.is_empty() || source.len() > MAX_TEXT_UNITS {
            continue;
        }
        let mut text = vec![0_u16; MAX_TEXT_UNITS];
        let mut font = vec![0_u16; 63];
        let decision = (host.decide_utf16)(
            host.context as *mut c_void,
            source.as_ptr(),
            source.len() as u32,
            text.as_mut_ptr(),
            text.len() as u32,
            font.as_mut_ptr(),
            font.len() as u32,
        );
        if decision.status != STATUS_OK {
            rejected.push(observation);
        }
    }
    rejected
}

#[no_mangle]
pub extern "C" fn glyphshift_adapter_entry_v1() -> NativeAdapterApiV1 {
    NativeAdapterApiV1 {
        struct_size: std::mem::size_of::<NativeAdapterApiV1>() as u32,
        descriptor: NativeAdapterDescriptorV1::observe_target(
            ADAPTER_ID,
            (0, 1, 0),
            FEATURE_TEXT_OBSERVE,
            PLATFORM_WINDOWS,
            ARCH_X86_64,
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
    use glyphshift_adapter_native_abi::NativeDecisionV1;
    use glyphshift_adapter_unity_standard_ui::{ManagedObjectId, StandardUiKind};
    use std::sync::atomic::AtomicUsize;

    static DECISION_ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

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
    fn native_descriptor_and_negotiation_are_observe_only_windows_x64() {
        let api = glyphshift_adapter_entry_v1();
        assert_eq!(api.descriptor.adapter_id.as_str(), Ok(ADAPTER_ID));
        assert_eq!(api.descriptor.feature_bits, FEATURE_TEXT_OBSERVE);
        assert_eq!(api.descriptor.platform_bits, PLATFORM_WINDOWS);
        assert_eq!(api.descriptor.architecture_bits, ARCH_X86_64);
        assert_eq!(
            negotiate_features(FEATURE_TEXT_OBSERVE, FEATURE_TEXT_OBSERVE).status,
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
}
