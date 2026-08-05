# GTK/Pango 之后的通用文字入口评审

Status: Complete

## 结论

下一轮只进入一个有界诊断：**动态 Tk 8.6 的公开文字绘制链与现有 GDI Adapter 的双路对照**。
Tk 具备公开、稳定的 C API，标准 classic/ttk 控件会把 UTF-8 文字送入 `Tk_ComputeTextLayout`、
`Tk_DrawTextLayout` 与 `Tk_DrawChars`；这些入口比最终 GDI 字体 run 更靠近应用原文。但 Windows Tk
最终仍会按字体切段并调用 `TextOutW`/`TextOutA`，因此它可能只是现有 GDI 覆盖的重复层。

只有以下任一差异被确定性宿主证明时，Tk Adapter 才继续实现：

- Tk 层稳定取得完整 Dictionary 词条，而 GDI 只得到无法命中的字体 run 或片段；
- 同一条 `source -> target` 在 Tk 层能由 Tk 自己重新布局并选择中文 fallback，而 GDI 写回出现缺字、
  错误测量或不可接受的分段结果。

若现有 GDI 已能完整观察、正确显示译文、热更新并停用恢复，则 Tk 作 **No-Go**，不进入 Runtime
Bundle，也不增加一个只按框架名称区分的重复选项。

## Tk 为什么值得做一次诊断

Tk 的公开声明文件明确把 `Tk_ComputeTextLayout`、`Tk_DrawTextLayout`、`Tk_DrawChars` 与
`Tk_FreeTextLayout` 列为受支持的导出接口，并保留稳定的 stubs 索引。文字布局文档说明布局由 UTF-8
字符串与字体计算，之后交给绘制和释放函数使用。
([Tk public declarations](https://github.com/tcltk/tk/blob/core-8-6-branch/generic/tk.decls),
[Tk TextLayout C API](https://www.tcl-lang.org/man/tcl8.6.16/TkLib/TextLayout.htm))

官方 ttk Label 源码给出了标准控件的真实路径：Text/Label 元素从 `-text` 取得字符串，调用
`Tk_ComputeTextLayout`，随后用 `Tk_DrawTextLayout` 绘制并用 `Tk_FreeTextLayout` 释放；Button 等使用
同一个 ttk text/label element。generic font 实现继续把 layout 中仍保留的 UTF-8 区间交给
`Tk_DrawChars`。
([ttk Label source](https://github.com/tcltk/tk/blob/core-8-6-branch/generic/ttk/ttkLabel.c),
[generic font source](https://github.com/tcltk/tk/blob/core-8-6-branch/generic/tkFont.c))

这条链可以采用不改目标控件状态的绘制时方案：保留应用自己的 layout，命中 Dictionary 时临时计算
译文 layout 并只替换本次完整绘制；下一次绘制自然读取最新 Dictionary，停用后直接绘制原 layout。
它比 `SetWindowTextW` 更符合 GlyphShift，因为不会改按钮标题、输入值或窗口业务状态。

## 与现有 GDI 的重叠

Windows Tk 的平台字体实现会先按 fallback font 和长度切分 UTF-8，再转换为目标编码并调用
`TextOutW`/`TextOutA`。因此短、单字体的普通界面文字很可能已经被现有 `TextOutW` Adapter 完整覆盖；
Tk 层的潜在增量只在分段前原文和 Tk 自身字体 fallback。
([Tk Windows font source](https://github.com/tcltk/tk/blob/core-8-6-branch/win/tkWinFont.c))

本地动态 Tk 8.6 构建的 PE 导出检查确认上述公开函数可按名称解析；这只证明诊断可实施，不等于任何
具体软件已获得实时翻译支持。

## 有界原型范围

- 只接受 Windows x64、动态 Tk 8.6，且公开导出唯一可解析；静态 Tk、Tk 9 与私有符号首轮拒绝。
- 优先比较 ttk Label/Button、classic Label、Menu 与 Canvas 的完整短文本；输入框选区、Text 富文本、
  多行局部绘制和 mnemonic 索引首轮 fail-open。
- Tk 路径只在完整 layout 绘制或完整 `Tk_DrawChars` 调用中替换；不修改应用持有的 layout、控件值或
  Tcl 变量。
- 验收必须同时覆盖首版译文、第二代 Dictionary、停用恢复、无模块拒绝和与 GDI 的观察/像素对照。
- 所有本机运行库、路径、进程、截图与原始日志只保存在 `target/local-test/`。

## 其余候选

| 候选 | 判断 | 原因 |
| --- | --- | --- |
| `SetWindowTextW` / `WM_SETTEXT` | No-Go | 修改控件状态而非本次绘制，可能触发校验、事件与保存；注入前对象、热更新和安全恢复均不完整。微软也明确区分 `SetWindowText` 与窗口消息语义。([SetWindowTextW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowtextw), [WM_SETTEXT](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-settext)) |
| `LoadStringW` | No-Go | 只发生在资源加载时，调用方可长期缓存；不能保证已显示控件立即更新或停用恢复。([LoadStringW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadstringw)) |
| Java Swing/AWT/JavaFX | 当前 No-Go | 外部接入需要 Java agent/JVMTI/Instrumentation；动态 attach 是独立授权与运行时兼容产品面，且 Swing、Java2D、JavaFX 不共享一个公开绘制返回入口。([Java Instrumentation](https://docs.oracle.com/en/java/javase/21/docs/api/java.instrument/java/lang/instrument/package-summary.html), [JEP 451](https://openjdk.org/jeps/451)) |
| wxWidgets / VCL / LCL | 不新增 | Windows 标准控件与常见绘制大多继续下沉到 Win32/GDI；先由现有 Adapter 验证，不按框架名重复包装。 |
| SDL2_ttf | 当前 No-Go | 公开 C ABI 能取得完整 UTF-8 并生成译文 Surface，但调用方可缓存 Surface/Texture；不能保证既有文字立即热更新或停用恢复。 |
| SDL3_ttf | Roadmap 候选 | 新 `TTF_Text` 与 `TTF_Draw*Text` 具备绘制时完整原文，但需要真实动态目标证明采用率、属性复制和可见收益。 |
| Skia / Dear ImGui | 当前 No-Go | 常见部署为静态 C++，缺少统一动态模块、稳定 ABI 或可恢复的完整文字绘制入口。 |

## Decision

进入 [Tk 文字绘制链对照诊断](../slices/tk-text-draw-path-diagnostic.md)，先证明相对现有 GDI 的真实
增量。诊断通过前不修改生产 Bundle、Adapter 目录或软件支持声明。
