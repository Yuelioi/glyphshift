# Autodesk Alias 原位文字支持：第一方资料核验

核验日期：2026-09-07。范围：Alias 2027 与 Qt 6.8；仅资料研究，未向 Alias 写入文字，未运行 UIA 或修改生产代码。

## 结论

建议先验证 **Qt Quick 对象层的只读标签**，再依据真实覆盖率选择 QTranslator 翻译层。Qt 官方已提供可读写文本属性、场景树和重译入口，但这些是进程内 API；还需要适配器取得真实 QObject、正确线程和版本兼容的桥接。官方资料不能证明 Alias 的所有 UI 都使用这些入口，也不能仅由已加载 Qt DLL 推导“已经支持”。

本地只读调查提供的前提：目标为 Alias Concept 2027，加载 Qt 6.8.3 Core/Gui/Qml/Quick/Templates2/Widgets；应用模块直接导入 QQmlApplicationEngine 构造及 load、QQuickItem/QQuickWindow，并存在 QML 资源标记及 QPainter::drawText 导入。它们支持继续调查 Quick 路线，但绘制导入不证明其对应任何具体可见文字。原始证据保留在本地测试区域，不作为仓库必需资源。

## 已证实的 Qt 入口及限制

| 入口 | 官方证据 | 对适配器的含义 |
| --- | --- | --- |
| QQuickText / QML Text | Qt 6.8.3 声明 QString text 的 READ/WRITE/NOTIFY，setter 为 setText，另有 textFormat、elide、contentWidth 等属性。[源码](https://raw.githubusercontent.com/qt/qtdeclarative/v6.8.3/src/quick/items/qquicktext_p.h) | 适合优先调查静态标签；写入后必须检查布局、富文本和截断。类头文件属于 Qt 私有实现接口。 |
| QQuickTextInput / QML TextInput | 同样有可写 text，同时包含 readOnly、validator、inputMask、echoMode、cursorPosition 和 selection 等状态。[源码](https://raw.githubusercontent.com/qt/qtdeclarative/v6.8.3/src/quick/items/qquicktextinput_p.h) | text 是输入内容，不能当作普通标签批量替换；首阶段排除其 text。 |
| QQuickTextEdit / QML TextEdit | 可写 text，包含 readOnly、textDocument、富文本格式和编辑状态。[源码](https://raw.githubusercontent.com/qt/qtdeclarative/v6.8.3/src/quick/items/qquicktextedit_p.h) | 可能承载用户文档，即使 readOnly 也不自动代表 UI 文案；首阶段排除其 text。 |
| 元对象属性 | QQmlProperty 支持检查属性有效性、类型、可写性和读取；QML 对象使用 Qt 元类型系统。[文档](https://doc.qt.io/qt-6.8/qqmlproperty.html) | 优先通过 QObject/QMetaProperty 检查属性，避免首版硬编码私有类布局或 vtable 偏移。 |
| 场景树 | QQuickWindow::contentItem 是场景根；QQuickItem::childItems 返回子项目。[窗口](https://doc.qt.io/qt-6.8/qquickwindow.html#contentItem-prop)、[项目](https://doc.qt.io/qt-6.8/qquickitem.html#childItems) | 在真实窗口中遍历视觉对象；结合 QObject 所有权树做去重。不要把“可枚举 QObject 数”当成可见标签覆盖率。 |

**写回与绑定：** Qt 明确说明 QObject::setProperty 保留 QML binding，后续依赖更新可能覆盖写入；QQmlProperty::write 则移除 binding。直接 setter 与任意外部属性修改也可能触发应用信号处理。因而“恢复原字符串”不能保证恢复已移除的 binding 或消除业务副作用。首个可写实验应仅选择已识别为 UI 标签的白名单对象，保留宿主 binding，记录原值及最后写值，在对象存活且当前值仍等于适配器写值时才考虑恢复；避免与宿主更新反复争抢。[Qt C++/QML 交互说明](https://doc.qt.io/qt-6.8/qtqml-cppintegration-interactqmlfromcpp.html)

上述回滚、白名单和防竞争措施是工程建议，尚未在 Alias 实测。对象由宿主持有，适配器不应重设 parent 或销毁它们；需要追踪 destroyed/生命周期并在 GUI 对象所属线程工作。进程内桥接加载方式、编译器/Qt ABI 和实际可见节点覆盖率仍是实现门槛。

## QTranslator 与重译路径

QTranslator 可以加载 qm，或覆写 translate(context, sourceText, disambiguation, n) 查询自有词典；后安装的 translator 优先。未命中应返回 null QString 让其他翻译器继续查找。翻译键必须考虑上下文、消歧及复数，不能仅按英语原文全局替换。[QTranslator](https://doc.qt.io/qt-6.8/qtranslator.html)

安装/卸载 translator 会产生 LanguageChange；Widgets 是否即时更新还依赖宿主的事件处理及重新设置文字。它不是自动扫描任意现有字符串的机制。[QCoreApplication](https://doc.qt.io/qt-6.8/qcoreapplication.html#installTranslator)

QQmlEngine::retranslate 只重新计算使用了翻译标记字符串的 binding；官方建议在 installTranslator 后调用。qmlEngine(object) 可取得关联引擎；6.6 起另有 markCurrentFunctionAsTranslationBinding，供自定义翻译函数参与重译。[QQmlEngine](https://doc.qt.io/qt-6.8/qqmlengine.html#retranslate)

qsTr / qsTranslate / qsTrId 是对应 QML 翻译入口，普通 text 字符串赋值不因安装 translator 自动变成翻译调用。[QML Qt 类型](https://doc.qt.io/qt-6.8/qml-qtqml-qt.html#qsTr-method)

**对 Alias 的推断：** 已存在 QQmlApplicationEngine 是有利线索，但不能证明内置文案经过上述函数。静态文件中没有找到 qsTr ASCII 字样也不能证明没有翻译绑定，QML 可能经过编译。后续应测量真实翻译调用及 context 覆盖，并对已有窗口、新开面板、动态菜单分别验证。无需为了试翻译清除 component cache 或重载整套应用 QML。

## Autodesk SDK 能否直接修改内置 UI

不能笼统说“Alias SDK 不支持改文字”：Autodesk 的 Alias 2026 更新明确新增 AlEditor::setTitle、setPopupItems，以及 AlFunctionHandle::setToolTitle。[2026 API 更新](https://help.autodesk.com/cloudhelp/2026/ENU/Alias-WhatsNew/files/wn-whatsnewinalias2026/wn-alias-api-updates-2026.html)

可访问的 AlFunctionHandle 正式类文档说明，它负责将插件接入菜单；create 的标签属于该插件，setToolTitle 显示于工具提示和列表菜单。文档没有给出枚举全部内置工具句柄并重命名的通用入口。[AlFunctionHandle 2026](https://help.autodesk.com/view/ALIAS/2026/ENU/?guid=GUID-6081C689-954F-495D-B3FB-051B18381D53)

因此已证实的边界是“SDK 可控制它管理的工具和编辑器文字”；**未证实**可遍历、更改整套内置 UI 文案或取得全部 QML 对象。2027 API 更新没有公布通用 UI 本地化接口，不能把未公布当作绝对不存在。[2027 API 更新](https://help.autodesk.com/cloudhelp/2027/ENU/Alias-WhatsNew/files/wn-whatsnewinalias2027/2027-Updates-and-Enhancements/wn-alias-api-updates-2027.html)

资料限制：2027 部分类参考地址返回未找到或抓取失败，故具体工具接口引用可读取的 2026 正式文档及 2027 更新，不声称已逐项确认 2027 SDK 二进制。后续可用目标版本 SDK 头文件补足。

## 最小验证顺序（建议，尚未执行）

1. 只读统计实际 Quick 可见标签：类型链、可写字符串属性、可见性和尺寸；同时区分纯标签、可编辑输入、文档数据。机器证据仅保留在 local-test/evidence/。
2. 先在合成 Qt 6.8 场景验证：静态 Text、带 binding 的 Text、销毁重建、动态菜单、回滚、长中文；明确 QObject 写入保留绑定的行为。
3. 对 Alias 仅选择少量明确的只读 UI 标签试验原位替换，并核对无业务数据变更、无重复翻译、面板重开和卸载回滚。TextInput/TextEdit 的内容不纳入第一阶段。
4. 独立测量 QTranslator 命中覆盖。若多数文案已经走翻译调用，以 translator + retranslate 为主；其余使用受控属性适配。若只覆盖部分区域，按区域报告支持范围，不上报全软件支持。

本研究没有证明 painter 路径适合原位文本替换，也不建议以绘制 hook 替代上述第一轮对象层验证。
