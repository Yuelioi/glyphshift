# 收缩观察器与划词翻译

Status: Finished

## Goal

两个仅观察 Adapter 暂时不再被产品列出、默认选择或运行使用，但其实现源码保留；划词翻译从桌面产品、
命令合同和测试中完整移除，不留下不可达入口或半失效状态。

## Current

两个仅观察 Adapter 已从产品 Catalog、目标兼容表、工作流/探针选择和 Runtime Bundle 注册中撤下；
产品以 `TextReplace` 能力过滤可用 Adapter，观察器实现源码与通用 Runtime seam 保留。旧工作流引用
未知或已停用 Adapter 时会自动撤销启用状态，不再阻断应用启动。

划词翻译的标题栏入口、面板、悬浮窗、F9 快捷键、桌面命令、产品编排 crate、UIA/OCR 打包工件、文案
和专属测试已删除。旧设置中的快捷键字段只做反序列化兼容并在下次写入时丢弃。

## Next

None.

## Progress

- 已在干净基线提交后独立开始本 Work。
- 已确认两个观察器的共同能力边界，并枚举划词翻译从 UI 到 Bundle 的完整产品依赖面。
- 已按能力过滤产品 Adapter，并生成 9 个写回 Adapter、0 个独立观察 Worker、0 个取词 Worker 的
  Runtime Bundle。
- 已删除划词翻译产品实现和桌面命令，保留通用 acquisition 与观察器实现源码。
- 前端生产构建、Desktop Shell 62 项合同、6 条真实 Runtime 启动合同通过；Playwright 全套先通过
  64 项，修正 3 条过时合同后逐项通过，设置/帮助专项 6 项再次通过。
- 新版开发窗口已使用更新后的 Runtime Bundle 启动。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
