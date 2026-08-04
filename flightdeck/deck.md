# Flightdeck

## Open Work

- **Focus:** [运行时能力升级 Roadmap](work/runtime-capability-roadmap/index.md) — 当前按真实软件证据
  扩展 Windows Adapter：真实 MTA UIA Client 已在 Isolated Worker 中通过标准 Win32 控件、变化事件、
  handler 移除和密码拒绝合同；密码属性不可读时会失败关闭，Worker 内部也能稳定报告
  `uia_permission_denied`；Host 会把该拒绝和 Worker 超时上送为现有的“目标访问失败”与“激活超时”
  用户错误。顶层窗口和标准控件树销毁重建后的重新发现、旧元素失效和继续采集合同也已通过。通用
  Host 会在超时后立即回收并按 60 秒最多 3 次限频重启，预算耗尽后稳定报告
  `isolated_worker_restart_exhausted`。真实 Provider 永久阻塞后的目标释放、新 generation 重连和预算
  耗尽后的停止/重新连接恢复均已通过；授权 AE observe-only smoke 已连续两轮得到 21 条唯一公开文本
  且 health 为 Healthy。授权 UAC 合同也证明更高完整性目标会在 observation 发布前稳定失败关闭，且
  合成目标有界退出。UIA Descriptor 与 Worker artifact 也已进入正式 Bundle / Desktop catalog；正式
  Desktop capture 合同证明公开标准控件可采集且密码不入库。它只作为 Probe 的 observe-only 能力，
  不进入 Workflow，也不修改 Dictionary。下一步推进 Observation Stream Identity。

## Project links

- [领域语言](../CONTEXT.md)
- [产品契约](../PRODUCT.md)
- [设计系统](../DESIGN.md)
