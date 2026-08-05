# 通用实时翻译覆盖

Status: Open

## Goal

让用户完成稳定、可解释的端到端流程：选择软件，捕获原文，使用或编辑 Dictionary，并由真实
`TextReplace` Adapter 把译文立即写回目标界面。通用性按可复用文字技术扩展，不以“支持所有软件”
或成功注入代替真实翻译结果。

## Current

现有 GDI、USER32 DrawText、GDI+、Qt、GTK 3/Pango 与 DirectWrite TextLayout Adapter 已进入正式
Runtime Bundle。Console 与 UI Automation 目前只有采集价值，不能作为实时翻译成功。Direct2D
`DrawText` 已通过合成写回，但在授权目标中零命中，因此仍未进入生产 Bundle。

DirectWrite TextLayout 已从窄原型升级为生产 Adapter：同时覆盖普通 Render Target 与 Scintilla 缓冲
绘制使用的 Compatible Bitmap Render Target，在 `CreateTextLayout` 取得完整 source，并在
`DrawTextLayout` 时按当前 Dictionary 创建瞬时译文布局。单一格式布局可替换，局部格式、内联对象或
无法证明格式一致的布局安全放行；停止后目标继续使用原布局，不需要修改目标文本。

当前已补齐两段实时更新合同：Probe 内编辑译文会保存到绑定 Dictionary 并发布下一代预览；GDI+
确定性 Windows 宿主在不重启目标的情况下能从首版译文切换到第二代，诊断确认两代均为
`Matched + Replaced`，停止后恢复原文。既有软件上的补充可见验收不再占用当前主线，后续只在明确
需要补充发布证据时恢复。

Probe 与 Dictionary 的创建流程也已收敛：每次打开都使用干净表单，任务名称与新词典名称不再根据
软件自动生成，取消或关闭不会遗留旧草稿；创建阶段只询问名称和语言等必要信息，发布版本、作者、
许可证、主页与标签继续由词典设置维护。目标软件未启动时任务仍保存为“未连接”，不会把连接失败
误报成创建失败；之后显式重连仍显示具体原因。

探针管理列表也已收敛为二态连接语义：运行与暂停都显示“已连接”，其他持久状态显示“未连接”；
应用重启会把旧的运行、暂停或中断记录恢复为未连接，不再制造一排历史故障。任务行只保留名称，
程序位置与 Adapter 技术明细进入可悬浮、可聚焦的详情入口。创建后的首次连接即使失败，只要任务
已经持久化，创建 Modal 仍会关闭并进入详情，同时保留具体连接错误。

混合 Placement 的启动也已改为部分成功语义：Target Process 原位 Hook 与 Isolated Worker 观察器会
分别尝试激活；任一侧成功即可保留会话，失败侧的能力进入 Failed，只有全部 Placement 都失败才拒绝
连接。这样仅支持 UIA 观察的目标不会再被同组选中的不兼容 Native Hook 阻断。

同一 Target Process 内的多条候选 Hook 也已改为部分成功语义。Qt、GTK 等不适用于当前进程的技术只
标记自身失败，不再撤销已经成功的 GDI、GDI+ 或 DirectWrite；只有全部原生技术都失败时才拒绝连接。
注入 Runtime 通过独立激活回执上报真实成功集合，Session 不会把未激活技术冒充为 Active。这样用户
取消 UIA/Console 观察器后，健康的实时写回技术仍可单独建立连接。

`runtime.component_incompatible` 的界面文案不再猜测“刚升级过”或要求重启目标软件，而是明确说明
所选探针技术不适用于当前软件，并引导用户调整技术。HandBrake 类 WPF 目标当前仍只能归为“仅采集
原文”，不能因 UIA 连接成功显示成实时翻译支持。

Probe 连接状态现已使用实际会话 ACK，而不是用户勾选的 Adapter 能力推断。桌面 Runtime 只汇总宿主
真正确认 Active 的 Feature；当前连接据此显示“可直接替换 / 仅采集 / 无信号”。“仅采集”会明确说明
目标软件界面不会被修改；能力状态只属于本次连接，释放连接或重启后不会写进可恢复任务配置。

条目状态也不再把 Dictionary 未命中含糊显示为“待翻译”，而是明确显示“词典未命中”。用户在 Probe
内保存译文后，如果下一代预览没有成功发送到目标 Runtime，界面会说明“词典已保存，但目标界面可能
仍显示旧译文”，不再误报成停止失败。最终应用回执复核也已完成：GDI 可返回 API 状态，但 Qt/GTK
绘制入口没有返回值，而且 API 接受调用仍不等于像素可见，因此不存在可信的跨 Adapter 最终成功状态。
运行诊断现将 `Matched + Replaced` 明确显示为“替换决策已生成”，不再显示“已替换”。

Qt Widgets `QPainter::drawText` 绘制链现已完成。Qt 5/6 MSVC x64 ABI 分支均通过本地 QImage 合成
宿主的像素替换、同进程第二代 Dictionary 更新与停用恢复；首个授权目标因静态链接 Qt 在注入前被
安全拒绝，没有退化成软件专用签名扫描。随后动态链接 Qt 6 Widgets 真实目标完成可见验收：菜单原文
按 Qt 助记符语义命中字典，首版译文、第二代热更新译文和停用后的原文恢复均可见，诊断同时记录
`Matched + Replaced`。Adapter 已进入正式 Runtime Bundle 与中文目录，边界明确为动态链接 Qt 5/6
Widgets；QML、静态 Qt、富文本与静态文字缓存仍不在承诺内。

WinUI 3 / MRT Core 三层诊断也已完成。显式 `ResourceLoader.GetString` 与声明式 `x:Uid` 分别命中
`MrmLoadStringResource` 和 `MrmLoadStringOrEmbeddedResourceByIndex`，合成宿主可按 MRM 配对分配器完成
精确替换，并在停用后重新加载窗口恢复原文。但 `x:Uid` 是加载时赋值，已显示控件不能通用地即时刷新或
回滚；同时两条路径并不存在一个共同的窄入口。因此它不进入当前实时翻译 Bundle，只保留为未来可选的
“加载时资源替换”能力。

动态 GTK 3 / Pango 绘制链也已完成。Adapter 只挂公开 `gtk_render_layout`，读取原始 UTF-8 后复制
`PangoLayout` 并在副本上应用 Dictionary 译文；普通文本与全范围样式可替换，markup、mnemonic、link 等
局部 byte-index 样式安全放行。离屏像素合同、第一代/第二代热更新、停用恢复、无模块拒绝激活和正式控制器
注入链均通过，Adapter 已进入正式 Runtime Bundle 与中文目录。GTK 4 标准控件依赖私有 snapshot 入口，
继续维持 No-Go。

此前运行时 Roadmap 将 Observation Stream、跨启动 Binding 和共享 Hook 提到当前主线；这些工作不能
增加可翻译软件数量，现已退回后续 Roadmap。尚未提交的 Observation Stream 实现已撤回，现有简单
Workflow/Probe 互斥继续保留。

首个“真实软件缺口驱动”目标也已完成验收。授权 WPF 工具加载 .NET/WPF、DirectWrite 与 Direct2D；
正式 UIA Worker 可健康观察 72 条唯一公开文本，但 GDI、DrawText、GDI+、Direct2D DrawText 和本地
DirectWrite TextLayout 六条可写回路径在持续重绘中全部零命中。WPF 因此不进入当前 Native Bundle；
后续已选择 UIA 取词与外部译文呈现路线并转入 Roadmap，当前不建设 Managed WPF Agent。

下一真实目标的 Go/No-Go 已完成。Scintilla 自身确认使用 DirectWrite 模式；编辑区完整测试文本在正式
Runtime Bundle 中完成首代 Dictionary 替换、同进程第二代热更新和停止恢复。真实验证同时暴露并修复
了 Scoop `current` 目录链接：软件登记路径与进程报告路径现在都会解析为同一真实可执行文件身份，
不会再把已运行实例误判为未启动。

桌面端到端复核随后发现，先前仍在运行的开发 App 使用的是生产接入前的旧 Bundle，因此用户实际只能
看到 UI Automation 观察，不能据此证明 DirectWrite 写回。开发 Bundle 已重新构建为 8 个 Adapter，
DirectWrite 制品与清单摘要一致；既有 Probe Run 保留创建时的 Adapter Plan，在显式加入 DirectWrite
并使用新构建重连后，用户已确认编辑区实时翻译正常工作。

同一真实目标的级联菜单也完成了边界复核：一级和二级菜单均能由 GDI `ExtTextOutW` 与 USER32
`DrawTextW` 观察，二级菜单词条在干净、同版本的目标实例中取得明确的 `Matched + Replaced`。因此
级联菜单不是新的 Adapter 缺口。开发期替换 Runtime DLL 后，如果目标进程仍加载旧 DLL，旧译文可能
暂时可见但新观察不会继续回传；这种跨二进制版本升级仍要求重启目标软件，不能误判为二级菜单不支持。

## Next

- DirectWrite 桌面端到端验收与级联菜单边界均已闭环；下一轮从尚未覆盖的真实软件文字栈选择一个
  小型、可重复授权目标，先用现有 Adapter 矩阵测量缺口，再决定是否需要新增 Adapter。
- DirectWrite TextLayout 的底层生产接入已完成；WPF 等不经过公开 TextLayout 绘制入口的目标仍不在
  覆盖范围，不因加载 DirectWrite 模块而扩大支持声明。
- 下一轮继续用“真实软件缺口 → 一手入口证据 → 有界原型 → 可见写回”的顺序选择文字技术，不以
  Adapter 数量或框架名称驱动实现。
- [交互式取词 Seam](../runtime-capability-roadmap/slices/interactive-text-acquisition-seam.md)继续留在 Roadmap，
  当前不抢先实现热键、浮层、OCR 或翻译器。

## Progress

- 已重新确认产品主线是“探针发现原文 → Dictionary 提供译文 → Adapter 实时写回”，而不是通用观察
  数据平台。
- 已将采集能力与实时翻译能力分级；UIA、Console 等 observe-only 能力保留，但不计入实时翻译覆盖。
- 已用合成目标验证 DirectWrite 两种布局绘制入口，并在授权真实目标上完成零命中否决；没有把成功
  注入或加载模块误报成翻译支持。
- 已验证 Probe 译文编辑会产生下一代预览发布；桌面 Shell 单元测试 38/38 通过。
- 已完成[探针与词典创建流程收敛](slices/creation-flow-distillation.md)：取消后不再保留任务、Adapter
  或词典草稿，创建界面不再暴露内部词典 ID 与低频发布元数据；桌面 Playwright 53/53 通过。
- 已修复探针创建后首次连接失败导致 Modal 残留的问题，并把管理列表压缩为二态连接状态与悬浮技术
  详情；针对性 Playwright 3/3、组件与视觉合同 5/5、Capture 与 Desktop Shell 单元测试 54/54 通过。
- 已修复混合 Placement 的全有或全无启动：Native 激活失败时健康的 UIA Worker 仍会保持会话，失败
  能力不会冒充 Active；Isolated Worker Host 11/11、Desktop Runtime 13/13 常规测试通过。
- 已修复同一 Target Process 内多条 Native Hook 的全有或全无启动：不可用候选不再撤销健康 Adapter，
  Controller 会回传真实激活集合并由 Session 分别标记 Active / Failed。GDI 成功与 Qt 无模块失败的
  原生回归 1/1、Controller/Host 针对性回归 3/3、完整 Rust workspace 与架构检查通过；探针相关
  Playwright 5/5 通过。
- 已移除组件不兼容错误中“升级后重启目标软件”的无依据推断；针对性 Playwright 回归 1/1 与前端
  生产构建通过。
- 已修复“请求能力冒充已激活能力”：Session 只公开宿主确认 Active 的 Feature，Desktop Runtime 按
  实际 ACK 汇总，Probe 将其临时投影为“可直接替换 / 仅采集 / 无信号”；仅采集说明明确告知不会修改
  目标界面。Session 针对回归 1/1、Desktop Runtime 13/13、Desktop Shell 40/40、Probe Playwright
  10/10 与前端生产构建通过。
- 已把无 Dictionary 译文的条目明确标记为“词典未命中”；预览发布失败现在说明译文已经保存、但目标
  Runtime 未收到更新，不再伪装成停止失败。Desktop Shell 41/41、Probe Playwright 10/10 与格式检查
  通过。
- 已否决跨 Adapter 的“最终写回成功”回执：部分绘制入口没有返回值，有返回值也不能证明像素可见；
  运行诊断改为“替换决策已生成”并明确说明证据边界。WPF 外部翻译回退继续由交互式取词 Roadmap
  承接，当前不建设 Managed Agent。前端生产构建、针对性 Playwright 1/1 与 Impeccable 机械检测通过。
- 已验证 GDI+ Dictionary 在同一目标进程内发布第二代并立即替换，停止后恢复原文。
- 已比较 Qt、WinUI/MRT、WPF 与 Web 桌面壳入口；Qt 绘制链进入下一有界原型，其余候选保留在
  调研结论中，不并行扩张实现面。
- Qt Painter 纯逻辑 6/6 通过；既有 Native Host 常规合同 8/8 通过，另有 1 项无 Qt 模块 fail-open 通过；
  显式配置的 Qt 5 与 Qt 6 本地像素合同各 1/1 通过，并逐一覆盖 4 个核心 overload。原始本机日志只
  保留在本地证据目录。
- 首个授权 Qt 6 目标在注入前确认为静态链接 Qt；没有动态 Qt 模块或文字导出，因此按既定边界安全
  否决，没有把产品适配器降级成单软件、单版本签名扫描。
- 动态链接 Qt 6 Widgets 目标完成真实可见验收：首版与热更新译文各有 2 次
  `Matched + Replaced`，停用后恢复原文；正式 Bundle 复测捕获 40 次绘制并命中替换 2 次。
- Qt Painter 已加入正式 Bundle 构建与 Adapter 中文目录；完整仓库测试通过，本机截图与原始日志仍只
  位于忽略的本地证据目录。
- WinUI 3 合成宿主确认 `ResourceLoader.GetString` / 显式 C API 命中 `MrmLoadStringResource`，默认
  `x:Uid` 命中 `MrmLoadStringOrEmbeddedResourceByIndex`；三类文本均完成精确替换，停用后重新加载恢复
  原文。由于旧控件不能即时刷新或回滚，该能力未进入真实目标 smoke 与正式 Bundle。
- GTK 3 / Pango 纯逻辑 6/6 与无模块拒绝激活 1/1 通过；真实 Pango 属性范围、普通/全范围样式替换、
  局部样式放行和停用恢复均由像素合同确认。正式 Bundle 的后台注入复测中，第一代与第二代 Dictionary
  各命中替换 31 次，停止后恢复原文。
- GTK/Pango 后续候选已重新排序：`SetWindowTextW`、`LoadStringW`、Java agent 与常见静态 C++ 绘制库
  均不满足当前安全热更新合同；动态 Tk 8.6 只进入相对 GDI 的增量诊断，不因框架名称直接立项。
- Tk 隐藏双路宿主确认 GDI 与公开 Tk 路径都取得 3/3 个完整 source；两代中文译文均与原生 Tk 像素
  一致并可停用恢复。Tk 没有形成独立覆盖价值，实验实现已撤回且未进入 Bundle。
- SDL2_ttf 的稳定 UTF-8 C API 已复核，但其 Surface/Texture 缓存语义不能满足当前实时恢复合同；
  SDL3_ttf 的 `TTF_Text` 绘制链进入 Engine-aware Roadmap，等待真实目标触发，不先建设空置 Adapter。
- Web transport 与 DOM ownership 原型虽已通过，但产品适用性复核确认其依赖目标宿主主动开放接口；
  已停止自有目标 DOM 验收，不把 Host-assisted 能力计入通用实时翻译覆盖。
- 完成授权 WPF 真实目标验收：UIA 观察 72 条公开文本且健康；六条 Native 写回入口全部完成有效激活
  但零命中。WPF 当前归为“仅结构化观察”，后续由交互式取词与外部译文呈现承接。
- 完成下一真实目标一手资料筛选：Notepad++ 的 Scintilla 编辑区具有明确
  `IDWriteTextLayout → DrawTextLayout` 路径，进入唯一下一实机 Go/No-Go；VS Code 与 Godot 仅保留为
  Web/引擎文字栈边界，不并行立项。
- 完成 Scintilla DirectWrite 编辑区 Go：正式 Adapter 覆盖直接与 Compatible Bitmap 两类
  `DrawTextLayout`，首版和热更新代次均取得 `Matched + Replaced`，停止后恢复原文；复杂格式继续
  fail-open。正式 Bundle 真实复测首版命中 16 次、第二代命中 18 次。
- 修复 Windows 进程发现对目录链接两侧路径身份不一致的问题；通过 Scoop `current` 路径启动的真实
  目标已完成发现与 25 次稳定替换命中，控制器路径合同 6/6 通过。
- DirectWrite 纯逻辑 4/4、Native 绘制合同 1/1、Clippy 零警告、架构检查和完整 Rust workspace 测试
  均通过；正式 Bundle 构建成功。运行诊断文案的桌面生产构建与针对性 Playwright 1/1 也通过。
- 已纠正“独立 Runtime 通过即等于当前 App 可用”的错误结论：旧开发 App 的 Bundle 只有 7 个 Adapter，
  当前 Probe 也没有 DirectWrite。重新构建后根 Bundle 为 8 个 Adapter，DirectWrite 制品摘要验证通过，
  新桌面进程已启动；用户随后确认加入 DirectWrite 的 Probe 可正常实时工作。
- 已验证真实级联菜单的二级词条同时命中 GDI ExtTextOut 与 USER32 DrawText；其中 ExtTextOut 二级词条
  取得 2 次 `Matched + Replaced`。级联菜单不新增 Adapter；开发构建替换已加载 Runtime DLL 时需重启
  目标进程，避免旧译文残留被误认为当前探针仍在连接。

## References

- [产品契约](../../../PRODUCT.md)
- [领域语言](../../../CONTEXT.md)
- [Windows 软件支持分级](../runtime-capability-roadmap/references/windows-software-support-and-console-gap.md)
- [后续运行时能力 Roadmap](../runtime-capability-roadmap/index.md)
- [下一 Adapter 第一方资料评审](references/next-adapter-primary-source-review.md)
- [WinUI 3 / MRT Core 原生入口诊断](references/winui-mrt-primary-source-diagnostic.md)
- [GTK/Pango 实时写回候选评审](references/gtk-pango-adapter-primary-source-review.md)
- [GTK/Pango 后续入口评审](references/post-gtk-next-adapter-primary-source-review.md)
- [Tk 之后的实时文字入口复核](references/post-tk-runtime-seam-review.md)
- [WPF 真实目标缺口验收](slices/wpf-real-target-gap.md)
- [下一真实目标候选筛选](references/next-real-target-candidates.md)
