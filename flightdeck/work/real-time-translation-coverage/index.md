# 通用实时翻译覆盖

Status: Open

## Goal

让用户完成稳定、可解释的端到端流程：选择软件，捕获原文，使用或编辑 Dictionary，并由真实
`TextReplace` Adapter 把译文立即写回目标界面。通用性按可复用文字技术扩展，不以“支持所有软件”
或成功注入代替真实翻译结果。

## Current

现有 GDI、USER32 DrawText 与 GDI+ Adapter 已能覆盖命中这些调用的经典 Windows 界面，并在一个授权
桌面工具的部分界面得到写回证据。Console 与 UI Automation 目前只有采集价值，不能作为实时翻译
成功。Direct2D `DrawText` 已通过合成写回，但在当前授权目标中零命中，因此尚未进入生产 Bundle。

此前运行时 Roadmap 将 Observation Stream、跨启动 Binding 和共享 Hook 提到当前主线；这些工作不能
增加可翻译软件数量，现已退回后续 Roadmap。尚未提交的 Observation Stream 实现已撤回，现有简单
Workflow/Probe 互斥继续保留。

## Next

- 完成[下一种实时写回 Adapter 选择](slices/next-writeback-adapter.md)：按现有 Bundle、真实证据与通用
  Windows 文字路径建立支持矩阵，选择一个有明确目标软件和可见写回验收的 Adapter，不先扩建观察
  平台。

## Progress

- 已重新确认产品主线是“探针发现原文 → Dictionary 提供译文 → Adapter 实时写回”，而不是通用观察
  数据平台。
- 已将采集能力与实时翻译能力分级；UIA、Console 等 observe-only 能力保留，但不计入实时翻译覆盖。

## References

- [产品契约](../../../PRODUCT.md)
- [领域语言](../../../CONTEXT.md)
- [Windows 软件支持分级](../runtime-capability-roadmap/references/windows-software-support-and-console-gap.md)
- [后续运行时能力 Roadmap](../runtime-capability-roadmap/index.md)
