//! Descriptor for the unpublished Unity IL2CPP Standard UI observer prototype.

use glyphshift_adapter_sdk::{AdapterDescriptor, AdapterVersion};
use glyphshift_domain::{AdapterId, ApplyModel, Feature, Placement};

pub const ADAPTER_ID: &str = "windows.unity.il2cpp.standard-ui";

#[must_use]
pub fn descriptor() -> AdapterDescriptor {
    AdapterDescriptor::new(
        AdapterId::new(ADAPTER_ID),
        AdapterVersion::new(0, 1, 0),
        ApplyModel::ObserveOnly,
        Placement::TargetProcess,
        [Feature::TextObserve],
    )
    .with_platforms(["windows"])
    .with_architectures(["x86_64"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_is_unpublished_observe_only_windows_x64() {
        let descriptor = descriptor();

        assert_eq!(descriptor.adapter_id().as_str(), ADAPTER_ID);
        assert_eq!(descriptor.apply_model(), ApplyModel::ObserveOnly);
        assert_eq!(descriptor.placement(), Placement::TargetProcess);
        assert_eq!(
            descriptor.features().collect::<Vec<_>>(),
            [Feature::TextObserve]
        );
        assert_eq!(descriptor.platforms().collect::<Vec<_>>(), ["windows"]);
        assert_eq!(descriptor.architectures().collect::<Vec<_>>(), ["x86_64"]);
    }
}
