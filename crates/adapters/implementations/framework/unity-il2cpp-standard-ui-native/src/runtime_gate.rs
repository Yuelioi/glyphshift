use std::ffi::{c_char, c_void};

const IL2CPP_MODULE: &str = "GameAssembly.dll";
const MONO_MODULE: &str = "mono-2.0-bdwgc.dll";

#[derive(Clone, Copy)]
struct RequiredExport {
    name: &'static str,
    nul_terminated: &'static [u8],
}

const OBSERVE_EXPORTS: [RequiredExport; 17] = [
    required_export("il2cpp_domain_get", b"il2cpp_domain_get\0"),
    required_export(
        "il2cpp_domain_get_assemblies",
        b"il2cpp_domain_get_assemblies\0",
    ),
    required_export("il2cpp_assembly_get_image", b"il2cpp_assembly_get_image\0"),
    required_export("il2cpp_class_from_name", b"il2cpp_class_from_name\0"),
    required_export(
        "il2cpp_class_get_field_from_name",
        b"il2cpp_class_get_field_from_name\0",
    ),
    required_export("il2cpp_field_get_value", b"il2cpp_field_get_value\0"),
    required_export(
        "il2cpp_unity_liveness_allocate_struct",
        b"il2cpp_unity_liveness_allocate_struct\0",
    ),
    required_export(
        "il2cpp_unity_liveness_calculation_from_statics",
        b"il2cpp_unity_liveness_calculation_from_statics\0",
    ),
    required_export(
        "il2cpp_unity_liveness_finalize",
        b"il2cpp_unity_liveness_finalize\0",
    ),
    required_export(
        "il2cpp_unity_liveness_free_struct",
        b"il2cpp_unity_liveness_free_struct\0",
    ),
    required_export("il2cpp_stop_gc_world", b"il2cpp_stop_gc_world\0"),
    required_export("il2cpp_start_gc_world", b"il2cpp_start_gc_world\0"),
    required_export("il2cpp_string_chars", b"il2cpp_string_chars\0"),
    required_export("il2cpp_string_length", b"il2cpp_string_length\0"),
    required_export("il2cpp_thread_current", b"il2cpp_thread_current\0"),
    required_export("il2cpp_thread_attach", b"il2cpp_thread_attach\0"),
    required_export("il2cpp_thread_detach", b"il2cpp_thread_detach\0"),
];

const WRITEBACK_EXPORTS: [RequiredExport; 8] = [
    required_export(
        "il2cpp_class_get_method_from_name",
        b"il2cpp_class_get_method_from_name\0",
    ),
    required_export("il2cpp_runtime_invoke", b"il2cpp_runtime_invoke\0"),
    required_export("il2cpp_string_new_utf16", b"il2cpp_string_new_utf16\0"),
    required_export("il2cpp_gchandle_new", b"il2cpp_gchandle_new\0"),
    required_export(
        "il2cpp_gchandle_new_weakref",
        b"il2cpp_gchandle_new_weakref\0",
    ),
    required_export(
        "il2cpp_gchandle_get_target",
        b"il2cpp_gchandle_get_target\0",
    ),
    required_export("il2cpp_gchandle_free", b"il2cpp_gchandle_free\0"),
    required_export("il2cpp_format_exception", b"il2cpp_format_exception\0"),
];

const fn required_export(name: &'static str, nul_terminated: &'static [u8]) -> RequiredExport {
    RequiredExport {
        name,
        nul_terminated,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostPlatform {
    WindowsX64,
    #[cfg(any(test, all(windows, not(target_arch = "x86_64"))))]
    WindowsOther,
    #[cfg(any(test, not(windows)))]
    Other,
}

trait RuntimeSource {
    type Module: Copy;

    fn platform(&self) -> HostPlatform;
    fn loaded_module(&self, name: &'static str) -> Option<Self::Module>;
    fn export_address(
        &self,
        module: Self::Module,
        nul_terminated_name: &'static [u8],
    ) -> Option<usize>;
}

/// Resolved public IL2CPP export contract for the current process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Il2CppRuntimeGate {
    observe_export_addresses: [usize; OBSERVE_EXPORTS.len()],
    writeback_export_addresses: Option<[usize; WRITEBACK_EXPORTS.len()]>,
}

impl Il2CppRuntimeGate {
    /// Inspects only modules that are already loaded in the current process.
    pub fn inspect_current_process(require_writeback: bool) -> Result<Self, RuntimeGateError> {
        inspect_with(&CurrentProcessRuntime, require_writeback)
    }

    #[must_use]
    pub const fn resolved_export_count(&self) -> usize {
        OBSERVE_EXPORTS.len()
            + if self.writeback_export_addresses.is_some() {
                WRITEBACK_EXPORTS.len()
            } else {
                0
            }
    }

    fn address(&self, name: &str) -> usize {
        if let Some(index) = OBSERVE_EXPORTS
            .iter()
            .position(|required| required.name == name)
        {
            return self.observe_export_addresses[index];
        }
        let index = WRITEBACK_EXPORTS
            .iter()
            .position(|required| required.name == name)
            .expect("IL2CPP export is part of the validated gate");
        self.writeback_export_addresses
            .expect("writeback exports were validated before use")[index]
    }

    pub(crate) fn exports(self) -> Il2CppExports {
        macro_rules! resolve {
            ($name:literal, $kind:ty) => {
                std::mem::transmute::<usize, $kind>(self.address($name))
            };
        }
        unsafe {
            let writeback = self
                .writeback_export_addresses
                .map(|_| Il2CppWritebackExports {
                    class_get_method_from_name: resolve!(
                        "il2cpp_class_get_method_from_name",
                        ClassGetMethodFromName
                    ),
                    runtime_invoke: resolve!("il2cpp_runtime_invoke", RuntimeInvoke),
                    string_new_utf16: resolve!("il2cpp_string_new_utf16", StringNewUtf16),
                    gchandle_new: resolve!("il2cpp_gchandle_new", GcHandleNew),
                    gchandle_new_weakref: resolve!(
                        "il2cpp_gchandle_new_weakref",
                        GcHandleNewWeakRef
                    ),
                    gchandle_get_target: resolve!("il2cpp_gchandle_get_target", GcHandleGetTarget),
                    gchandle_free: resolve!("il2cpp_gchandle_free", GcHandleFree),
                    format_exception: resolve!("il2cpp_format_exception", FormatException),
                });
            Il2CppExports {
                domain_get: resolve!("il2cpp_domain_get", DomainGet),
                domain_get_assemblies: resolve!(
                    "il2cpp_domain_get_assemblies",
                    DomainGetAssemblies
                ),
                assembly_get_image: resolve!("il2cpp_assembly_get_image", AssemblyGetImage),
                class_from_name: resolve!("il2cpp_class_from_name", ClassFromName),
                class_get_field_from_name: resolve!(
                    "il2cpp_class_get_field_from_name",
                    ClassGetFieldFromName
                ),
                field_get_value: resolve!("il2cpp_field_get_value", FieldGetValue),
                liveness_allocate: resolve!(
                    "il2cpp_unity_liveness_allocate_struct",
                    LivenessAllocate
                ),
                liveness_from_statics: resolve!(
                    "il2cpp_unity_liveness_calculation_from_statics",
                    LivenessFromStatics
                ),
                liveness_finalize: resolve!("il2cpp_unity_liveness_finalize", LivenessFinalize),
                liveness_free: resolve!("il2cpp_unity_liveness_free_struct", LivenessFree),
                stop_gc_world: resolve!("il2cpp_stop_gc_world", StopGcWorld),
                start_gc_world: resolve!("il2cpp_start_gc_world", StartGcWorld),
                string_chars: resolve!("il2cpp_string_chars", StringChars),
                string_length: resolve!("il2cpp_string_length", StringLength),
                thread_current: resolve!("il2cpp_thread_current", ThreadCurrent),
                thread_attach: resolve!("il2cpp_thread_attach", ThreadAttach),
                thread_detach: resolve!("il2cpp_thread_detach", ThreadDetach),
                writeback,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeGateError {
    UnsupportedPlatform,
    UnsupportedArchitecture,
    MonoBackend,
    Il2CppRuntimeUnavailable,
    MissingExport(&'static str),
}

fn inspect_with(
    source: &impl RuntimeSource,
    require_writeback: bool,
) -> Result<Il2CppRuntimeGate, RuntimeGateError> {
    match source.platform() {
        HostPlatform::WindowsX64 => {}
        #[cfg(any(test, all(windows, not(target_arch = "x86_64"))))]
        HostPlatform::WindowsOther => return Err(RuntimeGateError::UnsupportedArchitecture),
        #[cfg(any(test, not(windows)))]
        HostPlatform::Other => return Err(RuntimeGateError::UnsupportedPlatform),
    }
    if source.loaded_module(MONO_MODULE).is_some() {
        return Err(RuntimeGateError::MonoBackend);
    }
    let module = source
        .loaded_module(IL2CPP_MODULE)
        .ok_or(RuntimeGateError::Il2CppRuntimeUnavailable)?;

    let mut observe_export_addresses = [0; OBSERVE_EXPORTS.len()];
    for (index, required) in OBSERVE_EXPORTS.iter().enumerate() {
        observe_export_addresses[index] = source
            .export_address(module, required.nul_terminated)
            .ok_or(RuntimeGateError::MissingExport(required.name))?;
    }
    let writeback_export_addresses = if require_writeback {
        let mut addresses = [0; WRITEBACK_EXPORTS.len()];
        for (index, required) in WRITEBACK_EXPORTS.iter().enumerate() {
            addresses[index] = source
                .export_address(module, required.nul_terminated)
                .ok_or(RuntimeGateError::MissingExport(required.name))?;
        }
        Some(addresses)
    } else {
        None
    };
    Ok(Il2CppRuntimeGate {
        observe_export_addresses,
        writeback_export_addresses,
    })
}

struct CurrentProcessRuntime;

impl RuntimeSource for CurrentProcessRuntime {
    type Module = CurrentModule;

    fn platform(&self) -> HostPlatform {
        #[cfg(all(windows, target_arch = "x86_64"))]
        {
            HostPlatform::WindowsX64
        }
        #[cfg(all(windows, not(target_arch = "x86_64")))]
        {
            HostPlatform::WindowsOther
        }
        #[cfg(not(windows))]
        {
            HostPlatform::Other
        }
    }

    fn loaded_module(&self, name: &'static str) -> Option<Self::Module> {
        current_module(name)
    }

    fn export_address(
        &self,
        module: Self::Module,
        nul_terminated_name: &'static [u8],
    ) -> Option<usize> {
        current_export_address(module, nul_terminated_name)
    }
}

#[cfg(windows)]
#[derive(Clone, Copy)]
struct CurrentModule(windows::Win32::Foundation::HMODULE);

#[cfg(not(windows))]
#[derive(Clone, Copy)]
struct CurrentModule;

#[cfg(windows)]
fn current_module(name: &str) -> Option<CurrentModule> {
    use windows::core::PCWSTR;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;

    let wide = name.encode_utf16().chain([0]).collect::<Vec<_>>();
    unsafe {
        GetModuleHandleW(PCWSTR(wide.as_ptr()))
            .ok()
            .map(CurrentModule)
    }
}

#[cfg(not(windows))]
fn current_module(_name: &str) -> Option<CurrentModule> {
    None
}

#[cfg(windows)]
fn current_export_address(module: CurrentModule, name: &[u8]) -> Option<usize> {
    use windows::core::PCSTR;
    use windows::Win32::System::LibraryLoader::GetProcAddress;

    unsafe { GetProcAddress(module.0, PCSTR(name.as_ptr())).map(|address| address as usize) }
}

#[cfg(not(windows))]
fn current_export_address(_module: CurrentModule, _name: &[u8]) -> Option<usize> {
    None
}

pub(crate) type DomainGet = unsafe extern "C" fn() -> *mut c_void;
pub(crate) type DomainGetAssemblies =
    unsafe extern "C" fn(*mut c_void, *mut usize) -> *mut *const c_void;
pub(crate) type AssemblyGetImage = unsafe extern "C" fn(*const c_void) -> *const c_void;
pub(crate) type ClassFromName =
    unsafe extern "C" fn(*const c_void, *const c_char, *const c_char) -> *mut c_void;
pub(crate) type ClassGetFieldFromName =
    unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;
pub(crate) type ClassGetMethodFromName =
    unsafe extern "C" fn(*mut c_void, *const c_char, i32) -> *const c_void;
pub(crate) type FieldGetValue = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void);
pub(crate) type RuntimeInvoke = unsafe extern "C" fn(
    *const c_void,
    *mut c_void,
    *mut *mut c_void,
    *mut *mut c_void,
) -> *mut c_void;
pub(crate) type RegisterObjects = unsafe extern "C" fn(*mut *mut c_void, i32, *mut c_void);
pub(crate) type Reallocate = unsafe extern "C" fn(*mut c_void, usize, *mut c_void) -> *mut c_void;
pub(crate) type LivenessAllocate = unsafe extern "C" fn(
    *mut c_void,
    i32,
    Option<RegisterObjects>,
    *mut c_void,
    Option<Reallocate>,
) -> *mut c_void;
pub(crate) type LivenessFromStatics = unsafe extern "C" fn(*mut c_void);
pub(crate) type LivenessFinalize = unsafe extern "C" fn(*mut c_void);
pub(crate) type LivenessFree = unsafe extern "C" fn(*mut c_void);
pub(crate) type StopGcWorld = unsafe extern "C" fn();
pub(crate) type StartGcWorld = unsafe extern "C" fn();
pub(crate) type StringChars = unsafe extern "C" fn(*mut c_void) -> *const u16;
pub(crate) type StringLength = unsafe extern "C" fn(*mut c_void) -> i32;
pub(crate) type StringNewUtf16 = unsafe extern "C" fn(*const u16, i32) -> *mut c_void;
// `Il2CppGCHandle` changed from an integer handle in older Unity releases to a
// pointer-sized handle in newer IL2CPP. This adapter is Windows x64-only, where
// `usize` is ABI-compatible with both forms: a 32-bit return is zero-extended
// in RAX and an older callee consuming the handle reads the low 32 bits.
pub(crate) type GcHandle = usize;
pub(crate) type GcHandleNew = unsafe extern "C" fn(*mut c_void, bool) -> GcHandle;
pub(crate) type GcHandleNewWeakRef = unsafe extern "C" fn(*mut c_void, bool) -> GcHandle;
pub(crate) type GcHandleGetTarget = unsafe extern "C" fn(GcHandle) -> *mut c_void;
pub(crate) type GcHandleFree = unsafe extern "C" fn(GcHandle);
pub(crate) type FormatException = unsafe extern "C" fn(*const c_void, *mut c_char, i32);
pub(crate) type ThreadCurrent = unsafe extern "C" fn() -> *mut c_void;
pub(crate) type ThreadAttach = unsafe extern "C" fn(*mut c_void) -> *mut c_void;
pub(crate) type ThreadDetach = unsafe extern "C" fn(*mut c_void);

#[derive(Clone, Copy)]
pub(crate) struct Il2CppExports {
    pub domain_get: DomainGet,
    pub domain_get_assemblies: DomainGetAssemblies,
    pub assembly_get_image: AssemblyGetImage,
    pub class_from_name: ClassFromName,
    pub class_get_field_from_name: ClassGetFieldFromName,
    pub field_get_value: FieldGetValue,
    pub liveness_allocate: LivenessAllocate,
    pub liveness_from_statics: LivenessFromStatics,
    pub liveness_finalize: LivenessFinalize,
    pub liveness_free: LivenessFree,
    pub stop_gc_world: StopGcWorld,
    pub start_gc_world: StartGcWorld,
    pub string_chars: StringChars,
    pub string_length: StringLength,
    pub thread_current: ThreadCurrent,
    pub thread_attach: ThreadAttach,
    pub thread_detach: ThreadDetach,
    pub writeback: Option<Il2CppWritebackExports>,
}

#[derive(Clone, Copy)]
pub(crate) struct Il2CppWritebackExports {
    pub class_get_method_from_name: ClassGetMethodFromName,
    pub runtime_invoke: RuntimeInvoke,
    pub string_new_utf16: StringNewUtf16,
    pub gchandle_new: GcHandleNew,
    pub gchandle_new_weakref: GcHandleNewWeakRef,
    pub gchandle_get_target: GcHandleGetTarget,
    pub gchandle_free: GcHandleFree,
    pub format_exception: FormatException,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[derive(Clone, Copy)]
    struct FakeModule;

    struct FakeRuntime {
        platform: HostPlatform,
        mono: bool,
        il2cpp: bool,
        exports: BTreeSet<&'static [u8]>,
    }

    impl FakeRuntime {
        fn compatible() -> Self {
            Self {
                platform: HostPlatform::WindowsX64,
                mono: false,
                il2cpp: true,
                exports: OBSERVE_EXPORTS
                    .iter()
                    .chain(WRITEBACK_EXPORTS.iter())
                    .map(|required| required.nul_terminated)
                    .collect(),
            }
        }
    }

    impl RuntimeSource for FakeRuntime {
        type Module = FakeModule;

        fn platform(&self) -> HostPlatform {
            self.platform
        }

        fn loaded_module(&self, name: &'static str) -> Option<Self::Module> {
            match name {
                IL2CPP_MODULE if self.il2cpp => Some(FakeModule),
                MONO_MODULE if self.mono => Some(FakeModule),
                _ => None,
            }
        }

        fn export_address(&self, _module: Self::Module, name: &'static [u8]) -> Option<usize> {
            self.exports.contains(name).then_some(1)
        }
    }

    #[test]
    fn accepts_complete_loaded_il2cpp_runtime() {
        let runtime = FakeRuntime::compatible();
        let observe = inspect_with(&runtime, false).expect("observe gate");
        assert_eq!(observe.resolved_export_count(), OBSERVE_EXPORTS.len());
        let writeback = inspect_with(&runtime, true).expect("writeback gate");
        assert_eq!(
            writeback.resolved_export_count(),
            OBSERVE_EXPORTS.len() + WRITEBACK_EXPORTS.len()
        );
    }

    #[test]
    fn observe_only_does_not_require_writeback_exports() {
        let mut runtime = FakeRuntime::compatible();
        for required in WRITEBACK_EXPORTS {
            runtime.exports.remove(required.nul_terminated);
        }
        assert_eq!(
            inspect_with(&runtime, false)
                .expect("observe-only gate")
                .resolved_export_count(),
            OBSERVE_EXPORTS.len()
        );
        assert_eq!(
            inspect_with(&runtime, true),
            Err(RuntimeGateError::MissingExport(WRITEBACK_EXPORTS[0].name))
        );
    }

    #[test]
    fn rejects_mono_before_considering_il2cpp() {
        let runtime = FakeRuntime {
            mono: true,
            ..FakeRuntime::compatible()
        };
        assert_eq!(
            inspect_with(&runtime, false),
            Err(RuntimeGateError::MonoBackend)
        );
    }

    #[test]
    fn rejects_missing_runtime_and_export_with_stable_reasons() {
        let runtime = FakeRuntime {
            il2cpp: false,
            ..FakeRuntime::compatible()
        };
        assert_eq!(
            inspect_with(&runtime, false),
            Err(RuntimeGateError::Il2CppRuntimeUnavailable)
        );

        for required in OBSERVE_EXPORTS {
            let mut runtime = FakeRuntime::compatible();
            runtime.exports.remove(required.nul_terminated);
            assert_eq!(
                inspect_with(&runtime, false),
                Err(RuntimeGateError::MissingExport(required.name))
            );
        }
        for required in WRITEBACK_EXPORTS {
            let mut runtime = FakeRuntime::compatible();
            runtime.exports.remove(required.nul_terminated);
            assert_eq!(
                inspect_with(&runtime, true),
                Err(RuntimeGateError::MissingExport(required.name))
            );
        }
    }

    #[test]
    fn rejects_other_architecture_and_platform() {
        let runtime = FakeRuntime {
            platform: HostPlatform::WindowsOther,
            ..FakeRuntime::compatible()
        };
        assert_eq!(
            inspect_with(&runtime, false),
            Err(RuntimeGateError::UnsupportedArchitecture)
        );

        let runtime = FakeRuntime {
            platform: HostPlatform::Other,
            ..FakeRuntime::compatible()
        };
        assert_eq!(
            inspect_with(&runtime, false),
            Err(RuntimeGateError::UnsupportedPlatform)
        );
    }
}
