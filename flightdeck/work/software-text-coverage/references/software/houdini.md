# Houdini

最新验收：用户在最新同步 review App 上手动测试后确认当前界面已完全汉化，并授权提交、打 tag 与推送。
本轮界面覆盖通过用户验收；这不等于新增第二代更新 / 停用恢复的独立证据，下面的缓存生命周期边界继续保留。

当前决定：Houdini 22.0.429 **部分支持**，组合使用 `windows.qt.painter-draw-text`、通用 `windows.qt.text-document`，并新增通用 `windows.sidefx.cv-paint-buffer-text` 候选。Qt Painter 继续覆盖 shelf 等即时绘制文字；Qt Text Document 覆盖经 `QTextDocument::setHtml` 建模的纯文本，以及只有一个可见文字节点的保守富 HTML；SideFX CV 适配器覆盖动态 `libCV.dll` 的 `CV_PaintBuffer::textWrappedInBox` 完整 UTF-8 文字入口。三条能力都按框架 / 渲染接口建模，没有新增 Houdini 专属 Adapter。Houdini 的 shelf 与富 help 卡片仍存在下游 retained cache：Runtime 若在首次生成前激活，可以产生可见替换；缓存建立后，普通 Qt / Windows 重绘不足以保证字典热更新或停止恢复立即反映到像素。

## 已验证事实

- 目标使用 Qt 6.8.3 MSVC x64、标准全局 Qt 命名空间；现有 Qt Painter profile 可以解析并安装。
- `Quick Shapes`、`Helix`、`Spiral` 的完整字符串真实经过现有 Adapter 已覆盖的 `QPainter::drawText(QPointF, QString)` 和 `QPainter::drawText(QPointF, QString, from, length)`。因此这三个标签不需要 `QTextLayout`、`QTextItem`、`QStaticText` 或新的 QPainter 重载才能进入 Glyphshift。
- Qt6Gui 另外导出的 `QPainter::drawText(QPoint, QString)` 与 `QPainter::drawText(x, y, QString)` 也做过定向实机采样；前者没有调用，后者虽有调用但没有上述 shelf 标签。不能通过补这两个重载解决当前问题。
- 在已经稳定运行并建立 shelf 缓存的单一目标进程中，生产 Runtime 能激活 Qt Painter 且持续产生其他诊断记录，但 `Quick Shapes` 替换保持 0；排除了“挂错第二个 Houdini 实例”与“Adapter 整体没有工作”。
- 在目标启动早期激活同一生产 Runtime 后，首代 `Quick Shapes -> 快速形状` 得到 2 次真实 replacement hit，且 shelf 像素可见为中文。该实验没有增加任何 Houdini 专属代码。
- 同一会话发布第二代译文后，目标标签不再进入相关 `drawText`，第二代 replacement hit 为 0；正常停止 Runtime 后，已经生成的 shelf 仍保持第一代译文，直到 Houdini 自己重建该界面或目标重启。
- 针对已建立的界面，Windows redraw/theme/font/settings/syscolor 通知，以及 Qt `ApplicationFontChange`、`LanguageChange`、`FontChange`、`StyleChange`、`updateGeometry()`、`ensurePolished()`、`setUpdatesEnabled(false -> true)` 加 repaint 都没有让上述标签重新进入目标 `drawText`。
- 首次生成 `Quick Shapes` 的调用栈在 QPainter 上游进入 Houdini 自有 Pluto/UI 延迟绘制链，包含 `UI_QtPlutoUtils::drawShelfButton`、`UI_QtPlutoStyle::drawShelfButton` 与 `OPUI_ToolbarItem::ohHandleDeferredUpdate`。这解释了为什么标准 QWidget repaint 不会失效 shelf 的 retained cache；这些符号只作为根因证据，不作为产品 Hook 入口。
- 实机计时中 Qt 模块在启动早期已经可用，而 `Quick Shapes` 的首次 shelf 生成明显更晚；提前激活生产 Runtime 因而存在可用窗口。桌面端对已启用但尚未找到目标的 Workflow 会周期性重试，所以当前推荐顺序是 **先启用 Workflow，再启动 Houdini**，让 Qt Painter 在 shelf 首次生成前进入目标。
- 悬停提示正文已经建立独立实机红例。可见提示 `The currently active Desktop` 出现时，标准 `QToolTip::showText` 没有命中，现有 QPainter / `QTextLayout::draw` / `QStaticText` 等字符串入口也拿不到完整正文；完整字符串进入的是 Qt 6.8.3 的 `QTextDocument::setHtml`。
- 针对该标准 seam 新增了通用 `windows.qt.text-document` retained-object Adapter。当前接受纯文本，或保留原 HTML / CSS 结构、只有一个可见 literal 文本节点的保守富 HTML；译文写回前做 HTML 转义。多可见文本段、实体、控制空白或结构不确定的 HTML 继续 fail-open。ABI 精确限制为已实机验证的 Qt 6.8.3 MSVC x64、全局 Qt namespace，未知版本 fail-closed。
- 纯文本 `The currently active Desktop` 的 production real-host 已从旧 Qt Painter 的目标 0 hit 变为 Qt Text Document 首代 2 hit / 第二代 1 hit，并完成可见中文与停用后英文原文验证。
- 扩样本后，Tube 的 `Creates open or closed tubes, cones, or pyramids.` 与 Spiral 的 `Creates spirals and helices.` 都确认以“CSS + 单个 `<p>` 正文”的富 HTML 进入同一个 `QTextDocument::setHtml`。最新 production Runtime 在两者首次生成时都得到非零 replacement hit，并分别可见中文正文，原样保留 help 卡片布局与样式。
- Tube 的第二代 publication 仍得到 replacement hit，但可见卡片继续显示第一代译文；停止 Runtime 后再次 hover 也继续显示译文。定向生命周期探针确认首次 hover 只有一次 `QTextDocument::setHtml`，隐藏 / 再次 hover 不 clone、不析构、也不再次 setHtml；对 tooltip 顶层 Qt 窗口执行通用 Windows `RedrawWindow` 仍不能刷新正文。这说明该富 help 卡片在 `QTextDocument` 之后还有下游显示缓存，不能把“代次命中”记成“像素热更新通过”。
- Adapter 仍按 retained-object 合同记录原文与最后一次自身写入、监听 document 析构，并把 publication 更新 / 停用恢复调度回 Qt GUI thread；恢复只在当前文档内容仍等于 Glyphshift 最后一次写入时执行，应用自己的后续改写优先。GUI refresh 请求若超时会主动移除临时线程 hook，避免留下悬挂回调。
- Network Editor 右下角菜单的 `Add`、`Edit`、`Go`、`View`、`Tools`、`Layout`、`Help` 已确认继续走现有通用 Qt Painter 的 `QPainter::drawText(QRectF, int, QString const&, QRectF*)`，真实传入值只是带有首尾布局空格，例如 `" Add "`。这不是新的 Houdini 绘制 API。
- Capture 使用 `SourceTextPolicy::key()` 聚合时会去除首尾空白，而原生 Runtime 此前只按原始 / normalize 后文本做实时查找，导致集合中的 `Add -> 添加` 能正常显示和累计，实际 `" Add "` 绘制却不命中。现在 Runtime 在精确查找失败后统一使用同一 source-policy key 回退，并把调用者原有首尾空白补回译文；带空白的精确词典键仍保持最高优先级。修复位于通用 target Runtime，不在 Qt Painter 或 Houdini 内增加特判。
- 该行为已有确定性回归：`" Add "` 能命中 `Add` 并输出 `" 添加 "`，同时 `" Exact "` 若存在精确带空格词条则不会被无空格词条覆盖。最新生产 Runtime 在真实 Network Editor 中对七个菜单项都记录了“带空格原文 no-match -> 规范 key matched/replaced”的连续决策，诊断 0 丢弃，屏幕像素同步显示中文。
- Network Editor 剩余的 `Non-Commercial Edition`、`Objects` 与组合正文 `Empty Network\nPress Tab to Add Nodes` 已确认最终汇入动态 `libCV.dll` 的 `CV_PaintBuffer::textWrappedInBox`。改变窗口尺寸能稳定触发该绘制入口重新进入，证明这是实际可见绘制链的一部分，而不是只用于测量的旁路。
- 一次性动态替换实验已在该入口把 `Non-Commercial Edition -> 非商业版本`、`Objects -> 对象`、`Empty Network\nPress Tab to Add Nodes -> 空网络\n按 Tab 添加节点` 真实画成中文，并保留目标自己的布局 / 字体路径；因此当前没有证据要求为这条 seam 新增字体兜底。
- 新增通用 `windows.sidefx.cv-paint-buffer-text`：Windows x64 only，要求目标动态加载 `libCV.dll` 且存在已验证的精确 MSVC `textWrappedInBox` 导出；缺模块 / 缺导出 fail-closed。Adapter 只瞬时替换完整 C 字符串参数，不引用 Houdini 进程名、软件 ID、OPUI / Pluto 私有符号。
- SideFX CV 已有确定性 ABI fixture：首代替换、同进程第二代更新、停用恢复都通过；缺少 `libCV.dll` 的激活拒绝合同也通过。活动包全仓验证通过，最新同步 Runtime Bundle 已验证包含 16 个 Adapter。
- Houdini 测试 Workflow 已把 `windows.sidefx.cv-paint-buffer-text` 加入 adapter plan。仍运行旧驻留集合的目标在重新启用时正确返回 `runtime.target_restart_required`，证明没有为了实机测试放宽驻留集合保护。production Adapter 的真实目标首代像素、第二代更新与停用恢复仍需在 Houdini 正常重启后完成最终闭环。

## 当前边界

### 更新恢复后的实机进度

已通过 `scripts/review-app.ps1` 再次同步构建桌面壳和 Runtime Bundle，production loader 验证 16 个 Adapter。
正常重启目标后 revision 6 Workflow 激活成功，Runtime 无错误，旧驻留集合的重启要求已解除。
SideFX CV 已真实采集 `Non-Commercial Edition`、`Objects`、`Empty Network\nPress Tab to Add Nodes`，
采集丢弃为 0；前两项首代中文像素已观察到。中心组合提示原先是待翻译，已写入
`空网络\n按 Tab 添加节点`，但未发生新一轮 CV 绘制，像素仍是英文。不能把这次 publication 或 Qt 重绘
当作 CV 热更新通过。随后用户接管手动测试并确认当前界面完全汉化；第二代和停用恢复不再自动续测。
原生窗口截图与 Playwright 诊断保留在本地证据目录。
等待期间全适配器 diagnostics 缓冲出现 71 条丢弃，已关闭 diagnostics；后续每轮重绘前重新开启并及时读取，
不能把这个等待窗口作为零丢弃验收。该诊断缓冲与上述采集目录的 droppedObservations 是不同计数。
目标启动需要本机正确的 HFS / SHFS 环境；已保存本地启动入口，不修改系统环境或产品配置。

当前不能把 Houdini 记为完整“已验证可用”：

- 已经建立的 shelf retained cache 不接受通用 Qt 刷新，因此运行中的字典热更新不能保证立刻改变这些标签。
- 停止 Runtime 可以正常完成，但同一 retained cache 不会立刻重画原文；需要 Houdini 自己重建对应界面或重启目标才能看到恢复。
- Qt Text Document 当前只承诺已验证的 Qt 6.8.3 x64 `setHtml`，以及纯文本 / 单可见文本节点 HTML 模板；复杂或多段富 HTML、其他 Qt 版本以及不经过 `QTextDocument` 的自绘提示不从本轮结果外推。
- 富 help 卡片的首代可见替换通过，但第二代只验证到 replacement 决策命中，像素仍停留在第一代；停用后的文档恢复请求可以执行，但卡片像素仍受下游缓存阻断。需要 Houdini 自己重建帮助卡片缓存或重启目标后才会重新生成可见内容。
- Network Editor 菜单栏的首尾布局空白已经通过通用 Runtime source-policy 回退解决；SideFX CV production Adapter 的界面覆盖已获用户手动验收通过，第二代更新 / 停用恢复仍缺独立实机验收，不能外推为完整生命周期支持。
- 当前规则禁止为了刷新 shelf 去 Hook Houdini 自有 Pluto / OPUI 符号。只有找到能跨软件复用的 retained-text 失效机制，才进入通用实现。

## 推荐使用方式

需要翻译 Houdini 主界面时，同时启用 Qt Painter、Qt Text Document 与 SideFX CV PaintBuffer，并优先在启动 Houdini、首次打开对应 help 之前启用 Workflow。首屏 shelf 和富 help 卡片都已经证明首次生成阶段可以产生可见替换。纯文本 Qt Text Document 提示可完成运行期更新 / 停用恢复；富 help 卡片和 shelf 若保持旧译文，重新建立对应界面或重启 Houdini。Network Editor 的 SideFX CV overlay 需要目标自身重画；当前已验证改变 Network Editor 尺寸可触发重新绘制。

## 恢复时先做

Network Editor 菜单栏已经证明不是缺少 Qt Painter 重载，而是 Capture key 与实时决策 key 的通用一致性问题，并已在 target Runtime 修复。revision 6 Workflow 已在重启后的目标成功加载 `windows.sidefx.cv-paint-buffer-text`，前两项首代中文像素已通过。继续时重新检查目标与 diagnostics 状态，通过改变 Network Editor 尺寸触发 overlay 重画，验证 `Non-Commercial Edition`、`Objects`、`Empty Network\nPress Tab to Add Nodes` 的 capture、matched/replaced、首代中文像素、第二代译文与停用恢复。只有这轮 production real-host 闭环完成后，才更新 Network Editor 的产品支持结论。富 help 的后续工作仍只允许研究可跨 Qt 软件复用的 `QTextDocument` 下游 cache 失效 seam；标准 Windows redraw 已证实无效，不能从 Houdini 自有 Help / Pluto / OPUI 分支实现专属刷新。
