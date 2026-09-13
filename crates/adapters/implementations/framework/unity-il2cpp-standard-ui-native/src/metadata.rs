use crate::objects::ObjectRegistry;
use crate::runtime_gate::{Il2CppExports, Il2CppRuntimeGate};
use crate::snapshot::{capture_snapshot, CapturedSnapshot};
use crate::writeback::{apply_writes, WritebackError};
use glyphshift_adapter_unity_standard_ui::{StandardUiKind, StandardUiProfile, TextWrite};
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
    pub(super) text_setter: Option<usize>,
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
    fn method(&self, class: usize, name: &str, parameter_count: i32) -> Option<usize>;
}

fn locate_standard_ui(
    source: &impl MetadataSource,
    require_writeback: bool,
) -> Result<StandardUiMetadata, MetadataError> {
    let images = source.images();
    let mut targets = Vec::with_capacity(2);
    if let Some(target) = locate_text_target(
        source,
        &images,
        TMP_NAMESPACE,
        TMP_CLASS,
        TMP_TEXT_FIELD,
        StandardUiKind::TextMeshPro,
        require_writeback,
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
        require_writeback,
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
    require_writeback: bool,
) -> Option<TextTarget> {
    let class = images
        .iter()
        .find_map(|image| source.class_from_name(*image, namespace, name))?;
    let text_setter = source.method(class, "set_text", 1);
    if require_writeback && text_setter.is_none() {
        return None;
    }
    Some(TextTarget {
        kind,
        class,
        text_field: source.field(class, field_name)?,
        text_setter,
    })
}

pub(crate) struct Il2CppStandardUiRuntime {
    exports: Il2CppExports,
    metadata: StandardUiMetadata,
    objects: Option<ObjectRegistry>,
}

impl Il2CppStandardUiRuntime {
    pub(crate) fn resolve(gate: Il2CppRuntimeGate) -> Result<Self, MetadataError> {
        let exports = gate.exports();
        let require_writeback = exports.writeback.is_some();
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
        let metadata = locate_standard_ui(&source, require_writeback);
        if current_thread.is_null() {
            unsafe { (exports.thread_detach)(attached_thread) };
        }
        Ok(Self {
            exports,
            metadata: metadata?,
            objects: require_writeback.then(ObjectRegistry::default),
        })
    }

    pub(crate) fn profile(&self) -> StandardUiProfile {
        self.metadata.profile()
    }

    pub(crate) unsafe fn snapshot_on_attached_thread(
        &mut self,
    ) -> Result<CapturedSnapshot, MetadataError> {
        capture_snapshot(self.exports, &self.metadata.targets, self.objects.as_mut())
    }

    pub(crate) unsafe fn apply_writes_on_current_thread(
        &self,
        writes: &[TextWrite],
    ) -> Result<(), WritebackError> {
        let objects = self
            .objects
            .as_ref()
            .ok_or(WritebackError::WritebackUnavailable)?;
        apply_writes(self.exports, &self.metadata.targets, objects, writes)
    }

    pub(crate) unsafe fn release_object_handles(&mut self) {
        let Some(writeback) = self.exports.writeback else {
            return;
        };
        let Some(objects) = self.objects.as_mut() else {
            return;
        };
        let domain = (self.exports.domain_get)();
        if domain.is_null() {
            return;
        }
        let current = (self.exports.thread_current)();
        let attached = if current.is_null() {
            (self.exports.thread_attach)(domain)
        } else {
            current
        };
        if attached.is_null() {
            return;
        }
        objects.clear(writeback);
        if current.is_null() {
            (self.exports.thread_detach)(attached);
        }
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

    fn method(&self, class: usize, name: &str, parameter_count: i32) -> Option<usize> {
        let writeback = self.exports.writeback?;
        let name = CString::new(name).ok()?;
        let method = unsafe {
            (writeback.class_get_method_from_name)(
                class as *mut c_void,
                name.as_ptr().cast::<c_char>(),
                parameter_count,
            )
        };
        (!method.is_null()).then_some(method as usize)
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
        methods: BTreeMap<(usize, &'static str, i32), usize>,
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
            self.methods.insert((class, "set_text", 1), class + 200);
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

        fn method(&self, class: usize, name: &str, parameter_count: i32) -> Option<usize> {
            self.methods.iter().find_map(
                |((candidate_class, candidate_name, candidate_count), method)| {
                    (*candidate_class == class
                        && *candidate_name == name
                        && *candidate_count == parameter_count)
                        .then_some(*method)
                },
            )
        }
    }

    #[test]
    fn finds_tmp_and_ugui_fields_across_different_images() {
        let source = FakeMetadata::default()
            .with_text(2, TMP_NAMESPACE, TMP_CLASS, TMP_TEXT_FIELD, 30)
            .with_text(3, UGUI_NAMESPACE, UGUI_CLASS, UGUI_TEXT_FIELD, 31);
        let metadata = locate_standard_ui(&source, true).expect("standard UI metadata");

        assert!(metadata.profile().supports(StandardUiKind::TextMeshPro));
        assert!(metadata.profile().supports(StandardUiKind::UGui));
        assert_eq!(metadata.targets.len(), 2);
    }

    #[test]
    fn accepts_one_standard_ui_family_but_rejects_none_or_stripped_field() {
        let tmp =
            FakeMetadata::default().with_text(2, TMP_NAMESPACE, TMP_CLASS, TMP_TEXT_FIELD, 30);
        let metadata = locate_standard_ui(&tmp, true).expect("TMP metadata");
        assert!(metadata.profile().supports(StandardUiKind::TextMeshPro));
        assert!(!metadata.profile().supports(StandardUiKind::UGui));

        assert_eq!(
            locate_standard_ui(&FakeMetadata::default(), false),
            Err(MetadataError::StandardUiMissing)
        );

        let mut stripped =
            FakeMetadata::default().with_text(2, TMP_NAMESPACE, TMP_CLASS, TMP_TEXT_FIELD, 30);
        stripped.fields.clear();
        assert_eq!(
            locate_standard_ui(&stripped, false),
            Err(MetadataError::StandardUiMissing)
        );

        let mut stripped =
            FakeMetadata::default().with_text(2, TMP_NAMESPACE, TMP_CLASS, TMP_TEXT_FIELD, 30);
        stripped.methods.clear();
        assert!(locate_standard_ui(&stripped, false).is_ok());
        assert_eq!(
            locate_standard_ui(&stripped, true),
            Err(MetadataError::StandardUiMissing)
        );
    }
}
