# 下一种实时写回 Adapter：第一方资料评审

Status: Complete

## 结论

明确排序如下：

1. **Qt 文本翻译入口**：进入下一轮有界原型，优先验证自定义 `QTranslator`，其次才考虑拦截
   `QCoreApplication::translate`。
2. **WinUI / MRT Core 资源入口**：保留为第二候选，但必须先证明典型 WinUI XAML 的文字解析会命中可拦截的
   原生入口；WPF 不与它共用实现。
3. **Chromium / CEF / Electron 页面入口**：不作为默认注入式 Adapter；后续如建设，应定位为需要显式授权、
   扩展安装或调试连接的独立 Extension。

这里的排序只评价 GlyphShift 当前目标：从现有软件中取得字典原文，在同一条业务调用链安全返回译文，停用后恢复；
不按框架流行度、可观察文字数量或能否执行任意脚本排序。

## 判据矩阵

| 候选 | 1. 原文可取得 | 2. 同链安全写回 | 3. 无需目标改造/开放调试 | 4. 跨软件复用 | 5. 主要风险 | 6. 确定性宿主 | 7. 明确场景 | 当前判断 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Qt `translate` / `QTranslator` | 强：文本式翻译直接带 `sourceText`、context、disambiguation | 强：调用返回 `QString`；自定义 Translator 可直接返回译文 | 条件满足：动态 Qt、受支持 major/toolchain 可进程内安装；不要求调试端口 | 高：Qt Widgets 与 QML 的文本式翻译共用体系 | C++ ABI、Qt 5/6、静态链接、当前界面重译依赖目标处理 `LanguageChange` | 强：可分别构造 Widgets/QML/硬编码/ID 翻译宿主 | 跨平台桌面工具、创作工具、设备控制台、启动器 | **第 1** |
| WinUI / MRT Core | 中：入口首先给资源 ID/URI；调用原实现后可得到当前资源字符串 | 中到强：原生 C API 返回需由 `MrmFreeResource` 释放的缓冲区，理论上可用同族分配器返回译文 | 条件满足：若真实调用链命中 MRT Core 原生导出，则无需调试；尚不能从文档推出所有 XAML 必经该入口 | 中：仅覆盖使用 MRT/MRT Core 的资源化软件 | 公开高层 API、XAML 内部解析与 C 导出的真实调用关系尚未证明；资源 ID 不等于源文 | 强：可构造 ResourceLoader、MRT C API 与 x:Uid 三类宿主 | WinUI 3 管理工具、商店/桌面混合软件 | **第 2，先诊断** |
| WPF 资源体系 | 弱到中：`ResourceManager.GetString` 输入是资源名，BAML/x:Uid 还有独立离线资源路径 | 条件：托管方法改写可改返回值，但不是单一原生 Hook | 弱：需要 CLR Profiler/ReJIT 或更侵入的托管注入 | 中：只覆盖遵守相同资源模式的 .NET 桌面软件 | .NET Framework/.NET 运行时差异、托管代码改写、绑定/ResourceDictionary/BAML 多路径、停用恢复 | 强：可构造多个 .NET 宿主 | 企业桌面表单、配置工具、内部业务客户端 | 不与 WinUI 合并实现，当前不选 |
| Chromium / CEF / Electron | DOM 路径可读当前可见文本，但不等于稳定的“源文”；画布、Shadow DOM、框架状态另算 | 弱：改 DOM 不是原本地化调用的返回值，随重渲染可能被覆盖 | 不满足默认条件：扩展要安装并授权，外部 CDP 要开放调试；CEF/Electron 官方入口由宿主代码持有 | 中：DOM 逻辑可复用，应用语义不可复用 | 浏览器/渲染器多进程、frame 生命周期、隔离世界、权限、应用状态与停用恢复竞争 | 强：可构造网页、Electron、CEF 宿主 | Web 技术桌面壳、聊天/协作客户端、启动器 | **第 3，只做独立 Extension** |

## 1. Qt：推荐进入有界原型

### 为什么它符合产品主线

`QCoreApplication::translate(context, sourceText, disambiguation, n)` 的公开合同直接接收源文本并返回翻译结果；
找不到翻译时返回与源文本等价的 `QString`。`QObject::tr()` 就建立在这个入口上，Qt 的翻译指南也把 C++ 的
`tr()`、`QCoreApplication::translate()` 和 QML 的 `qsTr()` / `qsTranslate()` 放在同一套文本式翻译流程中。
这意味着 Dictionary 可以继续以 `source -> target` 工作，不需要把控件位置或绘制技术塞回词条。
([QCoreApplication](https://doc.qt.io/qt-6/qcoreapplication.html),
[Qt 翻译源代码指南](https://doc.qt.io/qt-6/i18n-source-translation.html),
[QML Qt 翻译函数](https://doc.qt.io/qt-6/qml-qtqml-qt.html))

官方源代码进一步确认：`QCoreApplication::translate` 按优先级遍历已安装的 `QTranslator`，把同一个
`context/sourceText/disambiguation/n` 传给其虚函数，首个非空结果成为返回值；没有结果才从 UTF-8 源文本构造
`QString`。这是“同一调用链写回”，不是在文字已经渲染后猜测布局。
([Qt `qcoreapplication.cpp`](https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp),
[QTranslator](https://doc.qt.io/qt-6/qtranslator.html))

### 首选接入形态

首选是进程内安装一个只读查询 GlyphShift 快照的自定义 `QTranslator`：

- `translate()` 收到真实 `sourceText` 后查询当前 Dictionary；命中就返回译文，未命中返回 null `QString`，让原有
  Translator 或源文本继续处理。
- Qt 按“后安装先查询”处理多个 Translator，因此不必覆盖目标已有语言包；停用时调用
  `removeTranslator()` 即可撤出。安装与移除会产生 `LanguageChange` 事件。
- `QTranslator::translate` 与 `QCoreApplication::translate` 的文档都标为线程安全，适合渲染/工作线程并发查询；
  安装、移除和 UI 刷新仍应调度到应用线程，不能从该线程安全声明外推。

([QTranslator 查询与优先级](https://doc.qt.io/qt-6/qtranslator.html),
[安装、移除与 LanguageChange](https://doc.qt.io/qt-6/qcoreapplication.html))

只有自定义 Translator 无法在受支持 ABI 上可靠安装时，才原型化 `QCoreApplication::translate` trampoline。
直接 Hook 虽仍拥有完整原文和返回值，但更容易与目标已有 Translator、递归调用和 C++ 返回值 ABI 发生冲突。

### 必须承认的覆盖边界

- 只覆盖目标实际使用 Qt 文本翻译函数的文字。直接 `setText()` 的硬编码、图像文字、画布文字不会经过该入口。
- `qtTrId` / `qsTrId` 在运行时首先携带 ID，不保证携带 Dictionary 所需的源文；首版应明确放行 ID 模式，除非
  后续能从目标自己的 catalog 得到稳定 `id -> source` 关系。
- 安装 Translator 会发出 `LanguageChange`，但 Qt Widgets 要由目标在 `changeEvent()` 中重新设置文字，Designer
  界面通常调用 `retranslateUi()`；因此“之后创建/重新查询的文字能翻译”与“已经显示的所有文字立刻刷新”必须分开
  验收，不能强行遍历控件改状态。
- 静态链接 Qt 时没有独立 QtCore 动态模块可作为通用入口，应在能力检测阶段拒绝，而不是做签名扫描。

([动态语言变更](https://doc.qt.io/qt-6/i18n-source-translation.html#prepare-for-dynamic-language-changes),
[ID 翻译](https://doc.qt.io/qt-6/qttranslation.html),
[Qt 静态与动态部署](https://doc.qt.io/qt-6/deployment.html))

### ABI 边界

Qt 只承诺同一 major 内、相同工具链与系统环境、相同构建配置的动态二进制向后兼容；不承诺跨 major 或任意
C++ ABI。返回 `QString`、构造 `QTranslator` 和调用虚函数都落在这个边界内。因此首版不得声称“一个 DLL 覆盖所有
Qt 软件”，而应：

1. 按已加载模块识别 Qt major、位数和受支持工具链；
2. Qt 5 与 Qt 6 分开构建 Adapter；
3. 首版只接受一个明确工具链族与动态 release 构建；
4. 无法证明兼容时返回“不兼容”，绝不尝试调用猜测出的 C++ ABI。

([Qt 版本与二进制兼容承诺](https://doc.qt.io/qt-6/qt-releases.html#compatibility-promises))

### 首个确定性合同

应先建设一个仓库内合成宿主，不依赖真实软件：

1. Qt Widgets 标签通过 `tr("Open")` 创建；QML 文本通过 `qsTr("Save")` 创建。
2. 激活 Adapter 后，首次查询取得原文并返回 Dictionary 译文。
3. 更新 Dictionary 快照后，重新触发翻译得到第二代译文；旧快照在并发查询结束前保持有效。
4. 停用并移除 Translator 后，重新翻译恢复宿主原有 Translator 或源文。
5. 硬编码 `setText()` 与 ID-based 翻译明确放行并记录不支持原因。
6. 至少覆盖 Qt major 不匹配、静态链接/缺少 QtCore、安装失败、异常卸载的 fail-open。

真实发布门槛仍是：一个授权目标出现翻译入口命中、Dictionary 匹配、可见译文和停用恢复；仅成功注入或收到
`LanguageChange` 不算完成。

## 2. WPF / WinUI：拆开判断

### WinUI / MRT Core：第二候选

Windows App SDK 的 `ResourceLoader.GetString(resourceId)` 接收资源 ID 并返回最佳匹配字符串；MRT Core 还公开
`MrmLoadStringResourceFromResourceUri` 等原生 C API。后者返回资源字符串缓冲区，调用方必须用
`MrmFreeResource` 释放；同一 API 家族也提供 `MrmAllocateBuffer`。因此，如果真实 WinUI 文字解析确实命中这些
导出，Adapter 可以先调用原函数取得当前字符串，以该字符串查 Dictionary，再使用同族分配器返回译文，内存
所有权比跨 C++ `QString` 更清楚。
([ResourceLoader.GetString](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.windows.applicationmodel.resources.resourceloader.getstring),
[MrmLoadStringResourceFromResourceUri](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringresourcefromresourceuri),
[MRT Core C API](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/))

但这仍只是“可行入口”，不是已证明的常用调用链。WinUI 的 `x:Uid` 会形成 `Uid.PropertyName` 资源键，MRT Core
在构建期把资源写入 PRI；公开文档没有承诺 XAML 内部解析必经上述某一个 C 导出。因此第二候选的第一步只能是
三层合成命中矩阵：显式 `ResourceLoader.GetString`、显式 MRT C API、声明式 `x:Uid`。只有第三层也命中，才有
资格继续真实目标测试。
([WinUI `x:Uid`](https://learn.microsoft.com/en-us/windows/apps/develop/platform/xaml/x-uid-directive),
[MRT Core 架构](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/mrtcore/mrtcore-overview))

### WPF：当前不选

.NET 的 `ResourceManager.GetString(name)` 是清晰的“资源名 -> 字符串”入口，但 WPF 官方同时支持 BAML/x:Uid、
`.resx`、ResourceDictionary、绑定和代码文字等多种路径。官方 BAML 本地化流程是构建后从资源程序集提取
key/value、生成新的卫星资源程序集，并不是一个保证所有运行时文字都会经过的公共回调。
([.NET ResourceManager 取字符串](https://learn.microsoft.com/en-us/dotnet/core/extensions/retrieve-resources),
[WPF 全球化与 BAML 本地化](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/advanced/wpf-globalization-and-localization-overview),
[WPF ResourceDictionary](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/advanced/how-to-use-a-resourcedictionary-to-manage-localizable-string-resources))

要在不改目标代码的前提下统一修改托管返回值，通常要进入 CLR Profiler / ReJIT。官方说明 Profiler DLL 在 CLR
内运行、推荐通过 ReJIT 修改 CIL，并明确指出这类 IL 修改十分敏感；.NET Framework 虽提供运行中
`AttachProfiler`，仍有位数、权限、运行时兼容和“已有 profiler”限制。这比当前 Adapter 的原生 fail-open Hook
多出一整套运行时产品面，不应为了“支持 WPF”混入 Qt 或 WinUI slice。
([CLR Profiling 概览](https://learn.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/profiling-overview),
[运行中 AttachProfiler](https://learn.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/iclrprofiling-attachprofiler-method))

## 3. Chromium / CEF / Electron：改为独立 Extension

### 为什么 DOM 可修改仍不等于 Adapter

Chrome content script 与页面共享 DOM，所以可以读取文字节点并修改它们；但默认运行在隔离世界，注入需要扩展
安装以及对应 host permission。DOM 中的当前文本也不一定是稳定源文：响应式框架会重新渲染，Shadow DOM、
iframe、虚拟列表与 canvas 各有独立生命周期。修改 DOM 是第二套状态，不是拦截应用本地化函数并返回译文。
([Content scripts 与隔离世界](https://developer.chrome.com/docs/extensions/develop/concepts/content-scripts),
[扩展 host permissions](https://developer.chrome.com/docs/extensions/develop/concepts/declare-permissions))

CDP 的 `Runtime.evaluate` 与 `Page.addScriptToEvaluateOnNewDocument` 可以注入脚本，但外部连接要求目标开放调试。
Chrome 从 136 起还要求 remote debugging 使用非默认 user data directory，说明该通道被明确视为开发/自动化边界，
不适合作为任意已运行软件的默认能力。
([CDP Runtime](https://chromedevtools.github.io/devtools-protocol/tot/Runtime/),
[CDP Page](https://chromedevtools.github.io/devtools-protocol/tot/Page/),
[remote debugging 安全边界](https://developer.chrome.com/blog/remote-debugging-port))

Electron 的 preload 必须由应用在创建 `BrowserWindow` 时配置，`webContents.debugger` 也由 Electron main process
持有；Electron 的扩展 API 只支持 Chrome 扩展子集、只加载 unpacked extension，并要求应用每次启动主动调用
`loadExtension`。这些是良好的“应用主动集成”入口，不是外部通用注入承诺。
([Electron 进程模型与 preload](https://www.electronjs.org/docs/latest/tutorial/process-model),
[Electron Debugger](https://www.electronjs.org/docs/latest/api/debugger/),
[Electron 扩展边界](https://www.electronjs.org/docs/latest/api/extensions))

CEF 同样是多进程架构。`CefResourceBundleHandler::GetLocalizedString` 只接收 Chromium 资源的整数 ID，并由宿主
通过 `CefApp::GetResourceBundleHandler` 提供；页面 JavaScript、V8 context 和 DevTools 方法则由 browser/render
process 中的宿主对象持有。它们适合软件作者集成，却没有提供一个外部进程可枚举全部 browser 并安全接管页面
本地化的稳定全局入口。
([CEF ResourceBundleHandler](https://cef-builds.spotifycdn.com/docs/112.3/classCefResourceBundleHandler.html),
[CEF CefApp](https://cef-builds.spotifycdn.com/docs/127.3/classCefApp.html),
[CEF 多进程与 IPC](https://chromiumembedded.github.io/cef/general_usage.html),
[CEF BrowserHost DevTools](https://cef-builds.spotifycdn.com/docs/141.0/classCefBrowserHost.html))

因此这一族若进入 roadmap，应采用独立 Extension 合同：用户显式安装/授权；每个 frame 建立 MutationObserver；
只修改文本节点而不替换元素；保存原值与应用最新值的版本关系；停用时仅恢复仍由 Extension 持有的值；框架重绘
冲突时让应用优先。它可以扩大 Web 桌面软件覆盖，但不应伪装成与 GDI+、Qt Translator 相同的原生 Adapter。

## 执行建议

下一 slice 只实现 **Qt 合成宿主 + 能力探测 + 最小 Translator 生命周期**，不同时建设 WPF profiler、MRT Hook
或 DOM 注入。进入生产前使用以下否决条件：

- QtCore 动态模块、major、位数或工具链不在白名单：不兼容。
- 合成宿主拿不到真实 `sourceText`、不能 fail-open 或移除后不能恢复：否决实现。
- 授权真实目标只有注入成功或事件命中，没有 Dictionary 匹配和可见替换：不进 Bundle。
- 目标主要使用硬编码、ID-based catalog 或自绘文字：记录覆盖边界，不增加绘制层猜测来“补命中”。

如果 Qt 在真实目标上被否决，下一步不是回到 DirectWrite，而是执行 WinUI 三层 MRT 命中矩阵；只有典型
`x:Uid` 宿主能落到可安全替换的原生资源返回入口，才把它提升为正式 Adapter 候选。
