use crate::runtime_gate::Il2CppWritebackExports;
use glyphshift_adapter_unity_standard_ui::ManagedObjectId;
use std::collections::{BTreeMap, HashMap};
use std::ffi::c_void;

#[derive(Clone, Copy, Debug)]
struct TrackedObject {
    handle: usize,
}

#[derive(Debug)]
pub(crate) struct ObjectRegistry {
    next_id: u64,
    objects: BTreeMap<ManagedObjectId, TrackedObject>,
    pointers: HashMap<usize, ManagedObjectId>,
}

impl Default for ObjectRegistry {
    fn default() -> Self {
        Self {
            next_id: 1,
            objects: BTreeMap::new(),
            pointers: HashMap::new(),
        }
    }
}

impl ObjectRegistry {
    pub(crate) unsafe fn refresh(
        &mut self,
        exports: Il2CppWritebackExports,
    ) -> Vec<ManagedObjectId> {
        let mut live_pointers = HashMap::with_capacity(self.objects.len());
        let mut collected = Vec::new();
        let tracked = self
            .objects
            .iter()
            .map(|(object_id, tracked)| (*object_id, *tracked))
            .collect::<Vec<_>>();
        for (object_id, tracked) in tracked {
            let target = unsafe { (exports.gchandle_get_target)(tracked.handle) };
            if target.is_null() {
                self.objects.remove(&object_id);
                unsafe { (exports.gchandle_free)(tracked.handle) };
                collected.push(object_id);
            } else {
                live_pointers.insert(target as usize, object_id);
            }
        }
        self.pointers = live_pointers;
        collected
    }

    pub(crate) unsafe fn adopt_snapshot_handle(
        &mut self,
        exports: Il2CppWritebackExports,
        handle: usize,
    ) -> Option<ManagedObjectId> {
        if handle == 0 {
            return None;
        }
        let target = unsafe { (exports.gchandle_get_target)(handle) };
        if target.is_null() {
            unsafe { (exports.gchandle_free)(handle) };
            return None;
        }
        if let Some(object_id) = self.pointers.get(&(target as usize)).copied() {
            unsafe { (exports.gchandle_free)(handle) };
            return Some(object_id);
        }
        let Some(object_id) = self.next_object_id() else {
            unsafe { (exports.gchandle_free)(handle) };
            return None;
        };
        self.objects.insert(object_id, TrackedObject { handle });
        self.pointers.insert(target as usize, object_id);
        Some(object_id)
    }

    pub(crate) unsafe fn target(
        &self,
        exports: Il2CppWritebackExports,
        object_id: ManagedObjectId,
    ) -> Option<*mut c_void> {
        let tracked = self.objects.get(&object_id)?;
        let target = unsafe { (exports.gchandle_get_target)(tracked.handle) };
        (!target.is_null()).then_some(target)
    }

    pub(crate) unsafe fn clear(&mut self, exports: Il2CppWritebackExports) {
        for tracked in std::mem::take(&mut self.objects).into_values() {
            unsafe { (exports.gchandle_free)(tracked.handle) };
        }
        self.pointers.clear();
    }

    fn next_object_id(&mut self) -> Option<ManagedObjectId> {
        for _ in 0..2 {
            let value = self.next_id;
            self.next_id = self.next_id.wrapping_add(1);
            if self.next_id == 0 {
                self.next_id = 1;
            }
            if value != 0 {
                let object_id = ManagedObjectId::new(value);
                if !self.objects.contains_key(&object_id) {
                    return Some(object_id);
                }
            }
        }
        None
    }
}
