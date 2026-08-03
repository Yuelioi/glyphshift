# 软件删除入口与失败反馈

## Goal

让单个软件可以从行级操作直接删除，并让批量删除的成功、被引用拒绝和其他失败都产生准确、可恢复
的界面结果。

## Current

已完成。软件表提供行级删除，单项和批量操作共享确认流程。未被引用的软件从最新后端快照移除；
被 Workflow 或 Probe Run 引用的软件保留并在表内显示具体数量与解除方法，失败行继续保持选择。
Tauri 在触碰 Runtime 前完成引用检查，并以 `software.referenced` 和类型化计数保留失败语义。

## Decisions

- 单项删除与批量删除复用同一个确认 Modal 和后端命令，不建立第二套删除流程。
- 被 Workflow 或 Probe Run 引用的软件保留，明确显示引用数量和解除方法，不级联删除资产。
- 只有实际从最新快照消失的行才从选中集合移除；删除失败的行继续保留选择，便于用户处理后重试。
- 引用检查必须发生在停止 Runtime 前，拒绝删除不能改变现有运行状态。

## Verification

- 红回归先稳定捕获缺少行级删除、失败无反馈和错误语义丢失；相同回归实现后全部转绿。
- 完整 Playwright 44/44、Desktop Shell 34/34、生产构建、Clippy、fmt 和 UI detector 通过。
- Playwright 覆盖行级删除、批量删除和引用拒绝；Rust 覆盖 Workflow + Probe 引用在 Runtime 清理前
  被拒绝，以及无引用记录完成 Runtime 清理后删除。

## Next

- 返回[工作流 Runtime 错误反馈](workflow-runtime-error-feedback.md)，修复 `AlreadyActive` 误报与多进程
  目标选择。
