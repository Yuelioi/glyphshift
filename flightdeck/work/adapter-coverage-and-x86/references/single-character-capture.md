# 游戏单字采集诊断

用户已确认目标是求生之路 2，先前称绝地求生是口误。

保存的探针目录中，113 个条目有 112 个长度为 1，全部来自 `windows.gdi.ext-text-out`。本地快照断言复现了预期词组缺失、组成字符单独存在的问题。原始文本和快照仅留在本地测试证据目录。

源码检查：GDI 原生适配器按 ExtTextOutW 提供的 count 读取整个缓冲区；普通 UTF-16 和 glyph-index 解码均形成一个字符串，再提交一次决策。未发现该路径主动把整句拆成多个词条。现有数据已在展示前成为单字，因此不是表格排版导致。

优先假设是命中了逐字绘制或字体纹理生成阶段；两者仍需实际回调的 count、调用模块、绘制目标及重绘复用情况来区分。目录数据不保留足够的空间和调用上下文，不能靠采集顺序可靠还原句子。盲目拼接或过滤所有单字都不能修复原位翻译。

Valve 官方 [ISurface](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui/ISurface.h) 同时定义整段 `DrawPrintText`、单字 `DrawUnicodeChar` 和 `GetTextSize`。若确认目标是求生之路 2，可将 VGUI 的完整文字入口作为候选；公开 SDK 2013 的接口顺序不直接证明该游戏版本的 ABI，不能直接照抄虚表索引。若目标确为绝地求生，应重新定位对应探针和框架。

后续只读核对当前游戏的 CreateInterface 注册链、实际返回对象和函数体，确认存在 VGUI_Surface031；该游戏的表布局与当前 SDK 2013 不能直接混用。实机地址、模块副本、反汇编和布局证据仅在本地测试目录。

已建立本地限定的 x86 观察原型，挂接核对过的整段绘制和测量入口；独立 MSVC 宿主执行 300 次调用，完整字符串、宽高和停用透传均通过。这是调用约定证据，尚未证明游戏菜单实际命中整段入口。

实机激活被已有版本保护拒绝：TargetRuntimeRestartRequired。未绕过该保护，已请求用户退出并重启游戏、停在主菜单且不要先开旧探针；等待完成后再用生产 Controller 运行观察原型。产品代码尚未改变。

用户已完成重启。观察原型正常激活，前台重绘获得 13 条完整文本事件（11 条测量、2 条绘制），之后正常停用。用户进一步强调通用解决方案，后续依据 [共用设计约束](generic-text-completeness.md)推进；不将临时样本的地址和虚表位置作为产品规则。

Valve 的 [TextImage::Paint](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/TextImage.cpp) 在持有完整文本时仍逐字调用 DrawUnicodeChar。因此若 DrawPrintText 无法覆盖当前菜单，应上移至对应文字对象/布局入口，不能把“有整段 API”当作菜单命中的证明。最终还需处理原低层 GDI 来源和新来源的选择，避免整句可采集后仍混入字形缓存单字。
