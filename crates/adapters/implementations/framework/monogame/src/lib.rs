//! Generic MonoGame `SpriteBatch.DrawString` / `SpriteFont.MeasureString` Adapter policy.
//!
//! Runtime method attachment is owned by the platform companion. This crate owns
//! framework-independent decisions: bounded UTF-16 conversion, glyph coverage,
//! fail-open behavior, and dictionary generation hand-off.

use glyphshift_adapter_sdk::{authorize, ActivationGrant, AdapterDescriptor, AdapterError, AdapterVersion};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement, RenderDecision, TextDecision};
use std::panic::{catch_unwind, AssertUnwindSafe};

pub const ADAPTER_ID: &str = "windows.monogame.sprite-batch-draw-string";
pub const MAX_TEXT_UNITS: usize = 64 * 1024;

#[must_use]
pub fn descriptor() -> AdapterDescriptor {
    AdapterDescriptor::new(
        AdapterId::new(ADAPTER_ID),
        AdapterVersion::new(1, 0, 0),
        ApplyModel::InlineRender,
        Placement::TargetProcess,
        [Feature::TextObserve, Feature::TextReplace],
    )
    .with_platforms(["windows"])
    .with_architectures(["x86", "x86_64"])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonoGameAdapter {
    text_replace: bool,
}

impl MonoGameAdapter {
    pub fn activate(
        requested: impl IntoIterator<Item = Feature>,
        grant: &ActivationGrant,
    ) -> Result<Self, AdapterError> {
        let active = authorize(requested, grant)?;
        Ok(Self { text_replace: active.contains(&Feature::TextReplace) })
    }

    /// Returns a replacement only when the translated UTF-16 string is safe for
    /// the resolved target or fallback font. The caller supplies that font's glyph predicate;
    /// the original call remains untouched on every failure.
    pub fn prepare(
        self,
        source: &[u16],
        has_glyph: impl Fn(char) -> bool,
        decide: impl FnOnce(&str) -> Result<RenderDecision, AdapterError>,
    ) -> Option<Vec<u16>> {
        if source.is_empty() || source.len() > MAX_TEXT_UNITS || source.contains(&0) {
            return None;
        }
        let text = String::from_utf16(source).ok()?;
        let decision = catch_unwind(AssertUnwindSafe(|| decide(&text))).ok().and_then(Result::ok)?;
        let TextDecision::Replace(replacement) = decision.text else { return None };
        if !self.text_replace || replacement.is_empty() || replacement.chars().any(|ch| !has_glyph(ch)) {
            return None;
        }
        let units: Vec<u16> = replacement.encode_utf16().collect();
        (units.len() <= MAX_TEXT_UNITS && !units.contains(&0)).then_some(units)
    }
}

impl Default for MonoGameAdapter {
    fn default() -> Self { Self { text_replace: true } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glyphshift_domain::{FontDecision, Generation};
    use std::sync::Arc;

    fn replacement(value: &str) -> RenderDecision {
        RenderDecision { text: TextDecision::Replace(Arc::from(value)), font: FontDecision::Keep, generation: Generation::new(2) }
    }

    #[test]
    fn descriptor_is_a_generic_monogame_target() {
        let descriptor = descriptor();
        assert_eq!(descriptor.adapter_id().as_str(), ADAPTER_ID);
        assert_eq!(descriptor.apply_model(), ApplyModel::InlineRender);
        assert_eq!(descriptor.placement(), Placement::TargetProcess);
        assert!(descriptor.features().any(|feature| feature == Feature::TextObserve));
        assert!(descriptor.features().any(|feature| feature == Feature::TextReplace));
    }

    #[test]
    fn replacement_requires_complete_target_font_coverage() {
        let adapter = MonoGameAdapter::default();
        let source: Vec<u16> = "AAA".encode_utf16().collect();
        assert_eq!(adapter.prepare(&source, |ch| ch == '中', |_| Ok(replacement("中文"))), None);
        assert_eq!(adapter.prepare(&source, |ch| ch == '中' || ch == '文', |_| Ok(replacement("中文"))), Some("中文".encode_utf16().collect()));
    }

    #[test]
    fn malformed_input_and_callback_fail_open() {
        let adapter = MonoGameAdapter::default();
        assert!(adapter.prepare(&[0xd800], |_| true, |_| panic!("callback")).is_none());
        assert!(adapter.prepare(&[0], |_| true, |_| Ok(replacement("译文"))).is_none());
        assert!(adapter.prepare(&vec![b'a' as u16; MAX_TEXT_UNITS + 1], |_| true, |_| Ok(replacement("译文"))).is_none());
    }

    #[test]
    fn observe_only_does_not_replace() {
        let grant = ActivationGrant::new([Feature::TextObserve]);
        let adapter = MonoGameAdapter::activate([Feature::TextObserve], &grant).unwrap();
        let source: Vec<u16> = "AAA".encode_utf16().collect();
        assert!(adapter.prepare(&source, |_| true, |_| Ok(replacement("译文"))).is_none());
    }
}
