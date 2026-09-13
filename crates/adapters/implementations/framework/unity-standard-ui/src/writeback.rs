use crate::{ManagedObjectId, ManagedText, StandardUiKind, MAX_TEXT_UNITS, MAX_TRACKED_OBJECTS};
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

#[derive(Clone, Debug, Default)]
pub struct UnityStandardUiWriteback {
    active: bool,
    tracked: BTreeMap<ManagedObjectId, TrackedWriteback>,
}

impl UnityStandardUiWriteback {
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

    #[must_use]
    pub fn needs_generation_refresh(&self, generation: u64) -> bool {
        self.tracked
            .values()
            .any(|tracked| tracked.generation != generation)
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
            let Some((object_id, kind, source)) = text.into_decoded_parts() else {
                continue;
            };
            if decoded.len() == MAX_TRACKED_OBJECTS && !decoded.contains_key(&object_id) {
                continue;
            }
            decoded.insert(object_id, (kind, source));
        }

        let mut writes = Vec::new();
        for (object_id, (kind, source_text)) in decoded {
            let source = source_text.encode_utf16().collect::<Vec<_>>();
            let decision = if source.is_empty() {
                TextDecision::keep(0)
            } else {
                decide(&source_text)
            };
            let applied = decision.validated_replacement(&source);
            if let Some(replacement) = &applied {
                writes.push(TextWrite::new(object_id, kind, replacement.clone()));
            }
            self.tracked.insert(
                object_id,
                TrackedWriteback {
                    kind,
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
        let Some((object_id, kind, source_text)) = text.into_decoded_parts() else {
            return SetterOutcome::Untracked;
        };
        if self.tracked.len() == MAX_TRACKED_OBJECTS && !self.tracked.contains_key(&object_id) {
            return SetterOutcome::Untracked;
        }

        let source = source_text.encode_utf16().collect::<Vec<_>>();
        let decision = if source.is_empty() {
            TextDecision::keep(0)
        } else {
            decide(&source_text)
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
            object_id,
            TrackedWriteback {
                kind,
                source,
                applied,
                generation: decision.generation,
            },
        );
        outcome
    }

    /// Reconciles a retained-object snapshot with the latest business source.
    ///
    /// IL2CPP cannot rely on a stable public setter entry address, so its
    /// observer periodically sees the retained `text` field after our own
    /// replacement may already be visible. A snapshot value equal to the last
    /// applied replacement is therefore adapter output and must never replace
    /// the saved business source. A different value is treated as a fresh
    /// business update and is decided normally.
    pub fn reconcile_snapshot(
        &mut self,
        snapshot: Vec<ManagedText>,
        mut decide: impl FnMut(&str) -> TextDecision,
    ) -> Vec<TextWrite> {
        if !self.active {
            return Vec::new();
        }

        let mut decoded = BTreeMap::new();
        for text in snapshot {
            let Some((object_id, kind, source)) = text.into_decoded_parts() else {
                continue;
            };
            if decoded.len() == MAX_TRACKED_OBJECTS && !decoded.contains_key(&object_id) {
                continue;
            }
            decoded.insert(object_id, (kind, source));
        }

        let mut writes = Vec::new();
        for (object_id, (kind, visible_text)) in decoded {
            let visible = visible_text.encode_utf16().collect::<Vec<_>>();

            if let Some(tracked) = self.tracked.get(&object_id) {
                if tracked.kind == kind
                    && tracked
                        .applied
                        .as_deref()
                        .is_some_and(|applied| applied == visible.as_slice())
                {
                    // The retained field still contains our own last write.
                    // Keep the saved business source intact.
                    continue;
                }
            }

            let decision = if visible.is_empty() {
                TextDecision::keep(
                    self.tracked
                        .get(&object_id)
                        .map_or(0, |tracked| tracked.generation),
                )
            } else {
                decide(&visible_text)
            };
            let applied = decision.validated_replacement(&visible);

            match self.tracked.get_mut(&object_id) {
                Some(tracked) if tracked.kind == kind && tracked.source == visible => {
                    // Business code reasserted the same source while a
                    // replacement was visible. Reapply the current decision.
                    if let Some(replacement) = &applied {
                        writes.push(TextWrite::new(object_id, kind, replacement.clone()));
                    }
                    tracked.applied = applied;
                    tracked.generation = decision.generation;
                }
                Some(tracked) => {
                    // A value different from both source and applied output is
                    // the latest business source for this retained object.
                    if let Some(replacement) = &applied {
                        writes.push(TextWrite::new(object_id, kind, replacement.clone()));
                    }
                    *tracked = TrackedWriteback {
                        kind,
                        source: visible,
                        applied,
                        generation: decision.generation,
                    };
                }
                None => {
                    if self.tracked.len() == MAX_TRACKED_OBJECTS {
                        continue;
                    }
                    if let Some(replacement) = &applied {
                        writes.push(TextWrite::new(object_id, kind, replacement.clone()));
                    }
                    self.tracked.insert(
                        object_id,
                        TrackedWriteback {
                            kind,
                            source: visible,
                            applied,
                            generation: decision.generation,
                        },
                    );
                }
            }
        }

        writes
    }

    pub fn refresh(&mut self, mut decide: impl FnMut(&str) -> TextDecision) -> Vec<TextWrite> {
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

    /// Conservatively restores every tracked object's latest business source.
    ///
    /// This is intended for writeback error recovery where a partially applied
    /// setter batch means the retained state can no longer prove which objects
    /// still display adapter output.
    pub fn deactivate_all(&mut self) -> Vec<TextWrite> {
        self.active = false;
        let writes = self
            .tracked
            .iter()
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
        let mut engine = UnityStandardUiWriteback::default();
        let writes = engine.activate(vec![text(1, "Old"), text(1, "Hello")], |_| {
            TextDecision::replace_text(4, "你好")
        });

        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].object_id(), ManagedObjectId::new(1));
        assert_eq!(rendered(&writes[0]), "你好");
    }

    #[test]
    fn setter_uses_latest_source_and_replacement() {
        let mut engine = UnityStandardUiWriteback::default();
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
        let mut engine = UnityStandardUiWriteback::default();
        let _ = engine.activate(vec![text(1, "Hello")], |_| {
            TextDecision::replace_text(1, "你好")
        });

        assert!(engine
            .refresh(|_| TextDecision::replace_text(1, "你好"))
            .is_empty());
    }

    #[test]
    fn generation_refresh_updates_and_removes_replacement() {
        let mut engine = UnityStandardUiWriteback::default();
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
        let mut engine = UnityStandardUiWriteback::default();
        let _ = engine.activate(vec![text(1, "Hello")], |_| TextDecision::keep(1));

        let outcome = engine.intercept_setter(text(1, "Changed"), |_| {
            TextDecision::replace_utf16(2, [0xd800])
        });

        assert_eq!(outcome, SetterOutcome::ForwardOriginal { generation: 2 });
    }

    #[test]
    fn deactivation_restores_latest_app_source_and_forgets_collected_objects() {
        let mut engine = UnityStandardUiWriteback::default();
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

    #[test]
    fn snapshot_ignores_its_own_replacement_and_restores_business_source() {
        let mut engine = UnityStandardUiWriteback::default();
        let initial = engine.activate(vec![text(1, "Hello")], |_| {
            TextDecision::replace_text(1, "你好")
        });
        assert_eq!(rendered(&initial[0]), "你好");

        let echoed = engine.reconcile_snapshot(vec![text(1, "你好")], |_| {
            panic!("adapter output must not be decided as a business source")
        });
        assert!(echoed.is_empty());

        let restores = engine.deactivate();
        assert_eq!(rendered(&restores[0]), "Hello");
    }

    #[test]
    fn snapshot_accepts_dynamic_business_source_and_restores_latest_value() {
        let mut engine = UnityStandardUiWriteback::default();
        let _ = engine.activate(vec![text(1, "Score: 0")], |_| {
            TextDecision::replace_text(1, "分数：0")
        });

        let writes = engine.reconcile_snapshot(vec![text(1, "Score: 248")], |source| {
            assert_eq!(source, "Score: 248");
            TextDecision::replace_text(2, "分数：248")
        });
        assert_eq!(rendered(&writes[0]), "分数：248");

        let restores = engine.deactivate();
        assert_eq!(rendered(&restores[0]), "Score: 248");
    }

    #[test]
    fn snapshot_reapplies_when_business_reasserts_same_source() {
        let mut engine = UnityStandardUiWriteback::default();
        let _ = engine.activate(vec![text(1, "Open")], |_| {
            TextDecision::replace_text(1, "打开")
        });

        let writes = engine.reconcile_snapshot(vec![text(1, "Open")], |_| {
            TextDecision::replace_text(2, "开启")
        });
        assert_eq!(rendered(&writes[0]), "开启");
        assert_eq!(engine.known_generation(), Some(2));
    }

    #[test]
    fn snapshot_adds_new_objects_but_only_explicit_collection_forgets_missing_ones() {
        let mut engine = UnityStandardUiWriteback::default();
        let _ = engine.activate(vec![text(1, "One")], |_| {
            TextDecision::replace_text(1, "一")
        });

        let writes = engine.reconcile_snapshot(vec![text(2, "Two")], |_| {
            TextDecision::replace_text(1, "二")
        });
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].object_id(), ManagedObjectId::new(2));

        let restores = engine.deactivate();
        assert_eq!(restores.len(), 2, "missing snapshot entries remain tracked");

        let mut engine = UnityStandardUiWriteback::default();
        let _ = engine.activate(vec![text(1, "One"), text(2, "Two")], |_| {
            TextDecision::replace_text(1, "译文")
        });
        engine.collected(ManagedObjectId::new(1));
        assert_eq!(engine.deactivate().len(), 1);
    }

    #[test]
    fn generation_refresh_detects_mixed_object_generations() {
        let mut engine = UnityStandardUiWriteback::default();
        let _ = engine.activate(vec![text(1, "One"), text(2, "Two")], |_| {
            TextDecision::replace_text(1, "译文")
        });

        let _ = engine.reconcile_snapshot(vec![text(1, "One changed")], |_| {
            TextDecision::replace_text(2, "新译文")
        });

        assert_eq!(engine.known_generation(), Some(2));
        assert!(engine.needs_generation_refresh(2));
    }

    #[test]
    fn conservative_deactivation_restores_even_after_replacement_removal_was_planned() {
        let mut engine = UnityStandardUiWriteback::default();
        let _ = engine.activate(vec![text(1, "Hello")], |_| {
            TextDecision::replace_text(1, "你好")
        });
        let _ = engine.refresh(|_| TextDecision::keep(2));

        assert!(!engine.has_applied_replacements());
        let restores = engine.deactivate_all();
        assert_eq!(restores.len(), 1);
        assert_eq!(rendered(&restores[0]), "Hello");
    }
}
