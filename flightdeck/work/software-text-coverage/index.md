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

TouchDesigner 的 Slug 集成已找到二进制标记，但按公开符号定位的字符串入口未通过，尚无可见替换。PotPlayer 的重载字幕使用办法已由用户确认，实时缓存适配已取消。其他软件调查分别保留，不自动继续测试。

## Next

用户已确认当前界面完全汉化，本轮通用适配器与 Runtime 修复随 `v0.5.0` 提交、打 tag、推送。
保持停止自动实机测试；生命周期补验仅在后续任务明确要求时开展。
若后续改善富 help 热更新 / 停止恢复，只研究可跨 Qt 软件复用的
QTextDocument 下游 cache 失效 seam；标准 Windows redraw 已证实不足，不继续枚举 Houdini 私有 Help / Pluto /
OPUI 入口。
Silhouette 的节点图复验保留在其软件记录中，不与本轮混做。所有适配仍按
[文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md) 区分“文字命中”“代次更新”
“像素刷新”和“停止恢复”。

## 软件记录

- [软件兼容状态总表](../../knowledge/rendering/software-compatibility.md)

- [MTool](references/software/mtool.md)（引擎适配参考，后续任务转游戏 Work）

- [TouchDesigner](references/software/touchdesigner.md)
- [PotPlayer](references/software/potplayer.md)
- [Houdini](references/software/houdini.md)
- [Vovious](references/software/vovious.md)
- [Blender](references/software/blender.md)
- [Fluffy Mod Manager](references/software/fluffy-mod-manager.md)
- [Autograph](references/software/autograph.md)
- [Silhouette](references/software/silhouette.md)
- [ZBrush](references/software/zbrush.md)

## References

- [四目标初查](../adapter-coverage-and-x86/slices/four-target-survey.md)
- [13 项字符串分支覆盖矩阵](../adapter-coverage-and-x86/references/string-branch-coverage.md)
- [CatSystem2 经典分支与验证](../adapter-coverage-and-x86/slices/catsystem2-classic.md)
