# Target Execution 所有权

Status: Deferred

## Outcome

一个目标实例只维护一套注入、Adapter Host 和 Hook；最多一个 Publication Owner 写入文字/字体，
多个 Observation Subscriber 只能消费同一份有界观察流，且任何订阅失败都不能提升写入能力或影响
其他订阅者。

## Decisions

- Target Execution 属于短期 Runtime actual state，不进入 Dictionary、Software metadata 或
  Workflow artifact。
- Publication Owner 与 Observation Subscriber 是执行租约，不是 Adapter Feature 或 UI 权限标签。
- 在并发价值和失败隔离被合同证明以前，不移除当前软件级 Workflow/Probe 互斥规则。

## Delivery

- [x] 盘点 Desktop Runtime、SessionManager、TargetProcessHost 与 Controller 中当前注入所有权。
- [ ] 冻结 acquire/update/release/subscribe 的最小 Interface 和稳定拒绝语义。
- [ ] 用合成目标证明一个 Owner、多 Subscriber 不重复注入且 Subscriber 不能发布。
- [ ] 验证 Owner/Subscriber 任一退出不破坏其他参与者，并保留有界观察与 dropped 语义。

## Current

Process Family 已让一个 Software 的多个目标实例可被 Controller 同时发现，但 Desktop Runtime 仍取
inventory 第一项并把 Controller Connection 移交单个 SessionManager。SessionManager 已能管理多个
Session，TargetProcessHost 也能登记多个 target；缺口不是再包一层所有权 Map。Target Runtime 当前
只有 deployment 携带的单一 FileCaptureSink，capture control 不带 subscriber identity，diagnostics
query 会取走最近批次。现在增加 Owner/Subscriber Interface 无法提供真实多读，只会复制
DesktopRuntimePool 已有的软件级互斥。

## Next

先完成 Observation Stream Identity、有界多游标读取和重启重匹配合同。具备真实可共享的数据源后，
再从本 Slice 恢复，证明 Owner/Subscriber 生命周期而不重复注入。
