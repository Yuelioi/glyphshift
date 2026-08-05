# WPF 真实目标缺口验收

Status: Complete

## Goal

用一个用户授权、正在运行的常见 WPF 桌面工具验证现有 Adapter 是否经过其真实文字路径，并判断 WPF
能否按当前 Native `TextReplace` 合同继续立项。

## Evidence

- 进程模块确认目标运行在 .NET/WPF，加载 WPF 原生渲染组件、DirectWrite 与 Direct2D；未加载 Qt 或
  GTK。模块存在只用于技术识别，不算文字入口命中。
- 正式 UIA Isolated Worker 合同 1/1 通过：5 秒内得到 72 条唯一公开文本，health 为 Healthy，证明
  结构化观察有真实收益，但仍是 observe-only。
- 对窗口及其子树持续强制重绘后，以下六条路径均完成注入、激活、诊断、停止且命中为 0：

  - `windows.gdi.ext-text-out`
  - `windows.gdi.text-out`
  - `windows.user32.draw-text`
  - `windows.gdiplus.draw-string`
  - `windows.direct2d.draw-text`
  - 本地实验 `windows.directwrite.text-layout`

- 初次测试使用软件更新器暴露的入口目录，进程报告实际版本目录，精确路径 inventory 为 0；改用与
  产品预检相同的规范化实际路径后，六项测试均正常进入 Runtime。没有放宽为不安全的仅文件名授权。
- 原始进程、模块、路径、日志和公开文本只保存在忽略的本机证据目录，不进入 Work 或提交。

## Decision

WPF 对当前公共 Native 绘制入口作 **No-Go**。它采用 retained-mode：Visual 持久化序列化绘制数据，
系统负责后续重绘；加载 `dwrite.dll` / `d2d1.dll` 不代表界面会经过公开 `DrawText` 或
`DrawTextLayout`。真实零命中与该模型一致。

直接设置 `TextBlock.Text` 也不作为替代方案：该属性是影响 Measure/Render 的 Dependency Property，
写入会改变应用对象状态，并可能干扰 Binding、Inline 和业务更新，不满足“只替换本次显示、停用恢复”
合同。

WPF 仍有两个独立后续方向，但都不是把现有 DirectWrite 原型重新发布：

1. **Managed WPF Agent**：通过官方 .NET Diagnostics/Profiler attach 与 ReJIT 进入托管文字格式化链，
   还必须证明不修改业务属性、可使 retained visual 重新格式化、字典热更新和停用恢复。它属于
   Engine-aware Extension，风险与实现面显著高于 Native Adapter。
2. **UIA 驱动的外部应用模型**：复用已验证的结构化文本，再用独立翻译面板或最小定位 Overlay 呈现；
   不宣称原位写回，也不建立万能 Renderer。

在产品选择 Apply Model 前，不实现 WPF 私有符号 Hook、Dependency Property 全局改写或版本地址表。

## Primary sources

- [WPF Graphics Rendering Overview](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/graphics-multimedia/wpf-graphics-rendering-overview)
- [`TextBlock.Text` Dependency Property](https://learn.microsoft.com/en-us/dotnet/api/system.windows.controls.textblock.text)
- [WPF `TextFormatter` source](https://github.com/dotnet/wpf/blob/main/src/Microsoft.DotNet.Wpf/src/PresentationCore/System/Windows/Media/textformatting/TextFormatter.cs)
- [WPF internal `TextFormatterImp` source](https://github.com/dotnet/wpf/blob/main/src/Microsoft.DotNet.Wpf/src/PresentationCore/MS/internal/TextFormatting/TextFormatterImp.cs)
- [.NET DiagnosticsClient profiler attach](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/diagnostics-client-library)
- [`ICorProfilerInfo4::RequestReJIT`](https://learn.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/icorprofilerinfo4-requestrejit-method)
