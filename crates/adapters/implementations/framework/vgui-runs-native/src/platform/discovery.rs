use super::*;
use iced_x86::{Decoder, DecoderOptions, FlowControl, Mnemonic, OpKind};
use std::collections::BTreeSet;
use windows_sys::Win32::{
    Foundation::FreeLibrary,
    System::{LibraryLoader::*, Memory::*, ProcessStatus::*},
};

pub(super) struct Pin(pub usize);
impl Drop for Pin {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.0 as *mut c_void);
        }
    }
}
pub(super) unsafe fn pin(address: usize) -> Option<Pin> {
    let mut module = std::ptr::null_mut();
    (GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        address as *const u16,
        &mut module,
    ) != 0)
        .then_some(Pin(module as usize))
}
pub(super) unsafe fn readable(address: usize, length: usize) -> bool {
    let Some(end) = address.checked_add(length) else {
        return false;
    };
    let mut at = address;
    while at < end {
        let mut info: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
        if VirtualQuery(
            at as *const _,
            &mut info,
            size_of::<MEMORY_BASIC_INFORMATION>(),
        ) == 0
            || info.State != MEM_COMMIT
            || info.Protect & (PAGE_GUARD | PAGE_NOACCESS) != 0
        {
            return false;
        }
        if !matches!(
            info.Protect & 0xff,
            PAGE_READONLY
                | PAGE_READWRITE
                | PAGE_WRITECOPY
                | PAGE_EXECUTE_READ
                | PAGE_EXECUTE_READWRITE
                | PAGE_EXECUTE_WRITECOPY
        ) {
            return false;
        }
        let Some(next) = (info.BaseAddress as usize).checked_add(info.RegionSize) else {
            return false;
        };
        if next <= at {
            return false;
        }
        at = next;
    }
    true
}
pub(super) unsafe fn read<T: Copy>(address: usize) -> Option<T> {
    readable(address, size_of::<T>()).then(|| (address as *const T).read_unaligned())
}
pub(super) unsafe fn bytes(address: usize, length: usize) -> Option<&'static [u8]> {
    readable(address, length).then(|| std::slice::from_raw_parts(address as *const u8, length))
}
pub(super) unsafe fn executable(address: usize) -> bool {
    let mut info: MEMORY_BASIC_INFORMATION = std::mem::zeroed();
    VirtualQuery(
        address as *const _,
        &mut info,
        size_of::<MEMORY_BASIC_INFORMATION>(),
    ) != 0
        && info.State == MEM_COMMIT
        && info.Protect & PAGE_GUARD == 0
        && matches!(
            info.Protect & 0xff,
            PAGE_EXECUTE_READ | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY
        )
}

pub(super) unsafe fn surface() -> Option<(Pin, usize, Vec<usize>, Layout, Factory)> {
    let name = "vguimatsurface.dll"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let module = GetModuleHandleW(name.as_ptr());
    if module.is_null() {
        return None;
    }
    let held = pin(module as usize)?;
    let factory = GetProcAddress(module, c"CreateInterface".as_ptr().cast())?;
    let factory: unsafe extern "C" fn(*const u8, *mut i32) -> *mut c_void =
        std::mem::transmute(factory);
    let surface = factory(c"VGUI_Surface031".as_ptr().cast(), std::ptr::null_mut()) as usize;
    if surface == 0 {
        return None;
    }
    let table = read::<usize>(surface)?;
    let mut entries = Vec::new();
    for slot in 0..384 {
        let entry = read::<usize>(table + slot * 4)?;
        if !executable(entry) {
            break;
        }
        entries.push(entry);
    }
    if entries.len() == 384 {
        return None;
    }
    for layout in [
        Layout {
            font: 17,
            color: 19,
            pos: 20,
            get_pos: 21,
            print: 22,
            character: 23,
            flush: 24,
            measure: 72,
            width: 71,
            query_start: 66,
        },
        Layout {
            font: 23,
            color: 25,
            pos: 26,
            get_pos: 27,
            print: 28,
            character: 29,
            flush: 30,
            measure: 79,
            width: 78,
            query_start: 73,
        },
    ] {
        if entries.len() <= layout.measure {
            continue;
        }
        if position_pair(
            bytes(entries[layout.pos], 32)?,
            bytes(entries[layout.get_pos], 32)?,
        ) && measure_factory(entries[layout.measure]).is_some()
        {
            return Some((held, surface, entries, layout, factory));
        }
    }
    None
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Value {
    Unknown,
    This,
    Frame,
    Argument(usize),
    Field(i32),
}
fn accessor(code: &[u8], setter: bool) -> Option<[i32; 2]> {
    if !code.starts_with(&[0x55, 0x8b, 0xec]) {
        return None;
    }
    let mut regs = [Value::Unknown; 8];
    regs[1] = Value::This;
    regs[5] = Value::Frame;
    let mut fields: [Option<i32>; 2] = [None, None];
    let mut at = 3;
    while at < code.len() {
        if code.get(at..at + 4) == Some(&[0x5d, 0xc2, 8, 0]) {
            let pair = [fields[0]?, fields[1]?];
            return (pair[0] >= 4 && pair[0].checked_add(4) == Some(pair[1])).then_some(pair);
        }
        let opcode = *code.get(at)?;
        if opcode != 0x8b && opcode != 0x89 {
            return None;
        }
        let modrm = *code.get(at + 1)?;
        at += 2;
        let mode = modrm >> 6;
        let register = ((modrm >> 3) & 7) as usize;
        let base = (modrm & 7) as usize;
        if mode == 3 {
            if opcode == 0x8b {
                regs[register] = regs[base];
            } else {
                regs[base] = regs[register];
            }
            continue;
        }
        if base == 4 || (mode == 0 && base == 5) {
            return None;
        }
        let offset = match mode {
            0 => 0,
            1 => {
                let n = *code.get(at)? as i8 as i32;
                at += 1;
                n
            }
            2 => {
                let n = i32::from_le_bytes(code.get(at..at + 4)?.try_into().ok()?);
                at += 4;
                n
            }
            _ => return None,
        };
        if opcode == 0x8b {
            regs[register] = match (regs[base], offset) {
                (Value::This, n) => Value::Field(n),
                (Value::Frame, 8) => Value::Argument(0),
                (Value::Frame, 12) => Value::Argument(1),
                _ => return None,
            };
        } else {
            let (index, field) = match (setter, regs[base], regs[register], offset) {
                (true, Value::This, Value::Argument(index), field) => (index, field),
                (false, Value::Argument(index), Value::Field(field), 0) => (index, field),
                _ => return None,
            };
            if fields[index].replace(field).is_some() {
                return None;
            }
        }
    }
    None
}
fn position_pair(set: &[u8], get: &[u8]) -> bool {
    accessor(set, true).is_some_and(|fields| accessor(get, false) == Some(fields))
}
pub(super) unsafe fn measure_factory(address: usize) -> Option<usize> {
    let code = bytes(address, 64)?;
    for i in 0..code.len() - 16 {
        if code[i] == 0xe8
            && code[i + 5..i + 8] == [0x8b, 0xc8, 0xe8]
            && code[i + 12..i + 16] == [0x5d, 0xc2, 16, 0]
        {
            let rel = i32::from_le_bytes(code[i + 1..i + 5].try_into().ok()?);
            return Some(
                (address as u32)
                    .wrapping_add(i as u32 + 5)
                    .wrapping_add(rel as u32) as usize,
            );
        }
    }
    None
}

/// Walk bounded reachable instructions, not raw byte matches: every returning
/// path must use the declared x86 callee stack cleanup. Unknown tail dispatch is
/// rejected before any typed method call is installed.
pub(super) unsafe fn cleanup_matches(address: usize, expected: u16) -> bool {
    let mut pending = vec![address];
    let mut visited = BTreeSet::new();
    let mut returns = 0;
    while let Some(ip) = pending.pop() {
        if !visited.insert(ip) {
            continue;
        }
        if visited.len() > 1024 || ip.abs_diff(address) > 64 * 1024 || !executable(ip) {
            return false;
        }
        let Some(code) = bytes(ip, 15) else {
            return false;
        };
        let instruction = Decoder::with_ip(32, code, ip as u64, DecoderOptions::NONE).decode();
        if instruction.is_invalid() {
            return false;
        }
        match instruction.flow_control() {
            FlowControl::Return => {
                if instruction.mnemonic() != Mnemonic::Ret {
                    return false;
                }
                let count = if instruction.op_count() == 0 {
                    0
                } else if instruction.op0_kind() == OpKind::Immediate16 {
                    instruction.immediate16()
                } else {
                    return false;
                };
                if count != expected {
                    return false;
                }
                returns += 1;
            }
            FlowControl::UnconditionalBranch => {
                if instruction.op0_kind() != OpKind::NearBranch32 {
                    return false;
                }
                pending.push(instruction.near_branch_target() as usize);
            }
            FlowControl::ConditionalBranch => {
                if instruction.op0_kind() != OpKind::NearBranch32 {
                    return false;
                }
                pending.push(instruction.near_branch_target() as usize);
                pending.push(instruction.next_ip() as usize);
            }
            FlowControl::Next | FlowControl::Call | FlowControl::IndirectCall => {
                pending.push(instruction.next_ip() as usize)
            }
            _ => return false,
        }
    }
    returns > 0
}
pub(super) unsafe fn font_query(address: usize, factory: usize) -> bool {
    let Some(code) = bytes(address, 64) else {
        return false;
    };
    (0..code.len() - 7).any(|i| {
        code[i] == 0xe8
            && code[i + 5..i + 7] == [0x8b, 0xc8]
            && (address as u32)
                .wrapping_add(i as u32 + 5)
                .wrapping_add(i32::from_le_bytes(code[i + 1..i + 5].try_into().unwrap()) as u32)
                as usize
                == factory
    })
}

pub(super) unsafe fn images() -> Vec<(Pin, usize, usize)> {
    let mut modules = vec![std::ptr::null_mut(); 512];
    let mut needed = 0;
    if K32EnumProcessModules(
        GetCurrentProcess(),
        modules.as_mut_ptr(),
        (modules.len() * 4) as u32,
        &mut needed,
    ) == 0
        || needed as usize > modules.len() * 4
    {
        return Vec::new();
    }
    let mut found = Vec::new();
    for module in modules.into_iter().take(needed as usize / 4) {
        let Some(held) = pin(module as usize) else {
            continue;
        };
        if GetProcAddress(module, c"CreateInterface".as_ptr().cast()).is_none() {
            continue;
        }
        let Some(ranges) = data_ranges(module as usize) else {
            continue;
        };
        let mut tables = Vec::new();
        for (base, data) in &ranges {
            for (index, _) in data
                .windows(b".?AVTextImage@vgui@@\0".len())
                .enumerate()
                .filter(|(_, s)| *s == b".?AVTextImage@vgui@@\0")
            {
                if index < 8 {
                    continue;
                }
                let type_info = base + index - 8;
                for location in references(&ranges, type_info as u32) {
                    let Some(col) = location.checked_sub(12) else {
                        continue;
                    };
                    if read::<[u32; 3]>(col) != Some([0, 0, 0]) || !image_hierarchy(col, type_info)
                    {
                        continue;
                    }
                    for location in references(&ranges, col as u32) {
                        if let Some(paint) = read::<usize>(location + 4) {
                            if executable(paint) {
                                tables.push((location + 4, paint));
                            }
                        }
                    }
                }
            }
        }
        tables.sort_unstable();
        tables.dedup();
        if tables.len() == 1 {
            found.push((held, tables[0].0, tables[0].1));
        }
        if found.len() >= 32 {
            break;
        }
    }
    found
}
unsafe fn image_hierarchy(col: usize, root_type: usize) -> bool {
    let Some(hierarchy) = read::<usize>(col + 16) else {
        return false;
    };
    if read::<u32>(hierarchy) != Some(0) {
        return false;
    }
    let Some(count) = read::<u32>(hierarchy + 8).filter(|count| (2..=64).contains(count)) else {
        return false;
    };
    let Some(array) = read::<usize>(hierarchy + 12) else {
        return false;
    };
    let mut image_base = false;
    for index in 0..count as usize {
        let Some(base) = read::<usize>(array + index * 4) else {
            return false;
        };
        let Some(kind) = read::<usize>(base) else {
            return false;
        };
        if index == 0 && kind != root_type {
            return false;
        }
        let name = b".?AVIImage@vgui@@\0";
        let alternative = b".?AUIImage@vgui@@\0";
        if bytes(kind + 8, name.len()).is_some_and(|value| value == name || value == alternative) {
            image_base = read::<i32>(base + 8) == Some(0) && read::<i32>(base + 12) == Some(-1);
        }
    }
    image_base
}
fn references(ranges: &[(usize, &[u8])], needle: u32) -> Vec<usize> {
    ranges
        .iter()
        .flat_map(|(base, data)| {
            data.as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .filter_map(move |(i, part)| {
                    (u32::from_le_bytes(*part) == needle).then_some(base + i * 4)
                })
        })
        .collect()
}
unsafe fn data_ranges(base: usize) -> Option<Vec<(usize, &'static [u8])>> {
    let mut info: MODULEINFO = std::mem::zeroed();
    if K32GetModuleInformation(
        GetCurrentProcess(),
        base as *mut _,
        &mut info,
        size_of::<MODULEINFO>() as u32,
    ) == 0
    {
        return None;
    }
    if read::<u16>(base)? != 0x5a4d {
        return None;
    }
    let pe = read::<u32>(base + 60)? as usize;
    if pe > 1024 * 1024 || pe + 24 >= info.SizeOfImage as usize || read::<u32>(base + pe)? != 0x4550
    {
        return None;
    }
    let count = read::<u16>(base + pe + 6)? as usize;
    let optional = read::<u16>(base + pe + 20)? as usize;
    if count > 96 {
        return None;
    }
    let mut ranges = Vec::new();
    for i in 0..count {
        let at = base + pe + 24 + optional + i * 40;
        let flags = read::<u32>(at + 36)?;
        if flags & 0x40000000 == 0 || flags & 0x20000000 != 0 {
            continue;
        }
        let offset = read::<u32>(at + 12)? as usize;
        let size = read::<u32>(at + 8)? as usize;
        if size == 0
            || size > 16 * 1024 * 1024
            || offset.checked_add(size)? > info.SizeOfImage as usize
        {
            continue;
        }
        if let Some(data) = bytes(base + offset, size) {
            ranges.push((base + offset, data));
        }
    }
    Some(ranges)
}
