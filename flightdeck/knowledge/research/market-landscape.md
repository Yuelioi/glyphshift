# Windows 全局文本 Hook / 翻译工具市场调研

更新时间：2026-07-30

## 结论摘要

市场上已经存在三类相邻能力，但没有一个公开项目同时提供“全局字典目录 + 应用选择 + 多渲染路径适配 + 可版本化 Adapter 插件”的完整组合：

1. **文本 Hook/翻译器**（Textractor、LunaTranslator）擅长进程注入和游戏文本提取，插件/扩展成熟，但目标主要是视觉小说或游戏语境，不是桌面软件 UI 的通用字典替换。
2. **动态插桩框架**（Frida、Microsoft Detours）提供强大的注入/函数拦截基础设施，却不提供字典、UI 语义、应用选择或安全的翻译产品模型。
3. **Windows 可访问性接口**（UI Automation TextPattern）可以无注入地读取部分标准控件文本，但覆盖率取决于应用是否暴露 Provider，不能替代绘制层 Hook。

因此 Glyphshift 的差异化方向仍然成立：把“选择哪个应用”和“如何取文本”分离；把字典做成数据插件；把原生 Adapter 作为受控、版本化的能力插件，而不是把地址和软件逻辑写死在 DLL 中。

## 代表性项目

### Textractor / NextHooker

官方仓库：[Artikash/Textractor](https://github.com/Artikash/Textractor)

官方 README 明确描述了其架构：宿主向目标进程注入 `texthook`，通过管道和共享内存交换 Hook 信息；注入到文本输出函数（例如 `TextOut`、`GetGlyphOutline`），再将文本发送到 GUI；GUI 把文本分派给扩展。项目还支持 `/H` hook code、自动搜索和可编译扩展。

**可借鉴**：进程注入后使用 IPC；把文本后处理和扩展分开；扩展点应位于“文本事件”之后而不是让每个扩展直接操作核心 Hook 状态。

**边界**：它面向游戏文本，Hook code/引擎规则是中心概念；没有通用的 Windows 应用身份到字典快照的目录模型，也没有菜单、面板、效果参数等 UI scope 语义。

### LunaTranslator / LunaHook

官方仓库：[HIllya51/LunaTranslator](https://github.com/HIllya51/LunaTranslator)

项目 README 将 Hook 描述为主要文本提取方式，并支持直接嵌入翻译；其文档也强调兼容大量视觉小说。

**可借鉴**：把 Hook 结果统一成可翻译文本流；针对不同引擎维护配置；将翻译显示与提取解耦。

**边界**：仍然是游戏/视觉小说优先，规则和适配器高度依赖引擎；不能直接推导出 QQ、AE、Office 等桌面软件的控件语义或安全写回策略。

### Frida

官方文档：[Frida JavaScript API](https://frida.re/docs/javascript-api/)

Frida 提供跨平台动态插桩、NativePointer、函数拦截、消息发送和脚本运行时。它展示了“注入引擎”和“用户脚本/策略”分离的成熟做法。

**可借鉴**：版本化的运行时能力、消息通道、脚本与宿主解耦、按进程加载策略。

**边界**：Frida 是通用调试/插桩工具，不提供翻译字典、UI scope、应用级选择，也不替用户解决 ABI、签名、权限和崩溃隔离问题。Glyphshift 不应把任意 Frida 脚本直接等同于安全 Adapter 插件。

### Microsoft Detours

官方研究资料：[Detours: Binary Interception of Win32 Functions](https://www.microsoft.com/en-us/research/project/detours/)

Detours 是 Win32 函数拦截库，核心是把目标函数调用重定向到 detour，同时保留 trampoline 调用原函数。

**可借鉴**：原生 Hook 的最小机制、原函数保留、安装/卸载边界。

**边界**：它只解决函数拦截，不解决文本抽取、翻译、字典切换、Host 识别和插件治理；Glyphshift 的 Adapter 应包住此类底层库，而不是把 Detours API 暴露到 Core。

### Windows UI Automation TextPattern

官方文档：[UI Automation TextPattern Overview](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-ui-automation-textpattern-overview)

Microsoft 的 TextPattern 用于暴露控件的文本内容、范围、格式和样式。它是标准控件可访问性通道，通常无需向目标进程注入。

**可借鉴**：在 Hook 之前优先尝试无注入读取；将“可读文本”和“可写文本”能力分开声明；把 Provider 不支持视为正常能力缺失。

**边界**：只有实现 UI Automation Provider 的控件才能提供可靠文本；自绘、游戏渲染、GPU 合成、画布和部分 Electron/Qt 场景可能没有所需 TextPattern。UIA 也不能自动把翻译写回所有控件。

## 能力对照

| 能力 | Textractor/LunaTranslator | Frida/Detours | UI Automation | Glyphshift 目标 |
|---|---|---|---|---|
| 进程注入/函数拦截 | 强 | 强 | 无需注入 | 由 Windows Adapter 提供 |
| 标准控件无注入读取 | 弱 | 无 | 有条件支持 | UIA Adapter（后续） |
| 全局字典目录 | 未见通用模型 | 无 | 无 | `DictionaryCatalog` |
| 应用选择/切换 | 以游戏为中心 | 由用户脚本决定 | 由调用方决定 | `ApplicationSelector` |
| 菜单/面板/效果 Scope | 少量游戏语境 | 无 | 控件树语义 | Core Scope + Host Profile |
| 可插拔扩展 | 有扩展/脚本 | 有脚本 | Provider 模型 | 数据插件 + 受控 Native Adapter |
| 翻译写回 | 游戏/显示层特定 | 用户实现 | 取决于控件 Pattern | 能力声明后按 Adapter 实现 |

## 对 Glyphshift 架构的决策

### 1. Adapter 不应按“每个软件一个”划分

优先按渲染/读取技术划分：UIA、Win32/GDI、GDI+、DirectWrite/D2D、Qt/Electron/自绘等。软件只需要一个 Host Profile，声明：

- 进程识别（路径、签名、产品名、版本范围）；
- 可用能力和安全级别；
- 默认字典 ID；
- Scope 规则和文本清洗规则；
- 是否允许写回。

只有当软件使用了特殊协议、私有渲染或独特写回机制时，才增加专用 Adapter。

### 2. 插件分两级

- **Dictionary/Host Profile 插件**：JSON/TOML + 字典文件，进程外加载，默认允许热切换；不得包含任意地址或可执行代码。
- **Native Adapter 插件**：稳定版本化 C ABI，使用 opaque event/capability 接口；必须显式授权，建议签名校验和崩溃隔离。插件不能拿到 Core 内部 Rust 类型，也不能直接提交任意函数地址。

### 3. 推荐的通用流水线

```text
Process Discovery
  -> ApplicationSelector
  -> HostProfile + DictionarySnapshot
  -> Capability Planner
  -> UIA / GDI / DWrite / Custom Adapter
  -> Normalized TextObservation
  -> Scope + Dictionary Runtime
  -> Optional Writeback Adapter
```

关键是 `Normalized TextObservation`：所有 Adapter 只产生统一事件，Core 不知道文本来自 UIA、GDI 还是 DWrite。

### 4. QQ 这类目标的现实预期

QQ 的标准菜单、设置窗口和普通控件可能复用 UIA/GDI/DirectWrite Adapter；聊天气泡、自绘控件、GPU 合成内容则必须先 Capture 取证。不能因为“有通用 Adapter”就假设所有 QQ 文本都可读取或安全写回。正确顺序是：先识别进程与能力，再以 Capture-only profile 探测，最后才启用写回。

## 不应重复建设的部分

- 不重新实现通用函数拦截库：复用 `retour`/Detours 类成熟机制，并将其封装在 Adapter 内。
- 不把翻译引擎写入 Hook DLL：Core 只做规范化、字典查找和事件路由。
- 不把每个软件的地址表硬编码进主 DLL：放进版本化 Profile/Adapter 插件。
- 不默认允许写回：先提供 Observe/Capture，再按能力声明开启 Replace。

## 仍然值得建设的空白

1. 跨软件统一的 `ApplicationId -> DictionarySnapshot` 目录和 UI 切换模型。
2. 同一软件同时使用 UIA、GDI、DWrite 时的去重、优先级和 Scope 合并。
3. 不可信 Native Adapter 的签名、权限、版本兼容和崩溃隔离。
4. Capture 结果到 Host Profile 的半自动生成流程。
5. “可观察”与“可写回”分离的安全能力模型。

## 参考来源

- [Textractor 官方仓库](https://github.com/Artikash/Textractor)
- [LunaTranslator 官方仓库](https://github.com/HIllya51/LunaTranslator)
- [Frida JavaScript API 官方文档](https://frida.re/docs/javascript-api/)
- [Microsoft Detours 官方研究页面](https://www.microsoft.com/en-us/research/project/detours/)
- [Microsoft UI Automation TextPattern 官方文档](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-ui-automation-textpattern-overview)
