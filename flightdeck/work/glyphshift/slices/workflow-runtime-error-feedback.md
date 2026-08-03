# 工作流 Runtime 错误反馈

## Goal

让启用中的工作流直接说明没有实际运行的原因，并给出恢复动作；保持持久启用期望、Runtime actual
state 与软件兼容事实相互独立。

## Current

首轮 UI 已删除“需要处理”，并能展开 Backend 的逐软件 `CommandError/1`；但实机验收证明后端错误
分类仍然过度压缩，本 Slice 重新进入实现阶段。

授权目标中，AE 进程已载入当前 Runtime 与全部四个 Adapter，远程诊断控制返回 active，但桌面仍
显示“组件不兼容”。这与桌面重启后再次激活已存活 Runtime 得到 `AlreadyActive(10)`、随后被统一
映射为 `runtime.component_incompatible` 完全吻合。另一个多进程目标存在 9 个同路径、同架构实例，
首个 inventory 候选是无窗口 utility 进程；Desktop 固定取第一个目标，最终 Runtime 未载入任何实例。
位数不匹配和 Bundle 整体损坏均已排除，正常对照目标能载入同一 Runtime。

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

## Next

- 先为 `AlreadyActive` 和具体 `TargetRuntimeRejected(status)` 建立红绿合同，再实现 Desktop 重启后的
  幂等接管；同时把多进程“固定取第一项”作为 Target Selection 缺陷接入
  [运行时能力升级 Roadmap](../../runtime-capability-roadmap/index.md)。不先修改表层文案掩盖底层状态。
