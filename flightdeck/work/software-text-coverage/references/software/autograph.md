# Autograph

当前决定：Autograph 2026 已通过现有 `windows.qt.quick-text` **通用 Qt Quick 适配器**完成生产 Runtime 实机验收，没有新增 Autograph 专属适配器。适配器现在精确接受已验证的 Qt 6.8.3 和 6.11.1 MSVC x64 ABI；不要把这一结论放宽为任意 Qt 6.x。

## 已验证事实

- 修复前真实工作流使用 13 个生产适配器运行后为 0 条观察、0 条丢弃；根因是 Qt Quick 原生适配器在解析阶段硬性拒绝目标的 Qt 6.11.1。
- 有窗口的实际目标进程是 `Autograph-bin`；启动器进程不承担主界面窗口。目标同时加载 Qt 6.11.1 Core、Gui、Qml、Quick、Quick Controls 2 和 Widgets。
- Qt Quick 适配器改为精确版本集合，只允许 Qt 6.8.3 与 6.11.1；其他 Qt 6.x 仍拒绝激活。
- 适配器当前依赖的 17 个 Qt Core / Gui / Quick MSVC 导出在 Autograph 的 Qt 6.11.1 中全部存在，包括 `QGuiApplication::allWindows`、`QQuickWindow::contentItem`、`QQuickItem::childItems`、`QQuickText::text/setText`、QString 读写和 QArrayData 释放入口。
- 使用与生产适配器相同的窗口枚举、QQuickItem 树遍历和 `QQuickText::text()` getter 做只读实机探针，可从当前主窗口枚举 3038 个 QQuickItem、230 个非空 QQuickText 标签；218 个满足现有内容过滤规则，当前可见且符合规则的有 54 个。
- 读到的完整界面原文包括 File、Edit、Help、Properties、Inspector、Open Project、New Composition、Timeline 等，证明这不是字形级或纹理级入口，而是可直接用于字典决策的完整字符串层。
- 当前可见标签绝大多数使用适配器已经接受的 `textFormat = 2`；因此 0 条不是现有富文本过滤规则造成的。
- Qt 6.11.1 retained-object 合同在显式重建原生 DLL 后完整通过，覆盖首次替换、第二代刷新、动态标签、对象销毁与停用恢复。这个集成测试会动态加载 target 目录里的 `cdylib`，运行合同前必须先显式 `cargo build -p glyphshift-adapter-qt-quick-native`，否则可能误用旧 DLL 得到假失败。
- 最新同步 Runtime Bundle 在真实 Autograph 上捕获 47 条唯一文字，`droppedObservations = 0`；47 条全部由 `windows.qt.quick-text` 产生。稳定条目包括 File、Edit、Help、Properties、Timeline、Viewer、Render Manager 等。
- 实机将 `File` 第一代替换为中文后，目标 QQuickText 对象立即返回译文；第二代译文再次热更新成功，原始 `File` 在运行中不再存在。停止工作流后译文消失，原始可见 `File` 恢复，工作流回到 ready，最终仍为 47 条观察、0 条丢弃。
- 验收用临时译文已清空；保留真实采集自动写入的原文条目。生产边界仍是 QQuickText 标准标签，TextInput、TextEdit、富文本和自绘文字不纳入这一支持声明。

## 当前判断

Autograph 与 ZBrush 的情况不同。ZBrush 需要继续向自有 glyph / 纹理缓存上游寻找字符串入口；Autograph 已经通过稳定的 Qt Quick 完整字符串入口完成生产验收，因此继续复用通用 Qt Quick 适配器，不需要 Autograph 专属适配器。

## 下一步

Autograph 2026 当前验收闭环已完成。后续只在新的 Autograph / Qt 版本出现时重新跑 ABI 导出检查、retained-object 合同和实机闭环；不要因为同属 Qt 6 就自动加入白名单。
