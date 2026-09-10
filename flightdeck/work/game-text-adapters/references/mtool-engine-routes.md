# MTool 清单中的游戏引擎：Glyphshift 接入路线

研究日期：2026-09-10。状态：官方资料与源码检查；未下载、运行游戏，未验证注入、捕获或替换。优先级是工程判断，不代表已支持，也不是 MTool 的内部实现说明。

## 建议先做什么

先选 RPG Maker MV/MZ，Ren'Py 和 TyranoScript 作为后续候选。它们能找到文字进入排版前的公开代码或接口，比从 GPU 最终绘制结果反推句子更有依据。不过，**有引擎接口，不等于 Glyphshift 已经能无文件修改地接入已发布游戏**。接入解释器、执行线程、版本识别与恢复原函数是第一道验证门槛。

“通用”按引擎家族验证：两个独立游戏共用识别与处理逻辑，不能靠游戏名或固定地址。共用 JavaScript、Python、DirectX 等技术，不等于一个适配器覆盖所有使用它的软件。

## RPG Maker MV/MZ：首选验证

已确认：MV CoreScript 官方分发项目公开 JavaScript 源码；MV 的 `Window_Message.startMessage` 在进入文字状态时调用 `$gameMessage.allText()`，随后处理转义字符。MZ 官方 `Bitmap` 文档公开 `drawText`、字体、字号和测量接口，源码最终用 Canvas `strokeText/fillText` 绘制，再作为纹理使用。[MV 项目](https://github.com/rpgtkoolmv/corescript)、[MV 对话入口](https://github.com/rpgtkoolmv/corescript/blob/master/js/rpg_windows/Window_Message.js)、[MZ Bitmap](https://developer.rpgmakerweb.com/rpg-maker-mz/Bitmap.html)、[MZ Bitmap 源码](https://developer.rpgmakerweb.com/rpg-maker-mz/Bitmap.js.html)

推导出的路线：先接完整对话进入文字状态的阶段，再补菜单、选择项等显示入口；底层 Canvas 作为补充，不能假定它收到的每次调用都是完整句子。MV 与 MZ 应分别建立能力检测和版本测试，不能把 MV 源码直接当作 MZ 合约。

可以验证“运行期提前收集已加载数据中的原文”。MV 的 `DataManager` 维护数据库，并按地图载入 `$dataMap`，因此只扫描已加载对象不能宣称已经拿到整部游戏。只读取明确的显示文本字段，不能递归替换所有字符串：事件参数、脚本、资源路径、标识符都不是译文位置。[MV DataManager](https://github.com/rpgtkoolmv/corescript/blob/master/js/rpg_managers/DataManager.js)

主要难点：无安装目录改动地进入已发布游戏的 JS 执行环境；插件重写文字方法；转义码和变量插值；当前窗口与位图缓存刷新；暂停打字机时更新译文；停止后恢复原文但不重复触发剧情事件。不以安装游戏插件、改 `plugins.js` 或替换项目 JS 文件作为交付办法。

## Ren'Py：公开文字接口明确，运行时接入仍须证明

官方 `config.replace_text` 能返回替换文本，作用于实际显示文字，但调用发生在插值之后、文本按标签分段之后，因此不保证每次收到完整对白。`config.say_menu_text_filters` 更早处理对白与选择项，发生在翻译与插值前；老版本接口还存在单一回调形式，必须按实际版本检测并保留原回调。[配置接口](https://www.renpy.org/doc/html/config.html#var-config.replace_text)、[版本演进记录](https://github.com/renpy/renpy/issues/7031)

推导出的路线：对白和选择项采用保留标签、变量的完整文本入口，UI 再用晚期文字入口补充；不能仅挂 `replace_text` 就宣称整句覆盖。官方翻译系统具备对白单元和字符串翻译能力，可研究只读提取已加载翻译/脚本对象中的显示文本；本轮未确认稳定的“所有原文枚举”运行时 API，不承诺完整预采集。[翻译机制](https://www.renpy.org/doc/html/translation.html)

主要难点：解释器与引擎版本、线程/GIL、已有回调叠加、文字标签、字体覆盖、回滚/读档与历史记录。官方 `renpy.restart_interaction` 提供交互重建能力，但是否足以刷新既有文本缓存需测试；不得通过重跑剧情来假装刷新成功。[交互函数](https://www.renpy.org/doc/html/other.html#renpy.restart_interaction)

官方示例通常是开发者向项目加脚本；这不证明我们能够仅在内存里接入成品游戏。不得以添加 `.rpy` 文件或翻译目录替代这一验证。

## TyranoBuilder / TyranoScript：明确的段落入口

TyranoBuilder 官网提供 TyranoScript 教程；作者公开的 TyranoScript 源码中，文字处理的 `showMessage(message_str, is_vertical)` 先接收文本，处理日志、消息层及后续逐字显示。源码可定位 DOM 消息层及分页逻辑，比逐帧抓字符更适合作为研究起点。[TyranoBuilder](https://tyranobuilder.com/)、[作者源码](https://github.com/ShikemokuMK/tyranoscript/blob/master/tyrano/plugins/kag/kag.tag.js)

推导出的路线：在文字标签进入消息处理前保留原串和上下文，替换显示参数；分别检查选择项、角色名、回看记录。适配器只修改表现层或传给表现层的副本，不回写脚本指令或存档数据。

主要难点：成品的 Web 容器和版本差异、在正确 JS 上下文接入、宏与插件覆盖、文字被标签分段、竖排/ruby/逐字效果、现有 DOM 与日志缓存恢复。源码公开不代表所有 TyranoBuilder 发行版入口一致。不能把任意 DOM 文本替换当成引擎层完整句子支持。

## Kirikiri 2 / Z：可研究，但晚于脚本入口清楚的候选

Kirikiri Z 官方源码的 TJS `Layer.drawText` 绑定到 `tTJSNI_Layer::DrawText`，再调用图像绘字并更新区域，存在带文本参数的引擎文字入口。[Kirikiri Z Layer 源码](https://github.com/krkrz/krkrz/blob/master/visual/LayerIntf.cpp)

推导出的路线：检查完整对白在 KAG/脚本层的组织位置，再判断是否能共用 TJS 或图层入口。`drawText` 可以接收字符串，但上层可能逐字调用，不能据此宣称它总能捕获完整对白。

主要难点：Kirikiri 2 与 Z 分支、游戏自带扩展、静态编译入口识别、调用约定和字符串寿命、ruby、竖排、字形/图层缓存。当前只确认 Z 源码线索；没有核实 Kirikiri 2 二进制或两个分支可共用一个 hook。

## ChoiceScript：公开 JS 语义，可做后续对照目标

发行方官方介绍链接到源码。源码 `Scene.printLine` 先替换变量，再将文本加入段落；这提供了区别剧情命令和显示正文的起点。[官方介绍](https://www.choiceofgames.com/make-your-own-games/choicescript-intro/)、[场景源码](https://github.com/dfabulich/choicescript/blob/master/web/scene.js)

候选路线是段落输出和选项名称处理，而非把场景脚本整体送去翻译。需要验证多行合并、变量、内嵌格式、选项提交与存档不受影响。浏览器版和不同桌面打包方式的无文件修改接入仍未验证；不能因为也用 JavaScript 就并入 RPG Maker 适配器。

## 其余清单：保留候选，未确认实际入口

| 引擎 | 本轮可以确认的内容 | 对下一步的影响 |
| --- | --- | --- |
| RPG Maker XP / VX / VX Ace | 属于旧版 RGSS 路线；本轮直接核实 Ace 官方说明为 Ruby 1.9.2 / RGSS3。XP、VX 的具体 ABI 与入口未逐版核实。[Ace 官方](https://www.rpgmakerweb.com/products/rpg-maker-vx-ace) | 独立于 MV/MZ；需要逐版核对 Ruby/原生绘字入口，不能直接复用 JS 适配器。 |
| RPG Maker 2000 / 2003 | 本轮未找到可直接作为完整字符串接入合约的官方运行时 API。 | 独立候选；不按 RGSS 或 MV/MZ 覆盖计算。 |
| Visual Novel Maker | 官方文档说明 CoffeeScript 编译为 JavaScript，Windows 游戏包使用 NW.js。[脚本](https://asset.visualnovelmaker.com/help/GameScript_Getting_Started.htm)、[打包](https://asset.visualnovelmaker.com/help/Create_Game_Package.htm) | 可考虑共享宿主接入研究，但显示对象及文本接口必须单独核实。 |
| WOLF RPG | 作者官网记录 3.00 引入 Unicode，并提供开发者翻译支持工具。[官网](https://silversecond.com/WolfRPGEditor/)、[作者翻译工具](https://booth.pm/ja/items/5151747) | 版本跨度和字符编码是验证维度。开发者翻译工具不代表成品运行期存在通用替换接口，本轮未确认 hook。 |
| RPG Developer Bakin | 官方公开 C# 插件文档，包括战斗系统等扩展。[官方接口](https://rpgbakin.com/csreference/) | 不足以推断使用 MonoGame 标准 DrawString；需另查文字类型与宿主。 |
| SMILE GAME BUILDER | 官方有导出 Unity 的工作流文档。[官方导出说明](https://smileboom.com/dl/file/sgb/SGB_How_to_create_an_Android_app_EN.pdf) | 原生发行与导出版本应分开识别；不代表所有作品都是 Unity，更不代表一个适配器可全覆盖。 |
| SRPG Studio | 本轮官方网页访问未成功，未核实可引用的运行期文字合约。 | 保留清单，不能仅因第三方插件使用 JS 就判定已能接入。 |
| Pixel Game Maker MV | 官方插件规范公开 JavaScript 接口，包含 `cc.Node` 等对象。[官方规范](https://rpgmakerofficial.com/product/act/en/manual/Plug-in_Spec_en_20200428.pdf) | 名称带 MV 不代表使用 RPG Maker MV 的文字管线；完整字符串入口和不改文件接入未确认。 |

## 原型验收边界

1. 先证明不改安装资源也能接入目标运行时；纯开发项目中的插件成功只算接口实验。
2. 完整原文只采集一次，不把打字机的每个前缀当成词条。区分已加载数据预采集与当前实际显示的命中，不混成一种状态。
3. 一处对白和一组选项完成原文、译文 A、译文 B、停止恢复；中文缺字、长短句、格式标签和变量保持正确。
4. 当前画面收到 AI 新译文后能更新，无须重新读档或重跑剧情；没有译文时继续显示原文，不能阻塞渲染线程等待 AI。
5. 读档、回滚、跳过、自动播放与退出无额外存档修改；第二个独立游戏无需游戏专用代码。

上述步骤尚未执行。现阶段应称为研究候选，不能把截图中的引擎清单复制成 Glyphshift 支持列表。
