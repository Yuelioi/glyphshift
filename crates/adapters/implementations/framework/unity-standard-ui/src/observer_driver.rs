use crate::{ObservedText, ObserverEvent, StandardUiKind, UnityStandardUiObserver};

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

pub trait StandardUiObservationRuntime {
    fn recognize_standard_ui(&mut self) -> Result<StandardUiProfile, ObserverDriverError>;
    fn next_observer_event(&mut self) -> Result<Option<ObserverEvent>, ObserverDriverError>;
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

pub struct ObserverDriver<R> {
    runtime: R,
    profile: StandardUiProfile,
    observer: UnityStandardUiObserver,
    phase: DriverPhase,
}

impl<R: StandardUiObservationRuntime> ObserverDriver<R> {
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
            observer: UnityStandardUiObserver::default(),
            phase: DriverPhase::AwaitingInitialSnapshot,
        })
    }

    #[must_use]
    pub const fn profile(&self) -> StandardUiProfile {
        self.profile
    }

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
