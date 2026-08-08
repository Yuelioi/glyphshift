use crate::{MonoRuntimeGate, RuntimeGateError};
use glyphshift_adapter_unity_mono_standard_ui::{
    StandardUiKind, MAX_TEXT_UNITS, MAX_TRACKED_OBJECTS,
};
use std::ffi::{c_char, c_int, c_void, CStr, CString};

use crate::StandardUiProfile;

const MAX_LOADED_ASSEMBLIES: usize = 1024;

#[repr(C)]
struct MonoDomain {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoThread {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoAssembly {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoImage {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoClass {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoMethod {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoClassField {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoType {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoObject {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoArray {
    _private: [u8; 0],
}

#[repr(C)]
struct MonoString {
    _private: [u8; 0],
}

type AssemblyCallback = unsafe extern "C" fn(*mut MonoAssembly, *mut c_void);
type GetRootDomain = unsafe extern "C" fn() -> *mut MonoDomain;
type ThreadAttach = unsafe extern "C" fn(*mut MonoDomain) -> *mut MonoThread;
type ThreadDetach = unsafe extern "C" fn(*mut MonoThread);
type AssemblyForeach = unsafe extern "C" fn(Option<AssemblyCallback>, *mut c_void);
type AssemblyGetImage = unsafe extern "C" fn(*mut MonoAssembly) -> *mut MonoImage;
type ClassFromName =
    unsafe extern "C" fn(*mut MonoImage, *const c_char, *const c_char) -> *mut MonoClass;
type ClassGetName = unsafe extern "C" fn(*mut MonoClass) -> *const c_char;
type ClassGetMethodFromName =
    unsafe extern "C" fn(*mut MonoClass, *const c_char, c_int) -> *mut MonoMethod;
type ClassGetFieldFromName =
    unsafe extern "C" fn(*mut MonoClass, *const c_char) -> *mut MonoClassField;
type CompileMethod = unsafe extern "C" fn(*mut MonoMethod) -> *mut c_void;
type RuntimeInvoke = unsafe extern "C" fn(
    *mut MonoMethod,
    *mut c_void,
    *mut *mut c_void,
    *mut *mut MonoObject,
) -> *mut MonoObject;
type ClassGetType = unsafe extern "C" fn(*mut MonoClass) -> *mut MonoType;
type TypeGetObject = unsafe extern "C" fn(*mut MonoDomain, *mut MonoType) -> *mut MonoObject;
type ArrayLength = unsafe extern "C" fn(*mut MonoArray) -> usize;
type ArrayAddrWithSize = unsafe extern "C" fn(*mut MonoArray, c_int, usize) -> *mut c_char;
type ClassIsAssignableFrom = unsafe extern "C" fn(*mut MonoClass, *mut MonoClass) -> c_int;
type ObjectGetClass = unsafe extern "C" fn(*mut MonoObject) -> *mut MonoClass;
type FieldGetValue = unsafe extern "C" fn(*mut MonoObject, *mut MonoClassField, *mut c_void);
type StringChars = unsafe extern "C" fn(*mut MonoString) -> *mut u16;
type StringLength = unsafe extern "C" fn(*mut MonoString) -> c_int;
type StringNewUtf16 = unsafe extern "C" fn(*mut MonoDomain, *const u16, c_int) -> *mut MonoString;
pub(crate) type MonoGcHandle = usize;
type GcHandleNew = unsafe extern "C" fn(*mut MonoObject, c_int) -> MonoGcHandle;
type GcHandleNewWeakRef = unsafe extern "C" fn(*mut MonoObject, c_int) -> MonoGcHandle;
type GcHandleGetTarget = unsafe extern "C" fn(MonoGcHandle) -> *mut MonoObject;
type GcHandleFree = unsafe extern "C" fn(MonoGcHandle);

#[derive(Clone, Copy)]
pub(crate) struct MonoApi {
    get_root_domain: GetRootDomain,
    thread_attach: ThreadAttach,
    thread_detach: ThreadDetach,
    assembly_foreach: AssemblyForeach,
    assembly_get_image: AssemblyGetImage,
    class_from_name: ClassFromName,
    class_get_name: ClassGetName,
    class_get_namespace: ClassGetName,
    class_get_method_from_name: ClassGetMethodFromName,
    class_get_field_from_name: ClassGetFieldFromName,
    compile_method: CompileMethod,
    runtime_invoke: RuntimeInvoke,
    class_get_type: ClassGetType,
    type_get_object: TypeGetObject,
    array_length: ArrayLength,
    array_addr_with_size: ArrayAddrWithSize,
    class_is_assignable_from: ClassIsAssignableFrom,
    object_get_class: ObjectGetClass,
    field_get_value: FieldGetValue,
    string_chars: StringChars,
    string_length: StringLength,
    string_new_utf16: StringNewUtf16,
    gchandle_new: GcHandleNew,
    gchandle_new_weakref: GcHandleNewWeakRef,
    gchandle_get_target: GcHandleGetTarget,
    gchandle_free: GcHandleFree,
}

impl MonoApi {
    unsafe fn from_gate(gate: &MonoRuntimeGate) -> Result<Self, LateAttachError> {
        Ok(Self {
            get_root_domain: unsafe { load_function(gate, "mono_get_root_domain")? },
            thread_attach: unsafe { load_function(gate, "mono_thread_attach")? },
            thread_detach: unsafe { load_function(gate, "mono_thread_detach")? },
            assembly_foreach: unsafe { load_function(gate, "mono_assembly_foreach")? },
            assembly_get_image: unsafe { load_function(gate, "mono_assembly_get_image")? },
            class_from_name: unsafe { load_function(gate, "mono_class_from_name_case")? },
            class_get_name: unsafe { load_function(gate, "mono_class_get_name")? },
            class_get_namespace: unsafe { load_function(gate, "mono_class_get_namespace")? },
            class_get_method_from_name: unsafe {
                load_function(gate, "mono_class_get_method_from_name")?
            },
            class_get_field_from_name: unsafe {
                load_function(gate, "mono_class_get_field_from_name")?
            },
            compile_method: unsafe { load_function(gate, "mono_compile_method")? },
            runtime_invoke: unsafe { load_function(gate, "mono_runtime_invoke")? },
            class_get_type: unsafe { load_function(gate, "mono_class_get_type")? },
            type_get_object: unsafe { load_function(gate, "mono_type_get_object")? },
            array_length: unsafe { load_function(gate, "mono_array_length")? },
            array_addr_with_size: unsafe { load_function(gate, "mono_array_addr_with_size")? },
            class_is_assignable_from: unsafe {
                load_function(gate, "mono_class_is_assignable_from")?
            },
            object_get_class: unsafe { load_function(gate, "mono_object_get_class")? },
            field_get_value: unsafe { load_function(gate, "mono_field_get_value")? },
            string_chars: unsafe { load_function(gate, "mono_string_chars")? },
            string_length: unsafe { load_function(gate, "mono_string_length")? },
            string_new_utf16: unsafe { load_function(gate, "mono_string_new_utf16")? },
            gchandle_new: unsafe { load_function(gate, "mono_gchandle_new_v2")? },
            gchandle_new_weakref: unsafe { load_function(gate, "mono_gchandle_new_weakref_v2")? },
            gchandle_get_target: unsafe { load_function(gate, "mono_gchandle_get_target_v2")? },
            gchandle_free: unsafe { load_function(gate, "mono_gchandle_free_v2")? },
        })
    }

    unsafe fn loaded_images(self) -> Result<LoadedImages, LateAttachError> {
        let mut images = LoadedImages::new(self.assembly_get_image);
        unsafe {
            (self.assembly_foreach)(
                Some(collect_assembly_image),
                (&mut images as *mut LoadedImages).cast(),
            );
        }
        if images.overflowed {
            Err(LateAttachError::AssemblyLimitExceeded)
        } else {
            Ok(images)
        }
    }

    pub(crate) unsafe fn attach_foreign_thread(
        self,
        domain: usize,
    ) -> Result<MonoThreadGuard, MonoApiError> {
        let thread = unsafe { (self.thread_attach)(domain as *mut MonoDomain) };
        if thread.is_null() {
            Err(MonoApiError::ThreadAttachFailed)
        } else {
            Ok(MonoThreadGuard {
                detach: self.thread_detach,
                thread,
            })
        }
    }

    pub(crate) unsafe fn enumerate_weak_handles(
        self,
        domain: usize,
        class: usize,
        enumeration_method: usize,
    ) -> Result<Vec<MonoGcHandle>, MonoApiError> {
        let class = class as *mut MonoClass;
        let managed_type = unsafe { (self.class_get_type)(class) };
        if managed_type.is_null() {
            return Err(MonoApiError::ManagedTypeUnavailable);
        }
        let type_object =
            unsafe { (self.type_get_object)(domain as *mut MonoDomain, managed_type) };
        if type_object.is_null() {
            return Err(MonoApiError::ManagedTypeUnavailable);
        }

        let mut include_inactive: c_int = 1;
        let mut arguments = [
            type_object.cast::<c_void>(),
            (&mut include_inactive as *mut c_int).cast::<c_void>(),
        ];
        let mut exception = std::ptr::null_mut();
        let array = unsafe {
            (self.runtime_invoke)(
                enumeration_method as *mut MonoMethod,
                std::ptr::null_mut(),
                arguments.as_mut_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            return Err(MonoApiError::ManagedException);
        }
        if array.is_null() {
            return Err(MonoApiError::ObjectEnumerationFailed);
        }

        let array_handle = unsafe { (self.gchandle_new)(array, 0) };
        if array_handle == 0 {
            return Err(MonoApiError::GcHandleUnavailable);
        }
        let result = unsafe { self.copy_array_to_weak_handles(array_handle, class) };
        unsafe { (self.gchandle_free)(array_handle) };
        result
    }

    unsafe fn copy_array_to_weak_handles(
        self,
        array_handle: MonoGcHandle,
        expected_class: *mut MonoClass,
    ) -> Result<Vec<MonoGcHandle>, MonoApiError> {
        let array = unsafe { (self.gchandle_get_target)(array_handle) }.cast::<MonoArray>();
        if array.is_null() {
            return Err(MonoApiError::ObjectEnumerationFailed);
        }
        let length = unsafe { (self.array_length)(array) };
        if length > MAX_TRACKED_OBJECTS {
            return Err(MonoApiError::ObjectLimitExceeded);
        }

        let mut handles = Vec::with_capacity(length);
        for index in 0..length {
            let array = unsafe { (self.gchandle_get_target)(array_handle) }.cast::<MonoArray>();
            if array.is_null() {
                unsafe { self.free_handles(&handles) };
                return Err(MonoApiError::ObjectEnumerationFailed);
            }
            let element = unsafe {
                (self.array_addr_with_size)(
                    array,
                    std::mem::size_of::<*mut MonoObject>() as c_int,
                    index,
                )
            };
            if element.is_null() {
                continue;
            }
            let object = unsafe { *element.cast::<*mut MonoObject>() };
            if object.is_null() {
                continue;
            }
            let actual_class = unsafe { (self.object_get_class)(object) };
            if actual_class.is_null()
                || unsafe { (self.class_is_assignable_from)(expected_class, actual_class) } == 0
            {
                continue;
            }
            let handle = unsafe { (self.gchandle_new_weakref)(object, 0) };
            if handle == 0 {
                unsafe { self.free_handles(&handles) };
                return Err(MonoApiError::GcHandleUnavailable);
            }
            handles.push(handle);
        }
        Ok(handles)
    }

    pub(crate) unsafe fn weak_handle_for_object(self, object: usize) -> Option<MonoGcHandle> {
        let handle = unsafe { (self.gchandle_new_weakref)(object as *mut MonoObject, 0) };
        (handle != 0).then_some(handle)
    }

    pub(crate) unsafe fn handle_target(self, handle: MonoGcHandle) -> Option<usize> {
        let target = unsafe { (self.gchandle_get_target)(handle) };
        (!target.is_null()).then_some(target as usize)
    }

    pub(crate) unsafe fn free_handle(self, handle: MonoGcHandle) {
        unsafe { (self.gchandle_free)(handle) };
    }

    unsafe fn free_handles(self, handles: &[MonoGcHandle]) {
        for handle in handles {
            unsafe { (self.gchandle_free)(*handle) };
        }
    }

    pub(crate) unsafe fn read_handle_field_utf16(
        self,
        handle: MonoGcHandle,
        field: usize,
    ) -> Option<Vec<u16>> {
        let object = unsafe { (self.gchandle_get_target)(handle) };
        if object.is_null() {
            return None;
        }
        let mut string = std::ptr::null_mut::<MonoString>();
        unsafe {
            (self.field_get_value)(
                object,
                field as *mut MonoClassField,
                (&mut string as *mut *mut MonoString).cast(),
            );
        }
        unsafe { self.copy_string_utf16(string as usize) }
    }

    pub(crate) unsafe fn copy_string_utf16(self, string: usize) -> Option<Vec<u16>> {
        let string = string as *mut MonoString;
        if string.is_null() {
            return Some(Vec::new());
        }
        let length = unsafe { (self.string_length)(string) };
        let length = usize::try_from(length).ok()?;
        if length > MAX_TEXT_UNITS {
            return None;
        }
        if length == 0 {
            return Some(Vec::new());
        }
        let chars = unsafe { (self.string_chars)(string) };
        if chars.is_null() {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts(chars, length) }.to_vec())
    }

    pub(crate) unsafe fn new_string_handle_utf16(
        self,
        domain: usize,
        units: &[u16],
    ) -> Option<MonoGcHandle> {
        if units.len() > MAX_TEXT_UNITS || String::from_utf16(units).is_err() {
            return None;
        }
        let length = c_int::try_from(units.len()).ok()?;
        let string =
            unsafe { (self.string_new_utf16)(domain as *mut MonoDomain, units.as_ptr(), length) };
        if string.is_null() {
            return None;
        }
        let handle = unsafe { (self.gchandle_new)(string.cast(), 0) };
        (handle != 0).then_some(handle)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MonoApiError {
    ThreadAttachFailed,
    ManagedTypeUnavailable,
    ManagedException,
    ObjectEnumerationFailed,
    ObjectLimitExceeded,
    GcHandleUnavailable,
}

unsafe fn load_function<T: Copy>(
    gate: &MonoRuntimeGate,
    name: &'static str,
) -> Result<T, LateAttachError> {
    let address = gate
        .export_address(name)
        .ok_or(LateAttachError::Gate(RuntimeGateError::MissingExport(name)))?;
    if std::mem::size_of::<T>() != std::mem::size_of::<usize>() {
        return Err(LateAttachError::InvalidFunctionPointer);
    }
    Ok(unsafe { std::mem::transmute_copy(&address) })
}

pub(crate) struct MonoThreadGuard {
    detach: ThreadDetach,
    thread: *mut MonoThread,
}

impl Drop for MonoThreadGuard {
    fn drop(&mut self) {
        unsafe { (self.detach)(self.thread) };
    }
}

struct LoadedImages {
    get_image: AssemblyGetImage,
    items: [*mut MonoImage; MAX_LOADED_ASSEMBLIES],
    len: usize,
    overflowed: bool,
}

impl LoadedImages {
    fn new(get_image: AssemblyGetImage) -> Self {
        Self {
            get_image,
            items: [std::ptr::null_mut(); MAX_LOADED_ASSEMBLIES],
            len: 0,
            overflowed: false,
        }
    }

    fn as_slice(&self) -> &[*mut MonoImage] {
        &self.items[..self.len]
    }
}

unsafe extern "C" fn collect_assembly_image(assembly: *mut MonoAssembly, data: *mut c_void) {
    if assembly.is_null() || data.is_null() {
        return;
    }
    let images = unsafe { &mut *data.cast::<LoadedImages>() };
    if images.len == images.items.len() {
        images.overflowed = true;
        return;
    }
    let image = unsafe { (images.get_image)(assembly) };
    if !image.is_null() {
        images.items[images.len] = image;
        images.len += 1;
    }
}

struct MonoMetadataSource<'a> {
    api: MonoApi,
    images: &'a [*mut MonoImage],
}

impl MonoMetadataSource<'_> {
    fn c_string(value: &str) -> Option<CString> {
        CString::new(value).ok()
    }
}

trait MetadataSource {
    type Class: Copy;
    type Method: Copy;
    type Field: Copy;

    fn find_class(&self, namespace: &str, name: &str) -> Option<Self::Class>;
    fn find_method(
        &self,
        class: Self::Class,
        name: &str,
        parameter_count: i32,
    ) -> Option<Self::Method>;
    fn find_field(&self, class: Self::Class, name: &str) -> Option<Self::Field>;
    fn compile_method(&self, method: Self::Method) -> Option<usize>;
}

impl MetadataSource for MonoMetadataSource<'_> {
    type Class = *mut MonoClass;
    type Method = *mut MonoMethod;
    type Field = *mut MonoClassField;

    fn find_class(&self, namespace: &str, name: &str) -> Option<Self::Class> {
        let namespace = Self::c_string(namespace)?;
        let name = Self::c_string(name)?;
        self.images.iter().find_map(|image| {
            let class =
                unsafe { (self.api.class_from_name)(*image, namespace.as_ptr(), name.as_ptr()) };
            if class.is_null() {
                return None;
            }
            let actual_namespace = unsafe { (self.api.class_get_namespace)(class) };
            let actual_name = unsafe { (self.api.class_get_name)(class) };
            if actual_namespace.is_null() || actual_name.is_null() {
                return None;
            }
            let actual_namespace = unsafe { CStr::from_ptr(actual_namespace) };
            let actual_name = unsafe { CStr::from_ptr(actual_name) };
            (actual_namespace == namespace.as_c_str() && actual_name == name.as_c_str())
                .then_some(class)
        })
    }

    fn find_method(
        &self,
        class: Self::Class,
        name: &str,
        parameter_count: i32,
    ) -> Option<Self::Method> {
        let name = Self::c_string(name)?;
        let method =
            unsafe { (self.api.class_get_method_from_name)(class, name.as_ptr(), parameter_count) };
        (!method.is_null()).then_some(method)
    }

    fn find_field(&self, class: Self::Class, name: &str) -> Option<Self::Field> {
        let name = Self::c_string(name)?;
        let field = unsafe { (self.api.class_get_field_from_name)(class, name.as_ptr()) };
        (!field.is_null()).then_some(field)
    }

    fn compile_method(&self, method: Self::Method) -> Option<usize> {
        let address = unsafe { (self.api.compile_method)(method) };
        (!address.is_null()).then_some(address as usize)
    }
}

struct UiBinding<C, F> {
    kind: StandardUiKind,
    class: C,
    field: F,
    hook_address: usize,
}

struct MetadataPlan<C, M, F> {
    profile: StandardUiProfile,
    dispatch_hook_address: usize,
    enumeration_method: M,
    bindings: Vec<UiBinding<C, F>>,
}

type MetadataPlanFor<S> = MetadataPlan<
    <S as MetadataSource>::Class,
    <S as MetadataSource>::Method,
    <S as MetadataSource>::Field,
>;

fn discover_metadata<S: MetadataSource>(source: &S) -> Result<MetadataPlanFor<S>, LateAttachError> {
    let dispatch_class = source
        .find_class("UnityEngine.UI", "CanvasUpdateRegistry")
        .ok_or(LateAttachError::MainThreadDispatchUnavailable)?;
    let dispatch_method = source
        .find_method(dispatch_class, "PerformUpdate", 0)
        .ok_or(LateAttachError::MainThreadDispatchUnavailable)?;
    let dispatch_hook_address = source
        .compile_method(dispatch_method)
        .ok_or(LateAttachError::JitCompileUnavailable)?;

    let object_class = source
        .find_class("UnityEngine", "Object")
        .ok_or(LateAttachError::ObjectEnumerationUnavailable)?;
    let enumeration_method = source
        .find_method(object_class, "FindObjectsOfType", 2)
        .ok_or(LateAttachError::ObjectEnumerationUnavailable)?;

    let specs = [
        ("TMPro", "TMP_Text", "m_text", StandardUiKind::TextMeshPro),
        ("UnityEngine.UI", "Text", "m_Text", StandardUiKind::UGui),
    ];
    let mut bindings = Vec::with_capacity(specs.len());
    for (namespace, class_name, field_name, kind) in specs {
        let Some(class) = source.find_class(namespace, class_name) else {
            continue;
        };
        let Some(setter) = source.find_method(class, "set_text", 1) else {
            continue;
        };
        let Some(field) = source.find_field(class, field_name) else {
            continue;
        };
        let Some(hook_address) = source.compile_method(setter) else {
            continue;
        };
        bindings.push(UiBinding {
            kind,
            class,
            field,
            hook_address,
        });
    }
    let profile = StandardUiProfile::from_kinds(bindings.iter().map(|binding| binding.kind));
    if profile.is_empty() {
        return Err(LateAttachError::StandardUiUnavailable);
    }

    Ok(MetadataPlan {
        profile,
        dispatch_hook_address,
        enumeration_method,
        bindings,
    })
}

#[derive(Clone, Copy)]
pub(crate) struct PreparedUiBinding {
    pub(crate) kind: StandardUiKind,
    pub(crate) class: usize,
    pub(crate) field: usize,
    pub(crate) hook_address: usize,
}

pub(crate) struct PreparedRuntime {
    pub(crate) api: MonoApi,
    pub(crate) domain: usize,
    pub(crate) profile: StandardUiProfile,
    pub(crate) dispatch_hook_address: usize,
    pub(crate) enumeration_method: usize,
    pub(crate) bindings: Vec<PreparedUiBinding>,
}

pub(crate) unsafe fn prepare_runtime() -> Result<PreparedRuntime, LateAttachError> {
    let gate = MonoRuntimeGate::inspect_current_process().map_err(LateAttachError::Gate)?;
    let api = unsafe { MonoApi::from_gate(&gate)? };

    let domain = unsafe { (api.get_root_domain)() };
    if domain.is_null() {
        return Err(LateAttachError::RootDomainUnavailable);
    }
    let _thread = unsafe { api.attach_foreign_thread(domain as usize) }
        .map_err(|_| LateAttachError::ThreadAttachFailed)?;
    let images = unsafe { api.loaded_images()? };
    let source = MonoMetadataSource {
        api,
        images: images.as_slice(),
    };
    let plan = discover_metadata(&source)?;

    Ok(PreparedRuntime {
        api,
        domain: domain as usize,
        profile: plan.profile,
        dispatch_hook_address: plan.dispatch_hook_address,
        enumeration_method: plan.enumeration_method as usize,
        bindings: plan
            .bindings
            .into_iter()
            .map(|binding| PreparedUiBinding {
                kind: binding.kind,
                class: binding.class as usize,
                field: binding.field as usize,
                hook_address: binding.hook_address,
            })
            .collect(),
    })
}

/// A prepared late-attach metadata plan. This proves method availability and
/// JIT entry points but does not install detours or invoke Unity APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LateAttachReadiness {
    profile: StandardUiProfile,
    compiled_hook_count: usize,
}

impl LateAttachReadiness {
    /// Attaches the calling thread to an already initialized Mono runtime,
    /// validates the standard UI metadata contract, and resolves JIT entry
    /// points. The thread is detached before this function returns.
    ///
    /// This function does not install hooks or invoke managed methods.
    ///
    /// # Safety
    ///
    /// The current process must own the loaded Mono runtime selected by
    /// `MonoRuntimeGate`, and its exported functions must retain the public Mono
    /// embedding ABI represented by this package. The caller must be a foreign
    /// thread that has not already been attached to Mono; Unity's stripped
    /// runtime does not expose `mono_thread_current`, so this package cannot
    /// safely infer prior ownership.
    ///
    /// # Errors
    ///
    /// Returns before hook installation when the runtime, main-thread dispatch,
    /// object enumeration, standard UI metadata, or JIT entry point is missing.
    pub unsafe fn prepare_current_process() -> Result<Self, LateAttachError> {
        let prepared = unsafe { prepare_runtime()? };

        Ok(Self {
            profile: prepared.profile,
            compiled_hook_count: prepared.bindings.len() + 1,
        })
    }

    #[must_use]
    pub const fn profile(self) -> StandardUiProfile {
        self.profile
    }

    #[must_use]
    pub const fn compiled_hook_count(self) -> usize {
        self.compiled_hook_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LateAttachError {
    Gate(RuntimeGateError),
    InvalidFunctionPointer,
    RootDomainUnavailable,
    ThreadAttachFailed,
    AssemblyLimitExceeded,
    MainThreadDispatchUnavailable,
    ObjectEnumerationUnavailable,
    StandardUiUnavailable,
    JitCompileUnavailable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Default)]
    struct FakeMetadata {
        classes: BTreeMap<(&'static str, &'static str), usize>,
        methods: BTreeMap<(usize, &'static str, i32), usize>,
        fields: BTreeMap<(usize, &'static str), usize>,
        compiled: BTreeSet<usize>,
    }

    impl FakeMetadata {
        fn compatible() -> Self {
            let mut source = Self::default();
            source.add_class(1, "UnityEngine.UI", "CanvasUpdateRegistry");
            source.add_method(1, 11, "PerformUpdate", 0, true);
            source.add_class(2, "UnityEngine", "Object");
            source.add_method(2, 21, "FindObjectsOfType", 2, false);
            source.add_class(3, "TMPro", "TMP_Text");
            source.add_method(3, 31, "set_text", 1, true);
            source.add_field(3, 32, "m_text");
            source.add_class(4, "UnityEngine.UI", "Text");
            source.add_method(4, 41, "set_text", 1, true);
            source.add_field(4, 42, "m_Text");
            source
        }

        fn add_class(&mut self, class: usize, namespace: &'static str, name: &'static str) {
            self.classes.insert((namespace, name), class);
        }

        fn add_method(
            &mut self,
            class: usize,
            method: usize,
            name: &'static str,
            parameter_count: i32,
            compiled: bool,
        ) {
            self.methods.insert((class, name, parameter_count), method);
            if compiled {
                self.compiled.insert(method);
            }
        }

        fn add_field(&mut self, class: usize, field: usize, name: &'static str) {
            self.fields.insert((class, name), field);
        }
    }

    impl MetadataSource for FakeMetadata {
        type Class = usize;
        type Method = usize;
        type Field = usize;

        fn find_class(&self, namespace: &str, name: &str) -> Option<Self::Class> {
            self.classes.get(&(namespace, name)).copied()
        }

        fn find_method(
            &self,
            class: Self::Class,
            name: &str,
            parameter_count: i32,
        ) -> Option<Self::Method> {
            self.methods.get(&(class, name, parameter_count)).copied()
        }

        fn find_field(&self, class: Self::Class, name: &str) -> Option<Self::Field> {
            self.fields.get(&(class, name)).copied()
        }

        fn compile_method(&self, method: Self::Method) -> Option<usize> {
            self.compiled.contains(&method).then_some(method * 16)
        }
    }

    #[test]
    fn prepares_dispatch_and_both_standard_ui_setters() {
        let plan = discover_metadata(&FakeMetadata::compatible()).expect("metadata should pass");

        assert!(plan.profile.supports(StandardUiKind::TextMeshPro));
        assert!(plan.profile.supports(StandardUiKind::UGui));
        assert_eq!(plan.bindings.len() + 1, 3);
    }

    #[test]
    fn requires_main_thread_dispatch_and_object_enumeration_before_ui_hooks() {
        let mut source = FakeMetadata::compatible();
        source
            .classes
            .remove(&("UnityEngine.UI", "CanvasUpdateRegistry"));
        assert_eq!(
            discover_metadata(&source).err(),
            Some(LateAttachError::MainThreadDispatchUnavailable)
        );

        let mut source = FakeMetadata::compatible();
        source.classes.remove(&("UnityEngine", "Object"));
        assert_eq!(
            discover_metadata(&source).err(),
            Some(LateAttachError::ObjectEnumerationUnavailable)
        );
    }

    #[test]
    fn accepts_one_complete_standard_ui_surface_and_drops_an_incomplete_one() {
        let mut source = FakeMetadata::compatible();
        source.fields.remove(&(3, "m_text"));
        let plan = discover_metadata(&source).expect("uGUI remains usable");

        assert!(!plan.profile.supports(StandardUiKind::TextMeshPro));
        assert!(plan.profile.supports(StandardUiKind::UGui));
        assert_eq!(plan.bindings.len(), 1);
    }

    #[test]
    fn rejects_when_no_standard_ui_surface_has_a_field_setter_and_jit_entry() {
        let mut source = FakeMetadata::compatible();
        source.fields.clear();

        assert_eq!(
            discover_metadata(&source).err(),
            Some(LateAttachError::StandardUiUnavailable)
        );
    }

    #[test]
    fn rejects_an_unavailable_dispatch_jit_entry_before_installing_any_hook() {
        let mut source = FakeMetadata::compatible();
        source.compiled.remove(&11);

        assert_eq!(
            discover_metadata(&source).err(),
            Some(LateAttachError::JitCompileUnavailable)
        );
    }
}
