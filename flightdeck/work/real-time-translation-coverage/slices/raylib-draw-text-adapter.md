# raylib `DrawTextEx` 生产 Adapter

Status: Complete

## Goal

把已通过原型的 raylib 5.5 动态 `DrawTextEx` seam 接入正式 Native Runtime：观察完整 UTF-8 source，
查询当前 Dictionary，在有效渲染线程和 GPU context 中用经逐字形验证的 fallback Font 绘制译文，支持
同进程代次更新、停用恢复与关闭前安全释放。

## Delivery

- [x] 新增平台无关描述符/纯决策边界与 Windows x64 Native Adapter；功能只声明 `TextObserve` 和
  `TextReplace`，fallback Font 是内部实现要求，不把应用 Font 或 Dictionary Font 决策冒充成功。
- [x] 激活时验证动态 `raylib.dll` 和固定公开导出；缺模块、缺导出、非 UTF-8、过长文本、重入、线程
  不一致、字体缺字或 atlas 无效时全部安全放行。
- [x] 只在渲染线程构建公开 skyline atlas；新 atlas 成功后才切换，旧 atlas 在 `EndDrawing` 后释放，
  停用同样延迟到帧尾清理，`CloseWindow` 在 context 销毁前完成最终清理。
- [x] 接入 workspace、架构依赖合同、Native Host 激活合同、正式 Runtime Bundle 与中文 Adapter 目录。
- [x] 用正式 Bundle 在固定 Musializer 动态目标复测首代、第二代、停用和关闭清理，并完成 workspace、
  Clippy 与架构检查。

## Current

生产接入已完成。正式 Bundle 在固定 Musializer 动态目标中取得首代与第二代各 256 条
`Matched + Replaced` 诊断；首代空格与第二代换行均取得完整中文像素，停用后恢复英文原文。随后重新
激活含空格的第三代并在 atlas 仍活跃时直接关闭，目标通过正常窗口生命周期干净退出。多行译文仍使用
应用给定的位置和行距，不擅自解决与相邻应用控件的布局重叠。

平台无关逻辑 4/4、Native 控制字符归一化 1/1、无 raylib 模块的 Native Host fail-closed 合同 1/1、
完整 workspace、全 workspace Clippy `-D warnings` 与架构检查均通过。正式 Bundle 包含 9 个 Adapter；
真实字体、截图、原始日志、构建物和运行描述仍只保存在本机私有测试目录。

## Next

- 进入 [raylib 之后的下一写回 Adapter 选择](post-raylib-next-writeback-adapter.md)，继续增加真实
  `TextReplace` 覆盖；UI Automation 仍只作为查看/采集能力。

## Boundaries

- 首版只覆盖 Windows x64 动态 raylib 5.5 兼容 ABI 与公开 `DrawTextEx`。
- 不覆盖静态链接、DLL 改名、复制 raylib 文字实现、直接 `DrawTextCodepoint(s)`、业务 texture 缓存、
  自定义 `Font` ABI 或 context 在未经过 `CloseWindow` 的情况下被私下替换。
- fallback 字体必须来自显式本地测试输入或可验证的 Windows 字体候选；每个目标 codepoint 的 bitmap
  和 atlas rectangle 都必须通过，不能以 `?`、空 glyph 或默认 Font 充当译文。
- 真实软件、字体路径、构建物、日志与截图只保存在本机私有测试目录。

## References

- [raylib fallback Font/atlas 原型](raylib-fallback-font-atlas-prototype.md)
- [SDL3_ttf 之后的动态 C 文字入口复核](../references/post-sdl3-next-dynamic-c-seam-review.md)
