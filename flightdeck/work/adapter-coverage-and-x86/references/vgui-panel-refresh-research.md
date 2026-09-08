# VGUI 存量控件刷新研究

日期：2026-09-08。范围：Valve 官方公开源码能证明的控件语义，以及在授权目标核实后才能实施的刷新路线。SDK 不是当前游戏的二进制 ABI 规格，下面不提供可直接套用的虚表槽位、对象偏移或机器证据。

## 已确认的语义

`TextImage` 将本地化结果复制进自己的缓冲，绘制时使用副本。因此仅替换后续查询返回值，无法更新已经构造好的控件。Unicode setter 会按需重新分配、复制文本并使截断/换行计算失效；直接覆盖字符内存会绕过这些维护。[TextImage.cpp](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/TextImage.cpp)

标准 `Label` 注册了 `SetText` 消息；`text` 字段支持窄字符串和宽字符串。窄字符串可以重新提交本地化 token，宽字符串直接提交译文。`Label::SetText` 同时维护热键、布局和重绘，是比只调用底层文字图像 setter 更完整的更新路径。[Label.h](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui_controls/Label.h)、[Label.cpp](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/Label.cpp)

`IPanel` 提供树遍历、消息、信息请求以及重绘接口，但不提供 `GetSettings`。它还明确要求 `GetPanel` 检查目标模块，因为不同模块的 C++ `Panel` 虚表可能不同。因此“拿到公开 IPanel”不等于可以跨模块按 SDK 槽位调用 Label 方法。[IPanel.h](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui/IPanel.h)

## 读取原文的陷阱

`Label::GetSettings` 的 `labelText` 可以保留未本地化 token；特殊变量前缀会被转换。它是 C++ 方法，不能假设存在同名通用消息。`RequestInfo(GetText)` 返回当前显示文字，而非天然原文；标准实现还把固定缓冲传给按字节计长的读取函数，长文本不能视为可靠、无截断输出。[Label.cpp](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/Label.cpp)、[TextImage.cpp](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/TextImage.cpp)

普通 `Panel::RequestInfo` 在本地不能处理时会向父级递归。因此返回成功不能证明文字属于被查询的子控件；遍历时必须同时证明该控件自身支持文字协议，不能仅凭成功返回值写回。[Panel.cpp](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/Panel.cpp)

Unicode `TextImage::SetText` 默认保留本地化符号。它提供 `GetUnlocalizedText` / `GetUnlocalizedTextSymbol`，但这些也是需要核实对象与 ABI 的 C++ 接口。设计上应保存原文身份和最后一次写入值，不能每次把当前屏幕上的译文再次当作原文。[TextImage.h](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui_controls/TextImage.h)

## 调度、发现和生命周期

`IVGui` 声明了 `RunFrame`、`PostMessage`、tick 和 `PanelToHandle/HandleToPanel`；头文件把 HPanel 定义为安全句柄，但没有承诺任意外部线程可以调用这些操作。推荐刷新回调只递增待处理代次；在已经核实的游戏 UI 帧入口处理控件，再执行原帧流程或在已验证的边界执行。不能从 Runtime 工作线程直接枚举和写控件，也不能自行从其他线程调用 `RunFrame`。[IVGui.h](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui/IVGui.h)

生命周期缓存宜保存安全句柄，每次使用前在同一 UI 线程重新解析；公开 `PHandle` / `VPanelHandle` 也是这个模式。原始地址、类名和控件名都不足以抵御销毁后地址复用。[PHandle.h](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui_controls/PHandle.h)

存量发现可从 `ISurface::GetEmbeddedPanel` 获取根，通过 IPanel 子树遍历。弹出层可能需要额外根；源码虽有 popup 枚举，但标注不应绕过其他 VGUI 对象直接使用。因此覆盖范围要以实际根树核实，不声称“根树遍历必然包含所有菜单”。限制节点数、深度及每帧工作量属于本项目的实施约束。[ISurface.h](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui/ISurface.h)

`Panel::PostMessageToChild` 在找不到目标时自行销毁消息；成功路径交给 VGUI。由此可知投递路径涉及所有权转移，不能借用栈内存、投递后立即释放或重复释放；同步 SendMessage 的所有权也必须在实际实现中单独核实。[Panel.cpp](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/Panel.cpp)

## KeyValues 是消息方案的前置门槛

KeyValues 不是普通键值序列化结构：它含符号名称、类型、字符串、子节点及链指针。不能用 Rust 字段猜测或把另一个 SDK 的 `sizeof` 当成目标布局。符号必须属于目标使用的符号系统。[KeyValues.h](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/tier1/KeyValues.h)

`IKeyValuesSystem` 提供共享符号表和对象池，并专门注册对象尺寸以支持版本差异。但对象池接口不足以证明字符串存储兼容。[IKeyValuesSystem.h](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vstdlib/IKeyValuesSystem.h)

官方实现的对象 new/delete 走 KeyValuesSystem；字符串 setter 还会释放旧字符串并分配新字符串。最小可接受做法是使用已核实的目标构造、setter、销毁方法以及其分配器；仅获得 `AllocKeyValuesMemory` 后手工填字段，无法解决字符串析构和版本布局。[KeyValues.cpp](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/tier1/KeyValues.cpp)

## 路线比较与推荐

| 路线 | 收益 | 必须补齐的证明 |
| --- | --- | --- |
| IPanel + 标准 SetText 消息 | 利用控件自己的布局、热键和重绘；不依赖 Label 私有字段 | IPanel ABI、控件确实处理该消息、KeyValues ABI/所有权、原文身份、UI 线程 |
| 已核实的 Label setter | 完整控件更新，可避开 KeyValues 消息构造 | 模块内对象类型、setter ABI、安全句柄到对象关系、原文身份 |
| 已核实的 TextImage setter | 接近完整文本 owner，可保持 token | 对象发现/寿命、getter/setter ABI、外层布局/热键更新；不等于完整 Label 刷新 |
| 仅 Repaint 或反复查询 | 实施简单 | 不改变已有副本，不能满足本次诉求 |

以上比较是根据前述官方接口和实现做出的工程判断，不是对当前游戏兼容性的结论。

推荐先证明标准消息方案，在无法可靠构造目标 KeyValues 时改走目标自身已验证的 Label setter；TextImage setter 是次选。首批只纳入可证明身份、完整原文、当前值且不含动态占位符的短标签。不要为赶进度恢复 glyph 重放或修改本地化原表。

推荐的最小执行状态机：

1. 在授权目标证明根、句柄、UI 帧入口及至少一种完整 setter 路径；先用只读发现确认待更新控件。
2. 记录安全句柄、原文/token、最后写入值、字典代次。在同一 UI 线程比较当前值；只有仍是预期原文或本适配器写入值才更新。
3. 字典换代只合并最新请求，在后续 UI 帧批量更新；重复回调不重复分配或覆盖原文。
4. 游戏自行改字时，撤销旧翻译所有权并重新识别来源。动态变量控件必须另行验证，不持续强压覆盖游戏的新状态。
5. 停用先禁止新翻译，再请求 UI 线程恢复仍归适配器所有的存活控件，等待恢复确认后结束会话。UI 不运行或超时不能报告恢复成功，也不能跨线程强写；必须保留可完成恢复的驻留状态。

这些是本项目的实现/验收建议，尚不代表已完成代码。验收保持同一菜单不关闭：原文→译文一→译文二→删除词条恢复→重新启用→停用恢复；另验收控件销毁、地址复用、自行改字、窗口关闭和无 UI 帧超时。

## 当前必须实测的未知

- 目标工厂是否暴露对应接口，其版本、实际方法与 SDK 是否一致。
- 主菜单/设置控件是否继承标准 Label，还是自定义文字 owner；是否有不可从根到达的弹出树。
- 实际 GetText 长度与终止行为，token 获取是否无损；不能用 SDK 的固定缓冲读取去试探任意长文本。
- 目标 KeyValues 构造/销毁是否可定位，调用方与被调用方是否共享布局、符号和分配器。
- UI 帧 hook 是否覆盖真实绘制线程；停用等待是否会与 Runtime/host 决策锁互等。
- 游戏自行重建和动态变量更新的时序，字体大小、热键及布局是否随 setter 正确更新。

上述未知均应通过目标能力核实和合成负例约束解决；没有证据时拒绝相应刷新能力，不能把“找到同名函数/字符串”视为 ABI 证明。
