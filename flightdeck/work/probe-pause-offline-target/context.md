# 稳定上下文

## 产品语义

- “暂停收集”表达用户对本地 Probe Run 的控制意图，不等同于卸载目标 Runtime 或删除探针。
- 目标退出后，已经保存的观测与绑定词典仍应保留；用户可以在目标重新启动后继续收集。
- 目标 Runtime 的暂停确认可以作为诊断事实，但不能阻止本地任务进入 Paused。

## 当前实现边界

- DesktopApplication 维护 Probe Run 状态与 active probe 身份。
- DesktopRuntimePool 维护目标会话、Capture owner 与远端 `control_capture`。
- `runtime.stop_unconfirmed` 当前同时承载多种停用失败，不足以表达“本地已暂停但远端未确认”。

## 约束

- 暂停不得清除观测、词典或临时资产。
- 远端无法确认暂停时先保留 active session，因为目标仍可能可用；继续收集先复用它，控制仍失败后
  再清理陈旧会话并执行一次全新连接。
- 不把目标 PID、路径或本机运行证据写入跟踪文件。
