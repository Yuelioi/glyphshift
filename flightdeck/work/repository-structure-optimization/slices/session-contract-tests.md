# Session 合同测试结构

Status: Complete

## Goal

保持 18 项 Session 公开 interface 合同和 fake port 行为不变，把 1274 行单文件按 publication、activation、
generation、health/loss、stop/release、multi-session/adapter removal 场景组织。

## Baseline

- `tests/session_contract.rs` 1274 行；前 394 行与 441-507 行是共享 fake ports/fixture，其余为 18 项测试。
- 生产 Session 已拆为 Contract/Manager，本切片不修改生产代码或测试断言。

## Plan

- [x] 根合同文件保留共享 fake ports/fixture，让场景子模块作为后代读取私有 fixture。
- [x] 拆分 publication、activation、generation、health、stop、isolation 六类场景。
- [x] 核对 18 个测试名集合并运行完整 Session 合同。
- [x] 运行 Clippy、格式、架构和 diff；自审无重复 fixture、断言和生产耦合。

## Result

- 根合同文件从 1274 行降至 472 行，只保留共享 fake ports、fixture 与显式场景 module。
- Activation 233 行、Generation 147 行、Health 106 行、Stop 162 行、Isolation 125 行、Publication 47 行。
- 所有场景 module 是根 integration test crate 的后代，直接复用私有 fixture，没有 `pub(crate)` 扩张。

验证通过：18/18 Session 合同；原/新测试函数名集合 18 项完全一致；package Clippy `-D warnings`、
Rust 格式、架构检查与 `git diff --check` 通过。本切片只移动测试，未重复执行上一生产切片已覆盖的下游。

Standards 自审确认没有重复 fake port、最大文件 472 行、场景按状态机事件而非内部类型组织。Spec 自审
确认生产代码零改动，18 项测试体与断言原样迁移；publication、activation、generation、loss/health、
stop/release、多 session 和 adapter removal 合同均保持覆盖。

## Decisions

- fixture 留在 integration test crate 根，避免为兄弟 module 扩大 `pub(crate)` 可见性。
- 按状态机事件序列拆场景，不按 fake host 类型拆测试。
