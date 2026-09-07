# 活动适配器 x86 扩展

## 工件与验证矩阵

| 逻辑适配器 | x86 范围 | 本轮证据 |
| --- | --- | --- |
| 三项 GDI | 原有 Unicode 绘制入口 | 首批已验收，混合架构 Runtime 回归通过 |
| GDI+ | GdipDrawString | 系统绘制像素、独立文字/字体启停及 Runtime 注入恢复 |
| DirectWrite | 现有 TextLayout + D2D 路径 | 系统像素、复杂格式透传、Runtime 注入恢复 |
| Qt Painter | 全局命名空间 MSVC Qt 5；Qt 6 保持 x64 | 独立 MSVC C++ thiscall/导出名合同，四种 drawText、两代中文与恢复 |
| GTK 3/Pango | 现有动态 C ABI | 独立 C++ 调用者，浮点参数、layout 副本、局部属性透传与恢复 |
| raylib | 现有动态 5.5 C ABI | 独立 C++ 结构体传值/返回、译文动态字形、帧边界资源回收 |
| Unity Mono | 支持的 Mono 导出和标准 UI 主线程入口 | 独立 Mono 导出/JIT seam、对象快照、原生 setter 替换与恢复 |
| MonoGame | .NET 6+ CoreCLR 标准文字 | 实际 x86 CLR / MonoGame GPU 像素、六种调用、度量、缺字/压缩图集、更新和恢复 |
| Qt Quick | 暂无 x86 工件，仍为 Qt 6.8.3 x64 | 没有对应受支持上游 Windows x86 profile，未放宽门控 |

Qt/GTK/raylib/Unity 的新增 C++ 合同是 ABI 证据，不是真实框架像素或具体软件兼容性证明。MonoGame 宿主实际运行 MonoGame 与 CLR，但仍是合成场景。用户对首批 GDI 的验收不自动覆盖新增框架。

## 实现边界

Qt 的静态函数保持 C ABI，x86 成员函数使用 thiscall，并按平台宽度表达 WId / qsizetype。命名空间和版本匹配拒绝未核对的 Qt 6 x86；现有 x64 profile 保留。retour 补丁启用稳定工具链上的 thiscall 特性，并为该实现加 x86 条件。

Unity 原始时序问题由新原生合同复现：快照发布后，控制线程尚未安装 setter，主线程已推进 writeback 代次。修复后直到原 setter 可调用才刷新译文，不丢失首轮状态；x86 和 x64 同一合同均通过。

MonoGame 的 Host / Api 布局按指针宽度检查；Profiler 的 COM、Native ABI 和 P/Invoke 仍遵循各自调用约定。一个 DLL 继续承担 Adapter 和 Profiler，未新增长期助手。

## 可重复验证

- `scripts/test-x86-adapters.ps1`：x86 GDI+ / DirectWrite 系统像素和三个框架原生合同。
- `scripts/test-x86-adapters.ps1 -Architecture x64`：相同边界的 x64 回归。
- `scripts/test-monogame-x86.ps1 -DotnetPath <authorized-x86-dotnet-host>`：实际 32 位 .NET 6 / GPU 像素合同；路径必须由调用者提供。
- `scripts/build-runtime-bundle.ps1 -IncludeTestTarget -OutputRoot <local-test-root>/runtime` 后，通过 `GLYPHSHIFT_RUNTIME_ROOT` 指定该 Bundle，运行 desktop-runtime 的 `dual_architecture:: -- --ignored --test-threads=1` 合同。
- `scripts/test.ps1` 活动包回归：494 passed、0 failed、24 ignored。没有执行归档 UIA/OCR。

构建仍遵循全局 Cargo target 配置，原始证据仅在本地测试目录。最新 App 只能经 `scripts/review-app.ps1` 同步打包启动。

上游边界：[Qt 6.8 支持平台](https://doc.qt.io/qt-6.8/supported-platforms.html)、[MSVC thiscall](https://learn.microsoft.com/en-us/cpp/cpp/thiscall?view=msvc-170)。Windows Qt 6 没有官方 x86 profile；支持 Qt 5 Quick 或自编译 Qt 6 x86 属于新增版本适配，不能从现有 Qt Quick 工件直接推导。
