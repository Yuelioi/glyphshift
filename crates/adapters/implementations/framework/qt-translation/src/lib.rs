//! Product descriptor for Qt's source-text translation service.
//!
//! The production adapter supports observation and inline source substitution at the public
//! translation-service boundary. Native profile gating keeps unsupported Qt versions fail-closed
//! before hooks are installed.

use glyphshift_adapter_sdk::{AdapterDescriptor, AdapterVersion};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement};

pub const ADAPTER_ID: &str = "windows.qt.translation-service";

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_is_inline_translation_service_windows_x64() {
        let descriptor = descriptor();
        assert_eq!(descriptor.adapter_id().as_str(), ADAPTER_ID);
        assert_eq!(descriptor.apply_model(), ApplyModel::InlineRender);
        assert_eq!(descriptor.placement(), Placement::TargetProcess);
        assert_eq!(
            descriptor.features().collect::<Vec<_>>(),
            vec![Feature::TextObserve, Feature::TextReplace]
        );
        assert!(descriptor.platforms().any(|platform| platform == "windows"));
        assert!(descriptor.architectures().any(|arch| arch == "x86_64"));
        assert!(!descriptor.architectures().any(|arch| arch == "x86"));
    }
}
