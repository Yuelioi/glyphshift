# Glyphshift V2 架构

Status: Accepted

本文是 V2 的实际架构设计，不是阶段路线。现有 V1 代码只用于核对已验证行为；
本文中的模块、接口与目录均为 V2 新实现契约。

Hook/写回扩展边界的市场与平台依据见
[Hook 翻译工具与 Capability Adapter 调研](hook-translation-market-and-adapters.md)。

## 1. 架构结论

Glyphshift V2 由四个执行层组成：

```text
┌──────────────────────────────────────────────────────────────┐
│ Glyphshift Desktop Application                               │
│ Vue View + Tauri Command Adapter                             │
└──────────────────────────┬───────────────────────────────────┘
                           │ Rust Module Interface
┌──────────────────────────▼───────────────────────────────────┐
│ DesktopApplication composition root                          │
│ DesktopBackend · RuntimeBundle · DesktopRuntime              │
│ Workspace · Registry · Session Manager                       │
│ 不包含任何软件品牌、软件路径或软件专属分支                     │
└──────────────┬────────────────────────────┬──────────────────┘
               │ stdio JSON protocol        │ 目标进程 Runtime 加载
┌──────────────▼─────────────┐   ┌──────────▼──────────────────┐
│ Isolated Processes          │   │ Target Process              │
│  ├ Controller Plugin        │   │ Glyphshift Runtime Kernel   │
│  └ external Adapter Worker  │   │  ├ Adapter Host             │
│ 崩溃不影响 Service          │   │  ├ GDI/GDI+ Adapters       │
└────────────────────────────┘   │  └ Decision Engine          │
                                 └─────────────────────────────┘
```

硬性分工：

- Vue 不读取运行包、不发现进程、不接触 DLL 路径或 Controller token。
- Tauri handler 只做参数/错误映射；`DesktopApplication` 是桌面组合根。
- `DesktopBackend` 独立管理软件身份、Translation Workspace、revision CAS 与持久化。
- `RuntimeBundle` 验证 Controller/Runtime/Adapter artifact，动态读取 Adapter Descriptor；
  `DesktopRuntime` 隐藏发现、Recipe、目标令牌、部署、ACK 与 Session 生命周期。
- Controller Plugin 处理软件专属控制行为，但无权直接注入或宣告翻译成功。
- Capability Adapter 隐藏具体 Hook/组件/外部协议写回机制；第一方 Adapter 也走注册表。
- Decision Engine 是唯一通用 Route/Lookup/Decision 语义；可编译进 Runtime Kernel，也可
  由 Service 为外部 Adapter 调用。
- Runtime Kernel 只处理目标进程内的 Observation、Decision 和 Adapter 调用。
- Translation Catalog 只描述文字语义，不描述 GDI、GDI+ 或其他 Hook API。
- Font Policy 独立于翻译数据。

## 2. V2 领域模型

### Software Extension

一个可安装的软件扩展包，拥有软件识别信息、Translation Location、运行能力依赖、Route
Program、内置翻译，以及可选 Controller Plugin。

替代 V1 的 Host Profile、Driver 与 Host Bridge。普通界面称其为“软件”。

### Installation

Software Extension 发现的一份已安装软件。包含可供用户识别的版本、渠道与显示名称；真实
路径只在 Service 与插件进程之间流动。

### Target Instance

一个明确的运行中软件实例。Controller 内部持有进程标识与私有 token；普通 UI 只接收
短期 opaque ID 和可识别的窗口名称，不暴露 PID、令牌或路径。

### Capability Adapter

实现一种可验证 Observe/Writeback 能力的深 Module。Adapter 由动态 Registry 解析，不由
Core 按 ID 分支。V2 首批第一方 Adapter 为：

- `windows.gdi.ext-text-out`
- `windows.gdiplus.draw-string`

GDI+ Adapter 只安装一次 `GdipDrawString` Hook，但提供两个可独立启用的 Feature：

- Text Replace
- Font Substitute

这避免两套功能重复 Hook 同一入口。

每个 Adapter 声明：

- Feature：`text.observe`、`text.replace`、`font.substitute` 等稳定语义。
- Apply Model：`inline-render`、`retained-object`、`external-protocol` 或
  `observe-only`。
- Placement：`target-process` 或 `isolated-worker`。
- Adapter ABI、版本、架构、签名与 hash。

第一方 GDI/GDI+ Adapter 也必须使用同一 Descriptor、Registry、能力协商与状态模型，
不能拥有 Runtime Kernel 私有 `match adapter_id` 分支。

### Controller Plugin

运行在独立受控进程中的软件专属代码。它发现安装与 Target Instance、启动软件并生成
Session Recipe。它不进入目标进程，也不直接读写 Translation Snapshot。

### Route Program

Software Extension 拥有的、版本化且声明式的路由程序。它只组合 Decision Engine 已注册
的通用 operator，把 Observation 路由为 Location/Context；不包含 native code、WASM、
脚本、函数地址、文件或网络访问。未知 operator 在 Extension Registry reload 时拒绝。

V2 首版不提供目标进程内任意 Route Code Plugin。若新的写回或观察机制确实需要代码，它
必须作为 Capability Adapter 独立注册；若只是新的通用路由语义，则升级 Route Program
schema 与 Decision Engine operator，而不是新增软件 ID 分支。

### Session Recipe

Controller Plugin 为一个 Target Instance 生成的声明式运行配方。它只引用已注册
Adapter、Feature、Route Program 与 Font Policy，不允许包含任意
内存地址、Hook 脚本或 Core 内部类型。

### Text Observation

Capability Adapter 从绘制调用、组件对象或外部协议中安全产生的文字观测。它包含来源
Adapter、原文、Surface Token 和有限元数据，但不包含 Translation Location。

### Translation Location

软件扩展提供的、用户可以理解的文字位置，例如“菜单”或“效果参数”。它与 Hook Path
正交。一个 Location 可以由不同 Adapter 产生，一个 Adapter 也可以路由到多个 Location。

### Translation Context

区分同一 Location 内语义位置的结构化信息。例如效果参数 Location 可以使用
`{ kind: effect, key, label }`，而不是把 `effects/Color Range` 拼成路径。

### Translation Catalog

某个 Software Extension 与 locale 下的一组 Translation Entry。每条记录由 Location、
可选 Context、原文与译文组成；来源层信息由 Catalog Source 管理，不塞进条目 key。

### Font Policy

针对 Adapter、Feature 或 Route Scope 的字体替换规则。它不属于 Translation Entry，
即使没有任何译文也能产生 Font-only Decision。

### Translation Generation

Service 对一个已验证、可编译的 Translation Workspace 状态分配的单调递增版本。Desktop
与每个 Runtime Session 都报告自己当前看到的 Generation。

### Render Decision

Runtime 对一次 Text Observation 作出的最终决定：

```text
Pass
Text-only
Font-only
Text + Font
```

`inline-render` Adapter 无论得到哪种结果，都必须恰好调用一次原绘制函数。其他 Apply
Model 使用各自的恢复、对象生命周期或异步 ACK 不变量。

## 3. 进程与信任边界

### 3.1 Desktop

Vue 是薄客户端，只通过 Tauri command 访问桌面组合根：

```text
snapshot() -> DesktopSnapshot
discover(application) -> RuntimeView
start(target, features) -> RuntimeView
publish(workspace change) -> DesktopSnapshot
stop(application) -> RuntimeView
```

内部页面不直接调用插件命令或 Runtime 命令。UI 不知道 Adapter、Controller Plugin、
Route Program 内部 operator、文件路径或进程协议。

### 3.2 Desktop Application composition root

当前桌面版没有伪造一个尚未实现的长期后台服务。Tauri 进程持有 `DesktopApplication`；关闭
桌面时 `DesktopRuntime::drop` 先请求 PassThrough，再终止隔离 Controller。未来如果需要
常驻服务，只能替换这一组合边界，不能把进程协议泄漏给 Vue。

桌面组合根使用以下深 Module：

| Module | 小 Interface | 隐藏的 Implementation |
|---|---|---|
| Desktop Backend | `snapshot`、`runtime_spec`、`upsert_translation` | 软件身份、Workspace、CAS、原子文件 |
| Runtime Bundle | `open`、`discover` | 清单、artifact hash、动态 Descriptor、Controller 启动 |
| Adapter Registry | `reload`、`resolve` | Adapter Descriptor、签名、ABI、Placement、进程/DLL |
| Translation Workspace | `view`、`apply`、`rescan`、`subscribe` | 文件监视、CAS、分层、校验、编译、Generation |
| Decision Engine | `decide` | Route、Catalog lookup、Font Policy、Render Decision |
| Desktop Runtime | `targets`、`start`、`publish`、`stop` | Recipe、目标 token、注入、部署 JSON、ACK、Session |

这些 Module 的 Interface 同时是 contract test 的测试面。文件系统、stdio 协议、目标进程
加载和动态库都是内部 seam，通过生产 Transport 与内存/合成 Transport 验证。

Desktop composition root 只编排已验证的 Runtime Spec、Prepared Recipe、Adapter Registry
与 Session Manager。Generic Extension 和 Controller Extension 最终进入同一 Session
启动路径；未知软件/Adapter 的动态包不要求 Service 或 Runtime Kernel 增加 ID 分支。

### 3.3 Controller Plugin Process

Controller Plugin 采用独立进程，不采用 Rust 动态库 ABI：

- Transport：隔离子进程 stdin/stdout。
- Encoding：逐行、带 schema/request ID 的版本化 JSON envelope。
- Handshake：Protocol major/minor、Extension ID、版本、capability set、nonce。
- Lifecycle：Desktop Runtime 使用超时和隐藏窗口管理；崩溃或协议违规只降级该连接。
- Permission：默认当前用户权限；只收到可执行文件名、授权 Requirement 和必要请求。

Controller Plugin Interface：

```text
inventory() -> Installations + Target Instances
launch(InstallationId) -> Launch Receipt
prepare(TargetInstanceId, Requested Features) -> Session Recipe
```

Controller Connection 只有在 Extension ID、协议版本与一次性 nonce 全部匹配后可用。
Controller 私有 inventory token 被映射为 Runtime 分配的 opaque ID；Launch Receipt
不包含 Active 状态。prepare 先按 Extension 权限拒绝未知 Adapter、越权 Feature、地址、
Hook Code 与脚本，之后 Session Manager 仍通过 Adapter Registry 重新解析全部 Binding。
取消请求不产生 Recipe；timeout、崩溃或畸形消息只终止并降级该连接。

普通 `.exe` 使用通用 Windows Controller 的 executable matcher；只有安装发现、启动协议或
会话准备确实特殊时才增加独立 Controller Plugin。

### 3.4 Capability Adapter Hosts

Capability Adapter 的逻辑契约统一，但允许两个执行位置：

- `target-process`：由 Runtime Kernel 的 Adapter Host 通过版本化 C ABI 加载，适合
  GDI、GDI+、DirectWrite、Qt Painter 等进程内 Hook。
- `isolated-worker`：Session/Registry 已保留执行位置与 ACK 模型，适合 Chromium CDP 等
  外部协议写回；首版桌面尚未发布生产 Worker，也未承诺某个固定传输。

Controller Plugin 不能兼任持续翻译 Adapter。Controller 只产生 Target Facts 和 Session
Recipe；Adapter Host 才应用 Decision 并报告 Feature ACK。

Capability Adapter Interface 保持为：

```text
probe(Target Facts) -> Probe Evidence
activate(Session Binding, Host Port) -> Active Features
update(Generation Input) -> Feature ACK
deactivate() -> Pass/Restore Result
```

Descriptor 是签名包数据，由 Registry 读取，不要求先执行 Adapter。`probe` 只收集证据，
不能安装 Hook 或改变目标；具体 Hook、对象跟踪、协议重连和恢复全部隐藏在 Adapter
Implementation。Host Port 只提供 Decision、Generation ACK 与有界 Diagnostics，不暴露
Service、Catalog 路径或 Core 内部对象。

Decision 执行位置由 Placement 决定：

- `target-process` 的同步/组件 Adapter 由 Runtime Kernel 内嵌 Decision Engine，对本地
  不可变 Snapshot 执行有界查询，热路径不跨进程。
- `isolated-worker` 把带稳定 Observation ID 的 Observation 发给 Service；Service 使用
  同一 Decision Engine 返回带 Generation 的 Decision。Worker 不读取完整 Catalog。
- Generation 更新时，Service 对 Worker 已跟踪的 Observation 重新决策，或要求 Worker
  受控 rescan；只有新的可见结果应用完成后才能 ACK 该 Generation。

### 3.5 Target Runtime

Runtime Kernel 是注入目标进程的稳定小型 DLL。它负责：

- 与 Service 完成 nonce + protocol handshake。
- 加载并验证 Session Recipe。
- 解析已验证的 Adapter Binding，激活 target-process Adapter 与 Feature。
- 原子加载 Translation Snapshot 和 Font Policy。
- 使用共享 Decision Engine 对 Observation 执行 Route → Lookup → Render Decision。
- 发布计数、错误和已应用 Generation。
- fail-open；任何异常都保持原软件行为。

Runtime Kernel 不发现安装、不读取 Extension manifest、不管理翻译包、不知道软件品牌。
它只接受 Registry 已验证的 target-process Binding；isolated-worker 由 Service Host
管理。Translation Snapshot 与 Font Policy 按递增 Generation 一次性替换，旧 Generation
不能覆盖当前决策输入。

## 4. 真实代码插件

一个 V2 Software Extension 是单一发布包：

```text
extension/
  extension.json
  assets/
  routing/
    routes.json
  translations/
    <locale>.json
  controller/
    win-x64/
      controller.exe       # 可选，独立进程
```

### Controller Plugin

优先使用。适合软件专属安装发现、启动参数、目标识别、版本差异和 Session Recipe 生成。
因为在目标进程外运行，它可以被 Service 超时、终止和重启。

### Route Program

Route Program 保存在 `routing/routes.json`，由 Extension Registry 校验并由 Decision
Engine 编译。首版只允许 `direct`、`fallback` 与 `contextual-heading` 等注册 operator，
并对状态大小、metadata、字符串长度和每次 Observation 的执行步数设硬上限。

Route Program 是数据，不是代码插件。这样“不能安装 Hook、不能访问文件/网络、不能修改
Registry”是结构上可证明的事实，而不是对任意 native DLL 的约定。

Extension Registry 的 reload 是原子发布：dictionary-only 的代码夹带、manifest 原生
运行指令、未知/可执行/I/O Route、重复 Location、非法 Context 或超出协议上限的 Route
Program 都拒绝整个新 Revision 并保留 last-known-good。Controller 可执行身份只使用包内
Artifact ID、Signer、hash 与 Protocol Version，不向上层返回本机路径。

V2 首批 AE Extension 不需要自定义 Capability Adapter。AE 的 GDI/GDI+ 写回由通用
Adapter 完成；安装发现与启动进入 AE Controller Plugin；效果 heading context 由通用
`contextual-heading` Route Program 配置表达。如果该 operator 的输入事实不足，应先让
Adapter 产出更可靠的通用 metadata，再判断是否需要新增通用 operator。

Capability Adapter 是独立安装、独立签名和独立注册的发布包，不附着于某个软件品牌：

```text
capability-adapter/
  adapter.json
  runtime/
    win-x64/
      adapter.dll           # placement=target-process
  worker/
    win-x64/
      adapter.exe           # placement=isolated-worker
```

一个包只需要提供其 Placement 对应的实现。Software Extension 通过 Adapter ID 与版本
约束声明依赖；删除某个软件扩展不会删除共享 Adapter。

## 5. Extension manifest

V2 manifest 不再拥有 `runtime.driver`。示意结构：

```json
{
  "schema": "glyphshift.extension/1",
  "id": "vendor.product",
  "version": "1.0.0",
  "software": {
    "name": "Product",
    "vendor": "Vendor",
    "executables": ["Product.exe"]
  },
  "controller": {
    "kind": "generic"
  },
  "runtime": {
    "capabilities": [
      {
        "adapter": "windows.gdi.ext-text-out",
        "features": ["text.replace"]
      },
      {
        "adapter": "windows.gdiplus.draw-string",
        "features": ["text.replace", "font.substitute"]
      }
    ],
    "routes": "routing/routes.json"
  },
  "translation": {
    "locations": [
      {
        "id": "menu",
        "label": "菜单",
        "context": null
      },
      {
        "id": "effect-parameter",
        "label": "效果参数",
        "context": {
          "kind": "effect",
          "label": "效果"
        }
      }
    ]
  }
}
```

规则：

- Adapter ID 只描述运行机制，不充当 Translation Location。
- Location ID 只描述软件内语义，不选择 Hook。
- Route Program 连接两者。
- manifest 只引用已安装、已签名 Adapter；Route Program 只使用当前 schema 已注册的
  operator。
- Dictionary-only 包不能声明 Controller Plugin 或 Capability Adapter。
- manifest 不包含函数地址、Hook Code、Frida 脚本或 Runtime Kernel 内部类型。

Adapter 自己的 `adapter.json` 描述能力与执行约束：

```json
{
  "schema": "glyphshift.capability-adapter/1",
  "id": "windows.gdiplus.draw-string",
  "version": "1.0.0",
  "abi": "glyphshift.runtime-adapter/1",
  "placement": "target-process",
  "apply_model": "inline-render",
  "features": ["text.observe", "text.replace", "font.substitute"],
  "architectures": ["win-x64"]
}
```

Core 理解稳定 Feature 与 Apply Model 语义，但不穷举 Adapter ID。Registry 发现 Descriptor，
验证签名、hash、ABI、架构和 Placement，再向 Session Manager 返回可用 Binding。

首版版本约束只提供显式 `Exact(major.minor.patch)`：Descriptor、Requirement 与 Binding
都必须携带版本，不允许缺省选择“最新版本”。Registry 以 Adapter ID 和 Version 二级索引
包；同 ID 多版本可以共存，同 ID/版本但内容 hash 不同会让整个 reload 原子失败并保留旧
Revision。未来若需要兼容范围，必须增加新的显式 Requirement 变体和确定性选择契约。

Package Source 为已验证 Artifact 分配不透明的包内 ID；Registry 根据 Placement 把它转换为
`TargetProcess { library }` 或 `IsolatedWorker { executable }` Host Binding。Binding
不携带本机路径，也不携带 Translation Catalog 的位置或存储细节。

## 6. Runtime 管线

```text
inline-render Adapter 原始调用
  → bounded decode
  → Text Observation
  → Route Program
  → Lookup Key(location, context, source)
  → Translation Snapshot lookup
  → Text Decision
  → Font Policy lookup
  → Font Decision
  → Render Decision
  → Adapter 恰好一次调用原函数
```

核心类型示意：

```text
TextObservation {
  adapter_id
  source_text
  surface_token
  metadata
}

LookupKey {
  location_id
  context?
  source_text
}

TextDecision = Keep | Replace(text)
FontDecision = Keep | Substitute(font)

RenderDecision {
  text: TextDecision
  font: FontDecision
}
```

### GDI Adapter

`windows.gdi.ext-text-out` 深 Module 独占 `ExtTextOutW` Hook，隐藏 glyph-index 解码、边界
检查、重入保护、原函数调用和诊断。外部只看到 Capability Adapter Interface。

### GDI+ Adapter

`windows.gdiplus.draw-string` 深 Module 独占 `GdipDrawString` Hook。它在一次调用中同时
请求 Text Decision 与 Font Decision：

- 两者 Keep：原参数调用。
- Text Replace：替换字符串，保留原字体。
- Font Substitute：保留原字符串，替换字体。
- 两者 Replace：替换字符串和字体。

文字与字体是独立 Feature，但共享一个 Adapter Implementation 和一次原函数调用。

### Observe-only Adapter

当前 UIA、CDP Capture 或其他 Observe-only Adapter 只能产出 Observation/Event，不能
返回 Render Decision。
Service 与 UI 只在至少一个 Replace/Font Feature 已安装、激活且 Runtime ACK 后显示
“正在翻译”或“正在替换字体”。

### 非 inline-render Adapter

并非所有未来写回都发生在原绘制函数内：

- `retained-object`：修改 Unity Text Component、Qt Translator 等长生命周期对象；必须
  跟踪对象失效，并能恢复原值。
- `external-protocol`：通过 Chromium CDP 等受授权协议修改目标界面；必须处理重渲染、
  对象 ID 失效和异步 ACK。
- `observe-only`：只报告文字或图像事件，永远不能提升 Replace/Font Feature 状态。

它们复用 Observation → Route → Lookup → Decision 语义，但不伪装成原函数调用。
“恰好一次调用原函数”只约束 `inline-render`；其他 Apply Model 由 Adapter Descriptor
选择对应的 contract test。

## 7. Route Program

Route Program 是 Observation 与 Translation Location 之间的数据型 seam。首批注册
operator：

- `direct`：某 Adapter 的全部 Observation 进入一个 Location。
- `fallback`：按顺序尝试多个 Location。
- `contextual-heading`：可信 heading 更新当前 Surface Token 的 Context，后续 Observation
  进入带 Context 的 Location。

Program 配置由 Extension 拥有，编译器与执行器由 Decision Engine 提供。输入输出是通用
类型，Implementation 中不出现软件名称。

Program 不能调用 Adapter、Service、文件系统、网络或任意宿主函数，也不能绕过 Snapshot
与 Render Decision 管线。新增 operator 必须具备跨软件语义并进入协议版本与通用 contract
test；不能因为一个软件而在 Decision Engine 中增加品牌分支。

## 8. Translation Catalog

V2 不沿用旧 Dictionary `schema: 2` 的动态 Domain 路径。新的 Catalog 使用记录结构：

```json
{
  "schema": "glyphshift.translation/1",
  "extension_id": "vendor.product",
  "locale": "zh-CN",
  "entries": [
    {
      "location": "menu",
      "source": "File",
      "translation": "文件"
    },
    {
      "location": "effect-parameter",
      "context": {
        "kind": "effect",
        "key": "color-range",
        "label": "Color Range"
      },
      "source": "Fuzziness",
      "translation": "模糊度"
    }
  ]
}
```

唯一键：

```text
extension_id + locale + location + canonical_context + source
```

Catalog 不存储：

- Hook API 或 Adapter ID。
- Font family。
- 本机软件路径。
- PID。
- package layer 优先级。

Location 定义来自 Extension manifest。Catalog Source 决定来源层：

```text
builtin < installed package(priority, package id) < user
```

Translation Workspace 隐藏所有层合并、冲突、文件监视和快照编译。UI 只拿到用户可理解的
Location label、Context label、来源和冲突结果。

## 9. 字典编辑与 Runtime 热更新

磁盘文件、Desktop 与 Runtime 只能通过 Translation Workspace 进入新状态：

```text
UI Edit 或 File Watcher
  → parse
  → schema/location/context validation
  → compare-and-swap source revision
  → merge all Catalog Sources
  → compile immutable Snapshot
  → assign Generation N+1
  → atomically publish
  ├─ Event → Desktop reload View
  └─ Session Update → Runtime
                       → validate digest
                       → atomic swap
                       → ACK Generation N+1
```

UI 编辑提交携带 `base_revision`。如果文件已被外部修改，Service 返回 Conflict，不覆盖
新内容。用户可以重新加载、比较后再提交。

File Watcher 与 UI Save 调用同一 `apply` Implementation，不存在第二套刷新逻辑。

坏更新：

- 不产生新 Generation。
- 保留最后一个有效 Snapshot。
- Desktop 收到 `TranslationRejected` Event，显示文件和可恢复原因。
- 已连接 Runtime 继续使用上一 Generation。

Desktop 显示两个事实：

- Workspace Generation。
- 每个 Runtime Session 已应用 Generation。

两者不一致时显示“更新尚未应用”，不能继续显示“会自动刷新”。

## 10. Session 生命周期

### 启动

```text
Desktop execute(StartSession)
  → Service resolves Extension
  → Extension Registry starts/uses Controller Plugin
  → plugin inventory/prepare
  → Service validates Session Recipe
  → Adapter Registry resolves signed Adapter Bindings
  → Injector loads Runtime Kernel and/or Service starts isolated Adapter Worker
  → Adapter Host handshake
  → activate Adapters + Features + Routes + Font Policy
  → publish current Snapshot
  → each Adapter ACKs Active Features + Generation
  → Session becomes Active
```

只有最后一个 ACK 完成后，Desktop 才显示文字翻译或字体替换已启用。

Session Manager 以 `AdapterId + AdapterVersion + Feature` 作为 ACK 与状态键，不能只按
Feature 名称合并；同一种 `text.replace` 可由多个 Adapter 独立处于 Starting、Active 或
Failed。Controller Recipe Port、Adapter Registry 与 Adapter Host Port 是内部组合 seam，
对外 `start` 仍只接收显式 Target Instance 与 Requested Features。

Target Lifecycle Port 独立报告每个不透明 Target Instance 的存活状态。实例退出时只释放
并删除对应 Session；Controller、其他 Target Instance 及其 Feature、Generation、诊断
状态不受影响。

Host Activation 分别报告 acknowledged 与 failed Bound Feature；未报告项保持 Starting，
同一键同时出现在两个集合属于协议矛盾并拒绝 Session。Observe-only Adapter 的活动只更新
自己的 `text.observe` 键，不能提升任何 Text Replace 或 Font Substitute 状态。

Adapter ID 或精确版本不存在时，Session Manager 把该 Requirement 的 Bound Feature
返回为 Unavailable；这类启动拒绝不调用 Host、不写入 Session 表也不消耗 Session ID。
完整性、信任、ABI 与架构拒绝保留原始 Registry 错误，不能伪装成“未安装”。

### 更新

对于 target-process Adapter，Service 发送不可变 Snapshot 引用、Generation 与 digest，
Runtime Kernel 完整验证后原子替换；不能增量修改正在被 Hook 回调读取的 map。

Target Runtime 只返回一个原子 Generation ACK，Session Manager 将它应用到该 Runtime
的全部 Bound Feature。ACK 旧值时记录 desired、acknowledged 与上一个 applied Generation
并显示 mismatch；只有 ACK 等于本次 desired Generation 才能推进 applied。

对于 isolated-worker Adapter，Snapshot 留在 Service Decision Engine。Service 用新
Generation 重新决策 Worker 已跟踪对象或请求受控 rescan，Worker 应用带 Generation 的
Decision 后 ACK；不能仅因 Service 已加载文件就宣告目标窗口已更新。

Worker Generation ACK 按 Bound Feature 报告，只推进实际完成应用的键；没有 ACK 的键
保持 Updating，并保留上一次 applied Generation。Worker 收到 Decision 或 Service 完成
重新决策都不能代替应用 ACK。

### 停止

V2 不远程卸载已注入 Runtime DLL。停止 Session 只把 target-process Adapter 切到 Pass、
清理控制通道并更新状态；完全卸载仍由退出 Target Instance 完成。Retained-object 或
external-protocol Adapter 按 Descriptor 契约恢复原值或停止维护写回。

## 11. 状态模型

每个 Feature 独立报告：

```text
Unavailable
Ready
Starting
Active
Degraded
Failed
```

Session 汇总不能覆盖 Feature 事实。例如：

- GDI Text Replace Active
- GDI+ Text Replace Failed
- GDI+ Font Substitute Active

此时 UI 可以说“菜单翻译和字体替换已启用，面板文字翻译失败”，不能只显示一个绿色
“已连接”。

Capture/Observe 的事件数量不允许提升任何 Replace Feature 的状态。

## 12. 仓库目录与 Module

Glyphshift 在一个独立 workspace 中实现，不依赖历史仓库：

```text
Cargo.toml
crates/
    glyphshift-domain/
    glyphshift-protocol/
    glyphshift-extension/
    glyphshift-translation/
    glyphshift-decision/
    glyphshift-session/
    glyphshift-controller-sdk/
    glyphshift-controller-host/
    glyphshift-controller-windows/
    glyphshift-adapter-sdk/
    glyphshift-adapter-registry/
    glyphshift-runtime-kernel/
    glyphshift-adapter-gdi/
    glyphshift-adapter-gdiplus/
    glyphshift-target-runtime/
    glyphshift-target-process-host/
    glyphshift-desktop-backend/
    glyphshift-desktop-runtime/
apps/
    glyphshift-service/
    glyphshift-desktop/
extensions/
    reference-generic/
    reference-controller/
test-support/
    adapters/
      reference-inline/
      reference-retained/
      reference-worker/
      reference-observe/
```

Module 依赖方向：

```text
domain
  ↑
protocol     extension     translation
  ↑              \          /
plugin-sdk        decision
                    ↑  ↑
                session  runtime-kernel ← adapter-sdk
                  ↑  ↑                    ↑
            service  adapter-registry   capability adapters
```

禁止：

- `domain` 依赖 Windows、Tauri、文件系统或插件。
- `translation` 依赖 Capability Adapter。
- `decision` 依赖 Windows、进程、IPC 或具体 Adapter ID。
- Route Program 包含 native/WASM/脚本、I/O 或未注册 operator。
- `runtime-kernel` 依赖软件 Extension ID。
- Vue/Tauri handler 依赖 Controller 协议、目标令牌或 artifact 路径。
- target-process Adapter 调用 Service、文件系统或网络。
- Core 按 Adapter ID、软件品牌或引擎名称分支。

## 13. 安全与失败语义

- Extension manifest 与 Translation Catalog 始终先验证再发布。
- Controller Plugin 进程崩溃只让对应 Extension Degraded。
- target-process Adapter 属于目标进程内代码，只允许受信任签名、hash pin、兼容 ABI
  和用户授权。
- Route Program 是无代码、无 I/O、执行有界的验证数据；不能加载 native/WASM/脚本。
- Isolated Adapter Worker 使用受控进程、Job Object、超时与最小数据权限。
- Session Recipe 必须由 Service 依据 Extension 权限与已注册 Adapter 再验证。
- Session Recipe 显式声明 Controller 丢失后的 Continue/Degrade 策略；Controller
  崩溃不删除既有 Adapter Session，也不终止用户 Target Instance。
- Adapter Host 以 Adapter ID 与精确版本报告健康状态；单个 Adapter 故障只降级它拥有的
  Bound Feature，不覆盖同一 Session 中其他 Adapter 的事实。
- Adapter Registry 的共享 Revision 发布对新旧 Session 同时可见；删除或撤销 Adapter
  后拒绝新 Binding，既有 Binding 对应 Feature 明确 Degraded。
- Session stop 只向 Adapter Host 发送 PassThrough、RestoreOriginal、StopWriteback 或
  StopObserving；协议不提供远程卸载 target-process 代码的动作。
- Standard Extension 不能携带地址、脚本或未注册 Hook。
- Adapter 的 panic、非法 UTF、重入、对象失效、Snapshot 错误和 IPC 中断全部
  fail-open。
- 诊断队列有界且非阻塞；队列满丢诊断，不阻塞宿主绘制线程。
- Service 不终止用户 Target Instance，也不把注入成功当成可见翻译成功。

## 14. 验证架构

所有测试从 Module Interface 进入，不测试内部函数形状。
逐项 ID、Fixture、可见断言与执行门槛见
[Foundation Contract Tests](foundation-contract-tests.md)。

### Contract tests

- Extension Registry：dictionary-only、通用软件包、Controller Plugin 软件包与 Route
  Program 校验。
- Adapter Registry：Descriptor、签名、ABI、Placement、未知 Adapter、版本冲突与撤销。
- Controller protocol：版本协商、超时、崩溃、恶意 Recipe、取消。
- Translation Workspace：外部修改、CAS 冲突、坏文件、层优先级、Generation。
- Session Manager：Adapter 部分失败、Feature ACK、Generation mismatch、进程退出。

### Runtime test host

合成 Windows Host 明确绘制：

- GDI Unicode。
- GDI glyph-index。
- GDI+ 原字符串/原字体。
- 多 Surface contextual heading。

测试断言窗口实际像素或可访问文字变化，并分别覆盖 Pass、Text-only、Font-only、
Text+Font。

每种 Apply Model 还有一个 reference Adapter：

- `inline-render`：断言原函数恰好一次、重入与 fail-open。
- `retained-object`：断言对象销毁、重建、热更新与恢复原值。
- `external-protocol`：断言异步 ACK、对象失效、重连与未授权失败。
- `observe-only`：断言任何事件数量都不能提升 Replace/Font 状态。

### Architecture checks

- 扫描 V2 Core/Desktop，不允许软件品牌、Extension ID、Driver 或专属可执行文件名。
- 删除任意 Software Extension 后，V2 workspace 仍能构建并启动。
- 新增 reference Extension 只新增包，不修改 Core/Desktop。
- 新增第三个未知 Adapter 只新增 Adapter 包与测试，不修改 Core/Desktop/Session 源码。
- 扫描 Runtime Kernel，不允许按 Adapter ID、引擎名或软件名 `match`。
- Desktop Playwright 验证仓库内 Vue 表面与两种桌面视口；开发任务另行证明实际 Tauri
  窗口、Runtime bundle 和目标进程 E2E，浏览器页面不能单独替代桌面验收。

## 15. 历史实现边界

历史实现从未发布。当前产品不读取旧 manifest、Dictionary schema、`.gspack`、设置或 Driver；
迁移入口已经删除。旧数据只能通过明确的一次性导入 seam 进入产品，不能成为 workspace 依赖
或 Runtime 输入。

## 16. 已确认的架构决定

本设计已经选择以下方案，若无反对就作为 V2 契约：

1. Vue/Tauri adapter、DesktopBackend 与 DesktopRuntime 是三个独立边界；复杂运行编排不进入
   handler 或 Vue。
2. Controller Plugin 使用隔离进程 + 版本化 stdio JSON 协议，不使用 Rust DLL ABI。
3. Route Program 是由 Decision Engine 有界执行的版本化声明数据；V2 首版不存在可携带
   native/WASM/脚本的 Route Code Plugin。
4. GDI+ Text Replace 与 Font Substitute 是独立 Feature，但由一个深 Adapter
   Implementation 和一次 Hook 承载。
5. Translation Catalog 使用 Location/Context 记录，不再使用动态 Domain 路径。
6. 所有 UI 编辑、外部文件变化与 Runtime 更新都通过 Translation Workspace 和
   Generation/ACK 管线。
7. 当前仓库是唯一产品 workspace，不读取或兼容历史实现数据。
8. 所有 Hook/组件/协议写回实现都是 Capability Adapter；由动态 Registry 解析
   Descriptor、Feature、Apply Model 与 Placement，Core 不按 Adapter ID 写死。
