# 通用 MonoGame Adapter 策略

## Deliverable

新增 `glyphshift-adapter-monogame` 框架级策略 crate，供任何 MonoGame `SpriteBatch.DrawString` / `SpriteFont.MeasureString` 接入组件复用。它不包含星露谷类名、方法名或资源路径。

## Current

已完成。Adapter 描述为 `windows.monogame.sprite-batch-draw-string`，声明 Windows x64、TargetProcess、InlineRender、TextObserve/TextReplace。`prepare` 接收 UTF-16 原文、目标字体字形谓词和既有 Runtime 决策回调，只有译文完整可编码且每个字符有字形时才返回替换，否则 fail-open。

4 项定向 Rust 测试通过：描述、缺字透传、畸形输入/回调异常、只观察模式。运行时 ReJIT 与像素合同由独立 test-support fixture 验证，不由此 crate 依赖。

## Boundary

该 crate 是通用决策层；CoreCLR Profiler/MonoGame 方法识别和 GPU 图集由平台伴随组件负责。它不能证明任意 MonoGame 游戏都使用标准 `DrawString`，也不能覆盖自绘逐字纹理、复杂 shaping、缺字图集生成或真实目标的生命周期。

## Next

后续直接把该策略接入 Runtime Bundle 的 MonoGame 原生组件，并用至少两个独立 MonoGame 目标完成真实命中与恢复验收；在此之前保持产品目录不声明 MonoGame 支持。此前 review 启动已验证该阻断：生产 Bundle 仅含 9 个既有适配器，没有 MonoGame 条目，因此星露谷零命中属于正确的未发布状态，而不是可接受的预览失败。
