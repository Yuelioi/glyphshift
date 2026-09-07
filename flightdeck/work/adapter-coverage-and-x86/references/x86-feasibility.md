# Windows x86 目标软件支持研究

这是初轮可行性分析。关于是否需要独立 helper，以及复用现有 Controller 的选择，后续以[成熟项目对比与候选方案](architecture-comparison.md)为准；同位数执行方案不应被表述成 Windows 强制要求。

研究日期：2026-09-07。范围：保持 x64 桌面 App，支持 x64 Windows 上的 x86 目标软件。本文是源码与官方文档研究，未编译 x86、未注入、未运行真实软件；不将“能编译”视为验收通过。UIA/OCR 归档不进入方案。

## 结论

可行。第一步支持 GDI 文字属于**中等规模工程**，需要贯通构建、部署、位数路由、检查器和完整会话生命周期；把现有框架全部迁到 x86 属于**较大且分框架的不确定工作**。建议保持 x64 App / 主 Controller，新增同位数的 x86 Helper、Target Runtime 和首批 GDI Adapter。Helper 通过序列化协议沟通，隔离目标内 ABI。现有 Controller 已通过标准输入输出提供协议服务，可复用基础设施（`crates/runtime/controller/windows/src/main.rs:1`）。

Windows 明确不允许 x64 进程加载 x86 DLL，反向也一样；允许跨位数的进程间通信。因此“App 也必须改成 32 位”不是必要条件，但“把现有 DLL 直接给 32 位软件”不可行。[Microsoft Process Interoperability](https://learn.microsoft.com/en-us/windows/win32/winprog64/process-interoperability)

## 已有基础与实际阻塞

| 位置 | 源码事实 | x86 所需工作 |
| --- | --- | --- |
| `crates/runtime/controller/windows/src/platform.rs:233` | 已用 `IsWow64Process2` 区分 x86/x64；查询失败退回 Controller 自身架构 | 查询失败应返回未知并拒绝不安全部署，不能把未知目标当 x64。区分目标位数与系统位数 |
| `crates/runtime/controller/windows/src/remote.rs:429` | `remote_system_export` 计算本地 kernel32 导出偏移，再加目标同名模块基址 | x64 本地导出偏移不能作为 x86 DLL 导出偏移。推荐同位数 Helper；仍须核对转发导出及模块身份，不能默认同名即同映像 |
| `crates/runtime/controller/windows/src/remote.rs:403` | `remote_export` 用 `LoadLibraryExW` / `GetProcAddress` 本地解析目标 Runtime DLL | 现有实现依赖本地可加载的同位数映像。异位数检查与解析交给对应 Helper，或另做 PE 导出解析器，后者不是首选最小方案 |
| `crates/runtime/targets/contract/src/lib.rs:40`、`crates/runtime/controller/windows/src/remote.rs:260` | Command / Query 是含原始指针的 `repr(C)`，直接把当前进程结构字节写入目标 | x64 与 x86 指针宽度及结构布局不同。原样跨位数传递必错；同位数 Helper 可保持目标内现有 ABI，跨进程只传序列化数据 |
| `crates/runtime/controller/windows/src/remote.rs:457` | 模块枚举已有 `TH32CS_SNAPMODULE32` | 具备 x64 枚举 x86 模块的基础，但这不解决函数地址和 ABI；还应对 `ERROR_BAD_LENGTH` 做有限重试并保留失败原因 |
| `crates/runtime/desktop/src/bundle.rs:9`、`:296`、`:306` | Manifest 只有一个 Controller / Runtime；App 在加载 Bundle 时直接加载每个 Adapter DLL 读取描述符与源文本策略 | 增加架构分组和同位数 inspector，或使用受校验的静态元数据并由同位数加载器复验。只改注入器仍会在 App 读取 x86 包时失败 |
| `crates/runtime/targets/process-host/src/lib.rs:56` | Catalog 持有一个 Runtime，Adapter 按 PackageArtifactId 索引 | 根据目标架构选择 Runtime、Adapter 工件与连接；同一逻辑 Adapter 的 x86/x64 实现不可重复碰撞，必要时区分工件 ID |
| `crates/adapters/platform/native-abi/src/lib.rs:15`、`:203`、`:246` | 已有 `ARCH_X86`，同时 ABI 带指针、函数指针以及结构返回值 | 架构标志仅是能力声明，不表示已有 x86 工件。Runtime 与 Adapter 必须同位数编译，验证结构大小/对齐、C 调用约定和导出表 |
| `scripts/build-runtime-bundle.ps1:40`、`:95`、`:180` | 一次 Cargo build，未显式传目标 triple；按 profile 目录取一套 DLL；Manifest schema 3 为单套运行时 | 显式构建 x64 / i686 白名单，按 triple 取产物，分架构记录哈希和文件；双架构 Bundle 用生产加载器验收 |
| `scripts/review-app.ps1:81` | review 脚本统一构建 Bundle、App 并验证后启动 | 保留唯一 review 入口，在内部增加多架构构建/验证，不另造独立 shell+旧 Runtime 的流程 |

`targets/runtime` 本身是独立 `cdylib`，没有把具体框架 Adapter 全部静态链接进去，适合复用为 x86 宿主；远程入口使用 `extern "system"`（`crates/runtime/targets/runtime/src/lib.rs:700`），需与 x86 线程入口及导出名称实物核对。

## 官方约束核对

- `IsWow64Process2` 返回目标进程机器类型及宿主原生机器类型；目标不是 WOW64 时 `processMachine` 为 UNKNOWN，应结合 `nativeMachine` 解释，不能把 UNKNOWN 直接等同 x86。最低客户端是 Windows 10 1709。[Microsoft 文档](https://learn.microsoft.com/en-us/windows/win32/api/wow64apiset/nf-wow64apiset-iswow64process2)
- x64 调用方可用 `TH32CS_SNAPMODULE32` 枚举 x86 模块；x86 调用方枚举 x64 目标会遇到 `ERROR_PARTIAL_COPY`。因此 x86 Helper 负责 x86 目标，主清单继续由 x64 Controller 负责。[Microsoft 文档](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot)
- `CreateRemoteThread` 要求起始函数地址存在于远程进程，线程创建成功也不保证起始地址有效。API 不会替项目转换导出地址、指针布局或调用约定；本文不据此宣称“所有跨位数 CreateRemoteThread 都不可行”，选择 Helper 是降低工程风险的方案。[Microsoft 文档](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createremotethread)
- Rust 官方将 `i686-pc-windows-msvc` 列为 Tier 1 with host tools，支持标准库及 Windows 主机上的 MSVC 跨架构构建。需要安装目标标准库和相应 MSVC 组件。此支持不意味着本项目的依赖与 FFI 已通过 x86 验证。[Rust 官方平台文档](https://doc.rust-lang.org/rustc/platform-support/windows-msvc.html)
- Windows x86 的 Rust `extern "system"` 对普通函数是 `stdcall`，`extern "C"` 是 C ABI；x86 MSVC 非静态 C++ 成员一般使用 `thiscall`，通过 ECX 传 `this`。x64 的统一调用方式掩盖的差异，在 x86 上必须显式处理。[Rust ABI 文档](https://doc.rust-lang.org/reference/items/external-blocks.html#abi)、[Microsoft thiscall](https://learn.microsoft.com/en-us/cpp/cpp/thiscall?view=msvc-170)、[Microsoft stdcall](https://learn.microsoft.com/en-us/cpp/cpp/stdcall?view=msvc-170)

## 最小交付顺序

1. **基础设施与拒绝边界（中）**：x86 Helper / Runtime、目标位数复核、按位数工件路由、异位数包检查、协议握手、错误信息。错误架构、未知架构、Helper 缺失时在注入前明确拒绝；继续保证 x64 可用。
2. **GDI 首批（中，最值得先做）**：先 TextOutW，再 ExtTextOutW、DrawTextW / DrawTextExW。这些 Win32 入口现有签名已使用 `extern "system"`，且 GDI 描述符已经包含 x86；例如 `crates/adapters/implementations/native/gdi-text-out-native/src/lib.rs:19`、`:95`。仍须验证 retour 的 x86 trampoline、字体替换、重绘、更新与停用恢复，不能因声明包含 x86 就公开全通过。
3. **GDI+ / DirectWrite（中）**：复核 Win32 / COM FFI 与 vtable，按已发布产品入口逐项启用。每项独立验收，不用它们阻塞 GDI 首发。
4. **Qt / 托管框架（大，按需求做）**：Qt Painter 当前明确 MSVC x64，QString6 长度等签名写成 i64，且使用 x64 C++ 修饰名（`crates/adapters/implementations/framework/qt-painter-native/src/lib.rs:1`、`:46`）。x86 要做 `thiscall`、对象布局、隐藏返回参数和导出名映射；Qt Quick 目前更窄，固定 Qt 6.8.3 MSVC x64（`crates/adapters/implementations/framework/qt-quick-native/src/qt.rs:1`）。MonoGame 与 Unity 描述符目前也限 x64，应分别核对运行时/本机桥及 ABI，不随 GDI 自动放开（`crates/adapters/implementations/framework/monogame/src/lib.rs:24`、`crates/adapters/implementations/framework/unity-mono-standard-ui/src/lib.rs:26`）。

第一阶段的验收应是：x64 App 从双架构生产 Bundle 识别并启动 x86 链路，在确定性 x86 GDI harness 中通过采集、替换、字体回退、更新、停用恢复、重复启停、目标退出及 Helper 故障；再由用户指定的真实 x86 软件验收。合成测试保留在仓库，本机原始证据仅放 local-test，review 继续走 `scripts/review-app.ps1`。

这些是工作量分级，不是工期承诺。尚未做 x86 编译，第三方依赖与真实软件行为仍会影响范围；现阶段已有充分依据启动 GDI 方向，无需先完成全部框架迁移。
