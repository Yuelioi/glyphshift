# Windows UIA 交互式 Provider

Status: Finished

## Outcome

`glyphshift-adapter-uia-worker` 增加独立的一次性 acquisition executable：使用 Controller 签发的当前
process-instance grant，在 MTA 中执行 UIA Point / Text Range 读取，并通过
`glyphshift.acquisition-worker/1` 返回结构化原文和虚拟桌面物理像素 anchor。持续 observation Worker
与其协议保持不变。

## Scope

- 共享并收敛 process grant、创建时间、完整性级别与 COM apartment 校验，不复制安全判断。
- Point：`ElementFromPoint` 后优先 `TextPattern.RangeFromPoint` 的 Word，再退化到公开 Name + control
  rectangle。
- Text Range：起止点必须属于同一授权目标和同一 TextPattern document，返回有界文本及多行
  bounding rectangles；不支持时返回无文字，留给后续 Visual fallback。
- 密码、属性权限拒绝、跨进程 element、目标实例变化、元素失效和超时均失败关闭。

## Non-goals

- 不实现 Region/OCR、热键、UI、翻译或 Presenter。
- 不改变 `windows.uia.observe` Descriptor、持续 capture 或正式 Bundle 路由。
- 不使用真实开发机软件作为仓库 Fixture；标准控件 harness 保持确定性。

## Verification

- [x] 标准控件 Point Word、Name-only Control 与 Text Range 多行 anchors。
- [x] 密码、跨 target、目标退出/重建、权限拒绝和不支持范围失败关闭。
- [x] Worker wire、Host 超时/退出和 observation Worker 既有回归通过。
- [x] Workspace test、Clippy、fmt、architecture checks 与代码自审通过。

## Review

- process grant、进程实例、完整性级别与 MTA 初始化已收敛到共享 Windows support，持续 observation
  与一次性 acquisition 没有复制安全判定。
- 标准 `EDIT` 只稳定暴露 Value/Name；确定性 Text Range 合同改用系统 RichEdit provider，避免用
  测试替身伪造 `TextPattern` 能力。
- Point、Text Range、密码拒绝、目标退出和 Provider 卡顿均从独立 Worker 进程通过真实 wire 验证；
  并行测试窗口串行化，避免 `ElementFromPoint` 命中另一测试进程造成偶发错误。
- 专项合同 5 个通过、2 个需显式授权的本机权限合同保持忽略；59 包 workspace、Clippy、fmt、
  architecture 与 diff 检查全绿。

## Next

进入[交互式采集组合合同](interactive-acquisition-composition.md)：先证明采集结果可分别进入 Dictionary、
Translation Provider 与 External Presentation，且不会被误提升为目标内 `TextReplace`。生产 Target
Frame Source 与 OCR 引擎继续等待授权真实目标的收益证据。
