use crate::diagnostics::trace;
use crate::metadata::TextTarget;
use crate::objects::ObjectRegistry;
use crate::runtime_gate::{Il2CppExports, Il2CppWritebackExports};
use glyphshift_adapter_unity_standard_ui::{StandardUiKind, TextWrite};
use std::ffi::{c_char, c_void};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WritebackError {
    WritebackUnavailable,
    ThreadUnavailable,
    MissingSetter,
    InvalidStringLength,
    StringAllocationFailed,
    StringHandleFailed,
    InvokeException,
}

trait WritebackRuntime {
    fn thread_ready(&self) -> bool;
    fn object_target(&self, write: &TextWrite) -> Option<usize>;
    fn invoke_text(
        &mut self,
        kind: StandardUiKind,
        object: usize,
        units: &[u16],
    ) -> Result<(), WritebackError>;
}

pub(crate) unsafe fn apply_writes(
    exports: Il2CppExports,
    targets: &[TextTarget],
    objects: &ObjectRegistry,
    writes: &[TextWrite],
) -> Result<(), WritebackError> {
    let writeback = exports
        .writeback
        .ok_or(WritebackError::WritebackUnavailable)?;
    let mut runtime = NativeWritebackRuntime {
        exports,
        writeback,
        targets,
        objects,
    };
    apply(&mut runtime, writes)
}

fn apply(runtime: &mut impl WritebackRuntime, writes: &[TextWrite]) -> Result<(), WritebackError> {
    if !runtime.thread_ready() {
        return Err(WritebackError::ThreadUnavailable);
    }
    for write in writes {
        let Some(object) = runtime.object_target(write) else {
            // Weak handles deliberately make collection fail open.
            continue;
        };
        runtime.invoke_text(write.kind(), object, write.units())?;
    }
    Ok(())
}

struct NativeWritebackRuntime<'a> {
    exports: Il2CppExports,
    writeback: Il2CppWritebackExports,
    targets: &'a [TextTarget],
    objects: &'a ObjectRegistry,
}

impl NativeWritebackRuntime<'_> {
    fn target(&self, kind: StandardUiKind) -> Option<TextTarget> {
        self.targets
            .iter()
            .copied()
            .find(|target| target.kind == kind)
    }
}

impl WritebackRuntime for NativeWritebackRuntime<'_> {
    fn thread_ready(&self) -> bool {
        !unsafe { (self.exports.thread_current)() }.is_null()
    }

    fn object_target(&self, write: &TextWrite) -> Option<usize> {
        unsafe { self.objects.target(self.writeback, write.object_id()) }
            .map(|target| target as usize)
    }

    fn invoke_text(
        &mut self,
        kind: StandardUiKind,
        object: usize,
        units: &[u16],
    ) -> Result<(), WritebackError> {
        let target = self.target(kind).ok_or(WritebackError::MissingSetter)?;
        let setter = target.text_setter.ok_or(WritebackError::MissingSetter)?;
        let length = i32::try_from(units.len()).map_err(|_| WritebackError::InvalidStringLength)?;
        let string = unsafe { (self.writeback.string_new_utf16)(units.as_ptr(), length) };
        if string.is_null() {
            return Err(WritebackError::StringAllocationFailed);
        }
        let handle = unsafe { (self.writeback.gchandle_new)(string, false) };
        if handle == 0 {
            return Err(WritebackError::StringHandleFailed);
        }
        let managed = ManagedStringHandle {
            exports: self.writeback,
            handle,
        };
        let string = unsafe { (self.writeback.gchandle_get_target)(managed.handle) };
        if string.is_null() {
            return Err(WritebackError::StringHandleFailed);
        }

        let mut parameters = [string];
        let mut exception = std::ptr::null_mut::<c_void>();
        unsafe {
            (self.writeback.runtime_invoke)(
                setter as *const c_void,
                object as *mut c_void,
                parameters.as_mut_ptr(),
                &mut exception,
            )
        };
        if !exception.is_null() {
            trace_formatted_exception(self.writeback, exception);
            return Err(WritebackError::InvokeException);
        }
        Ok(())
    }
}

struct ManagedStringHandle {
    exports: Il2CppWritebackExports,
    handle: usize,
}

impl Drop for ManagedStringHandle {
    fn drop(&mut self) {
        unsafe { (self.exports.gchandle_free)(self.handle) };
    }
}

fn trace_formatted_exception(exports: Il2CppWritebackExports, exception: *mut c_void) {
    let mut buffer = [0 as c_char; 256];
    unsafe {
        (exports.format_exception)(
            exception.cast_const(),
            buffer.as_mut_ptr(),
            buffer.len() as i32,
        )
    };
    if buffer[0] == 0 {
        trace("writeback.invoke.exception.unformatted");
    } else {
        // The raw managed exception can contain target-specific text or local
        // paths. Diagnostics record only that public formatting succeeded.
        trace("writeback.invoke.exception.formatted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glyphshift_adapter_unity_standard_ui::ManagedObjectId;
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct FakeRuntime {
        ready: bool,
        objects: BTreeMap<u64, usize>,
        invoked: Vec<(StandardUiKind, usize, Vec<u16>)>,
        fail_kind: Option<StandardUiKind>,
    }

    impl WritebackRuntime for FakeRuntime {
        fn thread_ready(&self) -> bool {
            self.ready
        }

        fn object_target(&self, write: &TextWrite) -> Option<usize> {
            self.objects.get(&write.object_id().value()).copied()
        }

        fn invoke_text(
            &mut self,
            kind: StandardUiKind,
            object: usize,
            units: &[u16],
        ) -> Result<(), WritebackError> {
            if self.fail_kind == Some(kind) {
                return Err(WritebackError::InvokeException);
            }
            self.invoked.push((kind, object, units.to_vec()));
            Ok(())
        }
    }

    fn write(id: u64, kind: StandardUiKind, value: &str) -> TextWrite {
        let mut state = glyphshift_adapter_unity_standard_ui::UnityStandardUiWriteback::default();
        state
            .activate(
                vec![glyphshift_adapter_unity_standard_ui::ManagedText::text(
                    ManagedObjectId::new(id),
                    kind,
                    "source",
                )],
                |_| glyphshift_adapter_unity_standard_ui::TextDecision::replace_text(1, value),
            )
            .remove(0)
    }

    #[test]
    fn applies_all_live_writes_and_skips_collected_objects() {
        let mut runtime = FakeRuntime {
            ready: true,
            objects: [(1, 101)].into_iter().collect(),
            ..FakeRuntime::default()
        };
        let writes = [
            write(1, StandardUiKind::TextMeshPro, "你好"),
            write(2, StandardUiKind::UGui, "打开"),
        ];

        apply(&mut runtime, &writes).expect("live write");

        assert_eq!(runtime.invoked.len(), 1);
        assert_eq!(runtime.invoked[0].0, StandardUiKind::TextMeshPro);
        assert_eq!(String::from_utf16(&runtime.invoked[0].2).unwrap(), "你好");
    }

    #[test]
    fn fails_closed_before_mutation_when_thread_is_not_ready() {
        let mut runtime = FakeRuntime {
            ready: false,
            objects: [(1, 101)].into_iter().collect(),
            ..FakeRuntime::default()
        };
        let result = apply(
            &mut runtime,
            &[write(1, StandardUiKind::TextMeshPro, "你好")],
        );
        assert_eq!(result, Err(WritebackError::ThreadUnavailable));
        assert!(runtime.invoked.is_empty());
    }

    #[test]
    fn propagates_managed_invoke_failure_without_continuing() {
        let mut runtime = FakeRuntime {
            ready: true,
            objects: [(1, 101), (2, 102)].into_iter().collect(),
            fail_kind: Some(StandardUiKind::TextMeshPro),
            ..FakeRuntime::default()
        };
        let result = apply(
            &mut runtime,
            &[
                write(1, StandardUiKind::TextMeshPro, "你好"),
                write(2, StandardUiKind::UGui, "打开"),
            ],
        );
        assert_eq!(result, Err(WritebackError::InvokeException));
        assert!(runtime.invoked.is_empty());
    }
}
