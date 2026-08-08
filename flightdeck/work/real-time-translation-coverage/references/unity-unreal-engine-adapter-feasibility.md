# Unity / Unreal Engine 游戏文字 Adapter 可实施性复核

Status: Complete

Date: 2026-08-07

## 结论

Unity 和 Unreal Engine 都在渲染前保留过完整字符串，因此“引擎感知 Adapter”在技术上并非不存在；
但第一方资料只证明这些入口能被**游戏开发者随项目一起编译或在开发构建中使用**，没有证明它们是任意
第三方 Shipping 游戏都开放的稳定外部 ABI。

本轮结论是：

- **不能承诺覆盖 90%，也没有可信的一手统计支持“大部分 Unity/UE 游戏可覆盖”这个说法。** 标准 UI
  组件的公共 API 只在代码已经合法运行于游戏内部时有价值，最困难的仍是如何稳定进入第三方 Shipping
  Player，以及如何跨引擎版本、脚本后端、链接模式和游戏定制保持兼容。
- **Unity 比 UE 更适合先做严格限界的 Windows x64 外部原型**，但必须把 Mono 与 IL2CPP 当成两条
  实现。先验证 Mono 不代表 Unity 已支持；IL2CPP 若必须逐游戏恢复符号或偏移，通用路线立即 No-Go。
- **UE 的语义 seam 很清晰，外部 ABI 却更差。** `FText`、UMG/Slate、`FSlateFontCache` 在引擎源码层
  都有完整文本，但 Shipping 可以把模块单体链接、启用全优化或 LTCG、不给最终用户符号；模块导出宏在
  monolithic 模式甚至为空。因此源码中存在函数不等于发布游戏中存在稳定可定位的 Hook 点。
- **合作集成与任意游戏适配必须是两个产品能力。** 随项目构建的 Unity/UE 插件可以高质量观察和替换；
  对任意第三方 Shipping 游戏则只能宣称一个经过真实矩阵验证的兼容集合，不能用“支持 Unity”或
  “支持 Unreal”概括。
- **观察和替换也必须拆开验收。** 观察到完整字符串可以驱动旁路翻译或悬浮层；修改 retained widget、
  本地化资源或游戏包会破坏绑定、缓存、签名或持久状态，不能因“能改”就称为可恢复的运行时替换。

推荐的产品结论不是 `Universal Unity/Unreal Adapter`，而是：

> `Unity Windows Standard UI feasibility` 先行；只有 Shipping-like Mono 与 IL2CPP 都证明跨版本、
> 免符号、无需逐游戏偏移后，才建立明确版本矩阵。UE 暂留研究态，等 Unity 的通用性门槛被证明后再做
> UE5 单版本、无反作弊、标准 Slate/UMG 的有界原型。

## “支持”的证据门槛

本文把能力分成四级，避免把理论注入当产品支持：

| 等级 | 定义 | 能否对用户称为支持 |
| --- | --- | --- |
| A：合作集成 | Adapter 由游戏开发者放入项目，随 Player/Shipping 构建并得到官方运行时对象 | 可以，但必须标为“开发者集成” |
| B：外部已验证 | 对未预装 Adapter 的第三方 Shipping 游戏，在明确版本、架构、无反作弊条件下完成观察、停止与恢复 | 可以，只能按兼容矩阵宣称 |
| C：理论可 Hook | 源码或 API 中存在完整字符串函数，但正式二进制未证明有稳定定位、调用和恢复方法 | 不可以 |
| D：渲染后推断 | 只剩 glyph、vertex、texture 或 framebuffer，再尝试还原文字 | 不属于引擎文字 Hook；应归到 OCR/图像路线 |

每一个 B 级声明至少需要：真实 Shipping-like 构建、初始和动态文本、两次启动、运行后附加、场景切换、
停止恢复、未知版本拒绝、无持久文件改写，以及目标没有反作弊/平台保护的证据。

## 总覆盖矩阵

| 引擎/路径 | 完整字符串存在的位置 | 第三方 Shipping 默认入口 | 外部观察可行度 | 临时替换与恢复 | 本轮判定 |
| --- | --- | --- | --- | --- | --- |
| Unity Mono + uGUI/TMP | 托管组件的 `text` / `SetText`，进入 mesh 前 | 没有官方“后装任意 Player”入口；Debugger/Profiler 要开发构建 | **条件中**：若能跨 LTS 使用同一运行时接入且无需逐游戏偏移 | **条件中低**：组件写回会被绑定/游戏逻辑覆盖；旁路 overlay 更安全 | **Go，仅原型** |
| Unity Mono + UI Toolkit | `TextElement.text` 和运行时 Visual Tree | 同上，且对象树与 uGUI 不同 | **条件中** | **条件中低** | 纳入 Unity 原型第二层 |
| Unity IL2CPP + 标准 UI | C# 语义仍存在，但程序集经裁剪、转 C++、AOT 编译进原生二进制 | 没有 CLR attach；正式版不应分发生成 C++/PDB 备份 | **低到未知**：必须先证明免符号、免逐游戏偏移 | **低**：原生 Hook、缓存与版本风险叠加 | Unity 产品化的硬门槛 |
| Unity Localization | `StringTable`、格式化前后字符串 | 该包可选；表通常经 Addressables/AssetBundle 加载 | **中**，适合作为第二数据源，不等于当前屏幕观察 | 资源改写不是临时恢复；运行时事件需要内部代码 | **Go，数据源；No-Go，冒充通用 Hook** |
| Unity 自研 Mesh/Sprite/视频字幕 | 游戏自定义层；渲染层只剩 geometry/texture | 无统一入口 | **低/无** | **无通用方案** | **No-Go** |
| UE UMG `UTextBlock` / Slate `STextBlock` | `FText`、Slate Attribute、文本布局和 shaping 前 | 插件需项目启用/编译；外部无稳定反射/插件 attach ABI | **低到未知** | **低**：`SetText` 可破坏绑定；draw-time seam 无稳定外部 ABI | **Wait** |
| UE `FSlateFontCache::Shape*` | `FString` 或字符串区间进入 shaping 时 | 源码 API 可见，但 monolithic、优化、LTCG、无符号会改变可定位性 | **理论较广，工程低**；复杂文本可能按 run 分段 | **理论可临时替换**，但需同步测量、换行、字体和缓存 | **C 级，不能宣称支持** |
| UE `FSlateDrawElement::MakeText` | 非 shaped 路径仍接收 `FString`/`FText` | 同上；很多路径随后直接使用 shaped sequence | **局部** | **局部** | 只可作原型候选 |
| UE Localization/LocRes/Polyglot | `FText` identity/history、LocRes、Polyglot source | 需要内部 Localization Manager 或解析 cooked package | **中**，适合作为语料数据源 | Polyglot 可运行时 patch，但要求精确 namespace/key/native string 且需内部集成 | **Go，合作/数据源；外部 Wait** |
| UE Canvas、自研 UI、材质、视频字幕 | 自定义 draw 或资源层 | 无统一 Slate seam | **低/无** | **无通用方案** | **No-Go** |
| 任何带反作弊/强签名/受保护平台目标 | 即使引擎内仍有字符串 | 注入、改码、第三方模块可能被阻止或判为篡改 | **不应尝试** | **不应尝试** | **No-Go** |

### 分层 Go / No-Go，而不是按引擎名判断

| 分层 | 现在可以证明什么 | 外部 Adapter 判定 |
| --- | --- | --- |
| Unity Mono + 标准 uGUI/TMP | Player 内有托管完整字符串对象；外部接入尚待真实 Shipping 证明 | **Prototype Go**；没有真实样本不得进入生产实现 |
| Unity Mono + UI Toolkit | 有 `TextElement.text`，但对象树和 uGUI 不同 | **Prototype Go after uGUI/TMP** |
| Unity IL2CPP + 标准 UI | 语义 API 经裁剪/AOT 后存在于原生 binary，正式版无 CLR attach 保证 | **Research Go / Product No-Go until gate** |
| Unity 自研 Mesh/Sprite/视频 | 没有共享标准组件，GPU 前可能已只剩自定义数据 | **Generic No-Go** |
| UE localized `FText` + UMG/Slate | 有最完整的 identity/history/display string；合作插件可用 | **Cooperative Go / External Wait** |
| UE culture-invariant/generated `FText` | 有完整显示字符串，但没有稳定 localization identity | **Observation candidate / dictionary identity degraded** |
| UE 非 `FText` 的 `FString` + Slate/Canvas | `MakeText`、`UCanvas::DrawText` 等仍接收完整字符串，但已无本地化历史 | **C-level prototype candidate only** |
| UE shaped glyph / atlas | 只剩 `FShapedGlyphSequence`、glyph 与纹理映射 | **No-Go** |
| UE 自研材质/纹理/视频 UI | 不经过标准 `FText`、Slate text 或 Canvas string seam | **Generic No-Go** |

Epic 的 `UCanvas::DrawText` 同时公开 `FStringView` 与 `FText` overload，而 Blueprint Canvas/HUD 也允许
直接传 string；因此“UE 文字”不能等同于 localized `FText`。
[`UCanvas::DrawText`](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/UCanvas/DrawText?lang=en-US)、
[Blueprint Canvas Draw Text](https://dev.epicgames.com/documentation/en-us/unreal-engine/BlueprintAPI/Canvas/DrawText)、
[`FCanvasTextItem`](https://dev.epicgames.com/documentation/unreal-engine/API/Runtime/Engine/FCanvasTextItem?lang=en-US)
Localized `FText` 是语义质量最高的 lane；非 `FText` 字符串只能形成当前显示文字观察，不能伪造
namespace/key/native string。两者必须分别计数和验收。

### 开始写生产 Adapter 前的真实样本 gate

合成 harness 只能证明调用模型，不足以授权生产实现。每个准备立项的 lane 在新增生产 crate、Catalog、
Runtime Bundle 或版本签名表之前，必须先找到**至少两个可重复的外部真实目标**，并满足：

1. 目标不是为了 Glyphshift 专门构建，也没有预装 Glyphshift 插件；它是公开可复现或已明确授权的
   Windows x64 发布构建。
2. 无反作弊、无竞技联网保护，允许本地观察；启动、固定页面和动态文本触发步骤可重复。
3. 两个目标来自不同项目，且不能只共享同一 demo/template。至少一个有初始静态文字和运行时更新，
   至少一个使用真实本地化或格式化数据。
4. 先通过只读文件/模块与版本信息确认 lane：Unity 必须区分 Mono/IL2CPP 与 UI 技术；UE 必须区分
   localized `FText`、culture-invariant/generated `FText`、`FString` Canvas/Slate 或自研路径。
5. 必须另有一个负例目标证明 unsupported 能可靠拒绝，而不是只捕获窗口标题、日志或少量系统 UI。
6. 样本路径、运行日志、截图、PID 与模块证据只保存在本地测试区；研究记录只保留可移植结论。

具体最低集合为：

| 准备实施的 lane | 实现前所需外部真实目标 | 否决条件 |
| --- | --- | --- |
| Unity Mono standard UI | 2 个，覆盖 uGUI/TMP，最好跨两个 Unity 代际 | 找不到两例，或只能靠项目预装插件 |
| Unity IL2CPP standard UI | 2 个，至少跨两个 Unity 代际；产品化前扩展到三个 LTS/代际 | 任一目标需要逐游戏符号/偏移 |
| Unity UI Toolkit | 2 个真实 runtime UI Toolkit Player | 只有 Editor/样例项目 |
| UE localized `FText` UMG/Slate | 2 个，至少包含一个 UMG 动态 binding | 无法在不带 PDB 的 Shipping-like 目标确认 seam |
| UE 非 `FText` FString Canvas/Slate | 2 个，至少一个动态 HUD/Canvas | 只能在 glyph/atlas 层命中 |
| 任一通用引擎声明 | 每个引擎至少 1 个自研 Mesh/Sprite/texture 负例 | unsupported 被误报为 Hook 成功 |

这道 gate 的含义是：**没有真实候选，先不写 Adapter；只有合成 harness，也不新增产品技术条目。**
即使某个 lane 通过，也只能发布该 lane 的兼容范围，不能把它外推为整个 Unity 或 UE 的覆盖率。

### 公开实现源码交叉验证（匿名）

既有公开实现证明 Unity 标准 UI 并非只能停留在理论层：Mono 路径可从运行时导出解析 domain、assembly、
image、class 与 method，IL2CPP 路径也可从公开运行时导出取得对应元数据；在进程内部定位标准组件后，
可以拦截 TMP `set_text` / `SetText`、uGUI `Text.set_text`、UI Toolkit `TextElement.set_text` 与
`TextField.set_value`，再替换传入的托管字符串。这给 Unity Standard UI 外部原型提供了比逐游戏偏移更
可信的方向。

但该实现同时暴露出 Glyphshift 产品合同缺失的两部分：只挂 setter 会漏掉附加前已经存在的文字；直接
替换 retained widget 状态也无法天然证明停止后能枚举并恢复全部仍存活对象。因此原型必须补上“初始对象
枚举 + 后续变更捕获 + 所有权记录 + 停止恢复”，不能把 setter 命中直接算作 `TextReplace` 已完成。

同一批公开源码中的 Godot 与 UE 路线仍主要依赖内部函数的字节模式、版本特征或目标专用过滤。这类做法
可以形成固定版本 Recipe 或研究候选，但不能作为跨游戏、跨版本的通用 Adapter 基线，也不会改变本文的
No-Go 条件。

### 公开真实样本候选筛选

公开资料筛选后已获得授权下载两份正式 Windows 发布包，完成只读二进制盘点和无注入启动；仍未对
候选执行外部 attach 或写回，因此**真实样本 gate 仍未通过**：

| 候选 | 公开证据 | 可承担的样本角色 | 当前判定 |
| --- | --- | --- | --- |
| [PacManUnity](https://github.com/Im-Rises/PacManUnity) | 正式 Windows 1.0 包为 x64、Unity 2021.3.9 Mono；业务程序集实际引用 TMP `set_text` / `SetText` | 跨启动与动态 TMP 文本正例候选 | **运行候选通过**；主菜单、关卡选择及动态 `Score` / `High Score` 均可见，日志无异常并可正常退出；二进制版本与 README 不同，矩阵必须以发布包为准 |
| [Unity Open Project #1](https://github.com/UnityTechnologies/open-project-1) | Unity 2020.3 LTS；工程同时声明 Localization、TMP 与 uGUI，并提供公开 Alpha 发布 | 本地化数据源与标准 UI 候选 | **降级**；发布说明明确对话未进入当前构建，不能承担“真实本地化动态文本”正例 |
| [Open Brush](https://github.com/icosa-foundation/open-brush) | 正式 Desktop 2.30.0 包为 x64、Unity 2022.3.62f2 Mono；业务程序集实际引用 TMP、uGUI 与 Unity Localization，并包含 no-headset/monoscopic 类型 | 跨 Unity 代际、本地化与多标准 UI 正例候选 | **运行通过但可见 Localization 正例降级**；公开 `--DisableXrMode --ForceViewOnly` 与 `--EnableMonoscopicMode` 可稳定进入平面模式；同 tag 的 Localization Settings 注册 `-language=` selector，界面 Locale 按钮也更新 `SelectedLocale`，但英语/简体中文三次无注入平面运行都没有形成可判读的本地化文字画面，不能计入可见语言切换 |
| [MarkovCraft](https://github.com/DevBobcorn/MarkovCraft/releases/tag/v1.5.3) | 正式 Windows zip 为 x64 Unity 6000.3.9f1 Mono；业务程序集引用 TMP/uGUI、Unity Localization、`NextLanguage` 与 `SelectedLocale`，同 tag 提供中文和英语 Locale | Unity 6、可见 Localization 与标准 UI 替代正例 | **运行候选通过**；默认中文欢迎页显示“下载资源 / 简”，将该测试应用自己的 `AppLanguage` 偏好临时设为英语后显示“Download Res / En”，偏好随后逐字节恢复；两次均加载 UnityPlayer + Mono、没有 GameAssembly/Glyphshift 并正常退出 |
| [Boss Room](https://github.com/Unity-Technologies/com.unity.multiplayer.samples.coop) | Unity 官方多人游戏样例，使用 uGUI，公开 release 主要服务项目获取与教学 | 工程资料对照 | **拒绝作为外部 Shipping 正例**；当前证据不足以证明 release asset 是无需编辑器的独立 Windows Player |

第二轮先得到跨代际的 IL2CPP Windows 候选，随后在单独授权下完成正式包静态盘点；没有启动候选：

| 候选 | 公开证据 | 可承担的样本角色 | 当前判定 |
| --- | --- | --- | --- |
| [Random Heroes](https://aetheris.itch.io/gmtk2022) | 官方页面提供 26 MB Windows 成品；[发布脚本](https://github.com/Aetheris743/GMTK-2022/blob/master/Assets/Editor/Itch%20Uploads/UploadGames.cs)明确把 Windows x64 切到 IL2CPP；工程为 Unity 2021.3.5、TMP 3.0.6，主菜单、按钮与 Canvas Prefab 实际引用 `TextMeshProUGUI` | 2021 LTS、轻量、动态游戏 UI 的 IL2CPP 正例候选 | **运行候选通过**；x64 Unity 2021.3.5f1、metadata 29、235 个 `il2cpp_*` API；连续两次启动均显示标准主菜单，随后可重复从 `SCORE: 0` 推进到 `SCORE: 248` 并显示 `Current Score: 248`；公开 `ScoreText` 每帧写入 `TextMeshProUGUI.text`，运行模块与发布包一致、没有 Glyphshift 模块且正常退出 |
| [HiBoP 5.0.9](https://github.com/hbp-HiBoP/HiBoP/releases/tag/5.0.9) | 官方发布提供 Windows x64 成品；同 tag 为 Unity 6.0 代际，[构建脚本](https://github.com/hbp-HiBoP/HiBoP/blob/5.0.9/Assets/Scripts/HBP/Dev/Editor/HBPBuilder.cs)对 Windows x64 明确选择 IL2CPP；工程实际包含 TMP 与 uGUI | Unity 6、复杂桌面交互 UI 的 IL2CPP 正例候选 | **拒绝作为 Shipping-like 正例**；x64 Unity 6000.4.10f1、metadata 39、241 个 `il2cpp_*` API，菜单与首启 UI 可见，但成品明确标记 `Development Build`；首启模态框还阻断窗口关闭，强制清理后停止复测 |
| [DeedPlanner 3.2.2](https://github.com/Warlander/DeedPlanner-3/releases/tag/v3.2.2) | 官方发布提供 Windows x64 成品；同 tag 为 Unity 6.0 代际，[构建脚本](https://github.com/Warlander/DeedPlanner-3/blob/v3.2.2/Assets/Warlander/Deedplanner/Editor/BuildSystem.cs)在 Windows Editor 构建 Windows x64 时选择 IL2CPP；工程实际包含 TMP | IL2CPP Unity 6 桌面标准 UI 替代正例 | **运行候选通过**；x64 Unity 6000.3.4f1、metadata 39、241 个 `il2cpp_*` API，TMP/uGUI 标记存在；两次启动均显示完整编辑器文字 UI，随后可重复把搜索框从占位文本改为 `wall`；公开 `TooltipHandler` 还会排序并换行拼接运行时片段，再由 `Tooltip.Value` 写入 `TMP_Text.text`；没有 Development Build/Glyphshift 模块并正常退出 |

IL2CPP 同后端负例的后续筛选严格限制在两个公开搜索方向。唯一进入仓库级核验的候选有正式 Windows
x64 release，构建脚本也明确选择 IL2CPP，但项目实际携带 TextMeshPro 与 `UnityEngine.UI`，属于
Standard UI 正例而非自绘负例；另一方向没有得到同时具备同源仓库、Windows 成品、IL2CPP 和自绘文字
证据的候选。本轮未下载或启动新成品，也没有扩大到来源不明的游戏二进制。

负例已由一份[公开 NGUI 牌类游戏 release](https://github.com/664235822/GwentCard/releases/tag/2.2)补齐。
正式 Windows 包是 x64 Unity 2022.2 Mono；业务程序集包含 `UILabel`、`UIFont`、`BMFont` 与 `NGUIText`，
但没有 TMP、TextMeshPro 或 uGUI 标记。release tag 的[业务代码](https://github.com/664235822/GwentCard/blob/2.2/Assets/Scripts/Play/GameOver.cs)
直接把可见中文与数值写入 `UILabel.text`；[UILabel](https://github.com/664235822/GwentCard/blob/2.2/Assets/NGUI/Scripts/UI/UILabel.cs)
在 `OnFill` 中调用 `NGUIText.Print(text, verts, uvs, cols)` 生成自有 Mesh，
[UIFont](https://github.com/664235822/GwentCard/blob/2.2/Assets/NGUI/Scripts/UI/UIFont.cs)则持有 BMFont、atlas、
material 与 sprite/texture。runtime-only smoke 显示发布包中文主菜单，加载 UnityPlayer + Mono、没有
Glyphshift 模块并正常退出。它因此能承担 Mono/通用 Standard UI 的进程级 unsupported 合同：Standard
UI lane 必须明确拒绝，不能把 Unity 引擎身份或窗口文字误报成 TMP/uGUI 成功。但它不能承担 IL2CPP
专属负例，因为 IL2CPP Adapter 可只凭后端不匹配就拒绝，尚未验证同后端内的 UI 技术识别。

两份发布包都包含 `MonoBleedingEdge` 与 x64 `mono-2.0-bdwgc.dll`，没有 `GameAssembly.dll`；两个 runtime
分别导出约 1200 个 `mono_*` 符号，domain、assembly、image、class、method、invoke、string 与 thread
所需入口在 2021.3/2022.3 两代均存在。包内没有 PDB/MDB 等符号文件，native debugger wait 关闭，文件名
级扫描也没有发现常见反作弊组件；最后一点只能说明“未发现”，不能证明绝对不存在保护。

授权后的 runtime-only smoke 没有加载 Glyphshift 或注入模块。第一份样本稳定显示初始 TMP 菜单、关卡
选择及运行时 `Score` / `High Score`；第二份默认 OpenXR 启动在没有 OpenXR runtime 的机器上失败，但用
项目公开的无头显参数后，Sketch Viewer 的标题、作品名、作者和下载状态，以及完整 Monoscopic 工作区均
稳定可见并可正常退出。第二份的 Locale 已能正常初始化，不过实际语言切换入口尚未完成可见验收。

原第二候选因平面模式没有可判读文字而降级后，替代候选补齐了 **Unity Mono standard UI** 样本矩阵：
一份 Unity 2021 LTS 动态 TMP 正例、一份 Unity 6 TMP/uGUI + Localization 可见正例，以及一份 Unity
2022.2 NGUI Mesh/atlas 负例。Mono lane 的外部真实样本 gate 已通过，可以进入 Shipping-like 合成合同
和 observe-only 原型，但尚未 attach，也不能据此建立生产 Adapter。IL2CPP runtime-only smoke
先否决了 Development Build，再由备用 release 补齐 Unity 6 正例；目前已有跨 Unity 2021 LTS / Unity 6
的两个 Shipping-like 运行候选，证明静态门禁必须由运行时构建形态复核。两个候选现已补齐可重复动态
文本与真实运行时组合字符串来源；另有一份 Mono NGUI Mesh/atlas 负例已固定，但它不能验证 IL2CPP
后端内的 UI 技术拒绝，有界公开筛选也未找到合格 IL2CPP 自绘成品，因此该样本前置 gate 仍缺同后端
负例。Shipping-like 合成合同、observe-only 原型、外部 attach、Dictionary 替换和停止恢复也未开始。
在这些门槛齐备前，不建立生产 crate、Catalog 或 Runtime Bundle 项。

## Unity

### Mono 与 IL2CPP 不是同一个 Adapter 后端

Unity 目前把 Mono 描述为运行时 JIT 后端，把 IL2CPP 描述为把 MSIL 转成 C++、再由平台原生编译器
生成本机二进制的 AOT 后端；Mono 只覆盖部分桌面/Android 平台，IL2CPP 覆盖面更广，且不支持
`Reflection.Emit` 等动态代码能力。[脚本后端介绍](https://docs.unity3d.com/6000.0/Documentation/Manual/scripting-backends-intro.html)、
[IL2CPP 概述](https://docs.unity3d.com/6000.0/Documentation/Manual/scripting-backends-il2cpp.html)、
[脚本限制](https://docs.unity3d.com/6000.0/Documentation/Manual/scripting-restrictions.html)

IL2CPP 的构建过程先编译 C# 程序集，再做托管裁剪，然后转 C++ 并生成原生可执行文件或 DLL。
[Unity 对 IL2CPP 流程的说明](https://docs.unity3d.com/cn/2021.3/Manual/IL2CPP.html) 这意味着普通
.NET Framework 的 `ICLRProfiling::AttachProfiler` 不能被当成 Unity 通用入口：Microsoft 明确把该
接口限定为 CLR/.NET Framework，并且 attach 还受位数、权限和目标 CLR 兼容性约束。
[ICLRProfiling](https://learn.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/iclrprofiling-interface)、
[AttachProfiler](https://learn.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/iclrprofiling-attachprofiler-method)

Windows IL2CPP 构建把游戏脚本与 IL2CPP 运行时编入原生输出；官方还把生成 C++、调试资料等备份内容
定义为不应随游戏分发的构建辅助信息，因此不能假设第三方正式版携带这些资料。
[Windows IL2CPP Player](https://docs.unity3d.com/2023.2/Documentation/Manual/WindowsPlayerIL2CPPScriptingBackend.html)

此外，UnityLinker 会删除静态分析认为不可达的代码；反射不能总被识别，项目开发者必须在构建时用
`[Preserve]` 或 `link.xml` 保留动态访问目标。已发布第三方游戏无法事后补上这些配置。
[托管代码裁剪](https://docs.unity3d.com/6000.0/Documentation/Manual/managed-code-stripping-configure.html)、
[裁剪对内容的影响](https://docs.unity3d.com/6000.0/Documentation/Manual/managed-code-stripping-content.html)

**推断：** 至少需要 `unity-mono` 与 `unity-il2cpp` 两个 runtime 实现。Mono 可以先证明对象发现和
变更捕获模型；IL2CPP 则决定这条路线能否覆盖现代正式游戏。把两者合并成一个 Catalog 技术会隐藏真正
的兼容风险。

### 标准 UI 中完整字符串在哪里

#### uGUI

`UnityEngine.UI.Text` 的公共 `text` 属性就是组件显示的字符串。
[uGUI `Text`](https://docs.unity3d.com/Packages/com.unity.ugui@1.0/api/UnityEngine.UI.Text.html#UnityEngine_UI_Text_text)
因此，在代码已经处于 Player 内部的前提下，初始枚举组件并观察属性变化可得到完整显示字符串；但只
Hook setter 会漏掉探针安装前已经存在的值。

#### TextMeshPro

`TMP_Text` 是 `TextMeshPro` 与 `TextMeshProUGUI` 的共同基类，公开 `text` 和多个 `SetText`
入口；它还维护解析缓冲、字符/单词/行信息，随后为字符生成 geometry。
[TMP_Text](https://docs.unity3d.com/Packages/com.unity.textmeshpro@3.0/api/TMPro.TMP_Text.html)、
[`TMP_Text.text`](https://docs.unity3d.com/Packages/com.unity.textmeshpro@3.0/api/TMPro.TMP_Text.text.html)、
[`TMP_Text.SetText`](https://docs.unity3d.com/Packages/com.unity.textmeshpro@3.0/api/TMPro.TMP_Text.SetText.html)
Unity 也明确说明 TMP 每个字符最终生成两个三角形；一旦只剩 mesh、UV 和 SDF atlas，就已越过可靠的
原文 seam。[TextMeshPro 手册](https://docs.unity3d.com/cn/2023.1/Manual/com.unity.textmeshpro.html)

#### UI Toolkit

UI Toolkit 支持游戏运行时 UI；`Label` 继承自 `TextElement`，`TextElement.text` 保存显示字符串，
运行时 UI 则位于自己的 Visual Tree，而不是 Windows 原生控件树。
[运行时 UI Toolkit](https://docs.unity3d.com/6000.0/Documentation/Manual/UIE-support-for-runtime-ui.html)、
[`TextElement.text`](https://docs.unity3d.com/6000.0/Documentation/ScriptReference/UIElements.TextElement-text.html)、
[`Label`](https://docs.unity3d.com/6000.0/Documentation/ScriptReference/UIElements.Label.html)

#### 覆盖缺口

Unity 官方同时列出 UI Toolkit、uGUI 与 IMGUI，旧游戏和定制项目还可能使用旧 TextMesh、自研 Canvas、
Sprite 字符、位图字库、视频或直接生成 mesh；这些路径并不共享一个标准 `text` 组件。
[Unity UI 系统比较](https://docs.unity3d.com/6000.0/Documentation/Manual/UIToolkits.html)

因此标准组件的正确观察模型应是“**初始对象枚举 + 后续变更捕获**”，至少覆盖 uGUI、TMP、UI Toolkit
三棵对象体系。Unity 提供的 `Object.FindObjectsByType` 证明项目内部代码可以查询已加载对象，但不证明
第三方 Shipping Player 允许外部 Adapter 进入该对象模型。
[`Object.FindObjectsByType`](https://docs.unity3d.com/6000.0/Documentation/ScriptReference/Object.FindObjectsByType.html)

### Localization、Addressables 与 AssetBundle

Unity Localization 的 `StringTable` 保存特定 Locale 的字符串，`LocalizedStringDatabase` 加载表、
取得条目，并在启用 Smart String 或格式化参数时先生成最终字符串。
[`StringTable`](https://docs.unity3d.com/Packages/com.unity.localization@1.4/api/UnityEngine.Localization.Tables.StringTable.html)、
[`LocalizedStringDatabase`](https://docs.unity3d.com/Packages/com.unity.localization@1.4/api/UnityEngine.Localization.Settings.LocalizedStringDatabase.html)
`LocalizeStringEvent` 还能在 locale、引用或格式化数据变化时发出最终字符串更新。
[`LocalizeStringEvent`](https://docs.unity3d.com/Packages/com.unity.localization@1.4/api/UnityEngine.Localization.Components.LocalizeStringEvent.html)

Localization 包使用 Addressables 管理 Locale、String Table 与 Asset Table；Addressables 又在运行时从
本地或远程 Catalog/AssetBundle 加载内容。
[Localization 与 Addressables](https://docs.unity3d.com/Packages/com.unity.localization@1.4/manual/Addressables.html)、
[`Addressables.RuntimePath`](https://docs.unity3d.com/Packages/com.unity.addressables@1.21/api/UnityEngine.AddressableAssets.Addressables.RuntimePath.html)、
[运行时加载 Catalog](https://docs.unity3d.com/Packages/com.unity.addressables@1.21/manual/LoadContentCatalogAsync.html)

AssetBundle 是平台相关的非代码资源归档，可以压缩；不能承载新的脚本程序集，版本兼容也并非双向保证。
[AssetBundle 概述与兼容规则](https://docs.unity3d.com/cn/current/Manual/AssetBundlesIntro.html)、
[平台限制](https://docs.unity3d.com/ja/current/Manual/assetbundles-platforms.html) Addressables 会在加载单个
asset 时于后台加载承载它的 AssetBundle。
[Addressables 加载 Bundle](https://docs.unity3d.com/ja/Packages/com.unity.addressables@1.20/manual/LoadingAssetBundles.html)

**推断：** 这些资源非常适合作为 `Unity catalog source`：可以在屏幕之外一次取得更完整的候选语料，
但不能代替当前屏幕观察。Localization 是可选包，Catalog 可以远程更新，Bundle 可以压缩、加密或由
项目自定义 Provider 供应；资源条目还缺少场景状态和格式化参数。它应是第二数据源，而不是被包装成
“Unity Hook 成功”。

### Shipping 中默认不可依赖的入口

- Unity 明确要求 Player 必须是 Development Build 才能被 Profiler 连接；Development Build 才启用
  Profiler，Deep Profiling 与自动连接也是该模式下的构建选项。
  [Player Profiling](https://docs.unity3d.com/2022.2/Documentation/Manual/profiler-profiling-applications.html)
- C# Script Debugging 同样要求构建前开启 Development Build 与 Script Debugging。
  [Player C# 调试](https://docs.unity3d.com/6000.0/Documentation/Manual/managed-code-debugging.html)
- 官方托管插件流程是把 DLL 加入 Unity Project；原生插件也通过 Project/Plugin Inspector 配置平台与
  CPU，再随 Player 构建。低层原生接口会在插件已由 Unity 加载后向 `UnityPluginLoad` 传入
  `IUnityInterfaces`，不是“后装任意已发布 Player”的服务。
  [托管插件](https://docs.unity3d.com/6000.0/Documentation/Manual/UsingDLL.html)、
  [原生插件](https://docs.unity3d.com/2018.3/Documentation/Manual/NativePlugins.html)、
  [低层原生插件接口](https://docs.unity3d.com/cn/2023.2/Manual/NativePluginInterface.html)
- Unity 的公开 C# 参考源码还明确提醒其目录与文件布局会随 Unity 版本变化。
  [UnityCsReference](https://github.com/Unity-Technologies/UnityCsReference)

因此，Profiler、Managed Debugger、反射、`IUnityInterfaces` 都不能自动升级为第三方 Shipping 的稳定
外部入口。它们可以支撑 A 级合作集成和自有测试 harness，但 B 级外部支持必须另行实证。

### Unity 的观察与替换边界

| 位置 | 观察 | 替换 | 恢复风险 |
| --- | --- | --- | --- |
| Localization 格式化前 | 有 key、locale、原始条目与参数，语义最好 | 内部事件/数据库层可替换 | 需要项目内接入；游戏可能不用该包 |
| uGUI/TMP/UI Toolkit 属性 | 有当前完整显示字符串 | 可写组件属性 | 绑定或每帧逻辑可能覆盖；改变 retained state |
| TMP/UI geometry 生成前 | 仍可看到字符串或字符序列 | 理论可生成译文 geometry | 要同时处理测量、换行、字体、rich text 与缓存 |
| mesh/glyph atlas/GPU | 已失去可靠语义和原始 markup | 只能替换几何/纹理 | 不通用，属于 D 级 |

首期若只需要在旁边显示原文/译文，应优先验证只读观察与 overlay；不应把写组件属性作为默认“临时
替换”，更不能修改 AssetBundle 或游戏文件来伪造可恢复运行时能力。

## Unreal Engine（UE4 / UE5）

### FText 与 Localization 是最上游的语义层

Epic 把 `FText` 定义为用户可见本地化文本的核心类型。它支持本地化 literal、格式化、数字/日期生成与
派生文本；内部用共享的 `ITextData` 与 text history 支持文化切换后的重建。
[`FText`](https://dev.epicgames.com/documentation/unreal-engine/ftext-in-unreal-engine?lang=en-US)、
[Text Localization](https://dev.epicgames.com/documentation/en-us/unreal-engine/text-localization-in-unreal-engine)

`FText::ToString` 可以得到当前显示字符串，但 Epic 明确说明 `FText` 与 `FString` 转换会丢失
localization data。因此，若 Adapter 只在较低层取得 `FString`，通常已经失去 namespace/key、native
string、history 等可用于稳定字典关联的信息。
[`FText` conversion](https://dev.epicgames.com/documentation/unreal-engine/ftext-in-unreal-engine?lang=en-US)

UE 默认通过 LocRes 与 Polyglot text source 向 Localization Manager 提供文本；Polyglot 数据可以在
运行时注册以 patch 已有翻译，但匹配覆盖需要 namespace、key 与 native string 一致。
[Text Localization 的 Polyglot/Localized Text Sources](https://dev.epicgames.com/documentation/unreal-engine/text-localization-in-unreal-engine?lang=en-US)

**推断：** 对合作项目，Localization Manager/Polyglot 是质量最高的替换入口；对任意 Shipping 游戏，
则仍缺少进入 Manager 的稳定外部 ABI，并且资源可能已被 cook 到容器中。离线 LocRes/包语料解析可作为
`UE catalog source`，但不是当前屏幕上下文。

### UMG / Slate 的完整字符串 seam

UMG 的 `UTextBlock::SetText(FText)` 是公开 Blueprint/C++ API，但 Epic 特别警告直接设置会清除 Text
property 上的 binding。[`UTextBlock::SetText`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/UMG/UTextBlock/SetText)
因此，拦截或反复写回 `UTextBlock` 不是天然可恢复：它可能改变 retained widget 状态和数据绑定。

UMG 下层是 Slate。`STextBlock` 持有 `TSlateAttribute<FText>`；Slate 还允许属性由 delegate 提供，
widget 会在需要显示时轮询 model。只 Hook 一次 setter 会漏掉动态绑定和初始化前文本。
[`STextBlock`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/Slate/STextBlock)、
[Slate 属性与轮询数据流](https://dev.epicgames.com/documentation/unreal-engine/understanding-the-slate-ui-architecture-in-unreal-engine?lang=en-US)

往下游走时，`FSlateFontCache::ShapeBidirectionalText` / `ShapeUnidirectionalText` 接收完整 `FString`，或
接收同一字符串中的 start/length 区间，再生成 `FShapedGlyphSequence`；Font Cache 随后把 shaped glyph
映射到字体 atlas。
[`ShapeBidirectionalText`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/SlateCore/Fonts/FSlateFontCache/ShapeBidirectionalText/1?application_version=5.5)、
[`FSlateFontCache`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/SlateCore/FSlateFontCache)

`FSlateDrawElement` 同时证明了边界：`MakeText` overload 仍接收 `FString` 或 `FText`，而
`MakeShapedText` 只接收 `FShapedGlyphSequenceRef`。越过 shaping 后，通用 Adapter 已不能可靠还原完整
原文、格式化 identity 或 rich text run。
[`FSlateDrawElement`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/SlateCore/FSlateDrawElement)

**推断：** `FSlateFontCache::Shape*` 是 UE 最值得研究的 draw-time seam，因为它比单个 `UTextBlock`
覆盖更接近渲染且仍有字符串；但复杂 text layout 可能把内容拆成多个 range/run，缓存还可能避免每帧
重新 shaping。译文替换必须同时进入测量、换行、BiDi、字体 fallback 与缓存失效流程，不能只改一次
函数参数。

### Slate 不是所有 UE 游戏文字

Epic 把 UMG 视为常见游戏 UI，并把 Slate 定义为自有跨平台 UI 框架；但官方 UI/HUD 文档也保留 Canvas
绘制路径，项目还可以使用材质、纹理、Custom Verts 或专用 UI 系统。
[Slate/UMG](https://dev.epicgames.com/documentation/en-us/unreal-engine/slate-ui-framework?application_version=4.27)、
[User Interfaces and HUDs](https://dev.epicgames.com/documentation/en-us/unreal-engine/user-interfaces-and-huds-in-unreal-engine)、
[`FSlateDrawElement::MakeCustomVerts`](https://dev.epicgames.com/documentation/en-us/unreal-engine/API/Runtime/SlateCore/FSlateDrawElement)

Slate 的 Invalidation 和 Retainer Panel 还会缓存 widget 信息或把子树扁平化为纹理，未变化时不再重绘。
[UMG 优化与缓存](https://dev.epicgames.com/documentation/unreal-engine/optimization-guidelines-for-umg-in-unreal-engine)
这不会让字符串从对象模型消失，却会让“只监听绘制调用”漏掉静态 UI；UE Adapter 同样需要初始对象/树
发现与变更或 invalidation 捕获，不能只靠每帧 draw Hook。

### Shipping、链接与符号为什么阻断通用外部 ABI

UE Shipping 配置启用优化，并默认移除面向开发的 console commands、stats 与 profiling tools；符号只有
项目主动包含调试文件或以相应方式构建时才可供调试，不能假设最终用户能获得。
[Build Configurations](https://dev.epicgames.com/documentation/en-us/unreal-engine/build-configurations-reference-for-unreal-engine)、
[Packaging](https://dev.epicgames.com/documentation/en-us/unreal-engine/packaging-your-project)

UBT 可以构建 modular 或 monolithic 目标。Epic 明确说明模块 API 宏只在 DLL/modular 模式产生
`dllexport`/`dllimport`，monolithic 模式会把全部代码放入单一 executable，API 宏展开为空。
[UnrealBuildTool](https://dev.epicgames.com/documentation/unreal-engine/unreal-build-tool-in-unreal-engine?lang=en-US)、
[Module API Specifiers](https://dev.epicgames.com/documentation/unreal-engine/module-api-specifiers-in-unreal-engine?lang=en-US)
TargetRules 还允许 LTCG、改变优化级别、控制 plugin support、Shipping logging/console/profiling 等构建
特性。[UBT Target Reference](https://dev.epicgames.com/documentation/unreal-engine/unreal-engine-build-tool-target-reference?lang=en-US)

即使是官方插件，流程也是放入 Engine/Project 的 plugin 搜索路径、由项目启用并在构建时编译；代码插件
与项目版本要匹配。UE 的 BuildID 系统会拒绝不是与 executable 同一次构建产生的模块，以避免过期 DLL
造成崩溃或难以追踪的错误。
[Plugins](https://dev.epicgames.com/documentation/unreal-engine/plugins-in-unreal-engine?lang=en-US)、
[Binary BuildID](https://dev.epicgames.com/documentation/en-us/unreal-engine/how-to-version-binaries-in-unreal-engine)

Epic 也明确承认 Unreal API 会随版本变化，旧项目可能因为废弃 API 被移除而无法直接加载；UE 还允许
licensee fork 与自定义 branch/version。
[UE5 Migration Guide](https://dev.epicgames.com/documentation/unreal-engine/unreal-engine-5-migration-guide?lang=en-US)、
[Assets/Packages Versioning](https://dev.epicgames.com/documentation/unreal-engine/versioning-of-assets-and-packages-in-unreal-engine)

**推断：** 公共 C++ header 是项目编译接口，不是对任意第三方二进制稳定的 ABI。若外部 Adapter 需要
每个游戏扫描机器码、恢复类布局、定位 `GUObjectArray`/`FName`/Slate 函数或携带逐 build 偏移，它就
是游戏专用 recipe，不是通用 UE Adapter。

### Reflection、Insights 与 Widget Reflector 不能补上 Shipping 接入口

UE Reflection 会为用 `UCLASS`、`UPROPERTY`、`UFUNCTION` 标记的对象提供引擎内元数据，以支持
Blueprint、序列化、GC 等功能。[Reflection System](https://dev.epicgames.com/documentation/unreal-engine/reflection-system-in-unreal-engine?lang=en-US)、
[UObject Handling](https://dev.epicgames.com/documentation/unreal-engine/unreal-object-handling-in-unreal-engine)
它证明项目内插件可以发现标准 `UTextBlock`，但没有提供从外部进程 attach、取得引擎 globals、调用任意
反射函数的稳定协议；纯 Slate `SWidget` 也不属于 UObject reflection。

Widget Reflector 是 Editor 内置的开发调试工具；快照流程面向 PIE/Standalone。Slate Insights 也要求在
项目/Editor 中启用插件、以 `-trace=slate` 运行，再分析 trace。其记录重点是 widget paint、update、
invalidation 及其原因，不是一个公开的实时文本替换协议。
[Widget Reflector](https://dev.epicgames.com/documentation/unreal-engine/using-the-slate-widget-reflector-in-unreal-engine?lang=en-US)、
[Slate Insights](https://dev.epicgames.com/documentation/unreal-engine/slate-insights-in-unreal-engine?lang=en-US)

UE Trace 本身是结构化事件系统，channel 未启用时不会发出事件；默认通道和 Slate channel 也没有承诺
输出每个 widget 的完整 `FText` 与 localization identity。
[Trace](https://dev.epicgames.com/documentation/en-us/unreal-engine/trace-in-unreal-engine-5)、
[Insights Reference](https://dev.epicgames.com/documentation/en-us/unreal-engine/unreal-insights-reference-in-unreal-engine-5)

因此，Reflection/Insights 是 A 级合作集成与开发验证工具，不是 B 级任意 Shipping 入口。

### Cooked 资源也只能作为第二数据源

UE packaging 会编译 C++ 与插件，把 asset 转为平台运行格式、裁掉未引用内容，并打入 Pak 或 IoStore
容器；Shipping 还会启用完整优化。
[Packaging pipeline](https://dev.epicgames.com/documentation/en-us/unreal-engine/packaging-your-project)
项目可以加密 Pak index、UAsset 或所有 asset，并启用 Pak signing 防止篡改。
[Encryption and Signing](https://dev.epicgames.com/documentation/en-us/unreal-engine/project-section-of-the-unreal-engine-project-settings)

所以离线 LocRes/StringTable/asset 提取可以在开放包中形成高价值语料，但必须接受容器、版本、加密、
远程内容和自研格式缺口；任何直接改包都可能破坏签名和更新，不属于临时运行时替换。

## 安全、签名、平台封装与反作弊

Windows 能通过 Dynamic Code Policy 禁止进程生成动态代码或修改现有可执行代码；Mandatory Integrity
Control 也会阻止低完整性进程写入更高完整性对象或读取 AppContainer 进程。
[Dynamic Code Policy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_dynamic_code_policy)、
[Mandatory Integrity Control](https://learn.microsoft.com/en-us/windows/win32/secauthz/mandatory-integrity-control)
这些限制与 CPU 架构、权限边界一起意味着探针必须先做只读能力检查，不能把注入失败归咎于用户配置。

Epic 对 Easy Anti-Cheat 的公开说明强调其会阻止常见作弊技术；官方玩家支持页明确把某些后台第三方
软件视为“Game Security Violation”，而 Rocket League 的官方支持说明更直接写明 EAC 开启时 mods
不会运行。
[EAC 服务说明](https://onlineservices.epicgames.com/news/epic-online-services-launches-two-new-free-services?lang=en-US)、
[Game Security Violation](https://www.epicgames.com/help/c-202300000001639/c-202300000001736/a202300000015740?lang=en-US)、
[EAC 与 mods](https://www.epicgames.com/help/c-202300000001622/c-202300000001682/easy-anti-cheat-is-now-available-in-rocket-league-a202300000084633)

这不证明每一种反作弊都会拦截 Glyphshift，但足以确立产品安全边界：**带反作弊或竞技联网的目标默认
No-Go，不启动进程内注入，不尝试绕过保护。** 只有发行方提供受支持插件接口或明确授权的离线/无保护
模式，才重新评估。

首期还应直接排除：

- macOS Hardened Runtime、iOS、主机及商店沙箱目标；这些平台要求签名、entitlement 或平台 SDK，
  不存在把任意桌面 DLL 后装进第三方游戏的普遍权限。Unity 官方也把 macOS hardened runtime 描述为
  防止代码注入、动态库劫持和内存篡改的安全机制。
  [Unity macOS 构建与签名](https://docs.unity3d.com/6000.0/Documentation/Manual/macos-building.html)
- 未构建对应探针的 x86、ARM 或其他 CPU；Unity 插件本身就必须按 OS/CPU 配置，UE 也按 target platform
  进行构建与 cooking。
  [Unity Plugin Inspector](https://docs.unity3d.com/2021.1/Documentation/Manual/PluginInspector.html)、
  [UE Packaging platforms](https://dev.epicgames.com/documentation/en-us/unreal-engine/packaging-your-project)
- 无法确认用户授权的进程、受保护进程、启用强动态代码/签名策略的目标。

## 推荐原型与 Go / No-Go

### 先做 Unity，不先做 UE

Unity 标准 UI 的托管对象模型比 UE 单体原生 C++ binary 更容易形成可验证 seam；Mono 还能先验证统一的
对象发现、初始扫描、动态事件和生命周期模型。UE 虽有更集中、接近渲染的 shaping seam，但 Shipping
链接/符号/版本问题更可能使每个游戏变成独立逆向工程。因此建议按以下顺序：

1. **Unity Mono，Windows x64，无反作弊，Shipping-like，自有确定性 harness。** 不预装 Glyphshift
   插件、不打开 Development Build/Script Debugging、不分发符号。覆盖 uGUI Text、TMP UGUI、世界空间
   TMP、UI Toolkit Label；同时验证初始值、setter、TMP 格式化、动态拼接与场景切换。
2. **Unity IL2CPP，同一功能矩阵。** 至少覆盖 2019.4、2021.3、2022.3 和 Unity 6 的代表版本；版本数
   可先从两个极端版本做止损，但产品化前必须跨至少三个 LTS/代际。禁止使用生成 C++ 或 PDB 当运行依赖。
3. **Unity Localization/Addressables 作为独立数据源原型。** 验证表、格式化参数、本地与远程 Catalog，
   并明确无法映射当前屏幕时的产品表现。
4. 只有 Unity 证明“跨版本、免符号、无需逐游戏偏移”后，才做 **UE5 单版本、Windows x64、无反作弊、
   标准 UMG/Slate** 原型。先验证 `FText`/widget 初始枚举与 `FSlateFontCache::Shape*` 观察，再谈替换；
   UE4 和 licensee fork 不在第一轮承诺内。

原型必须验证：

- 自动识别引擎、版本、架构以及 Unity Mono/IL2CPP；未知目标可靠拒绝。
- 运行后附加能取得已经存在的文本，不仅是安装后的 setter。
- 动态文本、场景/关卡切换、缓存/失效、两次启动、探针停止和目标退出均无崩溃。
- 观察结果能标记 UI 系统、对象/场景 identity、可见性与来源层；不支持路径给出明确诊断。
- 默认只读；翻译 overlay 与 in-place replacement 分开验收。
- 不依赖 Development Build、Profiler、Debugger、项目预装插件、最终用户符号或逐游戏函数地址。
- 目标没有反作弊，且测试只在授权的合成 harness 或明确授权目标上进行。

### Go 门槛

只有全部满足时，才把 Unity Standard UI 从研究项升级为产品 Adapter：

1. Mono 与 IL2CPP 都能覆盖标准 uGUI/TMP；UI Toolkit 至少能可靠识别或明确降级。
2. 同一后端实现跨至少三个代表性 Unity LTS/代际，不按游戏保存机器码签名、函数偏移或补丁。
3. 初始与动态文本均能捕获，复杂 TMP 路径不会只得到碎片或 glyph。
4. 停止探针能完全撤销，不改游戏文件、不破坏签名、不留下 retained widget 状态。
5. 负例（自研 mesh/sprite、加密资源、未知版本）可靠报告“不支持”，不能把窗口标题或少量系统 UI
   当成游戏内容 Hook 成功。
6. 至少在一组未用于开发 Adapter 的授权 Shipping-like 样本上通过盲测，才讨论覆盖率。

UE 还需额外满足：

- modular 与 monolithic Shipping 至少有明确支持/拒绝矩阵；不能依赖 PDB。
- UE5 的两个代表版本或 build 能复用同一定位与调用机制。
- Slate Attribute、rich text run、Invalidation/Retainer、字体 shaping 与 fallback 均有正确行为。
- 若只能通过逐游戏恢复类布局或函数地址接入，归类为 recipe，停止通用 Adapter 产品化。

### 立即 No-Go 的模式

- 只能在 Editor、Development Build、Profiler、Debugger 或预装插件的自有 demo 中工作。
- Unity IL2CPP 必须逐游戏恢复符号，或每个小版本维护独立偏移/机器码签名。
- UE 必须逐游戏定位 `FText`/Slate/UObject globals，或只对带 PDB 的开发样本有效。
- 只能在 glyph/mesh/atlas/framebuffer 层取得信息，却把推断结果称为完整原文 Hook。
- 用修改组件 retained state、替换 AssetBundle/Pak/IoStore 或关闭安全保护冒充“临时可恢复替换”。
- 带 EAC/其他反作弊、竞技联网、受保护进程、强签名/沙箱或未明确授权的目标。

如果通用路线 No-Go，仍可保留三个边界清晰且有价值的能力：

1. 合作开发者集成的 Unity SDK / UE Plugin。
2. Unity Localization/Addressables 与 UE LocRes/cooked resource 的离线语料数据源。
3. 针对明确授权游戏与固定版本的社区 Adapter/Recipe；产品 UI 必须显示精确兼容范围，不能继承
   “支持 Unity/UE”的泛化标签。

## 最终判断

Unity/UE 都值得研究，但它们不是一个新增 Catalog manifest 就能获得大覆盖的 Adapter。**Unity 值得做
一次严格止损的外部原型，UE 当前应 Wait；两者都不应提前宣传“大部分游戏”或“90%”。**

最重要的判断点不是“引擎源码里有没有完整字符串”，而是：

> 对没有预装插件、没有符号、开启发布优化的第三方 Shipping 游戏，能否在不逐游戏逆向、不破坏安全
> 边界的前提下，跨版本稳定进入该字符串层并完整退出。

证明这一点之前，Glyphshift 应把 Unity/UE 标成“研究中 / 兼容矩阵制”，并继续保留资源数据源与最终
OCR/图像兜底，而不是把理论 Hook 当作任意游戏翻译能力。
