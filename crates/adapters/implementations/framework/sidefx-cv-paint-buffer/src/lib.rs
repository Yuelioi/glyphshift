//! SideFX CV `CV_PaintBuffer::textWrappedInBox` draw-time text Adapter.

use glyphshift_adapter_sdk::{
    authorize, ActivationGrant, AdapterDescriptor, AdapterError, AdapterVersion,
};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement, RenderDecision, TextDecision};
use std::panic::{catch_unwind, AssertUnwindSafe};

pub const ADAPTER_ID: &str = "windows.sidefx.cv-paint-buffer-text";
pub const MAX_TEXT_BYTES: usize = 64 * 1024;

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
    .with_architectures(["x86_64"])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidefxCvPaintBufferInlineAdapter {
    text_replace: bool,
}

impl SidefxCvPaintBufferInlineAdapter {
    #[must_use]
    pub const fn new() -> Self {
        Self { text_replace: true }
    }

    pub fn activate(
        requested: impl IntoIterator<Item = Feature>,
        grant: &ActivationGrant,
    ) -> Result<Self, AdapterError> {
        let active = authorize(requested, grant)?;
        Ok(Self {
            text_replace: active.contains(&Feature::TextReplace),
        })
    }

    #[must_use]
    pub fn prepare(
        self,
        source: &[u8],
        decide: impl FnOnce(&str) -> Result<RenderDecision, AdapterError>,
    ) -> Option<Vec<u8>> {
        if source.is_empty() || source.len() > MAX_TEXT_BYTES {
            return None;
        }
        let source = std::str::from_utf8(source).ok()?;
        let decision = catch_unwind(AssertUnwindSafe(|| decide(source)))
            .ok()
            .and_then(Result::ok)?;
        let TextDecision::Replace(text) = decision.text else {
            return None;
        };
        if !self.text_replace || text.is_empty() || text.as_bytes().contains(&0) {
            return None;
        }
        let replacement = text.as_bytes().to_vec();
        (replacement.len() <= MAX_TEXT_BYTES).then_some(replacement)
    }
}

impl Default for SidefxCvPaintBufferInlineAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glyphshift_domain::{FontDecision, Generation};
    use std::sync::Arc;

    fn replace(text: &str) -> RenderDecision {
        RenderDecision {
            text: TextDecision::Replace(Arc::from(text)),
            font: FontDecision::Keep,
            generation: Generation::new(1),
        }
    }

    #[test]
    fn descriptor_is_x64_inline_render_text() {
        let descriptor = descriptor();
        assert_eq!(descriptor.adapter_id().as_str(), ADAPTER_ID);
        assert_eq!(descriptor.apply_model(), ApplyModel::InlineRender);
        assert_eq!(descriptor.placement(), Placement::TargetProcess);
        assert_eq!(descriptor.architectures().collect::<Vec<_>>(), ["x86_64"]);
    }

    #[test]
    fn utf8_multiline_source_prepares_replacement() {
        let source = b"Empty Network\nPress Tab to Add Nodes";
        let replacement = SidefxCvPaintBufferInlineAdapter::new()
            .prepare(source, |_| Ok(replace("空网络\n按 Tab 添加节点")));
        assert_eq!(
            replacement.as_deref(),
            Some("空网络\n按 Tab 添加节点".as_bytes())
        );
    }

    #[test]
    fn malformed_unbounded_or_observe_only_text_fails_open() {
        assert!(SidefxCvPaintBufferInlineAdapter::new()
            .prepare(&[0xff], |_| Ok(replace("译文")))
            .is_none());
        assert!(SidefxCvPaintBufferInlineAdapter::new()
            .prepare(&vec![b'a'; MAX_TEXT_BYTES + 1], |_| Ok(replace("译文")))
            .is_none());
        assert!(SidefxCvPaintBufferInlineAdapter::new()
            .prepare(b"Open", |_| Ok(replace("含\0空字节")))
            .is_none());

        let grant = ActivationGrant::new([Feature::TextObserve]);
        let adapter = SidefxCvPaintBufferInlineAdapter::activate([Feature::TextObserve], &grant)
            .expect("observe-only activation");
        assert!(adapter.prepare(b"Open", |_| Ok(replace("打开"))).is_none());
    }
}
