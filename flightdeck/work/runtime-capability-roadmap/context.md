# Work Context

## 保持不变的核心

- Dictionary 保持纯 `source + translation`；探针证据、字体、平台、技术和运行路由不进入 Entry。
- Workflow Target 继续组合 Software、Adapter Plan、Dictionary 和内联 Font Policy。
- Capability Adapter 是观察或写回能力的执行单元；Platform 是兼容事实，Technology 只用于分类。
- 最小 `TextObservation`、Adapter Descriptor 和完整 Runtime Publication 是稳定 Interface；平台富信息
  只能进入可选且有界的证据 envelope。
- 所有失败路径 fail-open；Core、Desktop 和 GUI 不按软件品牌、可执行文件名或 Adapter ID 分支。

## 候选升级方向

### Process Family

- Software Extension 以授权根可执行文件和显式 `descendant_executables` 描述进程关系；Controller
  只接纳当前进程树中命中 allowlist 的后代，不从窗口标题、安装目录邻近或品牌名称猜测成员。
- Controller 内部以 PID 与进程创建时间识别一次进程实例，对外分配跨 inventory 稳定且永不复用的
  opaque token；PID 和创建时间都不跨 Controller seam。
- 已接纳成员在根先退出后可继续存活，并可授权自己新产生且命中 allowlist 的后代；成员退出会删除
  自己的 target/runtime 状态，不影响仍存活的兄弟实例。
- 无法取得稳定实例身份的单个进程不进入 inventory，但不能让其他可识别实例一起失败。
- PID、窗口标题和本机路径只属于短期 Runtime actual state，不进入持久字典或通用 UI。

### Target Execution

```text
Target Execution
├─ Publication Owner      0..1
└─ Observation Subscriber 0..n
```

- 一个目标实例只维护一套注入、Adapter Host 和 Hook，避免 Workflow 与 Probe 竞争或重复注入。
- Publication Owner 独占文字/字体写回；Observation Subscriber 只能消费有界观察流。
- 只有真实的并发探针、诊断或协作需求证明价值后，才替换当前简单所有权规则。
- Target Runtime capture 已改为 batch producer，Desktop TargetProcessHost 为每个 target 维护独立 cursor
  并由唯一 `FileCaptureSink` 写 checkpoint；Process Family Probe 不再多写文件。Runtime diagnostics
  仍是取走式单消费者，Workflow/Probe 也仍会重复注入，因此在独立多读 Observation Stream 交付前，
  Publication Owner/Subscriber 注册表继续延后，软件级互斥保留。
- Capture module 已提供可克隆、非阻塞的 `CaptureIngress`、有界 batch producer 与跨批次 cursor 校验；
  Target Runtime callback 只持有 ingress。batch 经 Runtime drain export、Windows Controller 与 Host
  跨进程返回，Desktop 已成为唯一 `FileCaptureSink` owner。UIA 已复用同一 ingress、Isolated Worker、
  IPC 与生命周期 ack；通用 Host 已具备超时立即回收和限频重启。UIA 对密码属性读取失败会
  失败关闭，并在 Worker health 中保留 `uia_permission_denied`；Host/Session/Desktop command 已将
  权限拒绝与 Worker 超时上送为稳定的现有用户错误，重启预算耗尽也会保留
  `isolated_worker_restart_exhausted`。完整窗口树销毁重建后的 root 重注册、旧元素失效和继续采集已有
  真实合同。真实 Provider 永久阻塞后的 Worker 回收、目标解除阻塞后的新 generation 重连，以及重启
  预算耗尽后的停止/重新连接恢复均已通过。授权 UAC 合同也证明更高完整性目标会在发布任何 observation
  前以稳定权限码失败关闭；UIA 的生产安全门已完成。

### Observation Stream

- Adapter 与同一 Adapter 内的候选文本流是两层身份；候选流属于 Runtime Observation，不是
  Dictionary location，也不能用 PID、绝对地址或窗口标题充当持久语义。
- Adapter 可提供不透明 Stream Fingerprint、样本历史和重匹配证据；Probe 保存 Stream Binding，
  目标重启后解析当前实例并分别报告 matched、degraded 或 unmatched。
- 只有 Adapter 证明指纹跨启动稳定且可诊断后，Workflow 才能引用 Stream Binding；会话内调用点
  不自动升级为 Region Binding。

### Adapter 扩展

- DirectWrite/Direct2D 首个候选已收敛为 `ID2D1RenderTarget::DrawText`：只在每次绘制时替换 UTF-16，
  仅声明 `TextObserve + TextReplace`；会保留译文的 `CreateTextLayout` 不满足停用恢复语义。合成
  DCRenderTarget 与 WIC 合同通过，但授权 AE 为零命中，因此原型不进入当前生产 Bundle。
- Console client 与 Console/Terminal host 是独立技术边界；当前四个目标进程内 GDI/GDI+ Adapter
  在隔离 CMD 持续输出中为零信号。`windows.console.write-console` 已作为
  `TextObserve + ObserveOnly + TargetProcess` 进入正式 Bundle，只观察被注入进程实际经过
  `KernelBase!WriteConsoleW` 的 UTF-16 调用；不声明子进程、重定向、完整 Terminal session 或
  `TextReplace` 覆盖。Adapter 在上报前剥离 ANSI/VT 控制序列：纯控制调用不生成候选，样式与 OSC
  包裹的可见文字继续保留。Process Family 与 ConPTY-owned session 继续单独评估。
- 真实进程合同已证明 TargetProcess Hook 不随父子关系继承；Controller 按 opaque target 对家族成员
  独立部署。Workflow 仍只选择一个 target，Probe 未显式缩小时会激活全部已授权成员；各 target 只产
  batch，Desktop 单写入 owner 串行写一个 checkpoint。
- UI Automation 优先作为 observe-only Adapter，不把可访问性树等同于实际绘制文字。
- UI Automation 固定为 `ObserveOnly + IsolatedWorker` 候选：专用无窗口 MTA 线程拥有 Client 与事件
  handler，不能伪装为 TargetProcess DLL。中央 Observation Ingress 与单写入 Capture owner 已完成；
  Isolated Worker `/1`、受监督进程 Host、Controller 签发的临时平台 grant、Hybrid Placement 路由、
  health/deactivate ack、合成文本策略与真实 Windows MTA Client 已完成。标准控件合同覆盖初始扫描、
  属性/文本/结构事件、handler 移除和密码拒绝；Worker 超时也会立即回收并按每 60 秒最多 3 次重启。
  `IsPassword` 读取失败已失败关闭，`E_ACCESSDENIED` 已保留为 Worker health reason，并已分类上送到
  Desktop command；完整窗口树重建后的 root 重注册和旧 RuntimeId 失效已有真实合同。授权 AE 连续
  两轮 5 秒 smoke 均观察到 21 条唯一公开文本且 health 为 Healthy。真实 Provider 阻塞恢复、预算耗尽
  后人工重连与更高完整性失败关闭均已通过。Descriptor 与 Worker artifact 现已进入正式 Bundle /
  Desktop catalog，但仍只作为 Probe 的 observe-only 候选，不扩大为 `TextReplace`。
- UIA 会话内可用 Target Instance + RuntimeId + 文本通道去重；AutomationId、ControlType、父链和样本
  只能作为跨重启重匹配证据，不能把 RuntimeId 或窗口标题持久化成稳定 Region。
- OCR 是无法取得结构化文字时的显式兜底，必须标明延迟、置信度和隐私影响，不进入目标进程热路径。
- Direct2D、Direct3D、OpenGL 和 Vulkan 仅在目标软件证据显示真实缺口后分别立项，不创建万能图形
  Renderer。

### 交互式取词与外部呈现

- 悬停、划词和框选 OCR 都从一次显式 `Interactive Selection` 进入同一个取词 Interface；热键、鼠标
  手势和呈现样式属于调用者，不为每种交互复制一套采集管道。
- 持续 Probe 的 `CaptureObservationBatch/1` 保持最小文字事实，不增加坐标、控件或截图字段；交互取词
  通过独立 request/result Seam 返回有界原文块与短期屏幕锚点。
- Point、Text Range 和 Region 都绑定当前授权 Target Instance。UIA 作为 Structured Acquirer，授权
  Target Frame + OCR 作为 Visual Acquirer；两者可以按策略回退，但不能扩大到任意桌面截图。
- 屏幕锚点使用虚拟桌面物理像素并只在当前交互会话内有效；窗口移动、滚动、DPI 或目标实例变化后
  重新获取，绝不持久化为 Region Binding、Dictionary Location 或 Probe 证据。
- 取词、Translation Resolver 与 External Translation Presentation 是三个独立 Module。外部呈现不会
  修改目标控件，也不能提升为 `TextReplace` 实际状态。
- UIA 密码内容继续失败关闭；Visual 取词只由用户显式触发，中间图像默认只驻留内存，未来发送到网络
  翻译器或 AI 必须另行取得明确授权。

### Engine-aware Integration

- 同类工具公开的引擎清单只作为市场覆盖参考，不作为 Glyphshift 的实现或支持证明；完整判断见
  [引擎翻译能力调研汇总](references/engine-translation-capability-summary.md)。
- 对外的“支持”必须拆成 Engine/runtime identity、build mode、观察来源、应用方式、已验证版本、
  失败语义和证据；Unity Mono 与 IL2CPP 等构建模式不能合并成一个技术能力。
- 引擎识别和能力组合属于 Software Extension / Adapter / Isolated Worker；Core、Dictionary 和 GUI
  不按引擎品牌分支。Galgame、乙游等内容类型以及 Live2D 等中间件不能直接作为 Adapter taxonomy。
- 项目文件、脚本或资源包的扫描与回写属于后续独立 Content Adapter / Importer Work，不复用实时
  Runtime Adapter 的生命周期，也不把资源位置与改写证据塞入 Dictionary Entry。
- 首个 Engine-aware 验证必须由授权真实目标和确定性 Fixture 驱动；对当前桌面工具方向，优先调研
  V8/Chromium/WebView 等结构化接入，再按实际需求评估游戏专用引擎。
- Web DOM 节点只有在当前值仍等于 GlyphShift 最后一次 applied translation 时才由会话继续拥有；应用
  写入不同值后，该值成为新 source。停用也只恢复仍由会话拥有的节点，不能覆盖应用后续状态。
- Web Session 的随机 loopback CDP 端口不构成授权：同机第二客户端无需凭据即可取得 browser 权限。
  生产 transport 必须由父进程独占的继承 pipe 或目标宿主主动提供，端口模式只能用于本地诊断。
- Chromium 的父子继承 pipe 已通过无调试 endpoint、零进程级 TCP 监听、隔离 Runtime 和退出断线合同；
  它只适用于 GlyphShift 启动并拥有生命周期的目标，不构成 Electron、CEF 或任意现有进程 attach 能力。
- Web/CDP 不属于通用实时翻译 Adapter：只有目标的官方扩展、受控启动或宿主 SDK 明确开放会话时，才可
  作为 Host-assisted Integration 建设。自有调试构建能连接只证明宿主协作可行，不能外推第三方软件覆盖。
- Web endpoint 与 Windows 完整性是正交边界：目标未在 WebView 创建前开放宿主接口时，提升 GlyphShift
  也不能连接 DOM；目标完整性更高时，Native/UIA 仍必须独立返回权限拒绝，不能用 Web transport 成功
  掩盖原生权限不匹配。
- 公开实现抽样只证明通用进程内文字观察和文本流选择，没有证明通用译文写回；类似能力只能作为
  隔离的 observe-only Adapter Pack 候选，不能提升为 `TextReplace`。

### 原生稳定性与布局

- 在现有 reentry/panic fail-open 基础上，单独研究 Windows SEH/VEH、Adapter Watchdog 和熔断；必须
  先确认故障能在宿主退出前被可靠观察。
- `LayoutAdjust` 已是 Feature 候选，只有字体度量、截断或控件布局的真实案例与合成 Host 测试齐备后
  才实现。
- 分裂 text run 由具体 Adapter 做观察重组并携带证据，Decision Engine 不做全局 substring fallback。

### 匹配与辅助创作

- Runtime 默认继续精确匹配。大小写、空白或 Unicode normalization 只有在字典级显式策略和编译期
  冲突诊断同时成立时才考虑。
- 模糊匹配和 AI 语义能力最多用于 Probe 的离线翻译建议，不进入目标进程实时决策。

### Transform Profile

- 捕获正规化、保护/排除和结果修正是独立、有序、版本化且有执行预算的规则资产，由 Probe 或
  Workflow 显式组合；它们不进入 Dictionary Entry。
- 首版只允许声明式、可预览和可测试的 Operator，不允许任意 Python、WASM 或脚本进入 Runtime。

## 明确拒绝

- 富 `TextEvent` 成为包含进程、窗口、控件、坐标、上下文和缓存字段的核心 ABI。
- Dictionary Context、逐条字体、Probe Metadata、Overlay/Draft 双写和通用 Renderer。
- 硬编码 UI Region、窗口句柄、DC 或调用栈 hash 作为稳定 Route。
- 事件溯源重建产品状态，以及没有深度/删除测试依据的 Module 合并。

## 立项门槛

- 每个升级项必须有目标软件证据或合成 Fixture、明确 Interface owner、失败语义和可见验收。
- 新能力应通过新增 Extension、Descriptor 或 Adapter 落地；若必须修改 Core 品牌分支，说明 Seam 不合格。
- 真实软件输入输出全部位于 `local-test/`，仓库只保留确定性 harness 和合成 Fixture。
