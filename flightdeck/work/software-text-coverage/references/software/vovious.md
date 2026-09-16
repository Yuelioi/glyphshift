# Vovious

当前决定：**JUCE 6.1.6 canonical 三入口 profile 已判定 No-Go**。静态门槛没有找到可证明的
`addJustifiedText` canonical 入口，因此不继续 observation-only Hook，不建立生产 Adapter 或支持声明。

## 已知

初查发现静态 JUCE 6.1.6、TextEditor/Typeface/LowLevelGraphicsContext 等类型信息，没有可用文字函数
导出。现有通用文字 Adapter 实测 0 命中；DirectWrite 模块加载及 `WindowsDirectWriteTypeface` 标记不等于
文字经过当前 `CreateTextLayout` 适配器。

JUCE 6.1.6 一手源码确认标准 `Graphics` 单行、多行、`drawText` 与 `drawFittedText` 都在
`GlyphArrangement` 阶段仍持有完整 `juce::String`。首个 profile 候选为
`addCurtailedLineOfText` + `addJustifiedText` + `addFittedText` 三个 canonical 布局入口，并用重入保护避免
内部嵌套重复观察。当前 JUCE 已改用 `ShapedText`，因此不能把 6.1.6 内部调用图通配到新版。

实际静态 locator 结果为：`addCurtailedLineOfText = 1`、`addFittedText = 1`、canonical
`addJustifiedText = 0`、`addLineOfText` 薄包装 `= 0`。目标内确实存在 CR/LF 分行与 32-byte glyph 记录的
相似逻辑，但它作用于调用者本地 `GlyphArrangement`，属于内联 / 组合路径，不能作为通用 JUCE Hook 的
canonical 身份证据。非 JUCE PE 会被版本 / RTTI gate 拒绝。

## 恢复时先做

只有找到新的、可跨 JUCE 软件复用的完整字符串 callable seam，或另一个精确 JUCE 版本 / ABI 的真实目标
保留了 canonical 入口时才重开。当前目标不做软件专属地址补丁，也不继续 attach / Hook。

初查与官方资料见 [四目标初查](../../../adapter-coverage-and-x86/slices/four-target-survey.md)。
完整选型与版本边界见 [JUCE：下一个通用文字 Adapter 选取](../juce-adapter-selection.md)。
