# 星露谷物语文本拦截可行性研究

本页保留初次研究的入口证据。用户随后明确要求使用可跨游戏复用的 Adapter，排除 SMAPI/游戏 Mod 路线；以下 SMAPI 建议不作为当前实施方案。当前结论见 [MonoGame 适配方案](monogame-adapter-design.md)。

日期：2026-09-07。范围：Windows 游戏文本适配；联网核对 SMAPI、Harmony、MonoGame 与 Content Patcher 的维护者文档/源码。本页不记录本机证据，不代表已完成注入、运行时捕获或文字替换实测。

## 结论

可以设计游戏专用适配器，最有依据的路线是 **SMAPI 承载托管插件，结合资源层编辑与 Harmony 文本入口拦截**。这是一项新适配能力，不能仅凭窗口可见或框架名称就宣称现有适配器能捕获当前画面。

SMAPI 明确在游戏启动时加载代码模组。因此，对于已正常启动、尚未加载该桥接插件的游戏，SMAPI 路线需要重新通过相应启动入口运行；文档没有提供向已运行原版进程热加载 SMAPI 的承诺。[SMAPI 源码仓库说明](https://github.com/Pathoschild/SMAPI)

## 第一方证据与技术含义

| 层级 | 已核实事实 | 对适配器的意义 |
| --- | --- | --- |
| 插件宿主 | SMAPI 官网当前提供 4.5.2，要求桌面版游戏 1.6.14 或更新版本，并列出 Windows 支持。 | 存在维护中的插件入口；具体目标版本仍须本地核对。 |
| 托管函数补丁 | Harmony Prefix 可读取、修改方法参数，也可跳过原方法。 | 可以先做只读文本探针，再实现明确命中的字符串替换。 |
| MonoGame 文本 | `SpriteBatch.DrawString` 接收 `string` 或 `StringBuilder`，并有字体、位置、缩放、颜色等参数。 | 对实际经过这些重载的调用，可以在文字变成绘制数据前读取文本及布局信息。 |
| 游戏专用文本 | SMAPI 的 `SpriteTextFacade` 调用 `StardewValley.BellsAndWhistles.SpriteText.drawString` 等方法，参数含字符串、位置、宽高及字符显示进度。 | `SpriteText` 应成为独立探测入口，不能假定全部文本都由一个 MonoGame 重载覆盖。 |
| 资源内容 | SMAPI `AssetRequestedEventArgs.Edit` 支持资源加载后编辑；Content Patcher `EditData` 支持修改数据资源条目/字段，文档直接以 NPC 对话资源为例。 | 对已知字符串资源，可按资源名与键替换，让游戏后续排版使用译文。 |

来源：[SMAPI 官网](https://smapi.io/)、[Harmony Prefix](https://harmony.pardeike.net/articles/patching-prefix.html)、[MonoGame SpriteBatch API](https://docs.monogame.net/api/Microsoft.Xna.Framework.Graphics.SpriteBatch.html)、[SMAPI SpriteText 兼容源码](https://raw.githubusercontent.com/Pathoschild/SMAPI/develop/src/SMAPI/Framework/ModLoading/Rewriters/StardewValley_1_6/SpriteTextFacade.cs)、[SMAPI 资源事件源码](https://raw.githubusercontent.com/Pathoschild/SMAPI/develop/src/SMAPI/Events/AssetRequestedEventArgs.cs)、[Content Patcher EditData 文档](https://raw.githubusercontent.com/Pathoschild/StardewMods/develop/ContentPatcher/docs/author-guide/action-editdata.md)。版本号仅反映研究时网站内容。

`SpriteTextFacade` 是 SMAPI 自身的兼容层，源码明确要求模组不要直接引用它。应核对目标游戏实际的 `SpriteText` 方法签名；该兼容源码只能证明相应入口存在及历史签名发生变化，不能证明目标画面一定经过这些方法。[源码](https://raw.githubusercontent.com/Pathoschild/SMAPI/develop/src/SMAPI/Framework/ModLoading/Rewriters/StardewValley_1_6/SpriteTextFacade.cs)

## 工程判断与建议

以下是基于上述机制的工程推断，尚未实测：

1. 优先制作 SMAPI 只读桥接探针，同时观察 `SpriteText` 和 `SpriteBatch.DrawString` 的实际入口，按菜单、对话、物品提示分别统计命中。只列出程序集和方法不算捕获成功。
2. 资源层适合稳定的内置文本与对话，但资源被加载不等于文字当前可见。渲染入口适合动态文本和覆盖率观测，两层职责应分别记录。
3. 不应在逐帧绘制回调里同步等待翻译服务。探针应限流去重，翻译结果走缓存；未命中时保留原文。
4. 原位替换还须验证字体字形、测量宽度、换行、逐字显示与语言切换。仅改变 DrawString 参数可能晚于游戏的排版计算，因此捕获成功不能直接推导出布局正确。
5. 应优先使用版本明确的游戏入口并检测补丁是否生效。Harmony 官方说明，方法内联会使常规补丁失效，静态初始化时机也可能影响补丁行为。[Harmony 边界情况](https://harmony.pardeike.net/articles/patching-edgecases.html)

## 验收边界

本次配套只读检查从目标程序集的托管元数据确认了以下静态入口：`SpriteText.drawString`、`getWidthOfString`、`getHeightOfString`，`Utility.drawTextWithShadow` 的两个重载，`LocalizedContentManager.LoadString` 的多个重载，以及 MonoGame `SpriteBatch.DrawString` 的六个重载。这增强了上述方案针对该目标的可行性，但没有执行方法、安装补丁或观察动态命中。它不能证明当前画面的文本已被捕获；原始检查证据只保存在本地测试目录。

最小成功证据应是：在指定游戏版本中启动桥接插件，打开指定界面，捕获与该界面对应的文本；随后用一个确定性测试映射证明原位替换，并确认停用插件后行为恢复。所有真实捕获文本、截图与原始日志只放在 `local-test/evidence/`，本页只保留可移植的结论。

本研究不涉及 UIA，也不要求恢复归档 UIA/OCR 包。对其他游戏的支持应按引擎及插件入口另行评估，不能从星露谷的托管模组能力推出通用游戏覆盖率。
