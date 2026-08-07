# Capability Adapter 注册、拦截机制与跨平台路径调研

Status: Complete
Date: 2026-08-02

## 1. 执行摘要

本次结论不是“把 GDI 改名成 `ExtTextOutW`”这么简单，而是：

1. **Glyphshift 已经有运行时 Registry 和 machine descriptor，缺的是可供产品/UI 使用的 Adapter Catalog。** 现有 Descriptor 已包含 ID、版本、Apply Model、Placement、Feature、架构和 ABI；Registry 已处理包、签名者、hash、授权、ABI、架构和 Feature 解析。当前 `GDI` / `GDI+` 文案来自开发 Runtime Bundle 的手写 `label`，不是 Adapter 自描述。
2. **市场证据支持“能力适配器”，不支持“一个可任意填写 DLL/函数的通用 Hook 表单”。** Textractor/LunaHook 的地址、参数偏移、编码、引擎匹配和嵌入写回都由 Hook Code 或引擎代码表达；XUnity 则在 Unity 文本组件层适配 UGUI、NGUI、TextMeshPro 等；Agent 将 Frida 引擎和逐游戏脚本分开。这些机制的生命周期、风险和可写回能力差异很大。
3. **ID 应精确到稳定技术入口，显示名应兼顾可读性。** 当前两个 Adapter 合理的稳定 ID 是 `windows.gdi.ext-text-out` 与 `windows.gdiplus.draw-string`，用户可见名称宜为“Windows GDI · ExtTextOutW”和“Windows GDI+ · GdipDrawString”。`GDI` / `GDI+` 适合做技术族标签，不足以充当唯一身份。
4. **当前 Windows 两个 Adapter 的 v1 参数 schema 应明确为空。** 它们的 Hook 目标、解码、长度边界、重入保护和字体替换方式都是实现不变量；激活入口仅协商 requested/granted Feature bits。若要支持 `TextOutW`、`DrawTextW`、其他 GDI+ 函数或任意地址，应增加新 Adapter 或受限的 Expert Hook Host，而不是让普通配置把现有 Adapter 变成另一种机制。
5. **推荐“最小 native descriptor + 签名包 manifest + Registry 投影”的混合模型。** 安全/执行事实由 Native Descriptor 声明并与签名 manifest 交叉验证；本地化名称、说明、典型场景和参数表单 schema 放在签名包的 presentation/catalog 资源中；Registry 生成当前平台的有效目录；UI 只做通用渲染，不保存每个 Adapter 的知识。
6. **macOS 不应从进程注入起步。** 首条可发布路线应是 Accessibility 外部 Adapter，先只承诺 Observe；只有属性明确可写时，才考虑范围受限的“可编辑值写入”，不能把它包装成静态标签的通用 Text Replace。Core Text / Objective-C 拦截与 Frida 均是候选研究路线，但任意发布版目标通常没有 `get-task-allow`，还会受到 SIP、Hardened Runtime、Library Validation 和代码签名约束，不能承诺为普通用户的通用方案。

## 2. 能力定义与证据判定

本文严格区分：

- **Observe**：从目标进程、UI 对象、协议或像素中取得文字。
- **Target Text Replace**：目标软件自己的可见 UI/渲染结果使用替换后的文字。
- **Font Substitute**：目标软件自己的绘制/组件使用替换字体。
- **Overlay / OCR**：在目标之外识别或覆盖显示；即使视觉上“在窗口上方”，也不是目标内写回。

观察到文字不等于可写回；Frida 等框架“技术上能改参数/内存”也不等于上层产品已经提供稳定、通用、可声明的 Text Replace 能力。

## 3. 证据矩阵

| 代表 | 实际拦截点/组件 | Observe | Target Text Replace | Font Substitute | Overlay / OCR | Hook/适配组织方式 |
|---|---|---:|---:|---:|---:|---|
| Textractor | 注入 `texthook`；`TextOut`、`GetGlyphOutline` 等函数；引擎特定地址 | 是 | 官方架构未证明通用写回 | 未证明 | 外部 GUI/Extension，不等同目标内写回 | Host + 注入 DLL + Pipe + Engine Code + `/H` Hook Code + Extensions |
| LunaTranslator / LunaHook | GDI/GDI+/D3DX/字符串函数、引擎/模拟器/JIT Hook、内存地址 | 是 | 仅 `EMBED_ABLE` Hook；可新分配、覆盖原缓冲或调用专用 embed function | 有独立嵌入字体设置；具体支持依赖 Hook | 有 OCR 与外部翻译显示 | `ENGINE` 检测/附加类 + `HookParam` + 通用 PC fallback + 专用游戏设置 |
| XUnity.AutoTranslator | Unity 的 UGUI、NGUI、IMGUI、TMP、TextMesh、FairyGUI、Utage 组件/方法 | 是 | 是，修改目标内组件文字 | 是，UGUI/TMP 分开处理 | 有 Translation Aggregator，但不是组件写回 | Loader/插件框架包 + 文本框架 Hook + INI 配置 + Endpoint 插件目录 |
| Agent | Frida 注入后的逐游戏/逐模拟器 JavaScript 地址 Hook | 是 | Agent 的公共产品契约以提取为主；不能据 Frida 能力推导为通用写回 | 未声明 | 外部 GUI 输出 | Agent Host + 自动同步 scripts repo + 用户脚本头元数据 + 公共脚本库 |
| Frida（底层框架） | Native `Interceptor`、内存、Objective-C/Java 方法 | 是 | 框架层可以改函数参数或替换实现 | 可编程实现，非内建翻译能力 | 不负责 OCR/Overlay 产品语义 | frida-core/GumJS + 注入/嵌入/预加载模式 + JS/TS 脚本 |
| macOS Accessibility | `AXUIElement` 属性、`AXObserver` 通知 | 是（依目标暴露质量） | 仅属性可写时；不等于静态文本通用替换 | 否 | 否 | 外部进程 API + 用户明确授予 Accessibility 权限 |
| macOS Core Text（候选） | `CTLineDraw`、`CTFrameDraw`、`CTFontDrawGlyphs` 等 | 候选，入口粒度不同 | 候选，需重建 layout/object；未有通用实证 | 候选 | 否 | 需要目标进程内拦截，受签名/注入限制 |

## 4. 产品机制详析

### 4.1 Textractor：Hook 描述是可执行技术数据，Extension 是后处理

[Textractor 官方架构](https://github.com/Artikash/Textractor#project-architecture)说明 Host 将 `texthook` 注入目标进程，通过两条 Pipe 连接；注入模块在 `TextOut`、`GetGlyphOutline` 等文字函数插入代码，Host 再将收到的文字交给 GUI 和 Extensions。官方功能还同时保留自动引擎 Hook、手工 `/H` Hook Code 和 Hook 搜索。

其 [`HookParam` 源码](https://github.com/Artikash/Textractor/blob/master/include/types.h#L21-L43)不是“GDI”一个枚举，而是包含：

- 地址，或 module + function；
- 数据偏移、二次解引用、split/context、长度位置、padding；
- 编码/codepage 和类型 flags；
- 可选 text/filter/hook/length 回调。

这说明长尾 Hook 的真实参数空间很大，而且不少字段等价于进程内任意地址/函数操作。Textractor 能让专家使用 Hook Code，不代表 Glyphshift 应把相同能力放进普通词典编辑器。它更像未来独立的 **Expert Hook Host**：高风险、目标版本敏感、单独授权、与第一方固定 Adapter 分离。

Textractor 的公开架构主要证明 Observe 管线。Extensions 在 Host/GUI 收到文本后工作，不能据此认定目标软件已显示译文。

### 4.2 LunaTranslator / LunaHook：引擎检测、通用 fallback 与 Embed 是三个层次

LunaTranslator 的 [HOOK 设置](https://docs.lunatranslator.org/en/hooksettings.html)将 Win32 通用 Hook、特殊 Hook Code、延迟注入和逐游戏专用设置分开；通用 Hook 包括 GDI、D3DX 和字符串函数，并明确提示过多 Hook 会拖慢游戏，因此默认不全部启用。[基础用法](https://docs.lunatranslator.org/en/basicuse.html)又把 Display 与 Embed 分开：只有支持嵌入的 Hook 才出现 Embed 操作。OCR 是另一种输入源，不是 Hook 的降级写回。

当前 LunaHook 源码展示了清晰但偏编译期的 Adapter 组织：

- [`ENGINE`](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/LunaHook/engine.h)以文件、文件集合、资源字符串或自定义逻辑判断引擎，再执行 `attach_function()`。
- [`enginecontrol.cpp`](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/LunaHook/enginecontrol.cpp)按顺序检查引擎；确定性引擎命中后停止，未命中才安装通用 PC hooks。
- [`pchooks.cpp`](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/LunaHook/engines/pchooks/pchooks.cpp)并不是只 Hook “GDI”：它列举 `TextOutA/W`、`DrawTextA/W`、宽度/测量函数、GDI+ 多个 Draw/Measure API、D3DX 和字符串转换函数。值得注意的是，该项目当前甚至注释掉了通用 `ExtTextOutW`，备注其捕获结果在该场景中不可用；这直接证明“同属 GDI”不代表同一函数在所有产品里都同样有效。

LunaHook 的 [`TextHook::Send`](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/LunaHook/texthook.cc#L362-L413)只有在 Hook 标记 `EMBED_ABLE` 且长度检查通过时才等待译文并写回；写回方式包括分配新缓冲、覆盖原缓冲、调用引擎专用 `embed_fun`，以及 Unity 字符串写入。其 Hook 安装又可使用 MinHook 或 VEH breakpoint。由此可得：

- Observe 与 Embed 是逐 Hook 能力，不是进程级布尔值。
- “地址 Hook”“Win32 API Hook”“引擎专用 embed”应该是不同风险/执行模型。
- 配置发现可由引擎规则完成，但最终激活能力必须由实际 Hook 回报，而不是 UI 猜测。

### 4.3 XUnity.AutoTranslator：正确的 seam 往往是组件，不是绘制 API

[XUnity.AutoTranslator 官方 README](https://github.com/bbepis/XUnity.AutoTranslator#text-frameworks)分别支持 UGUI、NGUI、IMGUI、TextMeshPro、TextMesh、FairyGUI 与 Utage；安装又适配 BepInEx、MelonLoader、IPA、UnityInjector 和 ReiPatcher 等环境。其配置按 `[TextFrameworks]` 分别启用组件族，而不是一个 `Unity=true`。

它的目标内能力也被拆开：

- 组件文字更新；
- UGUI `OverrideFont`、TextMeshPro `OverrideFontTextMeshPro` / fallback font；
- overflow、line spacing 等 UI resizing；
- TextAsset/纹理等资源重定向；
- `Translators` 子目录中的翻译 Endpoint 插件。

[MonoMod Hooks 说明](https://github.com/bbepis/XUnity.AutoTranslator#monomod-hooks)还显示 Hook 实现会根据方法是否有 body、目标 API surface 和依赖可用性在 Harmony 与 MonoMod 之间选择；[IL2CPP 支持](https://github.com/bbepis/XUnity.AutoTranslator#il2cpp-support)存在独立能力缺口。

对 Glyphshift 的含义是：Adapter ID 应表达可验证的 seam，例如 `unity.tmp.text-component`，而不是把 `unity` 当作 Hook 类型。字体、布局和资源是独立 Feature；它们可以由同一包实现，但不能因为文字替换成功就自动宣告字体替换成功。

### 4.4 Agent / Frida：脚本生态证明可扩展性，也放大供应链边界

[Agent 官方仓库](https://github.com/0xDC00/agent)将自身定义为 Frida 驱动的脚本文本 Hooker，程序目录中的 scripts 会从独立 [官方 scripts 仓库](https://github.com/0xDC00/scripts)同步；公共库覆盖 Yuzu、Vita3K、PPSSPP、RPCS3、Unity、MAGES、HCode 等。逐目标脚本通过类似 UserScript 的 `@name`、`@version`、`@author`、`@description` 提供轻量展示信息，并在脚本里选择游戏版本、地址和 handler；[一个实际脚本](https://github.com/0xDC00/scripts/blob/main/NS_01001DC01486A000_Tsukihime.js#L1-L45)就同时包含版本分支、地址映射、二进制文本解析和发送逻辑。

底层 [Frida 的运行模式](https://frida.re/docs/modes/)将 frida-core 注入、Gadget 嵌入和预加载分开；[JavaScript API](https://frida.re/docs/javascript-api/)提供 native Interceptor、Objective-C/Java bridge、内存读写、Host/Agent 消息与 RPC。[官方函数示例](https://frida.re/docs/functions/#modifying-function-arguments)明确展示了修改被拦截函数参数，因此 Frida 框架可以承担写回，但 Agent 产品及其大多数脚本仍以提取/发送为主，不能自动声明 `text.replace`。

Agent 的分层很值得借鉴：稳定执行引擎与快速变化的逐目标脚本分开。其弱点也同样明确：脚本拥有目标进程内代码执行能力，轻量注释元数据不等于签名、Feature contract、参数 schema 或稳定兼容声明。Glyphshift 首版不应允许字典或 Software Extension 携带任意 Frida/地址脚本；若以后开放，应是独立 Expert Script Adapter Host。

## 5. 当前 Glyphshift 实现审计

### 5.1 已经具备的基础

[`glyphshift-adapter-sdk`](../../../../crates/adapters/platform/sdk/src/lib.rs) 的 `AdapterDescriptor` 已包含：

- `adapter_id`、semantic version、ABI；
- `ApplyModel`、`Placement`；
- Feature 集合；
- architecture 集合。

[`glyphshift-adapter-registry`](../../../../crates/adapters/platform/registry/src/lib.rs) 已经是实质 Registry，而不是概念占位：

- `reload()` 构建 ID + version 的包索引并拒绝重复/冲突内容；
- `resolve()` 验证 artifact hash、trusted signer、adapter authorization、host model、ABI major、架构和请求 Feature；
- 根据 Placement 产出 target-process library 或 isolated-worker executable binding。

这部分应保留并扩展，没必要另造第二个“拦截方式注册中心”。需要新增的是 Registry 的 **Catalog / presentation projection** 与 OS/config 匹配。

### 5.2 UI label 的硬编码来源

该历史观察对应旧构建入口；当前 [`build-runtime-bundle.ps1`](../../../../scripts/build-runtime-bundle.ps1)
生成 `glyphshift.runtime-bundle/2`，展示信息来自独立 presentation 数据，Loader 固定第一方 authority
并在 Native artifact 加载前验证路径与 SHA-256。

因此当前 UI 名称：

- 不来自 Native Adapter；
- 没有说明、平台、技术目标、能力、风险、配置 schema；
- 由开发 bundle 构建脚本决定，生产包和其他构建者可能给出不同文案；
- 前端仍使用“Hook 类型”“全部 Hook”等产品概念。

### 5.3 Hook 目标与激活参数均写死

[`glyphshift-adapter-gdi-native`](../../../../crates/adapters/implementations/native/gdi-native/src/lib.rs)固定：

- `GetModuleHandleW("gdi32.dll")` + `GetProcAddress("ExtTextOutW")`；
- 通过 `retour::GenericDetour` 安装一次；
- UTF-16 上限、glyph-index 反查、trim、重入保护和字体创建/恢复均在实现内；
- 文字替换时清除 `ETO_GLYPH_INDEX` 并丢弃原 `lpDx`，让 GDI 重新排版。

[`glyphshift-adapter-gdiplus-native`](../../../../crates/adapters/implementations/native/gdiplus-native/src/lib.rs)固定：

- `LoadLibraryW("gdiplus.dll")` + `GetProcAddress("GdipDrawString")`；
- 同时解析一组 GDI+ Font/FontFamily helper；
- 对输入做 trim 和特定布局 padding 归一化；
- 字体替换沿用原 size/style/unit 创建临时 Font。

两者 `activate(host, requested, granted)` 只接收 Runtime Host 回调和 Feature bitsets。`deactivate()` 只把 Active Feature 清零；已安装 Detour 不更换目标，也没有 DLL、函数名、调用栈、线程、模块、窗口、过滤器或任意 JSON 参数。由于 `HOOK` / helper 使用 `OnceLock`，当前生命周期从设计上就是“固定实现，安装一次，Feature 开关控制透传”。

[`glyphshift-adapter-native-abi`](../../../../crates/adapters/platform/native-abi/src/lib.rs)也只有 x86/x86_64 architecture bits，没有 OS、arm64、最低 OS 版本、参数 schema 或 presentation 字段。

### 5.4 OS 声明的真实缺口

[`TargetFacts`](../../../../crates/core/domain/src/lib.rs)已经存储 `operating_system` 与 `architecture`，但只公开 `architecture()`；Registry `resolve()` 也只检查 architecture。因此当前描述符可能在错误 OS 上通过逻辑匹配，只会在更后面的文件/加载阶段失败。

跨平台前至少需要：

- Descriptor/manifest 声明 `platforms` 或结构化 target triples；
- `TargetFacts` 提供并规范化 OS、architecture，必要时提供 OS version/runtime facts；
- Registry 在产出 binding 前完成 OS + arch + ABI + placement 的统一拒绝；
- Native ABI 增加 `arm64`，是否需要 `arm64e` 应由真正的 macOS prototype 决定，不能先假设二者等价。

## 6. Windows 当前 ExtTextOutW / GdipDrawString 的参数化边界

Microsoft 的 [`ExtTextOutW` 文档](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-exttextoutw)表明，同一次调用本身就带有 HDC、位置、options、裁剪/背景矩形、字符串、长度和字符间距；`ETO_GLYPH_INDEX` 表示输入已经是 glyph indices，`ETO_PDY` 改变 spacing 数组语义。Glyphshift 当前 Adapter 已经逐调用读取这些参数，它们不是用户配置。

Microsoft 的 [`Graphics::DrawString`](https://learn.microsoft.com/en-us/windows/win32/api/gdiplusgraphics/nf-gdiplusgraphics-graphics-drawstring%28constwchar_int_constfont_constrectf__conststringformat_constbrush%29)说明字符串、长度、Font、layout rectangle、StringFormat 和 Brush 共同定义绘制。Glyphshift detour 接收并保留这些对象，只替换文字指针/长度和可选 Font；同样不需要把它们暴露为 Adapter 配置。

### 6.1 v1 应明确无参数

建议当前两个 Adapter 对外声明等价于：

```json
{
  "type": "object",
  "properties": {},
  "additionalProperties": false
}
```

“空 schema”比省略含义更清晰：它告诉 Registry/UI 该 Adapter 可激活但不可调参，也为未来真正需要参数的 Adapter 保留统一接口。

### 6.2 不应参数化的内容

以下内容一旦改变，就已经不是当前 Adapter，应使用新 ID/新包或 Expert Host：

- DLL、导出函数、绝对地址、pattern/signature；
- 任意参数偏移、指针解引用、调用约定；
- 任意回调/脚本；
- 把 `ExtTextOutW` 换成 `TextOutW` / `DrawTextW`；
- 把 `GdipDrawString` 扩成多个 Measure/DriverString Hook 的组合；
- 关闭实现内部的安全上限、重入保护、fail-open 或原函数调用约束。

原因不是“参数太多”，而是这些字段改变了代码执行位置、数据解释和故障半径，必须经过 Adapter 版本、签名、测试和能力证明。

### 6.3 未来可安全参数化的候选

只有在真实软件证明需要后，才考虑小而有界的 schema，例如：

- 明确枚举的 normalization profile；
- 目标模块 allowlist（仅模块身份，不接受裸地址）；
- 有上下限的观察长度或去重窗口；
- fail-open 的诊断采样级别。

其中 source trim、标点/布局归一化更可能属于通用匹配策略或 route policy，而不是 Hook target 参数。内部最大长度属于安全预算，不应由普通用户放大。词典的 `adapterIds` 是路由限制，也不是 Hook 参数。

## 7. macOS 可行性与限制

### 7.1 路线 A：Accessibility 外部 Adapter（首选起点）

Apple 的 [`AXUIElementCopyAttributeValue` / `AXUIElementSetAttributeValue`](https://developer.apple.com/documentation/applicationservices/1460434-axuielementsetattributevalue)允许读取属性，并在目标属性支持且可写时设值；API 同时提供 `AXUIElementIsAttributeSettable`。[`AXObserver`](https://developer.apple.com/documentation/applicationservices/1460133-axobservercreate)和通知注册可监听特定应用的 UI 变化。

但该路线有两个必须公开的边界：

1. 目标软件决定暴露哪些 Accessibility 元素、属性和通知；自绘/画布/游戏内容可能没有有用文字。
2. `SetAttributeValue` 只对可写属性成立。可编辑输入框的 `value` 写入会改变应用业务状态，不能等同于“把一个静态标签的显示文字替换掉”。

第三方 Accessibility 客户端需要用户明确授权；Apple 的 [`AXIsProcessTrustedWithOptions`](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions)用于判断当前进程是否受信任，系统也明确要求用户在 Privacy & Security 中授予 [Accessibility 控制权限](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)。

因此建议首个 macOS Adapter 是 isolated-worker、Observe-only：

```text
macos.accessibility.ax-ui-element
  placement: isolated-worker
  apply_model: observe-only
  features: [text.observe]
```

若未来验证可写场景，应另定义范围明确的 `editable-value.write` 类能力/Adapter，并在 UI 上说明它会修改控件值；不要把它静默提升为通用 `text.replace`。

### 7.2 路线 B：官方插件、资源或应用级协议（稳定但逐软件）

当目标软件正式提供插件、脚本、localization resource 或可控协议时，应优先使用该 seam。它通常能取得更高层的字符串/对象身份，也更容易正确处理布局和字体；代价是 Adapter 会更应用/框架特定。

macOS Hardened Runtime 默认启用 Library Validation。Apple 文档说明，目标进程默认只加载 Apple 或与主程序同 Team ID 签名的代码；只有目标程序自身具有 [`Disable Library Validation` entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.disable-library-validation)时，才可加载其他开发者签名的插件。**Glyphshift 自己拥有该 entitlement 并不能让第三方目标进程加载 Glyphshift 的 dylib**。因此插件路线必须尊重目标软件公开的加载/签名机制，不能由 Registry 宣告后强行加载。

### 7.3 路线 C：Core Text / AppKit / Objective-C 进程内拦截（候选，不承诺）

Core Text 是合理的技术候选，但没有单一等价于 `ExtTextOutW` 的万能入口：

- [`CTLineDraw`](https://developer.apple.com/documentation/coretext/ctlinedraw%28_%3A_%3A%29)绘制完整 line，line 内部包含 glyph runs；
- [`CTFrameDraw`](https://developer.apple.com/documentation/coretext/ctframedraw%28_%3A_%3A%29)绘制整个 frame；
- [`CTFontDrawGlyphs`](https://developer.apple.com/documentation/coretext/ctfontdrawglyphs%28_%3A_%3A_%3A_%3A_%3A%29)只拿到 font、glyphs 和 positions；
- [`CTLineGetStringRange`](https://developer.apple.com/documentation/coretext/ctlinegetstringrange%28_%3A%29)只返回“产生这些 glyph 的 backing store 范围”，并不直接返回原始字符串。

所以 Hook `CTLineDraw` 仍可能需要在更早的 `CTLineCreateWithAttributedString` 等创建点维护对象到原文的关联；到了 glyph 层则会失去稳定文本语义。AppKit `NSText*`/Objective-C setter 又是另一套 retained-object seam。二者应是不同 Adapter，不应统一叫 `macos.coretext` 后假装能力相同。

### 7.4 Frida 在 macOS 上“能运行”不等于可产品化通用注入

[Frida macOS 官方示例](https://frida.re/docs/examples/macos/)说明 attach 需要通过 `task_for_pid` 授权，并明确指出某些目标可能还需要关闭 SIP。Frida 确实支持 macOS、Objective-C 和 arm64，并持续改进 hardened Darwin 目标；但 Apple 的 [`Debugging Tool` entitlement 文档](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.debugger)明确限制：即使调试器拥有 entitlement，也不能取得那些没有 `get-task-allow` 且受 SIP 保护的进程 task port。发布版应用通常会移除 `get-task-allow`；Apple 还会因发布包携带它而拒绝 notarization，[官方 notarization 排错文档](https://developer.apple.com/documentation/security/resolving-common-notarization-issues)将其明确视为安全风险。

因此：

- Frida 可以作为研究、开发和经明确授权的 Expert Adapter Host；
- 不应成为 macOS 普通用户版本的默认 Controller/Adapter 基础；
- 不应要求用户关闭 SIP 作为正式产品安装步骤；
- 对某个第三方发布版应用能否注入，必须逐目标实证，不能只依据“Frida 支持 macOS”。

### 7.5 发布与签名底线

Apple 只 notarize 启用 Hardened Runtime 的 macOS 应用；运行时例外会移除相应安全保护，Apple 建议使用最窄的 entitlement 集合。[Hardened Runtime 官方说明](https://developer.apple.com/documentation/xcode/configuring-the-hardened-runtime)把 Debugging Tool、Disable Library Validation、Disable Executable Memory Protection 等都列为明确例外。Glyphshift 的 Adapter 包因此必须：

- 自身签名、hash pin、架构/OS/ABI 匹配；
- 区分“加载进 Glyphshift/worker”与“加载进第三方 target”，两者受不同进程的签名策略控制；
- 不因为 Adapter 包受 Glyphshift 信任，就假定 macOS 会允许它进入目标进程；
- 将 Accessibility 权限、目标插件授权和调试/注入授权分别建模，不能合并成一个 `authorized=true`。

## 8. 接口与元数据方案比较

### 8.1 方案一：所有信息都由 Native Adapter ABI 自描述

Native entrypoint 返回 machine descriptor、名称、说明、locale 文案和 JSON Schema。

优点：

- 单文件自包含；
- 不容易出现 manifest 找到但 binary 缺失的情况；
- Adapter 可以直接报告实现事实。

缺点：

- 为本地化和文案变更升级 ABI/二进制，耦合过重；
- 在验证展示信息前就要加载 native code，扩大发现阶段攻击面；
- C ABI 承载可变长度 locale/catalog/schema 复杂；
- 包签名、artifact mapping、说明资源仍无法真正消失。

结论：适合最小 machine descriptor，不适合完整 presentation catalog。

### 8.2 方案二：所有信息都由 Registry 中央表/UI 硬编码

Core/Registry 根据 Adapter ID 内置名称、平台、能力说明和表单。

优点：实现最快，UI 完全可控。

缺点：新增 Adapter 必须改 Core/UI；第三方包无法自描述；版本与文案易漂移；正是当前 `GDI` / `GDI+` label 问题的放大版。

结论：不符合动态注册目标，不建议。

### 8.3 方案三：最小 Native Descriptor + 签名 Package Manifest + Registry Catalog（推荐）

包由三个逻辑部分构成：

```text
signed adapter package
  ├─ manifest: artifact/trust/compat/config/presentation index
  ├─ native or worker artifact: runtime descriptor + implementation
  └─ locales: localized name/summary/help/examples
```

字段归属建议：

| 字段 | 权威来源 | Registry 的责任 | UI 的责任 |
|---|---|---|---|
| ID、version、ABI、Apply Model、Placement、Features | Native Descriptor；签名 manifest 重复声明并必须一致 | 交叉验证、解析、拒绝不兼容 | 只读展示 |
| OS/platform、architecture、最低 OS/runtime | Descriptor + manifest 的兼容约束 | 与 Target Facts 求交集 | 显示兼容/不可用原因 |
| artifact、hash、signer、entrypoint、依赖 | 签名 manifest | 信任验证、artifact binding | 不接触路径/签名细节，必要时显示发布者 |
| stable technical target，例如 `gdi32.dll!ExtTextOutW` | 签名 manifest 的 implementation metadata；运行时可回报实际 probe evidence | 保留诊断与版本事实 | 展示“技术入口”，不作为自由输入框 |
| display name、summary、typical uses、risk/help | 签名 package 的 locale resources | locale fallback、生成 Catalog View | 通用卡片/帮助面板渲染 |
| parameter schema、默认值、示例 | 签名 manifest；Adapter 实现做最终语义校验 | schema/version/大小验证；保存 binding config | 通用 schema form；无 schema 显示“无需配置” |
| 当前可用/Active Feature、错误、ACK | 运行时事实 | 汇总为 effective status | 只显示真实状态，不从声明推断 Active |

本方案的关键不是“同一字段只出现一次”，而是区分：

- **machine descriptor**：短小、稳定、影响安全和执行；
- **localized presentation metadata**：可本地化、可扩充、不参与能力授权；
- **runtime evidence/status**：本次目标上的事实，不能由静态文案替代。

## 9. 推荐的 Glyphshift 模型

### 9.1 Adapter 身份与展示

当前两个 Adapter 建议形成如下 Catalog：

```text
id: windows.gdi.ext-text-out
display_name.zh-CN: Windows GDI · ExtTextOutW
family: Windows GDI
technical_target: gdi32.dll!ExtTextOutW
summary: 拦截通过 ExtTextOutW 绘制的文字，并可在绘制调用内替换文字或字体。
platforms: [windows]
architectures: [x86, x86_64]
features: [text.observe, text.replace, font.substitute]
configuration: none
```

```text
id: windows.gdiplus.draw-string
display_name.zh-CN: Windows GDI+ · GdipDrawString
family: Windows GDI+
technical_target: gdiplus.dll!GdipDrawString
summary: 拦截 GDI+ DrawString 绘制，并保留原布局/格式对象后替换文字或字体。
platforms: [windows]
architectures: [x86, x86_64]
features: [text.observe, text.replace, font.substitute]
configuration: none
```

字段名不再叫“Hook 类型”，宜改成“适用适配器”或“文字适配器”。UI 可按平台/技术族分组，但选择值始终是稳定 Adapter ID。

### 9.2 词典与 Adapter 解耦

“菜单 / 对话框 / 效果”是软件语义 Location，不是 GDI/GDI+ 技术分类。长期模型宜为：

```text
Dictionary rules
  → semantic locations/context（菜单、对话框、效果等）
  → software route chooses compatible Adapter(s) per platform/version
  → Adapter applies generic Decision by its mechanism
```

词典可保留 `adapter constraints` 作为高级范围收窄，但不应以 Windows Adapter ID 作为主要分类，否则同一份词典无法自然复用到 macOS Accessibility、官方插件或 Core Text Adapter。

### 9.3 Registry 应增加的最小接口

不需要在首版做完整插件市场。最小新增面是：

```text
catalog(locale, target_facts?) -> AdapterCatalogEntry[]
resolve(requirement, target_facts, config) -> AdapterBinding
validate_config(adapter_id, schema_version, value) -> ValidatedConfig
```

`AdapterCatalogEntry` 是静态声明与当前兼容结果的投影；`AdapterBinding` 才是通过信任/兼容/Feature/config 检查的执行对象；Active 状态仍来自 activate/update ACK。

### 9.4 分阶段建议

1. **P0：补齐 Catalog 边界。** 增加 OS/platform 声明和匹配；把手写 label 移入签名包 presentation；当前两个 Adapter 声明空 config schema；UI 展示精确名称、说明、平台、Feature 和“无需配置”。
2. **P0：修正产品语言。** 从“Hook 类型”改为“适用适配器”；`GDI/GDI+` 做 family，`ExtTextOutW/GdipDrawString` 做技术目标；Observe/Replace/Font 分别显示声明与实际状态。
3. **P1：把词典语义 Location 与 Adapter routing 分开。** 迁移数据时可保留现有 adapter constraint，但不要继续扩散 `hookTypeId` 作为跨平台核心模型。
4. **P1：建立未知第三个 Adapter contract test。** 新包带新 ID、presentation 和空/简单 schema 时，不改 Core/UI 即可发现、解释、兼容判定和拒绝。
5. **P2：macOS 先做 Accessibility Observe 可行性原型。** 测量实际目标的元素覆盖、变化通知、文字质量和权限体验；没有证据前不承诺 Replace。
6. **P2：逐目标选择写回 seam。** 有官方插件/协议则优先；Core Text/ObjC/Frida 仅作为实验/专家路线，以真实签名目标验证后再注册 Feature。

## 10. 明确不建议事项

- 不建议只把 UI 的 `GDI` 改成 `ExtTextOutW`，但继续由脚本硬编码 `{id,label}`；这只改善文案，不形成可扩展目录。
- 不建议把 Adapter ID 退化成 `windows.gdi` 或 `macos.coretext` 这种技术族名；同一族内入口和能力差异已被 LunaHook/XUnity 证实。
- 不建议让用户在普通 UI 输入 DLL、函数名、绝对地址、pattern、参数偏移或 Frida JavaScript。
- 不建议把 localized name/description 塞进 Native C ABI，也不建议让 UI 内置每个 Adapter 的说明。
- 不建议让 manifest 单方面声明安全关键 Feature，而不与 binary descriptor/实际 activate 结果核对。
- 不建议把 OCR、Overlay、Accessibility Observe 或 Pipe 有数据显示成 Text Replace Active。
- 不建议把 Accessibility `SetValue` 宣称为 macOS 静态界面通用翻译；它只对目标声明可写的属性有效，并可能修改业务数据。
- 不建议把关闭 SIP、重签第三方应用或保留 `get-task-allow` 当作正式 macOS 安装流程。
- 不建议现在设计一个能表达 Textractor 全部 HookParam 的通用参数 schema；首版两个 Adapter 明确无参数更可靠。

## 11. 开放问题

1. **Adapter routing 的归属**：软件扩展是直接要求具体 Adapter ID，还是先要求 capability + semantic location，再由 Registry/route 选择？推荐后者为默认，前者仅作高级约束。
2. **多 Adapter 重复命中**：同一文字可能同时经过测量、绘制或多层框架。谁负责去重、优先级和“一次可见写回”？需要 route/arbitration contract，不能由词典顺序偶然决定。
3. **参数变更生命周期**：binding config 变化是 generation update 即可，还是必须 deactivate/reactivate？每个 schema 字段需要声明 apply timing，首版空 schema 可暂不实现复杂性。
4. **presentation 更新是否绑定代码版本**：建议随签名包版本发布并以 locale fallback 读取；若未来允许独立文案更新，需要额外签名和兼容策略。
5. **第三方 Adapter 信任**：是否只允许第一方 signer，何时开放组织/用户 signer，撤销与审计如何做？这应先于任意脚本 Host。
6. **macOS 首批目标**：必须先选 1–2 个明确软件/版本测量 Accessibility 覆盖和官方插件可能性；没有目标事实，无法决定 Core Text 与 AppKit seam。
7. **Feature 粒度**：`text.replace` 是否足以表达“仅可编辑 value 写入”？大概率需要 scope/constraints，避免 UI 把局部能力显示为全局能力。
8. **字体替换状态**：Adapter 支持 `font.substitute`、Session 请求、规则实际命中和屏幕已使用替换字体是四个不同事实，Catalog 与运行状态需要分别表达。

## 12. 最终判断

Glyphshift 不需要推倒现有 Adapter Registry；现有实现已经完成了“执行与信任注册”的主要骨架。下一步应补上：

```text
signed package metadata + localized presentation
  → Registry validates machine descriptor and target facts
  → Catalog explains what is available and why
  → Binding carries only typed, validated configuration
  → Runtime activation/ACK proves actual Feature state
```

对当前 Windows 实现，最诚实的产品表达是“Windows GDI · ExtTextOutW”和“Windows GDI+ · GdipDrawString”，并明确“无需配置”。对未来 macOS，先以 Accessibility Observe 和官方插件/协议建立可发布覆盖，再把 Core Text、Objective-C 或 Frida 视为逐目标、受签名约束的候选 Adapter，而不是预设一个跨平台万能 Hook。
