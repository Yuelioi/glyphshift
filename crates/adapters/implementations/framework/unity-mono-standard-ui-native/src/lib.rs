//! Windows x86/x64 Unity Mono standard-UI observation and retained-text replacement.
//! Activation requires the public Mono exports, recognized standard UI metadata,
//! a live object snapshot, and main-thread dispatch; IL2CPP is a different backend.

mod late_attach;
mod native_adapter;

pub use glyphshift_adapter_unity_standard_ui::{
    ObserverDriver, ObserverDriverError, StandardUiObservationRuntime, StandardUiProfile,
};
pub use late_attach::{LateAttachError, LateAttachReadiness};

const MONO_MODULE: &str = "mono-2.0-bdwgc.dll";
const IL2CPP_MODULE: &str = "GameAssembly.dll";

#[derive(Clone, Copy)]
struct RequiredExport {
    name: &'static str,
    nul_terminated: &'static [u8],
}

const REQUIRED_EXPORTS: [RequiredExport; 26] = [
    required_export("mono_get_root_domain", b"mono_get_root_domain\0"),
    required_export("mono_thread_attach", b"mono_thread_attach\0"),
    required_export("mono_thread_detach", b"mono_thread_detach\0"),
    required_export("mono_assembly_foreach", b"mono_assembly_foreach\0"),
    required_export("mono_assembly_get_image", b"mono_assembly_get_image\0"),
    required_export("mono_class_from_name_case", b"mono_class_from_name_case\0"),
    required_export("mono_class_get_name", b"mono_class_get_name\0"),
    required_export("mono_class_get_namespace", b"mono_class_get_namespace\0"),
    required_export(
        "mono_class_is_assignable_from",
        b"mono_class_is_assignable_from\0",
    ),
    required_export(
        "mono_class_get_method_from_name",
        b"mono_class_get_method_from_name\0",
    ),
    required_export("mono_compile_method", b"mono_compile_method\0"),
    required_export("mono_runtime_invoke", b"mono_runtime_invoke\0"),
    required_export("mono_class_get_type", b"mono_class_get_type\0"),
    required_export("mono_type_get_object", b"mono_type_get_object\0"),
    required_export("mono_array_length", b"mono_array_length\0"),
    required_export("mono_array_addr_with_size", b"mono_array_addr_with_size\0"),
    required_export(
        "mono_class_get_field_from_name",
        b"mono_class_get_field_from_name\0",
    ),
    required_export("mono_field_get_value", b"mono_field_get_value\0"),
    required_export("mono_object_get_class", b"mono_object_get_class\0"),
    required_export("mono_string_chars", b"mono_string_chars\0"),
    required_export("mono_string_length", b"mono_string_length\0"),
    required_export("mono_string_new_utf16", b"mono_string_new_utf16\0"),
    required_export("mono_gchandle_new_v2", b"mono_gchandle_new_v2\0"),
    required_export(
        "mono_gchandle_new_weakref_v2",
        b"mono_gchandle_new_weakref_v2\0",
    ),
    required_export(
        "mono_gchandle_get_target_v2",
        b"mono_gchandle_get_target_v2\0",
    ),
    required_export("mono_gchandle_free_v2", b"mono_gchandle_free_v2\0"),
];

const fn required_export(name: &'static str, nul_terminated: &'static [u8]) -> RequiredExport {
    RequiredExport {
        name,
        nul_terminated,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostPlatform {
    WindowsX86Family,
    #[cfg(any(
        test,
        all(windows, not(any(target_arch = "x86", target_arch = "x86_64")))
    ))]
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

/// A validated, observe-only Unity Mono runtime contract.
///
/// Resolving this value does not attach a managed thread, register a profiler,
/// walk the heap, invoke managed code, or install hooks.
#[derive(Debug)]
pub struct MonoRuntimeGate {
    export_addresses: [usize; REQUIRED_EXPORTS.len()],
}

impl MonoRuntimeGate {
    /// Inspects the current process without loading modules or changing runtime
    /// state.
    ///
    /// # Errors
    ///
    /// Returns a stable rejection when the host platform, managed backend, or
    /// required Mono export contract is unsupported.
    pub fn inspect_current_process() -> Result<Self, RuntimeGateError> {
        inspect_with(&CurrentProcessRuntime)
    }

    #[must_use]
    pub const fn resolved_export_count(&self) -> usize {
        self.export_addresses.len()
    }

    fn export_address(&self, name: &str) -> Option<usize> {
        REQUIRED_EXPORTS
            .iter()
            .position(|required| required.name == name)
            .map(|index| self.export_addresses[index])
    }
}

/// Stable reasons why the Unity Mono observer must not activate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeGateError {
    UnsupportedPlatform,
    UnsupportedArchitecture,
    Il2CppBackend,
    MonoRuntimeUnavailable,
    MissingExport(&'static str),
}

fn inspect_with(source: &impl RuntimeSource) -> Result<MonoRuntimeGate, RuntimeGateError> {
    match source.platform() {
        HostPlatform::WindowsX86Family => {}
        #[cfg(any(
            test,
            all(windows, not(any(target_arch = "x86", target_arch = "x86_64")))
        ))]
        HostPlatform::WindowsOther => return Err(RuntimeGateError::UnsupportedArchitecture),
        #[cfg(any(test, not(windows)))]
        HostPlatform::Other => return Err(RuntimeGateError::UnsupportedPlatform),
    }

    let Some(module) = source.loaded_module(MONO_MODULE) else {
        return if source.loaded_module(IL2CPP_MODULE).is_some() {
            Err(RuntimeGateError::Il2CppBackend)
        } else {
            Err(RuntimeGateError::MonoRuntimeUnavailable)
        };
    };

    let mut export_addresses = [0; REQUIRED_EXPORTS.len()];
    for (index, required) in REQUIRED_EXPORTS.iter().enumerate() {
        export_addresses[index] = source
            .export_address(module, required.nul_terminated)
            .ok_or(RuntimeGateError::MissingExport(required.name))?;
    }

    Ok(MonoRuntimeGate { export_addresses })
}

struct CurrentProcessRuntime;

impl RuntimeSource for CurrentProcessRuntime {
    type Module = CurrentModule;

    fn platform(&self) -> HostPlatform {
        #[cfg(all(windows, any(target_arch = "x86", target_arch = "x86_64")))]
        {
            HostPlatform::WindowsX86Family
        }
        #[cfg(all(windows, not(any(target_arch = "x86", target_arch = "x86_64"))))]
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
fn current_export_address(module: CurrentModule, nul_terminated_name: &[u8]) -> Option<usize> {
    use windows::core::PCSTR;
    use windows::Win32::System::LibraryLoader::GetProcAddress;

    unsafe {
        GetProcAddress(module.0, PCSTR(nul_terminated_name.as_ptr()))
            .map(|address| address as usize)
    }
}

#[cfg(not(windows))]
fn current_export_address(_module: CurrentModule, _nul_terminated_name: &[u8]) -> Option<usize> {
    None
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
                platform: HostPlatform::WindowsX86Family,
                mono: true,
                il2cpp: false,
                exports: REQUIRED_EXPORTS
                    .iter()
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
                MONO_MODULE if self.mono => Some(FakeModule),
                IL2CPP_MODULE if self.il2cpp => Some(FakeModule),
                _ => None,
            }
        }

        fn export_address(
            &self,
            _module: Self::Module,
            nul_terminated_name: &'static [u8],
        ) -> Option<usize> {
            self.exports
                .contains(nul_terminated_name)
                .then_some(nul_terminated_name.as_ptr() as usize)
        }
    }

    #[test]
    fn accepts_windows_x86_family_mono_with_the_complete_common_contract() {
        let gate = inspect_with(&FakeRuntime::compatible()).expect("runtime should pass");

        assert_eq!(gate.resolved_export_count(), REQUIRED_EXPORTS.len());
    }

    #[test]
    fn rejects_unsupported_platform_and_architecture_before_module_lookup() {
        let mut runtime = FakeRuntime::compatible();
        runtime.platform = HostPlatform::Other;
        assert_eq!(
            inspect_with(&runtime).unwrap_err(),
            RuntimeGateError::UnsupportedPlatform
        );

        runtime.platform = HostPlatform::WindowsOther;
        assert_eq!(
            inspect_with(&runtime).unwrap_err(),
            RuntimeGateError::UnsupportedArchitecture
        );
    }

    #[test]
    fn distinguishes_il2cpp_from_a_missing_managed_runtime() {
        let mut runtime = FakeRuntime::compatible();
        runtime.mono = false;
        runtime.il2cpp = true;
        assert_eq!(
            inspect_with(&runtime).unwrap_err(),
            RuntimeGateError::Il2CppBackend
        );

        runtime.il2cpp = false;
        assert_eq!(
            inspect_with(&runtime).unwrap_err(),
            RuntimeGateError::MonoRuntimeUnavailable
        );
    }

    #[test]
    fn rejects_each_missing_export_with_the_exact_contract_name() {
        for required in REQUIRED_EXPORTS {
            let mut runtime = FakeRuntime::compatible();
            runtime.exports.remove(required.nul_terminated);

            assert_eq!(
                inspect_with(&runtime).unwrap_err(),
                RuntimeGateError::MissingExport(required.name)
            );
        }
    }
}
