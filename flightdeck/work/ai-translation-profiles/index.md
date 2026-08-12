# AI 翻译配置与一键翻译

Status: Finished

## Goal

为 Glyphshift 增加可复用的 AI Profile 与批量翻译能力：支持主流云端协议和 Ollama 等本地端点；词典与
探针可一键只翻译尚无译文的条目，并在请求前用可解释、可预览的规则过滤不应翻译的原文。

## Current

单批条目数与输入 Token 预算已成为 AppSettings 中的唯一全局策略，默认分别为 100 与 16,000。
Profile `/2` 不再保存这两个字段；桌面和浏览器任务在启动时读取同一策略，并按条目数与保守 Token
估算自动排空全部候选。设置页提供立即持久化的全局控件，Dictionary 与 Probe 预览显示全局值。

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
- 已确认“50”来自 Probe GUI 分页；真实任务已处理 98/98。新增 51 条分批合同与 52 条探针全量规划合同，
  验证分页、预览数量和单次请求大小均不限制任务总量。
- 任务快照新增总批次、完成批次与失败批次；Dictionary / Probe 显示完整进度、全量完成文案与部分失败重试。
- 限流会根据 `Retry-After` 或指数退避进行有界重试；最终失败保留其他批次成功结果，重新规划只选择空白项。
- 后续回归通过：AI Rust 15 项、桌面壳 70 项、完整 Playwright 78 项、生产构建、Rust 格式和架构检查。
- 用户确认默认单批改为 100，并新增探针翻译状态筛选需求；工作重新打开。
- 曾先把新建 Profile 默认单批改为 100；用户随后确认批次策略应统一管理，该中间方案已由全局
  AppSettings 策略取代，Profile 不再持有条目或 Token 限制。
- Probe 已支持全部、未翻译和已翻译筛选；5000 条合成数据在一条人工译文写入后正确得到 2499 条未翻译，
  再与 Adapter 条件组合得到 14 条，证明筛选发生在完整数据集且早于分页；迟到的旧查询不会覆盖新筛选。
- 本轮 Capture 18 项、AI 15 项、桌面壳 70 项、AI / Probe Playwright 23 项、生产构建、格式与架构检查通过；
  最新本地实例的真实 IPC 验证通过并停在“未翻译”视图。
- 用户确认单批条目数与 Token 预算应统一管理；全局 AppSettings 现默认 100 条与 16,000 输入 Token，
  Profile artifact 升为 `/2` 并移除旧限制字段。任务层使用显式 Batch Policy，以条目上限和保守 Token
  估算共同切批；切换任何 Profile 都不改变策略。
- 最终回归通过：AI Rust 14 项、桌面壳 71 项、Settings / AI Playwright 13 项、生产构建、Rust 格式、
  架构与界面反模式检查；最新本地实例真实 IPC 通过并停在全局 AI 批次设置页面。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [设计简报](references/design-brief.md)
- [Provider 协议调研](references/provider-protocols.md)
