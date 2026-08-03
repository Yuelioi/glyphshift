# 工作流 Runtime 错误反馈

## Goal

让启用中的工作流直接说明没有实际运行的原因，并给出恢复动作；保持持久启用期望、Runtime actual
state 与软件兼容事实相互独立。

## Current

本 Slice 已完成。UI 不再把逐软件错误压缩成“需要处理”；软件未启动、访问失败、组件加载、协议、
超时和停止未确认均保留具体状态与恢复动作。

底层实机回归的两个根因也已修复：Target Runtime 对完全等价的 active deployment 返回成功，对更高
generation 原子更新，对同 generation 冲突 publication 明确拒绝；Windows Controller 只把没有同路径
匹配祖先的候选标记为根，将真正的顶层目标稳定排在同可执行文件 utility 后代之前。目标软件若仍
载入修复前的 Runtime，需要完整退出并重开一次；多个彼此独立的顶层实例仍留给后续 Target Selection。

## Decisions

- `runtime.target_not_found` 只表示当前没有匹配的运行进程，不表示 Hook 或 Adapter 不支持。
- 添加软件阶段只拦截不存在、不可访问、非文件、非绝对路径或非 `.exe` 的绑定。Hook 覆盖依赖软件
  真实绘制路径，只能在运行后由激活结果或探针证据确认；不按软件名称静态猜测。
- 软件未启动是可恢复等待状态，使用 warning；确定的权限、加载、兼容或激活失败使用 error。
- 多软件工作流保留一个紧凑状态入口，但详情逐软件展示，不丢失 Backend 的 `CommandError/1`。
- `TargetRuntimeRejected(status)` 不能继续丢弃 status；至少要区分 already active、Adapter load、
  Adapter changed、kernel activation、Adapter activation 与 Runtime unavailable。
- Desktop 重启后应重新接管或幂等激活目标内仍存活的同版本 Runtime，不能把 `AlreadyActive` 当成
  组件不兼容。
- 多进程软件不能固定选择 inventory 第一项；当前 Controller 的 opaque target 没有足够的实例角色
  信号，需在 Process Family / Target Selection seam 解决，不在 UI 猜主进程。

## Verification

- 红绿回归覆盖软件未启动不再显示“需要处理”，并可展开完整原因和刷新动作。
- 独立回归覆盖权限错误不会被误标为软件未启动。
- 桌面生产构建通过；38 项 Playwright 全部通过；Impeccable 机械检测无问题。
- 实机只读探针确认 Runtime、Controller 与目标均为同一架构；AE Runtime/四 Adapter 已载入且
  Runtime active；多进程目标的首候选为无窗口 utility，所有实例均未载入 Runtime。
- 红绿合同先复现 active Runtime 返回 `AlreadyActive`，再通过实际 C ABI 验证等价重连、冲突拒绝与
  新 generation 接管；Controller 合同先复现同路径后代排在根进程前，再验证根优先和独立根保留。
- `cargo test -p glyphshift-target-runtime -p glyphshift-controller-windows`、全仓测试、Clippy、fmt、
  架构检查、Desktop 生产构建和 49/49 Playwright 全部通过；两项 ignored 真实原生 Runtime 合同通过。

## Next

None
