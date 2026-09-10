# Houdini 与其他软件文字覆盖

Status: Open

## Goal

补齐真实软件遗漏的完整文字入口，交付可跨软件复用的适配器分支，并验证原位替换、字典更新、刷新和停止恢复。

## Current

用户决定保留 TouchDesigner 首轮研究，暂不继续。软件调查结果统一放在 `references/software/`，一款软件一份 Markdown；每份保存当前决定、已验证事实、未解决问题和恢复时的第一步。历史跨软件初查保留原文，通过链接追溯，避免把旧初查当作最新验收结果。

TouchDesigner 的 Slug 集成已找到二进制标记，但按公开符号定位的字符串入口未通过，尚无可见替换。PotPlayer 的重载字幕使用办法已由用户确认，实时缓存适配已取消。其他软件调查分别保留，不自动继续测试。

## Next

等待用户选定后续目标，再从对应软件记录恢复工作；仅重新授权 TouchDesigner 时才继续其静态库入口和 ABI 调查。所有适配仍需完整正文、两代译文、字体和停止恢复验收，遵守 [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)。

## 软件记录

- [MTool](references/software/mtool.md)（引擎适配参考，后续任务转游戏 Work）

- [TouchDesigner](references/software/touchdesigner.md)
- [PotPlayer](references/software/potplayer.md)
- [Houdini](references/software/houdini.md)
- [Vovious](references/software/vovious.md)
- [Blender](references/software/blender.md)
- [Fluffy Mod Manager](references/software/fluffy-mod-manager.md)

## References

- [四目标初查](../adapter-coverage-and-x86/slices/four-target-survey.md)
- [13 项字符串分支覆盖矩阵](../adapter-coverage-and-x86/references/string-branch-coverage.md)
- [CatSystem2 经典分支与验证](../adapter-coverage-and-x86/slices/catsystem2-classic.md)
