# 探针写回失败诊断

Status: Finished

## Goal

让探针在目标仍运行但写回或激活失败时报告真实、可恢复的原因；Glyphshift 提权重启后不得继续使用
失效实例，也不能把所有失败都解释为“目标退出或权限更高”。

## Current

真实复验已将本次失败归类为 Remote Memory：Glyphshift 已处于管理员完整性，目标仍拒绝探针所需的
远程内存访问。产品现在保留 `operation` 与 `controllerElevated`，分别解释 Target Process、Remote
Memory、Remote Thread 和 Observer Permission，不再对已经提权的 Glyphshift 重复建议提权。旧实例复用
也已排除：应用提权重启后重新解析当前目标实例。

该目标的保护机制不在本 Work 中绕过；用户决定停止继续尝试该受保护游戏。失败语义诊断与恢复提示目标
已完成，是否支持该目标的写回不属于成功结论。

仓库存在大量未提交的探针、Runtime、GUI 与测试改动；本 Work 必须在现状上增量诊断，不回退其他工作。
真实程序位置、进程身份、日志和截图只进入 `local-test/`。

## Next

None

## Progress

- 已恢复仓库事实并确认这是独立于已完成来源仲裁的新故障。
- Playwright 精确红灯已从失败转为通过：管理员上下文不再出现“开启始终以管理员身份启动”。
- Desktop Shell 83 项测试、前端生产构建、Clippy、Rust 格式与差异检查通过。
- 已通过受控的本机管理员辅助脚本只结束旧 Glyphshift 开发进程，新任务、Vite 和窗口均正常响应；目标
  进程未被修改。
- Remote Thread 管理员提示的 Playwright 合同通过，最新前端生产构建通过。
- 授权目标复验返回 Remote Memory，而非“目标未运行”或“请再次提权”；用户确认不再继续受保护目标的
  写回尝试。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [来源仲裁历史](../probe-source-arbitration/index.md)
- [Runtime 能力路线历史](../runtime-capability-roadmap/index.md)
