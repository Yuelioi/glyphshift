# UIA Isolated Worker transport 与合成 Provider

Status: Completed; real Windows UIA provider tracked separately

## Outcome

UIA 候选不再只是 Registry 中的 `IsolatedWorker` 标签。仓库已具备可监督的独立进程协议、Controller
授权 grant、Session Placement 组合、Desktop 单写入 capture 数据面，以及真实走过 Name/Text/Value
选择策略的合成 Worker。该链路只证明执行与生命周期架构，不代表 Windows UIA 已进入正式 Bundle。

## Delivered

- `glyphshift.isolated-worker/1`：握手、暂停/恢复、有界 observation batch、health、deactivate ack、
  tail drain 与 terminate；request identity 与 schema 均校验。
- producer generation 与 publication generation 明确分离：Worker 重启才改变前者，词典/运行时发布更新
  只推进后者。
- `ProcessIsolatedWorker` 与 `IsolatedWorkerHost`：隐藏子进程、250ms pull、每 Adapter/target 一 supervisor、
  generation ack、暂停排空、停用后尾批次排空和失败开放。
- `HybridAdapterHost`：一个 Session 内按 `TargetProcess` / `IsolatedWorker` 分发，合并 generation、health
  和 deactivation，不复制 Session 状态。
- Controller `/4` Worker Target Grant：generic Core 只保存 platform + payload；Windows Controller 仅从
  已授权 opaque target 签发包含 PID + start time 的临时 `windows-process-v1` grant。UI、Dictionary、
  checkpoint 和 Flightdeck 不保存该 payload。
- Desktop Runtime 真正拥有 `FileCaptureSink`；TargetProcess Host 和 IsolatedWorker Host 只接收 ingress，
  不创建、不暂停、不 finish 第二个 checkpoint owner。
- `windows.uia.observe` 纯策略：只声明 `TextObserve + ObserveOnly + IsolatedWorker`；按 TextPattern、
  ValuePattern、Name 选择一个主要通道，拒绝密码、空白、超长输入，支持变化去重和 element invalidation。

## Verification

- UIA 策略 5 条合同覆盖 descriptor、三个文本通道、CR/LF 规范化、密码/空白/超长拒绝、A→B→A 和
  invalidation。
- Worker SDK/Host 进程合同覆盖授权拒绝、健康、暂停/恢复、publication update、deactivate tail 与唯一
  checkpoint；合成 Worker 的输出真实经过 UIA 策略。
- Controller Host 进程合同覆盖 grant round-trip；Windows inventory 合同证明只有已授权 target 可签发
  与当前进程实例匹配的 grant，未知 token 被拒绝。
- 完整 workspace tests 通过；重建本地 `/4` Runtime Bundle 后，真实 Windows 父子 Process Family
  capture 合同再次通过。

## Not Yet Production-Ready

- 该切片的合成 Worker 不创建 `IUIAutomation`、COM MTA 或注册事件 handler；真实实现与剩余门槛
  记录在 [Windows UIA MTA Client](windows-uia-mta-client.md)。
- Worker timeout 会立即终止失败进程；supervisor 以新 producer generation 按每 60 秒最多 3 次限频
  重启。真实 UIA Provider 卡顿后的目标恢复和明确用户可见 reason code 尚未验证。
- grant payload 是受信本地进程间的临时能力描述，不是持久 token；真实 Worker 仍必须校验 PID start time，
  防止 PID reuse。
- UIA Descriptor 与 Worker artifact 已由后续
  [UIA Runtime Bundle 集成](uia-runtime-bundle-integration.md)加入正式构建脚本和 Desktop catalog。

## Next

通用 transport 无剩余交付；元素/窗口重建、永久阻塞 Provider 恢复、高完整性权限与正式 Bundle /
catalog 集成都已由后续切片覆盖。
