//! Conservative compiler-shape checks for standard VGUI control operations.
//! Member offsets and virtual positions are recovered from cooperating methods.
use iced_x86::{Decoder, DecoderOptions, Instruction, Mnemonic as M, OpKind, Register as R};

fn body(code: &[u8]) -> Option<Vec<Instruction>> {
    let mut d = Decoder::new(32, code, DecoderOptions::NONE);
    let mut out = Vec::new();
    for _ in 0..160 {
        let i = d.decode();
        if i.is_invalid() {
            return None;
        }
        let end = i.mnemonic() == M::Ret;
        out.push(i);
        if end {
            return Some(out);
        }
    }
    None
}
fn load(i: &Instruction, dst: R, base: R) -> Option<usize> {
    (i.mnemonic() == M::Mov
        && i.op0_register() == dst
        && i.op1_kind() == OpKind::Memory
        && i.memory_base() == base
        && i.memory_index() == R::None
        && i.memory_size().size() == 4)
        .then_some(i.memory_displacement32() as usize)
}
fn unique<T>(mut values: impl Iterator<Item = T>) -> Option<T> {
    let first = values.next()?;
    values.next().is_none().then_some(first)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageShape {
    pub text: usize,
    pub symbol: usize,
    pub setter: usize,
}

/// Wide setters must contain the guarded token reset, UTF-16 length scan,
/// allocation/copy path, and recalculate flag. Pair with a read-only wide getter.
pub fn image(methods: &[Vec<u8>]) -> Option<ImageShape> {
    let mut found = Vec::new();
    for (setter, code) in methods.iter().enumerate() {
        let Some(b) = body(code) else { continue };
        if !crate::query_abi::cleanup_matches(code, 8) {
            continue;
        }
        if b.len() < 40
            || b[0].mnemonic() != M::Push
            || b[0].op0_register() != R::EBP
            || b[2].mnemonic() != M::Cmp
            || b[2].memory_base() != R::EBP
            || b[2].memory_displacement32() != 12
            || b[2].memory_size().size() != 1
            || b[2].immediate8() != 0
        {
            continue;
        }
        let Some(symbol) = unique(b.iter().filter_map(|i| {
            (i.mnemonic() == M::Mov
                && i.op0_kind() == OpKind::Memory
                && i.memory_base() == R::ESI
                && i.memory_index() == R::None
                && i.memory_size().size() == 4
                && i.op1_kind() == OpKind::Immediate32
                && i.immediate32() == u32::MAX)
                .then_some(i.memory_displacement32() as usize)
        })) else {
            continue;
        };
        let Some(text) = unique(b.iter().filter_map(|i| load(i, R::EDX, R::ESI))) else {
            continue;
        };
        if text >= 512
            || symbol >= 512
            || !text.is_multiple_of(4)
            || !symbol.is_multiple_of(4)
            || text == symbol
        {
            continue;
        }
        let copy = b.windows(6).any(|w| {
            w[0].mnemonic() == M::Movzx
                && w[0].op0_register() == R::EAX
                && w[0].memory_base() == R::ECX
                && w[0].memory_size().size() == 2
                && w[1].mnemonic() == M::Mov
                && w[1].memory_base() == R::EDX
                && w[1].op1_register() == R::AX
                && w[2].mnemonic() == M::Add
                && w[2].op0_register() == R::ECX
                && w[2].immediate8to32() == 2
                && w[3].mnemonic() == M::Add
                && w[3].op0_register() == R::EDX
                && w[3].immediate8to32() == 2
                && w[4].mnemonic() == M::Test
                && w[4].op0_register() == R::AX
                && w[5].mnemonic() == M::Jne
        });
        let stores_pointer = b.iter().any(|i| {
            i.mnemonic() == M::Mov
                && i.op0_kind() == OpKind::Memory
                && i.memory_base() == R::ESI
                && i.memory_displacement32() as usize == text
                && i.op1_register() == R::EAX
        });
        let recalc = b.iter().any(|i| {
            i.mnemonic() == M::Or
                && i.op0_kind() == OpKind::Memory
                && i.memory_base() == R::ESI
                && i.memory_size().size() == 1
                && i.immediate8() == 1
        });
        let token_getter = methods
            .iter()
            .any(|m| crate::panel_abi::member_getter(m) == Some(symbol));
        let wide_getter = methods.iter().any(|m| {
            let Some(g) = body(m) else { return false };
            g.windows(6).any(|w| {
                load(&w[0], R::ECX, R::ESI) == Some(text)
                    && w[1].mnemonic() == M::Shr
                    && w[1].op0_register() == R::EAX
                    && w[1].immediate8() == 1
                    && w[2].mnemonic() == M::Push
                    && w[2].op0_register() == R::EAX
                    && w[3].mnemonic() == M::Push
                    && w[3].op0_register() == R::ECX
                    && w[4].mnemonic() == M::Push
                    && w[4].op0_register() == R::EDI
                    && w[5].mnemonic() == M::Call
            }) && crate::query_abi::cleanup_matches(m, 8)
        });
        if copy && stores_pointer && recalc && token_getter && wide_getter {
            found.push(ImageShape {
                text,
                symbol,
                setter,
            });
        }
    }
    unique(found.into_iter())
}

/// Label's wide setter forwards both arguments to the discovered TextImage,
/// followed by hotkey, layout and repaint virtual calls. No naked image writes.
pub fn label_setter(methods: &[Vec<u8>], member: usize, image: ImageShape) -> Option<usize> {
    unique(methods.iter().enumerate().filter_map(|(slot, code)| {
        let b = body(code)?;
        if !crate::query_abi::cleanup_matches(code, 8) {
            return None;
        }
        let at = b.windows(8).position(|w| {
            load(&w[0], R::ECX, R::ESI) == Some(member)
                && load(&w[1], R::EAX, R::EBP) == Some(12)
                && load(&w[2], R::EDX, R::ECX) == Some(0)
                && load(&w[3], R::EDX, R::EDX) == Some(image.setter * 4)
                && w[4].mnemonic() == M::Push
                && w[4].op0_register() == R::EDI
                && w[5].mnemonic() == M::Push
                && w[5].op0_register() == R::EAX
                && w[6].mnemonic() == M::Push
                && w[6].op0_register() == R::EBX
                && w[7].mnemonic() == M::Call
                && w[7].op0_register() == R::EDX
        })?;
        // The target compiler's standard tail includes four owner virtual calls.
        let tail = &b[at + 8..];
        let calls = tail
            .iter()
            .filter(|i| i.mnemonic() == M::Call)
            .collect::<Vec<_>>();
        if calls.len() != 4
            || calls.iter().any(|i| i.op0_kind() != OpKind::Register)
            || !b.iter().any(|i| load(i, R::EBX, R::EBP) == Some(8))
            || !b
                .iter()
                .any(|i| load(i, R::EAX, R::ECX) == Some(image.text))
        {
            return None;
        }
        Some(slot)
    }))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handles {
    pub count: usize,
    pub entries: usize,
    pub panel_slot: usize,
}

/// Recognize the serial/index/free-bit checked IVGui handle table. The fields
/// below are read only; neither this classifier nor its consumer calls resolver code.
pub fn handles(methods: &[Vec<u8>]) -> Option<Handles> {
    let panel_slot = unique(methods.iter().filter_map(|m| {
        let b = body(m)?;
        if b.len() != 10 || !crate::query_abi::cleanup_matches(m, 4) {
            return None;
        }
        if load(&b[2], R::ECX, R::EBP) != Some(8)
            || b[3].mnemonic() != M::Test
            || b[4].mnemonic() != M::Je
            || load(&b[5], R::EAX, R::ECX) != Some(0)
            || b[7].mnemonic() != M::Call
            || b[7].op0_register() != R::EDX
        {
            return None;
        }
        let offset = load(&b[6], R::EDX, R::EAX)?;
        (offset < 256 && offset.is_multiple_of(4)).then_some(offset / 4)
    }))?;
    let (count, entries) = unique(methods.iter().filter_map(|m| {
        let b = body(m)?;
        if !crate::query_abi::cleanup_matches(m, 4) || b.len() != 40 {
            return None;
        }
        let sequence = [
            M::Push,
            M::Mov,
            M::Push,
            M::Push,
            M::Push,
            M::Mov,
            M::Cmp,
            M::Je,
            M::Mov,
            M::Mov,
            M::And,
            M::Cmp,
            M::Jae,
            M::Mov,
            M::Mov,
            M::Mov,
            M::Mov,
            M::Shr,
            M::And,
            M::Cmp,
            M::Jne,
            M::And,
            M::Cmp,
            M::Je,
            M::Cmp,
            M::Jae,
            M::Mov,
            M::Mov,
            M::And,
            M::Cmp,
            M::Jne,
            M::And,
            M::Cmp,
            M::Je,
            M::Mov,
            M::Pop,
            M::Pop,
            M::Pop,
            M::Pop,
            M::Ret,
        ];
        if !b.iter().zip(sequence).all(|(i, m)| i.mnemonic() == m) {
            return None;
        }
        let invalid = b[7].near_branch_target();
        if invalid < b.last()?.next_ip()
            || ![12, 20, 23, 25, 30, 33]
                .iter()
                .all(|n| b[*n].near_branch_target() == invalid)
        {
            return None;
        }
        if ![11, 24]
            .iter()
            .all(|n| b[*n].op0_register() == R::EAX && b[*n].op1_register() == R::EBX)
            || ![19, 29]
                .iter()
                .all(|n| b[*n].op0_register() == R::EDI && b[*n].op1_register() == R::ECX)
            || ![22, 32]
                .iter()
                .all(|n| b[*n].op0_register() == R::EDX && b[*n].immediate32() == 0x80000000)
            || ![14, 26].iter().all(|n| {
                b[*n].op0_register() == R::EDX
                    && b[*n].memory_base() == R::ESI
                    && b[*n].memory_index() == R::EAX
                    && b[*n].memory_index_scale() == 8
                    && b[*n].memory_displacement32() == 0
            })
        {
            return None;
        }
        let count = load(&b[8], R::EBX, R::ECX)?;
        let entries = load(&b[13], R::ESI, R::ECX)?;
        let mask = b.iter().any(|i| {
            i.mnemonic() == M::And && i.op0_register() == R::EAX && i.immediate32() == 0xfffff
        });
        let serial = b
            .iter()
            .any(|i| i.mnemonic() == M::Shr && i.op0_register() == R::ECX && i.immediate8() == 20);
        let used = b
            .iter()
            .filter(|i| {
                i.mnemonic() == M::And
                    && i.op0_register() == R::EDX
                    && i.immediate32() == 0x80000000
            })
            .count()
            == 2;
        let generations = b
            .iter()
            .filter(|i| {
                i.mnemonic() == M::And
                    && i.op0_register() == R::EDI
                    && i.immediate32() == 0x7fffffff
            })
            .count()
            == 2;
        let result = b.iter().any(|i| {
            i.mnemonic() == M::Mov
                && i.op0_register() == R::EAX
                && i.memory_base() == R::ESI
                && i.memory_index() == R::EAX
                && i.memory_index_scale() == 8
                && i.memory_displacement32() == 4
        });
        (mask
            && serial
            && used
            && generations
            && result
            && count < 512
            && entries < 512
            && count.is_multiple_of(4)
            && entries.is_multiple_of(4))
        .then_some((count, entries))
    }))?;
    Some(Handles {
        count,
        entries,
        panel_slot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolver(count: u32, entries: u32) -> Vec<u8> {
        // Synthetic serial-table resolver with relocatable object members.
        let mut out = vec![
            0x55, 0x8b, 0xec, 0x53, 0x56, 0x57, 0x8b, 0x7d, 8, 0x83, 0xff, 0xff,
        ];
        let mut branches = Vec::new();
        let branch = |out: &mut Vec<u8>, branches: &mut Vec<usize>, op: u8| {
            out.push(op);
            branches.push(out.len());
            out.push(0);
        };
        branch(&mut out, &mut branches, 0x74);
        out.extend([0x8b, 0x99]);
        out.extend(count.to_le_bytes());
        out.extend([0x8b, 0xc7, 0x25, 0xff, 0xff, 0x0f, 0, 0x3b, 0xc3]);
        branch(&mut out, &mut branches, 0x73);
        out.extend([0x8b, 0xb1]);
        out.extend(entries.to_le_bytes());
        out.extend([0x8b, 0x14, 0xc6, 0x8b, 0xcf, 0x8b, 0xfa, 0xc1, 0xe9, 20]);
        for pass in 0..2 {
            if pass != 0 {
                out.extend([0x8b, 0x14, 0xc6, 0x8b, 0xfa]);
            }
            out.extend([0x81, 0xe7, 0xff, 0xff, 0xff, 0x7f, 0x3b, 0xf9]);
            branch(&mut out, &mut branches, 0x75);
            out.extend([0x81, 0xe2, 0, 0, 0, 0x80, 0x81, 0xfa, 0, 0, 0, 0x80]);
            branch(&mut out, &mut branches, 0x74);
            if pass == 0 {
                out.extend([0x3b, 0xc3]);
                branch(&mut out, &mut branches, 0x73);
            }
        }
        out.extend([0x8b, 0x44, 0xc6, 4, 0x5f, 0x5e, 0x5b, 0x5d, 0xc2, 4, 0]);
        let invalid = out.len();
        out.extend([0x33, 0xc0, 0x5f, 0x5e, 0x5b, 0x5d, 0xc2, 4, 0]);
        for at in branches {
            out[at] = u8::try_from(invalid - at - 1).unwrap();
        }
        out
    }
    fn methods(count: u32, entries: u32, slot: usize) -> Vec<Vec<u8>> {
        let mut m = vec![vec![0xcc]; 8];
        m[slot] = vec![
            0x55, 0x8b, 0xec, 0x8b, 0x4d, 8, 0x85, 0xc9, 0x74, 10, 0x8b, 1, 0x8b, 0x50, 12, 0xff,
            0xd2, 0x5d, 0xc2, 4, 0, 0x83, 0xc8, 0xff, 0x5d, 0xc2, 4, 0,
        ];
        // The null branch starts after the non-null return.
        m[slot][9] = 11;
        m[7] = resolver(count, entries);
        m
    }
    #[test]
    fn derives_handle_layout_and_rejects_broken_serial_checks() {
        for (count, entries, slot) in [(16, 4, 2), (36, 20, 5)] {
            let mut m = methods(count, entries, slot);
            assert_eq!(
                handles(&m),
                Some(Handles {
                    count: count as usize,
                    entries: entries as usize,
                    panel_slot: 3
                })
            );
            let at = m[7].windows(2).position(|b| b == [0x81, 0xe7]).unwrap();
            m[7][at + 2] = 0xfe;
            assert!(handles(&m).is_none());
        }
        let mut m = methods(16, 4, 2);
        m[7][13] = 0;
        assert!(handles(&m).is_none());
    }
}
