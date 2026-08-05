# 运行时能力升级 Roadmap

Status: Open

## Goal

在不扩大 Dictionary 职责、不泄漏平台细节到核心匹配 Interface 的前提下，按真实软件证据逐步扩展
多进程接管、观察流复用、Windows 文字技术覆盖、交互式取词、原生隔离和布局适配能力。

## Current

优先级已纠正：Observation Stream、跨启动 Binding 与 Probe/诊断/Workflow 共享 Hook 都不能直接增加
实时翻译覆盖，继续延后到真实软件案例证明必要时再恢复。Web 桌面原型证明 DOM ownership 和父进程
继承管道可以成立，但也证明这条路径依赖目标宿主在 WebView 创建前主动开放接口；绝大多数已运行的
Electron/WebView2 软件没有该接口，GlyphShift 不能通用 attach。因此 Web 已降级为后续
Host-assisted Integration，不再作为当前通用 Adapter 或首个 Engine-aware 交付目标。

Process Family Stage 已交付。Software Extension 现在可声明显式后代可执行文件 allowlist，并经
Desktop Runtime Spec、Controller Host 配置进入 Windows Controller；工作流和 Dictionary 不承担
进程关系。授权根仍优先使用用户选择的完整路径，后代必须同时命中 allowlist 且位于已授权进程树。

Windows Controller 使用 PID + 创建时间作为不跨 seam 的实例身份，输出稳定且永不复用的 opaque
token。合成 inventory 已覆盖嵌套后代、重排、根先退出、成员部分退出、身份不可得和 PID 复用；
退出实例的 Runtime 记录随之清理，仍存活实例继续可用。授权 AE 的本地快照证明一个根进程拥有
8 个后代、3 类辅助可执行文件；参数化实机合同发现 9 个目标实例。结论是首个生产 Process Family
应由独立 AE Software Extension 显式声明这些成员，Core 不增加品牌分支。

Workflow 仍选择 inventory 的第一个目标，Controller 将同路径顶层根稳定排在后代 utility 前；多个
独立顶层实例的写回仍需要显式 Target Selection。Probe 未显式缩小目标时现在会激活全部已授权
Process Family 成员，每个 target 独立部署 Hook 与 batch producer，但所有批次由 Desktop 单写入 owner
汇入一个 checkpoint。Workflow 与 Probe 继续以软件级互斥避免重复注入。

Target Execution 盘点已完成并否决立即增加租约注册表：Process Family capture 的多 producer/single
writer 已解决，但 Runtime diagnostics 仍是取走最近批次的单消费者语义，Workflow 与 Probe 也仍会
形成不同的 Publication 所有权。没有稳定 Observation Stream Identity 与多游标读取，Subscriber
Module 仍不能减少注入或提供安全共享。因此当前互斥保留，Observation Stream 合同仍是共享执行前置。

Direct2D `DrawText` 合成原型已完成，但授权 AE 中命中为零，未进入生产 Bundle。Windows Console
`WriteConsoleW` 观察器已完成并作为第五个 Adapter 进入正式 Bundle：它只供探针采集当前被注入进程
实际经过该 API 的 UTF-16 文本，不执行翻译写回，也不承诺子进程或完整 Terminal session 覆盖。授权
隔离 CMD 场景得到 18 次命中和 `Open`、`File`、`Edit` 三类干净候选；后续 Clink 场景暴露的
ANSI/VT 控制序列现已在 Adapter 边界剥离，纯控制调用不再污染观察索引。

Console Process Family 边界也已用真实 Windows 父子进程合同固定：父进程部署不会观察子进程，必须
对每个 target 独立部署。Desktop Probe 现会为全部已授权成员部署独立 batch producer，由进程外唯一
owner 写同一 checkpoint；暂停、恢复、成员退出隔离和 stop drain 均通过真实父子进程合同。

UI Automation Seam 调研已完成。它适合补充标准控件的结构化文字，但正确 Placement 是独立 MTA
Worker，不是目标进程 DLL；可访问性语义也不等同于实际绘制。Observation Ingress 与 Desktop 单写入
Capture owner 已完成。`glyphshift.isolated-worker/1`、受监督进程 Host、Controller Worker Target Grant、
Hybrid Placement 路由与合成 Provider 文本策略现也已贯通；暂停、generation、health、deactivate tail
和同一 checkpoint 均有跨进程合同。真实 Windows MTA UIA Client 现也已通过标准 Win32 控件进程合同：
只枚举授权 PID 的可见顶层窗口，初始读取 Name/TextPattern/ValuePattern，注册属性、文字和结构变化
handler，以 1 秒有界扫描弥补 Provider 漏事件，并在读取文本前拒绝密码控件。密码属性不可读时会
失败关闭，Worker health 也能报告 `uia_permission_denied`；Host 和 Desktop command 已将权限拒绝与
超时分类为现有用户错误。确定性合同也已证明顶层窗口和完整标准控件树销毁重建后，同一 Worker 会注册
新 root、失效旧元素并继续采集。授权 AE 已连续两轮在 5 秒内观察到 21 条唯一公开文本，health 均为
Healthy，因此真实收益门槛已通过。人为永久阻塞真实 Provider 后的目标恢复也已通过：Worker 超时会
被立即回收，解除目标阻塞后可用新 producer generation 重新连接；预算耗尽后现有停止/重新连接入口
也能移除终态 supervisor 并恢复 Healthy。显式授权 UAC 合同进一步证明：目标完整性高于 Worker 时，
激活会在任何 observation 发布前稳定返回 `uia_permission_denied`，不会把部分可见 UIA 根误报为
Healthy。UIA 的生产安全门已完成；Descriptor 与 Worker artifact 也已进入正式 Bundle / Desktop
catalog。Desktop 只为包含隔离 Adapter 的 recipe 签发临时 grant，UIA 只在 Probe capture 中启动；
Workflow 的翻译候选仍只接纳 `TextReplace`。正式合成目标合同已证明公开标准控件可采集且密码不入库。

交互式取词的底层设计现已独立路由：Point、Text Range 与 Region 共用一个一次性 request/result
Interface，UIA 与授权 Target Frame + OCR 分别作为 Structured / Visual Acquisition Adapter。该链路
不扩充持续 Probe 的 observation payload，也不把坐标写入 Dictionary 或 Region Binding；Translation
Resolver 与 External Translation Presentation 位于取词之后，是否使用热键、悬停、浮层、在线翻译器
或具体 OCR 引擎继续延后。已证明 Native 写回零命中的 WPF 目标选择这条外部呈现路线作为后续回退，
不再为当前目标建设 Managed WPF Agent，也不把外部呈现冒充原位 `TextReplace`。

同类工具的引擎翻译能力汇总已完成。公开清单可用作市场覆盖地图，但公开资料不足以证明逐项技术接入点；
Glyphshift 因此新增后续 Engine-aware Integration Stage，以 build mode、观察来源、应用方式、版本、
证据和失败语义定义支持。项目文件/资源改写将来另走 Content Adapter Work，不混入 Runtime Adapter
或 Dictionary。

Capture seam 已完整贯通：`CaptureIngress` 热路径非阻塞并区分 accepted、paused 与 dropped；Target
Runtime 只拥有 batch producer，Controller 跨进程 drain，Desktop TargetProcessHost 为每个 target 校验
producer/generation/sequence/gap，并把记录串行送入唯一 `FileCaptureSink`。

跨进程 observation payload 也已冻结为 `glyphshift.capture-observation-batch/1`：只包含受 supervisor
管理的 producer、generation、累计 dropped 和严格递增 sequence，记录数、JSON 大小、标识符与原文
均有硬上限，不携带 checkpoint 路径或翻译状态。Target Runtime 的互斥 batch producer、bounded drain
export、Windows remote query、Controller SDK `/3` 与 Host transport 已贯通；真实 Windows Console
注入合约证明观察可跨进程取回且不会与旧 file capture 双写。health/deactivate ack 留到
IsolatedWorker 成为第二个真实 producer 时共同固定。

另外修复了正式 Desktop 目录只暴露 `TextReplace` Adapter 的缺陷：App 现在会展示包含任一有效能力的
Adapter，Console Observer 可供 Probe 选择；Workflow 的翻译候选仍只保留 `TextReplace`，观察型能力
不会被误提升成翻译能力。Probe 选择器同时明确区分“可实时翻译”和“仅采集原文”；混合选择不会关闭
实时预览，Translation Snapshot 只绑定具备 `TextReplace` 的 Adapter。

## Next

- [交互式取词 Seam](slices/interactive-text-acquisition-seam.md)底层设计已完成并进入 Stage 4；实现继续
  延后，恢复时必须先做 UIA Point / Text Range 与 Geometry Fixture，再接 OCR 和外部呈现。
- [Web Desktop 首个真实目标验收](slices/web-desktop-first-target-acceptance.md)已暂停：隔离宿主实验足以
  证明“主动开放接口时可用”，继续修改自有目标 DOM 不能证明对第三方现有软件的通用覆盖。
- Web 后续只有在目标软件提供官方扩展、启动集成或宿主 SDK 时才恢复，并以 Host-assisted Integration
  单独标注；不把 CDP 端口或自有调试构建包装成通用 Adapter。
- 当前 Focus 返回[通用实时翻译覆盖](../real-time-translation-coverage/index.md)；SDL3_ttf 以及其他候选均
  等待授权真实软件先证明动态入口命中和可见增量。

## Progress

- 已完成 AI 评审路由，建立采纳门槛和明确的非目标；Roadmap 按真实证据逐 Stage 推进。
- 已完成交互式取词底层路由：持续 Observation 保持最小；Point、Text Range、Region 统一为短期选区，
  Structured/UIA 与 Visual/OCR 共用同一深 Interface，翻译与外部呈现继续分离。
- 已完成 WPF Apply Model 选择：六条 Native 写回路径零命中后，后续采用结构化取词与外部译文呈现，
  不建设当前 Managed Agent 或全局 Dependency Property 改写。
- 已完成同类 Hook/翻译工具的一手资料汇总，冻结 Observation Stream、Transform Profile
  及可选输入/输出能力的路由边界。
- Process Family Stage 完成：Extension → Runtime Spec → Controller 配置贯通显式后代 allowlist；
  合成生命周期与授权 AE 的 9 实例 inventory 合同通过，全 workspace、Clippy、fmt 与架构检查全绿。
- Target Execution seam 盘点完成：确认当前缺少的是可多读、可重匹配的 Observation Stream，而不是
  另一层租约 Map；在该前置合同完成前保留 Workflow/Probe 软件级互斥。
- 完成 Windows 软件支持分级与 CMD 隔离诊断：当前四个目标进程内 GDI/GDI+ Adapter 在持续输出的
  CMD 客户端中连续两轮均为零信号；Console client/host 是独立技术边界，进入 Adapter Roadmap，
  不按可执行文件名增加特例。
- 修复 Target Runtime 等价部署重连与 Controller 同路径根优先排序；完整回归通过。独立多根实例
  的显式选择继续保留在本 Roadmap，不把窗口标题或进程品牌写入通用 Controller。
- 完成窄 `ID2D1RenderTarget::DrawText` 原型：仅声明 `TextObserve + TextReplace`，真实 DCRenderTarget
  与 WIC render target 均验证观察、像素级替换、失败开放、停用恢复和重新激活；授权 AE 为零命中，
  因此未接入生产 Bundle。
- 完成 `windows.console.write-console` observe-only Adapter：正式 Bundle、探针路由和授权隔离 CMD
  18 次命中均通过；工作流不显示观察型 Adapter，换行符不会进入候选词条。
- 修复真实 Clink 输出中的 ANSI/VT 控制序列污染：Console 边界剥离 CSI/OSC，纯控制调用丢弃、可见
  正文保留；Probe 以能力分组展示 Adapter，混合观察/写回计划只向写回 Adapter 发布预览。
- 完成 Console 真实进程 parent/child 合同：父 Hook 不继承到子进程，Controller 逐 target 部署后
  观察与诊断隔离；因捕获 checkpoint 仍是单目标写入模型，不在 Desktop Runtime 中盲目全家族扇出。
- 完成 UI Automation observe-only Seam 评估：固定独立 MTA Worker、事件生命周期、最小文本通道、
  会话内去重和权限降级；单写入聚合、Worker/IPC、真实 MTA Client、权限与卡顿回收合同均已完成。
- 修复真实 Desktop Adapter 目录遗漏 observe-only 能力的问题；Console Observer 现可进入 Probe
  选择，但仍不会出现在 Workflow 的翻译 Adapter 列表。
- 交付进程内 `CaptureIngress` Interface：两个并发 producer 合流到唯一 checkpoint owner，旧 ingress
  在 owner 结束后明确 dropped；现有 capture 与 Target Runtime 回归通过。
- 冻结 `capture-observation-batch/1`：generation、sequence/gap 与 dropped 证据可诊断，所有输入有界且
  deny-unknown；12 个 Capture 合同、完整 workspace、Clippy、fmt、架构与隐私门禁通过。
- 贯通 observation transport：Target Runtime `/3` 以互斥 batch producer 模式导出有界批次，Windows
  Controller 与 stdio SDK `/3` 拉取，Controller Host 重建受校验 Capture 对象；真实 Console 注入、
  空批次心跳、独立进程 Host、完整 workspace、Clippy、fmt、架构与隐私门禁均通过。
- 完成 Desktop 单写入 Process Family Probe：`CaptureObservationCursor` 校验跨批次 replay/gap，多个
  target supervisor 共用一个 `FileCaptureSink`；真实父子 Console 合同覆盖暂停/恢复、统一 checkpoint、
  子进程退出后父进程继续与 stop 收尾。pull 突然退出尾窗风险已明确记录。
- 完成 Isolated Worker `/1` 与 Host：握手、独立 producer/publication generation、暂停、batch pull、
  health、deactivate ack/tail 和隐藏进程退出均有合同；合成 Worker 与 TargetProcess producer 可复用
  Desktop 拥有的同一 ingress/checkpoint。
- Controller `/4` 新增 Worker Target Grant：Windows 只在已授权 opaque target 上签发临时
  `windows-process-v1` 进程实例 grant，UI 与 Dictionary 不接触 PID 或平台 payload。
- 完成 `HybridAdapterHost` Placement 路由以及 UIA 纯策略：`windows.uia.observe` 仅声明
  `TextObserve + ObserveOnly + IsolatedWorker`；Name、TextPattern、ValuePattern 优先级、密码拒绝、
  UTF-16 上限、A→B→A 变化和元素失效均通过合成合同。Descriptor 后续已进入正式 Bundle。
- 完成真实 Windows MTA UIA Client 最小链路：Worker 校验 PID + 创建时间 grant，只注册授权进程的
  可见窗口；标准 Win32 控件合同证明初始 Name/Text/Value、变化观察、handler 移除和密码拒绝。事件
  队列、单次树宽和文本均有界，并用 1 秒全量扫描弥补 Provider 不发送 Name 变化事件。
  `IsPassword` 读取失败时不再读文本，`E_ACCESSDENIED` 在 Worker health 中具有稳定 reason。
- 完成 Worker timeout 最小 watchdog：超时立即终止进程，supervisor 以新 producer generation 恢复，
  每 60 秒最多重启 3 次；合成卡顿 Worker 的恢复、继续采集和 deactivate 合同通过。
- 保留并校验 Worker 拒绝码：UIA 权限拒绝与 Worker 超时已经 Host/Session/Desktop command 边界
  上送为现有“目标访问失败”和“激活超时”语义，不向 UI 暴露任意 Worker 字符串。
- 完成重启预算终态诊断：永久卡住的合成 Worker 耗尽每 60 秒最多 3 次重启后，Host health 稳定报告
  `isolated_worker_restart_exhausted`，不再丢成泛化“Worker 不可用”。
- 完成真实 UIA 窗口树重建合同：确定性目标销毁并重建顶层窗口和四类标准控件，同一 Worker 重新发现
  新 root、失效旧 RuntimeId 并继续采集公开文本，重建后的密码仍被拒绝；所有等待均有超时边界。
- 完成参数化授权真实目标 UIA smoke：授权 AE 连续两轮 5 秒采集均得到 21 条唯一公开文本，Worker
  health 为 Healthy 且无降级码；原文和机器身份只保存在本地 evidence。
- 完成真实 UIA Provider 阻塞恢复：标准控件窗口线程永久阻塞时 Worker 在有界时间内被回收；解除阻塞
  后新 Worker 重新采集。Supervisor 预算耗尽后，现有停止/重新连接生命周期也能恢复 Healthy。
- 增加同完整性显式断言和默认忽略的高完整性权限合同；授权 UAC 运行证明更高完整性目标在 observation
  发布前被稳定拒绝，capture 为空且测试目标有界退出。UIA 与 Host 专项保持全绿。
- 完成 [UIA Runtime Bundle 集成](slices/uia-runtime-bundle-integration.md)：Bundle 泛化验证
  observe-only Worker artifact，Desktop 按 recipe 最小化签发 grant；Probe catalog、缺件拒绝和正式
  Desktop capture 合同通过，初始空 publication generation `0` 不再误拒绝握手。
- 完成引擎翻译能力汇总：确认同类公开清单适合作为覆盖参考而非实现证明；新增后续 Engine-aware
  Support Matrix 路线，并将文件/资源汉化明确路由到独立 Content Adapter Work。
- 完成 SDL_ttf major 边界复核：SDL2 只能替换后续创建的 Surface，SDL3 才有公开绘制时 Text 对象；
  前者不计实时覆盖，后者等待代表性授权目标后再立项。
- 完成 [Web 桌面软件授权会话评审](references/web-desktop-authorized-session-review.md)：明确私有会话、
  固定本地脚本、DOM 最小范围、敏感数据拒绝、generation 更新和条件恢复合同；尚未进入生产实现。
- 完成 [Web Desktop Session 最小状态机](slices/web-desktop-session-prototype.md)：隐藏主/子 frame 的
  observation、两代译文、应用重渲染和条件恢复 verdict 6/6 通过；一次性代码已删除。
- 完成 [Web Desktop Session 进程外 transport](slices/web-desktop-session-transport-prototype.md)：临时
  endpoint 与退出 verdict 通过，但无凭据第二客户端证明 loopback 端口不私有；生产端口方案 No-Go。
- 完成 [Web Desktop Session 继承管道](slices/web-desktop-session-pipe-diagnostic.md)：隐藏 Chromium 的
  十项 transport/lifecycle 检查全过，无 `DevToolsActivePort`、无进程级 TCP Listen；一次性代码已删除。
- 完成 [Web Desktop 首个真实目标候选](references/web-desktop-first-target-candidates.md)筛选：Obsidian
  官方 CLI 是唯一已确认的宿主主动授权 seam；draw.io 覆盖价值高但 transport/profile 尚未闭合。
- 自有 Yotta 进入候选后重新排序：其 Wails + WebView2 宿主、按 Storage Root 派生的单实例身份和
  非 production 调试配置允许隔离验收；当前已运行实例没有 endpoint，因此不直接 attach。
- 完成 Yotta UAC/Web endpoint 因果诊断：production/admin 与 development/asInvoker manifest 合同、
  debug options 合同均通过；隔离开发 Host 的 CDP/page target 控制两轮全绿。提升 GlyphShift 不能替代
  WebView 创建前的宿主 endpoint。
- 完成 Web 路线产品适用性复核：宿主主动开放接口时 transport 与 DOM 写回可实现，但普通已运行软件
  不具备可通用连接的 endpoint；因此停止首个目标 DOM 验收，将该能力降为后续宿主协作集成。

## References

- [稳定约束](context.md)
- [阶段计划](plan.md)
- [AI 评审路由结论](references/ai-review-assessment.md)
- [产品领域语言](../../../CONTEXT.md)
- [Capability Adapter 调研](../glyphshift/references/adapter-registry-and-cross-platform-interception-research.md)
- [Process Family Controller Inventory](slices/process-family-controller-inventory.md)
- [Target Execution 所有权](slices/target-execution-ownership.md)
- [Observation Stream Identity](slices/observation-stream-identity.md)
- [Observation / Capture Ingress](slices/observation-capture-ingress.md)
- [Windows 软件支持分级与 Console 缺口](references/windows-software-support-and-console-gap.md)
- [DirectWrite / Direct2D Adapter Seam 调研](references/directwrite-adapter-seam-research.md)
- [Windows Console Adapter Seam 调研](references/windows-console-adapter-seam-research.md)
- [Windows Console `WriteConsoleW` 观察适配器](slices/console-write-console-observer.md)
- [Windows UI Automation Observer Seam 调研](references/windows-uia-observer-seam-research.md)
- [UI Automation observe-only Seam](slices/uia-observer-seam.md)
- [UIA Isolated Worker transport 与合成 Provider](slices/uia-isolated-worker-transport.md)
- [Windows UIA MTA Client](slices/windows-uia-mta-client.md)
- [UIA Runtime Bundle 集成](slices/uia-runtime-bundle-integration.md)
- [交互式取词 Seam](slices/interactive-text-acquisition-seam.md)
- [引擎翻译能力调研汇总](references/engine-translation-capability-summary.md)
- [Web 桌面软件授权会话评审](references/web-desktop-authorized-session-review.md)
