//! Returned storage belongs to the target process, not an activation/session.
//! Exposed pages are never freed or rewritten by this Adapter.
use crate::memory;
use std::collections::BTreeMap;
use windows_sys::Win32::System::{
    Diagnostics::Debug::WriteProcessMemory,
    Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE},
    Threading::GetCurrentProcess,
};

pub const MAX_RETURN_BYTES: usize = 1024 * 1024;
pub const MAX_RETURN_VALUES: usize = 4096;

#[derive(Default)]
pub struct ReturnedText {
    base: usize,
    used: usize,
    values: BTreeMap<(Vec<u16>, Vec<u16>), usize>,
    compromised: bool,
}

impl ReturnedText {
    pub fn contains(&self, source: &[u16], text: &[u16]) -> bool {
        !self.compromised && self.values.contains_key(&(source.to_vec(), text.to_vec()))
    }
    pub fn intern(&mut self, source: &[u16], text: &[u16]) -> Option<usize> {
        if self.compromised {
            return None;
        }
        let key = (source.to_vec(), text.to_vec());
        if let Some(&address) = self.values.get(&key) {
            // Copy through the OS: the API returns writable memory to foreign
            // callers, so it must never be borrowed as a shared Rust slice.
            if memory::utf16(address, text.len()).as_deref() != Some(text) {
                self.compromised = true;
                return None;
            }
            return Some(address);
        }
        let bytes = text.len().checked_add(1)?.checked_mul(2)?;
        let next = self.used.checked_add(bytes)?;
        if next > MAX_RETURN_BYTES || self.values.len() >= MAX_RETURN_VALUES {
            return None;
        }
        if self.base == 0 {
            self.base = unsafe {
                VirtualAlloc(
                    std::ptr::null(),
                    MAX_RETURN_BYTES,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_READWRITE,
                )
            } as usize;
            if self.base == 0 {
                return None;
            }
        }
        let address = self.base.checked_add(self.used)?;
        // This range has never been published. No later call writes it again.
        let terminated = text.iter().copied().chain(Some(0)).collect::<Vec<_>>();
        let mut written = 0;
        let ok = unsafe {
            WriteProcessMemory(
                GetCurrentProcess(),
                address as *const _,
                terminated.as_ptr().cast(),
                bytes,
                &mut written,
            )
        };
        if ok == 0 || written != bytes {
            self.compromised = true;
            return None;
        }
        self.used = next;
        self.values.insert(key, address);
        Some(address)
    }
}
