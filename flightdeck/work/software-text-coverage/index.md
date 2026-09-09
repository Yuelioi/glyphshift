# Houdini 与其他软件文字覆盖

Status: Open

## Goal

补齐真实软件遗漏的完整文字入口，交付可跨软件复用的适配器分支，并验证原位替换、字典更新、刷新和停止恢复。

## Current

最新目标为 PotPlayer 外挂 SRT。用户已验证：保持工作流运行，重新打开同一字幕后译文可见；先启动工作流再打开字幕也可翻译。因此缓存生成时机是当前优先方向，不以两个 GDI 入口共同命中判定冲突。尚未验证运行中新增译文无需重载即可更新，实时翻译目标未完成。未改适配器代码；原始采样只在本地忽略目录。

用户同时要求简化清空字典：改用现有确认弹窗，移除输入字典名称，保留取消、运行任务时的编辑保护，以及清空后保存才写文件的行为。已同步中英文案、使用文档和既有页面测试。

较早的目标交接如下，下一次优先处理当前 PotPlayer 请求。

Houdini 的工具栏短标签和悬停提示正文仍有漏采集。目标加载了 Qt Painter / Qt Quick 组件，Qt 库提供 drawText、QTextLayout、QTextItem、QStaticText 等入口，但尚未证明遗漏文字经过哪条路径。两次静态采样无绘制调用，截图返回无效绘制区域；Windows 自动化插件初始化报错。已请求用户恢复正在测试的窗口并显示提示，尚未取得可复现红例，不能将入口猜测当作结论。

此前四目标初查确认：Vovious 使用静态 JUCE，Blender 使用自绘界面及 Python/BLF，Fluffy Mod Manager 的完整字符串入口尚未定位。CatSystem2 经典分支已在上一项 Work 实现且用户确认可用，保留实验标识；经典历史页及更多作品未验收。

## Next

用户已取消 PotPlayer 缓存刷新任务，不继续修改或测试其字幕实时热更新。当前产品工作转至工作流 Work 的 SRT 提前导入及可扩展字典导入导出。若之后重新授权该目标，再使用原先的两代译文与停止恢复验收。

Houdini、Vovious、Blender 与 Mod Manager 的未完成覆盖继续保留，完成当前 PotPlayer 对照后再恢复。

## References

- [四目标初查](../adapter-coverage-and-x86/slices/four-target-survey.md)
- [13 项字符串分支覆盖矩阵](../adapter-coverage-and-x86/references/string-branch-coverage.md)
- [CatSystem2 经典分支与验证](../adapter-coverage-and-x86/slices/catsystem2-classic.md)
