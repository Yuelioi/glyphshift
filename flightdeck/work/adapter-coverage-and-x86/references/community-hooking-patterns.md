# 社区文字 Hook 项目的可借鉴边界

调研日期：2026-09-08。范围仅限 Textractor、LunaTranslator 内的 LunaHook、XUnity.AutoTranslator 的项目源码与文档；未运行第三方软件，未继续实机注入。链接指向本次读取的主分支文件，后续内容可能变化。

## 结论

**没有找到“收集任意底层单字，随后通用地改写原画面”的成熟现成方案。** 三个项目展示的是两类明确不同的能力：提取端可以通过上下文、静默时间和去重整理可读文本；原位替换需要掌握显示前的文本参数或仍然有效的控件。此结论仅限本次调查，不是对整个社区的不存在证明。

| 项目 | 已证实的文字完整性策略 | 已证实的显示路径 | 对 Glyphshift 的意义（推断） |
| --- | --- | --- | --- |
| Textractor | 按 Hook 上下文分流，解码并追加缓冲区；静默时间、大小或完整串标志触发输出 | 注入端将输入送给宿主，宿主经扩展交给 GUI | 可以借鉴采集整理，不能据此宣称同一串已经能原位替换 |
| LunaTranslator / LunaHook | LunaHost 同样有解码缓冲区和 flush delay | 外挂显示与内嵌分别选择；仅部分 Hook 支持内嵌 | Observe 与 Replace 必须独立确认 |
| XUnity.AutoTranslator | 从文本控件读取状态；渐显文本等待稳定，再读取完整文本 | 设置已知组件的文字，处理字体、布局及原文恢复 | 优先找拥有完整文字和生命周期的框架入口 |

表格证据见下面对应项目；“意义”是结合其机制对本项目的推断。

## Textractor：分流和超时合并属于提取链路

- `ThreadParam` 用进程、Hook 地址、两个上下文作为分流身份。默认主上下文通常取返回地址；这是调用来源标识，没有声明它等同控件身份或绘制事务。[types.h](https://github.com/Artikash/Textractor/blob/master/include/types.h)
- `TextThread::Push` 解码后追加文本，处理 DBCS 前导字节；`Flush` 在缓冲超限或距最后输入超过 `flushDelay` 时排队输出。`FULL_STRING` 和重复文本过滤提供其他输出条件。这是启发式句子整理，不是引擎给出的结束边界。[textthread.cpp](https://github.com/Artikash/Textractor/blob/master/host/textthread.cpp)
- 项目明确支持 x86/x64；文档描述的链路是目标进程 Hook → 管道 → 宿主处理 → GUI 扩展与显示。因此，本次证据证明其提取能力，未证明它具备相同范围的目标画面替换能力。[README：Project Architecture](https://github.com/Artikash/Textractor/blob/master/README.md#project-architecture)

**推断：** 用静默时间改善 Glyphshift 的采集列表可以是单独能力，但不能把超时输出当成安全替换点；到宿主凑齐文本时，原绘制调用可能早已结束。

## LunaTranslator / LunaHook：内嵌有独立契约

- LunaHost 的 `bufferDecoded`、`UpdateFlushTime`、`FlushBufferToQueue` 延续了解码缓冲和静默时间输出方式。[textthread.cpp](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/LunaHost/textthread.cpp)
- `HookParam` 分别提供 `text_fun`、`filter_fun`、`embed_fun` 以及字体 Hook 字段。引擎入口也明确分为 32/64 位集合，说明完整性/过滤/内嵌处理并没有被一个架构无关的底层 Hook 自动包办。[types.h](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/include/types.h)、[enginecollection32.cpp](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/LunaHook/enginecollection32.cpp)、[enginecollection64.cpp](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/LunaHook/enginecollection64.cpp)
- 官方内嵌文档明确区分软件窗口显示与游戏内嵌；仅部分游戏支持后者。它解释的内嵌过程是在显示前暂停相应函数，等待翻译，修改文字内存后继续执行，并提供等待上限。实现中 `waitforevent` 按剩余时间等待且检查当前线程是否仍可内嵌。[内嵌文档](https://github.com/HIllya51/LunaTranslator/blob/main/docs/zh/embedtranslate.md)、[embed_util.cc](https://github.com/HIllya51/LunaTranslator/blob/main/src/NativeImpl/LunaHook/LunaHook/embed_util.cc)
- 同一文档把字符编码、字体和行宽作为独立问题；也明确描述“清空原文后将外部窗口盖上去”的办法。它属于替代显示路径，不能作为原位替换已成功的证据。[内嵌文档](https://github.com/HIllya51/LunaTranslator/blob/main/docs/zh/embedtranslate.md)

**推断：** 可以共享翻译缓存、过滤器和运行时传输，但每个替换入口仍需证明写回目标、长度、编码和有效时机。字体支持确实是实际能力需求，不能由“中文决策命中”替代验证。

## XUnity.AutoTranslator：优先使用完整文本所有者

- UGUI 适配 Hook `text` setter 与 `OnEnable`，再调用 `Hook_TextChanged`；代码为 Managed 与 IL2CPP 提供各自入口。它没有先等待 GDI 字形绘制再重组控件文字。[UGUIHooks.cs](https://github.com/bbepis/XUnity.AutoTranslator/blob/master/src/XUnity.AutoTranslator.Plugin.Core/Hooks/UGUIHooks.cs)
- `WaitForTextStablization` 针对渐显内容等待后重读同一控件；完成翻译时检查当前原文与任务原文一致，才 `SetTranslatedText`。写入时关闭自己的 Hook/设置重入标记，保留原文与译文供模式切换恢复。这些都是控件身份与生命周期管理。[AutoTranslationPlugin.cs](https://github.com/bbepis/XUnity.AutoTranslator/blob/master/src/XUnity.AutoTranslator.Plugin.Core/AutoTranslationPlugin.cs)
- 支持列表是 Unity 的 UGUI、NGUI、IMGUI、TextMeshPro、TextMesh；字体配置区分 UGUI 字体覆盖、TMP 字体覆盖和 TMP fallback。它体现了框架级方案的复用价值，也明确有框架与运行时版本边界。[README：Text Frameworks / Fonts](https://github.com/bbepis/XUnity.AutoTranslator/blob/master/README.md)

**推断：** 对保留型 UI，应先找公开的文字 setter、资源加载/查询或可确认的控件文本入口，让原框架负责布局。只有获得可靠完整绘制事务时，才评估延迟绘制重放；两者不能仅凭采集输出相同而视为等价。

## VGUI / Source 与下一步

本次检查了 Luna 的 32/64 位引擎注册文件、上述项目树中的 VGUI/SourceEngine/Left4Dead 命名，以及 XUnity 的支持列表，**未找到可直接接入的 VGUI/Source 翻译适配器**。未命中名称不等于不存在：可能有通用 API 路径、不同命名、未检查分支或外部扩展，不能把此结果写成“不支持 Source”的确定事实。

建议保留共用文字完整性与作用域机制，将 VGUI 延迟绘制继续标记为实验能力。下一轮先比较“公开文本所有者/本地化入口”和“绘制重放”的证据与代价，再决定继续哪条路；不再仅以采集命中或译文决策作为晋升条件。验收应分别证明完整采集、可见替换、更新和停用恢复。以上是本次研究的工程建议，不是第三方项目对 Glyphshift 的兼容性保证。
