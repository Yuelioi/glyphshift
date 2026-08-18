# 探针暂停与离线目标收敛

Status: Finished

## Goal

让“暂停收集”始终先满足用户的本地暂停意图：目标软件仍在、刚退出或无法确认控制时，探针任务都应收敛到可恢复的暂停状态，而不是保持运行并显示“功能状态没有改变”。

## Current

暂停现在先停止本地 Capture Sink 写入并保留现有 Runtime。继续收集优先复用该连接；连接确实失效时才清理并执行一次全新重连。目标已恢复则一次点击继续，仍离线则保持 Paused 并明确提示未找到运行实例。

## Next

无。

## Progress

- 已确认用户复现路径：关闭目标软件后点击“暂停收集”，界面提示目标进程未确认停止。
- 已建立并修复红灯合同：目标拒绝暂停确认时，本地暂停成功、active probe 身份清空、陈旧 Capture 会话被放弃。
- 已验证在线暂停保留 Runtime，离线暂停后的继续收集重新发现 Runtime。
- Desktop shell Rust 73/73、Desktop Runtime Rust 33/33、Probe Playwright 17/17 通过，生产前端构建通过。
- Release 审阅流程已同步构建并校验 Runtime Bundle 的 9 个 Adapter，最新版桌面程序已启动。
- 用户实机发现：离线暂停成功后点击继续收集，错误显示为所选探针技术不适用。
- 红灯精确复现 `runtime.component_incompatible`：暂停失败路径过早放弃了仍可能可用的 Runtime，继续时被迫重新注入。
- 已改为本地 Capture Sink 先暂停、保留当前会话；继续失败时才放弃旧会话并重连一次。
- 已验证会话仍可用、旧会话失效但目标已恢复、目标仍离线三条路径；离线只返回 `runtime.target_not_found` 并保持 Paused。
- Desktop shell Rust 75/75、Desktop Runtime Rust 33/33、Probe Playwright 17/17 通过；同步 Release 与 9 Adapter Runtime Bundle 已启动。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
