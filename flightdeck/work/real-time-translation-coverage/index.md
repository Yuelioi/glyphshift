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
许可证、主页与标签继续由词典设置维护。目标软件未启动时任务仍保存为“待连接”，不会把连接失败
误报成创建失败；之后显式重连仍显示具体原因。

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

## Next

- [WinUI/MRT 文字路径诊断](slices/winui-mrt-path-diagnostic.md)已完成并对生产实时 Adapter 作 No-Go；
  不建设 CLR Profiler、PRI 修改或按软件写特例。
- [GTK 3 Pango 绘制时写回](slices/gtk3-pango-draw-writeback.md)已完成并进入正式 Bundle；GTK 4 维持
  No-Go。下一步重新按“可取得原文、同链安全写回、跨软件复用、停用恢复”排序剩余候选，不默认扩张到
  CLR Profiler 或 Web 调试连接。

## Progress

- 已重新确认产品主线是“探针发现原文 → Dictionary 提供译文 → Adapter 实时写回”，而不是通用观察
  数据平台。
- 已将采集能力与实时翻译能力分级；UIA、Console 等 observe-only 能力保留，但不计入实时翻译覆盖。
- 已用合成目标验证 DirectWrite 两种布局绘制入口，并在授权真实目标上完成零命中否决；没有把成功
  注入或加载模块误报成翻译支持。
- 已验证 Probe 译文编辑会产生下一代预览发布；桌面 Shell 单元测试 38/38 通过。
- 已完成[探针与词典创建流程收敛](slices/creation-flow-distillation.md)：取消后不再保留任务、Adapter
  或词典草稿，创建界面不再暴露内部词典 ID 与低频发布元数据；桌面 Playwright 53/53 通过。
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

## References

- [产品契约](../../../PRODUCT.md)
- [领域语言](../../../CONTEXT.md)
- [Windows 软件支持分级](../runtime-capability-roadmap/references/windows-software-support-and-console-gap.md)
- [后续运行时能力 Roadmap](../runtime-capability-roadmap/index.md)
- [下一 Adapter 第一方资料评审](references/next-adapter-primary-source-review.md)
- [WinUI 3 / MRT Core 原生入口诊断](references/winui-mrt-primary-source-diagnostic.md)
- [GTK/Pango 实时写回候选评审](references/gtk-pango-adapter-primary-source-review.md)
