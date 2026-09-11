# 游戏文字适配器

Status: Open

## Goal

沿用 Glyphshift Adapter 模式建立可跨游戏复用的文字翻译能力。保留已验证的 MonoGame 路线，下一阶段从 RPG Maker MV 验证引擎文字接入与运行时职责拆分。

## Current

[v0.4.0 发布](slices/release-040.md)进行中：版本已统一，前端生产构建和 588 项活动包测试通过，准备提交 main 并推送版本标签。

[字典返回入口](slices/dictionary-parent-return.md)已修正，从工作流打开后返回原工作流，并保留筛选及未保存修改确认。2 项页面和生产构建通过，评审脚本已同步构建启动，Bundle 双份校验通过。

[规则翻译行合并](slices/rule-row-grouping.md)已实现，默认按规则固定原文分组，任务操作可切换，保留最近数值并累加次数。117 项 shell、相关页面及生产构建通过，评审脚本已同步构建启动，Bundle 双份校验通过。

[清空与重新捕获](slices/dictionary-clear-blacklist.md)已撤掉错误的持续排除集合：清空只重置当前采集记录并恢复运行，相同文字正常重新收录；来源仅提示额外字典。116 项 shell、2 项页面通过，评审脚本已同步构建启动，Bundle 双份校验通过，窗口响应。

[规则行直接编辑](slices/dictionary-text-rules.md#规则行直接编辑修正)已修正：输入译文写入固定 key，显示完整规则结果；字典页和工作流文字列表的旧隐藏开关移除。113 项 shell、4 项页面及生产构建通过；修复版已由评审脚本同步构建启动，两份 Bundle 校验通过，窗口响应。

模型下拉误筛选已修正：重新展开显示全部已获取模型，仅手动输入时筛选；精确交互回归先失败后通过，2 项相关测试通过并已检查截图，修复版已由评审脚本打包启动，Bundle 校验通过。详见 [AI 模型列表](slices/ai-model-discovery.md#下拉完整列表修正)。

[托盘与置顶](slices/window-tray-topmost.md)及 [AI 模型列表](slices/ai-model-discovery.md)已打包启动。112 项 shell、15 项页面回归通过；生产构建、两份 Bundle 的 13 个适配器校验通过。评审窗口可见且响应，托盘宿主存在，哈希一致。真实托盘点击与置顶切换待用户本机验收，原生自动化调试端口未开放。

[AI 配置与模型列表](slices/ai-model-discovery.md)已实现：Key 与思考说明对齐，模型可手填或获取下拉，11 个服务预设；4 项接口及 2 项页面合同、8 项 AI 配置回归通过，已随最新 App 打包启动。

[字典文字规则与固定标签](slices/dictionary-text-rules.md)已实现：移除全局规则且无迁移，字典元数据打通运行、采集与 AI，设置/帮助标签统一并固定。活动包 579 项用例通过（旧 wire 断言修正后定向复跑），最终 shell 108 项及相关页面 8 项通过，生产构建通过；已随 AI 配置及托盘切片重新打包启动。

独立变量 `{{TR}}` 与单条规则测试器已实现：输入模拟原文及可选模拟译文，不读字典、不保存，显示捕获组和结果。7 项核心合同、5 项页面测试通过；评审脚本已重建并启动最新 App，原始和打包 Bundle 的 13 个适配器校验通过。详见[规则切片](slices/regex-translation-rules.md#独立译文变量与单条正则测试器)。

正则变量名已统一为英文 `{translate:$1}`，兼容旧配置，并补充整条译文优先的操作提示。6 项核心合同及 4 项页面测试通过；代码尚未重新打包，用户字典及编辑中的规则未修改。详见[规则切片](slices/regex-translation-rules.md#英文变量名与整条译文优先的澄清)。

多适配器绘制范围已改为“未替换放行下层、已输出文字替换才保护嵌套绘制”，详见[协同切片](slices/adapter-cooperation-stop.md#未替换时放行下层)。新增场景先红后绿，2 项 Runtime 库测试通过；已通过评审脚本重新构建并启动最新 App，原始及打包 Bundle 的 13 个适配器校验通过。目标需重启以加载新 Runtime，现场测试待用户完成。

[正常退出清理](slices/app-exit-cleanup.md)已实现：关闭前停止运行效果，失败保留窗口重试，退出期间不自动重连；工作流配置保留。108 项 shell 测试及 2 项关闭/最小化页面测试通过。最新评审 App 已由 `scripts/review-app.ps1 -UseUserData` 同步构建并启动，13 个适配器的原始及打包 Runtime Bundle 均校验通过；进程响应、窗口存在、可执行文件哈希一致。沿用用户工作区，等待本机退出恢复测试。

[多适配器协同与停止修复](slices/adapter-cooperation-stop.md)已完成：修复一个适配器停止失败导致后续候选未收到停止请求的短路，活动报告保留真实未停止者。真实 GDI 像素回归先红后绿，44 项相关测试通过。协同支持并存与线程内绘制隔离；用户现场 Qt 初始化失败仍待独立复现。本轮未重新打包 App。

[加勾 Windows 后启动成功的调查](references/adapter-activation-partial-success.md)：两个 Native 对照合同通过，确认至少一个候选激活即可整体成功；尚缺用户现场的原适配器选择与实际采集结果，未宣称修复。

[文字处理与收录过滤](slices/text-processing-collection.md)已实现：文字过滤与替换规则合到“文字处理”，命中的新原文不再自动进入写入字典；既有条目保留。修正属性计数被数值单位误判的问题。120 项相关 Rust 测试、4 项页面测试和前端生产构建通过；本轮未重新打包评审 App。

[收集列表状态与来源](slices/entry-resolution-status.md)已实现：保留正常译文与自动保存，移除过时添加按钮，增加规则/过滤/字典来源和对应筛选，附加字典已观测条目标明只读。相关 Rust 146 项通过；新增 Playwright 用例、宽窄截图独立复核及前端生产构建通过；旧工作流页面套件仍有 9 项失败，详见切片。本轮改动尚未重新构建评审 App，完整联合表另行设计。

正则规则版本已通过 `scripts/review-app.ps1 -UseUserData` 同步构建并启动最新 Release，13 个适配器的原始 Bundle 与打包副本均校验通过；沿用用户工作区。启动进程响应且存在窗口、哈希一致，等待用户本机测试。原生窗口的 Playwright CDP 连接被拒绝，未把这次启动算作页面像素验收。

设置中的[正则翻译规则](slices/regex-translation-rules.md)已实现：用户逐条管理匹配正则与替换模板，通用规则通过 `{译文:$1}$2` 翻译任意已知属性并保留数值，字典格式不变。完整原文译文优先，捕获查词遵循工作流路由和 Adapter 范围；规则支持下一代发布和移除恢复。9 项新增 Rust 合同、活动包 575 项回归、7 项页面流程及生产构建通过；该正则版本已按上文构建评审 App，未提交。早期[匹配研究](references/property-value-translation-research.md)中的词条模式已被用户选定的全局设置方案取代。[布局与对齐](references/translated-text-alignment-research.md)仍缺目标软件和错位表现，未修复现场布局。

[统一工作流运行模型提案](references/workflow-lifecycle-proposal.md)已形成，处于讨论状态：单一开始/停止、收集新原文作为设置、退出等待与主动停止分离、停止失败不被开关遮住、统一权威快照及 AI 拥有者语义。包含具体场景和分阶段验收；不把提案当已实现契约，未修改产品代码。

[工作流状态调查](references/workflow-state-review.md)已完成报告：Playwright 合成复现手动刷新后顶部活动徽标仍读取旧采集缓存；单独刷新记录的对照通过。确认切回工作流未触发刷新，另记录暂停意愿与连接状态混用风险。用户本次只要求报告，未修改产品代码；原生目标退出回收尚未实机验证。

工作流新选软件默认全选可选适配器，同一软件重选保留手动取消项；已有工作流加载不改选择。界面中英文统一 Adapter/适配器术语，并解释默认选择。Vue 类型检查与 6 项相关前端流程通过。此项是本地 Release 打包之后的修改，先前安装包不含此项，尚未再次构建。

最新评审 App 已通过 `scripts/review-app.ps1 -UseUserData` 构建、校验并启动；随后按用户要求用 `scripts/build-desktop-release.ps1` 生成本地 0.3.0 Windows x64 NSIS 安装包供群内测试。解包 Runtime 通过 13 个适配器校验；主程序除 Tauri 的 UNK→NSS 打包类型标记外逐字节一致，安装包与清单哈希一致。包内没有用户配置、字典或测试日志；未发布线上 Release、创建标签或提交。

[采集合并与筛选性能](references/collection-performance-review.md)首轮完成：摘要与独立每秒采集分离；字典快照、观测解析、合并排序和规则分类缓存；AI 单次固定快照；页面/分类请求合并；工作流隐藏跳过条目在分页前生效。30 项采集、99 项桌面和 1 项分类缓存合同通过，13 项不同前端流程及类型检查通过。合成万条原文 20 页约 138 ms，对照每次重建约 2.32 s，不推广为实机帧率。完整原子保存保留，当前无新 App 构建或提交。

自动补全新增按工作流合并的 FIFO 等待队列：全局任务忙时只排一次，出队重新生成最新计划，取消/暂停/失败移除等待项。用户确认保留间隔 0 的 250 毫秒检查，不改为 500 毫秒。批次仍交给现有模型配置的并发执行器；8 项自动补全 Playwright 流程和 2 项真实并发合同通过，类型检查通过。该交互仍需下次通过评审脚本构建后才能在 App 验证。

创建工作流现在直接打开软件选择窗口；取消后保留原有的空页签再次打开行为。语言管理增加按本地化名称/代码查找建议，覆盖宿主 Intl 识别的两字母语言及常用地区代码，不宣称完整 BCP 47 清单，仍允许自定义代码。类型检查与 8 项相关 Playwright 流程通过，语言建议截图已检查；这些最新交互尚未重新构建评审 App。

最新完成 [字体与字号归工作流](slices/workflow-typography.md)：工作流保存默认值及每字典覆盖，核心 Dictionary 已移除字体字段，旧字典包字体仅保留格式兼容。创建字典补共享语言下拉、字典行字体支持全部/常用筛选，关于增加在线文档及发布页，修正问号提示层级。61 项相关 Rust 合同与 6 项相关 Playwright 流程通过，Vue 类型检查及宽窄布局检查通过。当前修改尚未构建为新的评审 App；Unity 实际缺字修复仍在 Next。

设置页管理已扩展为通用、软件、字体、语言四个局部导航。最近软件与工作流选择器共享持久化记录，支持逐条删除与清空；常用字体支持搜索收藏，并在字典、工作流及备用字体选择器中优先显示；翻译语言列表支持增删、搜索和恢复 20 种内置语言，字典新建、编辑和批量导入共用选择器。候选删除不改动已有字典、工作流和备用字体配置。修复自定义语言创建后下拉未关闭的问题，防止干扰后续表单输入。

本轮验证：13 项 Rust settings 合同通过；最新 10 项相关 Playwright 回归通过，涵盖导入语言、字典完整元数据、最近记录、字体回退设置、软件选择及宽窄布局。语言管理/收藏的增删、恢复与重载流程另已通过；Vue 类型检查通过。宽窄字体页和语言列表已进行截图检查。旧工作流测试补齐设置保存接口 mock；下述两个旧 settings 套件断言仍是已知历史问题，未声称全套通过。运行时字体消费仍待 Next，不把配置管理完成算作 Unity 缺字修复。

设置页已新增“缺字备用字体”，提供 20 个常用语言选项并允许自定义代码，支持搜索本机字体、按语言修改/移除与即时保存。前后端均保存 `languageFallbackFonts`，旧配置默认空列表，损坏条目隔离、语言大小写去重。页面明确提示 Unity 尚在接入，不把已保存配置冒充运行时生效。新增 UI 流程测试与 2 项持久化/校验合同通过，宽/窄布局已检查。

验证：Vue 类型检查、7 项既有 Rust settings 合同和 2 项新增字体配置合同通过；新增 Playwright 字体配置流程通过。既有 `settings.spec.ts` 为 11 通过、2 失败：帮助页仍断言旧的软件/探针流程，另一项仍假定设置页没有任何文本框并继续导航已移除的软件页。本次未修改这些旧断言或无关帮助页，不将该套件报告为全通过。

用户实测的 Aooni 中文方框已完成字体对照：原 TMP 图集缺字；使用字体文件生成新图集后，两代中文实际可见，最后原文、字体、材质和文字像素恢复通过。此为主线程 API 实验，尚未接入正式 Adapter，当前 App 未修复。详见 [Unity 字体回退](slices/unity-font-fallback.md)。

游戏目录筛选已完成，并将 WOLF、RGSS2、RPG Maker 2000/2003、KiriKiri Z、CatSystem2 的目标与未完成验证明确加入 Plan。未找到独立 MV 游戏；Aooni 与 Mad Father 分别为 Unity Mono 和 WOLF，均不计入 MV 验收。详见 [游戏检查](references/aooni-mad-father-check.md)。

最新完成固定引擎脚本接入：`-ManagedBootstrap` 变体把对白脚本嵌入 DLL，由 Native Adapter 激活自动安装，停用自动撤销自己的 JS 与原生 Hook。实际运行器合同通过两代中文、单次采集、4 轮安装/撤销后的原函数引用恢复，以及错误页面超时回滚后重试。测试不再手工安装对白入口。详见 [Runtime 自动接入](references/mv-runtime-entry-validation.md#固定引擎脚本并入-runtime-激活)。

MV 受控运行器已通过不依赖 Frida 的端到端验证。原生 DLL 自己管理具名 V8 与 Node 加载 Hook、正确页面上的执行队列及停用；独立 x86 测试程序只负责装载和控制。程序在用于写入的同一进程句柄上核对路径、创建时间和位数，不假设远程模块地址与本机相同。Python 驱动使用标准库即可运行，不导入 Frida。实现及复跑说明见 [测试桥](../../../test-support/rpgmaker-mv-text/README.md)，细节见 [原生会话记录](references/mv-runtime-entry-validation.md#原生会话与无-frida-测试装载)。

实际合同覆盖完整对白、正式 Runtime 两代中文、每次一次采集、停用后新对白恢复，以及 12 轮 Runtime 启停。新增 4 轮原生 Hook 会话关闭/开启验证，停用时拒绝请求；错误页面等待 5 秒超时，页面恢复后过期请求不执行。两种 JS 异常与连续调用恢复正常。1 项实际运行器合同、12 项合成测试通过，项目脚本和数据文件保持不变。

当前范围仍是 x86 / NW.js 0.29.0 normal / Node 9.7.1 / V8 6.5.254.31 的自行编写项目。桥不在 DLL 构造函数注册 Node，先加载 DLL、后接入 Node 已验证。固定脚本已由 Runtime 激活接管，接入失败明确回滚。代码留在 test-support，未进入产品 Catalog 或 Bundle；产品 Controller/打包与正式会话绑定、当前对白刷新恢复及商业目标仍未完成。

最新评审 App 已通过 `scripts/review-app.ps1 -UseUserData` 同步构建并启动，13 个正式 Adapter 的 Bundle 校验通过，包含字典翻译状态筛选与精简提示。用户检查 App 时，MV 测试使用独立受控项目。

整体方案见 [引擎接入方案](references/engine-adapter-architecture.md) 与 [Plan](plan.md)。候选调研见 [引擎路线对比](references/mtool-engine-routes.md) 和 [MTool 记录](../software-text-coverage/references/software/mtool.md)。优先验证 MV，其次 Ren’Py、Tyrano；MZ 与其他引擎单独验收，第三方支持清单不代表 Glyphshift 已支持。

## 已有 MonoGame 工作

用户已确认说明翻译生效，随后发现同一句原文因折行位置不同出现多条记录。已落实 Adapter 声明的软折行归一化：采集与查找使用统一键，历史观测只读合并；已有译文冲突由用户在行内选择，保留原始词典。混选 Adapter 时依据真实观测来源，不改其他 Adapter 的正常换行。平铺译文按原文行宽重新折行。

辅助代码嵌入同一个受校验 DLL，只使用目标现有 MonoGame 类型与 Windows 字体；没有独立插件。缺字图集在 Present 帧边界发布，下一帧测量/绘制使用一致字体，旧纹理延迟释放。旧 fixture 34 项断言及 Rust Loader 合同回归通过；仍保留驻留到目标退出的生命周期约定。

最新桌面 App 已同步构建并启动，默认读取用户数据；可直接打开的程序目录包含配套 runtime，启动明确传递数据与 Runtime 路径。驻留标识已在探针选择器显示。当前进程若加载过前一版驻留组件，需要退出目标后再测试新构建；不得强制卸载 CLR 仍引用的代码。

只读 IL 检查确认普通文字调用 DrawString，对话经 SpriteText 直接提交纹理字形，标准 MonoGame Hook 无法完整覆盖。不按游戏名称的结构扫描找到 6 个候选但有误报，自动自绘文字替换仍未证实。已排除 SMAPI/游戏 Mod 作为当前交付路线。

## Next

[统一工作流运行生命周期](slices/workflow-lifecycle.md) 已实现并通过定向回归，已通过 review-app 构建并启动最新 Release，目标软件实机验收待进行。优先检查目标退出/重开、收集设置和自动补全；之后继续下面的游戏适配工作。

先按 [Unity 字体回退](slices/unity-font-fallback.md) 将已验证的 TMP 字体文件加载与补字形接入正式主线程 writeback，补资源寿命、恢复与能力边界合同，再用配套 Runtime Bundle 验证。两代实际字图和原状态恢复实验已完成，不必重新猜测编码问题。保留用户正在运行的工作流，不用独立测试控制器抢占其会话。

随后按 [MV 运行器接入实验](references/mv-runtime-entry-validation.md) 为固定引擎入口补正式会话绑定与失效处理，接入产品 Controller/打包边界；随后实现当前对白的安全刷新与恢复。无 Frida 的受控接入、Runtime 自动安装/撤销、正式查词和采集已验证，不再重新寻找入口。

MonoGame 既有覆盖和未解决的自绘纹理文字范围继续保留，不因新清单而改变已验收边界。

## References

- [工作约束](context.md)
- [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)
- [初次入口研究及已排除的 Mod 路线](references/stardew-initial-feasibility.md)
- [适配方案](references/monogame-adapter-design.md)
