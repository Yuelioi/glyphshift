# UIA 交互式取词合同与 Geometry Fixture

Status: Finished

## Outcome

在 `runtime/acquisition` 建立一次性 `acquire(request) -> result` 深 Module，并由 UIA Structured
Adapter 通过纯快照实现 Point / Text Range / Control 退化策略。调用者只认识授权目标、物理像素选区、
有界原文块和稳定失败类别；UIA RuntimeId、COM、窗口句柄、DPI 换算与密码属性不跨 Interface。

## Scope

- 固定 Point、Text Range、Region 与 StructuredOnly / VisualOnly / Automatic 请求合同。
- 固定虚拟桌面物理像素坐标、负坐标、右下不包含矩形和 DIP → 物理像素换算。
- 以确定性 UIA Fixture 证明 Point 取得 Word、Text Range 跨行返回多个 anchors、Name-only 控件退化为
  Control，并对密码、目标不匹配和无文字失败关闭。
- Acquisition Module 负责 Provider 排序、输出正规化、文本上限、去重、部分成功和稳定错误归并。

## Non-goals

- 不增加全局热键、悬停计时、浮层、Dictionary 写入或 Translation Provider。
- 不扩充持续 Probe / `CaptureObservationBatch/1`。
- 不在本切片增加 OCR、截图持久化或 Windows UIA Worker transport。

## Verification

- [x] `glyphshift-acquisition` 合同与 Geometry Fixture：5 个 acquisition + 4 个 geometry 合同通过。
- [x] `glyphshift-adapter-uia` Point / Text Range / Control / Privacy Fixture：7 个新增合同通过。
- [x] 55 包 Workspace test、全目标 Clippy、fmt 与 architecture checks 通过。
- [x] 代码自审通过：Interface 只有一次性 `acquire`；Automatic 仅在普通失败时回退，密码、目标不匹配和
  取消会终止整条链；目标、坐标和文本均有界，无机器数据。

## Review

- `runtime/acquisition` 是零依赖 L0 Module；Architecture Gate 明确允许 UIA → Acquisition，并禁止反向
  依赖 Adapter、Dictionary、Workflow 或 Product。
- Structured 优先于 Visual；同类 Provider 的部分成功会保留，但 Structured 已有结果时不会额外触发
  Visual/OCR。
- Point anchor 必须包含触发点；Text Range 保留多行矩形，但不要求拖拽端点落在 glyph bounds 内，因为
  UIA 会排除尾随空白。
- Geometry 使用每次请求的新 `SurfaceGeometry` 快照表示虚拟桌面原点、Per-Monitor DPI 与滚动；负坐标、
  左上包含/右下不包含和溢出均有确定性合同。

## Next

继续 [Target Frame + OCR Visual Fixture](interactive-visual-acquisition-fixture.md)，先证明第二 Adapter 能在
授权目标表面内完成 Region 裁剪、坐标正规化和置信度输出；随后再分别接 Windows UIA 与截图/OCR 的
进程外 transport。
