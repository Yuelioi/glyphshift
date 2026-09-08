use std::ffi::c_void;
use windows_sys::Win32::System::{
    Diagnostics::Debug::ReadProcessMemory, Memory::*, Threading::GetCurrentProcess,
};

pub fn read(address: usize, length: usize) -> Option<Vec<u8>> {
    address.checked_add(length)?;
    let mut output = vec![0; length];
    let mut count = 0;
    let ok = unsafe {
        ReadProcessMemory(
            GetCurrentProcess(),
            address as *const c_void,
            output.as_mut_ptr().cast(),
            length,
            &mut count,
        )
    };
    (ok != 0 && count == length).then_some(output)
}

pub fn word(address: usize) -> Option<usize> {
    Some(usize::from_ne_bytes(
        read(address, size_of::<usize>())?.try_into().ok()?,
    ))
}

pub fn region(address: usize) -> Option<MEMORY_BASIC_INFORMATION> {
    let mut info = unsafe { std::mem::zeroed::<MEMORY_BASIC_INFORMATION>() };
    let ok = unsafe {
        VirtualQuery(
            address as *const c_void,
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    (ok != 0 && info.State == MEM_COMMIT && info.Protect & (PAGE_GUARD | PAGE_NOACCESS) == 0)
        .then_some(info)
}

pub fn code(address: usize, owner: usize, limit: usize) -> Option<Vec<u8>> {
    let info = region(address)?;
    if info.AllocationBase as usize != owner
        || !matches!(
            info.Protect & 0xff,
            PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY
        )
    {
        return None;
    }
    let remaining = (info.BaseAddress as usize)
        .checked_add(info.RegionSize)?
        .checked_sub(address)?;
    read(address, remaining.min(limit))
}

pub fn utf16(address: usize, limit: usize) -> Option<Vec<u16>> {
    if address == 0 || !address.is_multiple_of(2) {
        return None;
    }
    let mut result = Vec::new();
    let mut at = address;
    while result.len() <= limit {
        let info = region(at)?;
        let available = (info.BaseAddress as usize)
            .checked_add(info.RegionSize)?
            .checked_sub(at)?;
        let size = available.min((limit + 1 - result.len()) * 2) & !1;
        if size == 0 {
            return None;
        }
        for bytes in read(at, size)?.as_chunks::<2>().0 {
            let unit = u16::from_le_bytes([bytes[0], bytes[1]]);
            if unit == 0 {
                return Some(result);
            }
            result.push(unit);
        }
        at = at.checked_add(size)?;
    }
    None
}
