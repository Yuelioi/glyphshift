# Protocol 内部 module

Status: Complete

## Goal

保持 Controller wire/model、transport trait、握手/nonce、inventory opaque ID、recipe 授权、取消与失败状态
不变，把无状态协议模型和有状态 Connection 按依赖方向组织。

## Baseline

- `src/lib.rs` 1107 行，无内嵌测试；前 617 行是 nonce、wire/domain model、inventory/recipe 值对象，
  后 490 行是 ControllerTransport、Connection、NonceLedger 和 degraded/terminated 失败处理。
- `tests/controller_contract.rs` 656 行，从公开 interface 覆盖 9 项握手、inventory、recipe、失败隔离与取消。

## Plan

- [x] 提取 model，并让 Connection 成为 model 的私有子模块，固定单向依赖。
- [x] 保持 crate 根公开 re-export 与所有协议字段私有性。
- [x] 运行 9 项合同、直接下游、Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审公开 surface、nonce replay、opaque ID、recipe violation 与失败状态。

## Result

- 根 `lib.rs` 从 1107 行降为 5 行，只私有声明 model 并稳定 re-export。
- `model.rs` 618 行，包含 nonce、wire/domain model、inventory/recipe 值对象，并私有拥有 Connection 子模块。
- `model/connection.rs` 492 行，包含 Transport trait、NonceLedger、Connection 状态和失败处理。
- Connection 作为 model 的子模块读取私有协议字段，无需增加任何 `pub(super)` 或公开 getter。

验证通过：Protocol 合同 9/9；5 个直接下游 package 的确定性测试全部通过，12 项授权 Desktop Runtime
合同保持 ignored；package Clippy `-D warnings`、Rust 格式、架构检查与 `git diff --check` 通过。

Standards 自审确认依赖只从 Connection 指向 Model，协议值对象未碎片化，所有文件低于 1200 行。Spec
自审将 Model 与 Connection 源码逐行和 `HEAD` 对应区段比较完全一致，公开类型/函数多重集一致；nonce
replay、opaque ID、recipe violation、cancel、degraded/terminated 由原 9 项合同和五个下游测试集覆盖。

## Decisions

- Connection 是协议模型的使用者，因此作为 model 子模块；model 不反向依赖连接状态。
- 不按每类 wire value 拆文件；它们共同定义 Controller Protocol 的一套语言。
