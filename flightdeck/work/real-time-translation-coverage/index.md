# 通用实时翻译覆盖

Status: Open

## Goal

让用户完成稳定、可解释的端到端流程：选择软件，捕获原文，使用或编辑 Dictionary，并由真实
`TextReplace` Adapter 把译文立即写回目标界面。通用性按可复用文字技术扩展，不以“支持所有软件”
或成功注入代替真实翻译结果。

## Current

现有 GDI、USER32 DrawText、GDI+、Qt、GTK 3/Pango 与 DirectWrite TextLayout Adapter 已进入正式
Runtime Bundle。Console 与 UI Automation 目前只有采集价值，不能作为实时翻译成功。Direct2D
`DrawText` 已通过合成写回，但在授权目标中零命中，因此仍未进入生产 Bundle。

DirectWrite TextLayout 已从窄原型升级为生产 Adapter：同时覆盖普通 Render Target 与 Scintilla 缓冲
绘制使用的 Compatible Bitmap Render Target，在 `CreateTextLayout` 取得完整 source，并在
`DrawTextLayout` 时按当前 Dictionary 创建瞬时译文布局。单一格式布局可替换，局部格式、内联对象或
无法证明格式一致的布局安全放行；停止后目标继续使用原布局，不需要修改目标文本。

当前已补齐两段实时更新合同：Probe 内编辑译文会保存到绑定 Dictionary 并发布下一代预览；GDI+
确定性 Windows 宿主在不重启目标的情况下能从首版译文切换到第二代，诊断确认两代均为
`Matched + Replaced`，停止后恢复原文。既有软件上的补充可见验收不再占用当前主线，后续只在明确
需要补充发布证据时恢复。

Probe 与 Dictionary 的创建流程也已收敛：每次打开都使用干净表单，任务名称与新词典名称不再根据
软件自动生成，取消或关闭不会遗留旧草稿；创建阶段只询问名称和语言等必要信息，发布版本、作者、
许可证、主页与标签继续由词典设置维护。目标软件未启动时任务仍保存为“未连接”，不会把连接失败
误报成创建失败；之后显式重连仍显示具体原因。

探针管理列表也已收敛为二态连接语义：运行与暂停都显示“已连接”，其他持久状态显示“未连接”；
应用重启会把旧的运行、暂停或中断记录恢复为未连接，不再制造一排历史故障。任务行只保留名称，
程序位置与 Adapter 技术明细进入可悬浮、可聚焦的详情入口。创建后的首次连接即使失败，只要任务
已经持久化，创建 Modal 仍会关闭并进入详情，同时保留具体连接错误。

混合 Placement 的启动也已改为部分成功语义：Target Process 原位 Hook 与 Isolated Worker 观察器会
分别尝试激活；任一侧成功即可保留会话，失败侧的能力进入 Failed，只有全部 Placement 都失败才拒绝
连接。这样仅支持 UIA 观察的目标不会再被同组选中的不兼容 Native Hook 阻断。

同一 Target Process 内的多条候选 Hook 也已改为部分成功语义。Qt、GTK 等不适用于当前进程的技术只
标记自身失败，不再撤销已经成功的 GDI、GDI+ 或 DirectWrite；只有全部原生技术都失败时才拒绝连接。
注入 Runtime 通过独立激活回执上报真实成功集合，Session 不会把未激活技术冒充为 Active。这样用户
取消 UIA/Console 观察器后，健康的实时写回技术仍可单独建立连接。

`runtime.component_incompatible` 的界面文案不再猜测“刚升级过”或要求重启目标软件，而是明确说明
所选探针技术不适用于当前软件，并引导用户调整技术。HandBrake 类 WPF 目标当前仍只能归为“仅采集
原文”，不能因 UIA 连接成功显示成实时翻译支持。

Probe 连接状态现已使用实际会话 ACK，而不是用户勾选的 Adapter 能力推断。桌面 Runtime 只汇总宿主
真正确认 Active 的 Feature；当前连接据此显示“可直接替换 / 仅采集 / 无信号”。“仅采集”会明确说明
目标软件界面不会被修改；能力状态只属于本次连接，释放连接或重启后不会写进可恢复任务配置。

条目状态也不再把 Dictionary 未命中含糊显示为“待翻译”，而是明确显示“词典未命中”。用户在 Probe
内保存译文后，如果下一代预览没有成功发送到目标 Runtime，界面会说明“词典已保存，但目标界面可能
仍显示旧译文”，不再误报成停止失败。最终应用回执复核也已完成：GDI 可返回 API 状态，但 Qt/GTK
绘制入口没有返回值，而且 API 接受调用仍不等于像素可见，因此不存在可信的跨 Adapter 最终成功状态。
运行诊断现将 `Matched + Replaced` 明确显示为“替换决策已生成”，不再显示“已替换”。

Qt Widgets `QPainter::drawText` 绘制链现已完成。Qt 5/6 MSVC x64 ABI 分支均通过本地 QImage 合成
宿主的像素替换、同进程第二代 Dictionary 更新与停用恢复；首个授权目标因静态链接 Qt 在注入前被
安全拒绝，没有退化成软件专用签名扫描。随后动态链接 Qt 6 Widgets 真实目标完成可见验收：菜单原文
按 Qt 助记符语义命中字典，首版译文、第二代热更新译文和停用后的原文恢复均可见，诊断同时记录
`Matched + Replaced`。Adapter 已进入正式 Runtime Bundle 与中文目录，边界明确为动态链接 Qt 5/6
Widgets；QML、静态 Qt、富文本与静态文字缓存仍不在承诺内。

WinUI 3 / MRT Core 三层诊断也已完成。显式 `ResourceLoader.GetString` 与声明式 `x:Uid` 分别命中
`MrmLoadStringResource` 和 `MrmLoadStringOrEmbeddedResourceByIndex`，合成宿主可按 MRM 配对分配器完成
精确替换，并在停用后重新加载窗口恢复原文。但 `x:Uid` 是加载时赋值，已显示控件不能通用地即时刷新或
回滚；同时两条路径并不存在一个共同的窄入口。因此它不进入当前实时翻译 Bundle，只保留为未来可选的
“加载时资源替换”能力。

动态 GTK 3 / Pango 绘制链也已完成。Adapter 只挂公开 `gtk_render_layout`，读取原始 UTF-8 后复制
`PangoLayout` 并在副本上应用 Dictionary 译文；普通文本与全范围样式可替换，markup、mnemonic、link 等
局部 byte-index 样式安全放行。离屏像素合同、第一代/第二代热更新、停用恢复、无模块拒绝激活和正式控制器
注入链均通过，Adapter 已进入正式 Runtime Bundle 与中文目录。GTK 4 标准控件依赖私有 snapshot 入口，
继续维持 No-Go。

此前运行时 Roadmap 将 Observation Stream、跨启动 Binding 和共享 Hook 提到当前主线；这些工作不能
增加可翻译软件数量，现已退回后续 Roadmap。尚未提交的 Observation Stream 实现已撤回，现有简单
Workflow/Probe 互斥继续保留。

首个“真实软件缺口驱动”目标也已完成验收。授权 WPF 工具加载 .NET/WPF、DirectWrite 与 Direct2D；
正式 UIA Worker 可健康观察 72 条唯一公开文本，但 GDI、DrawText、GDI+、Direct2D DrawText 和本地
DirectWrite TextLayout 六条可写回路径在持续重绘中全部零命中。WPF 因此不进入当前 Native Bundle；
后续已选择 UIA 取词与外部译文呈现路线并转入 Roadmap，当前不建设 Managed WPF Agent。

下一真实目标的 Go/No-Go 已完成。Scintilla 自身确认使用 DirectWrite 模式；编辑区完整测试文本在正式
Runtime Bundle 中完成首代 Dictionary 替换、同进程第二代热更新和停止恢复。真实验证同时暴露并修复
了 Scoop `current` 目录链接：软件登记路径与进程报告路径现在都会解析为同一真实可执行文件身份，
不会再把已运行实例误判为未启动。

桌面端到端复核随后发现，先前仍在运行的开发 App 使用的是生产接入前的旧 Bundle，因此用户实际只能
看到 UI Automation 观察，不能据此证明 DirectWrite 写回。开发 Bundle 已重新构建为 8 个 Adapter，
DirectWrite 制品与清单摘要一致；既有 Probe Run 保留创建时的 Adapter Plan，在显式加入 DirectWrite
并使用新构建重连后，用户已确认编辑区实时翻译正常工作。

同一真实目标的级联菜单也完成了边界复核：一级和二级菜单均能由 GDI `ExtTextOutW` 与 USER32
`DrawTextW` 观察，二级菜单词条在干净、同版本的目标实例中取得明确的 `Matched + Replaced`。因此
级联菜单不是新的 Adapter 缺口。开发期替换 Runtime DLL 后，如果目标进程仍加载旧 DLL，旧译文可能
暂时可见但新观察不会继续回传；这种跨二进制版本升级仍要求重启目标软件，不能误判为二级菜单不支持。

重新恢复覆盖主线后完成了有界本机候选盘点，但当前没有同时满足“已授权、可重复运行、属于尚未覆盖
文字栈”的目标；已配置候选要么属于既有覆盖路线，要么当前不可用于测量。未经本机配置授权的运行软件
不读取界面文字、不注入，也不为了推进议程扩大隐私边界。

执行方向已再次确认：UI Automation 继续只作为 `ObserveOnly` 查看/取词能力，不进入实时写回成功；
覆盖主线下一项必须由真实缺口驱动新的 `TextReplace` Adapter，交互式取词与外部呈现仍留在 Roadmap。

SDL3_ttf 外部真实目标筛选与官方 `showfont` 技术 smoke 已完成。外部开源软件中仍未找到同时满足动态
链接、公开 `TTF_Text` 绘制链与可重复 Windows 构建的目标；FreeRDP 的可见文字仍走 Surface/Texture
缓存，因此生产 Adapter 继续暂缓。官方动态 `showfont` 则证明 `TTF_DrawRendererText` 能取得完整 UTF-8
原文，瞬时译文 Text 可复制公开布局属性完成首代、同进程第二代和停用恢复的可见像素切换；中央缓存
纹理文字保持原文，覆盖边界符合源码。该 smoke 只证明技术 seam 可实施，不计入真实软件支持。

后续动态 C 绘制栈复核与 Allegro/Open Surge 实机 Go/No-Go 也已完成。Open Surge 正式 Windows 包没有
独立 DLL 或 Allegro PE import，属于静态/monolith；显式动态源码构建虽成功导入并实际加载 Allegro core、
font 与 TTF 模块，公开文字导出也存在，但预构建 `allegro_physfs` 与目标自身使用了不共享状态的 PhysFS
实例，初始化即在资源读取路径崩溃。继续需要项目外定制重建整套依赖，已不再是可重复真实目标，因此
Allegro 以 No-Go 收口，没有进入文字 Hook 或生产 Adapter。raylib 字符串入口仍受真实目标 ASCII glyph
atlas 阻断，直接替换中文只会回退为问号。

raylib fallback Font/atlas 原型与生产接入现已完成。纯状态机的 23 个转移先证明 GPU context、渲染线程、
代次交换、帧尾回收与关闭顺序；生产 Adapter 随后只挂动态 raylib 5.5 兼容 `DrawTextEx`、`EndDrawing`
和 `CloseWindow`，用公开 skyline atlas、48px fallback 栅格、mipmap 与双线性过滤绘制逐字形验证通过的
译文。无模块、缺导出、无可用字体、缺字、非法 atlas、非 UTF-8、过长文本、重入或线程不一致均安全
放行，静态链接、复制绘制实现和业务纹理缓存明确不在覆盖范围。

正式九 Adapter Bundle 已在 Musializer 项目自有动态 hot-reload 构建中通过：首代和第二代各取得 256 条
`Matched + Replaced`，像素分别显示两代完整中文；停用恢复英文，重新激活后在 atlas 仍活跃时直接关闭
也能干净退出。真实字体、截图、日志与构建物仍只保存在本机私有目录。

raylib 之后的候选复核也已完成。FLTK 有公开 UTF-8 `fl_draw`，但 Windows shared build 默认关闭、
C++ DLL 要求编译器匹配，且当前没有外部动态真实目标；SFML 在绘制时只剩缓存 vertices，Open Hexagon
显式静态链接；Dear ImGui 的 Tracy 候选也静态集成。本轮没有候选进入 Hook 或新增生产 Adapter。
UI Automation 继续只作为查看/采集能力。

Adapter 技术说明入口也已补齐。正式清单新增可选 `documentationUrl` 展示元数据，当前 10 个技术条目
全部指向官方 HTTPS 资料；Runtime 会拒绝相对路径、非 HTTPS、带凭据或缺少主机名的显式 URL，同时
兼容缺少字段的旧 Bundle。帮助页为每行提供“查看文档”，由受限 Tauri Opener 交给系统默认浏览器，
不展示裸 URL、DLL 目标或内部 Adapter ID。

Unity / Unreal Engine 的引擎感知可实施性已完成一手资料复核。两者都在内部保留过完整字符串，但任意
第三方 Shipping 游戏没有稳定外部 ABI；不能从引擎名推导覆盖率，更不能承诺 90%。Unity 只进入严格
限界的 Windows x64 Standard UI 原型候选，Mono 与 IL2CPP 必须分别立证；UE 暂留研究态。任何生产
实现开始前，每个技术分层必须先有至少两个无反作弊、未预装插件、可重复的外部 Shipping 真实目标，
并另有负例证明不支持路径会可靠拒绝。

公开实现源码交叉验证进一步确认：Unity Mono/IL2CPP 可沿运行时元数据定位 TMP、uGUI 与 UI Toolkit
setter，不必天然退化到逐游戏函数偏移；但只挂 setter 会漏掉附加前已有文字，直接改 retained widget
也不满足停止恢复合同。两份正式 Windows 候选的只读盘点现已通过：均为 x64 Mono，分别使用 Unity
2021.3 与 2022.3；业务程序集覆盖 TMP 动态 setter，后一份还实际引用 uGUI 与 Unity Localization。
两个 runtime 需要的 `mono_*` 元数据入口跨代存在，没有发现符号文件或常见反作弊制品。候选程序仍未
注入。授权 runtime-only smoke 已确认：第一份样本能稳定显示初始菜单、关卡选择和动态分数；第二份
通过公开参数稳定进入无头显 Viewer 与完整 Monoscopic 工作区，并正常退出。第二份的语言切换、两份
样本的外部 attach/停止恢复和可靠负例仍缺，因此样本 gate 仍未通过。

IL2CPP 第一轮正式包静态门禁也已完成：Unity 2021.3.5 与 Unity 6.0 的两份 Windows 包均为 x64
IL2CPP，`global-metadata.dat` 分别为 metadata 29 与 39；`GameAssembly.dll` 分别导出 235 与 241 个
`il2cpp_*` API，抽查的 domain、assembly、image、class、method、invoke、string 与 thread 入口全部存在。
两包 metadata 均含 TMP/uGUI 标记，没有 Mono runtime、符号文件、开发输出目录或常见反作弊文件名，
native debugger wait 关闭。盘点没有启动候选；文件名级保护扫描只能说明“未发现”。第三份 Unity 6
release 随后承担替换复核。

授权 runtime-only smoke 随后纠正了候选质量：Unity 2021.3.5 游戏连续两次启动，标准 TMP 主菜单、
High Score、Start/Quit 可见，两次都加载发布包内的 `UnityPlayer.dll` 与 `GameAssembly.dll`、没有
Glyphshift 模块，并经窗口关闭正常退出。Unity 6 桌面应用虽能显示完整菜单、工具栏和首启对话框，
但成品界面明确标记 `Development Build`，而且首启模态框阻断窗口关闭后需要强制清理；它因此不能承担
Shipping-like 正例，本轮不重复启动。两者都没有注入；首次 runtime smoke 当时只留下一个合格运行候选。

替代 Unity 6 release 随后通过同一门禁：正式 Windows 包为 x64 Unity 6000.3.4f1 IL2CPP，metadata 39
有效，导出 241 个 `il2cpp_*` API，核心入口与 TMP/uGUI 标记齐全；未发现 Mono、PDB/MDB/DBG、开发
输出目录、player-connection 调试键或常见反作弊文件名。连续两次启动都显示完整编辑器搜索、地形分类
和工具标签，运行模块与发布包一致、没有 Glyphshift 或 Development Build 标记，并经窗口关闭正常退出；
选定异常扫描为零。后续定向运行把两个正例的动态路径固定下来：Unity 2021 LTS 游戏从 `SCORE: 0`
稳定推进到 `SCORE: 248`，并显示 `Current Score: 248`；Unity 6 桌面应用把可见搜索框从占位文本改为
`wall`。前者公开源码每帧写入 `TextMeshProUGUI.text`，后者公开源码另有把 tooltip 片段排序、换行拼接
后写入 `TMP_Text.text` 的真实组合字符串路径。两次均无注入、无 Glyphshift 模块并正常退出。

可靠负例也已补齐。一份公开 Windows x64 Unity 2022.2 Mono 牌类游戏正式包只包含 NGUI `UILabel`、
`UIFont`、`BMFont` 与 `NGUIText`，业务代码把可见中文菜单和数值写入 `UILabel.text`；同 tag 的
`UILabel.OnFill` 再调用 `NGUIText.Print` 生成 vertices/UV/colors，字体来自 atlas/material。包内业务
程序集没有 TMP、TextMeshPro 或 uGUI 标记，runtime-only smoke 显示完整中文菜单、无 Glyphshift 模块并
正常退出。因此它可承担“Unity 引擎存在但 Standard UI lane 必须拒绝”的进程级 Mesh/atlas 负例，
也补上 Mono lane 的负例方向。但它不能承担 IL2CPP 专属负例：IL2CPP Adapter 可仅凭 Mono 后端拒绝，
尚未验证同为 IL2CPP 时能区分标准 UI 与 NGUI/自绘 Mesh。IL2CPP attach gate 因此仍未通过，也没有建立
生产 Adapter。

IL2CPP 同后端负例随后做了一次有界公开候选复核。唯一具有正式 Windows x64 release 和明确 IL2CPP
构建脚本的候选，仓库实际携带 TextMeshPro 与 `UnityEngine.UI`，仍属于 Standard UI 正例，不能承担
自绘 Mesh/Sprite/texture 负例；另一搜索方向没有得到同时具备同源仓库、Windows 成品、IL2CPP 与自绘
文字证据的目标。本轮没有下载或启动新候选，也不再用无界搜索阻塞主线。

Mono 第二候选的 Localization 路径也已复核。发布 tag 的 Localization Settings 注册了
`CommandLineLocaleSelector`，参数为 `-language=`，界面 Locale 按钮也直接更新 `SelectedLocale`；这能
证明真实 Localization 技术和切换入口存在。但英语与简体中文各一次 7 秒、简体中文一次 15 秒的无注入
平面运行均只显示无可判读文字的场景，三次都加载 UnityPlayer + Mono、没有 GameAssembly/Glyphshift
并正常退出。因此它不能承担“可见语言切换”正例，Mono attach gate 仍未通过。

替代 Mono 候选随后补齐了缺口。正式 Windows zip 是 x64 Unity 6000.3.9f1 Mono Player，包含
`MonoBleedingEdge`、Managed 业务程序集、Unity Localization、TMP 与 uGUI，没有 GameAssembly、IL2CPP
metadata、调试符号或常见反作弊文件名；业务程序集真实引用 `NextLanguage`、`SelectedLocale`、
`LocalizationSettings` 与 `TMP_Text`。默认中文运行显示“下载资源 / 简”，将该测试应用自己的
`AppLanguage` 偏好在 `try/finally` 内临时设为英语后，同一欢迎页显示“Download Res / En”；原偏好随后
逐字节恢复。两次都加载 UnityPlayer + Mono、没有 Glyphshift 并正常退出。Mono lane 至此拥有两个跨
Unity 代际的 Standard UI 正例和一份 NGUI Mesh/atlas 负例，真实样本 gate 已通过，但尚未 attach。

Unity Mono observe-only 原型现已完成。host-independent Observer 与未入 Bundle 的 native Adapter 用 25 项
实际使用的 late-attach 导出完成 metadata/JIT 门禁、`CanvasUpdateRegistry.PerformUpdate` 主线程 dispatch、
一次性对象枚举和 TMP/uGUI `set_text` detour。GC handle 已改为指针宽度 `*_v2`；激活 ACK 还必须等到
主线程枚举出真实标准 UI 对象，不能再用 Unity 自带程序集的类型存在冒充支持。初始 Observation 会缓存到
Target Runtime active 后再发布，避免 ACK 前丢失。

合成宿主取得附加前/后 4 条预期文字并验证停用；两个 package 共 23 个定向测试与各自 Clippy 通过。正式
Controller/Target Runtime 部署链在两个最终正例分别采集 10 条与 15 条唯一非空原文，额外跨代正例采集
389 条；全部停用并正常退出。NGUI-only 同后端负例被 live-object gate 拒绝且正常退出。动态路径仍只
承诺属性 `set_text`，不把 TMP `SetText(...)` 多重 ABI 冒充为已覆盖，也尚未提供 Dictionary 写回。

Unity Mono Standard UI TextReplace 原型随后完成，并在发布前把 Observe-only 两个 package 收敛为一个
host-independent `standard-ui` 状态内核和一个 native Adapter，避免重复持有 Mono FFI、GC handle 与 JIT
detour。新增 `mono_string_new_utf16` 后共使用 26 项导出；managed string 只在已验证主线程创建并由强
`*_v2` handle 覆盖 setter 调用。状态机保留最新业务原文、当前译文与 Dictionary generation，支持初始
写回、属性 setter、第二代热更新、停用恢复以及三秒超时后的延迟主线程恢复。

32 个相关定向测试与 Clippy 通过。确定性 Mono 宿主覆盖正常/延迟恢复；正式部署链在两个跨代 Standard UI
正例分别捕获 10 / 15 条唯一原文，首代译文、第二代译文和停用后的原文恢复均有可见证据且目标正常退出。
NGUI-only 同后端负例继续拒绝。

生产接入也已完成。Debug/Release Runtime Bundle 现在都包含该 native Adapter；Desktop Runtime 实际打开
两种 Bundle 后均将其识别为 Windows x64 Target Process 的 `TextReplace` 技术。前台技术卡显示精确边界并
链接 Unity 官方 Mono 手册，Playwright CLI 6/6 通过。正式 Debug Bundle 又在一个 Standard UI 正例完成
首代、generation 2 与停用恢复的可见验收并正常退出，NGUI-only 负例继续拒绝。

## Next

- Unity Mono Standard UI 已完成生产接入并保持窄边界。IL2CPP 同后端自绘 Mesh/Sprite/texture 负例仍缺，
  一次有界公开
  筛选也没有合格成品，在出现明确候选前不继续广泛搜索，更不能用 Mono 后端拒绝冒充 UI 技术拒绝。
- 等待新的、明确授权且可重复的 Windows 真实目标；按
  [raylib 之后的候选复核](references/post-raylib-next-adapter-primary-source-review.md)先查 PE import、运行模块
  与公开文字导出，固定可见词条命中后才恢复 Adapter 原型。
- DirectWrite TextLayout 的底层生产接入已完成；WPF 等不经过公开 TextLayout 绘制入口的目标仍不在
  覆盖范围，不因加载 DirectWrite 模块而扩大支持声明。
- 下一轮继续用“真实软件缺口 → 一手入口证据 → 有界原型 → 可见写回”的顺序选择文字技术，不以
  Adapter 数量或框架名称驱动实现。
- [交互式取词 Seam](../runtime-capability-roadmap/slices/interactive-text-acquisition-seam.md)的仓库内后端合同
  已完成；热键、浮层、生产 OCR 与真实翻译服务仍留在 Roadmap，等待独立产品选择与授权目标证据。

## Progress

- 已重新确认产品主线是“探针发现原文 → Dictionary 提供译文 → Adapter 实时写回”，而不是通用观察
  数据平台。
- 已将采集能力与实时翻译能力分级；UIA、Console 等 observe-only 能力保留，但不计入实时翻译覆盖。
- 已用合成目标验证 DirectWrite 两种布局绘制入口，并在授权真实目标上完成零命中否决；没有把成功
  注入或加载模块误报成翻译支持。
- 已验证 Probe 译文编辑会产生下一代预览发布；桌面 Shell 单元测试 38/38 通过。
- 已完成[探针与词典创建流程收敛](slices/creation-flow-distillation.md)：取消后不再保留任务、Adapter
  或词典草稿，创建界面不再暴露内部词典 ID 与低频发布元数据；桌面 Playwright 53/53 通过。
- 已修复探针创建后首次连接失败导致 Modal 残留的问题，并把管理列表压缩为二态连接状态与悬浮技术
  详情；针对性 Playwright 3/3、组件与视觉合同 5/5、Capture 与 Desktop Shell 单元测试 54/54 通过。
- 已修复混合 Placement 的全有或全无启动：Native 激活失败时健康的 UIA Worker 仍会保持会话，失败
  能力不会冒充 Active；Isolated Worker Host 11/11、Desktop Runtime 13/13 常规测试通过。
- 已修复同一 Target Process 内多条 Native Hook 的全有或全无启动：不可用候选不再撤销健康 Adapter，
  Controller 会回传真实激活集合并由 Session 分别标记 Active / Failed。GDI 成功与 Qt 无模块失败的
  原生回归 1/1、Controller/Host 针对性回归 3/3、完整 Rust workspace 与架构检查通过；探针相关
  Playwright 5/5 通过。
- 已移除组件不兼容错误中“升级后重启目标软件”的无依据推断；针对性 Playwright 回归 1/1 与前端
  生产构建通过。
- 已修复“请求能力冒充已激活能力”：Session 只公开宿主确认 Active 的 Feature，Desktop Runtime 按
  实际 ACK 汇总，Probe 将其临时投影为“可直接替换 / 仅采集 / 无信号”；仅采集说明明确告知不会修改
  目标界面。Session 针对回归 1/1、Desktop Runtime 13/13、Desktop Shell 40/40、Probe Playwright
  10/10 与前端生产构建通过。
- 已把无 Dictionary 译文的条目明确标记为“词典未命中”；预览发布失败现在说明译文已经保存、但目标
  Runtime 未收到更新，不再伪装成停止失败。Desktop Shell 41/41、Probe Playwright 10/10 与格式检查
  通过。
- 已否决跨 Adapter 的“最终写回成功”回执：部分绘制入口没有返回值，有返回值也不能证明像素可见；
  运行诊断改为“替换决策已生成”并明确说明证据边界。WPF 外部翻译回退继续由交互式取词 Roadmap
  承接，当前不建设 Managed Agent。前端生产构建、针对性 Playwright 1/1 与 Impeccable 机械检测通过。
- 已验证 GDI+ Dictionary 在同一目标进程内发布第二代并立即替换，停止后恢复原文。
- 已比较 Qt、WinUI/MRT、WPF 与 Web 桌面壳入口；Qt 绘制链进入下一有界原型，其余候选保留在
  调研结论中，不并行扩张实现面。
- Qt Painter 纯逻辑 6/6 通过；既有 Native Host 常规合同 8/8 通过，另有 1 项无 Qt 模块 fail-open 通过；
  显式配置的 Qt 5 与 Qt 6 本地像素合同各 1/1 通过，并逐一覆盖 4 个核心 overload。原始本机日志只
  保留在本地证据目录。
- 首个授权 Qt 6 目标在注入前确认为静态链接 Qt；没有动态 Qt 模块或文字导出，因此按既定边界安全
  否决，没有把产品适配器降级成单软件、单版本签名扫描。
- 动态链接 Qt 6 Widgets 目标完成真实可见验收：首版与热更新译文各有 2 次
  `Matched + Replaced`，停用后恢复原文；正式 Bundle 复测捕获 40 次绘制并命中替换 2 次。
- Qt Painter 已加入正式 Bundle 构建与 Adapter 中文目录；完整仓库测试通过，本机截图与原始日志仍只
  位于忽略的本地证据目录。
- WinUI 3 合成宿主确认 `ResourceLoader.GetString` / 显式 C API 命中 `MrmLoadStringResource`，默认
  `x:Uid` 命中 `MrmLoadStringOrEmbeddedResourceByIndex`；三类文本均完成精确替换，停用后重新加载恢复
  原文。由于旧控件不能即时刷新或回滚，该能力未进入真实目标 smoke 与正式 Bundle。
- GTK 3 / Pango 纯逻辑 6/6 与无模块拒绝激活 1/1 通过；真实 Pango 属性范围、普通/全范围样式替换、
  局部样式放行和停用恢复均由像素合同确认。正式 Bundle 的后台注入复测中，第一代与第二代 Dictionary
  各命中替换 31 次，停止后恢复原文。
- GTK/Pango 后续候选已重新排序：`SetWindowTextW`、`LoadStringW`、Java agent 与常见静态 C++ 绘制库
  均不满足当前安全热更新合同；动态 Tk 8.6 只进入相对 GDI 的增量诊断，不因框架名称直接立项。
- Tk 隐藏双路宿主确认 GDI 与公开 Tk 路径都取得 3/3 个完整 source；两代中文译文均与原生 Tk 像素
  一致并可停用恢复。Tk 没有形成独立覆盖价值，实验实现已撤回且未进入 Bundle。
- SDL2_ttf 的稳定 UTF-8 C API 已复核，但其 Surface/Texture 缓存语义不能满足当前实时恢复合同；
  SDL3_ttf 的 `TTF_Text` 绘制链进入 Engine-aware Roadmap，等待真实目标触发，不先建设空置 Adapter。
- Web transport 与 DOM ownership 原型虽已通过，但产品适用性复核确认其依赖目标宿主主动开放接口；
  已停止自有目标 DOM 验收，不把 Host-assisted 能力计入通用实时翻译覆盖。
- 完成授权 WPF 真实目标验收：UIA 观察 72 条公开文本且健康；六条 Native 写回入口全部完成有效激活
  但零命中。WPF 当前归为“仅结构化观察”，后续由交互式取词与外部译文呈现承接。
- 完成下一真实目标一手资料筛选：Notepad++ 的 Scintilla 编辑区具有明确
  `IDWriteTextLayout → DrawTextLayout` 路径，进入唯一下一实机 Go/No-Go；VS Code 与 Godot 仅保留为
  Web/引擎文字栈边界，不并行立项。
- 完成 Scintilla DirectWrite 编辑区 Go：正式 Adapter 覆盖直接与 Compatible Bitmap 两类
  `DrawTextLayout`，首版和热更新代次均取得 `Matched + Replaced`，停止后恢复原文；复杂格式继续
  fail-open。正式 Bundle 真实复测首版命中 16 次、第二代命中 18 次。
- 修复 Windows 进程发现对目录链接两侧路径身份不一致的问题；通过 Scoop `current` 路径启动的真实
  目标已完成发现与 25 次稳定替换命中，控制器路径合同 6/6 通过。
- DirectWrite 纯逻辑 4/4、Native 绘制合同 1/1、Clippy 零警告、架构检查和完整 Rust workspace 测试
  均通过；正式 Bundle 构建成功。运行诊断文案的桌面生产构建与针对性 Playwright 1/1 也通过。
- 已纠正“独立 Runtime 通过即等于当前 App 可用”的错误结论：旧开发 App 的 Bundle 只有 7 个 Adapter，
  当前 Probe 也没有 DirectWrite。重新构建后根 Bundle 为 8 个 Adapter，DirectWrite 制品摘要验证通过，
  新桌面进程已启动；用户随后确认加入 DirectWrite 的 Probe 可正常实时工作。
- 已验证真实级联菜单的二级词条同时命中 GDI ExtTextOut 与 USER32 DrawText；其中 ExtTextOut 二级词条
  取得 2 次 `Matched + Replaced`。级联菜单不新增 Adapter；开发构建替换已加载 Runtime DLL 时需重启
  目标进程，避免旧译文残留被误认为当前探针仍在连接。
- 完成 [SDL3_ttf 真实目标候选筛选](references/sdl3-ttf-real-target-candidates.md)：FreeRDP 等外部候选的
  可见文字仍落入 Surface/Texture 缓存，未满足生产门槛；官方 `showfont` 只作为确定性技术宿主。
- 完成 [SDL3_ttf `showfont` 技术 smoke](slices/sdl3-ttf-showfont-technology-smoke.md)：动态模块实际加载，
  完整 UTF-8 source 命中；第一代、第二代与停用恢复均取得可见像素证据，缓存纹理区域按预期不变。
  技术 seam 为 Go，但生产 Adapter 在外部真实目标出现前仍为 No-Go。
- 完成 [SDL3_ttf 之后的动态 C 文字入口复核](references/post-sdl3-next-dynamic-c-seam-review.md)：raylib
  真实候选只烘焙 95 个 ASCII glyph；Allegro/Open Surge 进入唯一下一实机门槛。
- 完成 [Allegro / Open Surge 真实目标 smoke](slices/allegro-opensurge-real-target-smoke.md)：正式包确认
  静态/monolith；显式动态构建通过 PE import、运行模块和公开导出检查，但目标在 PhysFS 状态隔离处
  初始化崩溃，未到文字绘制即按 No-Go 停止，没有新增生产 Adapter。
- 完成 [raylib fallback Font/atlas 原型](slices/raylib-fallback-font-atlas-prototype.md) 与
  [raylib `DrawTextEx` 生产 Adapter](slices/raylib-draw-text-adapter.md)：正式 Bundle 两代各取得 256 条
  `Matched + Replaced`，两代中文、停用恢复和活跃 atlas 关闭均有真实目标证据；完整 workspace、全
  workspace Clippy 与架构检查通过。
- 完成 [raylib 之后的下一 Adapter 第一方资料复核](references/post-raylib-next-adapter-primary-source-review.md)：
  FLTK 因缺外部动态真实目标暂缓，SFML 与 Dear ImGui 因当前真实候选静态集成或绘制阶段已无原文而
  No-Go；没有为数量新增空置 crate、目录选项或 Bundle 项。
- 完成 [Adapter 技术文档链接](slices/adapter-technical-documentation-links.md)：清单、Runtime 与桌面快照
  贯通可选 `documentationUrl`，正式目录 10/10 使用官方 HTTPS；帮助页通过系统默认浏览器打开，桌面
  Capability 只放行当前官方域名。完整 workspace、Clippy、架构、关键 Playwright 与双尺寸视觉回归通过。
- 完成 [Unity / Unreal Engine 游戏文字 Adapter 可实施性复核](references/unity-unreal-engine-adapter-feasibility.md)：
  Unity Mono、IL2CPP、Localization 与标准/自研 UI，以及 UE `FText`、Slate/UMG、Canvas、shaping、
  Shipping 链接和反作弊边界已分层；结论是不按引擎名承诺覆盖，真实样本 gate 通过前不新增生产实现。
- 已用匿名公开实现源码交叉验证 Unity 运行时元数据与标准 UI setter 路线，同时确认初始枚举、所有权和
  停止恢复仍是 Glyphshift 必须自行补齐的合同；Godot/UE 的字节模式路线继续按固定版本 Recipe 对待。
- 已筛选 Unity 外部样本：保留一份小型桌面标准 UI 候选和一份跨代际条件候选，拒绝把缺少关键内容的
  Alpha 或仅有工程发布证据的样例算作 Shipping 正例。
- 两份正式 Windows 包的只读盘点已完成：x64、Unity 2021.3/2022.3、Mono runtime 与 TMP/uGUI/
  Localization 业务引用均由发布二进制确认；盘点阶段未启动或注入，也没有把文件名扫描误写成无保护
  证明。
- runtime-only smoke 已确认第一份样本的初始菜单、关卡选择与动态分数；第二份用公开参数进入无头显
  Viewer 和完整 Monoscopic 工作区。两者都正常退出且未注入；Localization 可见切换仍未完成。
- Unity IL2CPP 两份正式包的静态门禁已完成：跨 Unity 2021 LTS / Unity 6，均为 x64，metadata 29/39
  与核心 `il2cpp_*` 导出有效，TMP/uGUI 标记存在；未启动、未注入，也未运行无关仓库测试。下一步必须
  先取得单独授权，才做 runtime-only smoke。
- Unity IL2CPP 首轮 runtime-only smoke 没有加载 Glyphshift：Unity 2021 LTS 候选两次启动、主菜单
  可见并正常退出；原 Unity 6 候选因明确是 Development Build 被否决，首启模态框阻断关闭后已强制
  清理。本轮未运行仓库测试。
- 备用 Unity 6 IL2CPP release 已补位：x64 Unity 6000.3.4f1、metadata 39、核心导出与 TMP/uGUI 标记
  通过静态门禁；两次启动均显示完整编辑器 UI、没有 Development Build/Glyphshift 模块并正常退出。
  IL2CPP 两个跨代际运行候选已齐，但动态/格式化路径和可靠负例尚未完成，仍未 attach。
- Unity Mono 替代 Localization 候选通过：x64 Unity 6 Mono 成品的业务程序集真实引用 TMP/uGUI、
  `LocalizationSettings` 与 `NextLanguage`；欢迎页在中文和英语偏好下分别显示对应文案，目标均正常退出、
  无 Glyphshift 模块，测试偏好已恢复。结合既有动态 TMP 正例和 NGUI 负例，Mono 样本 gate 已通过。
- Unity Mono Observe-only 原型已完成：纯 Observer 与 native 原型共 23 个定向测试通过；合成宿主和
  正式部署链证明真实对象 ACK、初始 Observation、属性 setter、停用和进程退出。两个最终正例、一个
  额外跨代正例均采集到非空原文，NGUI-only 同后端负例可靠拒绝；该结论随后由 TextReplace 原型继承。
- Unity Mono Standard UI TextReplace 原型已完成：未发布的 Observe-only package 已直接收敛改名；32 个
  定向测试与相关 Clippy 通过。两个跨代真实正例的首代、第二代和停用恢复均可见，NGUI-only 负例继续
  拒绝；当前转入生产 Catalog/Bundle 接入，不扩大到 IL2CPP、UI Toolkit、NGUI 或 `SetText(...)`。
- Unity Mono Standard UI 生产接入已完成：Debug/Release Bundle、Desktop Catalog、中文技术说明、官方文档
  入口、architecture/native-host 合同与 focused Playwright 均通过；正式 Bundle 正例可见更新/恢复，
  NGUI-only 负例拒绝。该技术现在进入实时翻译覆盖，但只承诺 Windows x64 Mono TMP/uGUI 属性 setter。

## References

- [产品契约](../../../PRODUCT.md)
- [领域语言](../../../CONTEXT.md)
- [Windows 软件支持分级](../runtime-capability-roadmap/references/windows-software-support-and-console-gap.md)
- [后续运行时能力 Roadmap](../runtime-capability-roadmap/index.md)
- [下一 Adapter 第一方资料评审](references/next-adapter-primary-source-review.md)
- [WinUI 3 / MRT Core 原生入口诊断](references/winui-mrt-primary-source-diagnostic.md)
- [GTK/Pango 实时写回候选评审](references/gtk-pango-adapter-primary-source-review.md)
- [GTK/Pango 后续入口评审](references/post-gtk-next-adapter-primary-source-review.md)
- [Tk 之后的实时文字入口复核](references/post-tk-runtime-seam-review.md)
- [SDL3_ttf 真实目标候选筛选](references/sdl3-ttf-real-target-candidates.md)
- [SDL3_ttf `showfont` 技术 smoke](slices/sdl3-ttf-showfont-technology-smoke.md)
- [SDL3_ttf 之后的动态 C 文字入口复核](references/post-sdl3-next-dynamic-c-seam-review.md)
- [Allegro / Open Surge 真实目标 smoke](slices/allegro-opensurge-real-target-smoke.md)
- [raylib fallback Font/atlas 原型](slices/raylib-fallback-font-atlas-prototype.md)
- [raylib `DrawTextEx` 生产 Adapter](slices/raylib-draw-text-adapter.md)
- [raylib 之后的下一 Adapter 第一方资料复核](references/post-raylib-next-adapter-primary-source-review.md)
- [raylib 之后的下一写回 Adapter 选择](slices/post-raylib-next-writeback-adapter.md)
- [Adapter 技术文档链接](slices/adapter-technical-documentation-links.md)
- [Unity / Unreal Engine 游戏文字 Adapter 可实施性复核](references/unity-unreal-engine-adapter-feasibility.md)
- [WPF 真实目标缺口验收](slices/wpf-real-target-gap.md)
- [下一真实目标候选筛选](references/next-real-target-candidates.md)
