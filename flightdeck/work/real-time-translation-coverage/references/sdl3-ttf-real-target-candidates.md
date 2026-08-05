# SDL3_ttf 真实目标候选审查

更新时间：2026-08-06

## 结论

本轮一手资料审查**没有找到满足下列条件的外部真实软件**：

- 有可审计的官方开源源码；
- Windows 构建或发布路径可重复；
- 运行时实际使用 SDL3_ttf 3.x；
- 绘制阶段经过 `TTF_Text` 公共 API，尤其是当前适配器目标中的
  `TTF_DrawRendererText` 或 `TTF_DrawSurfaceText`；
- 能从一手资料闭环确认动态链接方式和可观测的文字重建时机。

因此，**生产目标判定为 No-Go，真实外部目标仍然缺失**。不能用 SDL_ttf 自己的示例推导出第三方软件生态已经验证，也不能据此启动面向生产软件的 SDL3_ttf 适配承诺。

SDL_ttf 官方 `showfont` 示例可以作为隔离的**技术 smoke**，用于验证 DLL 加载、公开导出命中、`TTF_Text` 读取、代际恢复和 fail-open 行为。它不是外部真实软件，不能替代生产目标门槛。

| 候选 | 身份 | 当前结论 | 适用范围 |
| --- | --- | --- | --- |
| `showfont`（`release-3.2.2`） | SDL_ttf 官方示例 | 技术 smoke：Go；生产目标：No-Go | 首个 Renderer Text API smoke |
| `showfps`（固定开发提交） | SDL_ttf 官方示例 | 首个 smoke：No-Go；后续诊断：可用 | 高频 Renderer hook 命中与生命周期诊断 |
| `testgputext`（`release-3.2.2`） | SDL_ttf 官方示例 | 当前适配器：No-Go | 未来 GPU draw-data seam |

## 审查边界

本页只记录官方仓库源码、官方 API 文档和官方构建/发布资料中的可验证事实。本轮没有下载、构建或运行候选，因此：

- “可重复构建”表示官方源码和工程定义存在，不代表已经在本机复现；
- “动态链接”表示构建定义默认或明确生成共享库，不代表待测二进制已经完成 PE 导入和运行模块验证；
- 所有候选在进入本地 smoke 前仍须检查实际二进制导入、运行时加载模块和 hook 命中；
- 静态链接、自带私有修改版 SDL3_ttf、未导出或绕过公共 API 的构建继续 fail-open，并不进入当前支持范围。

## 候选一：`showfont` 3.2.2

### 身份与源码调用链

这是 SDL_ttf 官方稳定标签 `release-3.2.2` 中的示例，不是外部真实软件。

稳定源码定位：

- [`examples/showfont.c`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/examples/showfont.c)
  - `scene.textEngine` 默认选择 Renderer text engine；
  - 初始化阶段调用 `TTF_CreateRendererTextEngine(scene.renderer)`；
  - caption 通过 `TTF_CreateText(engine, font, string, 0)` 创建；
  - `DrawScene` 每帧调用 `TTF_DrawRendererText(scene->caption, ...)`；
  - 主循环持续调用 `DrawScene`；
  - 命令行文本会插入 editbox，可为 smoke 提供固定、可精确匹配的 UTF-8 输入。
- [`examples/editbox.c`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/examples/editbox.c)
  - `EditBox_Create` 用 `TTF_CreateText` 创建长期存在的 `TTF_Text`；
  - `EditBox_Insert` 用 `TTF_InsertTextString` 修改文字；
  - IME/编辑操作通过 delete/insert API 修改同一个文字对象；
  - `EditBox_Draw` 每帧调用 `TTF_DrawRendererText`。

文字重建时机可以明确区分：

1. 初始化时由 `TTF_CreateText` 创建 caption/editbox 文字对象；
2. 用户输入、IME composition 或预置文本插入时，通过 `TTF_InsertTextString` 等修改 API 更新对象；
3. 没有编辑时，现有 `TTF_Text` 在每帧绘制阶段重复进入 `TTF_DrawRendererText`。

这使 `showfont` 适合检查“文字只在变更时重建，但绘制公开 API 每帧可命中”的场景。

有一个必须避免的误判：`showfont` 的中央 message 使用旧式 `TTF_RenderText_* -> SDL_Surface -> SDL_Texture` 路径。首个 smoke 必须观察 caption 或 editbox，而不能以中央纹理文字作为 `TTF_DrawRendererText` 是否生效的判据。

### 链接方式

官方 3.2.2 CMake 定义显示：

- `BUILD_SHARED_LIBS` 默认 `ON`；
- 共享构建选择 `SDL3_ttf-shared` 和共享 SDL3 target；
- `showfont` 链接所选择的 SDL3_ttf/SDL3 target；
- 可以显式关闭共享构建而得到静态库，但静态构建不满足当前 DLL 公开导出 hook 的前提。

一手资料：

- [`release-3.2.2/CMakeLists.txt`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/CMakeLists.txt)
- [`VisualC/SDL_ttf.vcxproj`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/VisualC/SDL_ttf.vcxproj)：Win32/x64、Debug/Release 配置的 `ConfigurationType` 为 `DynamicLibrary`。
- [`VisualC/showfont/showfont.vcxproj`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/VisualC/showfont/showfont.vcxproj)：`showfont` 对官方 SDL_ttf 工程有 `ProjectReference`。

因此，**只有实际动态加载官方兼容 `SDL3_ttf.dll` 的产物进入 smoke**；静态或私有实现直接 No-Go。

### Windows 构建复现性

`release-3.2.2` 同时提供 CMake 和 Visual Studio 路径：

- [`INSTALL.md`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/INSTALL.md)：通过 `SDLTTF_SAMPLES=ON` 构建示例；
- [`docs/INTRO-cmake.md`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/docs/INTRO-cmake.md)：给出 SDL 与 SDL_ttf 作为子项目的构建方法，并说明 Windows 可执行文件的配置目录；
- [`docs/INTRO-visualstudio.md`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/docs/INTRO-visualstudio.md)：给出子模块获取、工程引用和 Visual Studio 构建路径；
- [`.gitmodules`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/.gitmodules)：声明官方 vendored FreeType、HarfBuzz、PlutoSVG 和 PlutoVG 来源；
- [`release-3.2.2` 发布页](https://github.com/libsdl-org/SDL_ttf/releases/tag/release-3.2.2)：稳定发布并提供 Windows 架构产物。

该标签自身声明 SDL_ttf 3.2.2，并要求 SDL 3.2.6。MSVC 下 vendored 依赖默认开启。`showfont` 运行仍需要明确提供字体文件；具体字体路径必须作为本机测试输入保留在 `target/local-test/`，不得写入跟踪内容。

### 判定

**技术 smoke：Go。生产目标：No-Go。**

建议首测固定 Renderer engine，并把一条固定、完整的 UTF-8 字典键作为命令行文本插入 editbox。验收对象是 editbox/caption 的每帧 `TTF_DrawRendererText`，不是中央 surface/texture message。

进入实现前仍需通过实际产物确认：

1. PE 导入或运行模块中确实存在动态 `SDL3_ttf.dll`；
2. 公开 `TTF_DrawRendererText` 导出能够被命中；
3. 完整 UTF-8 精确字典可以产生可见替换；
4. generation 1、generation 2、热关闭和异常路径均能恢复或 fail-open。

## 候选二：`showfps`

### 身份与源码调用链

该示例来自 SDL_ttf 官方仓库的固定提交
`a42434b8c96daaf7650dbd0befe480c090d1c2eb`，不是稳定 3.2.2 标签，也不是外部真实软件。

- [`examples/showfps.c`](https://github.com/libsdl-org/SDL_ttf/blob/a42434b8c96daaf7650dbd0befe480c090d1c2eb/examples/showfps.c)
  - 创建 Renderer text engine；
  - 用内嵌字体和 `"Calculating..."` 调用 `TTF_CreateText`；
  - 每秒生成新的 FPS 字符串并调用 `TTF_SetTextString`；
  - 每帧调用 `TTF_GetTextSize` 和 `TTF_DrawRendererText`。

文字重建发生在每秒更新 FPS 字符串时；绘制调用每帧发生。内嵌字体减少了运行输入，但可见文本随帧率持续变化，无法提供稳定的完整精确字典键。

### 链接和 Windows 复现性

固定提交的 [`CMakeLists.txt`](https://github.com/libsdl-org/SDL_ttf/blob/a42434b8c96daaf7650dbd0befe480c090d1c2eb/CMakeLists.txt)
把项目版本标为 3.3.0 开发版本，默认共享构建，并定义 `showfps` 示例 target。它可以通过 pin 提交保持源码复现性，但没有稳定发布标签提供的版本边界。

### 判定

**首个可见翻译 smoke：No-Go。后续命中诊断：可用。生产目标：No-Go。**

它适合在 `showfont` 之后验证高频 draw 命中、持续 `TTF_SetTextString` 更新和生命周期处理，不适合验证固定完整字典键的首个用户可见翻译。

## 候选三：`testgputext` 3.2.2

### 身份与源码调用链

这是 SDL_ttf 官方稳定标签 `release-3.2.2` 中的 GPU 示例，不是外部真实软件。

- [`examples/testgputext.c`](https://github.com/libsdl-org/SDL_ttf/blob/release-3.2.2/examples/testgputext.c)
  - 调用 `TTF_CreateGPUTextEngine`；
  - 调用 `TTF_CreateText` 创建文字；
  - 每帧随机修改字符串前缀并调用 `TTF_SetTextString`；
  - 通过 `TTF_GetGPUTextDrawData` 取得几何数据并提交 GPU 绘制。

它没有进入 `TTF_DrawRendererText` 或 `TTF_DrawSurfaceText`。文字几乎每帧重建，且随机前缀使完整精确字典键不稳定。

### 链接和 Windows 复现性

3.2.2 官方 CMake 在 `SDLTTF_SAMPLES=ON` 时定义 `testgputext`，并在默认共享配置下链接共享 SDL3_ttf/SDL3 target。源码和构建入口位于稳定标签，但运行还依赖可用的 SDL GPU 后端。

### 判定

**当前 Renderer/Surface 适配器：No-Go。未来 GPU seam：可作为技术 smoke。生产目标：No-Go。**

把它纳入当前 smoke 会错误地把 GPU draw-data 路径当成 Renderer/Surface 公开绘制 API。只有在独立设计和实现 GPU text draw-data 适配后，才应重新启用该候选。

## API 事实

以下事实来自 SDL_ttf 官方 API 文档：

- [`TTF_DrawRendererText`](https://wiki.libsdl.org/SDL3_ttf/TTF_DrawRendererText)
  - 接收 `TTF_Text *` 和绘制坐标；
  - 文字必须由 Renderer text engine 创建；
  - 需要在创建该资源的同一线程调用；
  - API 自 SDL_ttf 3.0.0 起提供。
- [`TTF_CreateText`](https://wiki.libsdl.org/SDL3_ttf/TTF_CreateText)
  - 从 text engine、font 和 UTF-8 字符串创建 `TTF_Text`；
  - 需要同线程调用；
  - API 自 SDL_ttf 3.0.0 起提供。
- [`TTF_Text`](https://wiki.libsdl.org/SDL3_ttf/TTF_Text)
  - 公共结构中的 `text` 是 UTF-8 字符串副本；
  - 修改文字时该字段会更新；
  - 对象销毁时该字段释放。
- [`TTF_SetTextString`](https://wiki.libsdl.org/SDL3_ttf/TTF_SetTextString)
  - 设置新的 UTF-8 字符串；
  - 可能导致内部文字表示重建；
  - 需要同线程调用。
- [`TTF_GetGPUTextDrawData`](https://wiki.libsdl.org/SDL3_ttf/TTF_GetGPUTextDrawData)
  - 返回 GPU text engine 创建的文字几何数据；
  - 这是与 `TTF_DrawRendererText`/`TTF_DrawSurfaceText` 不同的绘制 seam。

SDL_ttf 官方 [`3.2.0 发布说明`](https://github.com/libsdl-org/SDL_ttf/releases/tag/release-3.2.0)
把 text objects/text engines 以及 Surface、Renderer、GPU text engine API 列为该代的重要能力；稳定审查基线则采用后续 bugfix 版本
[`3.2.2`](https://github.com/libsdl-org/SDL_ttf/releases/tag/release-3.2.2)。

## 最终 Go/No-Go 门槛

### 生产适配

**No-Go。** 当前没有外部真实软件证据。继续寻找目标时，必须由该软件自己的官方源码和 Windows 构建/发布资料证明：

1. 动态加载 SDL3_ttf 3.x；
2. 实际绘制经过公开 `TTF_DrawRendererText` 或 `TTF_DrawSurfaceText`；
3. 文字重建时机和 `TTF_Text` 生命周期可明确定位；
4. Windows 产物可重复构建或由官方发布；
5. 实际运行模块和 hook 命中可在授权环境中验证。

### 技术 smoke

**仅 `showfont release-3.2.2` 为 Go。** 它的作用是降低适配器技术风险，不是证明存在可交付的外部软件目标。若 `showfont` 的实际动态产物无法满足 DLL 导入、公开导出命中、精确 UTF-8 替换和代际恢复门槛，则 SDL3_ttf 适配继续保持 No-Go。
