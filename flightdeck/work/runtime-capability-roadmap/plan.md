# 运行时能力升级计划

## Stage 1：多进程目标

- [x] [定义 Process Family 授权、发现、身份、退出和部分失败合同](slices/process-family-controller-inventory.md)。
- [x] 用合成 Controller/Target Instance 验证一个软件的主进程与子进程状态聚合。
- [x] 在授权目标软件上保存本地证据并决定首个生产 Extension。

## Stage 2：执行与观察复用

- [x] 将当前独占规则评估为单一 Target Execution、一个 Publication Owner 与多个只读 Subscriber；
  在多游标 Observation Stream 交付前保留现有互斥。
- [x] [定义进程内有界 `CaptureIngress`](slices/observation-capture-ingress.md)：多个 producer 只持有
  cloneable non-blocking ingress，唯一 `FileCaptureSink` owner 管理 checkpoint、revision 和暂停。
- [x] 冻结 `glyphshift.capture-observation-batch/1` payload：producer generation、严格递增 sequence、
  累计 dropped 与记录/字节硬上限均有确定性合同。
- [x] 将 batch 接入 Target Runtime drain export 与 Controller transport；真实 Windows 注入合约证明
  Console 观察可经 Controller Host 返回，现有文件式 capture 保持不变。
- [x] 把唯一 Capture owner 移到 Desktop，并让多个 Target Runtime producer 串行写入同一 checkpoint；
  pause/resume、成员退出隔离与 stop drain 已由真实 Windows 父子进程合同验证。
- [ ] 定义 Adapter 提供的 Observation Stream Identity、样本历史、匹配置信度与持久 Binding 合同。
- [ ] 验证目标重启后按当前实例重匹配 Stream，并分别报告 matched、degraded 与 unmatched。
- [ ] 证明 Probe、诊断和 Workflow 共享 Hook 时不会重复注入、竞争写回或提升错误 Feature 状态。

## Stage 3：Windows Adapter 覆盖

- [x] [研究并完成 DirectWrite / Direct2D `DrawText` 合成原型](slices/directwrite-adapter-prototype.md)。
- [x] 在授权 AE 记录 `DrawText` 零命中，否决为 AE 或当前生产 Bundle 提升该原型。
- [ ] 只有另一个授权目标先证明真实 `DrawText` 命中、render target 类型与可见替换，才重新评估
  Direct2D 生产化。
- [x] 按[Console Adapter Seam 调研](references/windows-console-adapter-seam-research.md)实现
  [`WriteConsoleW` observe-only Adapter](slices/console-write-console-observer.md)，正式 Bundle 与授权 CMD
  实测通过；不把客户端成功注入等同于 session 覆盖。
- [x] 以真实 Windows 合成 parent/child 验证 Console Process Family：父进程 Hook 不观察子进程，
  Controller 对子 target 单独部署后才观察且诊断流隔离；当前不立项 Controller-owned ConPTY。
- [x] 生产 Process Family Probe 使用 Desktop 单写入者 aggregation；每个 target 独立 producer/cursor，
  不再把同一个 checkpoint 路径传入多个目标进程。
- [x] [评估 UI Automation observe-only Seam](slices/uia-observer-seam.md)：确认其应为独立 MTA
  Worker；该评估确定的 Observation Ingress、单写入聚合与 Worker/IPC 前置现均已由后续切片交付，
  生产提升由后续安全与 Bundle 切片完成。
- [x] [交付 Isolated Worker/IPC 与合成 UIA Provider 策略](slices/uia-isolated-worker-transport.md)：
  Controller grant、Hybrid Host、generation、暂停、health、deactivate tail 与 Desktop 单写入 owner 均有
  进程合同；Name、TextPattern、ValuePattern、密码拒绝和变化去重有确定性合同。
- [x] [实现真实 Windows MTA UIA Client 与事件 handler](slices/windows-uia-mta-client.md)：标准 Win32
  控件合同覆盖初始 Name/Text/Value、属性/文本变化、结构失效触发、handler 移除、进程实例 grant 校验、
  有界队列和密码内容拒绝。
- [x] 补齐元素销毁重建后的真实 UIA 恢复：确定性目标销毁并重建顶层窗口和四类控件，同一 Worker
  重新发现新 RuntimeId、失效旧元素并继续采集，重建后的密码内容仍不进入 capture。
- [x] 在授权 AE 完成两轮 observe-only smoke：每轮 5 秒均得到 21 条唯一公开文本，Worker health 为
  Healthy 且无降级码；原文只保存在本地 evidence。
- [x] 补齐高完整性合成目标权限合同；真实 Provider 永久阻塞后的 Worker 回收、目标解除阻塞后的
  新 generation 重连，以及预算耗尽后的停止/重新连接恢复已完成。密码属性读取已失败关闭，Worker
  health 已能稳定报告 `uia_permission_denied`，Host 和 Desktop command 会分别上送权限拒绝与超时。
  授权 UAC 合同证明更高完整性目标在任何 observation 发布前稳定拒绝，测试进程也能有界退出。
- [x] [将 `windows.uia.observe` 接入正式 Runtime Bundle / Desktop catalog](slices/uia-runtime-bundle-integration.md)：
  只进入 Probe 的 observe-only 候选，Workflow 仍只接纳 `TextReplace`；Worker artifact 缺件、正式
  Desktop capture、密码排除和初始 generation `0` 均有合同。
- [ ] 只在结构化 Adapter 无法覆盖且用户明确选择时评估 Window Capture、OCR observe-only 与
  Change Trigger Policy 组成的兜底。
- [ ] 按真实缺口分别决定 Direct2D、Direct3D、OpenGL 或 Vulkan 是否立项。

## Stage 4：交互式翻译回退

- [x] [定义交互式取词 Seam](slices/interactive-text-acquisition-seam.md)：Point、Text Range 与 Region
  共用一次性 request/result Interface；坐标不进入持续 Observation、Dictionary 或 Region Binding。
- [x] [先以 UIA Fixture 交付 Point / Text Range 的结构化取词和多显示器 Geometry
  合同](slices/interactive-uia-acquisition-contract.md)，不同时建设热键、浮层、翻译器或 OCR。
- [x] [再让授权 Target Frame + OCR 作为第二个 Acquisition Adapter 复用同一
  Interface](slices/interactive-visual-acquisition-fixture.md)；截图默认只在内存中存在，跨进程选区、
  密码和受保护窗口失败关闭。
- [x] [为 Structured/UIA 与 Visual/Frame+OCR 固定受监督的一次性 Worker
  transport](slices/interactive-acquisition-worker-transport.md)，先覆盖有界 wire、grant、超时、取消和
  子进程退出，不绑定热键或 UI。
- [x] [接入 Windows UIA Point / Text Range Provider](slices/windows-uia-interactive-provider.md)，复用
  Controller process grant 和一次性 worker wire，以标准控件进程固定真实合同。
- [x] [最后分别组合 Dictionary / Translation Provider 与 External Translation
  Presentation](slices/interactive-acquisition-composition.md)；外部呈现不修改目标软件，也不冒充
  `TextReplace`。

## Stage 5：原生稳定性与布局

- [ ] 研究 SEH/VEH、Watchdog、熔断和运行成本，选择能保持 fail-open 的最小方案。
- [ ] 以真实截断案例验证字体度量和 `LayoutAdjust`，不建立万能 Renderer。
- [x] 在授权 WPF 目标证明 Native 原位写回不可行后完成 Apply Model 评估：后续复用 Stage 4 的结构化
  取词与 External Translation Presentation，不建设通用 Overlay Renderer 或当前 Managed Agent。

## Stage 6：有界 Transform Profile

- [ ] 定义捕获正规化、保护/排除和结果修正的有序、版本化、可预览规则合同。
- [ ] 为每个 Operator 固定输入类型、执行预算、冲突诊断和 fail-open 语义，不开放任意脚本。

## Stage 7：可选匹配与创作辅助

- [ ] 只有精确匹配出现可复现缺口时才设计 Dictionary Match Policy 和冲突诊断。
- [ ] 将模糊/AI 能力限制为 Probe 离线建议，并保留人工确认与确定性 Runtime Publication。

## Stage 8：引擎感知扩展（后续）

- [ ] 依据[引擎翻译能力调研汇总](references/engine-translation-capability-summary.md)定义正式
  Support Matrix：Engine/runtime、build mode、观察来源、应用方式、版本、证据与失败语义分别记录。
- [ ] 用授权真实目标验证首个 Engine-aware Extension；优先调查与桌面工具相关的
  V8/Chromium/WebView 或托管运行时结构化 Seam，不按游戏类型或宣传覆盖数量选择目标。
- [x] 完成 [Web 桌面软件授权会话评审](references/web-desktop-authorized-session-review.md)：固定为
  显式授权 Extension，不通过 Native DLL 绕过调试/宿主授权；生产实现等待安全会话与 DOM 恢复门槛。
- [x] [验证 Web Desktop Session 最小状态机](slices/web-desktop-session-prototype.md)：隔离 world 只处理
  可见普通文本，并证明 generation、应用重渲染与条件恢复不会互相覆盖。
- [x] [验证 Web Desktop Session 进程外 transport](slices/web-desktop-session-transport-prototype.md)：
  临时 endpoint、page attach 与目标退出必须确定，并将随机 loopback 端口与真正私有通道分开。
- [x] [验证 Web Desktop Session 继承管道](slices/web-desktop-session-pipe-diagnostic.md)：仅以 GlyphShift
  作为父进程启动的 Chromium 合成目标验证无监听 endpoint 的私有 transport，不外推宿主支持。
- [x] 复核 [Web Desktop 首个真实目标验收](slices/web-desktop-first-target-acceptance.md)的产品适用性：
  自有隔离宿主只能证明 Host-assisted Integration，不能证明第三方现有软件的通用连接与实时写回；暂停
  DOM 验收，不建设 Catalog/UI。
- [ ] 只有授权 Windows 目标真实使用动态 SDL3_ttf `TTF_Text` 绘制链时，才验证临时译文对象、属性
  复制、热更新与停用恢复；SDL2_ttf Surface 创建时替换不计入实时覆盖。
- [ ] Unity Mono 与 IL2CPP 等差异明显的构建后端分别立证，不发布无版本边界的引擎级支持声明。
- [ ] 评估隔离的通用进程内文字观察器：只声明 `TextObserve`，并先固定候选流选择、跨启动 Binding、
  权限与崩溃恢复合同；不把原文提取成功等同于实时写回。
- [ ] 若确认项目文件/资源汉化需求，另开 Content Adapter / Importer Work；Dictionary 只承接可复用的
  `source + translation`，不承接资源定位和回写状态。
