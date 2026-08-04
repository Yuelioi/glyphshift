# Windows UIA MTA Client

Status: Complete; production safety gates delivered

## Outcome

`windows.uia.observe` 现在有真实 Windows UI Automation 数据面，而不再只有合成策略。实现仍位于独立
Worker，不进入 Desktop、Controller 或目标进程；它只接受 Controller 已授权的进程实例 grant，并把
候选文本送入现有有界 batch / Desktop 单写入链路。

## Delivered

- `glyphshift-adapter-uia-worker` 在 worker 主线程初始化 COM MTA 并创建 `IUIAutomation`；非 Windows
  平台明确返回 `unsupported_operating_system`。
- `windows-process-v1` grant 在激活时重新校验 PID 与创建时间；实例已退出或 PID 已复用时拒绝启动。
- 仅枚举授权 PID 的可见顶层窗口，对其 subtree 注册 TextChanged、Name/Value property changed 与
  structure changed handler；不订阅整个桌面。
- callback 不跨线程传递 COM element，只把最多 2,048 个事件合并成重扫信号，不同步查询 Provider 或
  落盘；单次全树读取最多 4,096 个 element，文本继续受 16K UTF-16 单元上限约束。
- Provider 可能漏发 Name 变化事件，因此保留 1 秒全量扫描作为有界一致性修复；事件仍用于低延迟读取
  Value/Text 变化。
- 每次全量扫描对比活跃 RuntimeId，并从会话去重表移除已消失元素，避免长时间作业保留陈旧去重状态。
- element 先读取 `IsPassword`，敏感控件不会继续读取 Name、TextPattern 或 ValuePattern。
- `IsPassword` 无法读取时整个 element 失败关闭，不会把未知控件当成普通文本；UIA 读取的
  `E_ACCESSDENIED` 会进入稳定的 `uia_permission_denied` health reason。
- deactivate 逐 root 移除三类 handler，再排空已进入队列的尾事件并释放 UIA/COM 对象。
- 通用 Worker Host 对任一超时立即终止子进程；supervisor 使用新 producer generation 重启，并按每
  60 秒最多 3 次限频。合成卡顿 Worker 已证明恢复后可继续采集和正常 deactivate。
- Worker 拒绝码在 Host 边界受长度和字符集约束；`uia_permission_denied` 上送为通用隔离 Worker
  权限拒绝，Worker timeout 上送为隔离 Worker 超时，Desktop command 层复用现有的目标访问失败和
  激活超时提示。
- 永久卡住的合成 Worker 连续耗尽每 60 秒最多 3 次重启后，supervisor 保留终态，health 稳定报告
  `isolated_worker_restart_exhausted`，不再折叠成泛化的 Worker 不可用。
- 确定性目标可销毁并重建顶层窗口、label、单行 Edit、多行 Edit 和 Password Edit；Worker 下一轮
  root refresh 会移除旧 handler、注册新 root，并用新活跃 RuntimeId 集合失效旧去重键。测试命令、
  Worker round-trip 和目标退出均有超时边界，不再依靠无界等待。
- 确定性标准控件目标可人为永久阻塞窗口线程，同时保留独立解除入口。真实 UIA Worker 在下一轮全量
  扫描中被 Provider 阻塞后，Host round-trip 会在 2 秒内超时并立即终止 Worker；解除目标阻塞后，
  新 producer generation 可重新握手、继续采集并正常移除 handler。
- Supervisor 重启预算耗尽后可通过现有停止/重新连接生命周期移除终态 supervisor；新 activation 使用
  新 producer generation 并恢复 Healthy，人工恢复不需要修改 Dictionary 或重启 Desktop。
- Worker 激活前比较自身与目标进程的 Integrity Level；目标更高或完整性信息不可安全取得时，以
  `uia_permission_denied` 失败关闭，避免 UIA 只能读取部分根信息却错误报告 Healthy。
- 标准控件合同显式验证 Client 与目标处于相同 Integrity Level。默认忽略的高完整性合同只接受已授权
  合成目标，先证明目标 IL 更高，再要求 Worker 在发布任何 observation 前拒绝；合同本身不会请求提权。

## Verification

确定性 Windows 标准控件进程同时提供静态标签、单行 Edit、多行 Edit 和 Password Edit。真实 Worker
合同证明初始 `Name / ValuePattern / TextPattern` 候选、更新后的标签/值/文档、进程外 batch 传输、唯一
checkpoint 与 handler 移除；另一个合同销毁并重建完整窗口树，证明同一 Worker 可采到重建后的三类
公开文本。原始、更新和重建后的密码内容均未进入 capture catalog。

参数化的默认忽略 smoke 只接受环境变量提供的授权运行中目标，原文写入本地 evidence。授权 AE 连续
两轮 5 秒采集均得到 21 条唯一公开文本，Worker health 为 Healthy 且无降级码；这证明 UIA 对该目标有
真实观察收益，但不等同于绘制文字全覆盖。

真实 Provider 阻塞与恢复合同、Supervisor 预算耗尽后的手工重连合同均通过。显式授权的 UAC 合同也
已用更高完整性合成目标通过：Worker 返回稳定的 `uia_permission_denied`，capture 保持为空，测试目标
在有界保活时间后自行退出。默认测试仍保持 3 个真实 Windows 合同通过、2 个授权合同忽略；授权运行
时额外 1 个高完整性合同通过。

## Remaining

- 本切片无剩余安全门。observe-only Descriptor 与 Worker artifact 已由
  [UIA Runtime Bundle 集成](uia-runtime-bundle-integration.md)接入正式生产目录；仍不进入 Workflow
  翻译候选，也不修改 Dictionary。
