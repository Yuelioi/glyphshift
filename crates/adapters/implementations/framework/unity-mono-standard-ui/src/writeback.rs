use crate::{
    ManagedObjectId, ManagedText, StandardUiKind, MAX_TEXT_UNITS, MAX_TRACKED_OBJECTS,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextDecision {
    generation: u64,
    replacement: Option<Vec<u16>>,
}

impl TextDecision {
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub const fn keep(generation: u64) -> Self {
        Self {
            generation,
            replacement: None,
        }
    }

    #[must_use]
    pub fn replace_utf16(generation: u64, replacement: impl Into<Vec<u16>>) -> Self {
        Self {
            generation,
            replacement: Some(replacement.into()),
        }
    }

    #[must_use]
    pub fn replace_text(generation: u64, replacement: &str) -> Self {
        Self::replace_utf16(generation, replacement.encode_utf16().collect::<Vec<_>>())
    }

    fn validated_replacement(&self, source: &[u16]) -> Option<Vec<u16>> {
        let replacement = self.replacement.as_ref()?;
        if replacement.len() > MAX_TEXT_UNITS
            || String::from_utf16(replacement).is_err()
            || replacement == source
        {
            return None;
        }
        Some(replacement.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextWrite {
    object_id: ManagedObjectId,
    kind: StandardUiKind,
    units: Vec<u16>,
}

impl TextWrite {
    fn new(object_id: ManagedObjectId, kind: StandardUiKind, units: Vec<u16>) -> Self {
        Self {
            object_id,
            kind,
            units,
        }
    }

    #[must_use]
    pub const fn object_id(&self) -> ManagedObjectId {
        self.object_id
    }

    #[must_use]
    pub const fn kind(&self) -> StandardUiKind {
        self.kind
    }

    #[must_use]
    pub fn units(&self) -> &[u16] {
        &self.units
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetterOutcome {
    Untracked,
    ForwardOriginal { generation: u64 },
    Replace { generation: u64, units: Vec<u16> },
}

impl SetterOutcome {
    #[must_use]
    pub const fn generation(&self) -> Option<u64> {
        match self {
            Self::Untracked => None,
            Self::ForwardOriginal { generation } | Self::Replace { generation, .. } => {
                Some(*generation)
            }
        }
    }

    #[must_use]
    pub fn replacement_units(&self) -> Option<&[u16]> {
        match self {
            Self::Replace { units, .. } => Some(units),
            Self::Untracked | Self::ForwardOriginal { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TrackedWriteback {
    kind: StandardUiKind,
    source: Vec<u16>,
    applied: Option<Vec<u16>>,
    generation: u64,
}

#[derive(Debug, Default)]
pub struct UnityMonoWriteback {
    active: bool,
    tracked: BTreeMap<ManagedObjectId, TrackedWriteback>,
}

impl UnityMonoWriteback {
    #[must_use]
    pub fn known_generation(&self) -> Option<u64> {
        self.tracked
            .values()
            .map(|tracked| tracked.generation)
            .max()
    }

    #[must_use]
    pub fn probe_source(&self) -> Option<String> {
        self.tracked
            .values()
            .find(|tracked| !tracked.source.is_empty())
            .and_then(|tracked| String::from_utf16(&tracked.source).ok())
    }

    #[must_use]
    pub fn has_applied_replacements(&self) -> bool {
        self.tracked
            .values()
            .any(|tracked| tracked.applied.is_some())
    }

    pub fn activate(
        &mut self,
        snapshot: Vec<ManagedText>,
        mut decide: impl FnMut(&str) -> TextDecision,
    ) -> Vec<TextWrite> {
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
            decoded.insert(text.object_id, text);
        }

        let mut writes = Vec::new();
        for (object_id, text) in decoded {
            let source = text.source.encode_utf16().collect::<Vec<_>>();
            let decision = if source.is_empty() {
                TextDecision::keep(0)
            } else {
                decide(&text.source)
            };
            let applied = decision.validated_replacement(&source);
            if let Some(replacement) = &applied {
                writes.push(TextWrite::new(object_id, text.kind, replacement.clone()));
            }
            self.tracked.insert(
                object_id,
                TrackedWriteback {
                    kind: text.kind,
                    source,
                    applied,
                    generation: decision.generation,
                },
            );
        }
        writes
    }

    pub fn intercept_setter(
        &mut self,
        text: ManagedText,
        mut decide: impl FnMut(&str) -> TextDecision,
    ) -> SetterOutcome {
        if !self.active {
            return SetterOutcome::Untracked;
        }
        let Some(text) = text.decode() else {
            return SetterOutcome::Untracked;
        };
        if self.tracked.len() == MAX_TRACKED_OBJECTS
            && !self.tracked.contains_key(&text.object_id)
        {
            return SetterOutcome::Untracked;
        }

        let source = text.source.encode_utf16().collect::<Vec<_>>();
        let decision = if source.is_empty() {
            TextDecision::keep(0)
        } else {
            decide(&text.source)
        };
        let applied = decision.validated_replacement(&source);
        let outcome = applied.as_ref().map_or(
            SetterOutcome::ForwardOriginal {
                generation: decision.generation,
            },
            |replacement| SetterOutcome::Replace {
                generation: decision.generation,
                units: replacement.clone(),
            },
        );
        self.tracked.insert(
            text.object_id,
            TrackedWriteback {
                kind: text.kind,
                source,
                applied,
                generation: decision.generation,
            },
        );
        outcome
    }

    pub fn refresh(
        &mut self,
        mut decide: impl FnMut(&str) -> TextDecision,
    ) -> Vec<TextWrite> {
        if !self.active {
            return Vec::new();
        }

        let mut writes = Vec::new();
        for (object_id, tracked) in &mut self.tracked {
            if tracked.source.is_empty() {
                continue;
            }
            let Ok(source) = String::from_utf16(&tracked.source) else {
                continue;
            };
            let decision = decide(&source);
            let next = decision.validated_replacement(&tracked.source);
            if next != tracked.applied {
                writes.push(TextWrite::new(
                    *object_id,
                    tracked.kind,
                    next.clone().unwrap_or_else(|| tracked.source.clone()),
                ));
            }
            tracked.applied = next;
            tracked.generation = decision.generation;
        }
        writes
    }

    pub fn collected(&mut self, object_id: ManagedObjectId) {
        if self.active {
            self.tracked.remove(&object_id);
        }
    }

    pub fn deactivate(&mut self) -> Vec<TextWrite> {
        self.active = false;
        let writes = self
            .tracked
            .iter()
            .filter(|(_, tracked)| tracked.applied.is_some())
            .map(|(object_id, tracked)| {
                TextWrite::new(*object_id, tracked.kind, tracked.source.clone())
            })
            .collect();
        self.tracked.clear();
        writes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(id: u64, source: &str) -> ManagedText {
        ManagedText::text(
            ManagedObjectId::new(id),
            StandardUiKind::TextMeshPro,
            source,
        )
    }

    fn rendered(write: &TextWrite) -> String {
        String::from_utf16(write.units()).expect("write must remain valid UTF-16")
    }

    #[test]
    fn activation_replaces_final_snapshot_value_once() {
        let mut engine = UnityMonoWriteback::default();
        let writes = engine.activate(vec![text(1, "Old"), text(1, "Hello")], |_| {
            TextDecision::replace_text(4, "你好")
        });

        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].object_id(), ManagedObjectId::new(1));
        assert_eq!(rendered(&writes[0]), "你好");
    }

    #[test]
    fn setter_uses_latest_source_and_replacement() {
        let mut engine = UnityMonoWriteback::default();
        let _ = engine.activate(vec![text(1, "Score: 0")], |_| TextDecision::keep(1));

        let outcome = engine.intercept_setter(text(1, "Score: 248"), |source| {
            assert_eq!(source, "Score: 248");
            TextDecision::replace_text(2, "分数：248")
        });

        assert_eq!(
            outcome,
            SetterOutcome::Replace {
                generation: 2,
                units: "分数：248".encode_utf16().collect(),
            }
        );
    }

    #[test]
    fn unchanged_refresh_deduplicates_writes() {
        let mut engine = UnityMonoWriteback::default();
        let _ = engine.activate(vec![text(1, "Hello")], |_| {
            TextDecision::replace_text(1, "你好")
        });

        assert!(engine
            .refresh(|_| TextDecision::replace_text(1, "你好"))
            .is_empty());
    }

    #[test]
    fn generation_refresh_updates_and_removes_replacement() {
        let mut engine = UnityMonoWriteback::default();
        let _ = engine.activate(vec![text(1, "Hello")], |_| {
            TextDecision::replace_text(1, "你好")
        });

        let updated = engine.refresh(|_| TextDecision::replace_text(2, "您好"));
        assert_eq!(rendered(&updated[0]), "您好");
        assert_eq!(engine.known_generation(), Some(2));
        assert_eq!(engine.probe_source().as_deref(), Some("Hello"));
        assert!(engine.has_applied_replacements());
        let removed = engine.refresh(|_| TextDecision::keep(3));
        assert_eq!(rendered(&removed[0]), "Hello");
        assert!(!engine.has_applied_replacements());
    }

    #[test]
    fn invalid_replacement_fails_open() {
        let mut engine = UnityMonoWriteback::default();
        let _ = engine.activate(vec![text(1, "Hello")], |_| TextDecision::keep(1));

        let outcome = engine.intercept_setter(text(1, "Changed"), |_| {
            TextDecision::replace_utf16(2, [0xd800])
        });

        assert_eq!(outcome, SetterOutcome::ForwardOriginal { generation: 2 });
    }

    #[test]
    fn deactivation_restores_latest_app_source_and_forgets_collected_objects() {
        let mut engine = UnityMonoWriteback::default();
        let _ = engine.activate(vec![text(1, "Hello"), text(2, "Open")], |_| {
            TextDecision::replace_text(1, "已翻译")
        });
        let _ = engine.intercept_setter(text(1, "Changed"), |_| {
            TextDecision::replace_text(2, "已更改")
        });
        engine.collected(ManagedObjectId::new(2));

        let restores = engine.deactivate();

        assert_eq!(restores.len(), 1);
        assert_eq!(rendered(&restores[0]), "Changed");
    }
}
