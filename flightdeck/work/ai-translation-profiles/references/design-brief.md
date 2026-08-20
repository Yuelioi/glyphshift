# AI Profiles 与一键翻译：设计简报

Status: Implemented

## 产品结果

- 用户可以创建多个 AI Profile，选择云端供应商、OpenAI-compatible 网关或 Ollama 本地端点。
- Dictionary 和 Probe 都提供一个真正的一键入口：使用默认 Profile 和已保存规则，只补全当前没有译文的条目。
- “只补空白”是不可关闭的安全约束，不是一个容易误触的复选框；任务运行期间由用户填写的译文也不会被迟到的 AI 响应覆盖。
- 过滤器在请求发送前运行，支持预览命中数量、跳过原因和样例；纯数字、计数器、尺寸/帧率、符号、单字符和包含数字等规则均可独立配置。
- 批量任务可以显示进度、取消、保留成功批次并只重试失败或仍待翻译的条目。

## 信息架构与关键交互

### AI Profiles

在 Settings 增加独立的 `AI Profiles` 子页面，而不是把多套连接信息塞进当前单列设置表单。页面保持现有桌面管理工具的高密度语言：上方工具栏、Profile 表格、选中项编辑区。

每个 Profile 展示名称、协议、模型、端点类型、默认状态和最近一次连接测试结果。编辑字段分为：

1. 基本信息：名称、协议、默认 Profile。
2. 连接：Base URL、认证方式、凭据状态、模型 ID；模型列表只是辅助选择，手工模型 ID 始终可用。
3. 请求策略：超时、单批条数/字符数、并发；Ollama 默认并发为 1。
4. 高级能力：结构化输出模式、模型发现、兼容端点能力覆盖和受限的自定义 Header。

连接测试分开报告网络/认证、模型发现、模型调用和结构化输出，避免一个兼容端点没有 `/models` 就被误判为完全不可用。会产生模型调用费用的探测必须显式标注。

该初版凭据建议已被后续用户决策替代：API Key 随 Profile `/4` 明文保存在本机配置文件中，编辑界面默认
遮蔽并支持显式显示。日志、任务记录、词典和错误信息仍不得复制 Key。本机 loopback HTTP 可用于无密钥
Ollama；远程明文 HTTP 携带 Key 时继续拒绝。

### Dictionary / Probe 的一键入口

两个表格工具栏使用相同的 split-button：

- 主按钮 `AI 补全`：用默认 Profile 与已保存过滤规则立即规划并开始任务。
- 下拉项 `预览并翻译…`：先打开紧凑的计划面板，显示“可翻译 / 已有译文 / 被规则过滤 / 已忽略”，允许临时调整规则或切换 Profile。

任务开始后，工具栏原位显示 `已完成 n / 总数 · 跳过 n · 失败 n`、取消和“重试剩余项”，不使用占满屏幕的向导。没有默认 Profile 时，主按钮进入 Profile 创建流程；候选为零时直接解释原因，不发送网络请求。

Dictionary 中的成功结果合并到当前编辑草稿并标记为未保存，最终仍由用户点击 Save。Probe 中的成功结果逐批写入其绑定 Dictionary，并沿用现有实时预览刷新路径。

## 过滤与文本安全

### 不可关闭的资格条件

- 原文 trim 后不能为空。
- 译文 trim 后必须为空；已有任何有效译文都不会进入请求。
- Probe 中被忽略的 observation 不进入请求。
- 写回时再次比较原文、目标项状态和版本；任何已经被用户或其他任务修改的项都跳过。

### 推荐默认过滤预设

- 开启：纯数字/符号、数字型计数器与度量值（例如比例、分辨率、帧率）、URL、电子邮件、文件路径、纯快捷键。
- 开启但可关闭：单字符文本。
- 默认关闭、用户可开启：包含任意数字、超过指定长度、已经像目标语言、用户正则表达式。
- 相同原文在一次任务中去重，只请求一次。

过滤计划返回稳定的 `SkipReason`，不能只返回一个总数。预览按原因分组并给出少量样例，用户可以理解为什么 `0`、`0/4`、分辨率、帧率或单字符没有发送给模型。

占位符与格式标记不应简单过滤掉：`%s`、`%1`、`{name}`、`${value}`、转义序列和富文本标签需要提取、在提示中保护、响应后逐项校验。Qt mnemonic `&` 单独按语义校验，允许位置变化但不允许无意丢失或增加转义标记。校验失败的结果进入失败列表，不写回 Dictionary。

## Provider 范围

建议首批协议适配器：

- OpenAI Responses（OpenAI 原生首选）
- OpenAI Chat Completions
- OpenAI-compatible Chat / Responses 方言
- Anthropic Messages
- Gemini `generateContent`
- Ollama native `/api/chat`

Azure OpenAI 在 Profile 数据模型中预留 deployment、API version 与认证类型，建议第二阶段实现。OpenAI 官方建议新项目优先 Responses；不同供应商的结构化输出、流事件、错误体和模型发现并不等价，因此每种 wire protocol 保留独立 Adapter。详见[协议调研](provider-protocols.md)。

## 深模块与接口边界

`AiTranslation` 是独立的深 Module；Dictionary、Probe 和界面不接触厂商 JSON 字段：

```text
Dictionary draft ----\
                      > CandidatePlanner -> TranslationCoordinator -> ProviderAdapter
Probe observations --/          |                   |                       |
                                 |                   |                       +-- cloud / Ollama
                                 |                   +-- batches / retry / cancel / validation
                                 +-- eligible items / skip reasons

validated results -> DictionaryDraftAdapter | ProbeDictionaryAdapter
```

对外 Interface 只表达产品语义：

```text
plan_translation(scope_snapshot, filter_policy) -> TranslationPlan
start_translation(plan_token, profile_id) -> TranslationJobId
translation_job(job_id) -> TranslationJobSnapshot
cancel_translation(job_id) -> CancellationOutcome
retry_translation(job_id, failed_or_pending) -> TranslationJobId
```

关键领域对象：

- `AiProfile`：连接、认证引用、模型与协议能力，不持有词典内容。
- `FilterPolicy`：确定性资格规则与用户排除规则。
- `TranslationPlan`：输入快照指纹、候选项、跳过原因和预计批次；过期计划不可启动。
- `TranslationJob`：批次状态、进度、失败和取消状态，不等同于供应商的云 Batch 资源。
- `TranslationBatch`：稳定 opaque item ID、原文、语言、可选上下文和受保护 token。
- `TranslationResult`：经过结构、ID 对齐、非空、语言和 token 守恒校验的候选译文。
- `ProviderAdapter`：唯一理解具体 HTTP endpoint、认证 Header、JSON/SSE/NDJSON、错误形状和能力降级的 Seam。

Profile 元数据、系统凭据和运行时 Job 分开存放；更换 Provider Adapter 不改变 Dictionary 或 Probe 的持久化格式。

## 即时任务语义

1. 统一规划器先执行“仅空白”和过滤规则，再按 item 数与字符预算切小批。
2. 优先严格 JSON Schema，依能力降级为 JSON mode 和 prompt-only JSON；所有模式都校验 item ID 无新增、遗漏或重复。
3. 云端默认低并发，Ollama 默认并发 1；429/503 遵守 Retry-After 并带抖动退避。
4. 每个成功小批独立合并。Probe 使用版本前置条件写入；Dictionary 使用草稿快照做本地 compare-and-set。
5. 取消只承诺停止本地排批、等待和写回，并丢弃迟到结果；不声称云端立即停止生成或计费。
6. 首版不使用供应商的异步 Batch API。其完成窗口可达约 24 小时，不符合点击后的即时反馈；未来可作为单独的“后台大词典翻译”能力。

## 已确认的产品决策

1. Dictionary 正式允许 source-only 待翻译条目持久化；Runtime 仅发布 completed 条目。
2. 一键主按钮直接执行，旁路“预览并翻译”承载临时选项。
3. 首发采用上述六种协议，Azure OpenAI 留待后续阶段。
