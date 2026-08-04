# GTK/Pango 实时写回候选评审

- 状态：完成
- 日期：2026-08-04
- 范围：GTK 3.24、GTK 4.20 与 Pango 的公开 C ABI、官方文档和官方源码；不以私有符号或软件专用
  签名作为通用能力。

## 结论

1. **动态 GTK 3 条件 Go。** 标准 `GtkLabel` 创建 `PangoLayout` 后通过公开
   `gtk_render_layout` 绘制，后者进入 `pango_cairo_show_layout`。最窄原型可在
   `gtk_render_layout` 处读取 Layout 原文、复制 Layout、只修改副本并调用原函数；目标原 Layout 不变，
   停用后下一次重绘自然恢复原文。
2. **GTK 4 当前 No-Go。** 标准标签进入 snapshot/GSK 路径，并通过 GTK 私有
   `gtk_snapshot_add_layout` 追加文字节点；公开 `gtk_snapshot_append_layout` 只是另一个调用该私有函数的
   包装，标准 `GtkLabel` 并不经过它。Hook 公开包装会漏掉主要场景，Hook 私有符号又不满足稳定 ABI 要求。
3. 直接 Hook `pango_layout_set_text` 虽能取得 UTF-8 原文，但会把译文写入目标持有的 Layout；停用后不能
   通用地立即恢复，且随后应用的 markup、助记符和链接属性仍按旧字节索引工作。因此不选它作为写回入口。
4. GTK 3 首版只承诺纯文本和全范围样式属性。任何依赖旧 UTF-8 byte index 的局部属性——包括 markup、
   mnemonic、link、selection 等——都必须放行原文，不能为了扩大命中而静默丢弃格式或重排属性。

## GTK 3 调用链

官方 GTK 3.24 源码可闭合常见按钮/标签路径：

```text
GtkButton / 其他容器
  -> GtkLabel
  -> gtk_widget_create_pango_layout(text)
  -> pango_layout_set_text
  -> gtk_render_layout(context, cairo, x, y, layout)
  -> pango_cairo_show_layout(cairo, layout)
```

`GtkLabel` 的绘制和 Layout 创建见官方 [`gtklabel.c`](https://github.com/GNOME/gtk/blob/3.24.49/gtk/gtklabel.c)，
`gtk_render_layout` 到 PangoCairo 的实现见 [`gtkrender.c`](https://github.com/GNOME/gtk/blob/3.24.49/gtk/gtkrender.c)。
Pango 官方也把 `PangoLayout` 定义为文字布局对象，并说明 `pango_cairo_show_layout` 负责把它绘制到 Cairo
surface。[Pango 渲染管线](https://docs.gtk.org/Pango/pango_rendering.html)、
[`pango_cairo_show_layout`](https://docs.gtk.org/PangoCairo/func.show_layout.html)

## 两种写回方案

### `pango_layout_set_text` 入参替换：否决

- 输入是调用方持有的 NUL 结尾 UTF-8，`length` 是字节数；Pango 会把文字设入 Layout。官方还明确提醒，
  `set_text` 不会清除此前由 markup 设置的属性。
  [`pango_layout_set_text`](https://docs.gtk.org/Pango/method.Layout.set_text.html)
- 该方案会改变目标对象状态。Dictionary 更新、停用和异常卸载后，已有 Layout 仍可能保存旧译文；要恢复就
  必须跟踪对象生命周期并回写目标状态，不符合现有绘制 Adapter 的 fail-open 合同。
- Pango 属性区间按 UTF-8 **byte index** 记录；源文与译文长度不同会让 mnemonic、链接、局部字体等范围失真。
  [Pango 文字属性与 markup](https://docs.gtk.org/Pango/pango_markup.html)

### `gtk_render_layout` 绘制时副本：进入原型

1. 用 `pango_layout_get_text` 借用原 Layout 的 UTF-8 原文；只做有界复制和精确 Dictionary 查询。返回指针
   归 Layout 所有，不释放、不修改。
   [`pango_layout_get_text`](https://docs.gtk.org/Pango/method.Layout.get_text.html)
2. `pango_layout_copy` 会按值深拷贝文字、属性列表和 tab array；只在副本上调用 `set_text`，原 Layout 和目标
   控件状态保持不变。绘制后对副本执行 `g_object_unref`。
   [`pango_layout_copy`](https://docs.gtk.org/Pango/method.Layout.copy.html)
3. 属性列表为空可替换；属性全部覆盖完整文本范围时可保留。发现任何有限字节区间、无法枚举的属性或
   markup/mnemonic/link 证据时放行原 Layout。
4. Dictionary 未命中、UTF-8 非法、模块/导出不匹配、复制失败、重入或异常时直接调用原函数。

该方案每次绘制都从目标原 Layout 重新读取原文，所以 Dictionary 新快照可在后续重绘生效；停用 Hook 后原函数
立即收到原 Layout，不需要遍历控件或恢复对象状态。

## GTK 4 边界

GTK 4 的 `GtkLabel` 通过 `gtk_css_style_snapshot_layout` 进入私有 snapshot 文字节点路径，标准控件并不调用
公开 `gtk_snapshot_append_layout`。官方 `GtkLabel` 源码见
[`gtklabel.c`](https://github.com/GNOME/gtk/blob/4.20.3/gtk/gtklabel.c)，私有入口声明见
[`gtksnapshotprivate.h`](https://github.com/GNOME/gtk/blob/4.20.3/gtk/gtksnapshotprivate.h)，公开包装合同见
[`gtk_snapshot_append_layout`](https://docs.gtk.org/gtk4/method.Snapshot.append_layout.html)。

因此首轮不能用 GTK 3 的证据宣称覆盖 GTK 4，也不进入私有 `gtk_snapshot_add_layout`、GSK 内部节点或签名扫描。
GTK 4 只有找到标准控件真实经过的稳定公开入口后才能重新评审。

## ABI 与能力探测

- 只接受目标进程已经动态加载、架构匹配且能解析全部所需导出的 GTK 3 / Pango / GObject 模块；静态链接、
  GTK major 不明或导出缺失均报告不兼容。
- Adapter 不硬编码安装目录或 DLL 落盘位置，只依据目标进程已加载模块与导出表。
- 首版不覆盖 GTK 2、GTK 4、自定义 PangoRenderer、直接 glyph 绘制、图像文字或未经过
  `gtk_render_layout` 的自绘控件。
- GTK/Pango 的 C ABI 比 C++ 框架入口更窄、更易校验，但模块名称和随应用部署方式仍须通过合成宿主与真实
  目标逐项确认。

## 使用场景与真实目标候选

该路径适合动态 GTK 3 桌面工具中的普通标签、对话框说明和未带局部格式的按钮文字。真实 smoke 应优先选择
免费、可重复安装且界面规模适中的 GTK 3 工具；候选可从 [Meld](https://meldmerge.org/)、
[Inkscape](https://inkscape.org/release/) 或 [GIMP](https://www.gimp.org/downloads/) 中按实际加载模块和
普通文本覆盖率选择一个。候选名称只用于授权测试选择，不构成产品支持声明。

## 准入合同

### 合成宿主

- 普通 `GtkLabel` 与普通按钮均命中 `gtk_render_layout`，取得 UTF-8 原文并得到可见译文。
- Dictionary 第一代、第二代在同一进程后续重绘中可见；停用后下一次重绘恢复原文。
- 空属性和全范围样式可替换；markup、mnemonic、link 与其他局部属性全部 fail-open。
- 覆盖空字符串、显式 byte length、长字符串、无效 UTF-8、未命中、重入、复制失败与模块缺失。
- 原 Layout 在替换前后 text、serial 与属性不发生变化；副本无泄漏、无双重释放。

### 真实目标

只有一个授权动态 GTK 3 目标同时出现入口命中、Dictionary 精确匹配、可见译文、同进程第二代更新和停用恢复，
才允许进入 Runtime Bundle。仅发现 Pango 模块、成功注入或观察到文字不算支持。

## 决策

建立 **GTK 3 Pango draw-time writeback** 合成切片；GTK 4 维持 No-Go。原型只使用公开
`gtk_render_layout`、Pango Layout API 与 GObject 生命周期函数，不扩张到私有 GTK 符号。
