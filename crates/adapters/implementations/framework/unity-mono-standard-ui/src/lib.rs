//! Unity Mono Standard UI descriptor over the shared retained-text state.

pub use glyphshift_adapter_unity_standard_ui::{
    ManagedObjectId, ManagedText, ObservedText, ObserverEvent, SetterOutcome, StandardUiKind,
    TextDecision, TextWrite, UnityStandardUiObserver, UnityStandardUiWriteback, MAX_TEXT_UNITS,
    MAX_TRACKED_OBJECTS,
};

pub type UnityMonoWriteback = UnityStandardUiWriteback;

use glyphshift_adapter_sdk::{AdapterDescriptor, AdapterVersion};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement};

pub const ADAPTER_ID: &str = "windows.unity.mono.standard-ui";

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
    .with_architectures(["x86", "x86_64"])
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            descriptor.architectures().collect::<Vec<_>>(),
            ["x86", "x86_64"]
        );
    }
}
