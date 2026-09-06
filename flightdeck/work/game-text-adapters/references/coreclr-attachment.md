# CoreCLR 游戏适配器接入研究

日期：2026-09-07。范围：Windows x64、.NET 6 CoreCLR、已运行游戏；仅文档和源码研究，未注入、未修改游戏、未完成真实拦截验证。

## 结论

建议将 **Diagnostics AttachProfiler + 定向 ReJIT/IL 改写**作为 CoreCLR 通用接入底座，将文字框架的方法选择留给 MonoGame 等适配配置。它不需要 SMAPI，也不依赖某个游戏的 Mod 加载器。能够加载 profiler 不代表已能翻译：托管 helper 引导、可见文字覆盖、字体和布局仍需要独立验证。

**特别限制：.NET 6 的 ReJIT profiler 不能承诺完全卸载。** 产品的“停用”应表示恢复原文、停止采集和停止新增改写；内置 Adapter 通过 `ProcessResidentAfterDeactivate` 标识告知组件可能保留到游戏退出。

## 已核实事实

### 1. 已运行进程的官方入口

Microsoft 的 `DiagnosticsClient.AttachProfiler` 向目标诊断通道发起 attach；CoreCLR 设计文档说明 Windows 使用命名管道，传入 profiler GUID、DLL 路径、超时及可选数据。这里加载的是原生 `ICorProfiler`，不是直接加载任意 C# DLL。当前客户端库名为 `Microsoft.Diagnostics.NETCore.Client`，不要照搬早期设计稿的旧类名。[客户端官方示例](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/diagnostics-client-library)、[CoreCLR attach 设计](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/profiling/Profiler%20Attach%20on%20CoreCLR.md)

.NET 6 的 `COR_PRF_ALLOWABLE_AFTER_ATTACH` 包含模块事件、JIT 事件和 `COR_PRF_ENABLE_REJIT`，不包含 `MONITOR_ENTERLEAVE`、参数跟踪及全局关闭优化/内联标志。因此不能设计为“后附加全局 enter hook，然后读取每个参数”。`ProfApi_RejitOnAttach` 的默认值为 1；`ReJitManager::IsReJITEnabled` 使用该配置判断 late attach 是否能 ReJIT。运行时配置可使其不可用，应保留明确失败状态。[v6 接口枚举](https://github.com/dotnet/runtime/blob/v6.0.0/src/coreclr/inc/corprof.idl)、[v6 配置默认值](https://github.com/dotnet/runtime/blob/v6.0.0/src/coreclr/inc/clrconfigvalues.h)、[v6 ReJIT 判定](https://github.com/dotnet/runtime/blob/v6.0.0/src/coreclr/vm/rejit.inl)

### 2. 内联、ReadyToRun 和 IL 改写

`RequestReJIT` 本身不追踪既有内联。`ICorProfilerInfo10::RequestReJITWithInliners` 专门覆盖目标及其内联调用者，自 .NET Core 3.0 可用。它比只替换目标入口更适合 late attach。对于没有可改写 IL 的方法，不能假设可用。[官方 API](https://learn.microsoft.com/en-us/dotnet/core/unmanaged-api/profiling/icorprofilerinfo10-requestrejitwithinliners-method)

ReadyToRun 不应被简单判为“不支持”，也不能只看 assembly 是托管程序集就认定所有路径可改写。应在合成验证中保留原始 ReadyToRun、分层编译和优化设置，验证已编译目标、已内联调用者、后续层级升级都走新逻辑。不要把关闭 ReadyToRun/优化的启动环境测试结果当作当前已运行游戏的结果。这里是验证策略推断；当前研究尚未完成具体游戏方法的 ReJIT 覆盖证明。

### 3. 托管 helper 是单独的难点

官方 profiling 说明禁止从普通 profiler 回调直接调用托管代码；安全设计是在被改写的方法 IL 中插入托管调用，由正常托管执行流执行。原生 profiler 应保持原生实现，不能在 `InitializeForAttach` 中直接运行 Harmony 或翻译 C# 逻辑。[官方 profiling 概述](https://learn.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/profiling-overview)

可选方案是第一次改写只插入一个有边界、幂等的托管引导过程：通过 BCL 可解析的加载入口加载 helper，再安装程序集解析并启用文字规则。必须先验证 metadata token 增补、加载上下文、重入、并发首次调用和异常隔离。该方案为工程推断，本次没有证据证明可直接用于星露谷。不能把增加 `AssemblyRef` 当成已经解决任意位置 DLL 的加载。

### 4. hostfxr 的用途和边界

.NET 6 的 native hosting 设计允许同一进程内创建 secondary host context，并复用通过 hostfxr 初始化的已有运行时；直接通过 coreclr API 创建的运行时不在其支持组合内。初始化会检查配置兼容性，不能重新设置已有 runtime 属性。`hdt_load_assembly_and_get_function_pointer` 可以装载 helper 并取得入口，但它默认放入隔离的 `AssemblyLoadContext`；将游戏自己的程序集再次这样加载会得到重复类型。[v6 native hosting 设计](https://github.com/dotnet/runtime/blob/v6.0.0/docs/design/features/native-hosting.md)

因此 hostfxr 适合“已有原生模块在目标进程中、安全时机加载 helper”这一环。它不是跨进程注入器，也不自动提供方法拦截。若使用这条备选路线，应通过反射定位目标进程已经加载的游戏/MonoGame 类型，避免 helper 私带一份框架后 patch 错对象。不要误用 .NET 8 才新增的独立 `hdt_load_assembly` API 解决 .NET 6 加载问题。

### 5. Startup hook 与 Harmony

`DOTNET_STARTUP_HOOKS` 在应用 `Main` 前调用 `StartupHook.Initialize()`，可用作框架级启动接入方式，无需专属 Mod。但对已经运行的游戏设置环境变量不会回到启动阶段，因此它只适合作为显式重启路线或合成测试引导方式。[官方 startup hook 设计](https://github.com/dotnet/runtime/blob/main/docs/design/features/host-startup-hook.md)

Harmony 是进入托管运行时后的 patch 工具，不是 CoreCLR 远程装载入口。官方明确说明既有内联可能导致 patch 不触发，需要更上层调用者处理；反射触及类型还可能提前执行静态构造器。可以比较它的实现成本，但不能由“支持 Harmony”推导出“支持所有 CoreCLR 游戏”。[Harmony 边界](https://harmony.pardeike.net/articles/patching-edgecases.html)

### 6. 恢复与卸载必须分开

`RequestRevert` 恢复指定函数未来调用的原始版本；正在执行的调用继续完成旧版本，并且必须逐方法检查返回状态，不能只看整体 `S_OK`。[官方 RequestRevert](https://learn.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/icorprofilerinfo4-requestrevert-method)

.NET 6 的 `EEToProfInterfaceImpl` 检查 `m_fModifiedRejitState`，设置过 ReJIT 状态后拒绝 profiler detach，返回不可逆 instrumentation 错误。不要用文档中“RequestRevert 恢复函数”推断 profiler DLL 也可卸载；也不要把 hostfxr context close 视为卸载 CoreCLR/helper。[v6 detach 源码及 SetEventMask 检查](https://github.com/dotnet/runtime/blob/v6.0.0/src/coreclr/vm/eetoprofinterfaceimpl.cpp)、[官方 detach API](https://learn.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/icorprofilerinfo3-requestprofilerdetach-method)

## 建议接入流程（设计推断）

1. 探测 CoreCLR 版本、架构、诊断通道以及目标文字框架，确定能力范围；没有 Mono runtime 就不能使用现有 Mono 导出函数。
2. 从适配器 worker 发起 attach，由原生 profiler 实现生命周期和模块枚举；只选择已知文字框架的完整签名，避免全局 string 方法改写。
3. 先对自有合成宿主验证 helper 引导，再对文字入口执行 `RequestReJITWithInliners`；等待实际命中才标记 capture-ready。
4. 被改写方法在布局/绘制的合适阶段读取本地不可变翻译快照。热路径不等待网络、不做同步文件 IO；错误及未命中均返回原文。
5. 配置同时声明测量和绘制入口，明确它是否覆盖自定义逐字绘制。采集到文字、替换成功和完整布局正确使用独立证据状态。
6. 停用先切换 identity bypass，再恢复所有被改写方法及相关调用者、检查逐项状态，排空事件；保留驻留组件直到游戏退出。恢复失败应继续 bypass，而不是强制卸载仍可能被调用的代码。

## 必须完成的最小验证

| 验证 | 合格条件 |
| --- | --- |
| late attach | 自有 .NET 6 x64 合成进程先 JIT/循环调用，再附加；不依赖启动环境改动 |
| helper 引导 | 无预置 helper 引用，仍可幂等加载；并发和加载失败不影响宿主 |
| 内联 / ReadyToRun / 分层编译 | 保留默认优化，附加前后均有确定命中证据 |
| 替换与恢复 | 合成字符串按配置替换；停用后新调用恢复原文 |
| 热路径 | 不阻塞渲染线程；缓存 miss 原文，队列有界，重复事件去重 |
| 游戏覆盖 | 可见菜单、对话、物品说明分别确认，不以 attach 成功代替翻译成功 |

以上是新增 CoreCLR 底座的研究方案，不是本次已实现或已验收的能力。
