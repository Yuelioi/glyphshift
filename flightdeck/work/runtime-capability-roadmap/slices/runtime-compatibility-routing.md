# Runtime 兼容性路由

Status: Complete

## Outcome

Desktop 不再用“整个软件是否支持”这一项粗粒度结论决定所有能力。Runtime 按目标实例、Adapter
Placement、目标架构和实际激活结果形成兼容性计划；可用路径继续运行，不适用路径作为有界诊断返回，
不会拖垮整次 Probe。

## Scope

- 将 `TargetProcess` 与 `IsolatedWorker` 的架构能力分开判断；x86 目标可以使用明确声明支持 x86 的
  进程外 UIA 能力，而 x64 Native Runtime 仍不得注入 x86 目标。
- 为同一软件的主窗口进程、Renderer、GPU 和显式 Process Family 成员保留逐实例激活事实；普通界面
  聚合结果，但不能用一个成员失败覆盖其他成员成功。
- Probe 启动按 Adapter/target 记录 activated、not-applicable 与 rejected，至少一个有效观察路径成功时
  返回部分成功；Workflow 的 `TextReplace` 仍只接受真实 ACK。
- 固定 Target Runtime 驻留后的 Adapter 集合重配合同：能安全重配时执行，否则返回明确的“重启目标后
  重试”，不能再建议用户原地调整技术。
- 将 Target Runtime 状态码与 Controller 拒绝保留为稳定产品类别，不向 UI 暴露内部状态码或进程身份。

## Non-goals

- 不在这一 Slice 建设 x86 Native Controller、Runtime DLL 或 x86 Native Adapter Bundle。
- 不按 PowerToys、网易云或其他软件品牌硬编码进程选择。
- 不把部分激活、UIA Observe 或成功注入显示为原位翻译成功。

## Verification

- [x] x86 软件前置检查允许已验证的 x86 Isolated Worker 观察能力，同时不会把 x64 TargetProcess
  Adapter 误判为 x86 兼容；正式 Bundle 合同固定 UIA Worker 的 x86 声明与 Placement。
- [x] 多进程合成目标中一个成员拒绝激活时，其余有效路径保持运行；Runtime status 返回 active / failed
  target 计数。Hybrid Placement 与 Target Runtime 的既有合同继续提供逐 Adapter feature ACK / failure。
- [x] 驻留 Runtime 使用不同 Adapter 集合重连时返回稳定 `runtime.target_restart_required`，产品明确要求
  完整退出并重启目标软件；当前不做无法证明卸载安全的 Native 热重载。
- [x] NetEase 类 CEF 目标以 UIA + GDI/GDI+ 混合计划完成 5 秒授权复测，得到 263 条唯一候选；只记录
  “混合 Placement 可激活”的可移植结论，原文、路径与运行身份留在本机忽略目录。
- [x] 本 Slice 的定向 Rust / Playwright、真实授权 CEF、Clippy/fmt/architecture 与代码自审通过；本次
  全仓测试也通过，但后续开发循环按 Flightdeck 验证纪律不再默认运行全仓门禁。

## Next

进入[快速探针测试流程](quick-probe-test-flow.md)，由兼容性计划提供默认技术选择与部分成功结果。

## Progress

- Runtime Bundle 的 Adapter presentation 已携带真实 Placement 与 architecture；Desktop 以验证后的
  metadata 形成支持表，而不是相信未加载制品的静态声明。
- 软件前置检查不再全局拒绝 x86；新建 Probe 只展示并默认选择目标架构可观察的 Adapter。
- 驻留 Runtime 的 Adapter 集合变化已从泛化“不兼容”中分离为可执行的重启恢复语义。
- 多进程启动不再因单个 target 拒绝而回滚健康会话；定向合同固定 1 个 active / 1 个 failed target。
- Probe 创建与设置复用相同兼容计划，后端在创建草稿词典前拒绝不兼容 Adapter，失败不留下资产。
- 当前回归：Runtime/Process Host/Desktop Shell 相关测试通过；Vue 类型检查、Probe 9/9 与 Management
  8/8 Playwright 通过，Impeccable UI detector 无命中。
