# 探针来源仲裁与占用冲突反馈

Status: Finished

## Goal

让探针连接失败准确说明目标已被工作流占用；让同一目标文字被多个 Adapter 重叠观察时，可写回的
绘制来源成为首选，observe-only 来源只在没有更强来源时兜底，且不按具体 Adapter ID 写品牌分支。

## Current

目标已完成。Desktop Runtime 现在返回带所有者的稳定冲突语义，Probe 与 Workflow 组合根分别映射成
可恢复的产品错误；探针连接已能明确提示目标正由工作流占用。Capture 由 Adapter Descriptor 的
`TextReplace` 能力推导首选/兜底角色：可写回来源优先，observe-only 来源保留 provenance 和独有文本，
并在有界容量不足时向首选来源让位。

所有 Adapter 仍可并发观察；“观察器最后”体现在确定性的结果排序、去重与容量仲裁，不引入依赖事件
到达时序的延迟启动。实现没有按 UIA/GDI ID、软件品牌或可执行文件名分支。

## Next

None.

## Progress

- 新增 Runtime 双向所有权合同、Tauri 错误映射合同、Capture 到达顺序/容量合同与探针页面精确提示合同。
- 受影响 Rust 包共 127 项测试通过，Clippy、Rust 格式检查、前端生产构建与差异检查通过；新增
  Playwright 合同通过。
- 本地授权 AfterFX 定向复验已显示精确冲突提示，验证后已恢复测试前工作流状态并重启开发实例。
- 完整页面回归仍有若干既有合同债务（旧词典弹窗、过期探针界面假设、严格定位歧义与原生按钮规则），
  与本 Work 新增合同无关，未在此扩张范围修复。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [当前总体架构](../glyphshift/references/architecture.md)
- [Observation Stream Identity](../runtime-capability-roadmap/slices/observation-stream-identity.md)
