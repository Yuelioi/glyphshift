# 阶段计划

1. [x] 调研主流供应商与 Ollama 官方协议，审计现有 Dictionary、Probe、Settings 与持久化模块。
2. [x] 确认产品简报、领域术语、Provider seam、凭据与批量任务语义。
3. [x] 以失败合同建立 Profile 存储、过滤规划与 Provider Transport 的[后端纵向切片](slices/backend-foundation.md)。
4. [x] 实现 AI Profile 管理界面及连接测试，覆盖中英文、错误与无凭据状态。
5. [x] 实现 Dictionary 与 Probe 的一键翻译、预览过滤、进度、取消和部分失败恢复。
6. [x] 运行定向 Rust / Playwright 回归，以合成 Transport 验证协议，并完成 Flightdeck 收尾。
