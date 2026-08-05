# SDL3_ttf `showfont` 技术 smoke

Status: Complete

## Goal

用 SDL_ttf 官方 `showfont` 示例验证 SDL3_ttf 3.2.2 的动态模块与公开 Renderer Text 绘制边界是否足以
承载 GlyphShift 的绘制时替换合同。该示例不是外部真实软件；本切片通过也不授权生产 Adapter 接入。

## Delivery

- [x] 在本机私有测试目录取得固定版本官方源码，并用 Windows x64 动态配置构建 `showfont`。
- [x] 确认运行目标实际加载 SDL3_ttf 动态库，固定测试文本在 caption 或 EditBox 路径进入
  `TTF_DrawRendererText`；缓存 Surface/Texture 的中央文字不计入验收。
- [x] 证明完整 UTF-8 原文可在绘制时安全读取，并评估瞬时译文 `TTF_Text` 所需的 Engine、字体、颜色、
  wrap、方向与脚本属性复制边界。
- [x] 建立有界本地原型，验证首代译文、同进程第二代更新、停用恢复和不明确属性时 fail-open。
- [x] 记录 Go/No-Go；无论结果如何，在外部真实目标出现前不新增生产 crate、Catalog 或 Runtime Bundle。

## Result

- 官方 3.2.2 `showfont` 的 Windows x64 动态构建成功，运行进程实际加载 SDL3 与 SDL3_ttf 动态库。
- 本机临时 Hook 在 `TTF_DrawRendererText` 读取到完整固定 UTF-8 source；瞬时译文 Text 复用同一 Engine
  与字体，并复制颜色、位置、wrap 宽度、wrap 空白可见性、方向与脚本后，原始 Draw 调用返回成功。
- 离屏 Renderer 像素依次显示第一代中文译文、同进程第二代中文译文和停用后的完整原文；目标进程
  全程没有重启。
- 中央大号文字始终保持原文，符合其初始化时生成 Surface/Texture 并缓存的源码边界；本切片没有把
  该区域误计为 Text Draw 覆盖。
- 原始日志、位图、字体、运行描述、源码、构建物与临时 Hook 都只保存在本机私有测试目录。

## Decision

**技术 seam：Go；生产 Adapter：No-Go，继续暂缓。** `TTF_Text` + `TTF_DrawRendererText` 已证明能实现
安全的绘制时替换、热更新和停用恢复，但官方示例不能证明外部软件覆盖价值。在合格的动态链接外部
真实目标出现前，不新增 SDL3_ttf 生产 crate、Catalog 项、Runtime Bundle 或支持声明。

## Boundaries

- 所有下载、字体、构建物、进程信息、截图、原始日志与临时探针仅保存在本机私有测试目录。
- 不把官方示例描述成真实软件覆盖，不扩大产品支持声明。
- 不处理 GPU draw-data、静态链接、Surface/Texture 缓存文字或自定义 Text Engine。
- 不为单个程序或版本引入签名扫描、私有结构或品牌分支。

## References

- [SDL3_ttf 真实目标候选筛选](../references/sdl3-ttf-real-target-candidates.md)
- [Tk 之后的实时文字入口复核](../references/post-tk-runtime-seam-review.md)
