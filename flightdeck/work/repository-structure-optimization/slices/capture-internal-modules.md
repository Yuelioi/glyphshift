# Capture 内部 module 与测试 seam

Status: Complete

## Goal

保持 batch wire schema、非阻塞 ingress、drop/gap 证据、catalog schema、双槽 checkpoint 和恢复语义不变，
把 Observation 合同、热路径 Batch Producer 与单写者 Catalog Sink 按所有权拆开，并移出 397 行内嵌测试。

## Baseline

- `src/lib.rs` 1339 行，其中生产实现 941 行、内嵌测试 397 行；生产本身未超过 1200 行。
- 生产实现包含三条独立变化原因：Observation Batch 合同/游标、热路径有界 producer、持久 Catalog Sink。
- 10 项内嵌测试覆盖 wire validation、replay/gap/drop、pause/lifetime、去重、容量、checkpoint 与恢复。

## Plan

- [x] 提取 Observation 合同：配置、record/batch、schema、decode/validate 和 cursor。
- [x] 提取 Batch Producer：非阻塞 ingress、有界队列、排序 drain、pause 与 owner lifetime。
- [x] 提取 Catalog Sink：单写者 ingress、builder、双槽 checkpoint、恢复与 JSON catalog。
- [x] 按 Observation/Batch/Catalog seam 拆分内嵌测试，保留公开 interface 断言。
- [x] 运行 Capture/Workspace/直接下游回归、Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审公开 API、schema、drop/gap、checkpoint revision 与恢复语义。

## Result

- 根 `lib.rs` 从 1339 行降为 15 行，只保留四个私有 module 与公开 re-export。
- `observation.rs` 342 行：配置、record/batch wire、schema/size validation、cursor 与共享 ingress 状态。
- `batch.rs` 143 行：热路径非阻塞 ingress、有界 producer、排序 drain、pause 和 owner lifetime。
- `catalog.rs` 465 行：catalog/builder、单写者 ingress、双槽 checkpoint、恢复与完成协议。
- 397 行内嵌测试拆为 Observation 118 行、Batch 84 行、Catalog 188 行；Workspace 自有 5 项测试未混入。

验证通过：Capture 底层 10/10、Workspace 5/5；11 个直接下游 package 的确定性测试全部通过，授权
实机合同保持 ignored；package Clippy `-D warnings`、Rust 格式、架构检查与 `git diff --check` 通过。

Standards 自审确认模块按所有权而非类型数量形成，最大文件 465 行，根没有业务转发层。Spec 自审将
Observation、Batch、Catalog 三段生产实现规范化后与 `HEAD` 逐段比较完全一致；公开类型/方法/常量
多重集一致，`glyphshift.capture-observation-batch/1` 与 `glyphshift.capture-catalog/2` 未变，drop/gap、
双槽 revision 和恢复由原 10 项公开行为测试覆盖。

## Decisions

- 397 行测试足以触发迁移；生产拆分则由三种所有权模型触发，不以 941 行本身作为理由。
- `CaptureIngressStatus` 是两类 ingress 的共同公开结果，归入 Observation 合同而非复制枚举。
- `workspace.rs` 保持独立：它编排一次 Capture Workspace，不拥有底层 wire 或 checkpoint 实现。
