//! Host-independent observation contracts shared by Unity managed backends.

mod observer_driver;

pub use observer_driver::{
    ObserverDriver, ObserverDriverError, StandardUiObservationRuntime, StandardUiProfile,
};

use std::collections::BTreeMap;

pub const MAX_TEXT_UNITS: usize = 16 * 1024;
pub const MAX_TRACKED_OBJECTS: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ManagedObjectId(u64);

impl ManagedObjectId {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StandardUiKind {
    TextMeshPro,
    UGui,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedText {
    object_id: ManagedObjectId,
    kind: StandardUiKind,
    units: Vec<u16>,
}

impl ManagedText {
    #[must_use]
    pub fn utf16(
        object_id: ManagedObjectId,
        kind: StandardUiKind,
        units: impl Into<Vec<u16>>,
    ) -> Self {
        Self {
            object_id,
            kind,
            units: units.into(),
        }
    }

    #[must_use]
    pub fn text(object_id: ManagedObjectId, kind: StandardUiKind, text: &str) -> Self {
        Self::utf16(object_id, kind, text.encode_utf16().collect::<Vec<_>>())
    }

    #[must_use]
    pub fn into_decoded_parts(self) -> Option<(ManagedObjectId, StandardUiKind, Box<str>)> {
        if !self.object_id.is_valid() || self.units.len() > MAX_TEXT_UNITS {
            return None;
        }
        let source = String::from_utf16(&self.units).ok()?.into_boxed_str();
        Some((self.object_id, self.kind, source))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedText {
    object_id: ManagedObjectId,
    kind: StandardUiKind,
    source: Box<str>,
}

impl ObservedText {
    #[must_use]
    pub const fn object_id(&self) -> ManagedObjectId {
        self.object_id
    }

    #[must_use]
    pub const fn kind(&self) -> StandardUiKind {
        self.kind
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObserverEvent {
    AttachSnapshot(Vec<ManagedText>),
    RefreshSnapshot(Vec<ManagedText>),
    Setter(ManagedText),
    Collected(ManagedObjectId),
    Deactivate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TrackedText {
    kind: StandardUiKind,
    source: Box<str>,
}

#[derive(Debug, Default)]
pub struct UnityStandardUiObserver {
    active: bool,
    tracked: BTreeMap<ManagedObjectId, TrackedText>,
}

impl UnityStandardUiObserver {
    #[must_use]
    pub fn apply(&mut self, event: ObserverEvent) -> Vec<ObservedText> {
        match event {
            ObserverEvent::AttachSnapshot(snapshot) => self.attach(snapshot),
            ObserverEvent::RefreshSnapshot(snapshot) => self.refresh(snapshot),
            ObserverEvent::Setter(text) => self.setter(text).into_iter().collect(),
            ObserverEvent::Collected(object_id) => {
                if self.active {
                    self.tracked.remove(&object_id);
                }
                Vec::new()
            }
            ObserverEvent::Deactivate => {
                self.active = false;
                self.tracked.clear();
                Vec::new()
            }
        }
    }

    fn attach(&mut self, snapshot: Vec<ManagedText>) -> Vec<ObservedText> {
        self.active = true;
        self.tracked = decode_snapshot(snapshot);
        observations_for_all(&self.tracked)
    }

    fn refresh(&mut self, snapshot: Vec<ManagedText>) -> Vec<ObservedText> {
        if !self.active {
            return Vec::new();
        }

        let next = decode_snapshot(snapshot);
        let observations = next
            .iter()
            .filter(|(object_id, text)| {
                !text.source.is_empty()
                    && self.tracked.get(object_id).is_none_or(|previous| {
                        previous.kind != text.kind || previous.source != text.source
                    })
            })
            .map(|(object_id, text)| observed(*object_id, text))
            .collect();
        self.tracked = next;
        observations
    }

    fn setter(&mut self, text: ManagedText) -> Option<ObservedText> {
        if !self.active {
            return None;
        }
        let (object_id, kind, source) = text.into_decoded_parts()?;
        if self.tracked.len() == MAX_TRACKED_OBJECTS && !self.tracked.contains_key(&object_id) {
            return None;
        }
        if self
            .tracked
            .get(&object_id)
            .is_some_and(|previous| previous.kind == kind && previous.source == source)
        {
            return None;
        }

        let observation = (!source.is_empty()).then(|| ObservedText {
            object_id,
            kind,
            source: source.clone(),
        });
        self.tracked.insert(object_id, TrackedText { kind, source });
        observation
    }
}

fn decode_snapshot(snapshot: Vec<ManagedText>) -> BTreeMap<ManagedObjectId, TrackedText> {
    let mut decoded = BTreeMap::new();
    for text in snapshot {
        let Some((object_id, kind, source)) = text.into_decoded_parts() else {
            continue;
        };
        if decoded.len() == MAX_TRACKED_OBJECTS && !decoded.contains_key(&object_id) {
            continue;
        }
        decoded.insert(object_id, TrackedText { kind, source });
    }
    decoded
}

fn observations_for_all(tracked: &BTreeMap<ManagedObjectId, TrackedText>) -> Vec<ObservedText> {
    tracked
        .iter()
        .filter(|(_, text)| !text.source.is_empty())
        .map(|(object_id, text)| observed(*object_id, text))
        .collect()
}

fn observed(object_id: ManagedObjectId, text: &TrackedText) -> ObservedText {
    ObservedText {
        object_id,
        kind: text.kind,
        source: text.source.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(id: u64, kind: StandardUiKind, source: &str) -> ManagedText {
        ManagedText::text(ManagedObjectId::new(id), kind, source)
    }

    #[test]
    fn attach_emits_one_final_valid_value_per_standard_ui_object() {
        let mut observer = UnityStandardUiObserver::default();
        let observations = observer.apply(ObserverEvent::AttachSnapshot(vec![
            text(2, StandardUiKind::UGui, "Open"),
            text(1, StandardUiKind::TextMeshPro, "Score: 0"),
            text(1, StandardUiKind::TextMeshPro, "Score: 248"),
            text(3, StandardUiKind::TextMeshPro, ""),
            ManagedText::utf16(
                ManagedObjectId::new(4),
                StandardUiKind::TextMeshPro,
                [0xd800],
            ),
            text(0, StandardUiKind::UGui, "invalid identity"),
        ]));

        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].object_id(), ManagedObjectId::new(1));
        assert_eq!(observations[0].source(), "Score: 248");
        assert_eq!(observations[1].kind(), StandardUiKind::UGui);
        assert_eq!(observations[1].source(), "Open");
    }

    #[test]
    fn refresh_emits_only_new_or_changed_values_and_forgets_collected_objects() {
        let mut observer = UnityStandardUiObserver::default();
        let _ = observer.apply(ObserverEvent::AttachSnapshot(vec![
            text(1, StandardUiKind::TextMeshPro, "Score: 0"),
            text(2, StandardUiKind::UGui, "Open"),
        ]));

        let observations = observer.apply(ObserverEvent::RefreshSnapshot(vec![
            text(1, StandardUiKind::TextMeshPro, "Score: 248"),
            text(3, StandardUiKind::UGui, "Settings"),
        ]));
        assert_eq!(observations.len(), 2);
        assert_eq!(observations[0].source(), "Score: 248");
        assert_eq!(observations[1].source(), "Settings");

        assert!(observer
            .apply(ObserverEvent::RefreshSnapshot(vec![
                text(1, StandardUiKind::TextMeshPro, "Score: 248"),
                text(3, StandardUiKind::UGui, "Settings"),
            ]))
            .is_empty());
        assert_eq!(
            observer.apply(ObserverEvent::Setter(
                text(2, StandardUiKind::UGui, "Open",)
            ))[0]
                .source(),
            "Open"
        );
    }

    #[test]
    fn setter_deactivation_and_invalid_inputs_keep_observer_bounded() {
        let mut observer = UnityStandardUiObserver::default();
        let _ = observer.apply(ObserverEvent::AttachSnapshot(vec![text(
            7,
            StandardUiKind::TextMeshPro,
            "Score: 0",
        )]));
        assert!(observer
            .apply(ObserverEvent::Setter(text(
                7,
                StandardUiKind::TextMeshPro,
                "Score: 0",
            )))
            .is_empty());
        assert_eq!(
            observer.apply(ObserverEvent::Setter(text(
                7,
                StandardUiKind::TextMeshPro,
                "Score: 248",
            )))[0]
                .source(),
            "Score: 248"
        );
        assert!(observer
            .apply(ObserverEvent::Setter(ManagedText::utf16(
                ManagedObjectId::new(8),
                StandardUiKind::UGui,
                [0xd800],
            )))
            .is_empty());

        let _ = observer.apply(ObserverEvent::Deactivate);
        assert!(observer
            .apply(ObserverEvent::RefreshSnapshot(vec![text(
                7,
                StandardUiKind::TextMeshPro,
                "Score: 999",
            )]))
            .is_empty());
    }
}
