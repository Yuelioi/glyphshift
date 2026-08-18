# B1 后端任务所有权

## Deliverable

Desktop 后端成为翻译任务的唯一 owner：只允许一个活动任务，绑定目标词典并在批次完成后安全写回；
页面不再负责等待结果或保存译文。

## Steps

- [x] 扩展任务快照与协调接口，保存来源、目标词典、写回统计和终态事实。
- [x] 在 Desktop 层拒绝第二个活动任务，并暴露当前任务与目标词典锁状态。
- [x] 让词典与探针任务使用同一后端写回路径；完成批次只写空白译文，取消保留已写入项。
- [x] 持久化活动/终态任务；启动时把未完成任务标记为中断，不自动重发。
- [x] 用定向 Rust 合同验证唯一性、锁、取消、写回和恢复。

## Current

Desktop 启动任务后由独立 Rust 监控线程持续同步 Core Job；前端页面或 WebView 计时器不再拥有写回。
任务历史 schema 2 保存脱敏批次、usage、写入与保留计数，并把 schema 1 的旧记录迁移为已完成计数。
取消立即停止后续批次并保留已写入结果；启动恢复只生成 Interrupted 记录，不重发请求。

## Evidence

- `glyphshift-ai-translation` 全部合同通过，包含唯一任务、usage、取消、历史恢复与 Codex Profile。
- `glyphshift-desktop-shell` 全部 76 条测试通过，目标词典写锁合同确认其他词典仍可写。
- Glyphshift Rust Codex Provider 真实合成烟测完成 2/2，返回输入、输出、推理与总 Token。

## Next

本 Slice 已完成；Work 下一步转到最终界面审阅与同步桌面启动。
