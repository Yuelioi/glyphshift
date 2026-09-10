# 游戏文字适配器

Status: Open

## Goal

沿用 Glyphshift Adapter 模式建立可跨游戏复用的文字翻译能力。保留已验证的 MonoGame 路线，下一阶段从 RPG Maker MV 验证引擎文字接入与运行时职责拆分。

## Current

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
