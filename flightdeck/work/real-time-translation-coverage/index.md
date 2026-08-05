# 通用实时翻译覆盖

Status: Open

## Goal

让用户完成稳定、可解释的端到端流程：选择软件，捕获原文，使用或编辑 Dictionary，并由真实
`TextReplace` Adapter 把译文立即写回目标界面。通用性按可复用文字技术扩展，不以“支持所有软件”
或成功注入代替真实翻译结果。

## Current

现有 GDI、USER32 DrawText 与 GDI+ Adapter 已能覆盖命中这些调用的经典 Windows 界面，并在一个授权
桌面工具的部分界面得到写回证据。Console 与 UI Automation 目前只有采集价值，不能作为实时翻译
成功。Direct2D `DrawText` 已通过合成写回，但在当前授权目标中零命中，因此尚未进入生产 Bundle。

DirectWrite TextLayout 的两种常见绘制入口也完成过窄原型：合成目标可以观察、替换，并让同一布局在
停用后恢复、重新启用后再次替换；复杂多段格式会安全放行。但 AE 与 WPF 授权测试均为零命中，所以
实验实现已从工作区撤回，只保留否决结论，不进入生产 Bundle，也不计入软件支持范围。

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

`runtime.component_incompatible` 的界面文案不再猜测“刚升级过”或要求重启目标软件，而是明确说明
所选探针技术不适用于当前软件，并引导用户调整技术。HandBrake 类 WPF 目标当前仍只能归为“仅采集
原文”，不能因 UIA 连接成功显示成实时翻译支持。

Probe 连接状态现已使用实际会话 ACK，而不是用户勾选的 Adapter 能力推断。桌面 Runtime 只汇总宿主
真正确认 Active 的 Feature；当前连接据此显示“可直接替换 / 仅采集 / 无信号”。“仅采集”会明确说明
目标软件界面不会被修改；能力状态只属于本次连接，释放连接或重启后不会写进可恢复任务配置。

条目状态也不再把 Dictionary 未命中含糊显示为“待翻译”，而是明确显示“词典未命中”。用户在 Probe
内保存译文后，如果下一代预览没有成功发送到目标 Runtime，界面会说明“词典已保存，但目标界面可能
仍显示旧译文”，不再误报成停止失败。现有 Runtime Trace 可以证明替换决策已产生，但不能证明 Adapter
最终改动了目标控件或像素；在增加窄的最终应用回执前，不展示虚假的“写回成功 / 写回失败”条目状态。

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
下一步必须在 Managed WPF Agent 与 UIA 驱动的外部应用模型之间先做产品选择。

## Next

- 暂停继续枚举 Native Hook。连接级能力、条目级“词典未命中”和预览发布失败均已明确；剩余缺口是
  Adapter 最终应用结果的窄回执合同。现有 Runtime Trace 只能证明替换决策，不能冒充目标控件或像素
  已被修改。UIA/坐标与 OCR 的共用底层已路由到
  [交互式取词 Seam](../runtime-capability-roadmap/slices/interactive-text-acquisition-seam.md)，当前不抢先实现
  热键、浮层或翻译器。
- [WPF 真实目标缺口验收](slices/wpf-real-target-gap.md)已完成：结构化观察有收益，但六条 Native
  写回路径全部零命中；不按已加载模块或成功注入虚报支持。
- 下一步先决定 WPF 的 Apply Model：若坚持原位翻译，只能进入独立 Managed WPF Agent 的有界原型；
  若优先覆盖率，则评估 UIA 驱动的翻译面板/最小 Overlay，并明确不是原位写回。
- [Tk 文字绘制链对照诊断](slices/tk-text-draw-path-diagnostic.md)已完成并作 No-Go：正式 GDI 与公开
  Tk 路径均取得 3/3 个完整词条，两代中文译文都与 Tk 原生像素一致，停用后恢复原文；实验实现已撤回。
- [Tk 之后的实时文字入口复核](references/post-tk-runtime-seam-review.md)已完成：SDL2_ttf 只有 Surface
  创建时替换，无法保证缓存文字热更新和停用恢复；SDL3_ttf 具备真正绘制时 Text API，但缺少代表性授权
  Windows 目标。两者都不直接加入当前 Bundle，SDL3 与 Web/托管运行时入口转入 Roadmap。
- Web/CDP 已从通用 Adapter 候选降为后续宿主协作集成：普通已运行的 Electron/WebView2 软件通常没有
  GlyphShift 可安全连接的 endpoint，自有调试宿主成功不能外推第三方覆盖。
- 当前不再以 Adapter 数量为目标枚举包装层；下一步先对一个用户真实需要翻译、且尚未被现有路径覆盖的
  授权软件做缺口验收。只有命中稳定的绘制时文字入口并证明可见增量，才新增生产 Adapter。

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
- 已移除组件不兼容错误中“升级后重启目标软件”的无依据推断；针对性 Playwright 回归 1/1 与前端
  生产构建通过。
- 已修复“请求能力冒充已激活能力”：Session 只公开宿主确认 Active 的 Feature，Desktop Runtime 按
  实际 ACK 汇总，Probe 将其临时投影为“可直接替换 / 仅采集 / 无信号”；仅采集说明明确告知不会修改
  目标界面。Session 针对回归 1/1、Desktop Runtime 13/13、Desktop Shell 40/40、Probe Playwright
  10/10 与前端生产构建通过。
- 已把无 Dictionary 译文的条目明确标记为“词典未命中”；预览发布失败现在说明译文已经保存、但目标
  Runtime 未收到更新，不再伪装成停止失败。Desktop Shell 41/41、Probe Playwright 10/10 与格式检查
  通过；最终 Adapter 应用结果仍等待独立的窄回执合同。
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
  但零命中。WPF 当前归为“仅结构化观察”，Managed Agent 或外部 Apply Model 需另行选择。

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
