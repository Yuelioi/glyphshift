# 稳定上下文

- 新建 AI Profile 的默认单批超时是 30 分钟；既有 Profile 的显式值不被静默覆盖。
- 预检主事实是待翻译字符串数与批次数。输入 Token 只能称为“原文输入粗估”，并明确排除输出、推理、
  缓存、重试和取消后服务端继续计算。
- 翻译运行记录是持久、脱敏、只读的诊断事实，保留最近 100 条；它不复制原文、译文、凭据或 Base URL。
- 每条记录包含时间、scope 类型、Profile 显示名、协议、模型、终态、字符串数、批次、请求尝试、耗时、
  Provider usage、批次安全错误与并发峰值。
- Provider usage 统一为 input、output、reasoning、cached input 和 total；未知字段保持缺失，不能用估算冒充。
- 取消只能证明本地停止等待；没有响应 usage 的取消请求仍记录尝试次数和“供应商用量未知”。
- 记录写入应用数据目录的 `ai-translation-history.json`，供本机测试和诊断工具读取，不进入用户界面。
