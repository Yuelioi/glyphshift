use crate::diagnostics::trace;
use crate::metadata::{FontFactory, FontMetadata, TmpFontAssetFactory};
use crate::objects::ObjectRegistry;
use crate::runtime_gate::Il2CppWritebackExports;
use glyphshift_adapter_unity_standard_ui::{ManagedObjectId, StandardUiKind};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{c_void, CString};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FontWrite {
    object_id: ManagedObjectId,
    kind: StandardUiKind,
    family: Box<str>,
    units: Vec<u16>,
}

impl FontWrite {
    pub(crate) fn new(
        object_id: ManagedObjectId,
        kind: StandardUiKind,
        family: impl Into<Box<str>>,
        units: impl Into<Vec<u16>>,
    ) -> Self {
        Self {
            object_id,
            kind,
            family: family.into(),
            units: units.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontSubstitutionError {
    FontUnavailable,
    FontMetadataMissing,
    ObjectAllocationFailed,
    HandleAllocationFailed,
    StringAllocationFailed,
    InvalidText,
    ThreadUnavailable,
    InvokeException,
    GlyphUnavailable,
}

struct ManagedHandle {
    exports: Il2CppWritebackExports,
    handle: usize,
}

impl ManagedHandle {
    unsafe fn new(
        exports: Il2CppWritebackExports,
        object: *mut c_void,
    ) -> Result<Self, FontSubstitutionError> {
        if object.is_null() {
            return Err(FontSubstitutionError::ObjectAllocationFailed);
        }
        let handle = unsafe { (exports.gchandle_new)(object, false) };
        if handle == 0 {
            return Err(FontSubstitutionError::HandleAllocationFailed);
        }
        Ok(Self { exports, handle })
    }

    unsafe fn optional(
        exports: Il2CppWritebackExports,
        object: *mut c_void,
    ) -> Result<Option<Self>, FontSubstitutionError> {
        if object.is_null() {
            Ok(None)
        } else {
            unsafe { Self::new(exports, object).map(Some) }
        }
    }

    unsafe fn target(&self) -> Option<*mut c_void> {
        let target = unsafe { (self.exports.gchandle_get_target)(self.handle) };
        (!target.is_null()).then_some(target)
    }
}

impl Drop for ManagedHandle {
    fn drop(&mut self) {
        unsafe { (self.exports.gchandle_free)(self.handle) };
    }
}

struct FontResource {
    font: Option<ManagedHandle>,
    tmp_asset: Option<ManagedHandle>,
    glyphs: BTreeSet<char>,
}

struct OriginalFont {
    kind: StandardUiKind,
    font: Option<ManagedHandle>,
    material: Option<ManagedHandle>,
    applied: usize,
}

#[derive(Default)]
pub(crate) struct FontSubstitutionState {
    resources: BTreeMap<Box<str>, FontResource>,
    originals: BTreeMap<ManagedObjectId, OriginalFont>,
}

impl FontSubstitutionState {
    pub(crate) fn has_applied(&self) -> bool {
        self.originals.values().any(|original| original.applied != 0)
    }

    pub(crate) fn collected(&mut self, object_id: ManagedObjectId) {
        self.originals.remove(&object_id);
    }

    pub(crate) unsafe fn apply(
        &mut self,
        exports: Il2CppWritebackExports,
        metadata: &FontMetadata,
        objects: &ObjectRegistry,
        writes: &[FontWrite],
    ) -> Result<(), FontSubstitutionError> {
        for write in writes {
            let Some(object) = (unsafe { objects.target(exports, write.object_id) }) else {
                continue;
            };
            let target = metadata
                .targets
                .iter()
                .find(|target| target.kind == write.kind)
                .copied()
                .ok_or(FontSubstitutionError::FontMetadataMissing)?;
            let applied = unsafe {
                self.ensure_resource(exports, metadata, &write.family, write.kind, &write.units)?
            };
            if let std::collections::btree_map::Entry::Vacant(entry) =
                self.originals.entry(write.object_id)
            {
                let font = unsafe { invoke_object(exports, target.font_getter, object, &mut [])? };
                let material = if let Some(getter) = target.shared_material_getter {
                    unsafe { invoke_object(exports, getter, object, &mut [])? }
                } else {
                    std::ptr::null_mut()
                };
                entry.insert(OriginalFont {
                    kind: write.kind,
                    font: unsafe { ManagedHandle::optional(exports, font)? },
                    material: unsafe { ManagedHandle::optional(exports, material)? },
                    applied: 0,
                });
            }
            let current = unsafe { invoke_object(exports, target.font_getter, object, &mut [])? };
            if current as usize != applied {
                unsafe { invoke_setter(exports, target.font_setter, object, applied as *mut c_void)? };
            }
            if let Some(original) = self.originals.get_mut(&write.object_id) {
                original.applied = applied;
            }
        }
        Ok(())
    }

    pub(crate) unsafe fn restore(
        &mut self,
        exports: Il2CppWritebackExports,
        metadata: &FontMetadata,
        objects: &ObjectRegistry,
    ) -> Result<(), FontSubstitutionError> {
        let ids = self.originals.keys().copied().collect::<Vec<_>>();
        for object_id in ids {
            let Some(object) = (unsafe { objects.target(exports, object_id) }) else {
                self.originals.remove(&object_id);
                continue;
            };
            let Some(original) = self.originals.get(&object_id) else {
                continue;
            };
            let target = metadata
                .targets
                .iter()
                .find(|target| target.kind == original.kind)
                .copied()
                .ok_or(FontSubstitutionError::FontMetadataMissing)?;
            let current = unsafe { invoke_object(exports, target.font_getter, object, &mut [])? };
            if current as usize != original.applied {
                self.originals.remove(&object_id);
                continue;
            }
            let original_font = original
                .font
                .as_ref()
                .and_then(|handle| unsafe { handle.target() })
                .unwrap_or(std::ptr::null_mut());
            unsafe { invoke_setter(exports, target.font_setter, object, original_font)? };
            if let Some(material_setter) = target.shared_material_setter {
                let original_material = original
                    .material
                    .as_ref()
                    .and_then(|handle| unsafe { handle.target() })
                    .unwrap_or(std::ptr::null_mut());
                unsafe { invoke_setter(exports, material_setter, object, original_material)? };
            }
            self.originals.remove(&object_id);
        }
        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        self.originals.clear();
        self.resources.clear();
    }

    unsafe fn ensure_resource(
        &mut self,
        exports: Il2CppWritebackExports,
        metadata: &FontMetadata,
        family: &str,
        kind: StandardUiKind,
        required_units: &[u16],
    ) -> Result<usize, FontSubstitutionError> {
        let key = Box::<str>::from(family.trim().to_ascii_lowercase());
        if !self.resources.contains_key(&key) {
            self.resources.insert(
                key.clone(),
                FontResource {
                    font: None,
                    tmp_asset: None,
                    glyphs: BTreeSet::new(),
                },
            );
        }
        let resource = self
            .resources
            .get_mut(&key)
            .ok_or(FontSubstitutionError::FontUnavailable)?;
        match kind {
            StandardUiKind::TextMeshPro => {
                if resource.tmp_asset.is_none() {
                    let path = resolve_font_file(family)
                        .ok_or(FontSubstitutionError::FontUnavailable)?;
                    if matches!(
                        metadata.tmp_factory.map(|factory| factory.create_font_asset),
                        Some(TmpFontAssetFactory::UnityFont(_))
                    ) && resource.font.is_none()
                    {
                        resource.font = Some(unsafe {
                            create_unity_font_from_path(exports, metadata, &path)?
                        });
                    }
                    resource.tmp_asset = Some(unsafe {
                        create_tmp_asset(
                            exports,
                            metadata,
                            family,
                            &path,
                            resource.font.as_ref(),
                        )?
                    });
                }
                unsafe { ensure_tmp_glyphs(exports, metadata, resource, required_units)? };
                resource
                    .tmp_asset
                    .as_ref()
                    .and_then(|handle| unsafe { handle.target() })
                    .map(|target| target as usize)
                    .ok_or(FontSubstitutionError::ObjectAllocationFailed)
            }
            StandardUiKind::UGui => {
                if resource.font.is_none() {
                    resource.font = Some(unsafe {
                        create_unity_font(exports, metadata, unity_font_name(family))?
                    });
                }
                resource
                    .font
                    .as_ref()
                    .and_then(|font| unsafe { font.target() })
                    .map(|target| target as usize)
                    .ok_or(FontSubstitutionError::ObjectAllocationFailed)
            }
        }
    }
}

unsafe fn create_unity_font(
    exports: Il2CppWritebackExports,
    metadata: &FontMetadata,
    family: &str,
) -> Result<ManagedHandle, FontSubstitutionError> {
    let object = unsafe { (exports.object_new)(metadata.font_class as *const c_void) };
    if object.is_null() {
        trace("font.resource.unity-font.object-new-null");
        return Err(FontSubstitutionError::ObjectAllocationFailed);
    }
    let object_handle = unsafe { ManagedHandle::new(exports, object)? };
    let string = unsafe { managed_string(exports, &family.encode_utf16().collect::<Vec<_>>())? };
    let string_object = unsafe { string.target() }.ok_or_else(|| {
        trace("font.resource.path-string.collected");
        FontSubstitutionError::StringAllocationFailed
    })?;
    match metadata.font_factory {
        FontFactory::StringConstructor(constructor) => {
            let mut parameters = [string_object];
            unsafe { invoke_object(exports, constructor, object, &mut parameters)? };
        }
        FontFactory::InternalCreateFont {
            default_constructor,
            factory,
        } => {
            unsafe { invoke_object(exports, default_constructor, object, &mut [])? };
            let mut parameters = [object, string_object];
            unsafe {
                invoke_object(
                    exports,
                    factory,
                    std::ptr::null_mut(),
                    &mut parameters,
                )?
            };
        }
    }
    Ok(object_handle)
}

unsafe fn create_unity_font_from_path(
    exports: Il2CppWritebackExports,
    metadata: &FontMetadata,
    path: &Path,
) -> Result<ManagedHandle, FontSubstitutionError> {
    if matches!(metadata.font_factory, FontFactory::StringConstructor(_)) {
        return unsafe { create_unity_font(exports, metadata, &path.to_string_lossy()) };
    }

    let object = unsafe { (exports.object_new)(metadata.font_class as *const c_void) };
    if object.is_null() {
        trace("font.resource.unity-font.path.object-new-null");
        return Err(FontSubstitutionError::ObjectAllocationFailed);
    }
    let object_handle = unsafe { ManagedHandle::new(exports, object)? };
    let path = path.to_string_lossy();
    let path = unsafe { managed_string(exports, &path.encode_utf16().collect::<Vec<_>>())? };
    let path = unsafe { path.target() }.ok_or_else(|| {
        trace("font.resource.unity-font.path-string.collected");
        FontSubstitutionError::StringAllocationFailed
    })?;

    const ICALL: &str =
        "UnityEngine.Font::Internal_CreateFontFromPath(UnityEngine.Font,System.String)";
    let name = CString::new(ICALL).map_err(|_| FontSubstitutionError::FontMetadataMissing)?;
    let factory = unsafe { (exports.resolve_icall)(name.as_ptr()) };
    if factory.is_null() {
        trace("font.resource.unity-font.path-icall-missing");
        return Err(FontSubstitutionError::FontMetadataMissing);
    }
    type InternalCreateFontFromPath = unsafe extern "C" fn(*mut c_void, *mut c_void);
    let factory = unsafe {
        std::mem::transmute::<*const c_void, InternalCreateFontFromPath>(factory)
    };
    trace("font.resource.unity-font.path-icall");
    unsafe { factory(object, path) };
    Ok(object_handle)
}

unsafe fn create_tmp_asset(
    exports: Il2CppWritebackExports,
    metadata: &FontMetadata,
    family: &str,
    path: &Path,
    font: Option<&ManagedHandle>,
) -> Result<ManagedHandle, FontSubstitutionError> {
    let factory = metadata
        .tmp_factory
        .ok_or(FontSubstitutionError::FontMetadataMissing)?;
    let asset = match factory.create_font_asset {
        TmpFontAssetFactory::FilePath(method) => {
            let path = path.to_string_lossy();
            let path = unsafe {
                managed_string(exports, &path.encode_utf16().collect::<Vec<_>>())?
            };
            let path = unsafe { path.target() }
                .ok_or(FontSubstitutionError::StringAllocationFailed)?;
            let mut face_index = 0i32;
            let mut sampling_point_size = 90i32;
            let mut atlas_padding = 9i32;
            // UnityEngine.TextCore.LowLevel.GlyphRenderMode.SDFAA.
            let mut render_mode = 4165i32;
            let mut atlas_width = 1024i32;
            let mut atlas_height = 1024i32;
            let mut parameters = [
                path,
                (&mut face_index as *mut i32).cast::<c_void>(),
                (&mut sampling_point_size as *mut i32).cast::<c_void>(),
                (&mut atlas_padding as *mut i32).cast::<c_void>(),
                (&mut render_mode as *mut i32).cast::<c_void>(),
                (&mut atlas_width as *mut i32).cast::<c_void>(),
                (&mut atlas_height as *mut i32).cast::<c_void>(),
            ];
            trace("font.resource.tmp-font-asset.file-path");
            unsafe { invoke_object(exports, method, std::ptr::null_mut(), &mut parameters)? }
        }
        TmpFontAssetFactory::FamilyStyle(method) => {
            let family = unsafe {
                managed_string(
                    exports,
                    &unity_font_name(family).encode_utf16().collect::<Vec<_>>(),
                )?
            };
            let style = unsafe {
                managed_string(exports, &"Regular".encode_utf16().collect::<Vec<_>>())?
            };
            let family = unsafe { family.target() }
                .ok_or(FontSubstitutionError::StringAllocationFailed)?;
            let style = unsafe { style.target() }
                .ok_or(FontSubstitutionError::StringAllocationFailed)?;
            let mut point_size = 90i32;
            let mut parameters = [
                family,
                style,
                (&mut point_size as *mut i32).cast::<c_void>(),
            ];
            trace("font.resource.tmp-font-asset.family-style");
            unsafe { invoke_object(exports, method, std::ptr::null_mut(), &mut parameters)? }
        }
        TmpFontAssetFactory::UnityFont(method) => {
            let font = font
                .and_then(|font| unsafe { font.target() })
                .ok_or_else(|| {
                    trace("font.resource.unity-font.collected-before-tmp");
                    FontSubstitutionError::ObjectAllocationFailed
                })?;
            let mut parameters = [font];
            trace("font.resource.tmp-font-asset.unity-font");
            unsafe { invoke_object(exports, method, std::ptr::null_mut(), &mut parameters)? }
        }
    };
    if asset.is_null() {
        trace("font.resource.tmp-font-asset.create-null");
        return Err(FontSubstitutionError::ObjectAllocationFailed);
    }
    unsafe { ManagedHandle::new(exports, asset) }
}

fn unity_font_name(family: &str) -> &str {
    family
        .split_once(" & ")
        .map_or_else(|| family.trim(), |(primary, _)| primary.trim())
}

unsafe fn ensure_tmp_glyphs(
    exports: Il2CppWritebackExports,
    metadata: &FontMetadata,
    resource: &mut FontResource,
    required_units: &[u16],
) -> Result<(), FontSubstitutionError> {
    let required = String::from_utf16(required_units).map_err(|_| FontSubstitutionError::InvalidText)?;
    let missing = required
        .chars()
        .filter(|character| !resource.glyphs.contains(character))
        .collect::<BTreeSet<_>>();
    if missing.is_empty() {
        return Ok(());
    }
    let characters = missing.iter().collect::<String>();
    let characters = unsafe {
        managed_string(exports, &characters.encode_utf16().collect::<Vec<_>>())?
    };
    let characters = unsafe { characters.target() }
        .ok_or(FontSubstitutionError::StringAllocationFailed)?;
    let factory = metadata
        .tmp_factory
        .ok_or(FontSubstitutionError::FontMetadataMissing)?;
    let asset = resource
        .tmp_asset
        .as_ref()
        .and_then(|handle| unsafe { handle.target() })
        .ok_or_else(|| {
            trace("font.resource.tmp-font-asset.collected-before-glyphs");
            FontSubstitutionError::ObjectAllocationFailed
        })?;
    let mut missing_characters = std::ptr::null_mut::<c_void>();
    let mut include_font_features = false;
    let mut parameters = [
        characters,
        (&mut missing_characters as *mut *mut c_void).cast::<c_void>(),
        (&mut include_font_features as *mut bool).cast::<c_void>(),
    ];
    let result = unsafe {
        invoke_object(
            exports,
            factory.try_add_characters,
            asset,
            &mut parameters,
        )?
    };
    if result.is_null() {
        trace("font.glyph.result.null");
        return Err(FontSubstitutionError::GlyphUnavailable);
    }
    let unboxed = unsafe { (exports.object_unbox)(result) };
    if unboxed.is_null() {
        trace("font.glyph.result.unbox-null");
        return Err(FontSubstitutionError::GlyphUnavailable);
    }
    let added = unsafe { *unboxed.cast::<bool>() };
    let missing_length = if missing_characters.is_null() {
        0
    } else {
        unsafe { (exports.string_length)(missing_characters) }
    };
    trace(match (added, missing_length > 0) {
        (true, false) => "font.glyph.result.true.missing-empty",
        (true, true) => "font.glyph.result.true.missing-nonempty",
        (false, false) => "font.glyph.result.false.missing-empty",
        (false, true) => "font.glyph.result.false.missing-nonempty",
    });
    if !added || missing_length > 0 {
        return Err(FontSubstitutionError::GlyphUnavailable);
    }
    resource.glyphs.extend(missing);
    Ok(())
}

unsafe fn managed_string(
    exports: Il2CppWritebackExports,
    units: &[u16],
) -> Result<ManagedHandle, FontSubstitutionError> {
    let length = i32::try_from(units.len()).map_err(|_| FontSubstitutionError::InvalidText)?;
    let string = unsafe { (exports.string_new_utf16)(units.as_ptr(), length) };
    if string.is_null() {
        return Err(FontSubstitutionError::StringAllocationFailed);
    }
    unsafe { ManagedHandle::new(exports, string) }
}

unsafe fn invoke_setter(
    exports: Il2CppWritebackExports,
    method: usize,
    object: *mut c_void,
    value: *mut c_void,
) -> Result<(), FontSubstitutionError> {
    let mut parameters = [value];
    unsafe { invoke_object(exports, method, object, &mut parameters) }.map(|_| ())
}

unsafe fn invoke_object(
    exports: Il2CppWritebackExports,
    method: usize,
    object: *mut c_void,
    parameters: &mut [*mut c_void],
) -> Result<*mut c_void, FontSubstitutionError> {
    let mut exception = std::ptr::null_mut::<c_void>();
    let parameters = if parameters.is_empty() {
        std::ptr::null_mut()
    } else {
        parameters.as_mut_ptr()
    };
    let result = unsafe {
        (exports.runtime_invoke)(method as *const c_void, object, parameters, &mut exception)
    };
    if !exception.is_null() {
        trace("font.invoke.exception");
        return Err(FontSubstitutionError::InvokeException);
    }
    Ok(result)
}

#[cfg(windows)]
fn resolve_font_file(family: &str) -> Option<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
    use winreg::RegKey;

    const FONT_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts";
    for hive in [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
        let Ok(fonts) = RegKey::predef(hive).open_subkey_with_flags(FONT_KEY, KEY_READ) else {
            continue;
        };
        for (label, _) in fonts.enum_values().flatten() {
            if !normalize_font_registry_label(&label).eq_ignore_ascii_case(family.trim()) {
                continue;
            }
            let Ok(value) = fonts.get_value::<String, _>(&label) else {
                continue;
            };
            if let Some(path) = resolve_registry_font_path(&value) {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn resolve_font_file(_family: &str) -> Option<PathBuf> {
    None
}

fn normalize_font_registry_label(label: &str) -> &str {
    let label = label.trim().trim_start_matches('@').trim();
    label
        .rfind(" (")
        .filter(|_| label.ends_with(')'))
        .map_or(label, |suffix| &label[..suffix])
}

fn resolve_registry_font_path(value: &str) -> Option<PathBuf> {
    let value = PathBuf::from(value.trim());
    if value.is_absolute() && value.is_file() {
        return Some(value);
    }
    let file = value.file_name()?;
    if let Some(windows) = std::env::var_os("WINDIR") {
        let path = PathBuf::from(windows).join("Fonts").join(file);
        if path.is_file() {
            return Some(path);
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let path = PathBuf::from(local)
            .join("Microsoft")
            .join("Windows")
            .join("Fonts")
            .join(file);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_labels_match_user_facing_font_families() {
        assert_eq!(
            normalize_font_registry_label("Microsoft YaHei & Microsoft YaHei UI (TrueType)"),
            "Microsoft YaHei & Microsoft YaHei UI"
        );
        assert_eq!(normalize_font_registry_label("@Synthetic Sans (OpenType)"), "Synthetic Sans");
    }

    #[test]
    fn composite_windows_font_labels_choose_a_concrete_unity_family() {
        assert_eq!(
            unity_font_name("Microsoft YaHei & Microsoft YaHei UI"),
            "Microsoft YaHei"
        );
        assert_eq!(unity_font_name("Noto Sans CJK SC"), "Noto Sans CJK SC");
    }
}
