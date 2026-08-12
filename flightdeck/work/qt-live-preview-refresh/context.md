# 稳定上下文

## 产品事实

- 探针译文编辑会先保存绑定 Dictionary，再为活动 Live Preview 构建并发布下一代完整 Snapshot；只有目标
  接受发布后，桌面侧才推进预览 generation。
- Target Runtime 在应用发布后会异步请求本进程顶层窗口及其子窗口重绘，避免在控制线程中同步重入目标
  绘制代码。
- Qt Painter Native Adapter 当前只拦截四个动态链接 Qt 5/6 MSVC x64 `QPainter::drawText` overload。
- 交互式 Qt 控件重新绘制时，现有 Adapter 能读取最新发布并替换文字；静态/缓存控件可能绕过这些入口。

## 诊断结论

- 隔离真实目标的静态文字复现中，首版与第二代目标译文均零命中，可见区域像素也零变化；运行时发布和
  通用窗口重绘仍成功。
- 同一隔离目标中的可编辑 placeholder 在主动交互重绘时，首版与第二代译文都能命中且可见像素变化。
- 额外主题刷新可触发大量 Qt Adapter 诊断，但目标静态文字依然零命中。因此问题不是“完全没有重绘”，
  而是静态/缓存文字没有重新经过当前可替换入口。
- Qt GUI 线程上的 Widget 树失效会让目标静态文字重新经过既有 `QPainter::drawText` hook，因此缺失能力是
  Framework 自有的刷新 seam，而不是已接受 Publication 的可见性或新的软件专属绘制分支。
- 修复不依赖目标具体使用 `QStaticText`、glyph run 或 pixmap cache，仍不把这些未证明候选写成产品事实。

## 实现决策

- Native Adapter ABI 提供无返回值的 best-effort `request_refresh`；不需要刷新语义的 Adapter 使用公共 no-op。
- Target Runtime 在激活、成功更新 Publication 与停用后调用刷新，并在调用前释放 Runtime 状态锁。
- Qt Adapter 从控制线程只使用 Win32 枚举 `Qt...` 原生顶层窗口，并按窗口线程与 generation 安装一次性
  `WH_GETMESSAGE` hook、投递标记消息；`QWidget::find`、Widget 枚举与同步 `repaint()` 只在对应 Qt GUI
  线程执行，快速连续请求由最新 generation 合并。
- Qt Widgets 模块或所需符号不存在时安全跳过，既有文字观察与替换能力继续工作。

## 约束

- Core、Desktop 与 GUI 不按 Adapter ID、框架名、软件品牌或可执行文件名分支。
- 刷新动作必须保持异步/可重入安全，不得从 Runtime 控制线程直接调用不具备线程安全保证的 Qt GUI 方法。
- 不改变原始用户目标进程；真实目标只通过 `local-test/` 下的隔离副本复验。
- 机器路径、用户名、PID、窗口标题、截图与原始日志不得进入跟踪文件。
- 不以隐藏/显示整窗、模拟用户输入或重启目标作为实时预览成功条件。

## 验收语义

- 活动探针编辑译文后，同一目标进程内的静态/缓存 Qt 文字无需重启即可显示第二代译文。
- 停用或断开后继续满足既有恢复语义；不支持的 Qt 路径安全跳过，不破坏目标绘制。
- 通用 Runtime/Adapter 合同覆盖发布后的刷新行为；真实隔离目标提供本机可见像素回归证据。
- 受影响的 Rust 合同、格式与静态检查通过；只有桌面 GUI 发生变化时才要求 Playwright 定向回归。
