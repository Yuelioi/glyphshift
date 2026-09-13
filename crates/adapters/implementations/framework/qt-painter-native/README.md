# Qt Painter drawText

支持 Windows x64 / MSVC ABI 的动态 Qt 5/6 Widgets。Qt Quick/QML 由独立的 Qt Quick Adapter 处理。

符号解析先从 QtCore 确定一套 ABI 命名空间，再一致地解析 QtGui 和 QtWidgets：

- 标准全局命名空间：保留原有 Qt 5/6 行为。
- `QT` C++ 命名空间：当前支持 Qt 6，覆盖 QString、四种 drawText 入口与 QWidget 重绘辅助接口。
- `isl0` C++ 命名空间：仅按 Silhouette 2026.0 / Qt 6.5.4 的已验证 MSVC x64 ABI 精确支持，不扩展成任意自定义命名空间匹配。

MSVC 名称中的类型回溯索引也随命名空间变化，因此使用明确的符号映射，不能靠字符串插入或混用两套符号。
未验证的 Qt 5 命名空间构建和未知映射拒绝解析。模块存在不等于激活或可见替换成功。

运行 `cargo test -p glyphshift-adapter-qt-painter-native --lib` 验证命名空间选择、标准 ABI 保持、Silhouette `isl0` 配置和未知配置拒绝。
真实目标验证使用本机环境提供的路径，原始文字、截图与日志不进入仓库。
