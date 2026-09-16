# Wireshark

当前决定：**作为 `windows.qt.translation-service` 的首个真实动态 Qt 代表目标，Qt 6.10.3 x64 已完成生产晋级。**

固定代表版本为 Wireshark 4.6.8，官方源码 tag 对应 commit
`e677bf052328efc1ed897a547fa161836a0e4ff7`。Windows 代表安装动态加载 Qt 6.10.3，主程序直接导入 MSVC x64
`QCoreApplication::translate` 导出，因此可以建立独立于软件身份的精确 Qt profile。

## 为什么它提供增量覆盖

- `ui/qt/main_window.cpp` 的空捕获主标题使用 `tr("The Wireshark Network Analyzer")`，随后交给
  `setWindowTitle()`。翻译服务仍持有完整源字符串时，最终非客户区标题由 Windows 绘制，不再经过目标进程内的
  Qt Painter、Qt Quick 或 QTextDocument 文字入口。
- `ui/qt/wireshark_main_window.cpp` 明确处理 `QEvent::LanguageChange`：调用 `main_ui_->retranslateUi(this)` 后
  `updateTitlebar()`。因此主标题同时能验证“未来调用替换”和 Qt 官方语言变更刷新，而不需要软件专属刷新 Hook。
- Qt 6.10.3 `QCoreApplication::installTranslator/removeTranslator` 会向 application 发送
  `QEvent::LanguageChange`；`QGuiApplication` 再把该事件排队传播到顶层窗口。Glyphshift 原型复用这条机制，
  `request_refresh` 只异步 `postEvent`，不从 Runtime 控制线程同步操作 Qt UI。

## 已验证 ABI

- 动态模块：默认 production build 只接受 `Qt6Core.dll` 且 `qVersion() == 6.10.3`。Qt 5.15.18 ABI 仅通过显式
  `research-qt5` Cargo feature 保留给研究构建，不进入默认生产 Bundle。
- 导出：`?translate@QCoreApplication@@SA?AVQString@@PEBD00H@Z`。
- MSVC x64 返回 ABI：隐藏首参为 `QString* result`，随后依次为 `context`、`sourceText`、
  `disambiguation`，`n` 位于栈上传参；返回寄存器指向 result。
- 写回不直接改 `QString` 内部布局。命中时只把 Glyphshift replacement 作为本次 `sourceText` 交给原始
  `QCoreApplication::translate`，原函数仍恰好调用一次并由 Qt 自己构造返回对象；未命中完全保留原参数。

## 真实运行结果

最新实机 smoke 已把 V2 context 与 plural 门槛一并闭环：

1. 激活前主窗口标题为 `The Wireshark Network Analyzer`。
2. Runtime 激活 `windows.qt.translation-service` 后，首代 Dictionary 将该 source 映射为
   `Glyphshift Generation One`，原生顶层窗口标题可见变更为首代文本。
3. 本轮取得 622 条唯一 `(source, context, disambiguation, plural_n)` observation、613 条唯一 source，
   `droppedObservations=0`；622 / 622 都带 context，1 条带非空 disambiguation。
4. 同一 `Cancel` source 真实出现在 `QPlatformTheme`、`SearchFrame` 与 `WiresharkMainWindow` 三个 context。早期研究阶段曾用
   分 context publication 分别取得对应 `Matched + Replaced` Runtime trace，证明 V2 evidence 能区分这些调用。该实验
   不再代表当前 Dictionary 模型：产品词典现已恢复 source-only，这些 context 只保留为观测证据 / AI 提示。
5. 用临时启动覆盖 `gui.interfaces_hidden_types:0,5,9` 触发 Welcome Page 的官方 numerus 路径，真实捕获
   `%n interface(s) shown, %1 hidden` / `WelcomePage` / `plural_n=7`。publication 故意包含同 source 的普通
   bait translation，generation 2 与 3 仍分别记录 `NoMatch + Unmatched`，因此真实 plural 调用保持 observe-only / fail-open。
6. 主窗口标题随 generation 2 / 3 分别更新为 `Glyphshift Generation Two` / `Glyphshift Generation Three`；
   停用 Runtime 后恢复 `The Wireshark Network Analyzer`，目标随后正常退出。
7. 完成产品晋级后，又直接使用 Release Runtime Bundle 中带 hash 文件名的 Runtime 与 Qt translation DLL 重跑同一
   smoke，仍取得 622 条 observation、0 capture drops、`plural_n=7`、context evidence、三代标题刷新、停用恢复与正常
   退出；因此证据直接覆盖实际分发工件，而不只覆盖 Cargo target 目录中的研究 DLL。

另有真实 Qt 6.10.3 native-host 合同直接调用被 detour 的 `QCoreApplication::translate`，验证首代
`First translation`、第二代 `Second translation` 与停用后的原文 `Open`。同一合同还使用 Wireshark 实采的
`MainStatusBar` source `Profile: %1` 验证 placeholder gate：保留 `%1` 的 `配置：%1` 可以替换，删除 `%1` 的
`配置` 被 native hook 拒绝并返回原文。这样 C++ 返回 ABI、placeholder fail-open 与 Wireshark UI 生命周期可以分层验证。

旧 source-only decision host 现在与产品 Dictionary 语义一致：任何 `n == -1` 的非复数调用都可按 source 使用 V1
replacement，即使带 context / disambiguation；plural 与未知负 `n` 继续 fail-open。

## 生产边界

- Qt 6.10.3 的即时刷新依赖 `QEvent::clone()` + `QCoreApplication::postEvent()` 的已验证公共导出。Qt 5.15.18
  没有同一 `QEvent::clone()` 导出，因此 Qt 5 研究 profile 只承诺未来 translate 调用读取新 generation，不宣称
  对既有 Widgets 立即刷新，也不进入默认生产工件。
- V2 context、真实 plural observe-only、placeholder gate 与生命周期门槛均已有代表目标证据。默认 Qt6-only native
  build 已加入 x64 Runtime Bundle；Release Bundle verifier 对整包 17 个唯一 Adapter ID 验证通过。
- 实现仍没有 Wireshark 品牌分支、固定地址或窗口标题判断；profile 只按 Qt 动态模块、版本、架构与已验证导出 gate。

## 恢复时先做

Qt 6.10.3 生产路径已经闭环。若继续本路线，只研究 Qt 5.15.18：先找到允许通用注入的代表目标并补齐同等级真实运行
证据；Qt 5 即时刷新还需独立证明 queue-owned `QEvent` 分配 / 所有权 ABI，不能由当前 Qt 6 证据外推。研究期间保持
`research-qt5` 显式 feature，不能重新进入默认生产 Bundle。

一手资料：

- [Wireshark 官方仓库](https://gitlab.com/wireshark/wireshark)
- [Qt `QCoreApplication`](https://doc.qt.io/qt-6/qcoreapplication.html)
- [Qt `QEvent::LanguageChange`](https://doc.qt.io/qt-6/qevent.html#Type-enum)
- [Qt Translator / QCoreApplication::translate 第一方资料复核](../qt-translator-primary-source-review.md)
