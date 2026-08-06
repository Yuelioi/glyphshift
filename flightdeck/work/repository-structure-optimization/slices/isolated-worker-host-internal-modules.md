# Isolated Worker Host 内部 module

Status: Complete

## Goal

保持公开 Worker/Host interface、进程协议、超时回收、重启预算、Capture generation 和混合 Placement
失败隔离不变，把单 Worker 传输、监督器、隔离 Host 与 Hybrid 编排按生命周期拆开。

## Baseline

- `src/lib.rs` 1479 行，混合了 artifact/catalog、JSONL 子进程、监督线程、重启策略、隔离 Host、
  Target Process + Isolated Worker Placement 编排与 3 项单元测试。
- `tests/process_contract.rs` 有 8 项公开 Host/进程合同；前一切片下游回归已确认 8/8 和 3/3 单元测试全绿。
- Supervisor 每 250ms drain/health，60 秒窗口最多重启 3 次；超时必须立即终止原进程。

## Plan

- [x] 提取 Worker 传输：artifact、JSONL round trip、health 解码、drain/deactivate 和强制终止。
- [x] 提取 Supervisor：命令通道、轮询、generation 分配、重启窗口和终态 health。
- [x] 提取 Isolated Host：target grant、worker ownership、Host 错误映射与 Capture ingress。
- [x] 提取 Hybrid Host：Placement 路由、部分激活、更新/诊断/停止/释放编排。
- [x] 迁移 3 项内部边界测试，并运行公开合同、下游回归、Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审公开 API、协议/错误映射、restart/timeout 和 Placement 失败隔离。

## Result

- 根 `lib.rs` 从 1479 行降为 17 行，只保留私有 module 与稳定公开 re-export。
- `worker.rs` 444 行：artifact/catalog、JSONL 协议、health 解码、drain/deactivate 和强制终止。
- `supervisor.rs` 280 行：命令通道、250ms 轮询、generation 分配、60 秒/3 次重启预算与终态。
- `isolated.rs` 276 行：target grant、每 target/adapter Worker 所有权、Capture ingress 与错误映射。
- `hybrid.rs` 452 行：Target Process/Isolated Worker Placement 的部分激活和完整 Session 生命周期。
- 3 项内部边界测试迁移到 57 行独立测试 module。

验证通过：内部测试 3/3、公开进程/重启合同 8/8；两个直接下游 package 的确定性测试通过，2 项授权
UIA 实机合同和 12 项 Desktop Runtime 实机合同仍正常 ignored；package Clippy `-D warnings`、Rust
格式、架构检查与 `git diff --check` 全部通过。

Standards 自审确认所有文件低于 1200 行，跨 module 只开放 crate 内所有权接口，未扩展 crate 公共 API。
Spec 自审将四段生产逻辑规范化后与 `HEAD` 对应源码逐段比较完全一致；唯一结构性改写是 Catalog 增加
crate 内 artifact 查询方法。公开类型/方法多重集一致，重启/超时、Worker code 限制、generation、错误
映射和 Placement 失败隔离由 11 项本 crate 测试及下游 Runtime 测试共同覆盖。

## Decisions

- Worker 传输只回答“一个进程如何安全对话并回收”；Supervisor 只回答“何时重启/停止”。
- Isolated Host 管 Worker 所有权与 Capture producer；Hybrid Host 管多个 placement 的 Session 语义。
- 不把 Worker SDK wire 类型或 Target Process Host 实现并入本 crate 的公共 surface。
