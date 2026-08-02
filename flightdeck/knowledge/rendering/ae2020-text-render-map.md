# AE 2020 文字渲染路径图（实证）

实测环境: AE 2020 (17.7, `AE_CApplication_17.7`), Win10, 注入在 AE 启动**之后**。

## 结论速查

AE 2020 UI 用 **Direct2D + D3D11**(d2d1/d3d11/dxgi/DWrite 全加载,无 OpenGL)。文字按区域分三类路径:

| 区域 | 路径 | 拦截点 |
|---|---|---|
| 弹出菜单 / 菜单激活态菜单栏 | Windows **原生菜单** = GDI | `ExtTextOutW`(带 `ETO_GLYPH_INDEX`) ✅ |
| 对话框文本(新建合成等) | GDI(原生控件) | `ExtTextOutW`(字形索引,字体≠菜单) ✅ |
| **面板/时间轴/属性/效果名/对话框** | **GDI+**(`gdiplus.dll`):DWrite 仅做字形整形(GGI 收明文),**绘制走 `GdipDrawString`** | ✅ hook `GdipDrawString`(收明文宽串),详见 [GDI+ 效果路径](effect-controls-cjk-source-route.md) |

## 探针实证（面板重绘时命中数）

注入后缩放窗口强制面板重排版,统计各拦截点命中:

| 拦截点 | vtable idx | 面板命中 | 说明 |
|---|---|:--:|---|
| GDI `ExtTextOutW` | — | **0** | PrintWindow 强制整窗重绘,0 次调用 = 面板完全不走 GDI |
| DWrite `CreateTextLayout` | IDWriteFactory[18] | **0** | AE 全程不用 text layout |
| DWrite `CreateGdiCompatibleTextLayout` | IDWriteFactory[19] | **0** | 同上 |
| D2D `DrawGlyphRun` | ID2D1RenderTarget[29] | **0** | (经自建 ID2D1DeviceContext 取共享 vtable) |
| DWrite `CreateGlyphRunAnalysis` | IDWriteFactory[23] | **0** | 非自合成 alpha 纹理路径 |
| **DWrite `GetGlyphIndices`** | **IDWriteFontFace[11]** | **98**(上限 400) | ✅ **运行时触发,收真 Unicode 码点 = 明文入口** |
| DWrite `GetGlyphRunOutline` | IDWriteFontFace[14] | **0** | ⚠ 第二轮(2026-06-26)实测:面板重绘**不触发**(详下) |

→ 6 个标准入口全空、唯独 `GetGlyphIndices` 满命中:AE 用 DWrite **整形**(text→glyph)但**绘制端不走任何被 hook 的 D2D/DWrite 画字 API**——包括原以为是绘制出口的 `GetGlyphRunOutline`。

## ⚠ GetGlyphRunOutline 不触发（2026-06-26 实证，推翻原绘制端假设）

加 IDWriteFontFace[14] 探针、缩放主窗口强制面板重排版:**GGO=0 命中**,而**同一张 vtable** 的 GGI(idx 11)同轮命中 98 次 → hook 装载有效,GGO 是真没被调用。
结论:**绘制端替换走 GetGlyphRunOutline 这条路在「启动后注入」下行不通**。

**未证实的推断(hypothesis,非观测)**:面板字形栅格化 = AE 启动时一次性建字形图集(atlas),其 GetGlyphRunOutline/栅格化发生在注入**之前**;运行时缩放只重跑**整形(GGI)**取 advance/排版,像素从 atlas 复用 → 故注入后看不到任何绘制端调用。也可能 AE 用**自带栅格器**(非 DWrite outline)。两者都使「启动后 hook 绘制端」无解。

## GetGlyphIndices 抓到的明文（GGI 上限 80→400 后，本项目核心词全部明文可见）

提高上限后捕到面板/属性区**核心词汇**(不再只是字体名):
- 工作区栏: `"Workspace:"` `"Default"` `"Standard"` `"All Panels"` `"Essential Graphics"` `"Motion Tracking"`
- 面板标签: `"Effect Controls"` `"Composition"` `"Footage"` `"Effects"`
- 时间轴列头: `"Source Name"` `"Name"` `"Mode"` `"TrkMat"` `"Parent & Link"` `"Type"`
- 字符/段落面板: `"Regular"` `"Metrics"` `"Ligatures"` `"Hindi Digits"` `"Auto"` `"px"` `"100"` `"%"`
- 其他: `"8 bpc"` `"(none)"` `"Snapping"` …(每条后常跟若干空格 + `"."`,AE 排版填充)

→ **面板/属性区文字 = 走 GetGlyphIndices,明文运行时可读**,line 37 旧推断(同走此路)已证实。

## 对汉化的含义（真相:绘制层 = GDI+，不是死路）

- 上面 6 个 DWrite/D2D 探针全 0 + GGO=0,当时误判"绘制端死路、要 font-remapping/开机注入"。
- **真相**(backtrace `AfterFXLib → dvaui → gdiplus`):AE 面板/效果/对话框文字由 **GDI+ `GdipDrawString`** 绘制(收明文宽字符串),DWrite 只是 GDI+ 内部做字形整形(故 GGI 命中)。**没探 GDI+ 才以为死路**。
- ⇒ 汉化已实现:hook `GdipDrawString` 收明文 → 换中文。详见 [GDI+ 效果路径](effect-controls-cjk-source-route.md)。
- `GetGlyphIndices` 处不能直接换(out 字形数 = 调用方 count、UI face 无中文字形 .notdef);font-remapping/开机注入/TryGetFontTable 那几条**已不需要**(GDI+ 这条更简单且跑通)。

## vtable 索引备忘（d2d1/dwrite，IUnknown 占 0-2）

- IDWriteFactory: CreateTextLayout=18, CreateGdiCompatibleTextLayout=19, CreateGlyphRunAnalysis=23, CreateTextFormat=15
- IDWriteFontFace: GetGlyphIndices=11, GetGlyphRunOutline=14, GetGlyphCount=9
- ID2D1RenderTarget: DrawText=27, DrawTextLayout=28, DrawGlyphRun=29

取共享 vtable 的技巧:自己 `DWriteCreateFactory` / 建 `ID2D1DeviceContext`,读其 vtable 槽 = dll 里共享实现,inline-hook 该地址 = hook AE 的同名调用。
