//! Conservative x86 query-shape proof. No target slots, addresses, object-field
//! offsets, instruction prefixes, or text are part of the profile.
//!
//! A name query must pass its unchanged argument and `this` to a virtual index
//! query, reject the invalid index, and compute the same wide-pool address as an
//! index-value query. Only the small instruction language below is accepted.
use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction, Mnemonic, OpKind, Register};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Atom {
    This,
    Argument,
    Index,
    Load(Box<Expr>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Expr {
    constant: u32,
    terms: BTreeMap<Atom, u32>,
}

impl Expr {
    fn number(n: u32) -> Self {
        Self {
            constant: n,
            terms: BTreeMap::new(),
        }
    }
    fn atom(atom: Atom) -> Self {
        Self {
            constant: 0,
            terms: BTreeMap::from([(atom, 1)]),
        }
    }
    fn add(mut self, rhs: Self) -> Self {
        self.constant = self.constant.wrapping_add(rhs.constant);
        for (term, coefficient) in rhs.terms {
            let value = self.terms.entry(term).or_default();
            *value = value.wrapping_add(coefficient);
        }
        self.terms.retain(|_, coefficient| *coefficient != 0);
        self
    }
    fn scale(mut self, coefficient: u32) -> Self {
        self.constant = self.constant.wrapping_mul(coefficient);
        for value in self.terms.values_mut() {
            *value = value.wrapping_mul(coefficient);
        }
        self.terms.retain(|_, value| *value != 0);
        self
    }
    fn virtual_slot(&self) -> Option<usize> {
        let [(Atom::Load(address), 1)] =
            self.terms.iter().map(|(a, b)| (a, *b)).collect::<Vec<_>>()[..]
        else {
            return None;
        };
        if self.constant != 0 || address.constant % 4 != 0 || address.constant >= 64 * 4 {
            return None;
        }
        (address.terms == BTreeMap::from([(Atom::Load(Box::new(Self::atom(Atom::This))), 1)]))
            .then_some(address.constant as usize / 4)
    }
    fn replace_index(&self) -> Self {
        self.terms.iter().fold(
            Self::number(self.constant),
            |result, (atom, coefficient)| {
                let atom = match atom {
                    Atom::Index => Atom::Argument,
                    Atom::Load(inner) => Atom::Load(Box::new(inner.replace_index())),
                    other => other.clone(),
                };
                result.add(Self::atom(atom).scale(*coefficient))
            },
        )
    }
    fn contains(&self, needle: &Atom) -> bool {
        self.terms.keys().any(|atom| {
            atom == needle || matches!(atom, Atom::Load(inner) if inner.contains(needle))
        })
    }
    fn wide_address(&self) -> bool {
        // A base pointer plus a UTF-16 offset read from an index-addressed entry.
        // Field offsets and entry stride are deliberately inferred, not fixed.
        self.constant == 0 && self.terms.len() == 2 && self.terms.iter().any(|(atom, coefficient)| {
            *coefficient == 2 && matches!(atom, Atom::Load(inner) if inner.contains(&Atom::Argument))
        }) && self.terms.iter().any(|(atom, coefficient)| {
            *coefficient == 1 && matches!(atom, Atom::Load(inner) if inner.contains(&Atom::This) && !inner.contains(&Atom::Argument))
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Value {
    Unknown,
    Frame(i32),
    Expression(Expr),
}
impl Value {
    fn expr(&self) -> Option<Expr> {
        if let Self::Expression(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }
    fn add(self, rhs: Self) -> Option<Self> {
        match (self, rhs) {
            (Self::Expression(a), Self::Expression(b)) => Some(Self::Expression(a.add(b))),
            (Self::Frame(at), Self::Expression(n)) | (Self::Expression(n), Self::Frame(at))
                if n.terms.is_empty() =>
            {
                Some(Self::Frame(at.wrapping_add(n.constant as i32)))
            }
            _ => None,
        }
    }
}

fn reg(register: Register) -> Option<usize> {
    match register {
        Register::EAX => Some(0),
        Register::ECX => Some(1),
        Register::EDX => Some(2),
        Register::EBX => Some(3),
        Register::ESP => Some(4),
        Register::EBP => Some(5),
        Register::ESI => Some(6),
        Register::EDI => Some(7),
        _ => None,
    }
}

#[derive(Clone)]
struct State {
    registers: [Value; 8],
    stack: BTreeMap<i32, Value>,
    lookup: Option<usize>,
    invalid: Option<bool>,
    compares_invalid: bool,
    steps: usize,
    visited: BTreeSet<usize>,
}
impl State {
    fn new() -> Self {
        let mut registers = std::array::from_fn(|_| Value::Unknown);
        registers[1] = Value::Expression(Expr::atom(Atom::This));
        registers[4] = Value::Frame(0);
        Self {
            registers,
            stack: BTreeMap::from([(4, Value::Expression(Expr::atom(Atom::Argument)))]),
            lookup: None,
            invalid: None,
            compares_invalid: false,
            steps: 0,
            visited: BTreeSet::new(),
        }
    }
    fn address(&self, instruction: &Instruction) -> Option<Value> {
        if !matches!(
            instruction.segment_prefix(),
            Register::None | Register::DS | Register::SS
        ) {
            return None;
        }
        let base = if instruction.memory_base() == Register::None {
            Value::Expression(Expr::default())
        } else {
            self.registers[reg(instruction.memory_base())?].clone()
        };
        let index = if instruction.memory_index() == Register::None {
            Expr::default()
        } else {
            self.registers[reg(instruction.memory_index())?]
                .expr()?
                .scale(instruction.memory_index_scale())
        };
        base.add(Value::Expression(
            index.add(Expr::number(instruction.memory_displacement32())),
        ))
    }
    fn operand(&self, instruction: &Instruction, operand: u32) -> Option<Value> {
        match instruction.op_kind(operand) {
            OpKind::Register => {
                Some(self.registers[reg(instruction.op_register(operand))?].clone())
            }
            OpKind::Immediate32 => Some(Value::Expression(Expr::number(instruction.immediate32()))),
            OpKind::Immediate8to32 => Some(Value::Expression(Expr::number(
                instruction.immediate8to32() as u32,
            ))),
            OpKind::Memory if instruction.memory_size().size() == 4 => {
                match self.address(instruction)? {
                    Value::Frame(at) => self.stack.get(&at).cloned(),
                    Value::Expression(address) => {
                        Some(Value::Expression(Expr::atom(Atom::Load(Box::new(address)))))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
    fn set(&mut self, instruction: &Instruction, value: Value) -> Option<()> {
        if instruction.op0_kind() == OpKind::Register {
            self.registers[reg(instruction.op0_register())?] = value;
        } else if instruction.op0_kind() == OpKind::Memory && instruction.memory_size().size() == 4
        {
            let Value::Frame(at) = self.address(instruction)? else {
                return None;
            };
            self.stack.insert(at, value);
        } else {
            return None;
        }
        Some(())
    }
    fn push(&mut self, value: Value) -> Option<()> {
        let Value::Frame(at) = self.registers[4] else {
            return None;
        };
        let at = at.checked_sub(4)?;
        self.registers[4] = Value::Frame(at);
        self.stack.insert(at, value);
        Some(())
    }
    fn pop(&mut self) -> Option<Value> {
        let Value::Frame(at) = self.registers[4] else {
            return None;
        };
        self.registers[4] = Value::Frame(at.checked_add(4)?);
        self.stack.remove(&at)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Query {
    lookup: Option<usize>,
    address: Expr,
}

fn query(code: &[u8]) -> Option<Query> {
    let mut pending = vec![(0usize, State::new())];
    let mut success = None;
    let mut failure = false;
    let mut total = 0;
    while let Some((mut ip, mut state)) = pending.pop() {
        loop {
            state.steps += 1;
            total += 1;
            if state.steps > 96 || total > 256 || ip >= code.len() || !state.visited.insert(ip) {
                return None;
            }
            let op = Decoder::with_ip(32, &code[ip..], ip as u64, DecoderOptions::NONE).decode();
            if op.is_invalid() {
                return None;
            }
            match op.mnemonic() {
                Mnemonic::Mov => {
                    let value = state.operand(&op, 1)?;
                    state.set(&op, value)?;
                }
                Mnemonic::Lea => {
                    let value = state.address(&op)?;
                    state.set(&op, value)?;
                }
                Mnemonic::Push => {
                    let value = state.operand(&op, 0)?;
                    state.push(value)?;
                }
                Mnemonic::Pop => {
                    let value = state.pop()?;
                    state.set(&op, value)?;
                }
                Mnemonic::Leave => {
                    state.registers[4] = state.registers[5].clone();
                    state.registers[5] = state.pop()?;
                }
                Mnemonic::Add | Mnemonic::Sub => {
                    let left = state.operand(&op, 0)?;
                    let mut right = state.operand(&op, 1)?.expr()?;
                    if op.mnemonic() == Mnemonic::Sub {
                        right = right.scale(u32::MAX);
                    }
                    state.set(&op, left.add(Value::Expression(right))?)?;
                    state.compares_invalid = false;
                }
                Mnemonic::Imul if op.op_count() == 3 => {
                    let right = state.operand(&op, 2)?.expr()?;
                    if !right.terms.is_empty() {
                        return None;
                    }
                    state.set(
                        &op,
                        Value::Expression(state.operand(&op, 1)?.expr()?.scale(right.constant)),
                    )?;
                    state.compares_invalid = false;
                }
                Mnemonic::Shl
                    if op.op0_kind() == OpKind::Register && op.op1_kind() == OpKind::Immediate8 =>
                {
                    let shift = op.immediate8();
                    if shift > 5 {
                        return None;
                    }
                    state.set(
                        &op,
                        Value::Expression(state.operand(&op, 0)?.expr()?.scale(1u32 << shift)),
                    )?;
                    state.compares_invalid = false;
                }
                Mnemonic::Xor
                    if op.op0_kind() == OpKind::Register
                        && op.op1_kind() == OpKind::Register
                        && op.op0_register() == op.op1_register() =>
                {
                    state.set(&op, Value::Expression(Expr::default()))?;
                    state.compares_invalid = false;
                }
                Mnemonic::Cmp => {
                    let index = if state.lookup.is_some() {
                        Atom::Index
                    } else {
                        Atom::Argument
                    };
                    if state.operand(&op, 0)?.expr()? != Expr::atom(index)
                        || state.operand(&op, 1)?.expr()? != Expr::number(u32::MAX)
                    {
                        return None;
                    }
                    state.compares_invalid = true;
                }
                Mnemonic::Je | Mnemonic::Jne => {
                    if !state.compares_invalid
                        || state.invalid.is_some()
                        || op.op0_kind() != OpKind::NearBranch32
                    {
                        return None;
                    }
                    let jump_invalid = op.mnemonic() == Mnemonic::Je;
                    let mut taken = state.clone();
                    taken.invalid = Some(jump_invalid);
                    taken.compares_invalid = false;
                    pending.push((op.near_branch32() as usize, taken));
                    state.invalid = Some(!jump_invalid);
                    state.compares_invalid = false;
                }
                Mnemonic::Jmp if op.op0_kind() == OpKind::NearBranch32 => {
                    ip = op.near_branch32() as usize;
                    continue;
                }
                Mnemonic::Call if op.flow_control() == FlowControl::IndirectCall => {
                    if state.lookup.is_some() || state.invalid.is_some() {
                        return None;
                    }
                    let slot = state.operand(&op, 0)?.expr()?.virtual_slot()?;
                    if state.registers[1].expr()? != Expr::atom(Atom::This)
                        || state.pop()?.expr()? != Expr::atom(Atom::Argument)
                    {
                        return None;
                    }
                    state.lookup = Some(slot);
                    state.registers[0] = Value::Expression(Expr::atom(Atom::Index));
                    state.registers[1] = Value::Unknown;
                    state.registers[2] = Value::Unknown;
                    state.compares_invalid = false;
                }
                Mnemonic::Ret => {
                    if op.op0_kind() != OpKind::Immediate16
                        || op.immediate16() != 4
                        || state.registers[4] != Value::Frame(0)
                    {
                        return None;
                    }
                    let value = state.registers[0].expr()?;
                    if state.invalid? {
                        if value != Expr::default() {
                            return None;
                        }
                        failure = true;
                    } else {
                        let value = value.replace_index();
                        if !value.wide_address() {
                            return None;
                        }
                        let found = Query {
                            lookup: state.lookup,
                            address: value,
                        };
                        if success.as_ref().is_some_and(|previous| previous != &found) {
                            return None;
                        }
                        success = Some(found);
                    }
                    break;
                }
                Mnemonic::Nop => {}
                _ => return None,
            }
            ip = op.next_ip() as usize;
        }
    }
    if failure {
        success
    } else {
        None
    }
}

/// Checks all reachable returns without invoking an unknown method. This is
/// calling-convention evidence only; query pairing supplies the semantic gate.
pub fn cleanup_matches(code: &[u8], expected: u16) -> bool {
    let mut pending = vec![0usize];
    let mut seen = BTreeSet::new();
    let mut returns = 0;
    while let Some(ip) = pending.pop() {
        if !seen.insert(ip) {
            continue;
        }
        if ip >= code.len() || seen.len() > 512 {
            return false;
        }
        let op = Decoder::with_ip(32, &code[ip..], ip as u64, DecoderOptions::NONE).decode();
        if op.is_invalid() {
            return false;
        }
        match op.flow_control() {
            FlowControl::Return => {
                if op.mnemonic() != Mnemonic::Ret
                    || op.op0_kind() != OpKind::Immediate16
                    || op.immediate16() != expected
                {
                    return false;
                }
                returns += 1;
            }
            FlowControl::UnconditionalBranch | FlowControl::ConditionalBranch
                if op.op0_kind() == OpKind::NearBranch32 =>
            {
                pending.push(op.near_branch32() as usize);
                if op.flow_control() == FlowControl::ConditionalBranch {
                    pending.push(op.next_ip() as usize);
                }
            }
            FlowControl::Next | FlowControl::Call | FlowControl::IndirectCall => {
                pending.push(op.next_ip() as usize)
            }
            _ => return false,
        }
    }
    returns > 0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuerySlots {
    pub find: usize,
    pub index: usize,
    pub value: usize,
}

pub fn identify(methods: &[Vec<u8>]) -> Option<QuerySlots> {
    if methods.len() > 64 {
        return None;
    }
    let queries = methods.iter().map(|code| query(code)).collect::<Vec<_>>();
    let mut result = None;
    for (find, candidate) in queries.iter().enumerate() {
        let Some(Query {
            lookup: Some(index),
            address,
        }) = candidate
        else {
            continue;
        };
        if *index == find || !cleanup_matches(methods.get(*index)?, 4) {
            continue;
        }
        for (value, candidate) in queries.iter().enumerate() {
            if value == find || value == *index {
                continue;
            }
            if candidate.as_ref()
                != Some(&Query {
                    lookup: None,
                    address: address.clone(),
                })
            {
                continue;
            }
            if result.is_some() {
                return None;
            }
            result = Some(QuerySlots {
                find,
                index: *index,
                value,
            });
        }
    }
    result
}
