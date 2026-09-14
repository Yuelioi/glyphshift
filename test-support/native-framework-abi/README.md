# 原生框架 ABI 合同

MSVC 编译的独立合成 DLL，验证 Qt 5 C++ thiscall / 导出修饰、SideFX CV PaintBuffer 的 x64 C++ member ABI，GTK C ABI、raylib 结构体传值/返回与资源回收，以及 Mono 导出、主线程 dispatch、setter 替换和恢复。

这些夹具不包含真实 Qt/SideFX CV/GTK/raylib/Unity 实现，不作为真实软件像素兼容证明。它们验证 Rust 钩子和另一种语言编译的调用者之间的 ABI；GDI+/DirectWrite 另有系统绘制像素合同。所有生成物放在本地测试目录。

Qt 夹具另提供可选的实际 GDI 单字绘制：同一测试进程同时加载生产 Qt/GDI Adapter 与 Runtime，验证完整调用范围内只采集完整原文、不重复翻译字形；范围外的 GDI 单字仍保留。测试同时覆盖换代与停用恢复，不依赖具体游戏或软件路径。

运行 `scripts/test-x86-adapters.ps1` 可重建夹具并执行 x86 合同；`-Architecture x64` 验证相同边界的 x64 回归。不存在 UIA/OCR 执行入口。

x86 构建需要 MSVC x86 工具和 `rustup target add i686-pc-windows-msvc`。发布工作流同步安装该 Rust target，桌面壳仍构建为 x64。

Qt Painter 的 x86 路径限定 MSVC Qt 5，全局命名空间；Qt 6 / QT 命名空间仍走已验证的 x64 路径。Qt Quick 仍限定现有 Qt 6.8.3 x64 profile，不能因基础 Runtime 支持 x86 而推导所有 Qt 版本可用。

SideFX CV fixture 只在 x64 构建，精确导出 `CV_PaintBuffer::textWrappedInBox` 的已验证 MSVC 修饰符，用来验证生产 Adapter 的文本参数替换、换代与停用恢复；缺少 `libCV.dll` 或该导出时，生产 Adapter 必须 fail-closed。
