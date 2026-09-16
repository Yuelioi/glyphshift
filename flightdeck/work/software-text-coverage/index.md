# Houdini 与其他软件文字覆盖

Status: Open

## Goal

补齐真实软件遗漏的完整文字入口，交付可跨软件复用的适配器分支，并验证原位替换、字典更新、刷新和停止恢复。

## Current

Houdini 22.0.429 当前组合两条通用 Qt 能力。`Quick Shapes`、`Helix`、`Spiral` 继续经过 Qt Painter 已覆盖的
QPointF `drawText`；新增 `windows.qt.text-document` 则覆盖真实 hover 提示经 `QTextDocument::setHtml` 建模的
纯文本与单可见文本节点 HTML。production real-host 已把 `The currently active Desktop` 从 Qt Painter 的 0 hit
红例变成 Text Document 首代 2 hit / 第二代 1 hit，并完成中文像素与停用后原文验证；Tube 与 Spiral 的富 help
正文也分别完成首次生成时的可见中文验证。Tube 第二代 publication 有 replacement hit，但像素仍停留在首代，
停用后再次 hover 也继续显示译文；生命周期探针证明 help 卡片首次 setHtml 后不再 clone / destroy / setHtml，普通
Windows redraw 也不能失效其下游缓存。新 Adapter 精确限制为 Qt 6.8.3 MSVC x64 全局 namespace，复杂 / 多段 HTML
fail-open，没有 Houdini 专属 Hook。当前因此继续标记部分支持，并推荐在启动目标、首次生成 shelf / help 前启用
Workflow。详细证据见
[Houdini](references/software/houdini.md)。

Houdini Network Editor 右下角的 `Add / Edit / Go / View / Tools / Layout / Help` 也已闭环：真实绘制继续使用
现有 `QPainter::drawText(QRectF, int, QString const&, QRectF*)`，只是源字符串带首尾布局空格。Capture 目录使用
`SourceTextPolicy::key()` 去边缘空白，而实时 Runtime 此前没有应用同一 key，造成“表里已有译文、像素仍英文”。
现在通用 target Runtime 保留精确键优先，失败后按 source-policy key 回退并保留原调用首尾空白；定向回归、
production diagnostics 的 matched/replaced 与真实中文像素均通过，0 丢弃。该修复不依赖 Houdini 或 Qt 特判。
同一区域剩余文字已经找到新的通用 seam：`Non-Commercial Edition`、`Objects` 与组合正文
`Empty Network\nPress Tab to Add Nodes` 都真实进入动态 `libCV.dll` 的 `CV_PaintBuffer::textWrappedInBox`。
一次性动态替换已经证明该入口能直接产生中文像素，并保留目标自己的字体与布局。现已新增 x64-only 的
`windows.sidefx.cv-paint-buffer-text`，精确 gate 动态模块与已验证 MSVC 导出，缺能力 fail-closed；合成 ABI
合同覆盖首代、第二代与停用恢复，缺模块激活合同也通过。最新同步 Runtime Bundle 验证为 16 个 Adapter，
活动包全仓测试通过。更新恢复后已通过 `scripts/review-app.ps1` 重新同步构建并验证 16-Adapter Bundle，
重启目标后 revision 6 Workflow 激活成功，已无 `runtime.target_restart_required`。production SideFX CV
已采集三个目标字符串，采集丢弃为 0；`Non-Commercial Edition` 与 `Objects` 首代中文像素通过。
组合中心提示原先没有译文，已补入中文。用户随后对最新同步 review App 手动验收，确认当前界面已完全汉化，
并授权提交、打 tag 与推送。本轮界面覆盖按用户验收通过记录；第二代更新与停用恢复未单独完成自动验收，
不据此消除已有 retained cache 边界。

Autograph 2026 已完成：Qt Quick 适配器精确加入 Qt 6.11.1 MSVC x64，retained-object 合同通过，最新生产 Runtime 在实机捕获 47 条唯一文字且 0 丢弃，并完成 `File` 两代热更新与停止恢复。Autograph 继续复用通用 Qt Quick 适配器，不新增专属适配器。详细证据见 [Autograph](references/software/autograph.md)。

Silhouette 2026.0 主界面标准区域已通过：Qt 6.5.4 Widgets 使用 `isl0` C++ 命名空间，Qt Painter 已加入精确 Qt 6/x64 ABI，干净 Runtime 曾捕获 56 条唯一文字且 `droppedObservations=0`，并完成热更新与停止恢复。但用户随后确认节点图仍漏 `Unproject`、`Right`、`Left`、`Time Shift` 和节点 `Output`。静态导入检查发现 Silhouette 主程序使用 QGraphicsScene，并直接导入两个 Qt Painter 尚未覆盖的标准重载；现已把这两个重载作为**通用 Qt Painter 可选能力**补入，包级回归 5/5 通过，最新 review build 已生成，等待目标进程重启后的实机验收。详细证据见 [Silhouette](references/software/silhouette.md)。软件调查结果统一放在 `references/software/`，一款软件一份 Markdown；每份保存当前决定、已验证事实、未解决问题和恢复时的第一步。

JUCE 6.1.6 的首轮静态可行性已经收口为 **No-Go**。Vovious 的版本 / RTTI gate 成立，且
`addCurtailedLineOfText` 与 `addFittedText` 各有一个唯一语义候选；但 canonical `addJustifiedText` 与
`addLineOfText` 薄包装均没有可证明入口。目标中发现的 CR/LF 分行与 glyph-loop 代码属于调用者本地
`GlyphArrangement` 的内联 / 组合路径，不能按通用 JUCE seam 身份 Hook。按预设 fail-closed 门槛，本轮在
静态阶段停止，没有继续 attach / observation Hook，也没有新增生产 crate、Catalog 或 Bundle。当前 JUCE 已
迁移到 `ShapedText`，后续版本若重开仍必须建立独立精确 profile。详见
[JUCE 选型](references/juce-adapter-selection.md) 与 [Vovious](references/software/vovious.md)。

Qt 翻译服务原型已从选型推进到真实软件闭环。qBittorrent 5.2.3 的正式 Windows release 实际载荷没有独立 Qt
DLL，因此不适合作为当前“动态模块 + 精确导出”gate 的代表目标；KeePassXC 2.7.12 虽提供动态 Qt 5.15.18，
但 release 会主动收紧进程 DACL，只允许 limited-query/terminate 等最小权限并拒绝 Glyphshift 所需的远程注入权限，
因此也没有通过代表目标门槛，且没有加入任何绕过或软件特判。最终代表目标改为 **Wireshark 4.6.8 / Qt 6.10.3**。
新增通用 `windows.qt.translation-service` 以精确 MSVC x64 `QCoreApplication::translate` 导出为 seam；默认生产构建
只接受 Qt 6.10.3，Qt 5.15.18 仅由显式 `research-qt5` feature 保留。共享 text-host V2 已把 `context/disambiguation/n` 从 native hook 贯通到
Runtime capture、probe 与 AI 提示层；这些字段保留为观测证据，不进入 Dictionary package 或 Workflow 的词条身份。
Dictionary 已恢复并保持 schema v3 / source-only，同 source 的非复数 Qt 调用共享一条译文，重复 source 继续禁止。
最新 Wireshark 实机用 622 条唯一 V2 observation（613 条唯一 source、capture 丢弃 0）证明同一个 `Cancel` 可在
`SearchFrame` 与 `WiresharkMainWindow` 两个 context 下被分别观测；早期 contextual publication 实验保留为 Qt seam
语义证据，但不再代表当前 Dictionary 模型。临时接口类型过滤触发 `WelcomePage` 的
`%n interface(s) shown, %1 hidden`，真实记录 `plural_n=7`；plural 继续 observe-only / fail-open。主窗口原生
标题同步完成三代可见刷新与停用恢复。真实 Qt 6.10.3 module 合同又用 Wireshark 实采的 `Profile: %1` 验证 placeholder
兼容替换与缺失 `%1` 时 fail-open；旧 source-only host 也已收紧为只有无 context、无 disambiguation 且 `n == -1`
才能 replacement。Qt 6.10.3 refresh 继续只复用官方 `QEvent::LanguageChange` 异步传播；Qt 5.15.18 因外部 bridge
尚无已验证的 queue-owned `QEvent` 分配 / 所有权 ABI，明确保持 future-call-only。Qt 6.10.3 profile 已完成生产晋级：
默认 native 构建只接受 Qt 6.10.3，Qt 5.15.18 改为显式 `research-qt5` Cargo feature，生产 Bundle 不会误激活 Qt 5。
`windows.qt.translation-service` 已接入 x64 Runtime Bundle 与产品展示目录，Release Bundle 真实 loader 验证通过（全 Bundle
17 个唯一 Adapter ID）；随后直接使用 Bundle 内 versioned Runtime / Qt translation DLL 重跑 Wireshark，仍取得 622 条
V2 observation、capture 丢弃 0、`plural_n=7`、两个 `Cancel` context 独立命中、三代标题刷新、停用恢复与正常退出。
随后按仓库规定运行 `scripts/review-app.ps1 -BuildOnly`，同步桌面 Release 壳与 Runtime Bundle，两次真实 loader 校验均报告 17 adapters，桌面构建通过。
详见
[Qt Context/Plural/Refresh 结论](references/qt-translation-context-plural-refresh.md)、
[Qt Translator 第一方复核](references/qt-translator-primary-source-review.md)、
[Wireshark](references/software/wireshark.md) 与 [qBittorrent](references/software/qbittorrent.md)。

Qt 5 研究已经新增 **QGIS LTR 3.44.12 / Qt 5.15.13 MSVC x64** 真实目标。首轮用户实机验证显示大部分菜单可翻，
但已创建的二/三级菜单 action 与 tooltip 存在英文缓存。目标 Qt5Core 的 allocator / deleting-destructor 证据使
`research-qt5` 可以安全 post queue-owned `QEvent::LanguageChange`，不过真实 QGIS 进一步证明该事件只会重译应用
实际实现了 retranslate 的控件；`New Virtual Layer` 这类已缓存 QAction tooltip 不再重新进入 `translate()`。
现在通用 Qt Painter 路径额外 Hook 标准 `QToolTip::showText`，对单可见文本节点 rich text 在 hover 显示阶段重新决策；
它直接使用普通 source-only Dictionary，因此 Qt translation context 在已缓存 QAction / tooltip 丢失后也不影响显示时 lookup。真实 QGIS
已把 `New Virtual Layer` 稳定显示为 `新建虚拟图层`；严格隔离移除 translation-service 后仍通过，证明不依赖启动时机。
随后用户实机确认 Vector 二/三级菜单也已完整翻译；这些 QAction 的最终可见文字由现有 Qt Painter 绘制时替换闭环。
同时发现 Probe AI 旧路径会直接跳过所有带 Qt context 的 translation-service observation。现在 Probe/AI 会保留
context/disambiguation 作为语义提示，但候选与 Dictionary writeback 都按 source 去重；同 source 的多个 context 只写回一条词条，
plural 仍保持 observe-only。真实 QGIS 已证明 contextual observation 能进入 AI 路径，而不会再迫使 Dictionary 扩展身份。
最新 `scripts/review-app.ps1 -ResearchQt5` 同步 build 通过
17-adapter verifier；默认 production 仍只接受 Qt 6.10.3，Qt 5.15.13 仍是 research-only，Qt 5.15.18 继续
future-call-only。详见 [QGIS](references/software/qgis.md)。

TouchDesigner 的 Slug 集成已找到二进制标记，但按公开符号定位的字符串入口未通过，尚无可见替换。PotPlayer 的重载字幕使用办法已由用户确认，实时缓存适配已取消。其他软件调查分别保留，不自动继续测试。

## Next

Qt 6.10.3 translation-service 的生产晋级已经完成。QGIS / Qt 5.15.13 的 toolbar tooltip、二/三级菜单以及
context evidence 进入 Probe/AI 提示层已闭环；Dictionary 已恢复 source-only。当前继续完成第二代更新、停用恢复、
per-launch capture drop delta，以及可自然取得的 V2 context / plural / placeholder 证据。Qt 5.15.13
在证据完成前始终只通过 `research-qt5` 暴露，不能进入默认生产 Bundle。之后再单独处理 Qt 5.15.18；它不能因为
5.15.13 的 allocator 证据而自动获得 immediate refresh，仍需自己的 queue-owned `QEvent` ABI 与真实目标证明。
立即恢复时先读
[Qt Context/Plural/Refresh 结论](references/qt-translation-context-plural-refresh.md) 与
[QGIS](references/software/qgis.md)。

## 软件记录

- [软件兼容状态总表](../../knowledge/rendering/software-compatibility.md)

- [MTool](references/software/mtool.md)（引擎适配参考，后续任务转游戏 Work）

- [TouchDesigner](references/software/touchdesigner.md)
- [PotPlayer](references/software/potplayer.md)
- [Houdini](references/software/houdini.md)
- [Vovious](references/software/vovious.md)
- [qBittorrent](references/software/qbittorrent.md)
- [Wireshark](references/software/wireshark.md)
- [QGIS](references/software/qgis.md)
- [Blender](references/software/blender.md)
- [Fluffy Mod Manager](references/software/fluffy-mod-manager.md)
- [Autograph](references/software/autograph.md)
- [Silhouette](references/software/silhouette.md)
- [ZBrush](references/software/zbrush.md)

## References

- [四目标初查](../adapter-coverage-and-x86/slices/four-target-survey.md)
- [13 项字符串分支覆盖矩阵](../adapter-coverage-and-x86/references/string-branch-coverage.md)
- [CatSystem2 经典分支与验证](../adapter-coverage-and-x86/slices/catsystem2-classic.md)
- [Qt translation context / plural / refresh 结论](references/qt-translation-context-plural-refresh.md)
- [Qt Translator / QCoreApplication::translate 第一方资料复核](references/qt-translator-primary-source-review.md)
