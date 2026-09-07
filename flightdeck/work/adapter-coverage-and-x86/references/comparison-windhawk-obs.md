# Windhawk / OBS：一个逻辑模块如何覆盖 x86 与 x64

研究日期：2026-09-07。范围仅为第一方源码和构建配置审阅；没有编译、启动、注入或实机验证。UIA / OCR 归档不在本次研究范围。

## 结论先行

两个项目都支持“一个逻辑功能、多架构目标内工件”。它们没有要求用户为 x86 目标换一个完整主 App，也没有让一个普通原生 DLL 同时装入两种位数。

**连接机制有两种真实先例：OBS 使用同位数注入助手作为异位数路径；Windhawk 的已检查注入入口直接选择目标架构 shellcode 和 engine DLL，自己承担跨位数细节。** 因此“必须有 x86 App”错误，“Windows 规定必须有独立 x86 helper”也不准确。Helper 是封装 ABI 和故障边界的工程选择。

对 Glyphshift，合理的产品边界是“一套逻辑 Adapter + 按架构声明和发布的 Runtime / Adapter 工件 + x64 App”。是否使用助手属于连接层实现，不能拿它代替 Adapter 自身的 x86 ABI 适配。

## 可追溯版本

本次固定的是研究时官方默认分支快照，不把它们写成已发布稳定版；尤其 Windhawk 当前含 Rust core / CLI 与前端迁移代码，不能由它推断旧安装版本行为。

- Windhawk：[`61d99ed8e182e1af1b60109612b6763ad1b4b74e`](https://github.com/ramensoftware/windhawk/commit/61d99ed8e182e1af1b60109612b6763ad1b4b74e)。
- OBS Studio：[`6b3e550729f125b6c5b3767df88c08f5aef9d264`](https://github.com/obsproject/obs-studio/commit/6b3e550729f125b6c5b3767df88c08f5aef9d264)。

下文“源码事实”仅限已检查入口；“建议”是对 Glyphshift 的推论。

## Windhawk

### 一个 mod 身份，共用源码，各架构 DLL

源码事实：`compile_mod` 接收一个 `ModMetadata`、一个 `source`，创建一个带 mod ID / version 的 DLL 文件名，然后遍历目标架构，分别调用编译器。输出目录使用目标的 `subfolder()`，不是为 x86 和 x64 创建两个独立 mod 身份。编译目标映射将 i686 指向 `32`，x86_64 指向 `64`；未填写架构列表时默认是 `x86` 与 `x86-64`。这里也允许作者限制架构，所以“一个 mod”并不自动等于每个架构均兼容。[编译编排](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk-core/core/src/services/compiler/orchestrate.rs#L187-L320)、[目标映射](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk-core/domain/src/compile_targets.rs#L62-L183)。

目标内 engine 项目本身具有 Win32 / x64 构建配置；`StorageManager::GetEnginePath(machine)` 与 `GetModsPath(machine)` 分别选择 `32` / `64` 子目录。`Mod` 在当前 engine 进程里按架构过滤设置，再从当前架构的 mods 目录寻找同一个库文件名。[engine 项目](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/engine/engine.vcxproj#L3-L27)、[路径选择](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/engine/storage_manager.cpp#L202-L288)、[架构过滤](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/engine/mod.cpp#L120-L168)。

可借鉴：一个 Adapter ID、版本与设置，对应架构索引下多个独立工件。不要把构建产物的数量映射成用户目录里的适配器数量；也不要默认所有适配器都必须发布 x86。

### 连接层不是“固定每架构一个 helper”

源码事实：`DllInject` 对目标调用 `GetProcessArch`，选择 `x32Shellcode` / `x64Shellcode`，并使用目标机器类型寻找 `windhawk.dll`。接着由当前注入者写入代码和参数，按 APC 或远程线程路径启动。此入口没有通过 `CreateProcess` 启动一个同位数注入助手。[架构选择和部署](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/engine/dll_inject.cpp#L675-L835)。

跨位数复杂度并未消失：源码包含 WOW64 APC 地址编码，以及 32 位调用方进入 64 位 Nt 函数的特殊路径。嵌入的 shellcode 有独立源码工程，而不是把本进程 `LoadLibrary` 地址不加处理地复用到另一架构。[WOW64 分支](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/engine/dll_inject.cpp#L521-L669)、[shellcode 源码](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/engine/inject_shellcode/main.cpp)。

解释边界：这证明“独立 x86 helper 并非实现跨位数注入的必要条件”，不证明直接注入路径更简单，也不等于 Windhawk 没有服务、daemon 或其他管理进程。当前源码明确有服务与按登录会话启动的 daemon，它们解决管理与权限问题。[服务与用户会话](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/app/service.cpp#L80-L101)。

对 Glyphshift 的建议：除非有可衡量的启动时延或部署收益，不把维护跨位数 shellcode / WOW64 特殊入口当成第一阶段要求。用户所希望的一个 Adapter 与此选择无关。

### 权限、错误和停用

源码事实：服务尝试开启 debug privilege；全进程注入器有关键进程、已知不兼容程序、游戏的过滤配置，并对进程访问与注入异常分别处理。能选择正确架构不代表能访问任何目标；它没有用位数路由取代权限处理。[服务权限](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/app/service.cpp#L246-L254)、[过滤与错误](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/engine/all_processes_injector.cpp#L364-L516)。

编译阶段也按目标报告诊断：取消时清理部分 DLL；非零退出返回带目标信息的错误；编译器退出零后还检查工件实际存在。不要把编译成功消息等同可用包发布成功。[编译结果处理](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk-core/core/src/services/compiler/orchestrate.rs#L257-L336)。

停用存在 `BeforeUninit` / `Uninitialize` 阶段：先调用 mod 预停用入口，等待正在创建的 hook 并禁止新建，排队禁用 hook，然后有 mod 的卸载回调。**这不是文字恢复证明**：插件自己改变的文本、字体或 UI 状态是否恢复，仍取决于插件实现。[停用代码](https://github.com/ramensoftware/windhawk/blob/61d99ed8e182e1af1b60109612b6763ad1b4b74e/src/windhawk/engine/mod.cpp#L1456-L1496)。

## OBS Studio

### 同一个游戏捕获功能，多个目标内 hook 工件

源码事实：`win-capture` 是 App 内的插件；`graphics-hook` 是独立目标内模块，两者不能混为一谈。`graphics-hook` 与 `inject-helper` 各自使用同一份源码目标按平台生成 `graphics-hook32/64` 与 `inject-helper32/64`。父 x64 构建触发 Win32 子构建，子构建只包含所需辅助目标，并非要求额外发布一套完整 32 位 OBS UI。[win-capture 构建](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/CMakeLists.txt)、[hook 构建](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/graphics-hook/CMakeLists.txt)、[helper 构建](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/inject-helper/CMakeLists.txt)、[子架构配置](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/cmake/windows/architecture.cmake)。

安装辅助函数把 hook / helper 放进 `obs-plugins/win-capture` 数据目录，并从子构建收集带 `32` 后缀的文件；目标架构成为内部工件命名，未创建一个额外的“32 位游戏捕获”功能身份。[安装规则](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/cmake/windows/helpers.cmake#L1-L80)、[子构建工件收集](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/cmake/windows/helpers.cmake#L183-L236)。

边界：这是“一项功能使用架构不同的后台组件”的先例，不是 64 位 OBS 能直接加载 32 位第三方 App 插件的证据，也不是游戏文字汉化的先例。

### helper 的精确选择条件

`open_target_process` 获取目标句柄后识别位数；`inject_hook` 据此选 hook DLL 和 `inject-helper32.exe` / `inject-helper64.exe`。只有“调用方与目标同架构，且未启用兼容注入”时走 `hook_direct`；否则启动选定 helper。启动使用 `CREATE_NO_WINDOW`，保留 helper 进程句柄以检查结果。[目标识别](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/game-capture.c#L668-L704)、[完整路由](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/game-capture.c#L839-L964)。

因此不能概括成“OBS 所有注入都用 helper”，也不能概括成“helper 只为 x86 存在”。同位数的兼容路径同样用它。Glyphshift 不需要照搬游戏兼容 / 反作弊相关模式。

helper 读取命令行，执行指定加载操作，以退出码报告结果；它尝试调整当前 token 的 debug privilege，打开目标失败有独立错误码。主路径使用普通 `CreateProcessW`，不是 `runas` 提权，因此 helper 文件本身不自动解决管理员权限差异。[helper 实现](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/inject-helper/inject-helper.c#L19-L145)。

### 成功不仅是“helper 退出零”

源码事实：主插件区分 helper 退出、hook ready、实际 capturing 等状态。退出非零会报告失败；退出零但尚未捕获，仍会停止当前获取尝试并安排后续处理。停止会发 hook stop 事件，取消映射、关闭资源、销毁纹理、清除活动状态。[退出与就绪检查](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/game-capture.c#L1739-L1801)、[停止清理](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/game-capture.c#L312-L377)。

跨进程 `hook_info` 使用显式字段宽度、packing、版本字段以及 `sizeof == 648` 静态断言，并用偏移描述图形入口，而不是直接传当前 App 的 C++ 对象。它为双架构协议提供具体先例，但其窗口和共享句柄字段规则不能机械套到所有指针上。[共享协议](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/shared/obs-hook-config/graphics-hook-info.h#L29-L146)。

值得避免照搬的边界：该快照的 `is_64bit_process` 在查询失败时返回 false，调用方会将其当 32 位。Glyphshift 应保留 unknown / unsupported / query failed，不把失败映射成一个有效架构。源码可借鉴不等于每个旧实现都应复制。[位数判断失败分支](https://github.com/obsproject/obs-studio/blob/6b3e550729f125b6c5b3767df88c08f5aef9d264/plugins/win-capture/game-capture.c#L679-L690)。

## 对 Glyphshift 的选择建议

以下是设计建议，不是上述项目直接替我们完成的验证。

| 边界 | 建议 | 依据 / 限制 |
| --- | --- | --- |
| 用户目录 | 一个逻辑 Adapter，配置共用 | Windhawk 一个 mod 编译多目标；OBS 一项游戏捕获内部选工件 |
| 工件 | 按 Adapter ID + version + target architecture 索引；每项独立校验 | 两者均生成不同机器码模块；不能只给 runtime.dll 增加 x86 而忽略 Adapter DLL |
| 主 App | 保留 x64，负责产品工作流 | OBS 的 x86 子构建不要求第二个完整 App |
| 连接层 | 首选同位数的小助手方案，但把它隐藏在连接接口后 | OBS 提供可读、可分割的先例；Windhawk 证明未来可换直接跨位数实现 |
| ABI | App—助手用序列化消息；目标内 Runtime—Adapter 同架构；所有外部字段明确定义 | OBS 固定布局示例；Windhawk 为直接跨位数额外维护 shellcode / WOW64 分支 |
| 元数据检查 | x64 App 不直接 LoadLibrary x86 Adapter；采用静态元数据与目标侧复验，或同位数 inspector | 从双架构目标内工件结构推得，本文不宣称这两个项目恰好采用该检查方案 |
| 成功状态 | 分开“工件完整、部署成功、Runtime 就绪、Adapter 启用、文字恢复完成” | OBS helper 退出与捕获 ready 分离；Windhawk 停用分阶段 |
| 权限 | 与架构分开报告，明确拒绝原因 | helper 并不自动获得管理员 token |
| 首批范围 | GDI 先通过 x86 完整生命周期；其他框架逐项开放 | 两个项目只证明基础架构模式，不证明 Qt / MonoGame ABI 可以直接跨编译 |

不建议纳入第一阶段：Windhawk 的全系统常驻服务、全进程扫描和跨位数特殊代码；OBS 的游戏捕获、纹理传输或兼容注入逻辑。它们服务于不同的产品目标。Glyphshift 还必须额外验收译文更新、字体回退、停用恢复、重启恢复和目标退出；“hook 能装入”不足以证明汉化可用。

本次没有核实两个项目所有安装版本，也未做源码移植或许可证兼容评估。当前结论适用于架构选择，不构成复制代码的建议。
