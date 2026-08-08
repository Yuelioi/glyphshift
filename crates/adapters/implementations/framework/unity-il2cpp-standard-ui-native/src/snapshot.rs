use crate::diagnostics::trace;
use crate::metadata::{MetadataError, TextTarget};
use crate::runtime_gate::{Il2CppExports, StartGcWorld};
use glyphshift_adapter_unity_standard_ui::{
    ManagedObjectId, ManagedText, StandardUiKind, MAX_TEXT_UNITS,
};
use std::ffi::c_void;

const MAX_SNAPSHOT_OBJECTS: usize = 4 * 1024;
const MAX_SNAPSHOT_UTF16_UNITS: usize = 1024 * 1024;
#[cfg(not(windows))]
const MAX_LIVENESS_ALLOCATIONS: usize = 8;

pub(super) unsafe fn capture_snapshot(
    exports: Il2CppExports,
    targets: &[TextTarget],
) -> Result<Vec<ManagedText>, MetadataError> {
    let domain = (exports.domain_get)();
    if domain.is_null() {
        return Err(MetadataError::RuntimeMissing);
    }
    let _thread = AttachedThread::enter(exports, domain)?;
    let mut workspace = SnapshotWorkspace::new()?;
    trace("snapshot.world.begin");
    {
        let _world = GcWorld::stop(exports);
        let SnapshotWorkspace {
            liveness,
            raw_texts,
            utf16,
        } = &mut workspace;
        for target in targets {
            let objects = liveness_objects(exports, target.class, liveness)?;
            for &object in objects {
                if raw_texts.len() == MAX_SNAPSHOT_OBJECTS {
                    break;
                }
                read_text_field_into(exports, object, *target, raw_texts, utf16);
            }
        }
    }
    trace("snapshot.world.end");
    Ok(workspace.finish())
}

unsafe fn read_text_field_into(
    exports: Il2CppExports,
    object: usize,
    target: TextTarget,
    raw_texts: &mut Vec<RawText>,
    utf16: &mut Vec<u16>,
) {
    if object == 0 {
        return;
    }
    let mut string = std::ptr::null_mut::<c_void>();
    (exports.field_get_value)(
        object as *mut c_void,
        target.text_field as *mut c_void,
        (&mut string as *mut *mut c_void).cast::<c_void>(),
    );
    if string.is_null() {
        return;
    }
    let Ok(length) = usize::try_from((exports.string_length)(string)) else {
        return;
    };
    if length > MAX_TEXT_UNITS || length > MAX_SNAPSHOT_UTF16_UNITS.saturating_sub(utf16.len()) {
        return;
    }
    let chars = (exports.string_chars)(string);
    if chars.is_null() && length != 0 {
        return;
    }
    let start = utf16.len();
    if length != 0 {
        utf16.extend_from_slice(std::slice::from_raw_parts(chars, length));
    }
    raw_texts.push(RawText {
        object: object as u64,
        kind: target.kind,
        start,
        length,
    });
}

struct SnapshotWorkspace {
    liveness: LivenessCallbackState,
    raw_texts: Vec<RawText>,
    utf16: Vec<u16>,
}

impl SnapshotWorkspace {
    fn new() -> Result<Self, MetadataError> {
        Ok(Self {
            liveness: LivenessCallbackState::new()?,
            raw_texts: Vec::with_capacity(MAX_SNAPSHOT_OBJECTS),
            utf16: Vec::with_capacity(MAX_SNAPSHOT_UTF16_UNITS),
        })
    }

    fn finish(self) -> Vec<ManagedText> {
        let Self {
            raw_texts, utf16, ..
        } = self;
        raw_texts
            .into_iter()
            .map(|raw| {
                ManagedText::utf16(
                    ManagedObjectId::new(raw.object),
                    raw.kind,
                    utf16[raw.start..raw.start + raw.length].to_vec(),
                )
            })
            .collect()
    }
}

#[derive(Clone, Copy)]
struct RawText {
    object: u64,
    kind: StandardUiKind,
    start: usize,
    length: usize,
}

struct AttachedThread {
    thread: *mut c_void,
    detach: Option<crate::runtime_gate::ThreadDetach>,
}

impl AttachedThread {
    unsafe fn enter(exports: Il2CppExports, domain: *mut c_void) -> Result<Self, MetadataError> {
        let current = (exports.thread_current)();
        if !current.is_null() {
            return Ok(Self {
                thread: current,
                detach: None,
            });
        }
        let thread = (exports.thread_attach)(domain);
        if thread.is_null() {
            return Err(MetadataError::ThreadAttachFailed);
        }
        Ok(Self {
            thread,
            detach: Some(exports.thread_detach),
        })
    }
}

impl Drop for AttachedThread {
    fn drop(&mut self) {
        if let Some(detach) = self.detach {
            unsafe { detach(self.thread) };
        }
    }
}

struct GcWorld {
    start: StartGcWorld,
}

impl GcWorld {
    unsafe fn stop(exports: Il2CppExports) -> Self {
        (exports.stop_gc_world)();
        Self {
            start: exports.start_gc_world,
        }
    }
}

impl Drop for GcWorld {
    fn drop(&mut self) {
        unsafe { (self.start)() };
    }
}

struct LivenessCallbackState {
    objects: Vec<usize>,
    active_allocations: usize,
    #[cfg(windows)]
    heap: windows::Win32::Foundation::HANDLE,
    #[cfg(not(windows))]
    allocations: [LivenessAllocation; MAX_LIVENESS_ALLOCATIONS],
}

impl LivenessCallbackState {
    fn new() -> Result<Self, MetadataError> {
        #[cfg(windows)]
        let heap = unsafe {
            use windows::Win32::System::Memory::{HeapCreate, HEAP_NO_SERIALIZE};
            HeapCreate(HEAP_NO_SERIALIZE, 0, 0).map_err(|_| MetadataError::LivenessFailed)?
        };
        Ok(Self {
            objects: Vec::with_capacity(MAX_SNAPSHOT_OBJECTS),
            active_allocations: 0,
            #[cfg(windows)]
            heap,
            #[cfg(not(windows))]
            allocations: [LivenessAllocation::EMPTY; MAX_LIVENESS_ALLOCATIONS],
        })
    }

    unsafe fn allocate(&mut self, size: usize) -> *mut c_void {
        #[cfg(windows)]
        let memory = {
            use windows::Win32::System::Memory::{HeapAlloc, HEAP_NO_SERIALIZE};
            HeapAlloc(self.heap, HEAP_NO_SERIALIZE, size)
        };
        #[cfg(not(windows))]
        let memory = {
            let Some(slot) = self
                .allocations
                .iter_mut()
                .find(|allocation| allocation.address == 0)
            else {
                return std::ptr::null_mut();
            };
            let memory = allocate_liveness_memory(size);
            if !memory.is_null() {
                *slot = LivenessAllocation {
                    address: memory as usize,
                    size,
                };
            }
            memory
        };
        if !memory.is_null() {
            self.active_allocations += 1;
        }
        memory
    }

    unsafe fn resize(&mut self, memory: *mut c_void, size: usize) -> *mut c_void {
        #[cfg(windows)]
        {
            use windows::Win32::System::Memory::{HeapReAlloc, HEAP_NO_SERIALIZE};
            HeapReAlloc(
                self.heap,
                HEAP_NO_SERIALIZE,
                Some(memory.cast_const()),
                size,
            )
        }
        #[cfg(not(windows))]
        {
            let Some(slot) = self
                .allocations
                .iter_mut()
                .find(|allocation| allocation.address == memory as usize)
            else {
                return std::ptr::null_mut();
            };
            let next = allocate_liveness_memory(size);
            if next.is_null() {
                return std::ptr::null_mut();
            }
            std::ptr::copy_nonoverlapping(
                memory.cast::<u8>(),
                next.cast::<u8>(),
                slot.size.min(size),
            );
            free_liveness_memory(memory, slot.size);
            *slot = LivenessAllocation {
                address: next as usize,
                size,
            };
            next
        }
    }

    unsafe fn release(&mut self, memory: *mut c_void) {
        #[cfg(windows)]
        let released = {
            use windows::Win32::System::Memory::{HeapFree, HEAP_NO_SERIALIZE};
            HeapFree(self.heap, HEAP_NO_SERIALIZE, Some(memory.cast_const())).is_ok()
        };
        #[cfg(not(windows))]
        let released = {
            let Some(slot) = self
                .allocations
                .iter_mut()
                .find(|allocation| allocation.address == memory as usize)
            else {
                return;
            };
            free_liveness_memory(memory, slot.size);
            *slot = LivenessAllocation::EMPTY;
            true
        };
        if released {
            self.active_allocations = self.active_allocations.saturating_sub(1);
        }
    }
}

impl Drop for LivenessCallbackState {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Memory::HeapDestroy;
            let _ = HeapDestroy(self.heap);
        }
        #[cfg(not(windows))]
        for allocation in &mut self.allocations {
            if allocation.address != 0 {
                unsafe { free_liveness_memory(allocation.address as *mut c_void, allocation.size) };
                *allocation = LivenessAllocation::EMPTY;
            }
        }
    }
}

unsafe fn liveness_objects(
    exports: Il2CppExports,
    class: usize,
    callback_state: &mut LivenessCallbackState,
) -> Result<&[usize], MetadataError> {
    callback_state.objects.clear();
    let state = (exports.liveness_allocate)(
        class as *mut c_void,
        MAX_SNAPSHOT_OBJECTS as i32,
        Some(register_liveness_objects),
        (callback_state as *mut LivenessCallbackState).cast::<c_void>(),
        Some(reallocate_liveness),
    );
    if state.is_null() {
        return Err(MetadataError::LivenessFailed);
    }
    (exports.liveness_from_statics)(state);
    (exports.liveness_finalize)(state);
    (exports.liveness_free)(state);
    callback_state.objects.sort_unstable();
    callback_state.objects.dedup();
    Ok(&callback_state.objects)
}

unsafe extern "C" fn register_liveness_objects(
    objects: *mut *mut c_void,
    size: i32,
    user_data: *mut c_void,
) {
    if objects.is_null() || user_data.is_null() || size <= 0 {
        return;
    }
    let state = &mut *user_data.cast::<LivenessCallbackState>();
    let remaining = MAX_SNAPSHOT_OBJECTS.saturating_sub(state.objects.len());
    for index in 0..usize::try_from(size).unwrap_or(0).min(remaining) {
        let object = *objects.add(index);
        if !object.is_null() {
            state.objects.push(object as usize);
        }
    }
}

unsafe extern "C" fn reallocate_liveness(
    memory: *mut c_void,
    size: usize,
    user_data: *mut c_void,
) -> *mut c_void {
    if user_data.is_null() {
        return std::ptr::null_mut();
    }
    let state = &mut *user_data.cast::<LivenessCallbackState>();
    if size == 0 {
        state.release(memory);
        return std::ptr::null_mut();
    }
    if memory.is_null() {
        state.allocate(size)
    } else {
        state.resize(memory, size)
    }
}

#[cfg(not(windows))]
#[derive(Clone, Copy)]
struct LivenessAllocation {
    address: usize,
    size: usize,
}

#[cfg(not(windows))]
impl LivenessAllocation {
    const EMPTY: Self = Self {
        address: 0,
        size: 0,
    };
}

#[cfg(not(windows))]
unsafe fn allocate_liveness_memory(size: usize) -> *mut c_void {
    use std::alloc::{alloc, Layout};

    let Ok(layout) = Layout::from_size_align(size, std::mem::align_of::<usize>()) else {
        return std::ptr::null_mut();
    };
    alloc(layout).cast::<c_void>()
}

#[cfg(not(windows))]
unsafe fn free_liveness_memory(memory: *mut c_void, size: usize) {
    use std::alloc::{dealloc, Layout};

    if let Ok(layout) = Layout::from_size_align(size, std::mem::align_of::<usize>()) {
        dealloc(memory.cast::<u8>(), layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liveness_allocator_tracks_growth_and_release() {
        let mut state = LivenessCallbackState::new().expect("private liveness allocator");
        let user_data = (&mut state as *mut LivenessCallbackState).cast::<c_void>();
        let first = unsafe { reallocate_liveness(std::ptr::null_mut(), 32, user_data) };
        assert!(!first.is_null());
        let second = unsafe { reallocate_liveness(first, 96, user_data) };
        assert!(!second.is_null());
        assert_eq!(state.active_allocations, 1);
        assert!(unsafe { reallocate_liveness(second, 0, user_data) }.is_null());
        assert_eq!(state.active_allocations, 0);
    }

    #[test]
    fn workspace_preallocates_all_stop_the_world_buffers() {
        let workspace = SnapshotWorkspace::new().expect("snapshot workspace");
        assert_eq!(workspace.liveness.objects.capacity(), MAX_SNAPSHOT_OBJECTS);
        assert_eq!(workspace.raw_texts.capacity(), MAX_SNAPSHOT_OBJECTS);
        assert_eq!(workspace.utf16.capacity(), MAX_SNAPSHOT_UTF16_UNITS);
    }
}
