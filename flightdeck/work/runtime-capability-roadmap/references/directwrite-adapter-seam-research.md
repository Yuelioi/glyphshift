# DirectWrite Adapter observe / replace Seam 调研

## 结论

DirectWrite / Direct2D 文字路径不是一个统一的“明文绘制函数”。可观察与可写回能力至少分成三层：

1. `ID2D1RenderTarget::DrawText`、`IDWriteFactory::CreateTextLayout` 和
   `CreateGdiCompatibleTextLayout` 直接接收 UTF-16，可以形成可靠的明文输入。
2. `DrawTextLayout` 只接收已经分析完成的 `IDWriteTextLayout`；调用点本身没有原文，且标准
   `IDWriteTextLayout` 没有取回或修改原文的接口。
3. `DrawGlyphRun`、`CreateGlyphRunAnalysis`、`GetGlyphRunOutline` 主要接收 glyph id、advance 和 offset。
   只有较新的 `ID2D1DeviceContext::DrawGlyphRun` 可选携带
   `DWRITE_GLYPH_RUN_DESCRIPTION`，而 description 可能为空。

因此不建议首版建立一个宣称覆盖所有 DirectWrite 路径的生产 Adapter。最小安全交付应先做一个窄
`Direct2D DrawText Inline Adapter`：在合成宿主验收后声明 `TextObserve + TextReplace`，复用原
`IDWriteTextFormat`、layout rect、brush、options 和 measuring mode；不声明 `FontSubstitute` 或
`LayoutAdjust`。`CreateTextLayout` / `DrawTextLayout` 路径先作为独立研究 seam，未解决 COM layout
身份、缓存、停用恢复和格式复制前，不把它提升为 `TextReplace`。

若产品命名必须保留为泛化的 `DirectWrite Adapter`，首版最多声明 `TextObserve`，而且不能把
`GetGlyphIndices` 命中直接发布成“已显示文字”；只有实际 draw 调用，或已被 draw 命中的 layout
身份关联，才能成为 Runtime Observation。

本结论不改变 AE 2020 的现有覆盖：仓库实证显示其面板重绘时 `CreateTextLayout`、
`CreateGdiCompatibleTextLayout` 和 `DrawGlyphRun` 没有命中，`GetGlyphIndices` 能看到 Unicode，但真实
可替换出口是 GDI+ `GdipDrawString`。DirectWrite 原型是为其他目标软件扩展覆盖，不是 AE 当前汉化
路径的替代品（[AE 渲染路径实证](../../../knowledge/rendering/ae2020-text-render-map.md)、
[AE GDI+ 路径](../../../knowledge/rendering/effect-controls-cjk-source-route.md)）。

## 1. 哪些调用点拥有文字

| 调用点 | 输入事实 | observe 判断 | replace 判断 |
|---|---|---|---|
| `ID2D1RenderTarget::DrawText` | `WCHAR* + UINT32`，官方定义为要绘制的 Unicode 字符数组 | 强；它是实际 draw 调用 | **首选**；可在一次同步调用内替换指针和长度 |
| `IDWriteFactory::CreateTextLayout` | `WCHAR* + UINT32 + IDWriteTextFormat + maxWidth/maxHeight`，允许 embedded NUL | 强，但它证明“创建 layout”，不必然证明 layout 最终被绘制 | 排版质量好，但会把决定烘焙进 COM layout，生命周期风险高 |
| `CreateGdiCompatibleTextLayout` | 同样接收完整字符数组，另有 `pixelsPerDip`、transform、`useGdiNatural` | 同上 | 同上，且必须原样保留 GDI 测量参数 |
| `ID2D1RenderTarget::DrawTextLayout` | 只有 `IDWriteTextLayout*`、origin、brush、options | 调用点没有原文；必须和创建时身份关联 | 需要替代 layout 或代理对象，不是换一个字符串参数 |
| `IDWriteTextLayout::Draw` → `IDWriteTextRenderer::DrawGlyphRun` | `Draw` 自身没有原文；renderer callback 会收到 glyph run 与 `DWRITE_GLYPH_RUN_DESCRIPTION*` | description 中含有关联 UTF-16、长度、cluster map 和 text position | 要替换必须重新 shaping，并保持 decorations、inline object 和 drawing effect；不适合首版 |
| `ID2D1RenderTarget::DrawGlyphRun` | `DWRITE_GLYPH_RUN` 只有 font face、glyph ids、advances、offsets、方向 | 不能还原可靠原文 | 不适合通用翻译写回 |
| `ID2D1DeviceContext::DrawGlyphRun` | 除 glyph run 外有**可选** description；Windows 8 / Windows 7 Platform Update 起可用 | description 非空时可观察关联文本；为空时仍只有 glyph | description 不参与渲染；替换仍需自行生成新的 glyph run |
| `IDWriteFontFace::GetGlyphIndices` | UCS-4 code points → nominal glyph indices | 可做诊断探针，但没有 layout/draw 身份，且可能只是字体查询 | 输出元素数受调用方 `codePointCount` 固定；不支持通用变长翻译 |
| `IDWriteTextAnalyzer::GetGlyphs` / `GetGlyphPlacements` | shaping 阶段接收原始 UTF-16、script、locale、feature ranges 和调用方输出缓冲区 | 可见文本，但可能重复、分 run，且不等于实际 draw | 修改长度会使 cluster map、feature ranges、输出上限及调用方状态失配 |
| `CreateGlyphRunAnalysis` / `GetGlyphRunOutline` | glyph run 或 glyph ids | 只有 glyph | 只有重建完整 glyph run 才可能写回 |

直接来源：

- [`ID2D1RenderTarget::DrawText`](https://learn.microsoft.com/en-us/windows/win32/api/d2d1/nf-d2d1-id2d1rendertarget-drawtext%28constwchar_uint32_idwritetextformat_constd2d1_rect_f__id2d1brush_d2d1_draw_text_options_dwrite_measuring_mode%29)
  明确接收 Unicode 字符数组、长度、format 与 layout rect。
- [`IDWriteFactory::CreateTextLayout`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritefactory-createtextlayout)
  与 [`CreateGdiCompatibleTextLayout`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritefactory-creategdicompatibletextlayout)
  接收显式长度字符数组并返回 `IDWriteTextLayout`。
- [`ID2D1RenderTarget::DrawTextLayout`](https://learn.microsoft.com/en-us/windows/win32/api/d2d1/nf-d2d1-id2d1rendertarget-drawtextlayout)
  只绘制传入的 layout object。
- [`DWRITE_GLYPH_RUN`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ns-dwrite-dwrite_glyph_run)
  不含原文；[`DWRITE_GLYPH_RUN_DESCRIPTION`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ns-dwrite-dwrite_glyph_run_description)
  才含 UTF-16、cluster map 与 text position。
- [`ID2D1DeviceContext::DrawGlyphRun`](https://learn.microsoft.com/en-us/windows/win32/api/d2d1_1/nf-d2d1_1-id2d1devicecontext-drawglyphrun)
  的 description 是可选参数，且官方说明它不参与渲染。
- [`IDWriteTextRenderer::DrawGlyphRun`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextrenderer-drawglyphrun)
  是 `IDWriteTextLayout::Draw` 触发的应用回调。
- [`IDWriteFontFace::GetGlyphIndices`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritefontface-getglyphindices)
  只是 UCS-4 到 nominal glyph 的映射；官方特别说明 OpenType substitution 可能使它不同于最终渲染映射。
- [`IDWriteTextAnalyzer::GetGlyphs`](https://learn.microsoft.com/zh-cn/windows/win32/api/dwrite/nf-dwrite-idwritetextanalyzer-getglyphs)
  与 [`GetGlyphPlacements`](https://learn.microsoft.com/zh-cn/windows/win32/api/dwrite/nf-dwrite-idwritetextanalyzer-getglyphplacements)
  展示了 shaping 输入、cluster map、feature range 和 caller-owned buffer 合同。

### Observation 语义

`CreateTextLayout` 发生时，应用可能缓存而不绘制、反复绘制或随后丢弃 layout。因此：

- `DrawText` 可直接发布 `TextObservation`。
- `CreateTextLayout` 只能先形成内部 `LayoutCandidate`；只有 `DrawTextLayout` 或
  `IDWriteTextLayout::Draw` 命中同一个 COM identity 后才升级为实际 Observation。
- `GetGlyphIndices` 只进入 bounded diagnostics/probe，不进入 Runtime Observation Stream。仓库在 AE 中
  已证明它能看到字符串，但也证明该命中本身不是可替换绘制出口。

## 2. 调用链与替换可行性

### 2.1 `DrawText`：首版最深的 seam

`DrawText` 一次调用同时具备原文、长度、format、layout rect、brush、draw options 与 measuring mode。
Adapter 可以：

1. 按显式 `stringLength` 有界读取 UTF-16，不依赖 NUL 终止。
2. 产生观察并查询完整 Runtime Publication。
3. `Pass` 时原参数调用 trampoline 恰好一次。
4. `Replace` 时只替换 UTF-16 pointer/length，其他参数原样转发；replacement buffer 保持到原调用返回。

这条路径无需持有应用 COM object，也不会让已创建对象跨 activate/deactivate 保留旧决定。停用只需像现有
native adapter 一样先把原子 active bits 清零；detour 可以继续安装但必须纯 pass-through。

限制是 `DrawText` 返回 `void`，Direct2D 把绘制错误延迟到 `EndDraw` 或 `Flush` 报告，Adapter 无法在
同一调用中发现“翻译绘制失败”后再重画原文。该行为由
[`DrawText`](https://learn.microsoft.com/en-us/windows/win32/api/d2d1/nf-d2d1-id2d1rendertarget-drawtext%28constwchar_uint32_idwritetextformat_constd2d1_rect_f__id2d1brush_d2d1_draw_text_options_dwrite_measuring_mode%29)
的返回合同明确说明。因此这里的 fail-open 只能覆盖输入验证、Runtime/decision、重入与 hook 自身错误；
不能承诺覆盖随后才在 `EndDraw` 暴露的设备/绘制失败。

### 2.2 `CreateTextLayout`：可排版，但替换被烘焙进对象

在 layout 创建前换入译文有一个明显优点：DirectWrite 会针对译文重新执行 script analysis、shaping、
font fallback、wrapping、trimming 和 metrics，而不是把中文硬塞进旧 glyph run。

但 `IDWriteTextLayout` 同时封装文本和指定 range 的格式。微软文档明确说明 layout 中的文本创建后不能
修改，要换文本必须创建新 layout；format range 又以字符位置和长度表示
（[Text Formatting and Layout](https://learn.microsoft.com/en-us/windows/win32/directwrite/text-formatting-and-layout)）。
由此产生四个不能忽略的问题：

- **停用不恢复：** 激活期间直接用译文创建的 layout 在停用后仍然是译文，直到应用自己销毁并重建。
- **range 失配：** 应用可能在创建后按原文 index 调用 `SetFontWeight`、`SetTypography`、
  `SetDrawingEffect`、`SetInlineObject` 等；变长译文会让这些区间落到错误字符或越界。
- **交互语义失配：** hit testing、caret、selection 和 accessibility 若仍以原字符串位置工作，会和译文
  layout 的 metrics 不一致。
- **GDI-compatible 参数必须保留：** `pixelsPerDip`、transform 与 `useGdiNatural` 共同决定布局度量，
  不能降级成普通 `CreateTextLayout`。

微软官方 PadWrite 的 [`EditableLayout.cpp`](https://github.com/microsoft/Windows-classic-samples/blob/main/Samples/Win7Samples/multimedia/DirectWrite/PadWrite/EditableLayout.cpp)
为“修改 layout 文本”建立了 forwarding adapter：内部重建 layout，并逐项复制 font collection、family、
locale、weight/style/stretch、size、underline、strikethrough、drawing effect、inline object 与 typography。
这是一手证据，说明保留多格式 layout 语义是独立模块，不是一次字符串换参。

### 2.3 `DrawTextLayout`：没有原文，生命周期映射不应偷懒

`DrawTextLayout` 只拿到 `IDWriteTextLayout*`。可考虑的三种替换实现都需要额外证明：

1. **创建时直接返回译文 layout：** 最简单，但违反停用后立即 pass-through，并有上述 range/交互风险。
2. **保留原 layout，关联一个译文 layout，draw 时二选一：** 需要管理两个 COM 对象和准确回收。借用裸
   pointer 会有悬空与地址复用；为 map 持有强引用又会阻止对象销毁。
3. **返回代理 `IDWriteTextLayout`：** 可以同时持有原/译文对象并控制 identity，但代理必须满足
   `QueryInterface` 的静态接口集合、IUnknown identity、所有 layout 方法和可能的
   `IDWriteTextLayout1..4`；还必须验证 Direct2D 不依赖实现私有接口。

COM 允许用 `QueryInterface(IID_IUnknown)` 得到对象 identity，但新取得的 interface pointer 必须管理
`AddRef/Release`。微软同时警告 `AddRef/Release` 返回值在某些实现中不稳定，不应依赖它判断对象是否
最终销毁（[QueryInterface rules](https://learn.microsoft.com/en-us/windows/win32/com/rules-for-implementing-queryinterface)、
[Reference count rules](https://learn.microsoft.com/en-us/windows/win32/com/rules-for-managing-reference-counts)）。
因此“map 住 layout pointer，再 hook `Release()==0` 清理”不是可靠的首版生命周期方案。

### 2.4 `IDWriteTextLayout::Draw` 与 glyph callback

`IDWriteTextLayout::Draw` 把实际渲染交给调用方提供的 `IDWriteTextRenderer`；callback 会收到 glyph run
和关联 description（[`IDWriteTextLayout::Draw`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextlayout-draw)、
[微软 Custom Text Renderer 教程](https://learn.microsoft.com/en-us/windows/win32/directwrite/how-to-implement-a-custom-text-renderer)）。

若 hook `IDWriteTextLayout::Draw` 并包装 renderer，observe 可以读取 description；但 replace 需要重新
shaping 新文本、选择 font face、生成 glyph ids/advances/offsets/cluster map，并同步 underline、
strikethrough、inline object 与 drawing effect。垂直文字还要求 renderer 支持较新接口。这个 seam 可作为
未来专用 Renderer Adapter，不应伪装成简单的 DirectWrite inline replacement。

## 3. COM vtable 与版本边界

### 3.1 稳定的是 interface ABI，不是实现地址

COM 定义二进制 interface/vtable；所有 COM interface 的前三个 vtable entry 是 `QueryInterface`、
`AddRef`、`Release`（[`IUnknown`](https://learn.microsoft.com/en-us/windows/win32/api/unknwn/nn-unknwn-iunknown)）。
当前 Windows SDK 基础接口 `um/dwrite.h` 与 `um/d2d1.h` 的声明顺序给出：

| Interface | 方法 | 基础 interface slot（IUnknown 从 0 计） |
|---|---|---:|
| `IDWriteFactory` | `CreateTextLayout` | 18 |
| `IDWriteFactory` | `CreateGdiCompatibleTextLayout` | 19 |
| `IDWriteFactory` | `CreateGlyphRunAnalysis` | 23 |
| `ID2D1RenderTarget` | `DrawText` | 27 |
| `ID2D1RenderTarget` | `DrawTextLayout` | 28 |
| `ID2D1RenderTarget` | `DrawGlyphRun` | 29 |

这些 slot 对**指定 IID 的 ABI**有意义，不代表所有 factory/render target object 都共享同一个函数地址。
COM 版本扩展通常通过新 interface/IID 实现；微软的 COM 版本规则明确建议扩展功能时创建继承旧接口的
新接口，而不是修改旧方法（[RPC and COM versioning](https://learn.microsoft.com/en-us/windows/win32/rpc/the-versioning-theory-for-rpc-and-com)）。

因此 Adapter 必须：

- 用准确的 SDK interface 类型和调用签名，按 x86/x64 分别构建；不能把 derived interface slot 猜成
  base interface slot。
- 对实际对象使用 `QueryInterface` 发现 `IDWriteFactory` / `ID2D1RenderTarget` /
  `ID2D1DeviceContext`，不直接把任意 interface pointer cast 成另一个接口。
- 将从实际对象 vtable 看到的每个不同 target address 去重安装 hook；不能把“自建共享 factory 的
  函数地址”当成所有宿主对象的 Windows 合同。
- 保留 reentry guard；不同 render target 实现、shared/isolated DirectWrite factory 和不同线程可能
  同时命中同一 detour。

### 3.2 需要显式测试的边界

- `DWriteCreateFactory` 可以创建 shared 或 isolated factory，二者内部状态隔离；官方只承诺接口，不承诺
  实现入口相同（[`DWriteCreateFactory`](https://learn.microsoft.com/en-us/windows/desktop/api/dwrite/nf-dwrite-dwritecreatefactory)）。
- `IDWriteFactory1..7`、`IDWriteTextLayout1..4` 是不同世代接口。基础 slot 保留，但必须针对实际 IID
  QueryInterface；不能假设所有系统都有最新接口。例：`IDWriteFactory1` 最低 Windows 8 / Windows 7
  Platform Update，而 `IDWriteFactory7` 最低 Windows 10 build 17134
  （[`IDWriteFactory1`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_1/nn-dwrite_1-idwritefactory1)、
  [`IDWriteFactory7`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_3/nn-dwrite_3-idwritefactory7)）。
- `ID2D1DeviceContext::DrawGlyphRun` 是 Windows 8 世代追加的 overload，不能只 hook
  `ID2D1RenderTarget::DrawGlyphRun` 后声称覆盖带 description 的调用。
- Windows App SDK 的 DWriteCore 使用 `DWriteCoreCreateFactory` 和独立库，不经过系统
  `DWriteCreateFactory`；它应作为明确的后续兼容项，而不是静默归入首版
  （[DWriteCore overview](https://learn.microsoft.com/en-us/windows/win32/directwrite/dwritecore-overview)）。
- 启动后注入可能错过已经发生的 factory/layout 创建。若仅 hook factory export，无法追溯旧 layout；
  若用合成对象定位 shared implementation address，则必须把“该目标、该模块版本、该实现地址一致”作为
  probe 证据，而不是平台保证。
- Direct2D 有 single-threaded 与 multi-threaded factory。multi-threaded factory 会序列化 Direct2D
  调用，混合 D3D/DXGI 时还有显式锁和回调死锁风险
  （[Multithreaded Direct2D Apps](https://learn.microsoft.com/en-us/windows/win32/direct2d/multi-threaded-direct2d-apps)）。
  Hook 热路径不能在持有宿主绘制锁时做阻塞 IPC 或递归 Direct2D 绘制。

即使使用微软 Detours，其官方文档也要求 target、detour 与 trampoline 具有完全相同的签名和 calling
convention，并明确不对被 detour 修改的软件提供支持
（[Using Detours](https://github.com/microsoft/detours/wiki/Using-Detours)）。所以 vtable/function detour
兼容性必须由 GlyphShift 自己的合成合同与目标软件 smoke 负责。

## 4. 字体、format 与 layout 风险

### 4.1 复用原 `IDWriteTextFormat`

`DrawText` replacement 首版应复用调用方 format，不在 hook 内修改共享对象。`IDWriteTextFormat` 包含
font、size、locale、alignment、wrapping、trimming、line spacing 与 flow/reading direction；其中 font
family、collection、weight/style/stretch、size 和 locale 创建后不能修改，其他段落属性可变，而且官方
明确提示该对象可能不是 thread-safe
（[`IDWriteTextFormat`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nn-dwrite-idwritetextformat)）。

因此 hook 内原地改 format 会污染同一对象的其他消费者，并与宿主线程竞争。首版不这样做。

### 4.2 字体覆盖与 fallback

复用 format 不能保证主 font 覆盖译文。DirectWrite 较新接口支持 system/custom fallback；
`IDWriteTextFormat1::SetFontFallback` 在未设置时使用系统 fallback，但该接口最低 Windows 8.1
（[`IDWriteTextFormat1`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_2/nn-dwrite_2-idwritetextformat1)）。
基础 Windows 7 合同不能假定能配置这套接口。

由此得到：

- `TextReplace` 可以沿用宿主已有 fallback，但不能宣称执行 `FontSubstitute`。
- 若译文显示 `.notdef`，这是独立 Font Policy / Font Adapter 需求；不能在每条 Dictionary Entry 中修补。
- 复制一个 format 也不是简单 `Clone`。创建新 format 需要读回创建属性，再复制 alignment、wrapping、
  trimming、line spacing 等可变状态；更新世代还要复制 font fallback、vertical orientation 等扩展属性。

### 4.3 布局变化不是 Adapter 隐式承诺

译文改变 glyph 数、advance、line break、wrapping、trimming、hit-test metrics 与 caret positions。
`DrawTextLayout` 官方建议用于多格式、OpenType 与 hit testing，且缓存 layout 比每次 `DrawText` 更高效
（[`DrawText / DrawTextLayout` 选择](https://learn.microsoft.com/en-us/windows/win32/direct2d/id2d1rendertarget-drawtext)、
[`DrawTextLayout`](https://learn.microsoft.com/en-us/windows/win32/api/d2d1/nf-d2d1-id2d1rendertarget-drawtextlayout)）。

首版 `TextReplace` 只保证“在相同 layout box 和 format 下让 DirectWrite重新布局译文”，不保证控件尺寸
调整，也不声明 `LayoutAdjust`。真实截断案例与独立 layout contract 出现后再立项。

## 5. 最小合成宿主

### 5.1 宿主结构

做一个确定性的 Windows 测试 executable，以 WIC 内存 bitmap 作为无窗口 render target。微软官方说明
`CreateWicBitmapRenderTarget` 可把 Direct2D 绘制到 `IWICBitmap`，并提供 software rendering 路径
（[`CreateWicBitmapRenderTarget`](https://learn.microsoft.com/en-us/windows/win32/api/d2d1/nf-d2d1-id2d1factory-createwicbitmaprendertarget%28iwicbitmap_constd2d1_render_target_properties_id2d1rendertarget%29)、
[server-side WIC sample](https://learn.microsoft.com/en-us/windows/win32/direct2d/server-side-rendering-overview)）。

宿主至少绘制五行固定内容：

1. `ID2D1RenderTarget::DrawText`。
2. `CreateTextLayout` → `DrawTextLayout`。
3. `CreateGdiCompatibleTextLayout` → `DrawTextLayout`。
4. `CreateTextLayout` → `IDWriteTextLayout::Draw` + 最小 custom renderer，记录 callback description。
5. 预先 shaping 后直接调用 base `DrawGlyphRun`，证明该调用自身没有 Unicode。

可选 Windows 8+ 分支再创建 `ID2D1DeviceContext`，分别以 null/non-null description 调用 extended
`DrawGlyphRun`。首轮同时跑 shared 与 isolated DirectWrite factory；后续再增加 HWND/DXGI render target，
避免把 WIC implementation address 当成全部 Direct2D 实现。

确定性像素断言先使用目标字体稳定覆盖的短 ASCII 替换，证明 hook/lifecycle；CJK 字体与 fallback 另做
环境化 smoke，不能让 CI 依赖某台机器安装的字体。若保存本机图片或原始日志，只能进入
`local-test/evidence/`。

### 5.2 验收状态机

| 阶段 | 操作 | 必须证明 |
|---|---|---|
| Baseline | 未安装 hook 渲染 | 取得内存 bitmap hash；零 Adapter event |
| Installed / inactive | 安装 detour，但 active bits 为 0 | bitmap 与 baseline 完全一致；original 每次恰好一次 |
| Observe | 只激活 `TextObserve` | `DrawText` 产生一次有界观察；像素与 baseline 一致；glyph-only path 不伪造原文 |
| Replace | 发布精确字典并激活 `TextReplace` | `DrawText` 输出与独立渲染译文的 expected bitmap 相同；未知文本保持 baseline |
| Fail-open | Runtime 未就绪、decision error/panic、TLS reentry、超长输入、embedded NUL、队列满 | 所有路径不崩溃、不阻塞，原文恰好绘制一次；诊断允许丢弃 |
| Deactivate | 清零 active bits 后再次渲染 | 立即恢复 baseline，停止新 observation/replacement；detour 可保持安装 |
| Reactivate | 重复激活 | 不重复安装、不重复事件、不产生 trampoline 链 |

必须额外加入 layout 跨边界用例：

- inactive 创建 layout，active 时绘制；
- active 创建 layout，inactive 时绘制；
- 同一 layout 在 active/inactive 间反复绘制；
- 创建后应用 range formatting，再绘制；
- Release 所有应用引用后重复分配 layout，检查没有陈旧 identity 命中。

如果采用“创建时直接替换 layout”，第二、三项会暴露停用不恢复；如果采用 pointer map，最后一项会暴露
悬空与地址复用。除非这些合同全部通过，否则 layout seam 不声明 `TextReplace`。

并发合同至少包括：两个线程调用不同 render target、detour 内 Runtime reentry、activate/deactivate 与
in-flight draw 交错。实现应先清 active bits，再等待自身 in-flight callback 退出；即使物理 detach 失败，
也必须保持已安装 hook 的 pass-through，而不能让宿主处于半拆卸状态。

## 6. 首版 Feature 建议

### 推荐拆分

#### A. `Direct2D DrawText Inline Adapter`（首个可生产候选）

```text
features = [TextObserve, TextReplace]
apply model = inline, per draw call
unicode seam = ID2D1RenderTarget::DrawText
```

只有在上述合成宿主通过后才这样声明。它不声明：

- `FontSubstitute`
- `LayoutAdjust`
- `DrawTextLayout` replacement
- glyph-run reconstruction
- DWriteCore support

Descriptor / probe 必须把覆盖边界说成 `DrawText`，不能仅用 `Technology = DirectWrite` 暗示覆盖所有
DirectWrite UI。

#### B. `DirectWrite Layout Adapter`（研究态）

首轮不进入生产 Bundle。若为了 Probe 先实现：

```text
features = [TextObserve]
```

但 observation 只在 layout 实际进入 `DrawTextLayout` / `IDWriteTextLayout::Draw` 后发布；创建时字符串只
是内部 candidate。待 layout identity、释放、跨 activate/deactivate、format range 和目标软件实证全部
通过，再单独评估 `TextReplace`。

#### C. glyph-level hooks

`GetGlyphIndices`、base `DrawGlyphRun`、`CreateGlyphRunAnalysis` 和 `GetGlyphRunOutline` 首版只作为
diagnostic probe，不声明 `TextObserve`、`TextReplace`、`FontSubstitute` 或 `LayoutAdjust`。带 non-null
description 的 `ID2D1DeviceContext::DrawGlyphRun` 可以提供观察证据，但仍不自动获得安全写回能力。

### 进入生产 Bundle 的门槛

- 合成宿主完整证明 observe、replace、fail-open、activate/deactivate 与 original exactly-once。
- 目标软件 smoke 证明实际命中的是声明的 Unicode seam；仅加载 `dwrite.dll` 或仅命中
  `GetGlyphIndices` 不算覆盖。
- shared/isolated factory、实际 render target implementation 和目标 Windows 版本进入兼容证据。
- 任何只能在进程启动前捕获的 factory/layout 路径，必须在 Adapter 限制中显式说明；不能把 late attach
  的零命中解释成“不使用 DirectWrite”。
- 不把 local module address、vtable address、进程信息或截图写入 tracked 文档；真实证据留在
  `local-test/evidence/`。

