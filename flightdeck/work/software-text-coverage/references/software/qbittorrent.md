# qBittorrent

当前决定：**保留为 Qt 翻译服务的增量覆盖研究样本，但不再作为当前动态 Qt 原型的真实代表目标。**
qBittorrent 5.2.3 的官方源码仍证明真实 `QTranslator` 使用，以及一条现有 Qt Painter / Qt Quick /
Qt Text Document 都不负责绘制的系统通知文字路径；但对正式 Windows 5.2.3 release 的实际载荷复核没有得到
独立 `Qt6Core.dll` 等动态模块，和先前从 CI 配方推断出的动态发行前提不一致。当前 Adapter 要求精确动态模块 /
导出 gate，因此不能为了继续使用 qBittorrent 而放宽为静态扫描或软件专属入口。

## 已验证事实

- qBittorrent 官方仓库把项目定义为 C++ / Qt BitTorrent 客户端；本轮固定复核稳定 tag
  `release-5.2.3`。`Application` 持有 `m_qtTranslator` 与 `m_translator` 两个 `QTranslator`，启动阶段加载
  Qt 与 qBittorrent 自身 `.qm` 后调用 `installTranslator()`。这不是由 Qt DLL 存在推断出来的框架关系，而是
  目标自己的翻译调用链。
- qBittorrent 5.2.3 的官方 Windows CI 使用 Qt 6.10.1，并有复制 Qt 动态库的构建步骤；但本轮随后对正式
  Windows 5.2.3 release 的实际载荷复核没有发现独立 Qt DLL。CI 配方因此不能替代发布物验证，当前动态模块
  gate 按 fail-closed 处理为不满足。
- 下载完成、添加 torrent、搜索结果以及最小化到托盘等通知文字在 qBittorrent 源码中由 `tr(...)` 产生，随后
  经 `DesktopIntegration::showNotification()` 进入 `QSystemTrayIcon::showMessage()`。
- Qt 6.10 一手源码确认 `QMetaObject::tr()` 调用 `QCoreApplication::translate()`；后者遍历已安装
  Translator。Windows 平台的 `QWindowsSystemTrayIcon::showMessage()` 把标题与正文写入
  `NOTIFYICONDATA::szInfoTitle/szInfo`，最后调用 `Shell_NotifyIcon()`。
- 因此这批系统通知不是 qBittorrent 进程内的 `QPainter::drawText`、Qt Quick Text 或
  `QTextDocument::setHtml` 绘制。当前生产 Adapter 也没有 `QSystemTrayIcon` / `Shell_NotifyIcon` 入口。
  Qt 翻译服务位于文字交给 Windows Shell 之前，能形成真实的**增量完整字符串 seam**，不是给既有 Qt
  Painter 再包一层框架名字。

## 当前边界

- 以上证据证明“值得做原型”，不证明晚附加后已经建立的 Widgets 会即时重译。qBittorrent 5.2.3 当前源码
  没有给出可直接依赖的应用级 `LanguageChange -> retranslateUi()` 全局刷新合同，因此已有界面的刷新能力必须
  与“未来发生的翻译查询能被观察 / 替换”分开验收。
- qBittorrent 的系统通知仍是很好的“Translator 相对 Painter/Quick/TextDocument 有增量价值”一手研究证据，
  但不再承担当前 ABI / 注入实机验收。
- Qt major、精确版本 / 工具链、`QString` 返回 ABI 与导出必须 fail-closed。当前只把 qBittorrent 作为代表
  目标，正式 Adapter 身份仍应定义为 Qt translation service，而不是 qBittorrent 分支。
- `qtTrId` / ID-based 翻译不能自动还原 Dictionary 所需 source text；首版应继续放行或单独标记能力缺口。

## 恢复时先做

若未来要重新把 qBittorrent 纳入真实验证，先重新确认目标 release 是否提供可独立门控的动态 Qt profile；如果
仍是单体 / 静态 Qt，则必须另立“静态 Qt translation seam”设计与通用识别合同，不能复用当前动态 DLL gate，
也不能按 qBittorrent 地址做补丁。当前动态原型的真实代表目标已转到 Wireshark 4.6.8。

一手资料：

- [Qt Translator / QCoreApplication::translate 第一方资料复核](../qt-translator-primary-source-review.md)
- [qBittorrent 官方仓库](https://github.com/qbittorrent/qBittorrent)
- [Qt QCoreApplication::translate](https://doc.qt.io/qt-6/qcoreapplication.html#translate)
- [Qt QTranslator](https://doc.qt.io/qt-6/qtranslator.html)
- [Qt `qcoreapplication.cpp`](https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp)
- [Qt Windows `QSystemTrayIcon` 实现](https://code.qt.io/cgit/qt/qtbase.git/tree/src/plugins/platforms/windows/qwindowssystemtrayicon.cpp)
