# Frida 与 Detours 的 Windows 双架构实现

研究日期：2026-09-07。仅研究第一方 Windows 源码和官方说明；未编译、注入或运行真实软件，未触及归档 UIA/OCR。源码快照固定为 Frida Core `39583a2761627c6dfaf90c78761d38c83914268a`、Microsoft Detours `adb07604aa56508448b95bf037c2a6d0d3b6831a`。下文区分源码事实和对 Glyphshift 的建议。

## 结论

两者都支持用统一上层入口处理 x86/x64，但目标内 DLL 仍分别编译。Frida 的 Windows 实现通过按目标架构选择 agent、按需选择同架构注入 backend 隐藏差异；Detours 通过配对 DLL 和临时 `rundll32.exe` 隐藏跨位数启动注入。它们证明“一个逻辑适配器，两种架构工件”是正常结构，并不证明 Windows 要求所有产品配备额外常驻 helper。[Frida 工件描述](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/winjector.vala#L132-L161)、[Detours 双架构说明](https://github.com/microsoft/Detours/wiki/OverviewHelpers)

Windows 的硬约束是原生进程与所加载原生 DLL 的架构匹配，跨位数进程可以 IPC。Helper 是否存在、何时退出属于实现选择；不能将同位数注入器设计解释成操作系统一律禁止跨位数进程操作。[Microsoft Process Interoperability](https://learn.microsoft.com/en-us/windows/win32/winprog64/process-interoperability)

## Frida：核对的是 Windows，不是 Linux helper

`WindowsHelperProcess.inject_library_file` 先比较目标 CPU 与 `Gum.NATIVE_CPU`。匹配时直接使用 `inprocess_backend`；不同架构时走 normal helper。权限失败另走 elevated helper，普通错误直接上抛，并非所有错误都尝试提权。[Windows 路由源码](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-process.vala#L59-L97)

Helper factory 启动宿主原生架构的 manager，manager 为支持的架构建立 helper 连接；`HelperService.can_handle_target` 以目标 CPU 等于自己的 `Gum.NATIVE_CPU` 为条件。没有匹配 helper 时明确返回 NOT_SUPPORTED。IPC 使用 pipe 上的 DBus，传递路径模板、字符串等数据；不是让 x64 直接把自身指针结构给 x86 backend。[manager 与 service 源码](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-service.vala)、[factory 与资源打包](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-process.vala)

`AgentDescriptor` 同时持有 x86、x86_64、arm64 的 agent blob 和同一命名模板。Backend 用实际目标 CPU 展开模板选文件。一个上层描述代表多份原生工件，而不是一份 DLL 包含两种可直接加载的机器码。[agent 资源](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/winjector.vala)、[目标架构展开](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-backend.vala#L49-L77)

注入 backend 的 C 实现分配目标内存、写入生成的机器码和 `FridaRemoteWorkerContext`、建立远程线程。上下文包含原生指针；x86/x64 stub 显式区分入口参数位置、系统调用与 C 调用约定。因此 helper 与目标同架构是这一实现的重要前提，不应移除架构路由后继续原样复制结构。[backend C：线程创建](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-backend-glue.c#L87-L148)、[上下文与机器码生成](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-backend-glue.c#L304-L559)

### 地址与失败检查的实际边界

- 该快照从当前 backend 的 `kernel32.dll` 枚举导出，把导出地址放入远程上下文；这里没有重新解析目标 PE 的实现。不能援引 Frida 来证明“本地 x64 的导出地址或 RVA 可给 x86 目标使用”。架构选择已经在上层完成。[地址初始化](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-backend-glue.c#L304-L345)
- 代码检查若干 kernel32 必需导出是否成功解析，但完整性谓词没有包含收集的 `GetLastError`。远程 stub 检查 `LoadLibraryW` 失败；其 `GetProcAddress` 返回值随后直接用于调用，在所读 x86/x64 分支未见空值检查。不能把“用了成熟项目的模式”视为缺失导出、错误 DLL 都能安全报告的保证。[完整性谓词](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-backend-glue.c#L573-L600)、[远程加载和调用](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-backend-glue.c#L493-L522)
- Backend 监视远程线程，helper close 会等待 pending 工作完成；Frida 这条链路的生命周期并非单次创建进程就结束。Glyphshift 若希望连接后 helper 退出，需要另定所有权与 agent IPC，不能直接照抄后把 helper 强行关掉。[监视和关闭](https://github.com/frida/frida-core/blob/39583a2761627c6dfaf90c78761d38c83914268a/src/windows/frida-helper-backend.vala)

## Detours：临时 helper 的直接例子

官方要求提供分别编译的配对 DLL，例如 `foo32.dll`、`foo64.dll`；混合架构路径通过 `rundll32.exe` 加载匹配 DLL 的 ordinal 1，即 `DetourFinishHelperProcess`。目标 DLL 的 `DllMain` 需识别 helper 环境并提前返回。[官方说明](https://github.com/microsoft/Detours/wiki/OverviewHelpers)

源码更明确：`DetourUpdateProcessWithDllEx` 的 32 位编译分支只更新 32 位目标导入表，64 位编译分支拒绝 32 位目标。`DetourCreateProcessWithDlls` 先以挂起状态创建进程，直接更新失败后尝试 helper，两条路径均失败则结束自己刚创建的进程。成功后按调用方原有挂起选项决定是否恢复主线程。这是启动时修改内存导入表的方案，不能当作 Glyphshift 附加已运行软件的现成替代。[架构拒绝](https://github.com/microsoft/Detours/blob/adb07604aa56508448b95bf037c2a6d0d3b6831a/src/creatwth.cpp#L818-L847)、[创建与失败路由](https://github.com/microsoft/Detours/blob/adb07604aa56508448b95bf037c2a6d0d3b6831a/src/creatwth.cpp#L1684-L1710)

Helper 分支复制任务 payload 后恢复 `rundll32`，等待它退出，再检查退出码。`DetourFinishHelperProcess` 在另一架构进程内执行导入表更新并结束。因此无需额外常驻服务；这个例子反驳的是“helper 必须常驻”，并没有证明“完全无需架构相关连接代码”。[helper 创建、等待与退出码](https://github.com/microsoft/Detours/blob/adb07604aa56508448b95bf037c2a6d0d3b6831a/src/creatwth.cpp#L1342-L1430)、[helper 工作入口](https://github.com/microsoft/Detours/blob/adb07604aa56508448b95bf037c2a6d0d3b6831a/src/creatwth.cpp#L1131-L1187)

## 对 Glyphshift 的启示与待决策项

以下是基于比较的设计建议，不是已实现或已验收的功能。

1. 保留单个逻辑适配器 ID；清单在其下列出 x86/x64 工件、ABI 版本、能力、哈希。目标 Runtime 和本机适配器必须同架构。缺少工件应显示该架构尚不支持，不能选另一架构凑合加载。
2. 控制平面继续保持 x64。会话内消息用固定宽度标量和序列化数据，不跨位数复制带指针的 Rust/C 结构。目标侧对外暴露稳定命令，而对象布局、Qt thiscall、COM vtable 等留在对应架构的实现内。
3. 可以复用现有 controller 的 x86 编译版本来执行 x86 会话，不一定再引入一个名字叫 helper 的常驻服务；这仍是按架构的执行组件。另一方案是只在附加阶段启动短命 broker，agent 握手后直连 x64 控制端。后者需另做 IPC、断线清理、恢复和卸载的所有权设计，不能只让 broker 提前退出。
4. 完全不启动额外进程的 x64→x86 连接也应作为备选，但需要实现目标架构的 PE/导出解析、目标机器码入口和明确的 x86 数据布局。这两份源码没有为 Glyphshift 提供可直接抄用且已验证的该方案，不能据此宣布工作量更小。
5. 借鉴工件选择和架构隔离，不照搬失败处理：部署前核对 PE machine、版本、必需导出和依赖；目标侧再次核对 ABI。远程加载、入口查找、握手失败都要有明确状态与清理。不要用“线程创建成功”替代 Runtime 已加载成功，也不要让缺失导出变成空地址调用。

是否采用短命 broker，应在本项目控制协议审计后确定。先把适配器身份、工件身份和进程架构分开，能够兼容常驻会话 controller、短命 broker 或直接跨位数连接，不必在产品层暴露三者差别。
