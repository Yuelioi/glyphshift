# 交互式取词 Seam

Status: Planned; interface design complete

## Outcome

划词翻译、悬停翻译和框选 OCR 共用一个交互式取词 Module。它只接收一次用户明确触发的选区，针对
已授权目标取得有界原文与短期屏幕锚点；Translation Resolver 和外部译文呈现位于其后，不进入取词
Module。现有持续 Probe、`CaptureObservationBatch/1`、Observation Index 和 Dictionary 均保持不变。

## Product Cases

- Point：用户触发后指向一个控件或单词；结构化 Provider 返回控件级或单词级原文与锚点。
- Text Range：用户触发后从起点拖到终点；同一 TextPattern 支持范围语义时返回结构化文本，否则由
  后续产品策略决定是否回退到该矩形的 OCR。
- Region：用户框选目标窗口内一块区域；结构化 Provider 可返回有序文本块，Visual Provider 可对
  授权窗口画面执行 OCR。

热键、修饰键、悬停时长、浮层样式、OCR 引擎和在线翻译器都不是本切片的底层决定。

## External Interface

Module 对调用者只提供一个深 Interface：

```text
acquire(AcquisitionRequest) -> AcquisitionResult

AcquisitionRequest
├─ authorized_target
├─ selection: Point | TextRange | Region
└─ source_policy: Automatic | StructuredOnly | VisualOnly

AcquisitionResult
└─ blocks[]
   ├─ source
   ├─ anchors[]
   ├─ granularity: Word | Control | Line | Region
   ├─ provenance: Structured | Visual
   └─ confidence: optional bounded value
```

Module 内部负责目标校验、Provider 排序、超时、取消、文本上限、空间裁剪、坐标正规化、去重、隐私拒绝
和部分成功。调用者不认识 UIA RuntimeId、窗口句柄、截图实现或 OCR 引擎。

## Internal Seams

```text
User Gesture
    │
    ▼
Interactive Text Acquisition
    ├── Structured Acquirer ── UIA point / text range / control rectangle
    └── Visual Acquirer ────── authorized target frame + OCR
    │
    ▼
Source Blocks + Ephemeral Anchors
    │
    ├── Dictionary / future Translation Provider
    └── External Translation Presentation
```

- Structured Acquirer 与 Visual Acquirer 是两个真实 Adapter，因此取词 Interface 不是为单实现预造的
  浅 Seam。
- Visual Acquirer 不直接截取任意桌面区域；它只消费由 Controller 授权的 Target Frame Source，并将
  用户选区映射、裁剪到该目标表面，避免采集覆盖在目标上方的其他应用。
- Translation Resolver 只消费原文，不理解 UIA、OCR 或坐标；Dictionary 仍只保存
  `source + translation`。
- Presentation 只消费译文和短期 anchors；它不修改目标控件，不冒充 `TextReplace` ACK。

## Geometry Contract

- `DesktopPoint` / `DesktopRect` 使用 Windows 虚拟桌面的物理像素坐标，允许多显示器产生负坐标；矩形
  使用左上包含、右下不包含的有界表示。
- Provider 的局部坐标、DPI、世界变换与窗口客户区转换全部在取词 Module 内正规化。调用者不执行
  UIA、HDC、Qt、Cairo 或截图坐标换算。
- anchors 只在当前交互会话中有效，不写入 Probe、Dictionary、Workflow、Software 或 Region Binding；
  窗口移动、滚动、缩放或 Target Instance 改变后必须重新获取。
- 一个文本块可以有多个矩形，以容纳换行的 UIA Text Range；控件只提供 Name 时允许退化为整个控件
  矩形，但必须标明 `Control` 粒度。

## Privacy And Failure Semantics

- 每次请求绑定一个当前授权 Target Instance；选区跨到其他进程、系统保护界面或 Glyphshift 自身时
  拒绝，不扩大为桌面全局读取权限。
- UIA 密码元素与属性读取失败继续 fail-closed。Visual 取词只能由用户显式触发，截图与 OCR 中间图像
  默认只驻留内存，不进入 Observation Index、本地证据或未来网络翻译请求。
- Provider 超时、元素失效、窗口移动和 OCR 无文本都只结束本次请求，不影响目标软件和持续 Probe。
- Automatic 允许一个 Provider 失败后尝试另一个；已有非空块必须保留为部分成功，不能因次要 Provider
  失败丢弃全部结果。
- 对外只返回稳定错误类别：目标不匹配、权限拒绝、无文字、Provider 不可用、超时、已取消；不暴露
  任意 UIA/OCR 内部字符串。

## Verification Gate

- UIA Fixture：Point 获得控件矩形；支持 TextPattern 时能按点取得 Word，Text Range 能跨多个显示行
  返回多个 anchors；只有 Name 的控件明确退化为 Control。
- Geometry Fixture：Per-Monitor DPI、多显示器负坐标、窗口移动和滚动后重新获取仍能与目标像素对齐。
- Visual Fixture：Region 只截取授权目标表面，OCR 文本块有坐标与置信度；其他窗口和 Glyphshift 浮层
  不进入输入。
- Privacy Fixture：密码、跨进程区域、受保护窗口和截图持久化均被拒绝。
- Lifecycle Fixture：取消、目标退出、Provider 超时和连续快速请求都有界；慢 Provider 不阻塞 App UI。
- Composition Fixture：同一 Acquisition Result 可分别交给本地 Dictionary、合成 Translation Provider
  和无窗口 Presenter Fake，不要求三者互相认识。

## Next

实现仍延后。恢复本切片时先交付 UIA Point / Text Range 的纯取词合同和 Geometry Fixture，不先做全局
热键、浮层 UI、在线翻译或 OCR；只有结构化 Interface 通过后，再用同一 Interface 接入授权窗口截图与
OCR Adapter。
