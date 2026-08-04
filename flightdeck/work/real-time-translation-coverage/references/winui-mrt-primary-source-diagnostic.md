# WinUI 3 / MRT Core 原生入口诊断

- 状态：完成
- 调研日期：2026-08-04
- 范围：只分析 Windows App SDK / WinUI 3 的资源加载链、公开 ABI 与可验证的一方源码；不修改生产代码。
- 源码快照：[WindowsAppSDK `ee3c5078`](https://github.com/microsoft/WindowsAppSDK/tree/ee3c507894501a659d859d09bb017b761ed0a7e3)、[microsoft-ui-xaml `188f602b`](https://github.com/microsoft/microsoft-ui-xaml/tree/188f602b27cdb47572b28c380e9c087b02e1ccee)。以下源码判断以这两个快照为准，而不是假定所有历史版本都相同。

## 结论先行

1. `Microsoft.Windows.ApplicationModel.Resources.ResourceLoader.GetString` 的原生实际入口已经确认：它直接调用导出的 `MrmLoadStringResource`；URI 版本调用 `MrmLoadStringResourceFromResourceUri`。两者返回由 MRT Core 分配的 `PWSTR`，调用者必须用 `MrmFreeResource` 释放。[`ResourceLoader.cpp`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Microsoft.Windows.ApplicationModel.Resources/src/ResourceLoader.cpp) 与 [Win32 API 文档](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringresource) 给出了同一所有权规则。
2. WinUI 3 声明式 `x:Uid` 在使用默认 MRT Core `ResourceManager` 时，确实进入同一组 `MRM` C 导出，但**不经过** `ResourceLoader.GetString` 或 `MrmLoadStringResource`。当前 WinUI 源码会枚举属性包，最后落到 `MrmLoadStringOrEmbeddedResourceByIndex`。[`ModernResourceProvider.cpp`](https://github.com/microsoft/microsoft-ui-xaml/blob/188f602b27cdb47572b28c380e9c087b02e1ccee/dxaml/xcp/components/mrt/ModernResourceProvider.cpp) 与 [`ResourceMap.cpp`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Microsoft.Windows.ApplicationModel.Resources/src/ResourceMap.cpp) 可以闭合这条调用链。
3. 因而不存在一个既窄、又能覆盖显式 `GetString` 和声明式 `x:Uid` 的公开 C 导出。最小可验证组合至少是：
   - 显式字符串：`MrmLoadStringResource`；
   - `x:Uid` 属性包：`MrmLoadStringOrEmbeddedResourceByIndex`。
4. 内存层面可以安全替换，但语义层面必须收紧：仅在原调用成功、类型明确为字符串、调用来源/资源图谱已被证明、属性名属于文本白名单、字典精确命中时替换；新 `PWSTR` 必须由 `MrmAllocateBuffer` 分配，旧值必须由 `MrmFreeResource` 释放。任何条件不满足都原样放行。
5. `x:Uid` 是**加载时赋值**，不是绘制时回调。字典改变只会影响之后重新加载的资源；已经显示的控件不保证即时变化或恢复，微软也明确提示已加载 UI 可能需要重新加载甚至重启。因此它目前最多应命名为“WinUI 3 加载时字典替换”，不能宣称为完整的实时翻译适配器。[`PrimaryLanguageOverride` 备注](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.windows.globalization.applicationlanguages.primarylanguageoverride?view=windows-app-sdk-1.8)
6. UWP 的 System MRT 是另一套产品边界。Windows App SDK 的 MRT Core C 导出不能据此宣称覆盖 UWP；若要支持 UWP，必须建立独立的一方源码与 ABI 证据链。

## 1. 显式 MRT Core C/C++ API 的真实字符串入口

### 1.1 公开导出面

当前官方 `MRM.def` 明确导出以下资源读取函数以及配对的分配/释放函数：[`MRM.def`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Core/src/MRM.def)。声明位于 [`MRM.h`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Core/src/MRM.h)，微软的 [MRT Core Win32 API 索引](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/_mrtcore/) 也列出同一组公开函数。

| 入口 | 定位方式 | 输出 | 主要调用者/用途 | 本次判断 |
| --- | --- | --- | --- | --- |
| `MrmLoadStringResource` | `resourceId` | `PWSTR` | `ResourceLoader.GetString` | 显式字符串主入口 |
| `MrmLoadStringResourceFromResourceUri` | `resourceUri` | `PWSTR` | `ResourceLoader.GetStringForUri` | 显式 URI 字符串入口 |
| `MrmLoadStringOrEmbeddedResource` | `resourceId` | `MrmType` + 字符串/路径/二进制 | `ResourceMap.GetValue*` | 覆盖更广，不宜无差别替换 |
| `MrmLoadStringOrEmbeddedFromResourceUri` | `resourceUri` | 同上 | URI 型 `ResourceMap` 访问 | 覆盖更广，不宜作为第一入口 |
| `MrmLoadStringOrEmbeddedResourceByIndex` | `ResourceMap` 内索引 | `MrmType` + `resourceName` + 值 | `ResourceMap.GetValueByIndex*`，当前 `x:Uid` 属性包枚举 | `x:Uid` 最窄公开 C 入口 |
| 带 `WithQualifierValues` 的三个变体 | 同上并返回 qualifier | 同上 | 需要观察候选选择依据的高级调用 | 当前链路未证明必经，不应先挂全量替换 |

`MrmLoadStringResource` 的公开文档称其可返回 `MrmType_String` **或** `MrmType_Path`，输出须由 `MrmFreeResource` 释放；URI 版本规则相同。[`MrmLoadStringResource`](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringresource)、[`MrmLoadStringResourceFromResourceUri`](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringresourcefromresourceuri)

`MrmLoadStringOrEmbeddedResourceByIndex` 额外返回 `resourceType` 与 `resourceName`，字符串/路径、名称或嵌入数据都由调用者通过 `MrmFreeResource` 释放。[`MrmLoadStringOrEmbeddedResourceByIndex`](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringorembeddedresourcebyindex)

### 1.2 实现内部发生了什么

在官方 [`MRM.cpp`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Core/src/MRM.cpp) 中：

- 导出的 `MrmLoadStringResource` 调用文件内 `static LoadStringResource`；后者完成候选解析和类型检查。
- 返回前调用 `StringResultReleaseOwnershipBuffer`。源码注释明确说明这是为了返回一份可由调用者持有的副本，而不是指向 PRI 文件的指针。
- `MrmAllocateBuffer` 最终调用 MRT Core 自己的 `Def_Alloc`；`MrmFreeResource` 调用配对的 `Def_Free`。

所以真正可依赖的 ABI 边界是 `MRM.def` 中的公开函数，而不是 `static LoadStringResource`、PRI 内部对象或未导出的 C++ 符号。不存在一个公开的“所有字符串最终统一经过这里”的单函数入口。

## 2. `ResourceLoader.GetString` 调用链与所有权

### 2.1 调用链

WinRT IDL 把接口定义为 `String GetString(String resourceId)` 和 `String GetStringForUri(Uri resourceUri)`：[`Microsoft.Windows.ApplicationModel.Resources.idl`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Microsoft.Windows.ApplicationModel.Resources/src/Microsoft.Windows.ApplicationModel.Resources.idl)。其实现链为：

```text
WinRT String/HSTRING resourceId
  -> ResourceLoader::GetString(hstring const&)
  -> resourceId.c_str()                         // 借用输入字符指针
  -> MrmLoadStringResource(..., PWSTR*)         // 返回 MRT Core 所有权域的副本
  -> string_resoure_ptr(PWSTR)                  // RAII，析构调用 MrmFreeResource
  -> winrt::to_hstring(PWSTR)                   // 复制为独立 winrt::hstring
  -> WinRT ABI 返回 HSTRING                     // 所有权转移给投影/调用者
```

这条链在 [`ResourceLoader.cpp`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Microsoft.Windows.ApplicationModel.Resources/src/ResourceLoader.cpp) 中是直接调用，不是推测。其 [`pch.h`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Microsoft.Windows.ApplicationModel.Resources/src/pch.h) 将 `string_resoure_ptr` 定义为 `std::unique_ptr<wchar_t, StringResourceFreer>`，而 deleter 只调用 `MrmFreeResource`。

### 2.2 `PWSTR` 与 `HSTRING` 是两个所有权域

- MRM C 层输出的是可写 `PWSTR`，由 `MrmFreeResource` 释放。[API 文档](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringresource)
- `winrt::hstring` 封装 Windows Runtime 的 `HSTRING`；从 `wchar_t const*` 构造会复制输入。`detach_abi` 用于把底层句柄移交给 ABI 调用者。[C++/WinRT `hstring` 文档](https://learn.microsoft.com/en-us/uwp/cpp-ref-for-winrt/hstring)
- `HSTRING` 的释放函数是 `WindowsDeleteString`；它减少底层缓冲区引用计数，并在归零时释放。[`WindowsDeleteString`](https://learn.microsoft.com/en-us/windows/win32/api/winstring/nf-winstring-windowsdeletestring)

因此不能把 MRM 的 `PWSTR` 直接冒充成 `HSTRING`，也不能用 `WindowsDeleteString` 释放 MRM 缓冲区，反之亦然。

### 2.3 两种拦截层的正确替换方式

**拦截 `MrmLoadStringResource` 导出：**

1. 先调用原函数；仅在 `SUCCEEDED(hr)` 且输出非空时读取。
2. 完成只读观察、精确字典查询与全部过滤。
3. 若决定替换，先做长度加法与字节乘法的溢出检查，再用 `MrmAllocateBuffer((length + 1) * sizeof(wchar_t))` 创建新缓冲区并复制终止零。
4. 用 `MrmFreeResource` 释放原输出，再把输出指针换成新缓冲区。
5. 分配失败、异常、字典未命中或状态不确定时返回原结果，不改变 HRESULT。

**拦截公开 WinRT `IResourceLoader::GetString` ABI：**

- 这是语义上更窄的 `GetString` 边界，但不是一个普通 C 导出；需要正确处理 WinRT 接口实例、vtable 与 ABI 生命周期。
- 替换值必须是新建并转移所有权的合法 `HSTRING`；在成功替换旧输出时，旧 `HSTRING` 要按 ABI 所有权用 `WindowsDeleteString` 处理。
- 不应定位或修改未导出的 `ResourceLoader::GetString` C++ 实现地址；那会退化为版本相关的符号/签名扫描。

对现有以模块导出为基础的 Adapter 体系，`MrmLoadStringResource` 更可实现；对“最窄语义入口”的定义，WinRT `IResourceLoader::GetString` ABI 更窄，但工程复杂度和漏接既存对象的风险更高。两者都不覆盖 `x:Uid`。

## 3. WinUI 3 声明式 `x:Uid` 是否经过同一入口

### 3.1 结论：同一 MRT Core 家族，不是同一个函数

官方文档说明 `x:Uid="GoButton"` 会把诸如 `GoButton.Content`、`GoButton.FlowDirection` 的资源值应用到对应属性；属性包并不只含可见文本。[`x:Uid` 指令](https://learn.microsoft.com/en-us/windows/apps/develop/platform/xaml/x-uid-directive) 更完整的 WinUI 本地化示例还包含 `Text`、`Width`、`Foreground` 与附加属性，证明盲目替换“所有字符串候选”会破坏布局或类型转换。[WinUI 3 本地化文档](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/mrtcore/localize-strings)

当前官方源码的真实链如下：

```text
XAML 元素 x:Uid
  -> WinUI XAML 生成/请求 ms-resource://... 属性包路径
  -> ModernResourceProvider::GetPropertyBag
  -> IResourceManager.MainResourceMap
  -> IResourceMap.TryGetSubtree(propertyBagResourcePath)
  -> IResourceMap.ResourceCount
  -> IResourceMap.GetValueByIndexWithContext(index, context)
  -> MRT Core ResourceMap::GetValueByIndexImpl
  -> MrmLoadStringOrEmbeddedResourceByIndex
  -> resourceName 作为属性名，ResourceCandidate.ValueAsString 作为属性值
  -> WinUI 把属性包应用到控件
```

证据逐段如下：

- WinUI 的 [`ModernResourceProvider::GetPropertyBag`](https://github.com/microsoft/microsoft-ui-xaml/blob/188f602b27cdb47572b28c380e9c087b02e1ccee/dxaml/xcp/components/mrt/ModernResourceProvider.cpp) 构造属性包 URI、取得主 ResourceMap、查找子树、按索引枚举，并把 key/value 克隆为属性名和值。
- Windows App SDK 的 [`ResourceMap::ResourceCount` / `GetSubtreeImpl`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Microsoft.Windows.ApplicationModel.Resources/src/ResourceMap.cpp) 分别调用 `MrmGetResourceCount` 与 `MrmGetChildResourceMap`。
- 同文件的 `ResourceMap::GetValueByIndexImpl` 调用 `MrmLoadStringOrEmbeddedResourceByIndex`，检查 `MrmType`，再把 `resourceName` 与字符串候选转换成独立 WinRT 对象。

所以只挂 `MrmLoadStringResource` 会完整漏掉当前默认 `x:Uid` 路径。

### 3.2 何时会经过，以及何时会绕开

它会在 WinUI 3 使用默认 `Microsoft.Windows.ApplicationModel.Resources.ResourceManager`、XAML 加载器为带 `x:Uid` 的元素取得并应用属性包时经过上述 `ByIndex` 入口。典型时机是页面/控件实例化或资源重新加载，而不是每一帧绘制。

它会在以下场景绕开或无法保证经过：

- 应用通过 `Application.ResourceManagerRequested` 提供自定义 `IResourceManager`。WinUI 官方设计明确写明，自定义实现会取代默认 MRT Core `ResourceManager`；为空才实例化默认实现。[自定义 ResourceManager 设计](https://github.com/microsoft/microsoft-ui-xaml/blob/188f602b27cdb47572b28c380e9c087b02e1ccee/docs/design-notes/custom-mrt-resourcemanager.md)、[正式规格](https://github.com/microsoft/microsoft-ui-xaml/blob/188f602b27cdb47572b28c380e9c087b02e1ccee/specs/custom-iresourcemanager-spec.md)
- 目标使用的是 UWP/System MRT，而不是 WinUI 3/Windows App SDK MRT Core。
- 目标版本的 WinUI/Windows App SDK 实现链与本报告源码快照不同，或者所需导出不在实际加载模块中。
- 文本是硬编码、运行时自行生成、网络返回，或通过别的 UI/本地化框架设置；这些本来就不属于 `x:Uid`。

### 3.3 实时性边界

`x:Uid` 的结果会被写入控件属性。官方 `PrimaryLanguageOverride` 文档明确说明：新设置反映在**之后加载**的资源上；已加载到 UI 的资源可能不会立即变化，可能需要重新加载，甚至重启应用。[`PrimaryLanguageOverride`](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.windows.globalization.applicationlanguages.primarylanguageoverride?view=windows-app-sdk-1.8)

对 GlyphShift 的直接含义：

- 字典修改后，新创建/重新加载的页面可以取得新译文。
- 已经显示的控件没有统一、公开的反向映射或刷新 API，不能承诺立刻更新。
- 停止 Adapter 只会停止之后的替换，不能自动把已写入控件的译文恢复为原文。
- 若产品必须满足“改字典后当前界面立即变化、停止后立即恢复”，则单靠 MRT Core 返回值替换不合格；还需要目标应用授权的页面刷新/重建契约，而这不是通用能力。

## 4. Packaged、unpackaged、Windows App SDK 版本与 UWP 差异

### 4.1 打包形态

| 维度 | Packaged WinUI 3 | Unpackaged WinUI 3 | 对 Adapter 的影响 |
| --- | --- | --- | --- |
| PRI 默认发现 | 包根目录的 `resources.pri` 在 `ResourceManager` 创建时自动加载。[MRT Core 概览](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/mrtcore/mrtcore-overview) | 从代码解析时没有默认 view，官方要求把 `.pri` 路径传给 `ResourceManager`；手工生成并部署 `resources.pri`。[本地化文档](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/mrtcore/localize-strings#loading-strings-in-unpackaged-applications) | 不要假设同一个 root map、PRI 路径或资源 URI host |
| 语言集合 | 支持语言声明在 package manifest；用户首选语言列表参与解析 | 没有 package manifest；显式声明支持语言，语言解析使用系统显示语言 | 观察到的候选可能不同，但 C API 所有权规则相同 |
| Windows App SDK 运行时 | 可依赖框架包，或把依赖作为 MSIX 内容自包含 | 框架依赖模式要初始化 Windows App SDK runtime；自包含模式把依赖复制到应用旁 | 不能硬编码 DLL 磁盘路径；应在目标进程内按已加载模块、架构、版本和导出表识别 |

微软说明自包含部署会把 Windows App SDK Framework 内容随应用部署：packaged 时进入 MSIX，unpackaged 时复制到可执行文件旁。[自包含部署](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/self-contained-deploy/deploy-self-contained-apps) 框架依赖的 unpackaged 应用则要先把 Windows App SDK framework package 加入包图，之后才能使用 MRT Core。[unpackaged 部署](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/deploy-unpackaged-apps)

结论是：packaged/unpackaged 并不改变本报告公开函数的签名，但会改变资源根、语言选择、运行时装载来源和模块位置。Adapter 探测必须依赖目标进程实际加载的模块与导出，而不是安装目录猜测。

### 4.2 Windows App SDK 版本

- Win32 MRT Core 文档把相关 C API 的最低支持标为 Windows 10 1809 + Windows App SDK 0.5 或更高。[`MrmLoadStringResource`](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringresource)
- Windows App SDK 1.0 Preview 1 及以后，WinRT 命名空间是 `Microsoft.Windows.ApplicationModel.Resources`；更早版本是 `Microsoft.ApplicationModel.Resources`。[MRT Core 概览](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/mrtcore/mrtcore-overview)
- .NET 项目的资源 Build Action 自动化从 0.8 开始；1.0 曾有构建问题，1.1 修复。该差异影响 PRI 生成，不改变已证明的内存所有权规则。[MRT Core 概览](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/mrtcore/mrtcore-overview)
- 本报告中的 `x:Uid -> ByIndex` 是对指定源码快照的证明，不应外推为全部版本的永久 ABI 承诺。每个实际版本都要做三项准入：识别 Windows App SDK/WinUI 模块版本、核验所需导出存在、用合成宿主验证调用链后才启用替换。

### 4.3 UWP/System MRT 不是 MRT Core

微软把 Windows App SDK MRT Core 定义为 UWP MRT 的精简版本，并明确指出两者 API 命名空间、`ResourceManager` 获取方式、`ResourceContext` 和 qualifier 自动填充行为不同：

- UWP 使用 System MRT，相关 API 位于 `Windows.ApplicationModel.Resources` / `Windows.ApplicationModel.Resources.Core`，`ResourceManager.Current` 与 current-view/view-independent context 属于该模型。
- Windows App SDK 使用 `Microsoft.Windows.ApplicationModel.Resources`，主动创建 `ResourceManager`；MRT Core 不再使用 current view / view-independent 的概念，默认只自动填充 Language，其他 qualifier 由应用处理。
- 不是所有 UWP MRT API 都存在于 MRT Core。

这些差异由微软的 [MRT 到 MRT Core 迁移指南](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/migrate-to-windows-app-sdk/guides/mrtcore) 明确列出。故本次 `MRM` 导出方案的支持范围只能写“Windows App SDK MRT Core / 默认 WinUI 3 资源管理器”；将其标为 UWP 适配器必须否决。

## 5. 最窄可行入口与否决项

### 5.1 建议准入矩阵

| 候选入口 | 只读观察 | 字典替换 | 覆盖 | 决策 |
| --- | --- | --- | --- | --- |
| `MrmLoadStringResource` | 可行：成功返回后复制观察 | 有条件可行；需 MRM 配对分配/释放，并建立调用来源或资源 map 证据，避免误改路径资源 | 显式 `ResourceLoader.GetString` | 允许做确定性原型 |
| `MrmLoadStringResourceFromResourceUri` | 可行 | 同上 | 显式 `GetStringForUri` | 仅在样本证明需要时增加 |
| `MrmLoadStringOrEmbeddedResourceByIndex` | 可行；能得到 type/name/value | 有条件可行；仅 `MrmType_String`、已证明的属性包子树、文本属性白名单、字典精确命中 | 当前默认 WinUI 3 `x:Uid` | 允许做加载时替换原型 |
| `MrmLoadStringOrEmbeddedResource` | 可行 | 默认否决；它同时承载 string/path/embedded，调用面明显更宽 | 通用 `ResourceMap.GetValue*` | 先观察，不进入第一版替换 |
| `IResourceLoader::GetString` WinRT ABI | 可行但需对象/ABI 拦截 | 语义最窄，HSTRING 所有权正确时可行 | 只覆盖显式 `GetString` | 作为后备研究，不做首个导出型 Adapter |
| `ResourceCandidate::get_ValueAsString` / `IResourceMap` vtable | 技术上可观察 | 覆盖面与对象管理复杂，容易包含路径和非文本候选 | 多种 ResourceMap 消费者 | 否决第一版 |
| 未导出 C++ 函数、PRI 内存、签名扫描 | 不稳定 | 不安全 | 实现内部 | 否决 |
| 用同一方案宣称支持 UWP/System MRT | 证据不足 | 证据不足 | UWP | 否决 |

### 5.2 `x:Uid` 替换必须同时满足的护栏

1. 目标进程、架构与模块经过现有授权和兼容性检查；找不到确切导出就不安装 hook。
2. 跟踪 `MrmGetChildResourceMap` 的成功结果，只对已证明来自 XAML 属性包查找的 map handle 启用 `ByIndex` 替换；不能把“被枚举的任意 ResourceMap”都当成 `x:Uid`。仅凭 URI 长得像 `ms-resource://...` 仍不足以证明来源；若无法用稳定元数据和合成测试建立来源证据，就只观察、不替换。
3. 原函数成功且 `resourceType == MrmType_String`；`Path` 与 `Embedded` 永不替换。
4. `resourceName` 必须属于经过合成宿主验证的文本属性白名单。首轮应从极窄的 `Text`、`Content` 测试面开始，不把 `Width`、`FlowDirection`、`Foreground` 等非文本属性纳入。
5. 原文经过有界 UTF-16 复制后再查字典；不在 hook 内持有原缓冲区指针，不执行网络请求，不做 AI 翻译，不阻塞 UI 线程。
6. 仅精确命中本地已发布字典快照时替换；不做模糊匹配、正则、自动分词或异步回写。
7. 使用 `MrmAllocateBuffer` / `MrmFreeResource` 维持分配器对称；替换失败时保留原 HRESULT、原指针和原行为。
8. 自定义 `IResourceManager`、未知版本、未知 map 来源、重入、异常或生命周期状态不明确时 fail-open。

### 5.3 产品命名与 Go/No-Go

**可以继续的最小成果：**一个只针对合成 WinUI 3 宿主的诊断原型，分开验证显式 `GetString` 与默认 `x:Uid` 两条链，支持观察以及下次资源加载时的精确字典替换。

**暂时不能宣称的能力：**

- 一个 hook 覆盖全部 WinUI 文本；
- 已显示界面随字典保存立即刷新；
- 停止 Adapter 后当前界面立即恢复原文；
- 覆盖自定义 `IResourceManager`；
- 覆盖 UWP/System MRT；
- 覆盖硬编码、网络或运行时动态生成文本。

因此建议状态为：**WinUI 3 / MRT Core“加载时字典替换”进入确定性原型，生产级“实时翻译 Adapter”暂缓准入。** 只有在产品接受“新页面/重载后生效”的语义，或另行找到通用且安全的 UI 刷新契约后，才升级名称和承诺。

## 6. 建议的确定性验证清单

所有测试都应使用仓库内的合成宿主与合成资源，不依赖真实第三方软件。

1. **显式资源宿主**：分别调用 `ResourceLoader.GetString`、`GetStringForUri`，证明对应两个导出命中、原文观察、精确替换、未命中放行。
2. **声明式属性宿主**：用 `x:Uid` 同时提供 `Text/Content` 与 `Width/FlowDirection`；证明只替换白名单文本，非文本属性完全不变。
3. **packaged/unpackaged 双宿主**：验证相同 ABI 所有权规则，并记录不同 PRI/root map/语言选择；不得依赖固定 DLL 路径。
4. **自定义 `IResourceManager` 宿主**：证明默认 MRM C hook 被绕开时 Adapter 会报告“不支持当前资源管理器”，而不是误报已启用。
5. **热更新语义**：修改字典后创建新页面/控件，确认新实例生效；同时记录旧实例不变，防止把加载时行为误写成实时刷新。
6. **停止语义**：停止后新加载实例回到原文；旧实例仍可能保留译文，并在 UI 中给出准确说明。
7. **所有权与压力**：循环加载/卸载页面，覆盖命中、未命中、长字符串、空字符串、分配失败、并发与重入；检查无泄漏、双重释放、悬空指针和 HRESULT 漂移。
8. **版本闸门**：至少覆盖计划支持的 Windows App SDK 稳定版本；任何未验证版本默认仅观察或直接禁用替换。

## 一方资料索引

- [MRT Core 概览](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/mrtcore/mrtcore-overview)
- [MRT Core Win32 API 索引](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/_mrtcore/)
- [`MrmLoadStringResource`](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringresource)
- [`MrmLoadStringOrEmbeddedResourceByIndex`](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/win32/mrm/nf-mrm-mrmloadstringorembeddedresourcebyindex)
- [Windows App SDK `MRM.cpp`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Core/src/MRM.cpp)
- [Windows App SDK `MRM.def`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Core/src/MRM.def)
- [Windows App SDK `ResourceLoader.cpp`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Microsoft.Windows.ApplicationModel.Resources/src/ResourceLoader.cpp)
- [Windows App SDK `ResourceMap.cpp`](https://github.com/microsoft/WindowsAppSDK/blob/ee3c507894501a659d859d09bb017b761ed0a7e3/dev/MRTCore/mrt/Microsoft.Windows.ApplicationModel.Resources/src/ResourceMap.cpp)
- [WinUI `ModernResourceProvider.cpp`](https://github.com/microsoft/microsoft-ui-xaml/blob/188f602b27cdb47572b28c380e9c087b02e1ccee/dxaml/xcp/components/mrt/ModernResourceProvider.cpp)
- [`x:Uid` 指令](https://learn.microsoft.com/en-us/windows/apps/develop/platform/xaml/x-uid-directive)
- [WinUI 3 本地化](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/mrtcore/localize-strings)
- [MRT 到 MRT Core 迁移](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/migrate-to-windows-app-sdk/guides/mrtcore)
- [C++/WinRT `hstring`](https://learn.microsoft.com/en-us/uwp/cpp-ref-for-winrt/hstring)
- [`WindowsDeleteString`](https://learn.microsoft.com/en-us/windows/win32/api/winstring/nf-winstring-windowsdeletestring)
