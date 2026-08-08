# Unity IL2CPP 自绘文字负例候选复核

日期：2026-08-08

## 结论

**No-Go：本轮没有找到可冻结为 Unity IL2CPP Standard UI lane 进程级负例的公开候选。**

本轮严格限制为 3 条检索方向、5 个官方仓库；达到可解释的止损点后停止，没有下载、运行或注入任何
候选。唯一同时具备公开 Windows x64 IL2CPP 发布闭环的候选，其被测文字仍由 TMP/uGUI 绘制；其余
自研 Sprite/font-atlas/stroke 路径候选均缺少 Windows x64 IL2CPP 发布闭环。不得把“某个自绘 marker
未被 Standard UI Adapter 捕获”冒充“目标进程可以可靠报告不支持”：只要同一发布路径仍会实例化
受支持的 TMP/uGUI live object，它就不是当前 gate 所需的进程级负例。

## 筛选口径与边界

候选必须同时满足：

1. 项目官方仓库能对应到公开、可重复取得的 Windows x64 发布；
2. 标签源码、官方构建脚本或官方发布流程能把该 Windows 成品闭环到 IL2CPP，而不是仅证明项目曾经
   支持 IL2CPP；
3. 无反作弊、竞技保护或绕过安全机制的风险；
4. 被测可见文字由项目自研 Mesh、Sprite、texture、bitmap-font 或等价的自绘路径生成，不依赖
   TextMesh Pro 或 uGUI `Text`；
5. 有固定页面或确定性动态触发；
6. 对当前 Standard UI lane 而言是**进程级可靠负例**：测试发布路径不得同时激活可被 Adapter 正常
   识别的 TMP/uGUI live object。

本轮只读取项目官方仓库、标签源码、官方 release 和项目官方文档。没有读取第三方拆包、日志、博客或
聚合站，也没有以默认分支的新配置反推旧 release 二进制。

## 有界检索范围

| 方向 | 已检查候选 | 结果 |
| --- | --- | --- |
| 从 Windows IL2CPP 构建与 release 反查文字栈 | `0x7c13/Pal3.Unity`、`icosa-foundation/open-brush` | 前者为 TMP 正例；后者 Windows release 构建脚本实际切回 Mono，且无头显路径会激活 TMP/uGUI |
| 从 Sprite/font-atlas 自绘实现反查 Windows IL2CPP 发布 | `VoxelBoy/MobileUO`、`RhenaudTheLukark/CreateYourFrisk` | 自绘路径成立，但 Windows IL2CPP 条件不成立 |
| 从文字型 Unity 移植项目反查同标签后端与渲染路径 | `xerysherry/uEmuera` | Standalone release 的 IL2CPP 与纯自绘文字均无法由同标签一手资料闭环 |

## 候选复核

### 1. `0x7c13/Pal3.Unity` v0.78 — 拒绝（Standard UI 正例）

- 官方 [v0.78 release](https://github.com/0x7c13/Pal3.Unity/releases/tag/v0.78) 同时发布
  `Windows_x64` 成品。
- 同一标签的
  [ReleaseBuildPipeline.cs](https://github.com/0x7c13/Pal3.Unity/blob/c1f3fcb009be5ddf317572c20a1be3e64cb71aed/Assets/Scripts/Editor/ReleaseBuildPipeline.cs)
  把 Windows x64 映射到 `StandaloneWindows64`，选择 x64 architecture，并显式调用
  `SetScriptingBackend(..., ScriptingImplementation.IL2CPP)`。
- 但同一标签的
  [InformationManager.cs](https://github.com/0x7c13/Pal3.Unity/blob/c1f3fcb009be5ddf317572c20a1be3e64cb71aed/Assets/Scripts/Pal3.Game/UI/InformationManager.cs)
  直接持有 `TextMeshProUGUI`，并用 `SetText`/`.text` 更新运行时提示和调试文字。

因此它能作为 Windows x64 IL2CPP + TMP 的正向交叉样本，不能作为自研 Mesh/Sprite/texture 负例。

### 2. `icosa-foundation/open-brush` 2.30.0 — 拒绝（Windows 为 Mono，且是混合文字栈）

这个候选最容易造成误判，因此保留完整否决链：

- 官方 [2.30.0 release](https://github.com/icosa-foundation/open-brush/releases/tag/2.30.0) 发布
  `OpenBrush_Desktop_2.30.0.zip`；标签的
  [build.yml](https://github.com/icosa-foundation/open-brush/blob/776b3772ed72f29cae3b0e832e071a76bc61bf1c/.github/workflows/build.yml)
  将 `Windows OpenXR` 指向 `StandaloneWindows64`，再把该 artifact 打包成 Desktop release。
- 项目确有合格的自绘触发。官方 API 文档公开 `draw.text`，并说明无 VR 头显时可运行 monoscopic
  版本：[API Commands List](https://docs.openbrush.app/user-guide/open-brush-api/api-commands)。同一标签的
  [ApiMethods.DrawStrokes.cs](https://github.com/icosa-foundation/open-brush/blob/776b3772ed72f29cae3b0e832e071a76bc61bf1c/Assets/Scripts/API/ApiMethods.DrawStrokes.cs)
  把 `draw.text` 交给 `TextToStrokes`，再调用 `DrawStrokes.DrawNestedTrList`；
  [TextToStrokes.cs](https://github.com/icosa-foundation/open-brush/blob/776b3772ed72f29cae3b0e832e071a76bc61bf1c/Assets/Scripts/TextToStrokes/TextToStrokes.cs)
  则把字体 outline 转成三维 `TrTransform` 路径。该 marker 的最终可见形态是 brush strokes，不是
  TMP/uGUI 文本。
- 但标签中的 `ProjectSettings.asset` 即使记录了 `Standalone: 1`，也不能证明 release 后端。官方
  Windows workflow **没有**传 `-btb-il2cpp`；同一标签的
  [BuildTiltBrush.cs](https://github.com/icosa-foundation/open-brush/blob/776b3772ed72f29cae3b0e832e071a76bc61bf1c/Assets/Editor/BuildTiltBrush.cs)
  让命令行选项默认 `Il2Cpp == false`，仅在收到 `-btb-il2cpp` 时改为 `true`，随后
  `TempSetScriptingBackend` 会把 `false` 明确映射到 `ScriptingImplementation.Mono2x`。官方 workflow
  只在 Android/iOS matrix 行传该参数，Windows Desktop 行没有传。因此 Windows release 不能作为
  IL2CPP 同后端样本。
- 即使未来另行构建 IL2CPP，它仍不是当前 gate 的进程级负例。同一标签的
  [InitNoHeadsetMode.cs](https://github.com/icosa-foundation/open-brush/blob/776b3772ed72f29cae3b0e832e071a76bc61bf1c/Assets/Scripts/InitNoHeadsetMode.cs)
  在 `Start()` 中激活 NoVR UI，并持有或动态创建 `TextMeshPro`、`TextMeshProUGUI`、`TMP_Dropdown`
  与 uGUI 控件。

它最多可作为将来的“混合栈局部覆盖缺口”样本：用唯一 marker 验证 stroke 文字不会被误报为已覆盖；
它不能证明 Standard UI Adapter 应对整个进程返回“不支持”。

### 3. `VoxelBoy/MobileUO` v0.28 — 拒绝（无 Windows IL2CPP 发布）

- 同一标签的
  [SpriteFont.cs](https://github.com/VoxelBoy/MobileUO/blob/v0.28/Assets/Scripts/ClassicUO/src/Renderer/SpriteFont.cs)
  自行维护 `Texture2D`、glyph rectangles、character map、cropping 和 kerning，文字路径符合
  texture/font-atlas 自绘方向。
- 但标签的
  [ProjectSettings.asset](https://github.com/VoxelBoy/MobileUO/blob/v0.28/ProjectSettings/ProjectSettings.asset)
  明确为 `Android: 1`、`Standalone: 0`；官方 [v0.28 release](https://github.com/VoxelBoy/MobileUO/releases/tag/v0.28)
  也只提供 ARM APK，没有 Windows x64 artifact。

文字路径成立，但平台和后端同时不满足。

### 4. `RhenaudTheLukark/CreateYourFrisk` v0.6.6.4 — 拒绝（Standalone 非 IL2CPP）

- 官方 [v0.6.6.4 release](https://github.com/RhenaudTheLukark/CreateYourFrisk/releases/tag/v0.6.6.4)
  提供 Windows 64-bit 成品。
- 同一标签的
  [SpriteFontRegistry.cs](https://github.com/RhenaudTheLukark/CreateYourFrisk/blob/f91267a17b0ad145ac8cfac9e6d69371dd1f8b3b/Assets/Scripts/Lua/StaticRegistries/SpriteFontRegistry.cs)
  从 PNG/XML atlas 构建 `Dictionary<char, Sprite>`，并为字母加载独立 Sprite，符合自绘方向。
- 但同一标签的
  [ProjectSettings.asset](https://github.com/RhenaudTheLukark/CreateYourFrisk/blob/f91267a17b0ad145ac8cfac9e6d69371dd1f8b3b/ProjectSettings/ProjectSettings.asset)
  明确记录 `Standalone: 0`，不满足 IL2CPP 同后端条件。

它是清晰的 Mono/Sprite-font 负例，不得改写成 IL2CPP 负例。

### 5. `xerysherry/uEmuera` 0.2.9c — 拒绝（同标签闭环不足）

- 官方 [0.2.9c release](https://github.com/xerysherry/uEmuera/releases/tag/0.2.9c) 提供一个通用
  `uEmueraStandalone_0_2_9c.zip`，但资产名和 release 说明没有给出 x64/IL2CPP 证明。
- 同一标签的
  [ProjectSettings.asset](https://github.com/xerysherry/uEmuera/blob/48b4f40a88a5ef2b98d79a2106339cc26578db13/ProjectSettings/ProjectSettings.asset)
  只为 Android 记录 IL2CPP 后端，没有 Standalone IL2CPP 映射。默认分支后来出现的 Standalone 配置
  不能追溯证明旧 release。
- 同一标签的
  [FirstWindow.cs](https://github.com/xerysherry/uEmuera/blob/48b4f40a88a5ef2b98d79a2106339cc26578db13/Assets/Scripts/FirstWindow.cs)
  明确使用 `UnityEngine.UI.Text` 显示标题、版本、名称和路径；而
  [Drawing.cs](https://github.com/xerysherry/uEmuera/blob/48b4f40a88a5ef2b98d79a2106339cc26578db13/Assets/Scripts/uEmuera/Drawing.cs)
  中的 `Graphics.DrawString` 仅记录日志，不能证明该发布中存在一个确定性、纯自绘的可见文字页面。

该候选既缺 Windows x64 IL2CPP 的同标签证据，也缺纯自绘测试面的闭环。

## 后续授权边界

本轮结论不会触发候选下载、静态包检查、启动或 attach。下一步只能二选一：

1. 用户或贡献者提供一个已知候选及其官方来源，再对该单一候选做一手证据复核；或
2. 用户明确授权新一轮仍有上限的公开筛选，并重新约定候选数/检索方向。

即使后续找到纸面合格候选，也应按权限分段：先单独授权下载与静态确认 Windows x64/IL2CPP，再单独
授权启动与 observe-only 验证；任何进程内 attach/injection 仍需另外授权。未跨过这些边界前，不把
候选写成已验证负例，也不开始 IL2CPP attach。
