use crate::diagnostics::trace;
use crate::objects::ObjectRegistry;
use crate::font_substitution::{FontSubstitutionError, FontSubstitutionState, FontWrite};
use crate::runtime_gate::{Il2CppExports, Il2CppRuntimeGate};
use crate::snapshot::{capture_snapshot, CapturedSnapshot};
use crate::writeback::{apply_writes, WritebackError};
use glyphshift_adapter_unity_standard_ui::{StandardUiKind, StandardUiProfile, TextWrite};
use std::ffi::{c_char, c_void, CStr, CString};

const TMP_NAMESPACE: &str = "TMPro";
const TMP_CLASS: &str = "TMP_Text";
const TMP_TEXT_FIELD: &str = "m_text";
const UGUI_NAMESPACE: &str = "UnityEngine.UI";
const UGUI_CLASS: &str = "Text";
const UGUI_TEXT_FIELD: &str = "m_Text";
const SYSTEM_NAMESPACE: &str = "System";
const STRING_CLASS: &str = "String";
const BOOLEAN_CLASS: &str = "Boolean";
const INT32_CLASS: &str = "Int32";
const UNITY_NAMESPACE: &str = "UnityEngine";
const FONT_CLASS: &str = "Font";
const MATERIAL_CLASS: &str = "Material";
const TEXTCORE_LOW_LEVEL_NAMESPACE: &str = "UnityEngine.TextCore.LowLevel";
const GLYPH_RENDER_MODE_CLASS: &str = "GlyphRenderMode";
const TMP_FONT_ASSET_CLASS: &str = "TMP_FontAsset";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetadataError {
    RuntimeMissing,
    StandardUiMissing,
    FontSubstitutionMissing,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FontTarget {
    pub(crate) kind: StandardUiKind,
    pub(crate) font_getter: usize,
    pub(crate) font_setter: usize,
    pub(crate) shared_material_getter: Option<usize>,
    pub(crate) shared_material_setter: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TmpFontFactory {
    pub(crate) class: usize,
    pub(crate) create_font_asset: TmpFontAssetFactory,
    pub(crate) try_add_characters: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TmpFontAssetFactory {
    FilePath(usize),
    FamilyStyle(usize),
    UnityFont(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FontMetadata {
    pub(crate) font_class: usize,
    pub(crate) font_factory: FontFactory,
    pub(crate) targets: Vec<FontTarget>,
    pub(crate) tmp_factory: Option<TmpFontFactory>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontFactory {
    StringConstructor(usize),
    InternalCreateFont {
        default_constructor: usize,
        factory: usize,
    },
}

#[derive(Clone, Copy)]
struct MethodParam {
    class: usize,
    by_ref: bool,
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
    fn exact_method(&self, class: usize, name: &str, params: &[MethodParam]) -> Option<usize> {
        self.method(class, name, params.len() as i32)
    }
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

fn class_across_images(
    source: &impl MetadataSource,
    images: &[usize],
    namespace: &str,
    name: &str,
) -> Option<usize> {
    images
        .iter()
        .find_map(|image| source.class_from_name(*image, namespace, name))
}

fn locate_font_metadata(
    source: &impl MetadataSource,
    standard_ui: &StandardUiMetadata,
) -> Result<FontMetadata, MetadataError> {
    let images = source.images();
    let Some(string_class) = class_across_images(source, &images, SYSTEM_NAMESPACE, STRING_CLASS) else {
        trace("font.metadata.missing.system-string");
        return Err(MetadataError::FontSubstitutionMissing);
    };
    let Some(boolean_class) = class_across_images(source, &images, SYSTEM_NAMESPACE, BOOLEAN_CLASS) else {
        trace("font.metadata.missing.system-boolean");
        return Err(MetadataError::FontSubstitutionMissing);
    };
    let Some(int32_class) = class_across_images(source, &images, SYSTEM_NAMESPACE, INT32_CLASS) else {
        trace("font.metadata.missing.system-int32");
        return Err(MetadataError::FontSubstitutionMissing);
    };
    let Some(font_class) = class_across_images(source, &images, UNITY_NAMESPACE, FONT_CLASS) else {
        trace("font.metadata.missing.unity-font");
        return Err(MetadataError::FontSubstitutionMissing);
    };
    let Some(material_class) = class_across_images(source, &images, UNITY_NAMESPACE, MATERIAL_CLASS) else {
        trace("font.metadata.missing.unity-material");
        return Err(MetadataError::FontSubstitutionMissing);
    };
    let font_factory = if let Some(method) = source.exact_method(
        font_class,
        ".ctor",
        &[MethodParam {
            class: string_class,
            by_ref: false,
        }],
    ) {
        FontFactory::StringConstructor(method)
    } else if let (Some(default_constructor), Some(factory)) = (
        source.exact_method(font_class, ".ctor", &[]),
        source.exact_method(
            font_class,
            "Internal_CreateFont",
            &[
                MethodParam {
                    class: font_class,
                    by_ref: false,
                },
                MethodParam {
                    class: string_class,
                    by_ref: false,
                },
            ],
        ),
    ) {
        trace("font.metadata.using.internal-create-font");
        FontFactory::InternalCreateFont {
            default_constructor,
            factory,
        }
    } else {
        trace("font.metadata.missing.font-file-factory");
        return Err(MetadataError::FontSubstitutionMissing);
    };
    let tmp_font_asset = class_across_images(source, &images, TMP_NAMESPACE, TMP_FONT_ASSET_CLASS);
    let tmp_factory = if standard_ui
        .targets
        .iter()
        .any(|target| target.kind == StandardUiKind::TextMeshPro)
    {
        let Some(tmp_font_asset) = tmp_font_asset else {
            trace("font.metadata.missing.tmp-font-asset");
            return Err(MetadataError::FontSubstitutionMissing);
        };
        let glyph_render_mode = class_across_images(
            source,
            &images,
            TEXTCORE_LOW_LEVEL_NAMESPACE,
            GLYPH_RENDER_MODE_CLASS,
        );
        // Prefer family/style when the runtime exposes it. This lets TextCore select the
        // correct face from TTC collections and follows the user's configured family name.
        // File-path creation remains the portable fallback for runtimes without DynamicOS.
        let create_font_asset = source
            .exact_method(
                tmp_font_asset,
                "CreateFontAsset",
                &[
                    MethodParam { class: string_class, by_ref: false },
                    MethodParam { class: string_class, by_ref: false },
                    MethodParam { class: int32_class, by_ref: false },
                ],
            )
            .map(TmpFontAssetFactory::FamilyStyle)
            .or_else(|| {
                glyph_render_mode.and_then(|glyph_render_mode| {
                    source
                        .exact_method(
                            tmp_font_asset,
                            "CreateFontAsset",
                            &[
                                MethodParam { class: string_class, by_ref: false },
                                MethodParam { class: int32_class, by_ref: false },
                                MethodParam { class: int32_class, by_ref: false },
                                MethodParam { class: int32_class, by_ref: false },
                                MethodParam { class: glyph_render_mode, by_ref: false },
                                MethodParam { class: int32_class, by_ref: false },
                                MethodParam { class: int32_class, by_ref: false },
                            ],
                        )
                        .map(TmpFontAssetFactory::FilePath)
                })
            })
            .or_else(|| {
                source
                    .exact_method(
                        tmp_font_asset,
                        "CreateFontAsset",
                        &[MethodParam { class: font_class, by_ref: false }],
                    )
                    .map(TmpFontAssetFactory::UnityFont)
            });
        let Some(create_font_asset) = create_font_asset else {
            trace("font.metadata.missing.tmp-create-font-asset");
            return Err(MetadataError::FontSubstitutionMissing);
        };
        trace(match create_font_asset {
            TmpFontAssetFactory::FilePath(_) => "font.metadata.tmp-factory.file-path",
            TmpFontAssetFactory::FamilyStyle(_) => "font.metadata.tmp-factory.family-style",
            TmpFontAssetFactory::UnityFont(_) => "font.metadata.tmp-factory.unity-font",
        });
        let Some(try_add_characters) = source.exact_method(
            tmp_font_asset,
            "TryAddCharacters",
            &[
                MethodParam { class: string_class, by_ref: false },
                MethodParam { class: string_class, by_ref: true },
                MethodParam { class: boolean_class, by_ref: false },
            ],
        ) else {
            trace("font.metadata.missing.tmp-try-add-characters");
            return Err(MetadataError::FontSubstitutionMissing);
        };
        Some(TmpFontFactory {
            class: tmp_font_asset,
            create_font_asset,
            try_add_characters,
        })
    } else {
        None
    };

    let mut targets = Vec::with_capacity(standard_ui.targets.len());
    for target in &standard_ui.targets {
        let (font_class_for_target, material_methods) = match target.kind {
            StandardUiKind::TextMeshPro => (
                tmp_font_asset.ok_or(MetadataError::FontSubstitutionMissing)?,
                true,
            ),
            StandardUiKind::UGui => (font_class, false),
        };
        let Some(font_getter) = source.exact_method(target.class, "get_font", &[]) else {
            trace(match target.kind {
                StandardUiKind::TextMeshPro => "font.metadata.missing.tmp-get-font",
                StandardUiKind::UGui => "font.metadata.missing.ugui-get-font",
            });
            return Err(MetadataError::FontSubstitutionMissing);
        };
        let Some(font_setter) = source.exact_method(
                target.class,
                "set_font",
                &[MethodParam {
                    class: font_class_for_target,
                    by_ref: false,
                }],
            ) else {
            trace(match target.kind {
                StandardUiKind::TextMeshPro => "font.metadata.missing.tmp-set-font",
                StandardUiKind::UGui => "font.metadata.missing.ugui-set-font",
            });
            return Err(MetadataError::FontSubstitutionMissing);
        };
        let (shared_material_getter, shared_material_setter) = if material_methods {
            (
                Some(if let Some(method) = source.exact_method(target.class, "get_fontSharedMaterial", &[]) {
                    method
                } else {
                    trace("font.metadata.missing.tmp-get-shared-material");
                    return Err(MetadataError::FontSubstitutionMissing);
                }),
                Some(if let Some(method) = source.exact_method(
                            target.class,
                            "set_fontSharedMaterial",
                            &[MethodParam {
                                class: material_class,
                                by_ref: false,
                            }],
                        ) {
                    method
                } else {
                    trace("font.metadata.missing.tmp-set-shared-material");
                    return Err(MetadataError::FontSubstitutionMissing);
                }),
            )
        } else {
            (None, None)
        };
        targets.push(FontTarget {
            kind: target.kind,
            font_getter,
            font_setter,
            shared_material_getter,
            shared_material_setter,
        });
    }
    Ok(FontMetadata {
        font_class,
        font_factory,
        targets,
        tmp_factory,
    })
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
    font_metadata: Option<FontMetadata>,
    font_substitution: FontSubstitutionState,
    objects: Option<ObjectRegistry>,
}

impl Il2CppStandardUiRuntime {
    pub(crate) fn resolve(
        gate: Il2CppRuntimeGate,
        require_font_substitution: bool,
    ) -> Result<Self, MetadataError> {
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
        let font_metadata = metadata.as_ref().ok().and_then(|metadata| {
            require_font_substitution
                .then(|| locate_font_metadata(&source, metadata))
                .transpose()
                .ok()
                .flatten()
        });
        if require_font_substitution && font_metadata.is_none() {
            if current_thread.is_null() {
                unsafe { (exports.thread_detach)(attached_thread) };
            }
            return Err(MetadataError::FontSubstitutionMissing);
        }
        if current_thread.is_null() {
            unsafe { (exports.thread_detach)(attached_thread) };
        }
        Ok(Self {
            exports,
            metadata: metadata?,
            font_metadata,
            font_substitution: FontSubstitutionState::default(),
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

    pub(crate) unsafe fn apply_fonts_on_current_thread(
        &mut self,
        writes: &[FontWrite],
    ) -> Result<(), FontSubstitutionError> {
        if unsafe { (self.exports.thread_current)() }.is_null() {
            return Err(FontSubstitutionError::ThreadUnavailable);
        }
        let writeback = self
            .exports
            .writeback
            .ok_or(FontSubstitutionError::FontMetadataMissing)?;
        let metadata = self
            .font_metadata
            .as_ref()
            .ok_or(FontSubstitutionError::FontMetadataMissing)?;
        let objects = self
            .objects
            .as_ref()
            .ok_or(FontSubstitutionError::FontMetadataMissing)?;
        unsafe {
            self.font_substitution
                .apply(writeback, metadata, objects, writes)
        }
    }

    pub(crate) unsafe fn restore_fonts_on_current_thread(
        &mut self,
    ) -> Result<(), FontSubstitutionError> {
        if !self.font_substitution.has_applied() {
            return Ok(());
        }
        if unsafe { (self.exports.thread_current)() }.is_null() {
            return Err(FontSubstitutionError::ThreadUnavailable);
        }
        let writeback = self
            .exports
            .writeback
            .ok_or(FontSubstitutionError::FontMetadataMissing)?;
        let metadata = self
            .font_metadata
            .as_ref()
            .ok_or(FontSubstitutionError::FontMetadataMissing)?;
        let objects = self
            .objects
            .as_ref()
            .ok_or(FontSubstitutionError::FontMetadataMissing)?;
        unsafe { self.font_substitution.restore(writeback, metadata, objects) }
    }

    pub(crate) fn has_font_substitutions(&self) -> bool {
        self.font_substitution.has_applied()
    }

    pub(crate) fn mark_collected_font_object(&mut self, object_id: glyphshift_adapter_unity_standard_ui::ManagedObjectId) {
        self.font_substitution.collected(object_id);
    }

    pub(crate) unsafe fn release_object_handles(&mut self) {
        let Some(writeback) = self.exports.writeback else {
            return;
        };
        self.font_substitution.clear();
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

    fn exact_method(&self, class: usize, name: &str, params: &[MethodParam]) -> Option<usize> {
        let writeback = self.exports.writeback?;
        let mut iterator = std::ptr::null_mut::<c_void>();
        loop {
            let method = unsafe {
                (writeback.class_get_methods)(class as *mut c_void, &mut iterator)
            };
            if method.is_null() {
                return None;
            }
            let method_name = unsafe { (writeback.method_get_name)(method) };
            if method_name.is_null() || unsafe { CStr::from_ptr(method_name) }.to_bytes() != name.as_bytes() {
                continue;
            }
            let parameter_count = unsafe { (writeback.method_get_param_count)(method) } as usize;
            if name == ".ctor" {
                trace(&format!("font.metadata.ctor-candidate.count-{parameter_count}"));
            }
            if parameter_count != params.len() {
                continue;
            }
            let mut matches = true;
            for (index, expected) in params.iter().enumerate() {
                let parameter = unsafe { (writeback.method_get_param)(method, index as u32) };
                let actual_by_ref = !parameter.is_null() && unsafe { (writeback.type_is_byref)(parameter) };
                let actual_class = if parameter.is_null() {
                    0
                } else {
                    (unsafe { (writeback.class_from_il2cpp_type)(parameter) }) as usize
                };
                if name == ".ctor" {
                    trace(&format!(
                        "font.metadata.ctor-param-{index}.byref-{}.classmatch-{}",
                        actual_by_ref,
                        actual_class == expected.class
                    ));
                }
                if parameter.is_null() || actual_by_ref != expected.by_ref || actual_class != expected.class {
                    matches = false;
                    break;
                }
            }
            if matches {
                return Some(method as usize);
            }
        }
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
