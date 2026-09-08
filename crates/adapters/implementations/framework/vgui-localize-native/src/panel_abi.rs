//! Read-only x86 forwarding proofs for a panel callback boundary.
//!
//! Table positions and member offsets are outputs, never profile constants.
use iced_x86::{Decoder, DecoderOptions, Mnemonic, OpKind, Register};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Unknown,
    Frame(i32),
    This,
    Arg(u8),
    Result(usize),
    Load(Box<Value>, u32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Call {
    this: Value,
    slot: usize,
    args: Vec<Value>,
}

#[derive(Debug)]
struct Forward {
    calls: Vec<Call>,
    cleanup: u16,
}

fn slot(target: &Value, this: &Value) -> Option<usize> {
    let Value::Load(table, offset) = target else {
        return None;
    };
    if **table != Value::Load(Box::new(this.clone()), 0) || offset % 4 != 0 || *offset >= 1024 {
        return None;
    }
    Some(*offset as usize / 4)
}

/// A pure getter can be used without calling foreign code.
pub fn member_getter(code: &[u8]) -> Option<usize> {
    let mut decoder = Decoder::new(32, code, DecoderOptions::NONE);
    let load = decoder.decode();
    if load.mnemonic() != Mnemonic::Mov
        || load.op0_register() != Register::EAX
        || load.op1_kind() != OpKind::Memory
        || load.memory_base() != Register::ECX
        || load.memory_index() != Register::None
        || load.memory_size().size() != 4
    {
        return None;
    }
    let offset = load.memory_displacement32() as usize;
    let end = decoder.decode();
    (end.mnemonic() == Mnemonic::Ret
        && end.op_count() == 0
        && offset < 4096
        && offset.is_multiple_of(4))
    .then_some(offset)
}

fn forward(code: &[u8]) -> Option<Forward> {
    let mut registers = BTreeMap::from([
        (Register::ECX, Value::This),
        (Register::ESP, Value::Frame(0)),
    ]);
    let mut stack = BTreeMap::new();
    for n in 0..5u8 {
        stack.insert(4 + i32::from(n) * 4, Value::Arg(n));
    }
    let mut decoder = Decoder::new(32, code, DecoderOptions::NONE);
    let mut calls = Vec::new();
    let mut base = None;
    for _ in 0..48 {
        let op = decoder.decode();
        if op.is_invalid() {
            return None;
        }
        let read = |index: u32| -> Option<Value> {
            match op.op_kind(index) {
                OpKind::Register => Some(
                    registers
                        .get(&op.op_register(index))
                        .cloned()
                        .unwrap_or(Value::Unknown),
                ),
                OpKind::Memory
                    if op.memory_index() == Register::None && op.memory_size().size() == 4 =>
                {
                    let value = registers.get(&op.memory_base())?.clone();
                    let offset = op.memory_displacement32();
                    Some(match value {
                        Value::Frame(at) => stack
                            .get(&at.wrapping_add(offset as i32))
                            .cloned()
                            .unwrap_or(Value::Unknown),
                        Value::Unknown => return None,
                        other => Value::Load(Box::new(other), offset),
                    })
                }
                _ => None,
            }
        };
        match op.mnemonic() {
            Mnemonic::Mov if op.op0_kind() == OpKind::Register => {
                let value = read(1)?;
                registers.insert(op.op0_register(), value);
            }
            Mnemonic::Push => {
                let value = read(0)?;
                let Value::Frame(at) = *registers.get(&Register::ESP)? else {
                    return None;
                };
                stack.insert(at - 4, value);
                registers.insert(Register::ESP, Value::Frame(at - 4));
                // The only preserved register in this deliberately small language.
                if op.op0_register() == Register::EBP && base.is_none() {
                    base = Some(at - 4);
                }
            }
            Mnemonic::Pop if op.op0_kind() == OpKind::Register => {
                let Value::Frame(at) = *registers.get(&Register::ESP)? else {
                    return None;
                };
                let value = stack.remove(&at)?;
                registers.insert(op.op0_register(), value);
                registers.insert(Register::ESP, Value::Frame(at + 4));
            }
            Mnemonic::Call => {
                let target = read(0)?;
                let this = registers.get(&Register::ECX)?.clone();
                let slot = slot(&target, &this)?;
                let Value::Frame(at) = *registers.get(&Register::ESP)? else {
                    return None;
                };
                let base = base.unwrap_or(0);
                if at > base || base - at > 16 {
                    return None;
                }
                let args = (at..base)
                    .step_by(4)
                    .map(|p| stack.get(&p).cloned())
                    .collect::<Option<Vec<_>>>()?;
                if args.iter().any(|v| !matches!(v, Value::Arg(_))) {
                    return None;
                }
                registers.insert(Register::EAX, Value::Result(calls.len()));
                registers.insert(Register::ECX, Value::Unknown);
                registers.insert(Register::EDX, Value::Unknown);
                registers.insert(Register::ESP, Value::Frame(base));
                calls.push(Call { this, slot, args });
                if calls.len() > 2 {
                    return None;
                }
            }
            Mnemonic::Ret if op.op0_kind() == OpKind::Immediate16 => {
                if registers.get(&Register::ESP) != Some(&Value::Frame(0)) {
                    return None;
                }
                return Some(Forward {
                    calls,
                    cleanup: op.immediate16(),
                });
            }
            Mnemonic::Nop => {}
            _ => return None,
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PanelBoundary {
    pub callback: usize,
    pub client_getter: usize,
    pub vpanel_client_getter: usize,
}

/// Requires a unique two-stage panel dispatch family, including sibling
/// callbacks and the helper's own unchanged-panel forwarding proof.
pub fn identify(methods: &[Vec<u8>]) -> Option<PanelBoundary> {
    if methods.len() > 96 {
        return None;
    }
    let forwards = methods.iter().map(|m| forward(m)).collect::<Vec<_>>();
    let mut answer = None;
    for (callback, f) in forwards.iter().enumerate() {
        let Some(f) = f else { continue };
        let [first, second] = f.calls.as_slice() else {
            continue;
        };
        if f.cleanup != 16
            || first.this != Value::This
            || first.args != [Value::Arg(0)]
            || second.this != Value::Result(0)
            || second.args != [Value::Arg(1), Value::Arg(2), Value::Arg(3)]
        {
            continue;
        }
        let Some(Some(helper)) = forwards.get(first.slot) else {
            continue;
        };
        let [getter] = helper.calls.as_slice() else {
            continue;
        };
        if helper.cleanup != 4 || getter.this != Value::Arg(0) || !getter.args.is_empty() {
            continue;
        }
        let siblings = forwards
            .iter()
            .flatten()
            .filter(
                |s| matches!(s.calls.as_slice(), [a,b] if a == first && b.this == Value::Result(0)),
            )
            .count();
        if siblings < 8 {
            continue;
        }
        if answer.is_some() {
            return None;
        }
        answer = Some(PanelBoundary {
            callback,
            client_getter: first.slot,
            vpanel_client_getter: getter.slot,
        });
    }
    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatch(helper: u32, target: u32, args: u8) -> Vec<u8> {
        let mut code = vec![0x55, 0x8b, 0xec, 0x8b, 0x01, 0x8b, 0x55, 0x08, 0x8b, 0x80];
        code.extend((helper * 4).to_le_bytes());
        code.extend([0x52, 0xff, 0xd0, 0x8b, 0xd0, 0x8b, 0x00]);
        // EDX keeps the client while EAX loads its destination method.
        code.extend([0x8b, 0x80]);
        code.extend((target * 4).to_le_bytes());
        for n in (0..args).rev() {
            code.extend([0xff, 0x75, 12 + n * 4]);
        }
        code.extend([0x8b, 0xca, 0xff, 0xd0, 0x5d, 0xc2, (args + 1) * 4, 0]);
        code
    }
    fn methods(helper: usize) -> Vec<Vec<u8>> {
        let mut out = vec![vec![0xcc]; 20];
        out[helper] = vec![
            0x55, 0x8b, 0xec, 0x8b, 0x4d, 8, 0x8b, 1, 0x8b, 0x90, 0x24, 0, 0, 0, 0xff, 0xd2, 0x5d,
            0xc2, 4, 0,
        ];
        for (n, method) in out.iter_mut().enumerate().take(9) {
            *method = dispatch(helper as u32, n as u32, if n == 5 { 3 } else { 0 });
        }
        out
    }
    #[test]
    fn derives_shifted_panel_and_client_slots() {
        for helper in [12, 17] {
            assert_eq!(
                identify(&methods(helper)),
                Some(PanelBoundary {
                    callback: 5,
                    client_getter: helper,
                    vpanel_client_getter: 9
                })
            );
        }
    }
    #[test]
    fn rejects_ambiguous_and_mutating_dispatches() {
        let mut m = methods(12);
        m[6] = dispatch(12, 6, 3);
        assert!(identify(&m).is_none());
        let mut m = methods(12);
        m[5].splice(0..0, [0x89, 0x01]);
        assert!(identify(&m).is_none());
    }
    #[test]
    fn derives_only_pure_aligned_member_getters() {
        assert_eq!(member_getter(&[0x8b, 0x81, 0x58, 1, 0, 0, 0xc3]), Some(344));
        assert_eq!(member_getter(&[0x8b, 0x41, 24, 0xc3]), Some(24));
        assert_eq!(member_getter(&[0x89, 0x41, 24, 0xc3]), None);
    }
}
