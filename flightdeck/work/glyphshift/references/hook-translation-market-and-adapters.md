# Hook 翻译工具与 Capability Adapter 调研

Status: Complete
Date: 2026-07-31

## 1. 结论

Glyphshift V2 的 Hook/写回机制应该实现为 **Capability Adapter**，不能在 Runtime
Kernel 或软件插件中写死 GDI、GDI+、DirectWrite、Qt、Unity 等分支。

这不是为了提前抽象未知需求，而是因为当前已经存在两个不同机制的 Adapter 候选：

- GDI `ExtTextOutW`
- GDI+ `GdipDrawString`

而公开工具和平台 API 又明确显示，后续可见翻译可能来自完全不同的执行模型：

- 同步拦截绘制函数并改写调用参数。
- 修改引擎或 UI Framework 的 Text Component。
- 修改 Chromium DOM。
- 安装应用 Framework 自己的翻译资源。
- 替换字体、纹理或其他资源，而不改文字。

因此 V2 Core 应固定的是 Observation、Translation Decision、Feature State、Generation/ACK
与 fail-open 等不变量；具体 API、函数入口、引擎版本和写回手段由可注册 Adapter 隐藏。

## 2. 调研范围与方法

只使用项目官方仓库、官方文档和平台 API 文档。选取的工具不是按下载量排名，而是覆盖四种
不同的扩展路线：

- 通用 Win32 文本函数注入。
- 多采集方式加可选嵌入式写回。
- 引擎/Text Framework 组件写回。
- Frida 脚本生态。

“支持翻译”在本文中必须区分：

- **Observe**：能够看到或导出文字。
- **Text Replace**：目标软件自身界面实际显示替换后的文字。
- **Font Substitute**：目标软件自身界面实际使用替换字体。
- **Resource Replace**：目标软件实际使用替换后的纹理或资源。

OCR、剪贴板、Overlay 和只输出到外部翻译窗口都不能证明目标软件内可见写回。

## 3. 代表性工具

### 3.1 Textractor

[Textractor 官方仓库](https://github.com/Artikash/Textractor)描述了典型的通用 Hook
提取架构：

- Host 把 `texthook` 注入目标进程。
- 注入模块拦截 `TextOut`、`GetGlyphOutline` 等文字函数。
- 目标进程与 Host 通过两条 Pipe 交换数据。
- Host 接收文字后分发给 Extensions。
- 支持已知引擎自动 Hook、手工 `/H` Hook Code、搜索候选 Hook Code 和扩展项目。

这证明两件事：

1. 自动识别不能覆盖所有软件，手工/搜索 Hook 是成熟工具的后备能力。
2. 文字采集后的处理扩展与目标进程 Hook 本身可以分离。

但其公开架构主要证明的是提取与外部扩展管线，并不能作为“目标软件内可见写回”的通用
证据。Glyphshift 不能把类似的 Pipe 有数据误报成正在翻译。

### 3.2 LunaTranslator

[LunaTranslator 官方文档](https://docs.lunatranslator.org/en/basicuse.html)同时提供
HOOK、模拟器 Hook、OCR、剪贴板等多种文字来源。其
[HOOK 设置](https://docs.lunatranslator.org/en/hooksettings.html)支持：

- Win32 通用 Hook，包括 GDI、D3DX 和字符串函数。
- 特殊 Hook Code。
- 延迟注入。
- Unity、Yuzu、PPSSPP、Vita3K、RPCS3 等 JIT/模拟器 Hook 形式。

更重要的是，其文字选择界面把 Display 与 Embed 分开；只有 Hook 支持嵌入时才显示
Embed。官方的
[嵌入式翻译说明](https://docs.lunatranslator.org/en/embedtranslate.html)描述了暂停
游戏、翻译、把结果写回内存再恢复运行的过程，也明确说明并非所有游戏都支持，错误写回
可能导致卡顿或崩溃。

嵌入式设置还把“修改游戏字体”和字体大小作为单独设置。这直接支持 Glyphshift 当前的
判断：

- Observe 不等于 Embed/Text Replace。
- 字体替换不依赖字典命中。
- 是否能写回是具体 Hook 的能力，不是软件级单一布尔值。
- 同一个软件可能需要不同 Hook Code、引擎 Adapter 或降级方式。

### 3.3 XUnity.AutoTranslator

[XUnity.AutoTranslator 官方仓库](https://github.com/bbepis/XUnity.AutoTranslator)不是
按 Win32 绘制函数处理所有 Unity 软件，而是分别适配：

- UGUI
- NGUI
- IMGUI
- TextMeshPro
- TextMesh
- FairyGUI
- Utage

它还适配 BepInEx、MelonLoader、IPA、UnityInjector 和 ReiPatcher 等不同加载环境，并根据
运行条件选择 Harmony 或 MonoMod Hook。其 IL2CPP 支持又有单独的能力缺口和兼容限制。

公开功能进一步把以下能力分开：

- 实时重载翻译文件。
- 开关已翻译/原始文字。
- UGUI 与 TextMeshPro 的字体覆盖或 fallback font。
- UI overflow/行距调整。
- 纹理扫描、导出、替换和 Resource Redirector 扩展。
- Translation Endpoint 插件。

这证明“Unity Hook”仍然太粗。即使是同一引擎，Text Framework、运行时、加载器、资源
类型和版本都会改变可用能力。正确的扩展单元应是带能力声明的 Adapter，而不是 Core
中的 `unity` 分支。

### 3.4 Agent

[Agent 官方仓库](https://github.com/0xDC00/agent)把自己定义为由 Frida 驱动的通用
脚本文本 Hooker；独立的
[官方脚本仓库](https://github.com/0xDC00/scripts)包含 Yuzu、Vita3K、PPSSPP、RPCS3、
Unity、MAGES 和 HCode 等库及大量逐游戏脚本。

它证明脚本生态可以快速扩展到模拟器、移动端、远程 Frida 和长尾游戏，不必等待主程序
发布。但对 Glyphshift 来说，任意脚本同时意味着目标进程内代码执行、地址/版本漂移、
供应链和用户授权风险。

V2 可以在 Adapter 契约中为未来的脚本 Host 保留位置，但首版不接受 Software Extension
直接携带未经注册、未签名的地址或 Hook 脚本。

## 4. 市场共同功能与对 Glyphshift 的含义

| 市场中的常见能力 | 观察到的实现 | V2 对应设计 |
|---|---|---|
| 自动 Hook | 引擎签名、通用 Win32 入口、组件扫描 | `probe()` 只报告证据，不直接宣告 Active |
| 手工 Hook/Hook Code | Textractor、LunaTranslator | 未来可作为受信任 Expert Adapter，不进入普通字典 |
| 多文字来源 | Hook、模拟器、OCR、剪贴板 | Observe/Capture 与 Replace 分开 |
| 嵌入式写回 | 内存写回、组件属性写回 | 独立 `text.replace` Feature |
| 字体替换 | 游戏字体、UGUI/TMP Font | 独立 `font.substitute` Feature |
| UI 适配 | 字号、overflow、行距 | 独立 `layout.adjust` Feature，不塞进字典 |
| 资源翻译 | 纹理/Asset Bundle 替换 | 未来 `resource.replace` Feature |
| 热重载 | 重新加载翻译文件 | Workspace Generation + Runtime ACK |
| 引擎/版本兼容 | Hook Code、Framework/Loader Adapter | Adapter ID、版本与动态能力协商 |
| 插件/脚本生态 | Extensions、Endpoint、Frida scripts | 注册表、签名、hash pin、隔离与明确授权 |

市场没有提供一个能覆盖所有软件的万能 Hook。相反，成熟工具都在增加引擎规则、Hook
Code、Framework Adapter、组件扫描或脚本。未来增加新 Hook 能力不仅可能，而且是产品
扩大软件覆盖面的必然路径。

## 5. 未来可见写回入口

### 5.1 DirectWrite / Direct2D

Microsoft 的
[Direct2D 文字绘制文档](https://learn.microsoft.com/en-us/windows/win32/direct2d/how-to--draw-text)
列出了 `DrawText` 与 `DrawTextLayout`；
[Direct2D/DirectWrite 关系说明](https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-and-directwrite)
还列出 `DrawGlyphRun` 和自定义 `IDWriteTextRenderer`。

因此 DirectWrite 很可能成为 Windows 软件覆盖面的下一组 Adapter，但不能只 Hook 一个
函数后声称完整支持：

- `DrawText` 能直接看到字符串。
- `DrawTextLayout` 接收已排版对象，需要恢复其文字、格式和生命周期。
- `DrawGlyphRun` 接收 glyph；官方文档明确说明 glyph 本身没有文字语义，字符与 glyph
  可能是多对多关系。

合理拆分可能是 `windows.direct2d.draw-text`、
`windows.direct2d.draw-text-layout` 和只提供有限能力的 glyph Adapter，而不是 Core
里的一个 `directwrite=true`。

### 5.2 Chromium / Electron DOM

Chrome DevTools Protocol 官方
[DOM Domain](https://chromedevtools.github.io/devtools-protocol/tot/DOM/)提供
`DOM.setNodeValue`、`DOM.setOuterHTML` 等修改接口，
[Runtime Domain](https://chromedevtools.github.io/devtools-protocol/v8/Runtime/)提供
`Runtime.evaluate`。

如果目标软件开放了受控 DevTools 连接，并且文字真实存在于 DOM 中，外部 Adapter 可以
修改 DOM，形成目标窗口内可见写回，不需要拦截 GDI/GDI+。限制包括：

- 必须能建立受授权的 Debug Protocol 会话。
- Canvas、WebGL、位图文字或封闭 Shadow/跨进程上下文不能按普通 DOM 处理。
- DOM 重渲染会覆盖修改，Adapter 需要稳定对象身份、Mutation 观察与恢复策略。

因此当前 CDP Capture 不应被称为翻译，但未来完全可以新增
`chromium.dom-text` Writeback Adapter。

### 5.3 Qt

Qt 官方
[QTranslator 文档](https://doc.qt.io/qt-6/qtranslator.html)提供应用翻译查找，
`QCoreApplication::installTranslator()`可以安装翻译器；官方
[QPainter 文档](https://doc.qt.io/qt-6/qpainter.html)则列出多种 `drawText` 入口。

这给出两条不同路线：

- 在应用确实使用 `tr()`/`QTranslator` 时，安装翻译资源。
- 在低层绘制时拦截 `QPainter::drawText`。

两条路线的对象、时机和恢复策略完全不同，应该是不同 Adapter/Apply Model，不能写成
一个 Core `qt` 特例。

### 5.4 Unity 与其他引擎

Unity TextMeshPro 官方 API 公开
[`TMP_Text.text`](https://docs.unity3d.com/ja/Packages/com.unity.textmeshpro%403.0/api/TMPro.TMP_Text.text.html)
属性；XUnity 的真实实现也证明组件写回、字体、布局、资源以及 Mono/IL2CPP 需要分别处理。

同样的模式会出现在 Unreal、WPF、WinUI、自绘框架或应用私有引擎：稳定写回位置可能是
组件属性、资源加载、Layout Object 或绘制调用。V2 需要允许 Adapter 选择最合适的 seam，
而不是强迫所有实现伪装成 `ExtTextOutW` 风格的同步函数 Hook。

### 5.5 Skia

Skia 官方
[`SkCanvas` API](https://api.skia.org/classSkCanvas.html)提供 `drawString` 与
`drawTextBlob`。它可能覆盖 Chromium Canvas、自绘客户端或跨平台 UI 的部分文字，但
Text Blob 与预排版 glyph 的还原限制类似 DirectWrite Glyph Run。

Skia 值得作为未来探针/Adapter 候选，不应在缺少可见写回实证时进入首批能力承诺。

### 5.6 UI Automation

Microsoft 的
[`ValuePattern.SetValue`](https://learn.microsoft.com/en-us/dotnet/api/system.windows.automation.valuepattern.setvalue)
只能修改支持 ValuePattern 且非只读的控件；官方文档还明确说明多行 Edit/Document
控件不支持相同方式。

所以 UIA 可以在特定“可编辑值”场景形成写入能力，但它不是通用 Label/Menu 翻译入口。
V2 当前继续把 UIA 标记为 Observe；未来若增加写入，也必须注册为范围受限的独立
Adapter，不得用 UIA 捕获数量提升 Text Replace 状态。

## 6. V2 Adapter 契约

### 6.1 三种插件角色

V2 应明确区分：

| 角色 | 负责什么 | 不负责什么 |
|---|---|---|
| Software Extension | 软件身份、Location/Context、翻译数据、Route Program、依赖声明 | 不实现通用 Hook，不让字典携带代码 |
| Controller Plugin | 安装发现、启动、Target Instance 识别、Session Recipe | 不处理每次绘制，不直接读写 Snapshot |
| Capability Adapter | Observe/Writeback/Font/Resource/Layout 的具体技术机制 | 不知道 AE、Premiere 等软件品牌或字典分类 |

Route Program 是无代码、无 I/O、执行有界的版本化声明数据。V2 首版不提供 target-process
Route Code Plugin；新的写回代码必须注册成 Capability Adapter，不能借路由绕过 Registry。

### 6.2 Adapter Descriptor

Adapter 通过注册表提供版本化 Descriptor。示意：

```json
{
  "schema": "glyphshift.capability-adapter/1",
  "id": "windows.gdiplus.draw-string",
  "version": "1.0.0",
  "abi": "glyphshift.runtime-adapter/1",
  "placement": "target-process",
  "apply_model": "inline-render",
  "features": [
    "text.observe",
    "text.replace",
    "font.substitute"
  ],
  "architectures": ["win-x64"]
}
```

稳定 Feature 语义由 Core 理解，但 Adapter ID 动态注册：

- `text.observe`
- `text.replace`
- `font.substitute`
- `layout.adjust`
- `resource.replace`

新增 Adapter 不修改 Core。新增 Core 尚不理解的 Feature 时，必须通过协议版本升级明确
引入，不能把未知字符串假装成 Active。

### 6.3 Apply Model

Adapter 还必须声明应用模型，因为并非所有写回都是同步绘制 Hook：

| Apply Model | 例子 | 特有不变量 |
|---|---|---|
| `inline-render` | GDI、GDI+、Direct2D | 同步有界；原函数恰好调用一次；重入保护 |
| `retained-object` | Unity Text Component、Qt Translator | 跟踪对象生命周期；更新与恢复原值 |
| `external-protocol` | Chromium CDP DOM | 授权连接；对象失效/重渲染恢复；异步 ACK |
| `observe-only` | 当前 UIA/CDP Capture | 永远不能报告 Replace/Font Active |

Core 的 Translation Decision 可以复用，但“恰好一次调用原函数”只约束
`inline-render` Adapter，不能错误套到 DOM/Component 写回。

### 6.4 Execution Placement

逻辑契约统一，执行宿主可以不同：

- `target-process`：由 Runtime Kernel 通过版本化 C ABI 加载，适合 GDI/GDI+/DirectWrite。
- `isolated-worker`：由 Service 使用 Named Pipe + Protobuf 管理，适合 CDP 等外部协议。

Controller Plugin 不能兼任长时间翻译 Adapter，否则软件控制与翻译能力又会粘回一个宿主
分支。首版没有需求的 Placement 不必实现，但协议必须允许协商和拒绝。

### 6.5 生命周期

Adapter Registry 与 Session Manager 使用小接口：

```text
descriptor() -> Adapter Descriptor
probe(Target Facts) -> Probe Evidence
activate(Session Binding, Requested Features) -> Active Features
update(Generation Input) -> ACK
deactivate() -> Pass/Restore + Status
diagnostics() -> Bounded Event Stream
```

`probe()` 成功只表示“可能可用”；只有 `activate()` 成功、实际 Feature 已安装并 ACK 当前
Generation 后，Session 才能报告 Active。

`Generation Input` 由 Placement 决定：target-process Adapter Host 原子接收不可变
Snapshot/Font Policy；isolated-worker 只收 Service Decision Engine 产生的、带
Observation ID 与 Generation 的 Decision，不要求 Worker 读取完整 Catalog。

`deactivate()` 对注入式 Adapter 只切换 Pass，不要求远程卸载 DLL；对象/协议 Adapter
则按自己的 Apply Model 恢复或停止维护写回。

### 6.6 软件扩展只声明依赖

Software Extension 的 Session Recipe 只引用能力，不指定函数地址或 Core 类型：

```json
{
  "capabilities": [
    {
      "adapter": "windows.gdi.ext-text-out",
      "features": ["text.replace"]
    },
    {
      "adapter": "windows.gdiplus.draw-string",
      "features": ["text.replace", "font.substitute"]
    }
  ]
}
```

Service 验证 Adapter 已注册、版本兼容、签名可信、架构匹配且用户已授权。Runtime Kernel
通过 Registry 激活，不允许出现：

```text
if software == "after-effects" ...
match adapter_id { "gdi" => ..., "gdiplus" => ... }
```

首批 GDI/GDI+ 也必须走与未来 Adapter 相同的 Descriptor、Registry、Capability
Negotiation 和状态模型。可以由第一方发布和签名，但不能享有 Core 私有分支。

## 7. 安全边界

- Translation Catalog 与 dictionary-only Extension 永远不能携带可执行代码。
- Target-process Adapter 必须签名、hash pin、ABI 兼容并由用户授权。
- Isolated-worker Adapter 使用受控进程、Job Object、超时和最小数据权限。
- Adapter 的 metadata 使用命名空间和有界 Schema；Core 不接受任意指针、地址或内部对象。
- 首版不开放任意 Frida/Hook Code；未来若开放，应作为单独 Expert Script Host，并明确
  显示代码来源、目标范围、版本匹配和风险。
- Adapter 失败只降级自己声明的 Feature；不能把 Observe 成功当成 Writeback 成功。

## 8. 对 V2 路线的直接调整

1. 把现有架构中的 `Capability Provider` 正式改名为 `Capability Adapter`。
2. 增加 Adapter Descriptor、Registry、Apply Model、Placement 与生命周期契约。
3. GDI 与 GDI+ 是首批第一方 Adapter，不是 Runtime Kernel 内置分支。
4. Software Extension 从 `runtime.providers` 改为声明 `capabilities` 依赖。
5. 删除宽泛的 Runtime Extension；路由收紧为无代码 Route Program，新增 Hook 必须注册
   成 Adapter。
6. Foundation Contract 增加“未知第三个 Adapter 不改 Core”的 contract test。
7. 首版实现仍只承诺已经实证的 GDI/GDI+ 可见写回；DirectWrite、Chromium DOM、Qt、
   Unity、Skia 与 UIA Writeback 均先进入候选表，逐项用合成 Host 和实际像素/可访问文字
   证明后再发布能力。

## 9. 最终判断

应该写成 Adapter，而且是**能力适配器注册表**，不只是一个名字叫 Adapter 的
`match driver` 包装层。

核心边界应为：

```text
Software Extension declares needs
  → Session Manager resolves registered Capability Adapters
  → Adapter reports Feature + Apply Model + Placement
  → Core routes Observation and produces generic Decision
  → Adapter applies it using its own mechanism
  → Feature-specific ACK determines the visible product state
```

这样增加 DirectWrite、Chromium DOM、Qt、Unity 或私有引擎时，新增的是 Adapter 包与对应
测试，不是新的 Core/GUI 软件分支。
