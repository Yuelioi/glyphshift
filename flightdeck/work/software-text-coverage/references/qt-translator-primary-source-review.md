# Qt Translator / QCoreApplication::translate：第一方资料复核

Status: Complete

## 结论

**GO：有界原型已在 Wireshark 4.6.8 / Qt 6.10.3 上通过真实软件验证；当前仍不等于生产支持。**

Qt 的 `QTranslator` / `QCoreApplication::translate` 已证明值得保留为 Glyphshift 的通用文字 Adapter seam，且
当前原型已经越过“只值得研究”的阶段：Wireshark 4.6.8 官方 Windows 安装动态加载 Qt 6.10.3，主程序直接导入
精确 MSVC x64 `QCoreApplication::translate` 导出。实机 Runtime 最新闭环捕获 229 条唯一 translation source、
`droppedObservations=0`；主窗口原生标题从 `The Wireshark Network Analyzer` 变成首代
`Glyphshift Generation One`，publication 更新后变成二代 `Glyphshift Generation Two`，停用后恢复原始标题。
该标题最终由 Windows 非客户区绘制，因此相对现有 Qt Painter / Qt Quick / QTextDocument 是真实增量覆盖。

早期 qBittorrent 5.2.3 研究仍保留在本文后续章节，因为它的系统通知链继续证明 translation seam 的结构性增量
价值；但**它不再是当前 ABI 实机代表目标**。正式 Windows 5.2.3 release 的实际载荷没有得到独立 Qt DLL，不能
满足当前动态模块 gate。KeePassXC 2.7.12 随包提供 Qt 5.15.18，但 release 会主动收紧自身进程 DACL，只开放
limited-query / terminate 等最小权限并拒绝 Glyphshift 远程注入所需权限，因此也按 fail-closed 排除，没有加入
绕过或软件专属逻辑。

本文记录的是早期第一方资料复核；后续实现已补齐 V2 evidence，并明确作出产品取舍：普通 Dictionary 保持
source-only，`context/disambiguation` 保留为观测证据和 AI 提示而不进入词典 identity，plural 仍 observe-only。
Qt 6.10.3 的即时刷新已通过异步 `QEvent::LanguageChange` 闭环并完成生产晋级；Qt 5.15.18 仍没有等价的安全即时
刷新方案。

## 代表软件：qBittorrent 5.2.3

### 知名、公开可获得、现实用户价值

qBittorrent 是长期公开发布的开源 C++ / Qt 桌面 BitTorrent 客户端；项目官网持续提供 Windows 等平台的正式下载，
源码由项目官方仓库公开维护。本研究固定到 **release-5.2.3 / commit
`0b63c3d17373f6132ea211c9dcd4241284ccdfaf`**，不依赖第三方软件榜单、下载站或二手资料来证明目标身份。

- 官方下载页：<https://www.qbittorrent.org/download>
- 官方仓库：<https://github.com/qbittorrent/qBittorrent>
- 本轮固定源码版本：<https://github.com/qbittorrent/qBittorrent/tree/0b63c3d17373f6132ea211c9dcd4241284ccdfaf>

这满足当前 Adapter 新增标准里的“至少一个相对知名、公开可获得、有现实用户价值的真实软件”门槛；后续生产
实现仍必须按 Qt translation seam 建模，不能出现 qBittorrent 品牌分支。

## qBittorrent 确实使用 Qt translation system

这条证据不是由 Qt DLL 存在反推，而是来自应用自己的官方源码。

`Application` 直接持有两个 `QTranslator` 成员：一个用于 Qt 自身翻译，一个用于 qBittorrent 资源翻译。
[application.h](https://github.com/qbittorrent/qBittorrent/blob/0b63c3d17373f6132ea211c9dcd4241284ccdfaf/src/app/application.h#L39)
[application.h](https://github.com/qbittorrent/qBittorrent/blob/0b63c3d17373f6132ea211c9dcd4241284ccdfaf/src/app/application.h#L200-L201)

启动阶段的 `initializeTranslation()` 从 Qt translation path 加载 `qtbase_<locale>` / `qt_<locale>`，再通过
`installTranslator(&m_qtTranslator)` 安装；随后从资源 `:/lang/qbittorrent_<locale>` 加载应用语言包，并通过
`installTranslator(&m_translator)` 安装第二个 Translator。这已经直接证明 qBittorrent 的本地化链使用
`QTranslator` / `QCoreApplication`，不是仅仅使用 Qt Widgets 绘制。
[application.cpp](https://github.com/qbittorrent/qBittorrent/blob/0b63c3d17373f6132ea211c9dcd4241284ccdfaf/src/app/application.cpp#L1155-L1177)

qBittorrent 的运行路径也持续使用 `tr()` 生成实际业务文字。例如磁盘错误、下载完成、添加 torrent 成功/失败的
通知标题和正文都由 `tr(...)` 生成，再传给 `DesktopIntegration::showNotification`。
[application.cpp](https://github.com/qbittorrent/qBittorrent/blob/0b63c3d17373f6132ea211c9dcd4241284ccdfaf/src/app/application.cpp#L921-L946)

Qt 6.10.1 官方源码则闭合 `tr()` 到 `QCoreApplication::translate` 的框架链：`QMetaObject::tr` 直接调用
`QCoreApplication::translate(className(), source, comment, n)`。
[Qt `qmetaobject.cpp`](https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qmetaobject.cpp?h=v6.10.1#n418)

`QCoreApplication::translate` 持有完整 `context/sourceText/disambiguation/n`，按已安装 Translator 的顺序调用
`QTranslator::translate`，首个非 null 结果即成为返回值；没有 Translator 命中才从 UTF-8 `sourceText` 构造
`QString`。因此这是绘制前的完整字符串入口，而不是 glyph / vertex / texture 之后的反推。
[Qt `qcoreapplication.cpp`](https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v6.10.1#n2311)

## 相对 Qt Painter / Qt Quick 的增量覆盖

qBittorrent 的 Windows 系统通知是本轮最关键的增量证据。

应用先把已经过 `tr()` 的 `QString title/msg` 传给 `DesktopIntegration::showNotification`；Windows 非 DBus 分支进一步
调用 `QSystemTrayIcon::showMessage(title, msg, ...)`。
[desktopintegration.cpp](https://github.com/qbittorrent/qBittorrent/blob/0b63c3d17373f6132ea211c9dcd4241284ccdfaf/src/gui/desktopintegration.cpp#L180-L192)

Qt 6.10.1 Windows 平台实现 `QWindowsSystemTrayIcon::showMessage` 随后把 `message` 与 `title` 分别复制到
`NOTIFYICONDATA::szInfo` / `szInfoTitle`，最后直接调用 `Shell_NotifyIcon(NIM_MODIFY, &tnd)`。
[Qt `qwindowssystemtrayicon.cpp`](https://code.qt.io/cgit/qt/qtbase.git/tree/src/plugins/platforms/windows/qwindowssystemtrayicon.cpp?h=v6.10.1#n205)

因此该通知链的结构是：

`qBittorrent tr()` → `QCoreApplication::translate` / `QTranslator` → `QString` →
`QSystemTrayIcon::showMessage` → `NOTIFYICONDATA` → `Shell_NotifyIcon`

而不是：

`QString` → `QPainter::drawText`，也不是 `QQuickText` retained object。

这意味着至少对 qBittorrent 的系统通知，Translator/`translate` Adapter 可以在完整源字符串仍存在时做采集/替换，
而现有 `windows.qt.painter-draw-text` 与 `windows.qt.quick-text` 没有等价入口可重复完成这件事。这个增量价值已经
达到“值得进入原型”的门槛。

需要保持边界：本研究没有证明 qBittorrent 所有主窗口文字都只有 Translator 才能覆盖。大量 Widgets 文字最终仍
可能进入 Qt Painter；这些部分若现有 Adapter 已可命中，就属于重复覆盖，不应计入 Translator 的新增收益。
Translator 的生产价值应按真实增量区域统计，而不是按“所有 `tr()` 调用数量”统计。

## 动态 ABI 可验证性

qBittorrent 5.2.3 官方 Windows CI 明确安装 **Qt 6.10.1**，并在打包步骤复制 `Qt6Core.dll`、`Qt6Gui.dll`、
`Qt6Widgets.dll` 等动态运行库到应用目录。这为 Windows x64 动态 Qt 原型提供了可复现的真实软件 profile，而不是
仅有静态链接或 Demo 宿主。

- Qt 6.10.1 安装：<https://github.com/qbittorrent/qBittorrent/blob/0b63c3d17373f6132ea211c9dcd4241284ccdfaf/.github/workflows/ci_windows.yaml#L116-L122>
- 动态 Qt DLL 打包：<https://github.com/qbittorrent/qBittorrent/blob/0b63c3d17373f6132ea211c9dcd4241284ccdfaf/.github/workflows/ci_windows.yaml#L176-L185>

这只能证明“存在精确动态 ABI 目标”，不能把现有 Qt Painter / Quick 已验证版本范围自动外推到 Translator。
新的 Translator/`translate` Adapter 仍应把 Qt major、精确工具链/命名空间、位数、调用约定与必要导出作为独立
profile 验证，无法证明时 fail-closed。

## 主要 ABI 与生命周期风险

### 1. `QString` 返回值与 C++ ABI

`QCoreApplication::translate` 返回 `QString`，而自定义 `QTranslator` 又涉及 Qt C++ 对象、虚函数与析构生命周期。
无论选择 trampoline 还是安装自定义 Translator，都不能按“Qt 6 都一样”猜 ABI。首个真实 profile 应严格固定
qBittorrent 5.2.3 官方 Windows 构建对应的 Qt 6.10.1 / MSVC / x64 动态运行时，再决定可否扩展版本集合。

### 2. Translator 优先级会改变宿主本地化链

Qt 6.10.1 的 `installTranslator()` 把新 Translator `prepend` 到列表；`QCoreApplication::translate` 从列表头开始查询，
首个非 null 结果获胜。这正适合 Glyphshift 命中时覆盖，但也意味着未命中必须返回 null / 完整放行，不能把源文本
作为“未命中结果”返回，否则会截断 qBittorrent 自己的 Translator。
[Qt `installTranslator`](https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v6.10.1#n2194)
[Qt `translate`](https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v6.10.1#n2311)

### 3. `LanguageChange` 不等于现有 UI 自动刷新

Qt 在安装/移除有效 Translator 时提供 `LanguageChange` 机制，但已经创建的 Widgets 是否重新取文字，仍取决于宿主
对该事件的处理及是否执行 `retranslateUi()` / 重新设置属性。qBittorrent 本轮源码能证明启动期安装 Translator 和大量
`tr()` 查询，却不能据此证明“Glyphshift 运行中插入 Translator 后，所有已经显示的控件会立刻刷新”。

此外，Qt 6.10.1 当前实现会先把 Translator 加入列表，再检查 `translationFile->isEmpty()`；空 Translator 不会继续发送
`LanguageChange`。如果 Glyphshift 采用“无 `.qm`、只覆写 `translate()` 的自定义 Translator”，这一细节必须在合成
合同中单独验证，不能假设 install 本身一定触发全局重译。
[Qt `installTranslator`](https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v6.10.1#n2194)

因此首版验收必须把两类能力拆开：

- **调用时替换**：之后发生的 `tr()` / `translate()` 查询是否稳定取得完整 source 并返回 Glyphshift 译文；
- **已有 UI 刷新**：已经存在的 Widgets/QML 是否会因宿主重译逻辑更新像素。

前者通过不能自动宣告后者通过。

### 4. source / context / disambiguation / plural 不能被压扁

Qt translation seam 原生带 `context`、`sourceText`、`disambiguation` 与复数 `n`。Adapter 必须保留这些参数用于观测、
诊断、AI 语义提示和 plural fail-open，但 Glyphshift Dictionary 有意继续只按 source 唯一；因此当前不能表达同 source
在不同 Qt context 使用不同 Glyphshift 译文。`%n` / `%1` 等占位符仍由确定性合同覆盖，避免译文破坏目标自己的后续格式化。

## 建议的有界下一步

动态 Qt 原型阶段已经由 Wireshark 4.6.8 / Qt 6.10.3 完成，因此下一步不再重复寻找 observation / writeback
可行性，而是补共享语义边界：

1. 为 native text evidence / decision 增加版本化的 `context/disambiguation/n` 字段，使真实 hook 已取得的完整
   tuple 可以进入 capture、诊断与 Probe/AI 提示；未知版本继续 fail-closed。
2. Dictionary lookup 保持 source-only；禁止把 context/disambiguation 编码进 source 或扩展成只有 Qt 能填充的全局
   词典 key。plural `n`、`%n` / `%1` 等占位符继续由单独的确定性合同约束。
3. 保留现有 Qt 6.10.3 `QEvent::LanguageChange` 异步刷新合同，并为 Qt 5.15.18 建立等价安全方案；如果无法建立，
   明确将 Qt 5 profile 限定为 future-translate-call 生效，不宣称已有 Widgets 即时刷新。
4. 上述共享能力完成前保持 Adapter 不进入生产 Catalog / Bundle，并继续禁止 qBittorrent、Wireshark 或其他软件
   品牌、窗口标题、固定地址参与 profile 选择或刷新逻辑。

## GO / NO_GO 判定

**后续结果：GO 已完成 Qt 6.10.3 生产晋级；Qt 5 profile 继续受各自刷新 ABI 证据约束。**

理由：

- Wireshark 4.6.8 是合格的知名真实代表目标，实际 Windows 安装动态加载 Qt 6.10.3，且主程序直接导入目标 seam；
- Qt 官方源码证明该 seam 持有完整 source/context/disambiguation/n，并在绘制前返回 `QString`；
- Wireshark 原生窗口标题已证明现有 Qt Painter / Qt Quick / QTextDocument 无法重复覆盖的真实增量路径；
- 实机已完成非零 observation、首代 / 第二代译文、Qt LanguageChange 刷新和停用恢复；
- qBittorrent 的系统通知研究仍作为另一条结构性增量证据保留，但不再承担当前动态 ABI 实机 gate。

共享 ABI 后续已保留完整 Qt tuple；产品 Dictionary 仍有意 source-only，而不是把它当作待修缺陷。Qt 5.15.18 尚无与
Qt 6.10.3 等价的即时刷新合同。已有 UI 不响应 `LanguageChange` 本身不否定“调用时 translation seam”，但必须
作为 profile 生命周期能力明确记录，不能据此宣称所有 Qt Widgets 全界面热更新支持。
