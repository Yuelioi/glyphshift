# 稳定上下文

## 产品事实

- Dictionary 是唯一翻译内容资产；Probe 只持有绑定 Dictionary 与独立 Observation Index。
- Probe 行内译文会写入绑定 Dictionary，实时预览仅消费已经保存的 Dictionary 内容。
- Dictionary 编辑使用显式草稿并保护未保存变更；AI 批量写入不能绕过该语义或覆盖用户译文。
- AI Profile 是应用级本机配置；Profile 元数据与明文 API Key 一起存放，仍与 Dictionary 内容和运行时
  Job 分离。
- 单批条目数与输入 Token 预算是统一的应用级翻译策略，切换 Profile 时不改变；Profile 不复制这两个值。

## 用户需求

- 支持多个可复用 AI Profile，并可选择不同翻译模型。
- 覆盖主流云端接口以及 Ollama 一类本地模型端点。
- Dictionary 与 Probe 都提供一键翻译。
- 一键翻译默认只处理没有译文的条目，绝不覆盖现有译文。
- 一键翻译必须处理规划中的全部候选；全局 Batch Policy 只限制单次 Provider 请求，不限制任务总量。
- 支持过滤纯数字、包含数字及其他无需翻译的文本，并让用户看得懂本次会跳过哪些内容。
- 全局 AI 批次策略默认单次请求 100 条；输入 Token 预算同样在设置中统一维护。探针联合表可按全部、
  未翻译和已翻译筛选完整数据集。

## 约束

- 供应商差异留在 Provider Adapter 后方，不扩散到 Dictionary、Probe 或 GUI。
- API 密钥、Authorization header 与原始响应不得进入项目数据导出、Flightdeck、日志或错误文案。
- 网络失败、限流、部分响应和取消必须保留安全、可重试的任务结果；重新执行仍只选择空白项。
- 真实凭据和真实模型响应只能进入 `local-test/`，跟踪测试使用合成 Transport 与确定性响应。
- UI 延续现有高密度 Windows 管理工具语言，过滤与批处理不能变成占满屏幕的向导。

## 已确认决策

- Dictionary 正式允许 source-only pending 条目持久化；Runtime 与发布快照只消费 completed 条目。
- 一键使用 split-button：主按钮用默认 Profile 与保存规则立即执行，旁路提供预览和临时选项。
- 首批实现 OpenAI Responses、OpenAI Chat Completions、OpenAI-compatible、Anthropic Messages、
  Gemini generateContent 与 Ollama native；Azure OpenAI 预留数据形状、第二阶段实现。
- 用户后续明确选择 API Key 随 Profile 明文保存在本机配置文件中；设置字段默认遮蔽并支持显示/隐藏，
  旁边明确提示明文风险。
- 编辑 Profile 会载入当前 Key，保存直接更新同一 Profile；删除 Profile 同时删除其中的 Key，不依赖
  Windows Credential Manager。

## 已收敛建议

- 首批实现 OpenAI Responses、OpenAI Chat/OpenAI-compatible、Anthropic Messages、Gemini generateContent 与 Ollama native Adapter。
- OpenAI-compatible 使用能力探测与用户覆盖，不假设模型发现、结构化输出或流协议完整兼容。
- Profile 元数据与明文 Key 同属本机 Profile artifact；模型 ID 始终允许手工填写。
- 一键即时任务采用客户端小批量、受控并发、逐批校验与写回，不使用最长可达约 24 小时的供应商云 Batch。
- Dictionary 成功结果合并到当前草稿，Probe 成功结果通过绑定 Dictionary 的现有写入与预览刷新路径提交。
- “译文为空”在规划与写回时各检查一次，任务期间的用户编辑优先于 AI 迟到结果。
- 超过单批条数或输入 Token 预算时，由 Translation Job 自动继续分批；总进度以完整计划为分母，部分失败不
  阻止其他批次继续执行，重新运行仍只补剩余空白项。
