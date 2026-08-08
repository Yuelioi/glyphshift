# Unity IL2CPP 自绘文字负例候选第三次有界复核

日期：2026-08-08

## 结论

**No-Go：第三次有界复核仍未找到可冻结为 Unity IL2CPP Standard UI lane 进程级负例的公开候选。**

本轮固定检查 3 个方向、8 个互不重复的候选，结果为 **0 接受 / 8 拒绝**。达到预先约定的候选上限后
即停止，没有继续扩搜，也没有下载、运行或注入任何候选。本轮没有重复上轮检查过的
`0x7c13/Pal3.Unity`、`icosa-foundation/open-brush`、`VoxelBoy/MobileUO`、
`RhenaudTheLukark/CreateYourFrisk` 和 `xerysherry/uEmuera`。

结构上最接近目标的是 `Interkarma/daggerfall-unity` 和 `KonsomeJona/UnityGB`：前者有可重复取得的
Windows x64 成品与自绘 font-atlas 路径，但 Windows Standalone 明确为 Mono；后者把整帧写入纹理，
却明确只使用 Mono，而且没有官方 release。`ste-art/Pianofall` 等从公开 Windows 发布反查得到的项目
则再次证明“仓库里出现 IL2CPP”不能推出“该 Windows 发布就是 IL2CPP”：它们要么明确把 Windows
构建为 Mono，要么没有证据证明发布流程传入了 IL2CPP 参数，并且可见文字仍来自 TMP/uGUI。

## 门槛与研究边界

候选必须同时满足以下硬门槛；任一项不能由同标签或同一官方发布流程闭环，即直接拒绝：

1. 有公开、可重复取得的 Windows x64 Shipping-like release；
2. 标签源码、官方构建脚本或官方发布流程能证明该 Windows x64 成品使用 IL2CPP；
3. 被测可见文字由项目自研 Mesh、Sprite、texture、bitmap-font 或等价自绘路径生成，而不是 TMP 或
   uGUI `Text`；
4. 有固定页面、固定输入或确定性动态事件，可以重复触发同一段可见文字；
5. 同一测试发布路径不会实例化可被现有 Standard UI Adapter 识别的 TMP/uGUI live objects，因此能
   成为**进程级可靠负例**，而不只是混合栈中的局部漏报。

本轮只使用项目官方仓库、标签或固定提交源码、官方 release 和项目 README。没有使用第三方拆包、
逆向结果或聚合资料；没有用默认分支的新配置反推旧 release 二进制。纸面候选也未进入下载、静态包
检查、启动或 attach 阶段。

## 有界范围

| 方向 | 候选数 | 结果 | 主要止损原因 |
| --- | ---: | ---: | --- |
| A. 从 NGUI、FairyGUI、NoesisGUI 与自研 bitmap-font 项目反查发布 | 3 | 0 / 3 | 实际成品为 Mono，或只有框架/示例源码而没有 Windows IL2CPP player |
| B. 从像素化、整帧纹理和复古自绘项目反查后端 | 2 | 0 / 2 | 无官方 release、明确只用 Mono，或可见文字实际为 uGUI `Text` |
| C. 从官方 Windows release 反查同标签构建脚本和文字栈 | 3 | 0 / 3 | Windows 构建明确为 Mono或缺 IL2CPP 参数闭环，同时存在 TMP/uGUI |
| **总计** | **8** | **0 / 8** | **没有候选同时跨过全部五项门槛** |

## 方向 A：非 TMP/uGUI UI 与 bitmap-font 反查

### A1. `Interkarma/daggerfall-unity` v1.1.1 — 拒绝（自绘成立，但 Windows 为 Mono）

- 官方 [v1.1.1 release 资产](https://github.com/Interkarma/daggerfall-unity/releases/expanded_assets/v1.1.1)
  包含 `dfu_windows_64bit-v1.1.1.zip`，Windows x64 Shipping-like 发布条件成立。
- 同一标签的
  [ProjectSettings.asset](https://raw.githubusercontent.com/Interkarma/daggerfall-unity/v1.1.1/ProjectSettings/ProjectSettings.asset)
  在 `scriptingBackend` 中记录 `Standalone: 0`，因此该发布源码的 Standalone 后端是 Mono，不是
  IL2CPP。
- 同一标签的
  [DaggerfallFont.cs 字形绘制路径](https://github.com/Interkarma/daggerfall-unity/blob/v1.1.1/Assets/Scripts/Game/UserInterface/DaggerfallFont.cs#L2094-L2114)
  使用 `Graphics.DrawTexture` 绘制 glyph；其
  [字体加载路径](https://github.com/Interkarma/daggerfall-unity/blob/v1.1.1/Assets/Scripts/Game/UserInterface/DaggerfallFont.cs#L2786-L2827)
  从 `.FNT` 数据创建 font atlas。自绘文字结构本身符合方向要求。

这是本方向最接近目标的项目，但后端门槛已经明确失败。它只能作为 Mono 自绘文字负例，不能被改写成
IL2CPP 同后端负例。

### A2. `fairygui/FairyGUI-unity` 5.2.0 — 拒绝（不是 Windows 成品，且启用混合栈）

- 官方 [仓库说明](https://github.com/fairygui/FairyGUI-unity) 将项目定义为 Unity 的 FairyGUI UI
  framework，而不是一个可独立验证的 Shipping-like 应用。
- [5.2.0 release 资产](https://github.com/fairygui/FairyGUI-unity/releases/expanded_assets/5.2.0)
  只有源码 zip/tar，没有 Windows x64 player。
- 同一标签的
  [ProjectSettings.asset](https://raw.githubusercontent.com/fairygui/FairyGUI-unity/5.2.0/ProjectSettings/ProjectSettings.asset)
  记录 `scriptingBackend: {}`，没有 Windows IL2CPP 映射；同时 Standalone define 包含
  `FAIRYGUI_TMPRO`。
- 同一标签的
  [Packages/manifest.json](https://raw.githubusercontent.com/fairygui/FairyGUI-unity/5.2.0/Packages/manifest.json)
  同时依赖 `com.unity.textmeshpro` 和 `com.unity.ugui`。

该候选缺少 Windows 成品与 IL2CPP 证明，并明确允许 TMP/uGUI 混合路径，不能作为进程级负例。

### A3. `Noesis/Tutorials` 3.0.0 — 拒绝（只有 Unity 示例源码，没有 Unity Windows IL2CPP 发布）

- 官方 [README](https://github.com/Noesis/Tutorials#noesisgui-samples-and-tutorials) 明确说明仓库包含
  C++、C#、Unity 与 Unreal 示例，Unity 示例还需要另行安装官方 Unity package。
- [3.0.0 release 资产](https://github.com/Noesis/Tutorials/releases/expanded_assets/3.0.0) 中唯一
  Windows 可执行文件名为 `DopesBench-native-Noesis.exe`，不是 Unity player；其余是 Android APK
  和源码包。
- 同一标签确有
  [HelloWorld Unity 示例目录](https://github.com/Noesis/Tutorials/tree/3.0.0/Samples/HelloWorld/Unity)，
  但没有对应的 Windows x64 Unity release，也没有把任何 Windows 成品闭环到 IL2CPP 的构建证据。

该项目只能证明 NoesisGUI 存在 Unity 集成示例，不能提供本 gate 所需的发布级进程负例。

## 方向 B：像素化与整帧纹理项目反查

### B1. `mofr/Diablerie` — 拒绝（无官方 release，且文字使用 uGUI）

- 官方 [README 的运行说明](https://github.com/mofr/Diablerie#how-to-run-the-game) 提供 Windows
  latest build 的取得方式，但 [GitHub Releases](https://github.com/mofr/Diablerie/releases) 为空，
  无法冻结一个带标签、可重复取得的 Shipping-like Windows x64 release。
- 项目的
  [ProjectSettings.asset](https://github.com/mofr/Diablerie/blob/master/ProjectSettings/ProjectSettings.asset#L530-L535)
  没有可将该 Windows build 闭环到 IL2CPP 的 Standalone backend 配置。
- [Label.cs](https://github.com/mofr/Diablerie/blob/master/Assets/Scripts/Diablerie/Engine/UI/Label.cs#L1-L52)
  直接使用 `UnityEngine.UI.Text`，因此主要可见标签属于现有 Standard UI 范围，不是纯自绘
  bitmap-font 负例。

发布、后端和文字栈三项均不满足，不再向运行验证推进。

### B2. `KonsomeJona/UnityGB` — 拒绝（自绘结构接近，但明确为 Mono且无 release）

- 官方 [README](https://github.com/KonsomeJona/UnityGB#unitygb) 明确说明项目只使用 Mono，后端与
  IL2CPP gate 直接冲突。
- README 的 [Video 路径说明](https://github.com/KonsomeJona/UnityGB#video) 显示模拟器把整帧画面通过
  `SetPixels` 更新到 texture；这种整帧纹理输出是本轮最接近“文字已经被栅格化进画面”的结构。
- 官方 [Releases](https://github.com/KonsomeJona/UnityGB/releases) 为空，没有可冻结的 Windows x64
  Shipping-like artifact。

即使画面中的文字天然绕过对象级 Standard UI 观察，它仍缺 Windows release，并且项目明确只用
Mono，不能充当 IL2CPP 同后端负例。

## 方向 C：从 Windows 发布反查构建后端与文字栈

### C1. `ste-art/Pianofall` v0.4.3 — 拒绝（Windows x64 明确为 Mono）

- 官方 [v0.4.3 release](https://github.com/ste-art/Pianofall/releases/tag/v0.4.3) 提供可公开取得的
  发布记录。
- 该标签对应提交的
  [EditorExtensions.cs](https://github.com/ste-art/Pianofall/blob/5ec9242306039992344c86033e0a99709bece878/Assets/Editor/EditorExtensions.cs)
  明确把 Windows x64 构建配置为 Mono，而把 Linux 构建配置为 IL2CPP。仓库存在 IL2CPP 路径不等于
  Windows 发布使用 IL2CPP。
- 同一提交的
  [MainMenu.prefab](https://github.com/ste-art/Pianofall/blob/5ec9242306039992344c86033e0a99709bece878/Assets/Gui/MainMenu.prefab)
  使用 uGUI；[Loader.cs](https://github.com/ste-art/Pianofall/blob/5ec9242306039992344c86033e0a99709bece878/Assets/Scripts/Loader.cs)
  更新 `UnityEngine.UI.Text`。

这是典型的发布反查误判：平台确有发布、仓库确有 IL2CPP，但 Windows x64 本身是 Mono，且可见文字
属于 uGUI。

### C2. `YunitStudios/ToyOffensiveYunit` v1_4_0 — 拒绝（Windows IL2CPP 发布闭环不足，且使用 TMP）

- 官方 [v1_4_0 release](https://github.com/YunitStudios/ToyOffensiveYunit/releases/tag/v1_4_0)
  提供发布记录。
- 同一版本提交的
  [BuildManager.cs](https://github.com/YunitStudios/ToyOffensiveYunit/blob/93187750a6234515060b2707281817785c6bbd1f/Assets/Editor/BuildManager.cs)
  默认使用 Mono，只有收到显式参数时才切换到 IL2CPP；公开 release 没有证明该参数曾用于 Windows
  artifact。
- 同一提交的
  [ProjectSettings.asset](https://github.com/YunitStudios/ToyOffensiveYunit/blob/93187750a6234515060b2707281817785c6bbd1f/ProjectSettings/ProjectSettings.asset)
  只记录 Android IL2CPP，不能证明 Standalone release 的后端。
- [HUDObjective.prefab](https://github.com/YunitStudios/ToyOffensiveYunit/blob/93187750a6234515060b2707281817785c6bbd1f/Assets/Prefabs/UI/HUD/HUDObjective.prefab)
  使用 TMP；[ObjectiveUI.cs](https://github.com/YunitStudios/ToyOffensiveYunit/blob/93187750a6234515060b2707281817785c6bbd1f/Assets/Scripts/UI/HUD/ObjectiveUI.cs)
  也直接操作 TMP 文字。

该候选既无法把 Windows release 闭环到 IL2CPP，又会在确定性 HUD 页面实例化受支持的 TMP live
objects，因此不是进程级负例。

### C3. `ozonexo3/FAForeverMapEditor` v0.703-alpha — 拒绝（Windows 构建为 Mono，且使用 uGUI）

- 官方 [v0.703-alpha release](https://github.com/ozonexo3/FAForeverMapEditor/releases/tag/v0.703-alpha)
  提供公开发布记录。
- 同版本提交的
  [BuildWithResources.cs](https://github.com/ozonexo3/FAForeverMapEditor/blob/481bc8d3d0bb17d6ed90827011bb11401f996ffb/Assets/Editor/BuildWithResources.cs)
  中 Windows IL2CPP 设置被注释，实际构建路径使用 Mono。
- [UiTitle.cs](https://github.com/ozonexo3/FAForeverMapEditor/blob/481bc8d3d0bb17d6ed90827011bb11401f996ffb/Assets/Scripts/UI/UiElements/UiTitle.cs)
  使用 uGUI `Text`；
  [UiTextField.cs](https://github.com/ozonexo3/FAForeverMapEditor/blob/481bc8d3d0bb17d6ed90827011bb11401f996ffb/Assets/Scripts/UI/UiElements/UiTextField.cs)
  使用 uGUI `InputField`。

该候选同时因 Windows Mono 与 Standard UI 文字栈被拒绝。

## 本轮可复用结论

1. “自绘结构成立”与“IL2CPP 同后端成立”必须保持为两个独立证据链。Daggerfall Unity、UnityGB
   说明高质量自绘负例往往因 Mono 或无 release 而止步。
2. “仓库支持 IL2CPP”不能替代“该 Windows release 使用 IL2CPP”。Pianofall、Toy Offensive 与
   FAForeverMapEditor 都会因平台分支、默认参数或被注释的设置造成误判。
3. 框架示例不是 Shipping-like 进程候选。FairyGUI 与 NoesisGUI 的官方示例适合验证 API 结构，
   但没有满足 gate 的 Windows x64 IL2CPP 发布闭环。
4. 即使局部存在 Sprite、texture 或自绘控件，只要同一发布路径还会实例化 TMP/uGUI live objects，
   它最多是混合栈覆盖缺口样本，不能证明整个进程应可靠返回“不支持”。

## 后续边界

第三次有界筛选结束后，公开检索仍处于 No-Go。继续推进只能由贡献者提供一个新的明确候选及其官方
来源，或重新明确授权一轮带候选数和方向上限的筛选。后续即使出现纸面合格候选，也必须分段授权：先
下载与静态确认 Windows x64/IL2CPP，再启动并做 observe-only 验证；任何 attach/injection 仍需另外
授权。在跨过这些门槛前，不把候选写成已验证负例，也不开始 IL2CPP attach。
