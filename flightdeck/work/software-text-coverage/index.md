# Houdini 与其他软件文字覆盖

Status: Open

## Goal

补齐真实软件遗漏的完整文字入口，交付可跨软件复用的适配器分支，并验证原位替换、字典更新、刷新和停止恢复。

## Current

Autograph 2026 已完成：Qt Quick 适配器精确加入 Qt 6.11.1 MSVC x64，retained-object 合同通过，最新生产 Runtime 在实机捕获 47 条唯一文字且 0 丢弃，并完成 `File` 两代热更新与停止恢复。Autograph 继续复用通用 Qt Quick 适配器，不新增专属适配器。详细证据见 [Autograph](references/software/autograph.md)。

Silhouette 2026.0 主界面标准区域已通过：Qt 6.5.4 Widgets 使用 `isl0` C++ 命名空间，Qt Painter 已加入精确 Qt 6/x64 ABI，干净 Runtime 曾捕获 56 条唯一文字且 `droppedObservations=0`，并完成热更新与停止恢复。但用户随后确认节点图仍漏 `Unproject`、`Right`、`Left`、`Time Shift` 和节点 `Output`。静态导入检查发现 Silhouette 主程序使用 QGraphicsScene，并直接导入两个 Qt Painter 尚未覆盖的标准重载；现已把这两个重载作为**通用 Qt Painter 可选能力**补入，包级回归 5/5 通过，最新 review build 已生成，等待目标进程重启后的实机验收。详细证据见 [Silhouette](references/software/silhouette.md)。软件调查结果统一放在 `references/software/`，一款软件一份 Markdown；每份保存当前决定、已验证事实、未解决问题和恢复时的第一步。

TouchDesigner 的 Slug 集成已找到二进制标记，但按公开符号定位的字符串入口未通过，尚无可见替换。PotPlayer 的重载字幕使用办法已由用户确认，实时缓存适配已取消。其他软件调查分别保留，不自动继续测试。

## Next

Silhouette 2026.0 当前 Next 是重启真实目标，让最新 Qt Painter DLL 生效，并针对节点图的 `Unproject / Right / Left / Time Shift / Output` 做捕获与可见替换验收；如果新增 QPainter 重载仍未覆盖，再沿 QGraphicsScene / QTextLayout 的通用完整字符串入口继续。Autograph 2026 当前没有未完成的适配任务。后续 Qt 版本或不同命名空间仍按单版本、单 ABI 重新验收，不扩大白名单。当前阶段禁止软件专属适配器；ZBrush 只继续研究能抽象为通用适配器的完整字符串入口。所有适配仍需完整正文、两代译文、字体和停止恢复验收，遵守 [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)。

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
