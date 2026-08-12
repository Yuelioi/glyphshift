# AI 翻译配置与一键翻译

Status: Finished

## Goal

为 Glyphshift 增加可复用的 AI Profile 与批量翻译能力：支持主流云端协议和 Ollama 等本地端点；词典与
探针可一键只翻译尚无译文的条目，并在请求前用可解释、可预览的规则过滤不应翻译的原文。

## Current

AI Profile、六种首发协议、Windows 凭据隔离、Dictionary / Probe 一键补全、可解释过滤、任务进度与
取消、结构化结果校验及写回保护均已实现。真实供应商连通性仍取决于用户配置的端点、模型和凭据；本轮
只用合成 Transport 验证 wire contract，界面提供会明确提示可能产生费用的连接测试。

## Next

None

## Progress

- 已建立长期 Work，并把真实网络调用、凭据安全、幂等写入和可恢复批处理列为首轮设计约束。
- 已用官方资料完成 OpenAI、Anthropic、Gemini、Ollama 与 Azure OpenAI 协议矩阵。
- 已确认 Dictionary 草稿与 Probe 直接写入不能共享同一种提交动作，但可以共享候选规划、Provider 和任务状态机。
- 已完成待确认的产品、界面、深模块和即时任务设计简报。
- 用户已确认全部推荐方案，包括底层 Dictionary pending 契约。
- Dictionary 已升级为 `/3`：pending 条目可保存，Runtime 与 Probe 发布快照只包含 completed 条目。
- `AiTranslation` 已覆盖 OpenAI Responses、OpenAI Chat、OpenAI-compatible、Anthropic Messages、
  Gemini generateContent 与 Ollama native，并统一处理批次、重试、取消、结构与占位符校验。
- Settings 已提供多 Profile 管理、系统凭据保存和分阶段连接测试；Dictionary 与 Probe 已提供只补空白、
  过滤预览、进度、取消及 compare-and-set 写回。
- 相关 Rust 回归全部通过：AI 11 项、Capture 17 项、Desktop Backend 9 项、Desktop Shell 70 项、
  Dictionary 5 项；架构合同通过。
- 前端生产构建通过；AI 与 Settings 定向 Playwright 10 项通过，完整桌面 Playwright 75 项通过；
  界面反模式检测无发现。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [设计简报](references/design-brief.md)
- [Provider 协议调研](references/provider-protocols.md)
