//! Opaque x86 method barriers preserve stack arguments, registers and FP state,
//! then tail-call the exact original method. No virtual-method signature is guessed.
use super::*;
use windows_sys::Win32::System::{Diagnostics::Debug::FlushInstructionCache, Memory::*};

pub(super) struct Shadow {
    pub table: usize,
    _entries: Box<[usize]>,
    _code: usize,
}
impl Drop for Shadow {
    fn drop(&mut self) {
        unsafe {
            VirtualFree(self._code as *mut c_void, 0, MEM_RELEASE);
        }
    }
}

pub(super) unsafe fn create(
    original: usize,
    entries: &[usize],
    overrides: &[(usize, usize)],
    queries: &[usize],
) -> Result<Shadow, ()> {
    let rtti = discovery::read::<usize>(original - 4).ok_or(())?;
    let stride = 64usize;
    let length = entries.len().checked_mul(stride).ok_or(())?;
    let memory = VirtualAlloc(
        std::ptr::null(),
        length,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_READWRITE,
    );
    if memory.is_null() {
        return Err(());
    }
    let mut table = Vec::with_capacity(entries.len() + 1);
    table.push(rtti);
    for (index, target) in entries.iter().copied().enumerate() {
        if let Some((_, replacement)) = overrides.iter().find(|(slot, _)| *slot == index) {
            table.push(*replacement);
            continue;
        }
        if queries.contains(&index) {
            table.push(target);
            continue;
        }
        let address = memory as usize + index * stride;
        let mut code = vec![0x9c, 0x83, 0x3d]; // pushfd; cmp low active-feature word,0
        code.extend_from_slice(&(std::ptr::addr_of!(ACTIVE) as u32).to_le_bytes());
        code.push(0);
        code.extend_from_slice(&[0x74, 0]);
        let branch = code.len() - 1;
        code.extend_from_slice(&[
            0x60, 0x8b, 0xec, 0x81, 0xec, 0x20, 0x02, 0, 0, 0x83, 0xe4, 0xf0, 0x0f, 0xae, 0x04,
            0x24, 0xb8,
        ]);
        code.extend_from_slice(&(super::barrier as *const () as u32).to_le_bytes());
        code.extend_from_slice(&[0xff, 0xd0, 0x0f, 0xae, 0x0c, 0x24, 0x8b, 0xe5, 0x61]);
        code[branch] = (code.len() - branch - 1) as u8;
        code.push(0x9d);
        code.push(0xe9);
        let displacement = (target as u32).wrapping_sub((address + code.len() + 4) as u32);
        code.extend_from_slice(&displacement.to_le_bytes());
        if code.len() > stride {
            VirtualFree(memory, 0, MEM_RELEASE);
            return Err(());
        }
        std::ptr::copy_nonoverlapping(code.as_ptr(), address as *mut u8, code.len());
        table.push(address);
    }
    let mut old = 0;
    if VirtualProtect(memory, length, PAGE_EXECUTE_READ, &mut old) == 0 {
        VirtualFree(memory, 0, MEM_RELEASE);
        return Err(());
    }
    FlushInstructionCache(GetCurrentProcess(), memory, length);
    let entries = table.into_boxed_slice();
    let table = entries.as_ptr().add(1) as usize;
    Ok(Shadow {
        table,
        _entries: entries,
        _code: memory as usize,
    })
}
