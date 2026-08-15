# AI 翻译并发可观测性与吞吐诊断

Status: Finished

## Goal

让用户能直接确认 AI 翻译请求是否真正并发，并区分 Provider 耗时、超时重试与客户端排队；以 258 条、单批 50、并发 3 为代表负载，不再只显示容易被误解为串行的累计完成数。

## Current

任务快照已暴露每批 queued/running/retrying/completed/failed/cancelled 状态、条目数、启动偏移、耗时、尝试次数、最后安全错误与并发峰值。Dictionary 与 Probe 共用同一进度面板，终态报告现可显式关闭。

## Next

None

## Progress

- 用并发 3 的合成 Provider 建立红灯合同，证明旧快照无法表达三个同时请求；新合同已通过。
- 重试退避已可见，成功恢复后仍保留尝试次数和最后安全错误；停止会立即将未结束批次标记为已取消。
- 进度面板优先展示请求中、等待重试和失败批次，258/50/3 的六批状态在 1280×720 可见。
- AI Rust 19 项、Desktop Shell 72 项、AI Playwright 10 项、生产构建与界面检测全部通过。
- 同步 Release 已同时构建桌面程序与 Runtime Bundle，9 个 Adapter 校验通过并已启动。
- 终态报告标题右侧增加“关闭批次详情”；关闭只清除本次报告，已写入译文保持不变。新回归、生产构建、界面检测与同步 Release 均通过。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [历史 AI 翻译工作](../ai-translation-profiles/index.md)
