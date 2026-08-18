# AI 成本控制与运行记录

Status: Finished

## Goal

避免 AI 翻译因短超时和不透明 usage 产生不可解释费用：新 Profile 默认超时 30 分钟，预检明确区分
字符串数与原文输入粗估，供应商 usage 和批次事实进入持久、脱敏的本机测试留档。

## Current

新 Profile 默认单批超时已改为 30 分钟，允许范围扩到 60 分钟，既有显式值不覆盖；前端固定 10 分钟
总截止已移除。预检先显示待翻译文本数，并把 Token 明确限定为不含输出、推理、缓存、重试和取消后
服务端计算的“原文输入粗估”。

六种 Provider 响应 usage 已归一化到 input/output/reasoning/cached input/total，并汇总进批次和 Job。
Desktop 在终态查询或取消时自动把最近 100 条脱敏记录写入应用数据目录，不提供任何用户界面。

工作区同时保留用户要求不提交的根文档与 README 截图改动，本 Work 不创建 commit。

## Next

None

## Progress

- 已从当前进程恢复三条匿名 Job 记录，并确认费用差异由口径、默认 thinking 和重复请求共同造成。
- 已确定翻译运行记录只保存诊断事实，不保存凭据、Base URL、原文或译文内容。
- AI Core 22 项、Desktop Shell 75 项、AI Playwright 10 项通过；生产前端构建、Rust 格式、定向
  Clippy 与界面机械检测通过。
- 历史记录用户界面方案已按用户澄清完整撤回，只保留本机测试留档文件。
- 用户选择不导出旧进程内存结果；同步 Release 已构建、校验并启动，嵌入页面、EXE/Runtime 哈希与
  空 stderr 检查通过。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [AI Profile 历史工作](../ai-translation-profiles/index.md)
- [AI 并发可观测性](../ai-translation-observability/index.md)
