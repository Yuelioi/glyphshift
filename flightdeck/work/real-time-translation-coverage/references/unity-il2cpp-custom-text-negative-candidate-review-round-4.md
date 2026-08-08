# Unity IL2CPP 自绘文字负例候选第四次有界复核

日期：2026-08-08

## 结论

**No-Go：第四次有界复核仍未找到可冻结为 Unity IL2CPP Standard UI lane 进程级负例的公开候选。**

本轮合并 3 个有界方向，共检查 **7 个互不重复候选**，结果为 **0 接受 / 7 拒绝**。
其中 RadratSoftworks/nofun 同时被方向 A 与方向 B 命中，只计一次。方向达到预先约定的候选上限，
或继续检索已没有新候选时即停止，没有为了凑数扩大范围。

本轮没有重复前三轮已经检查过的 13 个项目：

- 0x7c13/Pal3.Unity
- icosa-foundation/open-brush
- VoxelBoy/MobileUO
- RhenaudTheLukark/CreateYourFrisk
- xerysherry/uEmuera
- Interkarma/daggerfall-unity
- fairygui/FairyGUI-unity
- Noesis/Tutorials
- mofr/Diablerie
- KonsomeJona/UnityGB
- ste-art/Pianofall
- YunitStudios/ToyOffensiveYunit
- ozonexo3/FAForeverMapEditor

本轮最接近目标的是 RadratSoftworks/nofun：官方标签同时具有 Windows x64 IL2CPP 配置和真正独立于
TMP 的 guest bitmap-font atlas/mesh 绘制路径。但官方同标签来源不能证明发布包内有固定、可重复的
guest 文字触发，而且 system-font 分支会动态实例化 TMP_Text，显示面又通过 Canvas/RawImage 组合，
因此它只能证明局部自绘 seam 存在，不能成为进程级可靠负例。

## 门槛与研究边界

候选必须同时满足以下硬门槛；任一项不能由同标签或同一官方发布流程闭环，即直接拒绝：

1. 有公开、可重复取得的 Windows x64 Shipping-like release；
2. 标签源码、官方构建脚本或官方发布流程能证明该 Windows x64 成品使用 IL2CPP；
3. 被测可见文字由项目自研 Mesh、Sprite、texture、bitmap-font 或等价自绘路径生成，而不是 TMP 或
   uGUI Text；
4. 有固定页面、固定输入或确定性动态事件，可以重复触发同一段可见文字；
5. 同一测试发布路径不会实例化可被现有 Standard UI Adapter 识别的 TMP/uGUI live objects，因此能
   成为**进程级可靠负例**，而不只是混合栈中的局部漏报。

本轮只使用项目官方仓库、同标签或固定提交源码、官方 release、官方构建配置和项目 README。没有使用
第三方聚合、拆包或逆向资料，也没有用默认分支的新配置反推旧 release。本轮没有下载、运行或注入任何
候选，没有进行本地重建，也没有改变上述 gate。

## 有界范围

| 方向 | 本轮互不重复候选数 | 结果 | 停止条件与主要止损原因 |
| --- | ---: | ---: | --- |
| A. 从 Windows x64 IL2CPP 构建脚本反查自绘文字 | 2 | 0 / 2 | 没有更多新候选；一个没有文字触发面，另一个缺包内固定 guest 触发且混用 TMP |
| B. 从模拟器、整帧纹理和 bitmap-font 路径反查发布 | 2 | 0 / 2 | 另有 nofun 与 A 重复；新候选为 x86 或只有 unitypackage |
| C. 从官方 Windows release 与字体目录反查后端和 live objects | 3 | 0 / 3 | 达到 3 个上限；Windows 为 Mono、缺发布构建闭环或使用 uGUI Text |
| **总计** | **7** | **0 / 7** | **没有候选同时跨过全部五项门槛** |

## 方向 A：从 Windows x64 IL2CPP 构建脚本反查自绘文字

### A1. xue-fei/wallpaper-unity v1.01 — 拒绝（发布与 IL2CPP 成立，但没有文字触发面）

- 官方 [v1.01 release](https://github.com/xue-fei/wallpaper-unity/releases/tag/v1.01) 提供
  Windows 构建压缩包，Windows x64 Shipping-like 发布条件成立。
- 同一标签提交的
  [BuildTool.cs](https://github.com/xue-fei/wallpaper-unity/blob/7e67fccf80a422a0e86f6a8674d1c176070dc589/Assets/Editor/BuildTool.cs#L309-L313)
  先把 Standalone scripting backend 设置为 IL2CPP，再以 StandaloneWindows64 和
  BuildOptions.None 调用 BuildPipeline.BuildPlayer。
- 官方 [同标签源码树](https://github.com/xue-fei/wallpaper-unity/tree/7e67fccf80a422a0e86f6a8674d1c176070dc589)
  能证明这是 Windows 桌面壁纸应用，但没有提供自研 OnPopulateMesh、Mesh/Sprite 文字、
  Graphics.DrawTexture、bitmap-font 或其他可固定触发的可见文字路径。

该候选跨过发布与后端门槛，却没有可供测试的文字 marker，因门槛 3 和 4 直接拒绝。没有文字触发面
也就不能把它解释成“Standard UI 正确拒绝了自绘文字”的负例。

### A2. RadratSoftworks/nofun v0.0.3 — 拒绝（自绘 guest 字体成立，但没有包内固定触发且混用 TMP）

- 官方 [v0.0.3 release](https://github.com/RadratSoftworks/nofun/releases/tag/v0.0.3) 提供
  Windows 发布压缩包；同标签
  [README](https://github.com/RadratSoftworks/nofun/blob/ee18a3ab6c392dcf0ebffb698dd3633b62ddca78/README.md#L3-L9)
  将项目定义为运行在 Unity 下的 Mophun 模拟器，并推荐从 Releases 取得。
- 同一标签提交的
  [Windows CI](https://github.com/RadratSoftworks/nofun/blob/ee18a3ab6c392dcf0ebffb698dd3633b62ddca78/.github/workflows/main.yml#L10-L16)
  对 StandaloneWindows64 使用 windows-il2cpp Unity 镜像；同提交的
  [ProjectSettings.asset](https://github.com/RadratSoftworks/nofun/blob/ee18a3ab6c392dcf0ebffb698dd3633b62ddca78/ProjectSettings/ProjectSettings.asset#L690-L700)
  也记录 Standalone scripting backend 为 IL2CPP。
- 自绘 guest bitmap-font 路径确实成立：
  [Font.cs](https://github.com/RadratSoftworks/nofun/blob/ee18a3ab6c392dcf0ebffb698dd3633b62ddca78/Assets/Scripts/Module/VMGP/Text/Font.cs#L180-L270)
  从 guest 字体数据创建 atlas、计算逐字符 atlas 位置并调用 DrawText；
  [GraphicDriver.cs](https://github.com/RadratSoftworks/nofun/blob/ee18a3ab6c392dcf0ebffb698dd3633b62ddca78/Assets/Scripts/Driver.Unity/Graphics/GraphicDriver.cs#L748-L770)
  再把每个 glyph 作为 atlas quad 交给 texture/mesh batching 路径绘制。这一局部分支不依赖 TMP 或
  uGUI Text。
- 但 system-font 分支明确维护 TMP_Text 实例，并在不足时动态克隆：
  [GraphicDriver.cs](https://github.com/RadratSoftworks/nofun/blob/ee18a3ab6c392dcf0ebffb698dd3633b62ddca78/Assets/Scripts/Driver.Unity/Graphics/GraphicDriver.cs#L993-L1029)
  会设置 TMP 文本、ForceMeshUpdate 后把 TMP mesh 交给 CommandBuffer。
- 同一显示链还通过
  [Canvas 与 RawImage](https://github.com/RadratSoftworks/nofun/blob/ee18a3ab6c392dcf0ebffb698dd3633b62ddca78/Assets/Scripts/Driver.Unity/Graphics/GraphicDriver.cs#L433-L510)
  呈现模拟器 back buffer。官方同标签仓库与 release 没有给出一个可由发布包自身确定触发的 guest
  文字 fixture；guest bitmap-font 是否出现取决于另行加载的模拟内容。

该候选满足“存在 IL2CPP 自绘文字 seam”，但不满足“发布包内固定触发”与“同路径没有 Standard UI
live objects”。局部 bitmap-font marker 即使漏抓，也不能证明整个进程应可靠返回不支持，因此拒绝。
它在方向 B 的模拟器/整帧纹理反查中再次出现，本报告不重复计数或另建候选节。

## 方向 B：从模拟器、整帧纹理和 bitmap-font 路径反查发布

方向 B 共命中三个项目，其中 nofun 与方向 A 重复并已在 A2 综合记录；以下只列两个新增候选。

### B1. Sin365/AxibugEmuOnline v1.0.2.alpha — 拒绝（Windows 构建目标为 x86）

- 官方 [v1.0.2.alpha release](https://github.com/Sin365/AxibugEmuOnline/releases/tag/v1.0.2.alpha)
  提供发布记录。
- 同一标签的
  [ProjectSettings.asset](https://github.com/Sin365/AxibugEmuOnline/blob/v1.0.2.alpha/AxibugEmuOnline.Client/ProjectSettings/ProjectSettings.asset#L803-L806)
  记录 Standalone scripting backend 为 IL2CPP，但这只证明项目配置，不能替代成品架构证明。
- 同标签
  [AxiAutoBuild.cs](https://github.com/Sin365/AxibugEmuOnline/blob/v1.0.2.alpha/AxibugEmuOnline.Client/Assets/AxiProjectTools/Editors/AxiAutoBuild.cs)
  实际选择 BuildTarget.StandaloneWindows，即 32 位 x86，而不是门槛要求的
  StandaloneWindows64。
- NES 输出路径的
  [VideoProvider.cs](https://github.com/Sin365/AxibugEmuOnline/blob/v1.0.2.alpha/AxibugEmuOnline.Client/Assets/Script/AppMain/Emulator/NesEmulator/VideoProvider.cs)
  使用 Canvas/RawImage 呈现模拟视频帧；这只能说明 guest 画面可能已经栅格化，不能证明发布包内存在
  一个固定自绘文字 marker。

该候选的 Windows 架构已经在官方构建脚本中明确失败。即使模拟内容能够产生整帧文字，它仍不是
Windows x64 成品，也没有闭环到固定 guest 文本，因此不进入下载或运行阶段。

### B2. ls9512/UNES v0.0.1 — 拒绝（只有 Unity package，没有 Windows player）

- 官方 [v0.0.1 源码与 README](https://github.com/ls9512/UNES/tree/v0.0.1) 表明这是 Unity 中的
  NES 模拟器项目。
- 官方 [v0.0.1 release 资产](https://github.com/ls9512/UNES/releases/expanded_assets/v0.0.1)
  只有 unitypackage，没有 Windows x64 player。
- 同标签仓库没有可把 Windows 成品闭环到 IL2CPP 的 ProjectSettings 或官方 player 构建流程。

它既不是可重复取得的 Shipping-like Windows 进程，也没有 IL2CPP 发布证明；模拟器内部可能产生的
像素文字不能补足门槛 1 和 2，直接拒绝。

## 方向 C：从官方 Windows release 与字体目录反查后端和 live objects

### C1. Caeden117/ChroMapper 0.13.892 — 拒绝（官方构建始终使用 Mono）

- 官方 [0.13.892 release](https://github.com/Caeden117/ChroMapper/releases/tag/0.13.892)
  提供公开发布记录。
- 同标签 [BUILD.md](https://github.com/Caeden117/ChroMapper/blob/0.13.892/BUILD.md) 明确项目构建
  始终使用 Mono。
- 同标签
  [ProjectSettings.asset](https://raw.githubusercontent.com/Caeden117/ChroMapper/0.13.892/ProjectSettings/ProjectSettings.asset)
  也没有把 Standalone 配置为 IL2CPP。

官方来源已经直接否定门槛 2，因此不再把仓库里的其他绘制或字体代码误写成 IL2CPP 发布证据。

### C2. hankmorgan/UnderworldExporter V1.09 — 拒绝（缺 IL2CPP 构建闭环且使用 uGUI Text）

- 官方 [V1.09 release](https://github.com/hankmorgan/UnderworldExporter/releases/tag/V1.09)
  提供发布记录；同标签也有
  [字体目录](https://github.com/hankmorgan/UnderworldExporter/tree/V1.09/UnityScripts/Fonts)。
- 但官方 [同标签根目录](https://github.com/hankmorgan/UnderworldExporter/tree/V1.09) 没有
  ProjectSettings 或官方 Unity player 构建 CI，无法证明 release 中的 Windows 成品架构与 IL2CPP
  后端。
- 同标签
  [ConversationVM.cs](https://raw.githubusercontent.com/hankmorgan/UnderworldExporter/V1.09/UnityScripts/scripts/Conversations/ConversationVM.cs)
  直接使用 UnityEngine.UI.Text。字体文件的存在不能证明可见文字通过独立自绘路径生成，而该明确的
  uGUI live object 又违反进程级负例门槛。

该候选同时缺门槛 2 的发布闭环，并失败于门槛 3 和 5；不再推进。

### C3. shinyflvre/Mate-Engine Public-Release-X3.3.0 — 拒绝（Standalone 为 Mono且依赖 uGUI）

- 官方
  [Public-Release-X3.3.0 release](https://github.com/shinyflvre/Mate-Engine/releases/tag/Public-Release-X3.3.0)
  提供发布记录；同标签包含
  [字体目录](https://github.com/shinyflvre/Mate-Engine/tree/Public-Release-X3.3.0/Assets/MATE%20ENGINE%20-%20Fonts/Normal)。
- 同标签
  [ProjectSettings.asset](https://raw.githubusercontent.com/shinyflvre/Mate-Engine/Public-Release-X3.3.0/ProjectSettings/ProjectSettings.asset)
  记录 Standalone scripting backend 为 0，即 Mono，不是 IL2CPP。
- 同标签
  [Packages/manifest.json](https://raw.githubusercontent.com/shinyflvre/Mate-Engine/Public-Release-X3.3.0/Packages/manifest.json)
  包含 Unity uGUI 依赖。字体目录本身没有证明文字由自研 Mesh/Sprite/texture 路径生成，也不能排除
  Standard UI live objects。

该候选因 Windows Standalone 后端和混合 UI 栈直接拒绝。不能从“有字体资源”推导“该 release 是
IL2CPP 自绘文字负例”。

## 本轮可复用结论

1. 模拟器的 guest bitmap atlas 或整帧 framebuffer 可以形成真正绕过对象级文字 setter 的 seam，
   但必须由发布包自身提供固定 guest 内容和确定性文字触发。依赖用户另行加载 ROM 或内容的路径不能
   冻结为自动验证负例。
2. Standalone backend 为 IL2CPP 不代表 Windows artifact 是 x64。AxibugEmuOnline 的项目配置为
   IL2CPP，但构建脚本明确选择 32 位 StandaloneWindows，必须在架构门槛处止损。
3. “存在字体目录”不是自绘文字证据。必须继续闭环到实际 glyph 生成和提交路径；一旦确定性界面仍使用
   TMP 或 UnityEngine.UI.Text，就不能成为进程级可靠负例。
4. Release 只有 unitypackage，或标签缺少 ProjectSettings/构建流程时，不存在可冻结的
   Windows x64 IL2CPP player。不能用项目类型、默认分支或理论能力补写发布证据。
5. nofun 证明了同一进程可以同时存在 clean bitmap-font seam、TMP system-font 和 Canvas/RawImage
   显示面。此类混合栈适合以后验证能力分层，但局部漏抓不能冒充 Standard UI lane 的可靠拒绝。

## 后续边界

第四次有界筛选结束后，公开检索仍为 No-Go。三个方向均已在达到候选上限或没有新候选后停止，不继续
无界扩搜。后续只有在贡献者提供新的明确候选及官方来源，或重新授权一轮带方向和数量上限的筛选时才
恢复候选研究。

即使以后出现纸面合格候选，也仍需按既有 gate 分段验证：先确认 Windows x64/IL2CPP 与发布来源，再
验证固定自绘文字和同路径无 TMP/uGUI live objects。本文没有下载、运行、注入或本地重建任何候选，
也没有改变 gate；因此不能把上述拒绝项目写成已经运行验证过的负例，更不能据此开始 IL2CPP attach。
