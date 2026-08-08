# Qt Painter 实时写回 Adapter

Status: Complete

## Outcome

为动态链接、受支持 ABI 的 Qt Widgets 软件提供一个有界实时写回 Adapter：在
`QPainter::drawText` 的常见绘制入口取得 `QString` 原文，查询当前 Dictionary 快照，命中后只把
临时 `QString` 交给本次原绘制调用。更新 Dictionary 后随下一次重绘使用新译文，停用后原调用立即
恢复，不修改控件保存的文字。

## 为什么首版不是 QTranslator

Qt 的 `QCoreApplication::translate` / `QTranslator` 能取得更干净的源文和语境，适合长期建设；但
已经显示的 Widgets 是否刷新，依赖目标收到 `LanguageChange` 后主动执行重译。GlyphShift 当前更
需要“修改字典后立即看效果”，因此首版选择绘制时调用。Translator 保留为后续 retained/localization
Adapter，不与这一 slice 混合。

## Delivery

- [x] 用第一方资料比较 Qt、WinUI/MRT、WPF 与 Web 桌面壳入口。
- [x] 核对 Qt 5/6 MSVC x64 动态库常见 `drawText` 导出与转发关系，收敛需要覆盖的核心 overload。
- [x] 建立不依赖 Qt SDK 的纯逻辑合同：UTF-16 解码、长度上限、字典命中、失败开放、回调重入与
  原调用恰好一次。
- [x] 在同一 native package 内分离 Qt 5 与 Qt 6 ABI 分支；仅在目标已经加载对应 Qt Core/Gui
  动态模块时解析受支持导出，不主动把 Qt 加载进非 Qt 进程；同时加载两个 major 时安全拒绝。
- [x] 使用本机显式配置的合成宿主验证观察、替换、Dictionary 热更新、停用恢复和不兼容诊断；所有
  路径、日志与截图只进入 `local-test/`。
- [x] 合成合同通过后再选授权 Qt Widgets 目标做可见 smoke，通过后加入正式 Bundle 与目录。

## 首版边界

- Windows x86_64、MSVC ABI、动态 release Qt；Qt 5 与 Qt 6 分开识别和构建。
- 覆盖 `QPainter::drawText` 的点文本、矩形 flags 文本与 `QTextOption` 常见入口。
- 不覆盖 QML scene graph、`QStaticText`、`QTextDocument` 富文本、glyph-only 绘制、静态 Qt、MinGW
  或跨 major 猜测 ABI。
- 首版只做文字观察与替换，不混入字体替换、Translator 安装、控件遍历或区域路由。

## 验收

1. 合成 Qt Widgets 宿主绘制已知 Unicode 原文；Adapter 能观察该原文，Dictionary 命中后像素输出
   等价于宿主直接绘制译文。
2. 同一进程内发布第二代 Dictionary 后，下一次重绘使用第二代译文；无需重启目标或重装 Hook。
3. 停用后下一次重绘恢复原文；未命中、无效 UTF-16、过长文字、回调异常和重入都原样调用一次。
4. 缺少 Qt 模块、major/位数/工具链不受支持或任一必要导出缺失时，激活明确失败且目标继续运行。
5. 授权真实目标必须同时出现入口命中、Dictionary 匹配、可见替换与停用恢复；只有注入成功不算
   支持。

## Evidence

- 纯逻辑合同 6/6：点文本、矩形 flags、矩形 option、observe-only、无效/过长/重入/失败开放和
  replacement UTF-16 均通过。
- 既有 Native Host 常规合同 8/8，加上“目标未加载 Qt 时激活失败且不加载 Qt”1/1 通过。
- 显式配置的 Qt 5 与 Qt 6 MSVC x64 本地运行时各完成 1/1 像素合同，并逐一穿过点文本、整数矩形、
  浮点矩形和 `QTextOption` 4 个核心 overload；每个入口的原文、第一代译文、第二代译文和停用恢复
  图像签名均符合预期。机器路径与原始日志只位于本地证据目录。
- 首个授权真实 Qt 6 x64 目标在注入前完成能力检查：没有动态 Qt DLL 导入、进程内 Qt Core/Gui/
  Widgets 模块或可用的 Qt 文字导出，属于静态链接构建。按首版边界记录为不兼容，未执行注入，也
  未增加品牌或版本专用签名。
- 动态链接 Qt 6 Widgets 真实目标的菜单绘制完成首版译文、第二代 Dictionary 热更新与停用恢复的
  可见验收；首版和第二代各记录 2 次 `Matched + Replaced`。Qt 菜单助记符属于实际源文的一部分，
  验收按该原文精确匹配，没有在 Dictionary 层增加特殊规则。
- 正式 Runtime Bundle 构建加入 Qt Painter 包并校验内容哈希；正式 Bundle 的干净目标复测捕获 40 次
  绘制、命中替换 2 次，停用后重新打开菜单恢复原文。完整仓库测试通过。

## References

- [下一 Adapter 第一方资料评审](../references/next-adapter-primary-source-review.md)
- [Qt QPainter](https://doc.qt.io/qt-6/qpainter.html)
- [Qt QString](https://doc.qt.io/qt-6/qstring.html)
- [Qt QStyle](https://doc.qt.io/qt-6/qstyle.html)
- [Qt QStaticText](https://doc.qt.io/qt-6/qstatictext.html)
- [Qt 二进制兼容边界](https://doc.qt.io/qt-6/qt-releases.html#compatibility-promises)
