# 探针错误状态隔离

Status: Finished

## Goal

探针操作失败只影响发起操作的探针；用户离开探针 A 或切换到探针 B 时，A 的错误提示自动关闭，A 的
迟到异步结果也不能污染 B。

## Current

根因已确认：探针 composable 的 `message` 与 `lastError` 是模块级共享状态，而 `selectRun` 过去只更新
当前 ID。现在离开或切换 Probe Run 会同步清理旧反馈；带 `runId` 的异步操作在报告失败前还会核对当前
选择，从而丢弃 A 在切到 B 后才返回的迟到错误。页面级加载失败仍保留页面级语义。

## Next

None

## Progress

- 已从完成的写回诊断 Work 独立出错误状态生命周期问题。
- Playwright 红灯精确复现 B 标题已显示但仍存在一个 A 的 `alert`；修复后同一用例覆盖常规切换与迟到
  异步失败，连续两次通过。
- 探针启动软件、管理员拒绝恢复、错误隔离、工作流占用 4 项相邻合同通过；前端 TypeScript 与生产构建
  通过。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
