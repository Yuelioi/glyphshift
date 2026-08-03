# Process Family Controller Inventory

Status: Complete

## Outcome

Software Extension 能以授权根和显式后代可执行文件 allowlist 描述 Process Family；Windows
Controller 从真实进程树解析每个实例，以稳定 opaque token 独立报告生命周期，任何单实例退出或
识别失败都不覆盖其他成员事实。

## Decisions

- 根进程继续使用既有 executable name/full path 授权，存在完整路径时优先精确路径；后代只接受
  Extension 声明的 `descendant_executables`，不加入通配、安装目录邻近或品牌推断。
- 后代关系允许跨过不在 allowlist 中的中间进程，但必须沿当前父链到达授权根或先前已接纳且仍存活
  的成员。
- 一次进程实例的内部身份是 PID + 创建时间。对外 token 单调分配、跨 inventory 稳定且永不复用；
  PID、创建时间和路径都不跨 Controller seam。
- 根先退出不会撤销仍存活的已接纳成员；成员退出立即移除自己的 target 与 Runtime Library 记录，
  其余成员继续 prepare。
- 无法取得创建时间的进程按单实例不可识别处理，不让整个 inventory 失败。
- 授权 AE 的真实证据支持首个 AE Process Family Extension，但成员名只进入独立 Extension payload，
  不在 Core、Desktop 或 GUI 增加软件分支。

## Delivery

- [x] `SoftwareIdentity` 和 Desktop Extension artifact 保存可选后代可执行文件。
- [x] Desktop Runtime Spec、Controller Host 与 stdio 配置贯通后代 allowlist。
- [x] Windows Controller 使用可替换的内部 inventory seam 解析授权根、后代和存活成员。
- [x] target token 跨重排稳定、进程退出清理且 PID 复用不复用旧 token。
- [x] 添加参数化 ignored 实机合同；真实输入和原始输出只存本机忽略目录。

## Verification

- [x] 合成 Controller inventory 覆盖嵌套后代、无关同名进程、身份不可得、根先退出、成员部分退出
  和 PID 复用。
- [x] Extension → Backend Runtime Spec → Controller Host 配置合同通过。
- [x] 授权 AE 的只读快照确认 1 个根与 8 个后代；Windows Controller 实机合同发现 9 个目标实例。
- [x] 全 Rust workspace、Clippy `-D warnings`、fmt 与架构检查通过。

## Current

Slice 已完成。Process Family 只解决“哪些实例属于被授权软件”，尚未改变 Desktop Runtime 一次只
激活一个 target 的行为，也没有让多个 Workflow/Probe 共享同一注入。

## Next

None.
