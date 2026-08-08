use crate::runtime_gate::{Il2CppExports, Il2CppRuntimeGate};
use crate::snapshot::capture_snapshot;
use glyphshift_adapter_unity_standard_ui::{ManagedText, StandardUiKind, StandardUiProfile};
use std::ffi::{c_char, c_void, CString};

const TMP_NAMESPACE: &str = "TMPro";
const TMP_CLASS: &str = "TMP_Text";
const TMP_TEXT_FIELD: &str = "m_text";
const UGUI_NAMESPACE: &str = "UnityEngine.UI";
const UGUI_CLASS: &str = "Text";
const UGUI_TEXT_FIELD: &str = "m_Text";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetadataError {
    RuntimeMissing,
    StandardUiMissing,
    ThreadAttachFailed,
    LivenessFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct TextTarget {
    pub(super) kind: StandardUiKind,
    pub(super) class: usize,
    pub(super) text_field: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StandardUiMetadata {
    targets: Vec<TextTarget>,
}

impl StandardUiMetadata {
    fn profile(&self) -> StandardUiProfile {
        StandardUiProfile::from_kinds(self.targets.iter().map(|target| target.kind))
    }
}

trait MetadataSource {
    fn images(&self) -> Vec<usize>;
    fn class_from_name(&self, image: usize, namespace: &str, name: &str) -> Option<usize>;
    fn field(&self, class: usize, name: &str) -> Option<usize>;
}

fn locate_standard_ui(source: &impl MetadataSource) -> Result<StandardUiMetadata, MetadataError> {
    let images = source.images();
    let mut targets = Vec::with_capacity(2);
    if let Some(target) = locate_text_target(
        source,
        &images,
        TMP_NAMESPACE,
        TMP_CLASS,
        TMP_TEXT_FIELD,
        StandardUiKind::TextMeshPro,
    ) {
        targets.push(target);
    }
    if let Some(target) = locate_text_target(
        source,
        &images,
        UGUI_NAMESPACE,
        UGUI_CLASS,
        UGUI_TEXT_FIELD,
        StandardUiKind::UGui,
    ) {
        targets.push(target);
    }
    if targets.is_empty() {
        return Err(MetadataError::StandardUiMissing);
    }
    Ok(StandardUiMetadata { targets })
}

fn locate_text_target(
    source: &impl MetadataSource,
    images: &[usize],
    namespace: &str,
    name: &str,
    field_name: &str,
    kind: StandardUiKind,
) -> Option<TextTarget> {
    let class = images
        .iter()
        .find_map(|image| source.class_from_name(*image, namespace, name))?;
    Some(TextTarget {
        kind,
        class,
        text_field: source.field(class, field_name)?,
    })
}

pub(crate) struct Il2CppStandardUiRuntime {
    exports: Il2CppExports,
    metadata: StandardUiMetadata,
}

impl Il2CppStandardUiRuntime {
    pub(crate) fn resolve(gate: Il2CppRuntimeGate) -> Result<Self, MetadataError> {
        let exports = gate.exports();
        let domain = unsafe { (exports.domain_get)() };
        if domain.is_null() {
            return Err(MetadataError::RuntimeMissing);
        }
        let current_thread = unsafe { (exports.thread_current)() };
        let attached_thread = if current_thread.is_null() {
            unsafe { (exports.thread_attach)(domain) }
        } else {
            current_thread
        };
        if attached_thread.is_null() {
            return Err(MetadataError::RuntimeMissing);
        }

        let source = RuntimeMetadataSource { exports, domain };
        let metadata = locate_standard_ui(&source);
        if current_thread.is_null() {
            unsafe { (exports.thread_detach)(attached_thread) };
        }
        Ok(Self {
            exports,
            metadata: metadata?,
        })
    }

    pub(crate) fn profile(&self) -> StandardUiProfile {
        self.metadata.profile()
    }

    pub(crate) unsafe fn snapshot_on_attached_thread(
        &self,
    ) -> Result<Vec<ManagedText>, MetadataError> {
        capture_snapshot(self.exports, &self.metadata.targets)
    }
}

struct RuntimeMetadataSource {
    exports: Il2CppExports,
    domain: *mut c_void,
}

impl MetadataSource for RuntimeMetadataSource {
    fn images(&self) -> Vec<usize> {
        let mut length = 0;
        let assemblies = unsafe { (self.exports.domain_get_assemblies)(self.domain, &mut length) };
        if assemblies.is_null() {
            return Vec::new();
        }
        (0..length)
            .filter_map(|index| {
                let assembly = unsafe { *assemblies.add(index) };
                if assembly.is_null() {
                    return None;
                }
                let image = unsafe { (self.exports.assembly_get_image)(assembly) };
                (!image.is_null()).then_some(image as usize)
            })
            .collect()
    }

    fn class_from_name(&self, image: usize, namespace: &str, name: &str) -> Option<usize> {
        let namespace = CString::new(namespace).ok()?;
        let name = CString::new(name).ok()?;
        let class = unsafe {
            (self.exports.class_from_name)(
                image as *const c_void,
                namespace.as_ptr().cast::<c_char>(),
                name.as_ptr().cast::<c_char>(),
            )
        };
        (!class.is_null()).then_some(class as usize)
    }

    fn field(&self, class: usize, name: &str) -> Option<usize> {
        let name = CString::new(name).ok()?;
        let field = unsafe {
            (self.exports.class_get_field_from_name)(
                class as *mut c_void,
                name.as_ptr().cast::<c_char>(),
            )
        };
        (!field.is_null()).then_some(field as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct FakeMetadata {
        classes: BTreeMap<(usize, &'static str, &'static str), usize>,
        fields: BTreeMap<(usize, &'static str), usize>,
    }

    impl FakeMetadata {
        fn with_text(
            mut self,
            image: usize,
            namespace: &'static str,
            name: &'static str,
            field_name: &'static str,
            class: usize,
        ) -> Self {
            self.classes.insert((image, namespace, name), class);
            self.fields.insert((class, field_name), class + 100);
            self
        }
    }

    impl MetadataSource for FakeMetadata {
        fn images(&self) -> Vec<usize> {
            vec![1, 2, 3]
        }

        fn class_from_name(&self, image: usize, namespace: &str, name: &str) -> Option<usize> {
            self.classes.iter().find_map(
                |((candidate_image, candidate_namespace, candidate_name), class)| {
                    (*candidate_image == image
                        && *candidate_namespace == namespace
                        && *candidate_name == name)
                        .then_some(*class)
                },
            )
        }

        fn field(&self, class: usize, name: &str) -> Option<usize> {
            self.fields
                .iter()
                .find_map(|((candidate_class, candidate_name), field)| {
                    (*candidate_class == class && *candidate_name == name).then_some(*field)
                })
        }
    }

    #[test]
    fn finds_tmp_and_ugui_fields_across_different_images() {
        let source = FakeMetadata::default()
            .with_text(2, TMP_NAMESPACE, TMP_CLASS, TMP_TEXT_FIELD, 30)
            .with_text(3, UGUI_NAMESPACE, UGUI_CLASS, UGUI_TEXT_FIELD, 31);
        let metadata = locate_standard_ui(&source).expect("standard UI metadata");

        assert!(metadata.profile().supports(StandardUiKind::TextMeshPro));
        assert!(metadata.profile().supports(StandardUiKind::UGui));
        assert_eq!(metadata.targets.len(), 2);
    }

    #[test]
    fn accepts_one_standard_ui_family_but_rejects_none_or_stripped_field() {
        let tmp =
            FakeMetadata::default().with_text(2, TMP_NAMESPACE, TMP_CLASS, TMP_TEXT_FIELD, 30);
        let metadata = locate_standard_ui(&tmp).expect("TMP metadata");
        assert!(metadata.profile().supports(StandardUiKind::TextMeshPro));
        assert!(!metadata.profile().supports(StandardUiKind::UGui));

        assert_eq!(
            locate_standard_ui(&FakeMetadata::default()),
            Err(MetadataError::StandardUiMissing)
        );

        let mut stripped =
            FakeMetadata::default().with_text(2, TMP_NAMESPACE, TMP_CLASS, TMP_TEXT_FIELD, 30);
        stripped.fields.clear();
        assert_eq!(
            locate_standard_ui(&stripped),
            Err(MetadataError::StandardUiMissing)
        );
    }
}
