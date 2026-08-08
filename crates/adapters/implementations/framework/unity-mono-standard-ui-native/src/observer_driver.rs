use glyphshift_adapter_unity_mono_standard_ui::{
    ObservedText, ObserverEvent, StandardUiKind, UnityMonoObserver,
};

/// Standard Unity UI technologies recognized through managed metadata.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StandardUiProfile {
    text_mesh_pro: bool,
    ugui: bool,
}

impl StandardUiProfile {
    #[must_use]
    pub fn from_kinds(kinds: impl IntoIterator<Item = StandardUiKind>) -> Self {
        let mut profile = Self::default();
        for kind in kinds {
            match kind {
                StandardUiKind::TextMeshPro => profile.text_mesh_pro = true,
                StandardUiKind::UGui => profile.ugui = true,
            }
        }
        profile
    }

    #[must_use]
    pub const fn supports(self, kind: StandardUiKind) -> bool {
        match kind {
            StandardUiKind::TextMeshPro => self.text_mesh_pro,
            StandardUiKind::UGui => self.ugui,
        }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        !self.text_mesh_pro && !self.ugui
    }
}

/// Narrow seam between Mono-specific callbacks and host-independent observer
/// state.
///
/// A production implementation owns managed-thread attachment, managed-method
/// detours, GC handles, and unload coordination. The driver only consumes
/// serialized semantic events and never calls Mono APIs directly.
pub trait MonoObservationRuntime {
    /// Recognizes supported managed classes and their text setter methods.
    ///
    /// # Errors
    ///
    /// Returns a stable failure when the backend or standard UI metadata is not
    /// supported.
    fn recognize_standard_ui(&mut self) -> Result<StandardUiProfile, ObserverDriverError>;

    /// Returns the next serialized observation event, if one is ready.
    ///
    /// The first event after activation must be an `AttachSnapshot` produced at
    /// a verified Unity main-thread dispatch point. Implementations must not
    /// invoke Unity object enumeration from the injection thread.
    ///
    /// # Errors
    ///
    /// Returns a runtime failure without modifying target text.
    fn next_observer_event(&mut self) -> Result<Option<ObserverEvent>, ObserverDriverError>;

    /// Removes callbacks and releases runtime-owned handles.
    ///
    /// # Errors
    ///
    /// Returns a cleanup failure after attempting all safe local cleanup.
    fn stop(&mut self) -> Result<(), ObserverDriverError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverDriverError {
    BackendMismatch,
    StandardUiUnavailable,
    MainThreadDispatchUnavailable,
    InitialSnapshotUnavailable,
    UnexpectedSnapshot,
    RuntimeFailure,
    CleanupFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DriverPhase {
    AwaitingInitialSnapshot,
    Observing,
    Stopped,
}

/// Owns the observe-only session lifecycle while delegating Mono internals to a
/// replaceable runtime source.
pub struct ObserverDriver<R> {
    runtime: R,
    profile: StandardUiProfile,
    observer: UnityMonoObserver,
    phase: DriverPhase,
}

impl<R: MonoObservationRuntime> ObserverDriver<R> {
    /// Recognizes the target's standard Unity UI surface without starting text
    /// replacement.
    ///
    /// # Errors
    ///
    /// Returns the runtime's rejection, or `StandardUiUnavailable` when no TMP
    /// or uGUI text class was recognized.
    pub fn activate(mut runtime: R) -> Result<Self, ObserverDriverError> {
        let profile = match runtime.recognize_standard_ui() {
            Ok(profile) => profile,
            Err(error) => {
                let _ = runtime.stop();
                return Err(error);
            }
        };
        if profile.is_empty() {
            let _ = runtime.stop();
            return Err(ObserverDriverError::StandardUiUnavailable);
        }
        Ok(Self {
            runtime,
            profile,
            observer: UnityMonoObserver::default(),
            phase: DriverPhase::AwaitingInitialSnapshot,
        })
    }

    #[must_use]
    pub const fn profile(&self) -> StandardUiProfile {
        self.profile
    }

    /// Applies at most one serialized runtime event.
    ///
    /// # Errors
    ///
    /// Rejects a setter/collection event that arrives before the mandatory
    /// initial GC snapshot and fails closed on runtime errors.
    pub fn poll(&mut self) -> Result<Vec<ObservedText>, ObserverDriverError> {
        if self.phase == DriverPhase::Stopped {
            return Ok(Vec::new());
        }

        let event = match self.runtime.next_observer_event() {
            Ok(event) => event,
            Err(error) => {
                self.stop_best_effort();
                return Err(error);
            }
        };
        let Some(event) = event else {
            return Ok(Vec::new());
        };

        match (self.phase, &event) {
            (DriverPhase::AwaitingInitialSnapshot, ObserverEvent::AttachSnapshot(snapshot))
                if snapshot.is_empty() =>
            {
                self.stop_best_effort();
                return Err(ObserverDriverError::InitialSnapshotUnavailable);
            }
            (DriverPhase::AwaitingInitialSnapshot, ObserverEvent::AttachSnapshot(_)) => {
                self.phase = DriverPhase::Observing;
            }
            (DriverPhase::AwaitingInitialSnapshot, _) => {
                self.stop_best_effort();
                return Err(ObserverDriverError::InitialSnapshotUnavailable);
            }
            (DriverPhase::Observing, ObserverEvent::AttachSnapshot(_)) => {
                self.stop_best_effort();
                return Err(ObserverDriverError::UnexpectedSnapshot);
            }
            (DriverPhase::Observing, ObserverEvent::Deactivate) => {
                self.phase = DriverPhase::Stopped;
            }
            (DriverPhase::Observing | DriverPhase::Stopped, _) => {}
        }
        Ok(self.observer.apply(event))
    }

    /// Stops the runtime and always clears host-independent observer state.
    ///
    /// # Errors
    ///
    /// Returns a cleanup failure after local observer state has already been
    /// cleared.
    pub fn stop(&mut self) -> Result<(), ObserverDriverError> {
        if self.phase == DriverPhase::Stopped {
            return Ok(());
        }
        let result = self.runtime.stop();
        let _ = self.observer.apply(ObserverEvent::Deactivate);
        self.phase = DriverPhase::Stopped;
        result
    }

    fn stop_best_effort(&mut self) {
        let _ = self.runtime.stop();
        let _ = self.observer.apply(ObserverEvent::Deactivate);
        self.phase = DriverPhase::Stopped;
    }
}
