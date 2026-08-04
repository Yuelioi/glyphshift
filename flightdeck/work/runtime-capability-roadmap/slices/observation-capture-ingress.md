# Observation / Capture Ingress

Status: Completed

## Outcome

多个观察 producer 通过一个有界、非阻塞 Interface 把文字交给唯一 Capture owner；producer 不认识
checkpoint 路径、revision、暂停实现或文件格式。进程内 Interface 与 Target Runtime / Controller
transport 已完成，唯一文件 owner 已提升到 Desktop 并可聚合 Process Family。

## Decisions

- `CaptureIngress` 是 producer 唯一需要认识的 Interface：可克隆、`try_observe` 非阻塞，并明确返回
  `Accepted`、`Paused` 或 `Dropped`。
- `FileCaptureSink` 仍是唯一 owner，独占 checkpoint、revision、暂停和 finish；克隆 ingress 不会创建
  第二个文件 writer。
- `CaptureObservationBatch/1` 只携带受 supervisor 管理的 producer ID、generation、累计 dropped 和
  严格递增的 producer-local sequence；每批记录数、JSON 字节数、标识符和原文长度都有硬上限。
- 队列、丢弃计数和暂停状态都封装在 Capture module 内，不向 Adapter、UIA 或 Dictionary 泄漏。
- Target Runtime 内部已把 native callback 从 owner 改接 ingress；正式 Desktop capture 只部署 batch
  producer，不再把 checkpoint 路径传入目标进程。
- Target Runtime 部署支持互斥的 file capture 与 batch producer；batch producer 通过有界 drain export
  返回合法空心跳或最多 256 条记录，不会与旧文件 owner 双重采集。
- Controller SDK `/3`、Windows remote query、Controller Host transport 已贯通 batch；Host 在 wire seam
  后重新构造并校验 Capture 领域对象，不信任插件返回的松散字段。
- `CaptureObservationCursor` 跨批次拒绝 producer/generation 交换、replay 和无法由 dropped 解释的 gap；
  producer 累计 dropped 会合并进中央 Capture catalog。
- Desktop TargetProcessHost 为每个 target 启动独立 poll supervisor，但所有 supervisor 只持有同一个
  `FileCaptureSink` 的 ingress。只有全部 target 均暂停后才暂停 writer，停止最后一个 target 才 finish。
- health/deactivate acknowledgement 仍必须与真实 IsolatedWorker 生命周期一起设计，不先制造只包
  JSON 的浅 Module。

## Verification

- `glyphshift-capture` 合同证明两个线程中的 TargetProcess 与 IsolatedWorker producer 经两个 ingress
  clone 合流，最终只有一个 checkpoint owner，来源 Adapter ID 均保留。
- owner finish 后的旧 ingress 返回 `Dropped`，不会悄悄创建或写入另一个 checkpoint。
- 现有去重、容量、实时 checkpoint、暂停/恢复与断点续作合同保持通过。
- Target Runtime 默认合同通过，native callback 只持有 `CaptureIngress`。
- Observation batch JSON round-trip、未知字段、非法 producer、零 generation、重复 sequence、超长原文、
  超量记录和超大输入合同通过；payload 不含输出路径或 translation。
- 真实 Windows 合成目标证明 Console Adapter 注入后，观察批次可经 Runtime export 与 Windows
  Controller 返回；第二次 drain 是同 producer/generation 的合法空批次。
- Controller SDK 消息与独立进程 Host 合同覆盖 wire round-trip、领域重建和累计 dropped。
- 真实 Windows Process Family 合同证明父子 target 同时部署后写入一个 checkpoint；暂停期间两边均不
  入库，恢复后均入库，子进程退出不会终止 Controller，父进程继续采集且 stop 正常收尾。
- 完整 workspace test、Clippy `-D warnings`、fmt、architecture check、diff check 与新增文本隐私扫描通过。

## Residual Risk

- pull 周期为 250ms。目标正常暂停/停止会先冻结 producer、排空尾批次再卸载；但进程突然崩溃时，
  尚未被拉取的最后一个周期内存队列会随进程消失，无法在进程退出后恢复。
- 合法的 target-level `Rejected` 不再终止整个 Controller，兄弟 target 可继续采集；Isolated Worker
  进程宿主、health 与 deactivate ack 已完成，但自动限频重启、checkpoint 存储失败和权限/Provider
  卡顿仍需形成用户可见诊断。
- UIA Worker 已有通用进程宿主与 IPC，尚无真实 Windows MTA Client 和事件 handler。

## Next

按 [UI Automation observe-only Seam](uia-observer-seam.md)交付真实 Windows MTA UIA Client 与事件
Provider，让第二类真实 producer 复用现有 Worker Host 和 Desktop owner；同时补齐自动回收、权限与
突然退出时的可见 dropped/degraded 语义。
