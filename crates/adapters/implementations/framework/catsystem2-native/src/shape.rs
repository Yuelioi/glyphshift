//! Narrow MSVC x86 CatSystem2 object profiles. Addresses and member offsets
//! come from the loaded image; ambiguous tables and unsupported code are rejected.
use iced_x86::{Decoder, DecoderOptions, Instruction, Mnemonic as M, OpKind, Register as R};
use std::collections::BTreeSet;
use std::ops::Range;
mod classic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Utf8,
    Classic,
}

pub struct Image {
    pub base: usize,
    pub bytes: Vec<u8>,
    pub executable: Vec<Range<usize>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub profile: Profile,
    pub table: usize,
    pub setter: usize,
    pub append: usize,
    pub destroy: usize,
    pub tick: usize,
    pub text_member: usize,
    pub show: usize,
    pub finish: usize,
    pub history: Option<(usize, usize)>,
}

impl Image {
    pub fn read(&self, at: usize, len: usize) -> Option<&[u8]> {
        let start = at.checked_sub(self.base)?;
        self.bytes.get(start..start.checked_add(len)?)
    }
    fn word(&self, at: usize) -> Option<usize> {
        Some(u32::from_le_bytes(self.read(at, 4)?.try_into().ok()?) as usize)
    }
    fn code(&self, at: usize) -> bool {
        self.executable.iter().any(|range| range.contains(&at))
    }
    fn occurrences(&self, needle: &[u8]) -> Vec<usize> {
        self.bytes
            .windows(needle.len())
            .enumerate()
            .filter_map(|(i, bytes)| (bytes == needle).then_some(self.base + i))
            .collect()
    }
    fn references(&self, at: usize) -> Vec<usize> {
        self.occurrences(&(at as u32).to_le_bytes())
    }
    fn instructions(&self, at: usize, limit: usize) -> Option<Vec<Instruction>> {
        let range = self.executable.iter().find(|r| r.contains(&at))?;
        let code = self.read(at, limit.min(range.end - at))?;
        let mut result = Vec::new();
        for instruction in Decoder::with_ip(32, code, at as u64, DecoderOptions::NONE) {
            if instruction.is_invalid() || instruction.mnemonic() == M::Int3 {
                return None;
            }
            result.push(instruction);
            if instruction.mnemonic() == M::Ret {
                return Some(result);
            }
        }
        None
    }
    fn table(&self) -> Option<usize> {
        self.named_table(b".?AVkcFEScriptObjStringUTF8@@\0")
    }
    fn named_table(&self, type_name: &[u8]) -> Option<usize> {
        self.named_table_slots(type_name, 4)
    }
    fn named_table_slots(&self, type_name: &[u8], slots: usize) -> Option<usize> {
        let mut tables = BTreeSet::new();
        for name in self.occurrences(type_name) {
            let descriptor = name.checked_sub(8)?;
            for field in self.references(descriptor) {
                let Some(locator) = field.checked_sub(12) else {
                    continue;
                };
                if self.word(locator) != Some(0)
                    || self.word(locator + 4) != Some(0)
                    || self.word(locator + 8) != Some(0)
                    || self
                        .word(locator + 16)
                        .and_then(|p| self.read(p, 16))
                        .is_none()
                {
                    continue;
                }
                for reference in self.references(locator) {
                    let table = reference + 4;
                    if (0..slots)
                        .all(|slot| self.word(table + slot * 4).is_some_and(|p| self.code(p)))
                    {
                        tables.insert(table);
                    }
                }
            }
        }
        (tables.len() == 1).then(|| *tables.first().unwrap())
    }
    fn handler(&self, name: &str, next: &str) -> Option<Vec<Instruction>> {
        let mut padded = [0u8; 16];
        padded[..name.len()].copy_from_slice(name.as_bytes());
        let mut handlers = BTreeSet::new();
        for at in self.occurrences(&padded) {
            if self.read(at + 20, next.len()) != Some(next.as_bytes()) {
                continue;
            }
            if let Some(handler) = self.word(at + 16).filter(|p| self.code(*p)) {
                handlers.insert(handler);
            }
        }
        if handlers.len() != 1 {
            return None;
        }
        let instructions = self.instructions(*handlers.first()?, 4096)?;
        if !returns(&instructions, 16) {
            return None;
        }
        Some(instructions)
    }
    fn commands(&self, name: &str, next: &str) -> Option<Vec<usize>> {
        let instructions = self.handler(name, next)?;
        let mut methods = BTreeSet::new();
        for window in instructions.windows(4) {
            let [address, receiver, argument, call] = window else {
                unreachable!()
            };
            if address.mnemonic() == M::Lea
                && address.memory_base() == R::EBP
                && receiver.mnemonic() == M::Mov
                && receiver.op0_register() == R::ECX
                && receiver.op1_kind() == OpKind::Register
                && argument.mnemonic() == M::Push
                && argument.op0_register() == address.op0_register()
                && call.mnemonic() == M::Call
                && call.op0_kind() == OpKind::NearBranch32
                && self.code(call.near_branch_target() as usize)
            {
                methods.insert(call.near_branch_target() as usize);
            }
        }
        (!methods.is_empty() && methods.len() <= 8).then(|| methods.into_iter().collect())
    }
    fn display_methods(&self, append: usize, setter: &[Instruction]) -> Option<(usize, usize)> {
        let marker_handler = self.handler("apendmark", "clear")?;
        let finish_handler = self.handler("skip", "refex")?;
        let calls = |code: &[Instruction]| -> Vec<usize> {
            code.iter()
                .filter(|i| i.mnemonic() == M::Call && i.op0_kind() == OpKind::NearBranch32)
                .map(|i| i.near_branch_target() as usize)
                .collect()
        };
        let mut found = BTreeSet::new();
        for marker in calls(&marker_handler) {
            let Some(body) = self.instructions(marker, 64) else {
                continue;
            };
            // The marker command must call this object's verified append method
            // with the exact terminal marker, rather than an arbitrary control string.
            if body.len() != 8
                || body[0].mnemonic() != M::Push
                || body[0].op0_register() != R::EBP
                || body[1].mnemonic() != M::Mov
                || body[1].op0_register() != R::EBP
                || body[1].op1_register() != R::ESP
                || body[2].mnemonic() != M::Push
                || body[2].op0_register() != R::ECX
                || body[3].mnemonic() != M::Push
                || body[3].op0_kind() != OpKind::Immediate32
                || self.read(body[3].immediate32() as usize, 3) != Some(&[255, 255, 0])
                || body[4].mnemonic() != M::Call
                || body[4].op0_kind() != OpKind::NearBranch32
                || body[4].near_branch_target() as usize != append
                || body[5].mnemonic() != M::Mov
                || body[5].op0_register() != R::ESP
                || body[5].op1_register() != R::EBP
                || body[6].mnemonic() != M::Pop
                || body[6].op0_register() != R::EBP
                || body[7].mnemonic() != M::Ret
                || body[7].op_count() != 0
            {
                continue;
            }
            let end = body.last()?.next_ip() as usize;
            for offset in 0..32 {
                let show = end + offset;
                let Some(code) = self.instructions(show, 64) else {
                    continue;
                };
                let Some((limit, dirty, glyphs)) = show_members(&code) else {
                    continue;
                };
                if !setter.iter().any(|i| {
                    i.mnemonic() == M::Mov
                        && i.op0_kind() == OpKind::Memory
                        && i.memory_displacement64() as usize == glyphs
                        && i.op1_kind() == OpKind::Immediate32
                        && i.immediate32() == u32::MAX
                }) {
                    continue;
                }
                for finish in calls(&finish_handler) {
                    if finish <= show || finish - show > 256 {
                        continue;
                    }
                    let Some(code) = self.instructions(finish, 16) else {
                        continue;
                    };
                    if code.len() == 2
                        && immediate_member(&code[0], 1).is_some_and(|member| {
                            member != limit && member != dirty && member != glyphs
                        })
                        && code[1].mnemonic() == M::Ret
                        && code[1].op_count() == 0
                    {
                        found.insert((show, finish));
                    }
                }
            }
        }
        (found.len() == 1).then(|| *found.first().unwrap())
    }
    pub fn discover(&self) -> Option<Shape> {
        self.discover_utf8().or_else(|| self.discover_classic())
    }
    fn discover_utf8(&self) -> Option<Shape> {
        let table = self.table()?;
        let history = self
            .named_table(b".?AVkcFEScriptObjLogStringUTF8@@\0")
            .and_then(|table| {
                let tick = self.word(table + 8)?;
                returns(&self.instructions(tick, 4096)?, 4).then_some((table, tick))
            });
        let deleting = self.instructions(self.word(table)?, 128)?;
        if !returns(&deleting, 4) {
            return None;
        }
        let destroy = deleting
            .iter()
            .find(|i| i.mnemonic() == M::Call && i.op0_kind() == OpKind::NearBranch32)?
            .near_branch_target() as usize;
        // The complete destructor writes this class's vtable before releasing its buffer.
        let destroy_bytes = self.read(destroy, 256)?;
        let destroy_code: Vec<_> =
            Decoder::with_ip(32, destroy_bytes, destroy as u64, DecoderOptions::NONE)
                .into_iter()
                .take(64)
                .collect();
        if !destroy_code.iter().any(|i| {
            i.mnemonic() == M::Mov
                && i.op0_kind() == OpKind::Memory
                && i.memory_displacement64() == 0
                && i.op1_kind() == OpKind::Immediate32
                && i.immediate32() as usize == table
        }) {
            return None;
        }
        let tick = self.word(table + 8)?;
        if !returns(&self.instructions(tick, 4096)?, 4) {
            return None;
        }
        let setters = self.commands("str", "reset")?;
        let appenders = self.commands("apend", "apendmark")?;
        let mut shapes = Vec::new();
        for append in appenders {
            let Some(body) = self.instructions(append, 1024) else {
                continue;
            };
            if !returns(&body, 4) {
                continue;
            }
            // A nearby getter confirms the same independently allocated character buffer.
            let end = body.last()?.next_ip() as usize;
            for skip in 0..32 {
                let Some(bytes) = self.read(end + skip, 10) else {
                    continue;
                };
                let getter: Vec<_> = Decoder::with_ip(32, bytes, 0, DecoderOptions::NONE)
                    .into_iter()
                    .take(2)
                    .collect();
                if getter.len() != 2
                    || getter[0].mnemonic() != M::Mov
                    || getter[0].op0_register() != R::EAX
                    || getter[0].op1_kind() != OpKind::Memory
                    || getter[0].memory_base() != R::ECX
                    || getter[0].memory_index() != R::None
                    || getter[1].mnemonic() != M::Ret
                    || (getter[1].op_count() != 0 && getter[1].immediate16() != 0)
                {
                    continue;
                }
                let member = getter[0].memory_displacement64() as usize;
                if !(4..=8192).contains(&member) || !member.is_multiple_of(4) {
                    continue;
                }
                if !destroy_code.iter().any(|i| {
                    i.mnemonic() == M::Mov
                        && i.op1_kind() == OpKind::Memory
                        && i.memory_displacement64() as usize == member
                }) {
                    continue;
                }
                if !buffer_method(&body, member) {
                    continue;
                }
                for setter in &setters {
                    let Some(set) = self.instructions(*setter, 1024) else {
                        continue;
                    };
                    if buffer_method(&set, member) {
                        let Some((show, finish)) = self.display_methods(append, &set) else {
                            continue;
                        };
                        shapes.push(Shape {
                            profile: Profile::Utf8,
                            table,
                            setter: *setter,
                            append,
                            destroy,
                            tick,
                            text_member: member,
                            show,
                            finish,
                            history,
                        });
                    }
                }
            }
        }
        shapes.sort_by_key(|s| (s.setter, s.append, s.text_member));
        shapes.dedup();
        (shapes.len() == 1).then(|| shapes[0])
    }
}

fn immediate_member(i: &Instruction, value: u32) -> Option<usize> {
    let member = i.memory_displacement64() as usize;
    (i.mnemonic() == M::Mov
        && i.op0_kind() == OpKind::Memory
        && i.memory_base() == R::ECX
        && i.memory_index() == R::None
        && i.op1_kind() == OpKind::Immediate32
        && i.immediate32() == value
        && (4..=8192).contains(&member)
        && member.is_multiple_of(4))
    .then_some(member)
}
fn show_members(c: &[Instruction]) -> Option<(usize, usize, usize)> {
    if c.len() != 8
        || !returns(c, 4)
        || c[0].mnemonic() != M::Push
        || c[0].op0_register() != R::EBP
        || c[1].mnemonic() != M::Mov
        || c[1].op0_register() != R::EBP
        || c[1].op1_register() != R::ESP
        || c[2].mnemonic() != M::Mov
        || c[2].op0_register() != R::EAX
        || c[2].op1_kind() != OpKind::Memory
        || c[2].memory_base() != R::EBP
        || c[2].memory_displacement64() != 8
        || c[2].memory_index() != R::None
        || c[3].mnemonic() != M::Mov
        || c[3].op0_kind() != OpKind::Memory
        || c[3].memory_base() != R::ECX
        || c[3].memory_index() != R::None
        || c[3].op1_register() != R::EAX
        || c[6].mnemonic() != M::Pop
        || c[6].op0_register() != R::EBP
    {
        return None;
    }
    let limit = c[3].memory_displacement64() as usize;
    let dirty = immediate_member(&c[4], 1)?;
    let glyphs = immediate_member(&c[5], 1)?;
    ((4..=8192).contains(&limit)
        && limit.is_multiple_of(4)
        && limit != dirty
        && limit != glyphs
        && dirty != glyphs)
        .then_some((limit, dirty, glyphs))
}

fn returns(code: &[Instruction], bytes: u16) -> bool {
    code.last()
        .is_some_and(|i| i.mnemonic() == M::Ret && i.op_count() == 1 && i.immediate16() == bytes)
}
fn buffer_method(code: &[Instruction], member: usize) -> bool {
    returns(code, 4)
        && code.iter().any(|i| {
            i.mnemonic() == M::Mov
                && i.op0_kind() == OpKind::Memory
                && i.memory_displacement64() as usize == member
                && i.op1_register() == R::EAX
        })
        && code.iter().any(|i| {
            i.mnemonic() == M::Mov
                && i.op0_kind() == OpKind::Memory
                && matches!(i.op1_register(), R::AL | R::CL)
        })
        && code
            .iter()
            .any(|i| i.mnemonic() == M::Jne && i.near_branch_target() < i.ip())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unrelated_or_truncated_images_do_not_activate() {
        for size in [0, 8, 128, 1024] {
            assert!(Image {
                base: 0x10000,
                bytes: vec![0; size],
                executable: vec![]
            }
            .discover()
            .is_none());
        }
    }
    #[test]
    fn buffer_contract_rejects_wrong_stack_cleanup() {
        let bytes = [0xc2, 8, 0];
        let code: Vec<_> = Decoder::new(32, &bytes, DecoderOptions::NONE)
            .into_iter()
            .collect();
        assert!(!returns(&code, 4));
        assert!(!buffer_method(&code, 16));
    }
}
