//! Host-independent state contracts for Unity Mono TMP/uGUI text.

mod writeback;

pub use writeback::{SetterOutcome, TextDecision, TextWrite, UnityMonoWriteback};

use glyphshift_adapter_sdk::{AdapterDescriptor, AdapterVersion};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement};
use std::collections::BTreeMap;

pub const ADAPTER_ID: &str = "windows.unity.mono.standard-ui";
pub const MAX_TEXT_UNITS: usize = 16 * 1024;
pub const MAX_TRACKED_OBJECTS: usize = 64 * 1024;

#[must_use]
pub fn descriptor() -> AdapterDescriptor {
    AdapterDescriptor::new(
        AdapterId::new(ADAPTER_ID),
        AdapterVersion::new(0, 1, 0),
        ApplyModel::RetainedObject,
        Placement::TargetProcess,
        [Feature::TextObserve, Feature::TextReplace],
    )
    .with_platforms(["windows"])
    .with_architectures(["x86_64"])
}

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

    fn decode(self) -> Option<DecodedManagedText> {
        if !self.object_id.is_valid() || self.units.len() > MAX_TEXT_UNITS {
            return None;
        }
        let source = String::from_utf16(&self.units).ok()?.into_boxed_str();
        Some(DecodedManagedText {
            object_id: self.object_id,
            kind: self.kind,
            source,
        })
    }
}

pub(crate) struct DecodedManagedText {
    pub(crate) object_id: ManagedObjectId,
    pub(crate) kind: StandardUiKind,
    pub(crate) source: Box<str>,
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
pub struct UnityMonoObserver {
    active: bool,
    tracked: BTreeMap<ManagedObjectId, TrackedText>,
}

impl UnityMonoObserver {
    #[must_use]
    pub fn apply(&mut self, event: ObserverEvent) -> Vec<ObservedText> {
        match event {
            ObserverEvent::AttachSnapshot(snapshot) => self.attach(snapshot),
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
        self.tracked.clear();

        let mut decoded = BTreeMap::new();
        for text in snapshot {
            let Some(text) = text.decode() else {
                continue;
            };
            if decoded.len() == MAX_TRACKED_OBJECTS && !decoded.contains_key(&text.object_id) {
                continue;
            }
            decoded.insert(
                text.object_id,
                TrackedText {
                    kind: text.kind,
                    source: text.source,
                },
            );
        }

        let observations = decoded
            .iter()
            .filter(|(_, text)| !text.source.is_empty())
            .map(|(object_id, text)| ObservedText {
                object_id: *object_id,
                kind: text.kind,
                source: text.source.clone(),
            })
            .collect();
        self.tracked = decoded;
        observations
    }

    fn setter(&mut self, text: ManagedText) -> Option<ObservedText> {
        if !self.active {
            return None;
        }
        let text = text.decode()?;
        if self.tracked.len() == MAX_TRACKED_OBJECTS && !self.tracked.contains_key(&text.object_id)
        {
            return None;
        }
        if self
            .tracked
            .get(&text.object_id)
            .is_some_and(|previous| previous.kind == text.kind && previous.source == text.source)
        {
            return None;
        }

        let observation = (!text.source.is_empty()).then(|| ObservedText {
            object_id: text.object_id,
            kind: text.kind,
            source: text.source.clone(),
        });
        self.tracked.insert(
            text.object_id,
            TrackedText {
                kind: text.kind,
                source: text.source,
            },
        );
        observation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(id: u64, kind: StandardUiKind, source: &str) -> ManagedText {
        ManagedText::text(ManagedObjectId::new(id), kind, source)
    }

    #[test]
    fn descriptor_describes_retained_standard_ui_replacement() {
        let descriptor = descriptor();
        assert_eq!(descriptor.adapter_id().as_str(), ADAPTER_ID);
        assert_eq!(descriptor.apply_model(), ApplyModel::RetainedObject);
        assert_eq!(descriptor.placement(), Placement::TargetProcess);
        assert_eq!(
            descriptor.features().collect::<Vec<_>>(),
            [Feature::TextObserve, Feature::TextReplace]
        );
        assert_eq!(descriptor.platforms().collect::<Vec<_>>(), ["windows"]);
        assert_eq!(descriptor.architectures().collect::<Vec<_>>(), ["x86_64"]);
    }

    #[test]
    fn attach_emits_one_final_valid_value_per_standard_ui_object() {
        let mut observer = UnityMonoObserver::default();
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
    fn setters_emit_only_changes_and_empty_text_resets_deduplication() {
        let mut observer = UnityMonoObserver::default();
        let id = ManagedObjectId::new(7);
        assert_eq!(
            observer
                .apply(ObserverEvent::AttachSnapshot(vec![text(
                    7,
                    StandardUiKind::TextMeshPro,
                    "Score: 0",
                )]))
                .len(),
            1
        );
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
            .apply(ObserverEvent::Setter(ManagedText::text(
                id,
                StandardUiKind::TextMeshPro,
                "",
            )))
            .is_empty());
        assert_eq!(
            observer.apply(ObserverEvent::Setter(ManagedText::text(
                id,
                StandardUiKind::TextMeshPro,
                "Score: 248",
            )))[0]
                .source(),
            "Score: 248"
        );
    }

    #[test]
    fn collection_deactivation_and_reattach_bound_object_lifetime() {
        let mut observer = UnityMonoObserver::default();
        let _ = observer.apply(ObserverEvent::AttachSnapshot(vec![text(
            9,
            StandardUiKind::UGui,
            "Download Resources",
        )]));
        let _ = observer.apply(ObserverEvent::Collected(ManagedObjectId::new(9)));
        assert_eq!(
            observer
                .apply(ObserverEvent::Setter(text(
                    9,
                    StandardUiKind::UGui,
                    "Download Resources",
                )))
                .len(),
            1
        );

        let _ = observer.apply(ObserverEvent::Deactivate);
        assert!(observer
            .apply(ObserverEvent::Setter(text(
                9,
                StandardUiKind::UGui,
                "Download Res",
            )))
            .is_empty());
        assert_eq!(
            observer
                .apply(ObserverEvent::AttachSnapshot(vec![text(
                    9,
                    StandardUiKind::UGui,
                    "Download Res",
                )]))
                .len(),
            1
        );
    }

    #[test]
    fn invalid_or_unbounded_setter_does_not_replace_the_last_valid_value() {
        let mut observer = UnityMonoObserver::default();
        let _ = observer.apply(ObserverEvent::AttachSnapshot(vec![text(
            12,
            StandardUiKind::TextMeshPro,
            "Wall",
        )]));
        assert!(observer
            .apply(ObserverEvent::Setter(ManagedText::utf16(
                ManagedObjectId::new(12),
                StandardUiKind::TextMeshPro,
                [0xd800],
            )))
            .is_empty());
        assert!(observer
            .apply(ObserverEvent::Setter(text(
                12,
                StandardUiKind::TextMeshPro,
                "Wall",
            )))
            .is_empty());
        assert!(observer
            .apply(ObserverEvent::Setter(ManagedText::utf16(
                ManagedObjectId::new(13),
                StandardUiKind::UGui,
                vec![b'a' as u16; MAX_TEXT_UNITS + 1],
            )))
            .is_empty());
    }

    #[test]
    fn object_limit_rejects_new_identity_but_keeps_existing_updates() {
        let mut observer = UnityMonoObserver::default();
        let snapshot = (1..=MAX_TRACKED_OBJECTS)
            .map(|id| text(id as u64, StandardUiKind::UGui, "Open"))
            .collect();
        assert_eq!(
            observer
                .apply(ObserverEvent::AttachSnapshot(snapshot))
                .len(),
            MAX_TRACKED_OBJECTS
        );
        assert!(observer
            .apply(ObserverEvent::Setter(text(
                MAX_TRACKED_OBJECTS as u64 + 1,
                StandardUiKind::UGui,
                "New",
            )))
            .is_empty());
        assert_eq!(
            observer.apply(ObserverEvent::Setter(text(
                1,
                StandardUiKind::UGui,
                "Changed",
            )))[0]
                .source(),
            "Changed"
        );
    }
}
