# Qt translation-service：context / plural / placeholder / LanguageChange 语义边界

Status: Complete

本文只核对 Qt 第一方资料，并把结论分为 **官方事实**、**推导**、**待验证 / 当前实现选择**。目标版本固定为：

- Qt 5.15.18 LTS LGPL tag `v5.15.18-lts-lgpl`，commit `49adb85d34918034e0d6a4c23817407103fb9f73`。
- Qt 6.10.3 tag `v6.10.3`，commit `7ddbc87d8e14ce51d2957ea72d0a6077593d5ff4`。
- 附加 Windows ABI 研究样本：QGIS LTR 3.44.12 随包 Qt 5.15.13 / MSVC x64。

结论先行：Qt 自己的 `QCoreApplication::translate(context, sourceText, disambiguation, n)` 查找语义不是 source-only；
`context/sourceText/disambiguation` 决定 Qt translator 查找，`n` 决定 numerus form，并由
`QCoreApplication::translate()` 在返回前展开 `%n/%Ln`；`%1..%99/%L1..%L99` 则仍属于调用方后续
`QString::arg()`。Glyphshift 的产品 Dictionary 则明确保持 source-only：context/disambiguation 作为观测证据和 AI
语义提示保留，但不进入词典唯一键，因此同 source 的非复数调用共享一条译文；需要 context-specific 译文时当前
模型无法表达，这是有意接受的能力边界。Qt 5.15.18 的框架源码本身已经证明“向 application 异步 `postEvent(LanguageChange)`”是 Qt
自己采用的合法机制，且 Widgets 的传播路径与 Qt 6.10.3 同类；但这并不等于 Glyphshift 当前就有一个经过证明的
Qt 5 外部注入事件分配 / 所有权 ABI。没有这层证据前，Qt 5 profile 继续 **future-call-only**。

## 1. `QCoreApplication::translate()` 四个参数

### 官方事实

Qt 5.15.18 与 Qt 6.10.3 的核心路径一致：`QCoreApplication::translate()` 按 translator 列表顺序逐个调用
`QTranslator::translate(context, sourceText, disambiguation, n)`，遇到首个非-null `QString` 即停止；都没有命中时，
回退为 UTF-8 `sourceText`，随后执行 `%n/%Ln` 后处理。

- Qt 5.15.18：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v5.15.18-lts-lgpl>
- Qt 6.10.3：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v6.10.3>
- `QTranslator` 的查找语义：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qtranslator.cpp?h=v5.15.18-lts-lgpl>
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qtranslator.cpp?h=v6.10.3>

四个参数分别承担：

| 参数 | Qt 官方语义 | 对 Glyphshift 的含义 |
| --- | --- | --- |
| `context` | 查找上下文，通常是调用 `tr()` 的类名；`QTranslatorPrivate::do_translate()` 将 null 规范化为空字符串 | evidence 必须保留，可作为 Probe/AI 语义提示；普通 Dictionary 不以它分词条 |
| `sourceText` | 原文，是 lookup 的主体；若所有 translator 均未命中，`QCoreApplication::translate()` 返回它的 UTF-8 `QString` | 作为 Glyphshift Dictionary 的唯一词条身份；这不等于复制 Qt `.qm` 的完整 identity |
| `disambiguation` | 同 context、同 source 下区分不同语义；`QTranslator::translate()` 文档明确把它作为 key 的一部分；若精确项未找到，还会尝试空 disambiguation | evidence 必须保留，可作为翻译提示；普通 Dictionary 不以它分词条 |
| `n` | 当 `n >= 0` 时，`QTranslatorPrivate::do_translate()` 用 `.qm` 内 numerus rules 选择 plural form；公开 API 的非复数哨兵是 `-1` | 它是 plural 选择输入，不应简单拼进“每个数字一个词条”的 key |

`QTranslator` 文档还明确说明：精确 `(context, sourceText, disambiguation)` 没找到时，会继续尝试
`(context, sourceText, "")`；不完整翻译文件甚至可能产生与 disambiguation 预期不同的结果。这个 fallback 是 Qt 的
`.qm` 查找行为，不应被理解为 Glyphshift 可以忽略 disambiguation。

Qt 6.10 官方国际化文档同样把 context、source、comment/disambiguation 与 numerus 分开描述：
<https://doc.qt.io/qt-6.10/i18n-source-translation.html>

### 推导与产品选择

如果 Glyphshift 要完整模拟 Qt `.qm` 的查找 identity，就至少需要 `sourceText + context + disambiguation`；当前产品
不选择这条路线。普通 Dictionary 继续以 source 唯一，避免只有单个 Adapter 能填充的框架字段扩散到全局词典格式。
因此 context/disambiguation 留在 evidence / Probe / AI hint，最终译文仍按 source 合并。

`n` 同样不进入普通 Dictionary key，因为 Qt 的 plural 不是按“观察到某个数字就得到一个独立词条”建模，
而是用目标语言的 numerus rule 从一个 plural message 选择 form。若 Dictionary 还没有 plural-form 模型，`n >= 0`
的调用应保留证据但 fail-open，不应用某次观察到的译文覆盖所有 `n`。

公开 API 把 `-1` 定义为非 numerus；源码实际条件是 `n >= 0`。因此其他负值在这两个版本的实现里也不会选择
plural form，但不应把这种实现细节扩成 Glyphshift 的公共契约。最安全的生产规则是：`-1` 或 `>= 0` 有定义；
`< -1` 只记录 / 放行，不替换。

## 2. `%n`、`%Ln`、`%1`、`%L1` 的所有权和替换时机

### 官方事实

Qt 5.15.18 与 Qt 6.10.3 的 `QCoreApplication::translate()` 都在 translator lookup 完成、并完成 source fallback 后，
调用内部 `replacePercentN()`：

- `%n`：由 `QCoreApplication::translate()` 在返回前替换为 `n`。
- `%Ln`：同样由 `QCoreApplication::translate()` 在返回前替换，但通过本地化数字格式处理。
- `%1` … `%99`：`translate()` 不展开，保留给调用方之后的 `QString::arg()`。
- `%L1` … `%L99`：也是 `QString::arg()` 层的参数占位符；`L` 表示本地化数字格式，不属于 numerus form 选择。

对应源码：

- Qt 5.15.18 `replacePercentN()` / `QCoreApplication::translate()`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v5.15.18-lts-lgpl>
- Qt 6.10.3：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v6.10.3>
- `QString::arg()`：
  <https://doc.qt.io/qt-6.10/qstring.html#arg>
- Qt 国际化文档对 `%n/%Ln` 与 `%1` 参数的说明：
  <https://doc.qt.io/qt-6.10/i18n-source-translation.html>

### 推导：Glyphshift 应保留什么

Glyphshift 返回翻译文本时，不应抢走 Qt / 调用方拥有的格式化步骤：

1. 保留 `%n`、`%Ln`，让 `QCoreApplication::translate()` 按本次 `n` 展开。
2. 保留 `%1..%99`、`%L1..%L99`，让目标代码后续 `QString::arg()` 展开。
3. 可以改变占位符顺序，这是翻译正常需要；不能静默删除、增加或改变占位符类型。
4. 对生产 fail-closed 合同，建议比较占位符 **token 多重集合**：顺序可不同，但 `%n` 与 `%Ln`、`%1` 与
   `%L1` 应视为不同 token。任何丢失、额外或类型变化都放行原始 Qt 路径。
5. 普通 `%` 不应被 Glyphshift 自行“清理”或格式化。

如果实现方式仍是“把 Glyphshift 译文作为替代 `sourceText`，然后调用原始 `QCoreApplication::translate()`”，那么
`%n/%Ln` 仍由 Qt 原函数负责。若未来改成“Glyphshift 直接构造最终 `QString` 并跳过原函数”，则必须自己等价执行
Qt 的 `%n/%Ln` 后处理，同时仍然不能展开 `%1/%L1`。

### 待验证

`%Ln` 的具体数字外观依赖目标进程当时的 locale；语义已由源码确定，但真实软件像素格式仍可作为 profile 验收项。

## 3. `QEvent::LanguageChange`、Widgets / Quick 与 Qt 5.15.18 refresh

### 3.1 官方语义

`QEvent::LanguageChange` 的官方定义是“应用翻译发生变化”。

- Qt 5.15.18 `QEvent`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreevent.cpp?h=v5.15.18-lts-lgpl>
- Qt 6.10.3：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreevent.cpp?h=v6.10.3>

正常安装 / 移除非空 translator 时，两版 `QCoreApplication` 都向 application 自身同步发送
`LanguageChange`。一个实现差异是“空 translator”：

- Qt 5.15.18 在通常构建条件下把 translator prepend 后，`isEmpty()` 会使 `installTranslator()` 返回 `false`，且不发
  `LanguageChange`。
- Qt 6.10.3 对空 translator 返回 `true`，同样不发 `LanguageChange`。

所以“installTranslator 必然发 LanguageChange”不是无条件事实。

更关键的是，两版 `QTranslatorPrivate::clear()` 在 translator 已安装时都直接使用：

`QCoreApplication::postEvent(QCoreApplication::instance(), new QEvent(QEvent::LanguageChange))`

这证明 **Qt 5.15.18 官方源码本身就使用“异步 post LanguageChange 给 application”这一机制**：

- Qt 5.15.18：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qtranslator.cpp?h=v5.15.18-lts-lgpl>
- Qt 6.10.3：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qtranslator.cpp?h=v6.10.3>

`QCoreApplication::postEvent()` 文档 / 源码还明确要求 event 必须在 heap 上分配；队列接管所有权，并在投递后删除；
该函数是 thread-safe：

<https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v5.15.18-lts-lgpl>

### 3.2 Qt Widgets

两版的传播链在本研究关心的语义上等价：

1. application 收到 `LanguageChange`；
2. `QGuiApplication::event()` 向 top-level windows `postEvent(LanguageChange)`；
3. `QApplication::event()` 还补充没有 window handle 的 top-level widgets；
4. `QWidget::event()` 在收到 `LanguageChange` 后调用 `changeEvent()`，继续向 children 传播，并 `update()`。

第一方源码：

- Qt 5.15.18 `QGuiApplication`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/gui/kernel/qguiapplication.cpp?h=v5.15.18-lts-lgpl>
- Qt 5.15.18 `QApplication`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/widgets/kernel/qapplication.cpp?h=v5.15.18-lts-lgpl>
- Qt 5.15.18 `QWidget`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/widgets/kernel/qwidget.cpp?h=v5.15.18-lts-lgpl>
- Qt 6.10.3 对应文件：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/gui/kernel/qguiapplication.cpp?h=v6.10.3>
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/widgets/kernel/qapplication.cpp?h=v6.10.3>
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/widgets/kernel/qwidget.cpp?h=v6.10.3>

Qt 官方动态翻译方式仍要求应用自己的 widget 在 `changeEvent(LanguageChange)` 中重新设置用户可见文本；Qt Designer
生成的 UI 可调用 `retranslateUi()`。`LanguageChange` 提供刷新信号与传播，不会神奇地重建应用私有缓存。

### 3.3 Qt Quick / QML

Qt 5.15.18 与 Qt 6.10.3 都提供 `QQmlEngine::retranslate()`。两版 `QQmlEngine::event()` 在自己收到
`LanguageChange` 时都会调用 `retranslate()`。

- Qt 5.15.18：
  <https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/qml/qml/qqmlengine.cpp?h=v5.15.18-lts-lgpl>
- Qt 6.10.3：
  <https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/qml/qml/qqmlengine.cpp?h=v6.10.3>
- 官方 API：
  <https://doc.qt.io/qt-6.10/qqmlengine.html#retranslate>

但第一方源码并不能据此证明“只给 application post 一次 LanguageChange，就必然让任意现存 QQmlEngine 收到事件”。
对 Quick，官方稳定入口仍是 `QQmlEngine::retranslate()` / `uiLanguage` 等机制；如果 Glyphshift 只向 application post，
是否覆盖目标应用的 Quick engine 必须由目标生命周期实测证明。

### 3.4 Qt 5.15.18 能否像 Qt 6.10.3 一样异步 refresh？

**官方事实：语义上可以。** Qt 5.15.18 自己就在 `QTranslatorPrivate::clear()` 中 heap-alloc 一个
`QEvent(LanguageChange)` 并异步 post 给 application；`QGuiApplication` / `QApplication` / `QWidget` 的传播代码也明确
存在。因此不能把 Qt 5 标记为“框架不支持 application-level LanguageChange refresh”。

**当前实现选择：Glyphshift 仍维持 Qt 5.15.18 future-call-only。** 原因不是 Qt 5 缺少 LanguageChange 语义，而是
外部原生 Adapter 还没有足够第一方 / ABI 证据证明自己能安全构造一个由 Qt event queue 接管并最终 `delete` 的 Qt 5
heap `QEvent`。Qt 6 的 `QEvent` 有公开虚 `clone()`，可由目标 Qt 自己产生 queue-owned 副本；Qt 5.15.18 的 `QEvent`
头文件没有这条 `clone()` API：

- Qt 5.15.18 `qcoreevent.h`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreevent.h?h=v5.15.18-lts-lgpl>
- Qt 6.10.3 `qcoreevent.h`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreevent.h?h=v6.10.3>

缺少 `clone()` 并不证明 Qt 5 无法 refresh；它只意味着当前 Qt 6 使用的“由 Qt 自己 clone 出 heap 对象再 post”办法
不能直接宣称对 Qt 5 安全等价。自行猜测 `QEvent` 大小、allocator、CRT `new/delete` 配对或私有 ABI 都不符合
fail-closed 门槛。

### 待实机验证 / 晋级门槛

Qt 5 immediate refresh 至少要补齐以下证据后再开启：

1. **事件所有权 ABI**：用精确 Qt 5.15.18 / MSVC x64 的可证明入口创建 queue-owned heap `QEvent`，其分配与 Qt
   最终 `delete` 配对；不得靠 guessed layout / allocator。
2. **application 投递**：异步事件能稳定到达 application，目标退出 / teardown 时不崩溃、不泄漏、不 double free。
3. **Widgets 行为**：至少一个已创建 UI 在 `LanguageChange` 后实际再次执行翻译查询并更新可见文本；若宿主未实现
   `changeEvent/retranslateUi`，明确标记 refresh-limited。
4. **Quick 行为**：若目标是 QML/Quick，单独证明目标 `QQmlEngine` 被重译；不能只凭 application event 推断。
5. **生命周期闭环**：首代、第二代、停用恢复都通过；否则只能宣称 future calls 会使用新决策。

### 3.5 Qt 5.15.13 / MSVC x64：queue-owned `QEvent` ABI 已在研究 profile 闭环

QGIS LTR 3.44.12 的 Windows Qt5Core 真实二进制给出了 Qt 5.15.13 的独立 ABI 证据。该 DLL 导出
`QEvent(Type)`、虚析构、`QCoreApplication::postEvent` 与 `qMalloc`；同时从 UCRT heap API 导入
`malloc/free/calloc/realloc`。对实际代码与虚表继续复核后：

- `qMalloc` 直接跳到 Qt5Core 自己 IAT 中的 UCRT `malloc`；
- `qFree` 直接跳到同一 IAT 中的 UCRT `free`；
- `QEvent` 虚表的 scalar deleting destructor 在执行 `QEvent::~QEvent()` 后，删除路径最终也跳到同一个 UCRT
  `free`；
- 因而可以由目标 Qt5Core 的 `qMalloc` 分配 event storage、调用目标 Qt5Core 的 `QEvent(Type)` 构造函数，再把
  指针交给 `QCoreApplication::postEvent`。队列之后按 Qt 自己的 deleting-destructor 路径回收，分配与释放在同一
  Qt DLL / CRT heap 配对内完成。

这条结论只对已经核验的 **Qt 5.15.13 MSVC x64** profile 生效，并且只在显式 `research-qt5` 构建中开放；不能外推
到 Qt 5.15.18 或任意 Qt5。默认 production 仍只接受 Qt 6.10.3。

本地真实 Qt 5.15.13 生命周期红例进一步验证了行为：adapter 激活后，新的 `translate()` 调用已经能返回译文，
但启动前缓存到菜单 action / tooltip 的字符串保持英文；旧 `request_refresh()` 不产生 `LanguageChange`。加入上述
queue-owned event 路径后，同一复现收到一次 `LanguageChange`，根菜单、二级菜单、叶子 action 与 tooltip 同时重新
执行翻译并更新。这个结果证明原先的多级菜单 / tooltip 缺口首先是刷新生命周期问题，而不是层级本身不受
`QCoreApplication::translate` 支持。

## 4. 共享 native text evidence / decision ABI：最小、版本化、fail-closed 建议

### 推导：最小 V2 形状

保持现有 source text V1 前缀不变，在其后追加一个明确版本的 translation metadata block；不要把元数据编码进
`sourceText`。建议最小字段为：

```text
NativeTextEventV2
  base: NativeTextEventV1       # 完整 V1 前缀
  metadata_version: u32         # 首版固定 1
  metadata_flags: u32           # context/disambiguation/plural-n presence
  context_ptr / context_len     # UTF-16，bounded
  disambiguation_ptr / len      # UTF-16，bounded
  plural_n: i32
  reserved: u32                 # 必须为 0
```

最小合同：

- `base.struct_size` 明确指向 V2 实际大小；host 只在大小、版本、flags 都认识时解析扩展。
- `metadata_version != 1`、未知 flag、非零 reserved、pointer/length 不一致、超限、非法 UTF-16：**不得生成替换决策**；
  调用原 Qt 路径。可以记录有界诊断，但不能降级为 source-only replacement。
- `context` / `disambiguation` 必须有 presence flag，以便 evidence 区分“参数缺席”和“存在但为空”；普通 Dictionary
  不消费它们作为 key，但 Probe / 诊断 / AI 仍可使用这些元数据。
- `plural_n` 只在有明确 plural metadata 时参与语义；生产 replacement 接受 `-1` 或 `>=0`。异常 `< -1` 只观察 / 放行。
- context / disambiguation 使用固定上限，例如每项 1024 UTF-16 code units；超限 fail-open 到原函数，不做截断后匹配。

### Decision / Dictionary 规则

1. **普通调用**：Dictionary lookup 按 source；context/disambiguation 只保留为 evidence / AI hint，不建立同 source 多词条。
2. **plural 调用**：在有显式 plural-form Dictionary 模型之前，`n >= 0` 保留 evidence，但 replacement fail-open；不能把
   某个 `n` 的结果当作整条 message 的通用译文。
3. **placeholder gate**：替换前比较 `%n/%Ln/%1..%99/%L1..%L99` 的 token 多重集合；允许重排，任何缺失、增加或
   token 类型变化均拒绝 replacement。
4. **旧 host / 旧 ABI**：source-only V1 可以服务 `n == -1` 的 contextual / disambiguated 调用，因为当前 Dictionary 本来
   就只按 source；plural 与未知负 `n` 仍必须 observe / pass-through。
5. **未知版本**：fail-closed 指“拒绝 Glyphshift replacement、继续调用目标原始 Qt 行为”，而不是让目标应用失败。

这套边界使共享 ABI 保持小而通用：它表达的是“框架翻译调用身份 + plural 输入”，没有 Qt 软件品牌、窗口标题、
固定二进制定位、控件类型或 profile 私有字段；后续其他框架若有同类语义也可以复用一个版本化 metadata kind，而无需污染 source。

## 最终决定

- `context/sourceText/disambiguation/n` 作为 Qt translation-call evidence 完整保留；普通 Glyphshift Dictionary 仍为
  source-only，context/disambiguation 不进入词典 schema、唯一键或 Workflow publication identity。
- 非复数 contextual / disambiguated 调用共享 source 译文；context/disambiguation 可进入 Probe 展示和 AI 提示。
- Glyphshift 译文保留 Qt placeholder；`%n/%Ln` 由 translation 层处理，`%1/%L1` 等留给调用方 `QString::arg()`。
- Qt 5.15.18 **框架语义上支持**向 application 异步 post `LanguageChange` 并传播到 Widgets；这已经是官方源码事实。
- Qt 5.15.18 **Glyphshift profile 仍 future-call-only**：当前缺口是外部 Adapter 的 queue-owned `QEvent` 创建 / 所有权 ABI
  与真实目标生命周期证据，而不是 LanguageChange 语义本身。
- Qt 5.15.13 / MSVC x64 已在 `research-qt5` 下完成独立 queue-owned `QEvent` 分配 / 删除 ABI 与真实 Qt Widgets
  缓存刷新复现；它现在可异步 post `LanguageChange`。这仍是研究 profile，尚需 QGIS 真实界面的二/三级菜单、
  tooltip、第二代更新与停用恢复复验后才讨论生产晋级。
- shared native text ABI 采用 V1 prefix + bounded V2 metadata；未知版本、plural 模型缺失或 placeholder 不兼容时
  一律调用原 Qt 路径。旧 source-only decision 对 `n == -1` 仍可使用普通 source 译文。

## 第一方资料索引

- Qt 5.15.18 `qcoreapplication.cpp`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v5.15.18-lts-lgpl>
- Qt 6.10.3 `qcoreapplication.cpp`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreapplication.cpp?h=v6.10.3>
- Qt 5.15.18 `qtranslator.cpp`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qtranslator.cpp?h=v5.15.18-lts-lgpl>
- Qt 6.10.3 `qtranslator.cpp`：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qtranslator.cpp?h=v6.10.3>
- Qt 5.15.18 / 6.10.3 Widgets 传播：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/widgets/kernel/qapplication.cpp?h=v5.15.18-lts-lgpl>
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/widgets/kernel/qwidget.cpp?h=v5.15.18-lts-lgpl>
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/widgets/kernel/qapplication.cpp?h=v6.10.3>
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/widgets/kernel/qwidget.cpp?h=v6.10.3>
- Qt 5.15.18 / 6.10.3 GUI application 传播：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/gui/kernel/qguiapplication.cpp?h=v5.15.18-lts-lgpl>
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/gui/kernel/qguiapplication.cpp?h=v6.10.3>
- Qt 5.15.18 / 6.10.3 `QEvent` API：
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreevent.h?h=v5.15.18-lts-lgpl>
  <https://code.qt.io/cgit/qt/qtbase.git/tree/src/corelib/kernel/qcoreevent.h?h=v6.10.3>
- Qt 5.15.18 / 6.10.3 QML engine retranslate：
  <https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/qml/qml/qqmlengine.cpp?h=v5.15.18-lts-lgpl>
  <https://code.qt.io/cgit/qt/qtdeclarative.git/tree/src/qml/qml/qqmlengine.cpp?h=v6.10.3>
- Qt 6.10 国际化 / placeholder 官方文档：
  <https://doc.qt.io/qt-6.10/i18n-source-translation.html>
  <https://doc.qt.io/qt-6.10/qstring.html#arg>
  <https://doc.qt.io/qt-6.10/qqmlengine.html#retranslate>
