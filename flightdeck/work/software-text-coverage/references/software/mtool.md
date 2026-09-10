# MTool

当前决定：作为下一阶段游戏引擎适配的参考工具，先调查，不把官网支持列表直接写入 Glyphshift 产品能力。

## 官方公开功能

官网列出 RPG Maker、Wolf RPG、SMILE GAME BUILDER、RPG Developer Bakin、TyranoBuilder、Kirikiri 2/Z、Ren’Py、Visual Novel Maker、ChoiceScript、SRPG Studio、Pixel Game Maker MV。支持列表是 MTool 的厂商声明，不代表每一款作品、插件或改版引擎都已独立验证。[官网](https://www.mtool.app/?lang=chs)

官网强调可完整翻译游戏并离线使用。教程展示选择游戏程序、加载资源、选择语言和开始翻译，还包括导出原文、加载翻译文件、恢复原文、删除缓存后重载。这些说明其公开产品流程有资源加载和批量翻译阶段，而不只是屏幕逐句采集。[使用教程](https://www.mtool.app/tutorial.php)

公开页面未说明各引擎的底层注入方式、资源是否写回安装目录、是否替换解释器、缓存与停止恢复细节，也没有据此可核验的完整实现源码。本次不把上述未知项写成事实，不下载或执行该工具。

## 对 Glyphshift 的价值

优先寻找引擎仍持有完整对白、选择项、界面标签的层次，再接统一字典决策。可研究在运行时预采集已加载的文本数据，减少逐字显示与等待 AI 造成的碎片和延迟。预采集与替换能力分别声明：取得全部原文不代表运行时替换可用，更不代表动态生成文字全部可预取。

运行桥接负责进入引擎环境，具体引擎适配负责文字语义和刷新；共享 JavaScript/Python 等运行环境不等于共享全部文字入口。原文和变量、控制符、节点标识、表达式、脚本代码必须区分。保持不修改目标安装文件的约束。

已有字典导入导出的格式处理器可承接明确格式的文本文件；没有 MTool 导出样本时，不宣称兼容其专有文件。

## 后续记录

[引擎路线对比](../../../game-text-adapters/references/mtool-engine-routes.md) 由 [游戏文字适配器 Work](../../../game-text-adapters/index.md) 管理。首个原型仍需验证完整字符串、菜单和对白、中文字体、逐字显示、两代译文与停止恢复，并至少用两个独立目标判断是否真正复用。
