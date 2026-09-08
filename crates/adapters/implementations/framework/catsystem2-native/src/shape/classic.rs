//! Optimized x86 register convention. Every method is tied back to the
//! engine's command table and RTTI; this is deliberately a narrow profile.
use super::*;

fn member(i: &Instruction, base: R, destination: bool) -> Option<usize> {
    let offset = i.memory_displacement64() as usize;
    (i.mnemonic() == M::Mov
        && i.op_kind(if destination { 0 } else { 1 }) == OpKind::Memory
        && i.memory_base() == base
        && i.memory_index() == R::None
        && (4..=8192).contains(&offset)
        && offset.is_multiple_of(4))
    .then_some(offset)
}
fn call(i: &Instruction) -> Option<usize> {
    (i.mnemonic() == M::Call && i.op0_kind() == OpKind::NearBranch32)
        .then(|| i.near_branch_target() as usize)
}
fn ret0(c: &[Instruction]) -> bool {
    c.last()
        .is_some_and(|i| i.mnemonic() == M::Ret && i.op_count() == 0)
}
fn copies(c: &[Instruction], base: R, offset: usize) -> bool {
    c.iter()
        .any(|i| member(i, base, true) == Some(offset) && i.op1_register() == R::EAX)
        && c.iter().any(|i| {
            i.mnemonic() == M::Mov
                && i.op0_kind() == OpKind::Memory
                && matches!(i.op1_register(), R::AL | R::CL)
        })
        && c.iter()
            .any(|i| i.mnemonic() == M::Jne && i.near_branch_target() < i.ip())
}
impl Image {
    pub(super) fn discover_classic(&self) -> Option<Shape> {
        let table = self.named_table_slots(b".?AVkcFEScriptObjString@@\0", 3)?;
        let deleting = self.instructions(self.word(table)?, 128)?;
        if !returns(&deleting, 4)
            || !deleting.iter().any(|i| {
                i.mnemonic() == M::Mov && i.op0_register() == R::EDI && i.op1_register() == R::ECX
            })
        {
            return None;
        }
        let destroy = deleting.iter().find_map(call)?;
        let destruction: Vec<_> = Decoder::with_ip(
            32,
            self.read(destroy, 128)?,
            destroy as u64,
            DecoderOptions::NONE,
        )
        .into_iter()
        .take_while(|i| !matches!(i.mnemonic(), M::Ret | M::Jmp | M::Int3))
        .collect();
        if !destruction.iter().any(|i| {
            i.mnemonic() == M::Mov
                && i.op0_kind() == OpKind::Memory
                && i.memory_base() == R::EDI
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
        let str_command = self.handler("str", "reset")?;
        let append_command = self.handler("apend", "apendmark")?;
        let marker = self.handler("apendmark", "clear")?;
        let skip = self.handler("skip", "refex")?;
        let setters: Vec<_> = str_command
            .windows(2)
            .filter_map(|w| {
                (w[0].mnemonic() == M::Lea
                    && w[0].op0_register() == R::ESI
                    && matches!(w[0].memory_base(), R::ESP | R::EBP))
                .then(|| call(&w[1]))
                .flatten()
            })
            .collect();
        let appends: Vec<_> = append_command
            .windows(4)
            .filter_map(|w| {
                (w[0].mnemonic() == M::Lea
                    && matches!(w[0].memory_base(), R::ESP | R::EBP)
                    && w[1].mnemonic() == M::Push
                    && w[1].op0_register() == w[0].op0_register()
                    && w[2].mnemonic() == M::Mov
                    && w[2].op0_register() == R::EBX
                    && w[2].op1_register() == R::EAX)
                    .then(|| call(&w[3]))
                    .flatten()
            })
            .collect();
        let mut found = Vec::new();
        for setter in setters {
            let Some(set) = self.instructions(setter, 1024) else {
                continue;
            };
            if !ret0(&set) {
                continue;
            }
            for text_member in destruction.iter().filter_map(|i| member(i, R::EDI, false)) {
                if !copies(&set, R::EDI, text_member) {
                    continue;
                }
                for &append in &appends {
                    let Some(body) = self.instructions(append, 1024) else {
                        continue;
                    };
                    if !returns(&body, 4)
                        || !copies(&body, R::EBX, text_member)
                        || !marker.windows(2).any(|w| {
                            w[0].mnemonic() == M::Push
                                && w[0].op0_kind() == OpKind::Immediate32
                                && self.read(w[0].immediate32() as usize, 2) == Some(&[255, 0])
                                && call(&w[1]) == Some(append)
                        })
                    {
                        continue;
                    }
                    // Optimized display helpers precede the append implementation.
                    // Validate every instruction and cross-check the glyph/skip fields.
                    for show in append.saturating_sub(2048)..append {
                        let Some(c) = self.instructions(show, 32) else {
                            continue;
                        };
                        if c.len() != 5
                            || !ret0(&c)
                            || c[0].op1_register() != R::ECX
                            || c[1].mnemonic() != M::Mov
                            || c[1].op0_register() != R::ECX
                            || c[1].op1_kind() != OpKind::Immediate32
                            || c[1].immediate32() != 1
                            || c[2].op1_register() != R::ECX
                            || c[3].op1_register() != R::ECX
                        {
                            continue;
                        }
                        let (Some(limit), Some(dirty), Some(glyphs)) = (
                            member(&c[0], R::EAX, true),
                            member(&c[2], R::EAX, true),
                            member(&c[3], R::EAX, true),
                        ) else {
                            continue;
                        };
                        if limit == dirty
                            || limit == glyphs
                            || dirty == glyphs
                            || !set.iter().any(|i| {
                                member(i, R::EDI, true) == Some(glyphs)
                                    && i.op1_kind() == OpKind::Immediate32
                                    && i.immediate32() == u32::MAX
                            })
                        {
                            continue;
                        }
                        for finish in show.saturating_sub(128)..show {
                            let Some(f) = self.instructions(finish, 16) else {
                                continue;
                            };
                            if f.len() != 2
                                || !ret0(&f)
                                || f[0].op1_kind() != OpKind::Immediate32
                                || f[0].immediate32() != 1
                            {
                                continue;
                            }
                            let Some(flag) = member(&f[0], R::EAX, true) else {
                                continue;
                            };
                            if [limit, dirty, glyphs, text_member].contains(&flag)
                                || !skip.iter().any(|i| {
                                    member(i, R::EAX, true) == Some(flag)
                                        && i.op1_kind() == OpKind::Immediate32
                                        && i.immediate32() == 1
                                })
                            {
                                continue;
                            }
                            found.push(Shape {
                                profile: Profile::Classic,
                                table,
                                setter,
                                append,
                                destroy,
                                tick,
                                text_member,
                                show,
                                finish,
                                history: None,
                            });
                        }
                    }
                }
            }
        }
        found.sort_by_key(|s| (s.setter, s.append, s.show, s.finish, s.text_member));
        found.dedup();
        (found.len() == 1).then(|| found[0])
    }
}
