# Tk 之后的实时文字入口复核

Status: Complete

## 结论

当前没有另一个应直接进入生产 Bundle 的通用原生 Adapter。SDL_ttf 需要按 major version 分开判断：

- **SDL2_ttf 当前 No-Go。** 它能在公开 C API 中取得完整 UTF-8，并返回按译文重新生成的
  `SDL_Surface`；但这是文字表面创建时替换，不是每次显示时写回。应用可以长期缓存 Surface 或由其生成的
  Texture，GlyphShift 无法通用地使既有缓存立即热更新或在停用后恢复原文。
- **SDL3_ttf 进入 Roadmap 候选。** SDL_ttf 3 引入持有 UTF-8 副本的 `TTF_Text`、Text Engine 和公开
  `TTF_DrawRendererText` / `TTF_DrawSurfaceText` 绘制入口。理论上可以在每次绘制时创建临时译文对象，
  不修改应用持有的原对象，并自然满足下一代 Dictionary 与停用恢复；但目前没有一个有代表性的授权
  Windows 软件证明动态模块、真实入口命中和可见收益。

因此不新增 SDL2 Adapter，也不为 SDL3 先造生产选项。SDL3 只有在真实软件先证明使用动态 SDL3_ttf
文字对象绘制链时，才打开合成宿主和 Adapter 原型。

## SDL2_ttf 为什么不满足当前实时合同

SDL2_ttf 的 `TTF_RenderUTF8_Blended`、Solid、Shaded、LCD 及 Wrapped 族都接受完整 UTF-8 和字体参数，
并分配一个新的 Surface 返回给调用者。`TTF_SizeUTF8` 也能计算同一字符串的像素尺寸。这说明它是明确、
稳定且可精确匹配 Dictionary 的 C ABI，并非与静态 Skia 或 Dear ImGui 完全相同的问题。

但 API 的所有权边界也意味着 Adapter 只能替换“这一次创建的 Surface”。调用者取得 Surface 后可以转成
Texture、缓存标签或只在状态变化时重新生成；SDL_ttf 不拥有后续 `SDL_RenderCopy`，而复制 Texture 时已经
没有原文。Hook 尺寸函数也只能修正下一次创建前的测量，不能找回既有缓存。

这会破坏当前生产门槛中的两项：

1. 发布第二代 Dictionary 后，既有静态文字不保证在下一帧使用新译文；
2. 停用 Adapter 后，既有译文 Surface/Texture 不保证恢复原文。

SDL2_ttf 仍可作为未来“文字资源生成时替换”能力，并可覆盖 Pygame `pygame.font` 等基于 SDL_ttf 的
应用；但必须以弱于实时写回的能力名称和状态呈现，不能计入当前 `TextReplace` 覆盖。

## SDL3_ttf 为什么是不同候选

SDL_ttf 3 的 `TTF_CreateText` 直接接受 UTF-8，并由 `TTF_Text` 保存文字副本；官方 Text Engine 可以面向
Surface、SDL Renderer 或 SDL GPU。`TTF_DrawRendererText` 与 `TTF_DrawSurfaceText` 在每次绘制时接收
`TTF_Text`，因此 Adapter 可以：

1. 从原对象读取完整 UTF-8；
2. 命中 Dictionary 时，用公开 `TTF_GetTextEngine`、`TTF_GetTextFont` 和样式 getter 创建临时译文对象；
3. 调用原绘制函数绘制临时对象，随后销毁；
4. 未命中、属性无法完整复制、重入或异常时原样放行。

这条路径不会修改应用对象，下一次绘制会重新决策，因此在语义上具备实时热更新和停用恢复条件。需要继续
验证的风险是：自定义 Text Engine、属性复制完整性、SDL GPU 入口、动态/静态链接比例以及真实应用采用率。

## 使用场景与优先级

- SDL2 广泛用于视频播放、模拟器、游戏和多媒体应用；Pygame 官方也明确说明 `pygame.font` 基于
  SDL_ttf。它证明存在实际文字来源，但不消除 Surface/Texture 缓存边界。
- SDL_ttf 3 的 Text Engine 与文字对象在 2025 年正式发布，官方强调动态 glyph cache、字体变化后自动
  更新及 Surface/Renderer/GPU 三类输出；这是合适的未来绘制时 seam，但当前不以新 API 的存在替代真实
  软件证据。
- Chromium/Electron、Java 和托管 UI 仍需要 Extension、agent 或运行时级授权，不属于下一个可直接加入
  当前 Native Bundle 的 Adapter；继续保留在 Engine-aware Roadmap。

## Go gate

SDL3_ttf 重新立项必须先满足：

1. 一个授权 Windows 软件动态加载 SDL3_ttf，且实际命中公开 `TTF_Draw*Text`；
2. 真实文字以完整 UTF-8 可观察，Dictionary 精确匹配；
3. 临时对象能保留该目标实际使用的字体、颜色、wrap、方向、script 与 engine；
4. 第一代、第二代 Dictionary 和停用恢复均在同一进程可见；
5. 自定义 engine、静态链接或不完整属性必须稳定报告不兼容并失败开放。

满足之前，不建立 SDL3_ttf crate、Catalog 项或用户选项。

## 第一方资料

- [SDL2_ttf `TTF_RenderUTF8_Blended`](https://wiki.libsdl.org/SDL2_ttf/TTF_RenderUTF8_Blended)
- [SDL2_ttf `TTF_SizeUTF8`](https://wiki.libsdl.org/SDL2_ttf/TTF_SizeUTF8)
- [SDL3_ttf `TTF_Text`](https://wiki.libsdl.org/SDL3_ttf/TTF_Text)
- [SDL3_ttf `TTF_CreateText`](https://wiki.libsdl.org/SDL3_ttf/TTF_CreateText)
- [SDL3_ttf `TTF_DrawRendererText`](https://wiki.libsdl.org/SDL3_ttf/TTF_DrawRendererText)
- [SDL3_ttf `TTF_DrawSurfaceText`](https://wiki.libsdl.org/SDL3_ttf/TTF_DrawSurfaceText)
- [SDL_ttf 3 正式发布说明](https://discourse.libsdl.org/t/announcing-the-sdl-ttf-3-official-release/58268)
- [Pygame Font 文档](https://www.pygame.org/docs/ref/font.html)
