# Houdini 与其他软件文字覆盖

Status: Open

## Goal

补齐真实软件遗漏的完整文字入口，交付可跨软件复用的适配器分支，并验证原位替换、字典更新、刷新和停止恢复。

## Current

用户要求本轮先提交，Houdini 与其他软件作为下一项 Work；本次只整理交接，不继续操作目标软件。

Houdini 的工具栏短标签和悬停提示正文仍有漏采集。目标加载了 Qt Painter / Qt Quick 组件，Qt 库提供 drawText、QTextLayout、QTextItem、QStaticText 等入口，但尚未证明遗漏文字经过哪条路径。两次静态采样无绘制调用，截图返回无效绘制区域；Windows 自动化插件初始化报错。已请求用户恢复正在测试的窗口并显示提示，尚未取得可复现红例，不能将入口猜测当作结论。

此前四目标初查确认：Vovious 使用静态 JUCE，Blender 使用自绘界面及 Python/BLF，Fluffy Mod Manager 的完整字符串入口尚未定位。CatSystem2 经典分支已在上一项 Work 实现且用户确认可用，保留实验标识；经典历史页及更多作品未验收。

## Next

恢复工作时先重新确认授权目标及窗口状态，复现 Houdini 圈选的工具栏标签与提示正文。在实际绘制期间比较现有 drawText 和布局/静态文字入口，形成能对遗漏文字判红的本地采样，再按证据补通用分支。遵守 [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)。

Houdini 路线明确后，按复用价值处理 Vovious/JUCE、Blender 桥接与 Mod Manager；现有适配器的分支审计作为并列待办，不以单个软件通过代替全分支验收。

## References

- [四目标初查](../adapter-coverage-and-x86/slices/four-target-survey.md)
- [13 项字符串分支覆盖矩阵](../adapter-coverage-and-x86/references/string-branch-coverage.md)
- [CatSystem2 经典分支与验证](../adapter-coverage-and-x86/slices/catsystem2-classic.md)
