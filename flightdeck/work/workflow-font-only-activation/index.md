# 字体专用工作流启用误判

Status: Finished

## Goal

合法 Adapter 在无词典、`all_observations` 字体策略下可以启用，不再被误报为当前不可用；真正失效
的 Adapter 仍被拒绝。

## Current

字体专用工作流不再经过错误的二次 Adapter 产品可用性守卫；当前环境是否包含 Adapter 继续由
Workflow Resolver 统一判断。

## Next

None

## Progress

- 排除多目标遗漏和旧 Adapter ID；根因位于 `disabled_product_adapter_id` 对请求能力的误用。
- 后端合同先稳定复现合法 Adapter 被误报为 `workflow.unknown_adapter`，移除错误守卫后转绿。
- 新增反向合同，确认 Adapter 从当前 Desktop Environment 消失时仍返回 `workflow.unknown_adapter`。
- 桌面壳 65 项 Rust 测试和工作流页面 11 项 Playwright 回归通过。
- 开发桌面壳已重新编译并启动，当前运行实例包含修复。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
