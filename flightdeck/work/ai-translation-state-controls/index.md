# AI 翻译状态控件

Status: Finished

## Goal

让 AI 翻译的确认、冲突提示与批次详情都可理解、可收起、可关闭：确认前明确显示将使用的 AI 配置和
模型；“已有任务运行”提示可 dismiss；批次详情支持折叠，终态报告关闭后不会被后台同步立即重放。

## Current

确认 Modal 现在显示即将使用的 AI 配置、连接方式与模型；探针和词典的错误 Alert 都提供可访问关闭
动作；页面级批次详情默认收起、可展开，失败/重试时自动展开。关闭终态报告会按 job ID 记忆，同一
任务的后台轮询不再让它重现，新任务仍正常显示。

## Next

本 Work 无剩余实现。后续发布流程可复用本次 Playwright 合同，确保终态任务同步语义不会回退。

## Progress

- 已用两张用户截图确认问题覆盖 Dictionary/Probe 共用的 AI 确认与进度组件，以及页面级错误提示。
- 已定位现有 AI Playwright seam：确认 Modal、运行批次折叠、终态“关闭批次详情”均已有相邻场景。
- 红灯确认过缺少配置/模型、缺少折叠、同 job 轮询复活以及错误 Alert 不可关闭四类回归。
- 前端生产构建通过；AI/Probe Playwright 32/32 通过；960×640 深色界面的确认、折叠和展开状态通过
  Playwright 实际渲染检查。
- UI 机械检查无发现；同步桌面 Review 构建通过，Runtime Bundle 验证 9 个 Adapter 后成功启动。
- 未运行归档 UIA 测试。

## References

- [后台 AI 翻译任务](../background-ai-translation-tasks/index.md)
- [AI 翻译可观测性](../ai-translation-observability/index.md)
- [AI 翻译配置](../ai-translation-profiles/index.md)
