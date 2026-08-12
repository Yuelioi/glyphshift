# 完整批次排空回归

Status: Complete

## Deliverable

一键补全对本次规划的全部候选负责。全局 Batch Policy 的 `maxItemsPerRequest` 只决定 Provider 单次请求大小；
候选超过该值时自动生成后续批次，直到全部成功、逐批失败或用户取消，不要求用户重复点击。

## Steps

1. [x] 用 51 条合成候选建立可重复合同，确认表格每页 50 条不会截断任务规划。
2. [x] 为任务快照增加完整批次进度；单次请求条数与输入 Token 预算只负责拆批，总进度以完整计划为分母。
3. [x] 覆盖 Dictionary 与 Probe；确认某批失败不会阻断其他批次，重新运行只选择剩余空白项。
4. [x] 运行 AI Rust 合同、桌面后端/壳合同、生产构建、架构检查与完整 Playwright。

## Verification

- 合成 Provider 收到至少两个有界请求，最终结果覆盖 51 个稳定 item ID。
- Dictionary 草稿与 Probe Dictionary 最终各写入 51 条，人工已有译文仍不被覆盖。
- 真实凭据、响应和本机数据继续只留在 `local-test/`。

## Result

- 51 条候选在测试策略设为单次 20 条时拆成 `20 + 20 + 11`，任务终态为 51/51、3/3 批。
- 探针后端用 52 条候选验证完整 Observation Index 规划，与 GUI 每页 50 条无关。
- Provider 限流会根据 `Retry-After` 或指数退避进行有界重试；单批最终失败时保留其他批次成功结果，界面可只重试剩余空白项。
- AI Rust 15 项、桌面壳 70 项、完整 Playwright 78 项、生产构建、格式与架构检查全部通过。
