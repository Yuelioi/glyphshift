# 运行时能力升级 Roadmap

Status: Open

## Goal

在不扩大 Dictionary 职责、不泄漏平台细节到核心匹配 Interface 的前提下，按真实软件证据逐步扩展
多进程接管、观察流复用、Windows 文字技术覆盖、原生隔离和布局适配能力。

## Current

Process Family Stage 已交付。Software Extension 现在可声明显式后代可执行文件 allowlist，并经
Desktop Runtime Spec、Controller Host 配置进入 Windows Controller；工作流和 Dictionary 不承担
进程关系。授权根仍优先使用用户选择的完整路径，后代必须同时命中 allowlist 且位于已授权进程树。

Windows Controller 使用 PID + 创建时间作为不跨 seam 的实例身份，输出稳定且永不复用的 opaque
token。合成 inventory 已覆盖嵌套后代、重排、根先退出、成员部分退出、身份不可得和 PID 复用；
退出实例的 Runtime 记录随之清理，仍存活实例继续可用。授权 AE 的本地快照证明一个根进程拥有
8 个后代、3 类辅助可执行文件；参数化实机合同发现 9 个目标实例。结论是首个生产 Process Family
应由独立 AE Software Extension 显式声明这些成员，Core 不增加品牌分支。

当前 Desktop Runtime 仍只选择 inventory 的第一个目标，并把 Controller Connection 转移给单个
SessionManager；Workflow 与 Probe 继续以软件级互斥避免重复注入。下一阶段要判断是否把它深化为
单一 Target Execution：最多一个 Publication Owner，允许多个只读 Observation Subscriber，共享
同一套注入和 Hook。

Target Execution 盘点已完成并否决立即增加租约注册表：SessionManager 本身已经能管理多 Session，
真正缺口是 Target Runtime 只有 activation-time 单一 FileCaptureSink，Runtime diagnostics 也是
取走最近批次的单消费者语义。没有稳定 Observation Stream Identity 与多游标读取，Owner/Subscriber
Module 只会搬运现有互斥状态，不能减少注入或提供安全共享。因此当前互斥保留，执行顺序调整为先
冻结 Observation Stream 合同，再回到共享执行。

## Next

- 冻结 Observation Stream Identity、样本历史、有界多游标读取与重启重匹配合同。从
  [当前切片](slices/observation-stream-identity.md)、
  [Target Runtime](../../../crates/glyphshift-target-runtime/src/lib.rs)和
  [Capture Module](../../../crates/glyphshift-capture/src/lib.rs)开始；不把 surface token 或调用地址
  直接升级为持久 Binding。

## Progress

- 已完成 AI 评审路由，建立采纳门槛和明确的非目标；Roadmap 按真实证据逐 Stage 推进。
- 已完成 LunaTranslator 产品与运行时一手资料调研，冻结 Observation Stream、Transform Profile
  及可选输入/输出能力的路由边界。
- Process Family Stage 完成：Extension → Runtime Spec → Controller 配置贯通显式后代 allowlist；
  合成生命周期与授权 AE 的 9 实例 inventory 合同通过，全 workspace、Clippy、fmt 与架构检查全绿。
- Target Execution seam 盘点完成：确认当前缺少的是可多读、可重匹配的 Observation Stream，而不是
  另一层租约 Map；在该前置合同完成前保留 Workflow/Probe 软件级互斥。

## References

- [稳定约束](context.md)
- [阶段计划](plan.md)
- [AI 评审路由结论](references/ai-review-assessment.md)
- [产品领域语言](../../../CONTEXT.md)
- [Capability Adapter 调研](../glyphshift/references/adapter-registry-and-cross-platform-interception-research.md)
- [LunaTranslator 产品与运行时调研](../glyphshift/references/lunatranslator-product-runtime-research.md)
- [Process Family Controller Inventory](slices/process-family-controller-inventory.md)
- [Target Execution 所有权](slices/target-execution-ownership.md)
- [Observation Stream Identity](slices/observation-stream-identity.md)
