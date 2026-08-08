# Windows UI Automation observe-only Adapter seam 调研

Delivery update: 本文记录的是立项时的门槛；真实 MTA Client、权限、阻塞恢复与 Bundle 集成现均已完成，
当前交付状态以 [UIA Runtime Bundle 集成](../slices/uia-runtime-bundle-integration.md)为准。

## 结论

Windows UI Automation（UIA）适合补充 GlyphShift 的**结构化文字观察能力**，但它不是绘制 Hook，
也不能保证拿到屏幕上每一个实际绘制的字符串。UIA 暴露的是控件 Provider 提供的可访问性树、属性、
Control Pattern 和事件；可见文字可能来自 `Name`、`TextPattern` 或 `ValuePattern`，也可能根本没有被
Provider 暴露。Microsoft 将 UIA 定义为 Client 与 Provider 之间的跨进程可访问性基础设施，并通过
UIA Core 和兼容代理连接原生 UIA、MSAA 与其他框架 Provider。
[Architecture and interoperability](https://learn.microsoft.com/en-us/windows/win32/winauto/architecture-and-interoperability)、
[UI Automation and Microsoft Active Accessibility](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-msaa)

对 GlyphShift 的明确结论是：

- UIA Adapter 应声明为 `TextObserve + ObserveOnly + Placement::IsolatedWorker`，**不能为了复用现有
  Target Runtime 而伪装成 `TargetProcess`**。它不需要向目标进程注入 DLL，且 UIA 的 COM、事件、
  Provider 阻塞和权限边界都属于 Controller 侧独立进程职责。
- 生产形态需要一个独立的 MTA UIA worker。注册、移除事件和 COM 生命周期由 worker 内同一条控制线程
  串行管理；事件回调只做有界入队，不在回调里同步访问目标 Provider、写文件或等待 Runtime。
- 生产化前必须先有中央单写入 Observation/Capture Aggregator。当前每个目标 Runtime 各自持有
  `FileCaptureSink`；若 UIA worker 或多个 Process Family 成员直接写同一 checkpoint，会产生覆盖、
  revision 竞争和重复数据。UIA worker 只能经 IPC 发出有界 Observation，由中央聚合器负责排序、去重
  和一次性写入。
- 调研当时仓库虽已有 `Placement::IsolatedWorker`、`ObserveOnly` 和 Registry 绑定能力，但没有生产可用的
  isolated worker process、IPC host 与中央聚合数据面；Target Runtime 也只加载目标进程 native DLL。
  因此本轮只确定技术合同与合成验证计划，**不把 UIA Adapter 加入正式 Runtime Bundle**。
- UIA 只能作为 GDI、DirectWrite、Console 等绘制/输出 seam 的补充观察源，不能被描述成
  “通用翻译 Hook”或实际渲染覆盖证明；首版不声明 `TextReplace`、`FontSubstitute` 或 `LayoutAdjust`。

## 1. Placement 与进程模型

### 1.1 为什么是 `IsolatedWorker`

UIA Client 通过 UIA Core 查询另一个进程中的 Provider。调用边界本身已经是进程外的；将 Client 代码
注入目标进程不会让 Provider 更完整，反而把 COM 初始化、事件 handler 和异常/卡顿带进被观察软件。
Microsoft 的架构说明明确区分 Client、UIA Core、Provider 与跨框架代理，Client 不需要知道 Provider
是否位于另一个进程或使用何种 UI 框架。
[Architecture and interoperability](https://learn.microsoft.com/en-us/windows/win32/winauto/architecture-and-interoperability)

独立 worker 的职责边界应为：

```text
Desktop Controller
  ├─ 解析当前授权的目标实例 / Process Family
  ├─ 启动、监督、重启 UIA Worker
  └─ 中央 Observation/Capture Aggregator（单写入者）
           ▲
           │ bounded IPC observations / health
           │
UIA Worker（独立进程，MTA）
  ├─ UIA Client 与 CacheRequest
  ├─ 目标窗口根解析与事件生命周期
  ├─ Name / TextPattern / ValuePattern 读取
  └─ 会话内去重与 Provider 超时健康信号
           │
           ▼
UIA Core / Provider（目标进程或代理）
```

worker 不拥有字典、Workflow、checkpoint 或最终 revision。它只认识 Controller 下发的短期 target token、
允许的进程实例和窗口根，不把 PID、HWND、绝对路径或窗口标题升级为持久领域身份。

### 1.2 线程与 COM 合同

Microsoft 要求 UIA Client 若要与桌面上所有元素（包括自己的 UI）交互，应从不拥有窗口的独立线程调用
UIA；该线程应以 MTA 初始化 COM。文档同时提醒 UIA event handler 可能在与注册线程不同的线程上被
调用，因此 handler 必须线程安全。
[UI Automation Threading Issues](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading)

GlyphShift worker 采用以下收窄合同：

1. worker 启动一条不创建窗口的 MTA 控制线程，在该线程创建 UIA Client、解析根元素、安装/移除 handler。
2. 所有 Add/Remove handler 操作由该控制线程串行执行；停用时先停止接纳新任务，再移除全部 handler，
   释放缓存的 element/pattern 引用，最后退出 COM apartment。
3. event handler 允许在 UIA 选择的线程进入，只复制 event kind、sender 的短期 identity 和已随事件/缓存
   返回的有界属性，然后投递到无阻塞有界队列。队列满时丢弃并记聚合计数，不能反压 Provider。
4. handler 内不调用翻译引擎、不写 checkpoint、不持有跨 IPC 锁、不等待 Controller，也不递归读取整棵
   UIA tree。需要进一步读取 Pattern 的工作回到 MTA 控制线程执行。
5. `deactivate` 必须幂等；handler 移除与 in-flight callback 交错时，generation token 让旧 callback
   只丢弃而不发布。worker 崩溃或被 watchdog 回收时，Controller 将该观察源标记为 degraded，而目标
   软件继续原样运行。

这不是为了把 UIA 调用变成目标进程热路径，而是隔离一个可能执行第三方 Provider 代码的异步观察面。

## 2. 从哪里取得文字

同一个 UI 元素可能同时暴露多种来源。Adapter 必须保存 `source_kind` 作为短期证据，并使用确定性的
优先级；不能把不同来源同时无条件发布成三条字典候选。

### 2.1 `Name`

`Name` 是 UIA 元素的可访问名称，常由控件自身文本、关联 label 或 Provider 逻辑计算。它描述元素，
不等同于任意绘制字符串；兼容层还可能把 MSAA 的名称映射为 UIA Name。
[UI Automation and Microsoft Active Accessibility](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-msaa)

对标准 `Text` control，Microsoft 要求 `Name` 反映文本内容；`TextPattern` 则用于更高效或更丰富的文本
访问。但这只是该 control type 的 Provider 合同，不能外推到所有自绘控件。
[UI Automation Support for the Text Control Type](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-supporttextcontroltype)

首版读取规则：

- 非空、长度受限且未标记为密码/受保护内容时，`Name` 可形成 observation。
- `Name` 与子 `Text` control 的内容相同，或与随后取得的 Pattern 文本相同，只保留优先级更高且证据
  更明确的一条。
- `Name` 可能是控件用途而不是值，例如“用户名”；不能用它替代输入框当前值。
- 不从 `LocalizedControlType`、HelpText、窗口标题等属性拼接“可能可见”的句子。

### 2.2 `TextPattern`

`TextPattern` / `TextPattern2` 面向文档或包含连续文本的控件，Client 通过 document range 和
`IUIAutomationTextRange` 取得内容、选择、可见范围与文本属性。文本 range 可能包含嵌入对象，Provider
也可只实现部分能力；它是逻辑文本模型，不是 glyph、逐次绘制调用或屏幕像素的记录。
[About Text and TextRange Control Patterns](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-about-text-and-textrange-patterns)

首版读取规则：

- 元素支持 `TextPattern` 时，优先对 document range 做一次**有上限**的 `GetText(max_length)`，不传无限
  长度抓取大型文档。
- `TextChanged` 只视为“需要重新读取”的信号；发布前比较上一份有界快照，避免 Provider 为一次编辑
  发出多次通知造成重复。
- 不默认遍历所有 child range、格式属性或整篇文档；超出上限时发布截断/oversized 诊断，不把截断文本
  自动制成字典词条。
- 密码、受保护 document 或 Provider 拒绝访问时保持空观察，不尝试 OCR 或其他旁路。

### 2.3 `ValuePattern`

`ValuePattern` 表达一个可作为字符串读取（以及在非只读时设置）的控件值，典型用途是单值输入控件；
它与表达多段文档内容的 `TextPattern` 不是同一个语义。
[Implementing the Value Control Pattern](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-implementingvalue)

GlyphShift 首版只调用读取侧 `CurrentValue`，从不调用 `SetValue`：

- 单值 Edit/Combo 等元素支持 `ValuePattern` 且值非空时，Value 通常优先于 `Name`；`Name` 仍可作为
  label 观察，但两者用不同 `source_kind`，不能合并成同一原文。
- 元素同时支持 `TextPattern` 和 `ValuePattern` 时，若两者文本完全相同只发布一次；富文本/多段内容
  优先 `TextPattern`，明确单值控件优先 `ValuePattern`。
- 只读状态不妨碍观察；首版无论只读与否都不写回。
- 密码或安全输入控件一律不读取、不缓存、不发布。

### 2.4 建议的确定性优先级

| 元素能力 | 首选内容 | 补充内容 | 去重方式 |
|---|---|---|---|
| 富文本/文档 + `TextPattern` | 有界 document range | `Name` 仅在它是不同的可访问 label 时保留 | 文本 hash + element identity + source role |
| 单值控件 + `ValuePattern` | `CurrentValue` | `Name` 作为 label 单独观察 | 相同值只保留 Value |
| 标准 `Text` / Label | `Name` | 支持 TextPattern 时可用 range 验证 | 相同文本只保留一条 |
| 仅有 `Name` | `Name` | 无 | element identity + value hash |
| 三者均无 | 不发布 | 记录有界 coverage 计数 | 不猜测，不自动转 OCR |

这个优先级属于 GlyphShift 的归并策略，不是 Microsoft 对所有 Provider 的保证；必须在 observation 中
保留实际命中的属性/Pattern 证据。

## 3. 目标进程与后代订阅

### 3.1 不订阅整个桌面

UIA 支持在某个 element 上按 `TreeScope` 注册 Automation、PropertyChanged 和 StructureChanged 事件。
GlyphShift 不应从 Desktop Root 以 `Descendants` 订阅全部事件；这会越过用户授权目标、扩大隐私面，并
使故障 Provider 和事件洪水影响整个 worker。
[IUIAutomation::AddAutomationEventHandler](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomation-addautomationeventhandler)

建议流程：

1. Controller 只把已授权且仍存活的 root / descendant process instance 交给 worker，使用不可复用的
   短期 opaque target token，不把 PID 当跨会话 identity。
2. worker 枚举这些实例拥有的顶层窗口，用 `ElementFromHandle` 解析每个窗口根；只在该根的
   `Element | Descendants` 范围安装必要事件。
3. 最小事件集为 `Name` property change、支持文本控件的 text-changed，以及结构变化；焦点事件只用于
   触发对当前授权根的检查，不能把全局焦点元素直接发布。
4. 新窗口、窗口销毁、Process Family 成员出现/退出时，由 Controller inventory 触发 root 集合重算；
   移除旧根 handler 后再释放相关 snapshot。
5. 每次回读 element 的 CurrentProcessId，并与当前授权实例集合核对。跨进程控件宿主、浏览器 helper
   或代理导致 ProcessId 不同的元素默认不发布；只有它已被 Process Family 明确接纳才进入观察。

UIA tree 与 Windows process tree 不是同一棵树：一个顶层窗口可能代理另一个进程中的 Provider，某个
后代进程也可能拥有自己的顶层窗口。因此“订阅 root 窗口的 subtree”与“覆盖已授权 Process Family”
必须分别维护，不能互相推定。

### 3.2 缓存与跨进程往返

UIA Client/Provider 可能跨进程通信，逐 element、逐属性同步读取会产生大量往返，并把慢 Provider 放大
成 UI 卡顿。Architecture 文档说明 UIA Core 负责跨进程与代理通信；由此得到的项目约束是：事件注册和
树查询必须带精简 CacheRequest，一次只缓存判断/去重必需的 ProcessId、RuntimeId、ControlType、
AutomationId、Name 与所需 Pattern 可用性，不能缓存整棵桌面树。
[Architecture and interoperability](https://learn.microsoft.com/en-us/windows/win32/winauto/architecture-and-interoperability)

Cache 是读取性能工具，不是持久真相。元素失效、Provider 重启或窗口重建后，旧 cache 与 COM element
引用都必须丢弃并重新解析。

## 4. 跨位数、权限与故障边界

### 4.1 x86 / x64

UIA 的 Client/Provider/Proxy 架构用于跨进程互操作，也承接 MSAA 与 UIA Provider 的桥接；因此
GlyphShift 的 UIA Client 不应像 native TargetProcess Hook 一样为每个目标位数注入对应 DLL。
[Architecture and interoperability](https://learn.microsoft.com/en-us/windows/win32/winauto/architecture-and-interoperability)、
[UI Automation and Microsoft Active Accessibility](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-msaa)

但“无需注入”不等于“跨位数必定完整”。兼容代理、旧 MSAA Provider、自定义 out-of-process Provider
仍可能暴露不同属性或事件行为。生产声明前必须分别以 x64 client → x64 target、x64 client → x86
target 验证；若未来发布 x86 Controller，也补反向矩阵。覆盖差异应报告为 Adapter 兼容证据，不能用
进程名硬编码绕过。

### 4.2 UIPI、完整性级别与受保护界面

UI Automation 受 Windows 安全边界约束。Microsoft 说明 UIAccess 应用可以绕过 UI Privilege
Isolation 的一部分限制以服务辅助技术，但必须满足签名与受信安装位置等要求；UIA 也不能被当成绕过
受保护系统 UI 或安全桌面的通道。
[UI Automation Security Overview](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-securityoverview)

GlyphShift 首版不申请 `uiAccess=true`，也不为观察自动提权：

- 普通 worker 观察普通完整性目标；目标更高完整性、位于安全桌面或 Provider 拒绝访问时，明确返回
  `permission-denied / integrity-boundary / unavailable`，而不是笼统“组件不兼容”。
- 如果用户显式以提升后的 GlyphShift 启动，可在同一权限上下文重新 probe；不能静默重启为管理员。
- 未来若确需 UIAccess，必须作为独立安全议题评估签名、安装、授权提示和扩大后的桌面读取面，不属于
  Adapter 兼容小修。
- 密码字段、凭据 UI、受保护内容与 secure desktop 一律排除，即使某个 Provider 偶然返回文字也不保存。

### 4.3 Provider 卡顿、崩溃与事件洪水

UIA 调用可能跨进程进入应用或代理 Provider。Microsoft 的线程指导要求把 UIA 调用放在不拥有 UI 的
独立 MTA 线程，正是因为调用与事件可能重入、阻塞或造成死锁风险。
[UI Automation Threading Issues](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading)

GlyphShift 不能假设任意同步 COM 调用都能被安全取消：

- 每次解析/读取任务有软时间预算；超时先停止接纳该 root 的新任务并报告 degraded。
- 独立 supervisor 监测 heartbeat。控制线程长期卡在 Provider 调用时，由 Controller 终止并重建整个
  worker process；不在线程内强杀 callback，也不让目标进程承担恢复代价。
- 对单 root 做指数退避和熔断，其他授权 root 继续工作；失败开放意味着“不产生观察”，不是伪造空串。
- 队列、单条文本、每 element snapshot、每批事件和诊断都有硬上限。只保留聚合丢弃数，不记录无限
  原始事件日志。
- worker 被重启后 generation 改变，旧 IPC 消息和旧 RuntimeId 全部作废。

“把 UIA 放到独立 worker 可以由 watchdog 回收”是 GlyphShift 的隔离设计推论，不是 Windows 对单次
Provider 调用提供的超时保证。

## 5. Element identity 与去重

UIA RuntimeId 适合标识当前桌面会话中一个仍存活的 element，但不能作为跨进程重启、窗口重建或应用
版本变化后的领域主键。UIA 允许 Provider 生成 RuntimeId，元素失效后 Client 也必须重新取得对象。
[IUIAutomationElement::GetRuntimeId](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationelement-getruntimeid)

建议分两层：

```text
会话内 ElementKey
  = worker_generation
  + controller_target_token
  + top_level_root_token
  + RuntimeId（有则使用）
  + fallback ordinal（只在当前 root snapshot 内）

可选 Stream Fingerprint（仅用于重匹配证据）
  = adapter_id
  + bounded ancestry roles / ControlType
  + stable AutomationId（Provider 确实提供时）
  + source_kind
```

- PID、HWND、RuntimeId、树序号和内存地址全部是短期 Runtime actual state，不进入 Dictionary Entry。
- `AutomationId` 可以缺失、重复或随版本改变；只能作为 fingerprint 的一部分，不能单独成为持久 identity。
- 窗口标题、可见文本和翻译结果不能参与稳定 identity，否则文字变化会制造新 stream。
- 只有合成测试和同软件跨重启证据证明 fingerprint 可重匹配后，Probe 才保存 opaque Stream Binding；
  否则 UIA observation 仍是未绑定候选。

### 5.1 事件归并

Provider 可能为同一次 UI 更新发出 Name change、Text change、Structure change 和 focus change。建议：

1. callback 先按 `(generation, ElementKey, event kind)` 做极短窗口的调度合并，不按文字永久吞事件。
2. MTA 线程读取目标来源的有界快照，计算 `(ElementKey, source_kind, normalized UTF-16 hash)`。
3. 与该 element/source 的上一份已发布 snapshot 相同则丢弃；A → B → A 仍必须保留三次状态变化。
4. `Name`、Text range、Value 完全相同时按第 2 节优先级只发一条，并记录 `corroborated_by`，不制造三条
   词典候选。
5. 中央 Aggregator 再跨 Adapter 做可选的短时呈现去重，但保留来源证据；UIA 与 GDI 同时观察到相同
   文本不能让任一 Adapter 冒充另一技术覆盖。

## 6. 与 GlyphShift 现有架构的接缝

### 6.1 已具备

- 领域模型已有 `Placement::IsolatedWorker` 与 `ApplyModel::ObserveOnly`。
- Registry 能表达 IsolatedWorker binding，Feature 能收窄为 `TextObserve`。
- 现有 `TextObservation`、Adapter Descriptor 和 Runtime Publication 可作为 UIA observation 的最小上层
  合同；平台细节进入可选证据 envelope，不进入 Dictionary Entry。

### 6.2 已完成的前置与剩余生产门槛

1. **Isolated Worker Host（已完成）**：已有独立进程启动、握手、健康检查、暂停、停用、尾批次排空、
   超时立即回收、新 producer generation 与每 60 秒最多 3 次限频重启。
2. **有界 IPC 协议（已完成）**：已有 protocol version、producer/publication generation、临时 target
   grant、bounded observation batch、health、deactivate acknowledgement、事件洪水统计和受校验的
   Worker rejection code。
3. **中央 Observation/Capture Aggregator（已完成）**：Target Runtime 只产生有界 batch，Desktop
   为多个 producer 维护独立 cursor，并由一个进程外 `FileCaptureSink` owner 串行写 checkpoint。
4. **权限与能力 Probe（已完成）**：无窗口、权限不足、Provider 超时和 worker 崩溃已有独立内部
   reason；权限拒绝与超时可上送为现有用户错误，授权 UAC 合同已覆盖更高完整性失败关闭。
5. **Worker Watchdog 与资源预算（已完成）**：事件量、树宽度、文本长度和重启频率已有明确上限；
   真实 Provider 永久阻塞、Worker 回收、目标释放、新 generation 重连和预算耗尽后的人工恢复均有合同。

真实 MTA UIA Client、标准控件事件、密码拒绝、窗口树重建、watchdog、资源预算、高完整性与永久阻塞
Provider 合同均已完成。UIA descriptor 可以进入独立的 observe-only Bundle / catalog 集成切片。

### 6.3 不进入首版的能力

- `TextReplace`：UIA 写回会改变控件值而不是绘制文字，还可能触发业务逻辑、校验、保存与安全动作。
- `FontSubstitute` / `LayoutAdjust`：UIA 没有通用呈现字体或布局写回合同。
- 全桌面监听：超出 Software / Process Family 授权边界。
- OCR 自动兜底：它是另一条显式能力和隐私模型，不能因 UIA 无文字就自动启用。
- UIAccess 提权、secure desktop、凭据/密码观察。
- 把 UIA element/path 变成 Dictionary `location` 或逐条 context。

## 7. 合成验证矩阵

所有 tracked 测试只使用确定性合成 Provider/宿主；真实软件路径、PID、窗口标题、截图、raw tree dump 和
捕获文字全部放在 `local-test/evidence/`，不写入 Flightdeck 或源码。

| 维度 | 合成场景 | 必须证明 |
|---|---|---|
| 生命周期 | worker 启动 → 注册 → observe → deactivate → reactivate → shutdown | Add/Remove 幂等；停用后无新发布；旧 generation 消息被丢弃；无 handler 泄漏 |
| MTA / 回调 | UIA 控制线程无窗口；多个线程并发触发事件 | handler 线程安全、只入队；无回调内同步 IPC/落盘；不会与 remove 死锁 |
| `Name` | 标准 Text/Label、Button、Name 为空、Name 与 label 相同 | 正确读取与分类；空值不发布；不把控件类型/HelpText 拼成原文 |
| `TextPattern` | 多行文档、嵌入对象、大型文本、连续 TextChanged | 有界 `GetText`；快照去重；超限诊断；不无限遍历 range |
| `ValuePattern` | 单行 Edit、只读 value、Value 与 Name 相同、Value 快速变化 | 只读取不 SetValue；相同来源归并；A → B → A 不被永久去重 |
| 敏感内容 | Password / protected content fixture | 不读取、不缓存、不发布，日志无原文 |
| 目标边界 | root 窗口、同进程 child、明确授权 descendant、未授权 helper | 只发布授权 process instance；UIA subtree 与 process family 分别校验 |
| 窗口变化 | 新顶层窗口、销毁重建、Provider 重启 | handler 正确增删；RuntimeId/cache 失效；重建后新 generation/key |
| 事件洪水 | 同 element 重复 Name/Text/Structure 事件、队列满 | 有界合并和 drop counter；worker/Controller 内存稳定；目标不被反压 |
| Provider 卡顿 | 属性读取或 Pattern 调用永久阻塞 | heartbeat 失败、supervisor 回收 worker、其他目标可恢复、目标软件不受影响 |
| Provider 异常 | stale element、COM error、Provider crash | 明确 reason code；fail-open；不发布空串或陈旧 snapshot |
| 跨位数 | x64 Client → x64 synthetic target；x64 Client → x86 synthetic target | 属性、Pattern、事件合同分别通过；差异可诊断，不依赖 injected DLL |
| 完整性边界 | 同完整性、目标提升、拒绝访问 | 普通目标可用；高完整性明确 permission/integrity 状态；无自动提权 |
| 单写入聚合 | UIA worker + 两个 target-process observer 同时发布 | 只有 Aggregator 写 checkpoint；sequence/revision 单调；无覆盖与重复 writer |
| 崩溃恢复 | worker 在回调、查询、IPC 中分别退出 | Controller 标记 degraded、清旧 generation、限频重启、恢复后不重放陈旧事件 |

进入真实软件 smoke 前，还应有一个纯合成端到端合同：Controller 下发两个授权 target token，worker
订阅各自 root 并产生 Name/Text/Value observation，Aggregator 与一个 target-process observer 合流后写出
唯一 checkpoint；随后关闭一个 root、卡死另一个 Provider，验证隔离、诊断、重启和单写入语义。

## 8. 分阶段建议

### 阶段 A：当前 Work 内完成（已完成）

- 固化本文的 Placement、线程、内容优先级、权限、identity 和失败语义。
- 为 isolated worker handshake、generation 与 bounded observation batch 写协议合同，不加载真实 UIA。
- 复用已完成的多 producer / 单 writer Aggregator 合同，把合成 IsolatedWorker producer 接入同一 owner。
- UIA descriptor 在该阶段仅存在于测试 fixture 或 research slice，不进入构建脚本、catalog 或正式 Bundle。

### 阶段 B：技术原型（已完成）

- 实现最小独立 MTA worker，只支持显式 top-level root、`Name` 与合成 Text/Value Provider。
- 通过生命周期、事件洪水、卡死 watchdog、x86/x64 和完整性诊断矩阵。
- 真实目标 smoke 只产生 local evidence，用来判断 UIA 对现有 GDI/DirectWrite 缺口是否有实际增益。

### 阶段 C：生产候选门槛（已完成）

- Isolated Worker Host、IPC、中央 Aggregator、Watchdog 与诊断状态全部已有生产合同。
- 至少一个明确目标软件证明 UIA 能稳定补充高价值文字，且跨重启 fingerprint 的 degraded/unmatched 行为
  可解释。
- 关闭 UIA Adapter 后无残留 handler/worker；Provider 卡死不会卡 Desktop/Controller；敏感内容测试通过。
- 到此才把 descriptor/artifact 加入正式 Runtime Bundle，并仍只声明 `TextObserve + ObserveOnly`。

## 最终建议

UIA 值得保留为后续 observe-only Adapter，但它的价值在于“从可访问性 Provider 取得结构化候选”，不是
替代所有绘制 Hook。GlyphShift 已建立**进程外 Observation/Capture Aggregator 单写入数据面**，并完成
通用 Isolated Worker、有界 IPC 与合成 Provider 策略合同。真实 Windows MTA UIA Client、事件 handler、
权限 Probe、watchdog 回收与正式 Bundle 集成也已交付；发布边界保持
`IsolatedWorker / ObserveOnly / TextObserve`。当前下一步是 Observation Stream Identity。
