# 用户提供的两款游戏：引擎确认与首次采集

本机版本不能按旧版同名游戏或 RPG 游戏外观判断引擎。本轮仅记录可移植结论；路径、进程值、模块清单、部署及原始文字保留在 local-test。

| 目标 | 本轮实际发现 | 与 MV 原型的关系 |
|---|---|---|
| Aooni | Unity 2022.3.22 系列、Mono backend，运行中加载 UnityPlayer 与 Mono；包含 TextMeshPro 和 uGUI 程序集 | 不使用 MV/NW.js 路线，可验证现有 Unity Mono 标准界面适配器 |
| Mad Father | Game 可执行版本 ver2.21，Data.wolf，符合 WOLF RPG Editor 的打包；未发现 NW.js 或 RGSS 模块 | 需要 WOLF 入口验证，不属于当前 MV 原型范围 |

Aooni 使用最新评审 Bundle 的正式 Controller 和 Unity Mono 标准界面适配器完成一次独立、仅观测会话。
激活成功，8 个查询批次共 725 条观测、114 种不同原文；测试结束正常停用，两个游戏均继续响应。
没有提供替换字典，也没有改变游戏文件或存档。

这些记录包括菜单、存档槽和疑似未显示对象的模板文字。非零采集不等于当前画面命中，
也不证明对白完整性、可见替换、字体覆盖或停用画面恢复。下一步应选择实际可见的 Unity 文字，验证两代替换与停用恢复。
Mad Father 本轮只完成引擎确认，尚未进行文字 Hook 验证。两者均不计入 MV 商业目标验收数量。

## 用户后续画面反馈与待办

用户随后在自己的 Aooni 工作流中启用翻译：字典内中文正常，游戏菜单显示方框。该画面不能算作中文翻译验收通过。
静态检查确认 Unity Mono 的 `HostBridge::decide` 接收字体缓冲区，但返回的 `TextDecision` 只携带译文与代次；
现有标准 UI 绑定管理 `set_text`，没有应用字体或补齐字形的实现。缺字是当前优先验证方向，尚未完成活动字体对照，不能将其写成已证实的唯一根因。
后续已通过主线程实际字图对照确认本次 TMP 缺字原因，并验证字体文件加载、两代中文与恢复；见 [Unity 字体回退](../slices/unity-font-fallback.md)。正式 Adapter 接入尚未完成，uGUI 仍须独立验证。

WOLF 值得作为通用引擎候选：其[官方网站](https://silversecond.com/WolfRPGEditor/)列出 Mad Father、LiEat、片道勇者等作品，
并长期维护编辑器、作品投稿与比赛。此依据说明存在跨游戏复用价值，不代表掌握市场份额或已支持这些游戏。
官网明确将 Unicode 支持列为 3.00 的更新；本轮 Mad Father 为 2.21，因此编码与入口必须单独验证，不能套用 3.x 的结论。
用户要求的 WOLF 适配与 Aooni 缺字修复已列入 [Plan](../plan.md)，保留为未完成项。

## 扩展游戏目录静态筛选

用户授权检查更多已下载游戏。仅检查文件特征与运行器版本信息，没有启动或注入这些目标。

| 目标版本 | 本轮证据与归类 | 当前用途 |
|---|---|---|
| The Witch's House 1.09a | RGSS202E.dll、rgss2a 与 rvdata 脚本配置，属于 RPG Maker VX / RGSS2 | 后续 RGSS2 候选，不适用 MV 入口 |
| OFF English 3.0 | RPG_RT、ldb/lmt；所带运行器版本资源标为 RPG Maker 2003 | RPG Maker 2000/2003 路线候选，不适用 MV |
| END ROLL | RPG_RT 与 ldb/lmt，随附 Readme 明确写明 RPG Maker 2000 VALUE! | RPG Maker 2000 路线候选，不适用 MV |
| Yume Nikki | RPG_RT 与 ldb/lmt；所带运行器版本资源标为 RPG Maker 2003 | 仅描述实际运行器，不能据此推导原作制作版本；不适用 MV |
| ATRI -My Dear Moments- | 多个 xp3 资源包，程序版本资源明确标为 TVP(KIRIKIRI) Z core | KiriKiri Z 候选，需验证动态文字入口 |
| NEKOPARA Vol. 3 | cs2confx.dll 与多个 int 资源包，程序版本资源为 cs2 2.6.1.76 | CatSystem2 候选，需验证动态文字入口 |

搜索到的 MV 核心脚本仅属于编辑器模板与附带示例，没有找到独立 MV 游戏；也没有发现 MZ 核心脚本。
这些结果只用于挑选测试目标，不代表产品已经支持。目录、完整文件清单保存在本机证据目录，不进入受跟踪文档。
