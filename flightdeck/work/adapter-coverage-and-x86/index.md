# 活动适配器双架构扩展

Status: Open

## Goal

保持一个 x64 App 和一个逻辑 Adapter 身份，将默认活动适配器扩展为可验证的 x86/x64 工件；按上游框架版本和实际 ABI 保留边界，不把架构支持等同于任意软件覆盖。

## Current

首批双架构 GDI、App 探针门控修复及 0.3.0 已由用户验收，提交为 `29c74ac`。用户进一步授权补齐其他活动适配器，并要求提交本轮扩展、启动最新 App 供实机测试。

Runtime Bundle 已扩展为 11 个逻辑适配器、其中 10 个提供 x86 工件：GDI 三项、GDI+、DirectWrite、Qt Painter（Qt 5 MSVC）、GTK 3/Pango、raylib、Unity Mono、MonoGame。Qt Quick 仍限定 Qt 6.8.3 x64；Windows Qt 6 官方支持表没有 x86，不能直接把现有 profile 宣称为 32 位可用。

Qt Painter 增加 x86 thiscall 与 MSVC 导出名映射，保留 x64 Qt 5/6 和 Qt 6 QT 命名空间。GTK/raylib 原 C ABI 已通过独立 C++ 宿主合同。MonoGame 增加 x86 C++ 构建和原生结构体尺寸校验。Unity Mono 移除仅 x64 门控，同时修复原有的快照先于 setter 就绪导致首轮译文代次丢失的竞争窗口。

验证：活动包 494 passed、0 failed、24 ignored；x86 原生系统绘制合同 2 项、独立框架 ABI 合同 3 项，x64 框架 ABI 回归 3 项均通过。扩展后的实际 Runtime 双架构合同 3 项通过，其中包含 GDI+ / DirectWrite 的 x86 注入、像素替换与恢复。MonoGame 的 32 位 CLR + GPU 像素合同通过六种 DrawString、MeasureString、中文缺字与压缩图集回退、更新和停用恢复。受影响 x86 生产库严格 Clippy 通过。

合成 ABI 夹具不包含真实 Qt/GTK/raylib/Unity，不能替代这些框架的真实软件验收。此前字体候选/预算、DirectWrite 长寿命 layout、Qt Quick AutoText 等审计事项仍独立存在。本轮没有对求生之路实施新注入，也未新增 Source/VGUI/D3D9 文字入口。

新增验证脚本均已运行通过；发布工作流补充 i686 Rust target 安装并通过 actionlint。最新 0.3.0 已通过 review-app 同步构建启动，生产加载器两次验证通过，运行进程正常响应；打包清单确认 x64 11 项、x86 10 项。前端构建通过，WebView 调试端口仍未开放，本轮未将其表述为 GUI 自动验收通过。

## Next

等待用户在最新 App 中验收指定真实 x86 软件，按 [扩展验收说明](references/expanded-adapters.md)记录具体框架与入口。本轮提交前已检查暂存路径和文本，未包含本机测试证据。Qt 5 Quick / 自编译 Qt 6 x86 profile 和 Source/VGUI 游戏入口仍是独立后续任务。

## References

- [活动适配器扩展矩阵与验收](references/expanded-adapters.md)
- [双架构实现与验收边界](references/dual-architecture-implementation.md)
- [研究范围](context.md)
- [字体与适配器审计](references/adapter-audit.md)
- [x86 可行性与阶段](references/x86-feasibility.md)
- [成熟项目对比与方案权衡](references/architecture-comparison.md)
- [Windhawk 与 OBS 源码证据](references/comparison-windhawk-obs.md)
- [Frida 与 Detours 源码证据](references/comparison-frida-detours.md)
- [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)
