# Alias 本地技术入口与验证建议

本文保留初轮只读研究；后续真实注入、中文替换与恢复的验收结果见[实际测试结果](live-test-findings.md)。

## 已证实的事实

本轮对用户已打开的 Alias Concept 2027 做只读进程模块、磁盘 PE 导入/导出及资源标记检查；没有注入目标、修改安装文件或操作工程内容。

- 目标加载 Qt 6.8.3 的 Core、Gui、Qml、Quick、QuickTemplates2、Widgets 及 OpenGL 相关模块。
- 应用自身的 `libQuick.dll` 直接导入 `QQmlApplicationEngine` 构造与加载入口，以及 `QQuickItem`、`QQuickWindow`；不是仅凭安装目录中存在 Qt 得出判断。
- 同一界面库包含 303 个不同 QML 文件名标记以及菜单、按钮、组合框等 Qt Quick 类型标记。标记证明资源关联，不能证明每个资源当前可见或可翻译。
- `Qt6Quick.dll` 导出 `QQuickText`、`QQuickTextInput`、`QQuickTextEdit` 的 `text`、`setText` 和 `updatePaintNode`。`Qt6QuickTemplates2.dll` 导出按钮、Action、ToolTip 等文字 setter。
- 现有 Qt Painter 需要的四个 `QPainter::drawText` 入口及 Qt 6 `QString` 构造、size、utf16 导出存在。应用界面库还导入了 QPoint 版本的 drawText，但没有动态命中证据，不能认定现有 Hook 已覆盖该调用。

原始模块、导出/导入清单与字符串标记仅保留于本机忽略证据目录，不作为仓库必需资源。

## 路线判断

| 路线 | 本轮判断 | 下一步证据 |
|---|---|---|
| 现有 Qt Painter | 可作为部分绘制面的补充候选 | 动态命中与实际像素，不能由模块加载推导覆盖率 |
| 通用 Qt Quick 标签适配 | 主路线；官方文字属性与本机符号均提供可探索入口 | 活动对象、完整原文、UI 线程、QML binding 和销毁恢复 |
| QTranslator / retranslate | 若目标实际使用翻译绑定，可成为干净入口 | 先证明目标文字经过翻译服务；二进制关键词缺失不能排除编译后的绑定 |
| Autodesk 插件 SDK | 可扩展自身 UI，不据此推导能翻译所有内置 UI | 见官方资料研究；不以每软件插件作为默认方案 |
| OpenGL / 字形层 | 暂不优先；到纹理或字形阶段可能失去完整原文 | 仅在上层入口证实不足后研究 |

## 最小实验顺序

1. **只观察**：在匹配的 Qt 6 动态运行时下找到窗口与活动 Quick 对象；遍历需兼顾 QObject 所有权树与 QQuickItem 视觉树，去重并限定范围。读取非编辑文字属性，覆盖已打开界面和后续出现的菜单、Tooltip。建立 UI 线程调度与安全对象寿命规则。
2. **验证 setter 和绘制路线**：判断可否选择持有完整字符串且不改应用原对象的绘制/布局入口。如果必须属性写回，单独证明 binding 保留、textChanged 副作用可接受、目标更新不被旧快照覆盖。不得把 QObject 元属性读写当成无风险替换。
3. **合成合同**：覆盖晚附加、属性绑定重算、原文更新、对象销毁、重复文本、译文第二代与停用恢复；使用一个明确缺少中文字形的源字体验证回退、布局宽度和裁剪。
4. **真实窄验收**：选一个静态标签、一个动态菜单和一个提示框；证明采集非零、首代中文可见、第二代可见、停用恢复。真实原始输入输出只存本地证据目录。涉及桌面 App 重建/启动时使用 `scripts/review-app.ps1`。

## 尚未解决

- 当前目标中标准 Quick Text、自绘内容与其他文字面的实际占比。
- 从已运行应用安全取得完整 Quick 对象集合与 UI 线程调度的具体实现。
- Qt 私有 C++ 类跨版本 ABI、原文快照、属性绑定及缓存重绘的一致性。
- 中文字体回退、固定尺寸控件、快捷键助记符、富文本的可见效果。

本轮结论是“已有明确且可验证的技术路线”，不是“已支持 Alias”。

## 依据

- [现有 Qt Painter 实现](../../../../crates/adapters/implementations/framework/qt-painter-native/src/lib.rs)
- [真实目标晋级标准](../../../knowledge/rendering/runtime-text-adapter-validation.md)
- [Qt 与 Autodesk 官方资料](primary-source-research.md)
