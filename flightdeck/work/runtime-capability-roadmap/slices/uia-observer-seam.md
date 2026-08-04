# UI Automation observe-only Seam

Status: Real Windows client and reconstruction complete; production promotion deferred

## Outcome

UI Automation 被确认是标准 Windows 控件的高价值结构化观察来源，但不作为目标进程 DLL Hook。
正式实现必须运行在独立 MTA Worker 中，经有界 Observation Ingress 交给 Desktop 侧单写入者聚合。
Ingress、跨进程 batch transport、单写入 owner、Worker/IPC、授权 grant、Host 生命周期、文本策略与真实
Windows MTA Client 均已完成；标准控件、事件、密码拒绝和完整窗口树重建已有确定性合同。在高完整性与
永久阻塞 Provider 合同完成前，不进入正式 Runtime Bundle。

## Product Boundary

- 只声明 `TextObserve + ObserveOnly + IsolatedWorker`，不声明 `TextReplace`、字体替换或实际绘制覆盖。
- UIA 是控件 Provider 暴露的可访问性语义，不等同于屏幕上最终绘制的文本；与 GDI、GDI+、
  Console 等绘制/调用观察允许重叠。
- 订阅根以当前授权目标实例为界。Process Family 的每个进程成员仍需独立发现和授权，不能仅凭窗口标题
  或桌面全局扫描扩大范围。
- UIA 失败必须 fail-open：Provider 超时、元素失效、目标退出或权限不足只降低该观察源，不影响目标软件。

## Worker Contract

- Worker 使用无窗口的专用 COM MTA 线程创建 UIA Client；事件注册和移除由同一 MTA 所有者串行完成。
- 回调只提取有界、缓存过的最小事实并入队，不在回调中执行字典匹配、文件写入或无界树遍历。
- 目标退出、Worker 停止和异常恢复都必须显式移除 handler；若 Provider 调用卡住，由 Desktop Watchdog
  回收整个 Worker，而不是让 App UI 线程等待。
- 32/64 位跨进程帮助对象和 Automation Element 都是短生命周期运行时对象，不跨 Worker 重启保存。

## Text Extraction

按控件能力选择一个主要文本通道，避免把同一内容重复记录：

1. 简短标签、按钮、菜单等优先使用非空 `Name`。
2. Edit / Document 等真实文本容器优先使用 `TextPattern` 的 Document Range；读取必须有字符上限。
3. 支持 `ValuePattern` 且表达当前值的控件使用 `Value`；密码和受保护值不采集。
4. 同一元素同一通道只在规范化文本变化时发出；空白、超限、不可打印或仅布局噪声不进入候选。

会话内去重键使用 `Target Instance + RuntimeId + Channel + Normalized Text`。`RuntimeId` 只作当前 UIA
会话身份；跨重启绑定最多使用 AutomationId、ControlType、父链摘要和样本历史作为可诊断证据，不能承诺
永久稳定。

## Required Architecture

当前仓库已有 `Placement::IsolatedWorker`、observe-only Registry、有界 Observation batch transport、
逐 producer cursor、Desktop 单写入 Capture owner、Isolated Worker 进程宿主/IPC、Worker Target Grant、
Hybrid Host、真实 COM MTA Client、事件 handler、标准控件/重建 fixture、超时立即回收和限频重启。
生产链仍缺少：

1. 同完整性与高完整性目标的真实权限合同，不自动提权。
2. 人为永久阻塞 Provider 后的目标释放、Worker 重建和超过重启预算后的人工恢复入口。
3. UIA 与绘制 Hook 同时命中时的来源证据和可选短时呈现去重。

因此不能把 UIA 临时标为 `TargetProcess`，也不能让它另开一个 `FileCaptureSink` 与现有 Hook 竞争同一
探针文件。

## Verification Gate

- 合成 Provider：Name、Value、Text 三条通道分别命中且不重复；密码值被拒绝。
- 生命周期：目标退出、元素失效、暂停/恢复、Worker 重启和 handler 移除均有确定性合同。
- 安全：普通权限目标通过；高完整性、受保护/System UI 返回明确 degraded，不要求用户关闭系统保护。
- 性能：回调有界，慢 Provider 不阻塞 App；队列溢出报告 dropped/gap。
- 组合：UIA 与一个目标进程 Hook 同时观察时，只存在一个 checkpoint 写入者，并保留来源 Adapter ID。
- 实机：至少一个授权的标准控件软件证明稳定命中后，才评估进入正式 Bundle。

## Next

继续按 [Windows UIA MTA Client](windows-uia-mta-client.md)完成高完整性与永久阻塞 Provider 合同；
授权 AE smoke 已证明稳定观察收益；继续验证高完整性与永久阻塞 Provider，再决定是否进入 Bundle。
