# ZBrush

当前决定：已确认现有生产适配器没有覆盖 ZBrush 2026 的常规界面文字路径；当前项目规则禁止新增软件专属适配器，因此后续只继续寻找能够抽象成通用框架 / 渲染文字入口的“字符串 → glyph/纹理”上游 seam。找到可复用入口前维持“尚不支持”。

## 已验证事实

- 最新 Runtime Bundle 的 13 个生产适配器可以发现并激活 ZBrush 目标，generation 已应用，但重复强制重绘与收集仍稳定得到 `observedCount = 0`、`droppedObservations = 0`。因此不是前端过滤、目标发现或观察队列丢弃导致的 0 条。
- ZBrush 主进程加载 Qt 6.5.3 Core/Gui/Widgets。Qt Painter 适配器依赖的 QPainter、QString、QApplication、QWidget 和 QArrayData 精确 MSVC 导出均存在，Qt ABI/符号解析不匹配已排除。
- 运行时只读计数探针同时观察 `QPainter::drawText`、`QTextLayout::draw`、`QPainter::drawGlyphRun`、`QPainter::drawStaticText` 以及 FreeType glyph 接口；连续强制重绘期间这些入口全部为 0 次。现有屏幕文字的正常重绘阶段已经在复用更下游的缓存图形/glyph 资源。
- ZBrush 主程序不直接导入 TextOutW、ExtTextOutW、DrawTextW、GdipDrawString、DWriteCreateFactory 或 QPainter::drawText，但直接导入 `FT_Load_Char`、`FT_Get_Char_Index` 等 FreeType 接口。主进程没有发现承担界面绘制的 ZBrush 子进程。

## 已知与未解决

最符合现有证据的路径是：ZBrush 自有界面层在更早阶段把字符串经 FreeType/自有字体逻辑变成 glyph 或纹理，再由自绘/GPU 路径复用；这使当前以标准文字 API 为切入点的适配器看不到完整字符串。FreeType 本身只到字形级，直接 Hook 它不足以可靠做整句翻译。后续候选只有在能形成跨软件或跨同类渲染栈复用的通用适配路线时才进入实现。

目标当前已有进程驻留组件，改成单一 Qt Painter 计划会返回 `runtime.target_restart_required`。本轮未自动重启 ZBrush，避免破坏用户未保存状态。

## 恢复时先做

在一次可安全重启的 ZBrush 会话里，先建立干净的单适配器基线；随后从 `FT_Load_Char` / `FT_Get_Char_Index` 的调用点向上回溯，寻找仍持有完整 UTF-8/UTF-16 字符串、且可以抽象为通用渲染/文字入口的 seam。找到候选后先做只观察探针，要求能稳定采到当前界面的完整标签，再决定是否值得扩展现有通用适配器。
