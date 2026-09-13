# Silhouette 2026

## 当前决定

Silhouette 2026.0 继续复用 Qt Widgets / Qt Painter **通用框架适配器**，不新增 Silhouette 专属适配器。`isl0` 只是 `windows.qt.painter-draw-text` 内一个经过验证的精确 Qt ABI profile。主界面标准区域已经通过实机闭环，但节点图仍有已确认漏采；当前状态降回“部分支持”，直到新增的通用 QPainter 重载在真实节点图上完成验收。

## 根因

Silhouette 的主界面使用 Qt 6.5.4 Widgets。它的 Qt 导出符号不在标准全局命名空间，也不在此前支持的 `QT` 命名空间，而是在自定义 `isl0` 命名空间，例如 `QString@isl0`、`QWidget@isl0` 和 `QPainter@isl0`。旧 Qt Painter 解析器在命名空间检测阶段就会拒绝这套 ABI，因此不会安装 drawText Hook，表现为工作流运行但捕获 0 条文字。

Qt Painter 现在为这套已验证 ABI 提供精确符号映射，覆盖 QString 读取、QPainter drawText 入口、QApplication / QWidget 重绘辅助和 QArrayData 释放。实现保持 fail-closed：未知命名空间和未验证 ABI 不自动猜测。

另外，Silhouette 会绘制仅由空白字符组成的 QString。捕获层会拒绝这类源文本，旧实现因此把它记成一次 dropped observation。Qt Painter 现在在调用宿主前过滤 `trim().is_empty()` 的绘制文本，使这类非文字绘制不会进入捕获队列。

## 节点图漏采

用户实机截图确认节点图和部分面板仍有英文，例如 `Unproject`、`Right`、`Left`、`Time Shift`、节点 `Output`。当前运行记录中前四项没有观察；`Output` 虽已在其他界面区域由 Qt Painter 捕获并存在译文，但截图里的节点标签仍保持英文，因此同名文字来自另一条绘制调用。

静态导入检查确认 Silhouette 主程序直接使用 QGraphicsScene / QGraphicsView / QGraphicsTextItem，并导入了两个此前未被 Qt Painter Hook 的标准 QPainter 重载：`drawText(QPointF, QString)` 与 `drawText(x, y, w, h, flags, QString, QRect*)`。这两条是框架级通用 Qt API，不是软件专属入口。

Qt Painter 已新增这两个通用重载，且采用可选解析：目标 Qt 存在对应导出时才启用，新 Hook 缺失不会改变旧目标原有四条 drawText 路径的激活结果。包级回归保持 5/5 通过，最新同步 review build 已包含新 Runtime；当前真实 Silhouette 进程仍驻留旧 DLL，必须在目标重启后再做节点图最终验收。

## 实机验收

- 最新同步 Release review build 的 Runtime Bundle 校验通过，包含 13 个生产适配器。
- 在完全退出旧 Silhouette 进程后重新启动目标，启动工作流前确认没有旧 Glyphshift 模块驻留。
- 最新 Qt Painter DLL 成功进入 Silhouette 2026.0，并建立全新的采集记录。
- 最终干净记录捕获 56 条唯一文字，`droppedObservations=0`，唯一适配器来源为 `windows.qt.painter-draw-text`。
- 捕获样本包含 Workspace、Actions、Camera、Depth 等主界面文字；具体数量会随当前可见界面变化，因此 56 不是产品级固定上限。
- 在同一 `isl0` 适配路线的前序实机验收中，`File` 已完成第一代译文、第二代热更新，并在停止工作流后恢复原文。
- 最终验收停止后，工作流回到 ready；临时工作流、临时采集记录和测试前用户状态均已恢复。

## 验收边界

当前结论证明 Silhouette 2026.0 的 Qt Widgets 主界面标准区域可以被生产 Qt Painter 路线捕获并替换。节点图覆盖仍待新增 QPainter 重载实机确认；在该验收完成前，不再把 Silhouette 记为完整的“已验证可用”。它也不等于任意 Qt 自定义命名空间都受支持，不保证插件内嵌的第二套 Qt、OpenGL 自绘区域或未来 Silhouette 版本自动兼容。

## 恢复时第一步

先让 Silhouette 完全退出并重新启动，使最新 Qt Painter DLL 进入目标进程；随后打开包含节点图文字的同一界面，确认 `Unproject`、`Right`、`Left`、`Time Shift`、节点 `Output` 是否开始进入采集并可替换。若仍漏采，再继续沿 QGraphicsScene / QTextLayout 的通用完整字符串入口调查。
