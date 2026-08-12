# AI Translation Profiles：Provider 协议调研

调研日期：2026-08-12
范围：仅采用 OpenAI、Anthropic、Google、Ollama、Microsoft 的官方/第一方文档。本文关注 Glyphshift 的 AI 翻译 Profiles 所需协议能力，不讨论具体模型价格或质量排名。

## 结论摘要

1. Provider 层应采用“统一翻译接口 + 独立协议适配器”，而不是把所有服务都伪装成一个 OpenAI 客户端。建议首批适配器为：
   - `openai_responses`
   - `openai_chat_completions`
   - `openai_compatible`（可配置方言）
   - `anthropic_messages`
   - `gemini_generate_content`
   - `ollama_native`
   - `azure_openai`（可作为第二阶段，但 Profile 数据模型应预留）
2. OpenAI 原生连接优先使用 Responses。OpenAI 明确建议新项目使用 Responses，同时继续支持 Chat Completions；两者的请求、输出和 Structured Outputs 字段并不相同，应作为两个协议适配器实现，而不是一个布尔开关。[OpenAI：迁移到 Responses API](https://developers.openai.com/api/docs/guides/migrate-to-responses)
3. “OpenAI-compatible”不是正式标准。Ollama 官方只承诺兼容 OpenAI API 的“一部分”，其 Responses 也有状态能力限制；Google 的 OpenAI compatibility 又支持另一组端点和字段。由这些第一方实现可以推断：Glyphshift 必须把兼容端点建模为 `dialect + capability probing + 用户覆盖`，不能假定 `/models`、Responses、Structured Outputs、usage 或流事件全部存在。[Ollama：OpenAI compatibility](https://docs.ollama.com/api/openai-compatibility) [Gemini：OpenAI compatibility](https://ai.google.dev/gemini-api/docs/openai)
4. Ollama 必须保留原生适配器。原生 API 才完整暴露 `/api/tags`、`keep_alive`、默认 NDJSON 流、加载/生成耗时和 token 计数，以及本机排队语义；只走 `/v1/chat/completions` 会损失这些本地运行特性。[Ollama Chat API](https://docs.ollama.com/api/chat) [Ollama FAQ](https://docs.ollama.com/faq)
5. “一键翻译”首版应使用客户端小批量、受控并发、逐批落盘。云端 Batch API 是异步吞吐方案：OpenAI 承诺在 24 小时内完成，Anthropic 和 Gemini 也把 Batch 描述为可耗时至约 24 小时的非即时任务；不适合作为用户点击后等待结果的主路径。[OpenAI Batch](https://developers.openai.com/api/docs/guides/batch) [Anthropic Message Batches](https://platform.claude.com/docs/en/api/messages/batches/create) [Gemini Batch](https://ai.google.dev/gemini-api/docs/batch-api)
6. 普通 HTTP 推理请求的“取消”只能统一承诺：停止本地等待、忽略迟到结果、禁止继续写入词典。断开 HTTP/SSE 后服务端是否立即停止生成，各厂商没有一致保证。只有显式异步资源（例如 OpenAI background response、Anthropic/Gemini batch job）才可能有服务端取消端点，并且取消本身也可能是渐进状态。
7. 模型发现始终是可选能力。Profile 必须允许手工输入并持久保存模型 ID；连接测试或刷新模型列表失败，不应清空已保存的模型，也不应阻止用户尝试调用。

## 建议的统一边界

统一层只表达翻译业务语义，不泄漏厂商消息格式：

```text
TranslationProvider
  test_connection(profile) -> ConnectionReport
  discover_models(profile) -> Unsupported | ModelPage
  translate(profile, TranslationBatch, CancellationToken) -> TranslationBatchResult
  capabilities(profile) -> ProviderCapabilities
```

`TranslationBatch` 建议包含稳定的本地 `item_id`、原文、源/目标语言、可选上下文和术语提示。统一输出使用 JSON Schema，核心形状保持简单：

```json
{
  "translations": [
    { "item_id": "opaque-id", "text": "translated text" }
  ]
}
```

适配器职责：

- 把统一请求转换成厂商的 `input`、`messages` 或 `contents`。
- 根据实际能力选择严格 JSON Schema、JSON mode 或纯文本 JSON fallback。
- 校验 `item_id` 一一对应、无新增、无遗漏、无重复；任何 schema 合法但语义不合法的结果都不得直接写入词典。
- 归一化 token usage、请求 ID、限流提示、可重试性、拒绝/安全拦截与取消状态。
- 保存原始 HTTP 状态和安全裁剪后的错误摘要供诊断，但绝不记录 API key、Authorization header 或完整用户文本。

建议的能力结构：

```text
ProviderCapabilities
  model_discovery: supported | unsupported | unknown
  structured_output: json_schema | json_mode | prompt_only
  streaming: sse | ndjson | unsupported
  usage_metrics: token_counts | token_and_timing | unavailable
  rate_limit_hints: headers | error_body | unavailable
  async_batch: supported | unsupported | unknown
  server_cancel: background_only | batch_only | unsupported | unknown
```

能力来源采用三层合并：适配器已知默认值 < 连接时探测结果 < 用户显式覆盖。未知能力不得被当成“不支持”；可以在首次调用失败后降级并缓存结论。

## Profile 数据建议

```text
AiProfile
  id, name, enabled
  provider_kind
  protocol_dialect
  base_url
  auth_kind
  secret_ref
  model_id
  api_version?          # Anthropic/Azure 等需要
  organization_id?     # OpenAI 可选
  project_id?          # OpenAI/Google/Azure 可选
  timeout_ms
  max_concurrency
  max_items_per_request
  max_input_chars_per_request
  temperature?
  structured_output_preference
  custom_headers[]     # 加密值或 secret_ref；禁止覆盖 Host/Content-Length
  capability_overrides
```

设计注意：

- `base_url` 必须按 dialect 解释并由 endpoint resolver 拼接，避免重复 `/v1/v1` 或把 `/api` 错拼到 OpenAI-compatible 路径。
- 密钥只存 `secret_ref`，Profile 导出默认不包含密钥。
- `model_id` 是用户配置的权威值；发现列表只用于选择器和提示。
- `custom_headers` 适合企业网关，但应屏蔽日志并限制危险 header。
- 连接测试拆分展示：网络/认证、模型列表、模型可调用、Structured Outputs。不要因为 `/models` 缺失就把整个 Profile 判为不可用。

## Provider 协议矩阵

| 适配器 | 默认 base URL / 推理端点 | 认证 | 模型发现 | 结构化 JSON | 流式 | 重要私有语义 |
|---|---|---|---|---|---|---|
| OpenAI Responses | `https://api.openai.com/v1` + `POST /responses` | `Authorization: Bearer`；可选组织/项目 header | `GET /models` | `text.format: {type: json_schema, ...}` | SSE 事件 | typed Items/Events、Responses 专属状态与取消资源 |
| OpenAI Chat Completions | 同 base + `POST /chat/completions` | 同上 | `GET /models` | `response_format: {type: json_schema, ...}` | chat completion chunks | `choices[].message`，不可与 Responses 输出解析共用 |
| OpenAI-compatible | 用户配置，通常以 `/v1` 结尾 | Bearer/无认证/自定义 header | 可选 `/models` | 必须探测；可能只有 JSON mode | 必须探测 | 兼容程度不一，不保证 OpenAI 的错误、usage、事件或取消语义 |
| Anthropic Messages | `https://api.anthropic.com` + `POST /v1/messages` | `x-api-key` 或 WIF Bearer；必需 `anthropic-version` | `GET /v1/models` | `output_config.format.type=json_schema`（依模型） | SSE named events | content blocks、`stop_reason`、独立 RPM/ITPM/OTPM |
| Gemini generateContent | `https://generativelanguage.googleapis.com/v1beta` + `POST /models/{model}:generateContent` | `x-goog-api-key` | `GET /v1beta/models` | generation config 中 JSON MIME + schema（JSON Schema 子集） | `streamGenerateContent`，SSE | `contents/parts/candidates`、安全拦截、Google RPC 错误 |
| Ollama native | `http://localhost:11434/api` + `POST /chat` 或 `/generate` | 本机无认证；ollama.com Bearer | `GET /api/tags` | `format: "json"` 或 JSON Schema | 默认 NDJSON | `keep_alive`、加载/生成耗时、prompt/eval 计数、本机内存与队列 |
| Azure OpenAI | 资源 endpoint + `/openai/v1/responses` 或 `/chat/completions` | `api-key` 或 Microsoft Entra ID | 资源模型/部署 API；调用时 `model` 通常是 deployment name | OpenAI 形状，但依部署模型/API 能力 | SSE | endpoint、deployment、配额与 `retry-after-ms` 都是 Azure 私有配置 |

## OpenAI 原生

### 协议与认证

- API 使用 Bearer credential；多组织/项目场景可额外发送 `OpenAI-Organization` 和 `OpenAI-Project`。响应提供 `x-request-id`，并可接收调用方提供的 `X-Client-Request-Id` 以关联超时或网络失败。[OpenAI API overview](https://developers.openai.com/api/reference/overview)
- Responses 是新项目首选；Chat Completions 保持支持。Responses 输入可为字符串或 Items，返回 typed response object；Chat Completions 使用 `messages` 并返回 `choices`。两者应共享认证/HTTP 基础设施，但不能共享 wire codec。[OpenAI Responses migration](https://developers.openai.com/api/docs/guides/migrate-to-responses) [Chat Completions reference](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create)
- 模型发现使用 `GET /v1/models`；返回“当前 key 可见模型”即可，不应从模型名推导全部能力。能力仍需用已知模型元数据、探测或用户覆盖补足。[OpenAI models list](https://developers.openai.com/api/reference/resources/models/methods/list)

### 结构化输出

- Responses 使用 `text.format`；Chat Completions 使用 `response_format`。严格 JSON Schema 与只保证合法 JSON 的 JSON mode 是不同能力，必须分别建模。[OpenAI Structured Outputs](https://developers.openai.com/api/docs/guides/structured-outputs)
- 对翻译批次应优先严格 schema；仍需做业务校验，因为 schema 不能保证译文质量、语言正确或 `item_id` 与原输入语义匹配。
- 模型拒绝、输出截断或 incomplete 不应被当作可写入的成功翻译。

### 流、限流、错误与批处理

- Responses streaming 是 SSE typed events；事件类型未来可增加，解析器必须忽略未知事件而不是失败。[OpenAI streaming](https://developers.openai.com/api/docs/guides/streaming-responses)
- 响应头可提供请求数、token 数和重置时间，临时 429 可能带 `Retry-After`。应记录 `x-request-id`，按 `Retry-After` 或带 jitter 的指数退避；账单、余额、spend limit 类 429 不应盲目重试。[OpenAI rate limits](https://developers.openai.com/api/docs/guides/rate-limits) [OpenAI error codes](https://developers.openai.com/api/docs/guides/error-codes)
- Batch 支持 Responses 与 Chat Completions，使用 JSONL 和 `custom_id` 对齐结果，但完成窗口可达 24 小时，适合作为未来“后台大词典翻译”，不适合首版即时一键翻译。[OpenAI Batch](https://developers.openai.com/api/docs/guides/batch)

## OpenAI-compatible 自定义端点

这是产品方言，不是协议标准。建议 Profile 至少提供：

- `dialect`: `chat_completions` / `responses` / `auto`
- `base_url`
- `auth_kind`: Bearer / none / custom header
- 手工 `model_id`
- `/models` 路径覆盖或关闭模型发现
- Structured Outputs：`json_schema` / `json_mode` / `prompt_only` / `auto`
- streaming：SSE / NDJSON / off / auto

连接测试推荐顺序：

1. 如启用模型发现，尝试 `GET /models`；404/405/解析失败仅标记 discovery unsupported。
2. 使用手工模型 ID 发送最小、无用户数据的合成请求。
3. 用户允许时，再用极小 JSON Schema 探测严格结构化输出；该步骤可能产生费用，应在 UI 中明确。
4. 保存探测到的 endpoint、能力和时间，但保留手动覆盖。

不要把 vendor-specific 字段透传进统一请求。可把少数实验字段放在 Profile 的 `extra_body`，但需要命名空间、大小限制、日志脱敏，并明确它会降低可移植性。

## Anthropic Messages

### 协议与认证

- base URL 为 `https://api.anthropic.com`，推理为 `POST /v1/messages`。直接 API 可用 `x-api-key`，也支持 WIF 短期 Bearer token；所有请求都要发送 `anthropic-version`（SDK 自动处理）。[Anthropic authentication](https://platform.claude.com/docs/en/manage-claude/authentication) [Anthropic versioning](https://platform.claude.com/docs/en/api/versioning)
- 请求以交替的 `user`/`assistant` messages 和独立 `system` 指令组织，响应 `content` 是多个 typed content blocks，不应假设 `content[0].text` 永远存在。[Anthropic Messages API](https://platform.claude.com/docs/en/api/messages/create)
- 模型发现为 `GET /v1/models`，支持游标分页，并返回模型能力/上下文相关元数据；Profile 仍须保留手工 ID。[Anthropic Models API](https://platform.claude.com/docs/en/api/models/list)

### 结构化输出与流

- 支持模型可通过 `output_config.format` 指定 `type: "json_schema"`；支持范围随模型/平台变化，因此适配器默认值之外仍需按模型能力判断。[Anthropic Structured Outputs](https://platform.claude.com/docs/en/build-with-claude/structured-outputs)
- `stream: true` 使用 SSE named events，典型顺序为 `message_start`、content block start/delta/stop、`message_delta`、`message_stop`，中间还可能有 ping 或未来新事件。[Anthropic streaming](https://platform.claude.com/docs/en/build-with-claude/streaming)

### 限流、错误与批处理

- Messages 限流分 RPM、ITPM、OTPM，429 带 `retry-after`；响应还提供 `anthropic-ratelimit-*` 余量与重置时间。缓存 token 的计量规则与其他厂商不同，不应强行压成单一 TPM 数字。[Anthropic rate limits](https://platform.claude.com/docs/en/api/rate-limits)
- 错误 JSON 顶层含 `type: "error"`、`error.type`、`error.message` 和 `request_id`。SSE 在已经返回 200 后仍可能发送 error event；504 表示处理超时，529 表示过载。[Anthropic errors](https://platform.claude.com/docs/en/api/errors)
- Message Batches 可包含大量独立请求，以 `custom_id` 匹配乱序结果，可取消但正在执行的不可中断请求仍可能完成；创建后处理最长可达约 24 小时。[Anthropic batch create](https://platform.claude.com/docs/en/api/messages/batches/create) [Anthropic batch cancel](https://platform.claude.com/docs/en/api/go/beta/messages/batches/cancel)

## Google Gemini generateContent

### 协议与认证

- base URL 为 `https://generativelanguage.googleapis.com/v1beta`；标准推理为 `POST /models/{model}:generateContent`，流式为 `streamGenerateContent`。Gemini API 要求 `x-goog-api-key` header。[Gemini API reference](https://ai.google.dev/api)
- 请求使用 `contents[].parts[]`，响应使用 `candidates[].content.parts[]`；安全阻断可表现为 `promptFeedback.blockReason` 或 candidate 的 `finishReason`，必须与 HTTP 失败分开归一化。[Gemini content generation](https://ai.google.dev/gemini-api/docs/generate-content/text-generation) [Gemini safety settings](https://ai.google.dev/gemini-api/docs/safety-settings)
- `GET /v1beta/models` 可分页并返回 `supportedGenerationMethods`、token limits 等信息。选择器应过滤支持 `generateContent` 的条目，但仍允许手工模型 ID。[Gemini Models API](https://ai.google.dev/api/models)

### 结构化输出、流与错误

- generateContent 可指定 JSON MIME 类型和 JSON Schema；Google 明确说明只支持 JSON Schema 子集，复杂或深层 schema 可能被拒绝。应用仍须校验字段的业务语义。[Gemini Structured Outputs](https://ai.google.dev/gemini-api/docs/structured-output)
- `generateContent` 一次返回完整结果；`streamGenerateContent` 使用 SSE 返回同类响应块。翻译短批次默认非流式更容易保证完整 JSON，流式只作为进度/长输出优化。[Gemini API reference](https://ai.google.dev/api)
- generateContent 的错误为 Google RPC 风格：`error.code`（HTTP 数字）、`message`、`status`（如 `RESOURCE_EXHAUSTED`）及可选 `details`；429、503 应退避，504 可考虑增大客户端 deadline，安全/内容阻断则不应自动重试同一输入。[Gemini generateContent errors](https://ai.google.dev/gemini-api/docs/generate-content/api-errors)
- Gemini 限流按项目而非 API key，常见维度为 RPM、输入 TPM、RPD，具体值依模型和 usage tier 变化；不能把 UI 中的并发上限解释为服务端额度。[Gemini rate limits](https://ai.google.dev/gemini-api/docs/rate-limits)
- Batch 独立限流、最多 100 个并发 batch jobs，可通过 job name 取消；官方定位是最高约 24 小时的异步非紧急任务。[Gemini Batch](https://ai.google.dev/gemini-api/docs/batch-api)

## Ollama native

### 本机与云端语义

- 本机原生 base URL 默认 `http://localhost:11434/api`，无需认证；直接调用 `https://ollama.com/api` 则需要 Bearer API key。本机 Ollama 默认绑定 `127.0.0.1:11434`，暴露到网络需显式调整 `OLLAMA_HOST`，因此 Glyphshift 不应自动替用户开放监听地址。[Ollama API introduction](https://docs.ollama.com/api/introduction) [Ollama authentication](https://docs.ollama.com/api/authentication) [Ollama FAQ](https://docs.ollama.com/faq)
- 本地运行时官方说明不会把 prompt/answer 发回 ollama.com；使用 cloud model 时请求会由云端处理。UI 应把“本地模型”和“经本机代理调用的 cloud model”清楚区分，不能只根据 host 是 localhost 就宣称数据完全离线。[Ollama FAQ](https://docs.ollama.com/faq) [Ollama Cloud](https://docs.ollama.com/cloud)

### 端点与输出

- 翻译优先 `POST /api/chat`，保留 `POST /api/generate` 兼容单 prompt 模型。模型发现使用 `GET /api/tags`，不是 `/api/models`。[Ollama Chat](https://docs.ollama.com/api/chat) [Ollama Generate](https://docs.ollama.com/api/generate) [Ollama list models](https://docs.ollama.com/api/tags)
- `format` 可为 `"json"` 或 JSON Schema；本地 structured output 应优先 schema，并设置 `stream: false` 简化解析。Ollama 官方指出 Cloud 当前不支持 structured outputs，故本机/云模型仍需动态区分。[Ollama Structured Outputs](https://docs.ollama.com/capabilities/structured-outputs)
- REST 默认流式，内容类型为 NDJSON；`stream: false` 才是单个 JSON。中途错误也以含 `error` 字段的 NDJSON 对象出现，HTTP status 已无法变化。[Ollama streaming](https://docs.ollama.com/api/streaming) [Ollama errors](https://docs.ollama.com/api/errors)
- 成功响应包含 `total_duration`、`load_duration`、`prompt_eval_count`、`prompt_eval_duration`、`eval_count`、`eval_duration`，应映射为本机性能指标，而不是丢弃。[Ollama usage](https://docs.ollama.com/api/usage)
- `keep_alive` 控制模型驻留；默认模型约保留 5 分钟，可用 `0` 立即卸载。Profile 可提供高级设置，但一键翻译不应在每个小批次后卸载模型。[Ollama FAQ](https://docs.ollama.com/faq)

### 并发与排队

- Ollama 默认每模型并行数为 1；并行度会成倍增加上下文内存需求。请求过多会先排队，超过 `OLLAMA_MAX_QUEUE` 时返回 503。Glyphshift 对本机 Profile 的默认 `max_concurrency` 应为 1，并允许高级用户提高；遇到 503 应降低并发，而不是密集重试。[Ollama FAQ](https://docs.ollama.com/faq)
- OpenAI-compatible 路径为 `http://localhost:11434/v1/`，API key 字段可能被客户端要求但本机 Ollama 会忽略。该路径只兼容部分 OpenAI API，且 Responses 不支持完整状态语义，因此原生 Profile 不应实现为该方言的别名。[Ollama OpenAI compatibility](https://docs.ollama.com/api/openai-compatibility)

## Azure OpenAI（建议预留，第二阶段实现）

- 当前 GA 风格 endpoint 可使用 `https://<resource>.openai.azure.com/openai/v1/`，调用 `responses` 或 `chat/completions`；请求中的 `model` 通常是部署名而不是公共模型 ID。[Microsoft Foundry endpoints](https://learn.microsoft.com/en-us/azure/foundry/foundry-models/concepts/endpoints) [Azure Responses REST](https://learn.microsoft.com/en-us/rest/api/microsoft-foundry/azureopenai/responses)
- 支持 `api-key` 与 Microsoft Entra ID；这要求 Profile 将 Azure 认证建模为独立 auth kind，不能只复用 OpenAI Bearer key 文本框。[Azure Structured Outputs](https://learn.microsoft.com/en-us/azure/foundry/openai/how-to/structured-outputs)
- Structured Outputs 沿用 OpenAI Chat 的 `response_format: json_schema` 形状，但支持取决于部署的模型与 API 能力。[Azure Structured Outputs](https://learn.microsoft.com/en-us/azure/foundry/openai/how-to/structured-outputs)
- 资源 Models API 能列出资源可访问模型，但实际推理使用 deployment name；因此模型 catalog 与 deployment discovery 应分开，不应把 `GET /models` 的 ID 直接写入 `model`。[Azure models list](https://learn.microsoft.com/en-us/rest/api/azureopenai/models/list?view=rest-azureopenai-2024-10-21) [Microsoft Foundry deployments](https://ai.azure.com/api-reference/deployments/)
- Azure 配额分配到 deployment，并可能返回 `retry-after-ms`；错误归一化时需同时支持秒级 `Retry-After` 与毫秒级 Azure header。[Azure OpenAI quota](https://learn.microsoft.com/en-us/azure/foundry/openai/how-to/quota)

## 一键翻译的协议执行策略

即时路径建议：

1. 上层先选出“尚无有效译文”的条目，再应用过滤器；Provider 不负责判断哪些词典项已翻译。
2. 按字符数/token 估算和 `max_items_per_request` 切成小批；每项携带 opaque `item_id`。
3. 云端 Profile 默认低并发（例如 2），根据 429/503 和 rate-limit hints 自适应降低；Ollama 默认并发 1。
4. 优先严格 JSON Schema；不支持时降级 JSON mode，再降级 prompt-only JSON。每次都做语法和业务校验。
5. 每个成功小批独立提交词典，失败批次可重试，不回滚已经成功且仍满足版本前置条件的其他批次。
6. 写入前执行 compare-and-set：仅当条目仍未翻译且原文/版本未变化时写入，防止 AI 覆盖用户在任务运行期间完成的译文。
7. 取消后：停止排新批、取消本地 future/HTTP body、丢弃迟到响应、禁止任何后续写入；UI 文案不得承诺云端已停止计费或计算。

为什么不用云 Batch 作为首版默认：

- 三家云厂商都把 Batch 设计成异步高吞吐、低紧迫度作业，并给出最长约 24 小时的完成窗口。
- 一键翻译需要即时进度、逐批错误、用户取消和尽快写回探针/词典；客户端调度更契合。
- 将来可另加“后台批量翻译”模式，使用持久 job ID、轮询、恢复、取消和结果导入，而不是偷偷替换即时按钮的执行语义。

## 统一错误模型

建议归一化但保留厂商细节：

```text
ProviderError
  category:
    network | timeout | cancelled | authentication | permission
    invalid_request | model_not_found | rate_limited | quota_or_billing
    overloaded | safety_or_refusal | malformed_output | provider_internal
  retryable
  retry_after_ms?
  provider_code?
  request_id?
  http_status?
  safe_message
```

重试规则：

- 可重试：连接中断、部分 timeout、临时 429、502/503/529；优先遵守厂商 retry header，再指数退避 + jitter，并限制总次数和总时长。
- 不自动重试：认证/权限、模型不存在、请求 schema 错、余额/配额需人工处理、稳定的安全拒绝、语义校验失败超过一次修复重试。
- 兼容端点不能仅凭 HTTP 状态推断类别；优先解析已知 error shape，未知时保留状态与安全消息。
- 所有 provider 都可能在流已经以 200 开始后报告错误，因此 streaming parser 必须有终态校验；收到 EOF 不等于成功。

## 推荐交付顺序

1. `openai_responses`、`anthropic_messages`、`gemini_generate_content`、`ollama_native`。
2. `openai_chat_completions` 与保守的 `openai_compatible`，共享 HTTP 基础设施但独立 codec。
3. Azure OpenAI deployment-aware adapter。
4. 后台云 Batch 模式；不要阻塞即时一键翻译 MVP。

验收时每个适配器至少覆盖：认证失败、模型发现缺失、手工模型 ID、严格 schema 成功/不支持降级、乱序/缺项/重复项、429 retry hint、流中错误、timeout、用户取消后不写入、敏感 header 不进日志。
