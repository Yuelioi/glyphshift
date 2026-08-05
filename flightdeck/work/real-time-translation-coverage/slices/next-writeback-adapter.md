# 下一种实时写回 Adapter 选择

Status: Complete

## Outcome

选出一个能显著扩展 Windows 桌面软件实时翻译覆盖的文字路径，并固定合成与授权真实目标的可见验收；
不把只读采集、成功注入或架构完整度当作选择理由。

## Delivery

- [x] 盘点正式 Runtime Bundle 与现有原型的观察、写回和真实目标证据。
- [x] 对仍未覆盖的主要 Windows 文字路径列出可拦截入口、原文取得方式、写回方式和停用恢复风险。
- [x] 选择一个具有真实目标、跨软件复用价值和有界实现范围的候选；明确否决其他候选的当前理由。
- [x] 固定首个合成合同与授权真实目标 smoke，不包含本机路径或真实原文。

## Current

当前生产写回集中在 GDI、USER32 DrawText 与 GDI+。Direct2D `DrawText` 原型可以在合成 render target
写回，但授权目标没有命中；Console 与 UIA 只能采集。已知的大类缺口包括 DirectWrite/Direct2D 的
retained text、WPF/WinUI、Chromium/CEF/Electron、Qt 混合后端和 GPU 画布，但框架名称本身不能证明
具体文字路径。

DirectWrite 原型随后覆盖了 `CreateTextLayout` 与两种绘制入口：Direct2D `DrawTextLayout` 以及
TextLayout 自带的 `Draw`。合成合同全部通过，复杂格式安全放行；当前授权目标的真实测试运行成功但
两种入口合计仍为零命中，WPF 授权目标也没有命中。该结果否决了当前生产接入；实验实现已撤回，只
保留本页结论，未来只有先取得真实命中证据时才重新实现。

## 支持矩阵

| 文字路径 | 生产状态 | 合成写回 | 授权真实目标 | 当前等级 |
| --- | --- | --- | --- | --- |
| `ExtTextOutW` | Bundle | 已验证，含停用恢复 | 已观察并用于部分界面写回 | 已验证实时翻译 |
| `TextOutW` | Bundle | 已验证，含停用恢复 | 尚无独立证据 | 候选实时翻译 |
| `DrawTextW/DrawTextExW` | Bundle | 已验证，含停用恢复 | 尚无独立证据 | 候选实时翻译 |
| `GdipDrawString` | Bundle | 已验证，含停用恢复 | 已观察并用于部分界面写回 | 已验证实时翻译 |
| Qt `QPainter::drawText` | Bundle | Qt 5/6 均已验证，含热更新与停用恢复 | 动态 Qt 6 Widgets 可见写回；静态 Qt 安全拒绝 | 已验证实时翻译 |
| GTK 3 `gtk_render_layout` / Pango | Bundle | 已验证，含热更新、样式放行与停用恢复 | 正式注入链取得两代 Dictionary 命中 | 已验证实时翻译 |
| Direct2D `DrawText` | 实验 | 已验证，含停用恢复 | 测试执行成功但零命中 | 被当前目标否决 |
| DirectWrite TextLayout | Bundle | 直接与 Compatible Bitmap 绘制均已验证；复杂格式放行 | Scintilla 编辑区两代热更新与停用恢复可见；WPF 零命中 | 已验证实时翻译（不含 WPF） |
| `WriteConsoleW` | Bundle | 不支持写回 | 已验证采集 | 仅采集原文 |
| UI Automation | Bundle | 不支持写回 | 已验证采集与隐私边界 | 仅采集原文 |

## 候选判断

- **已验证并否决当前目标的 DirectWrite TextLayout 路径。** `CreateTextLayout` 能取得原文与基础格式，
  `DrawTextLayout` 在每次绘制时接收已格式化 layout。原型先把两者关联，在绘制时生成临时译文 layout；
  停用后立即传回原 layout，不修改目标保存的文字对象。原型也覆盖 TextLayout 自带的 `Draw`，但当前
  授权目标在两种入口上都没有真实命中，因此不进入正式 Bundle。
- **暂不选择 `SetWindowTextW/WM_SETTEXT`。** 它改变窗口或控件自身保存的文字，不是纯渲染写回；
  Dictionary 更新、停用恢复和应用自身随后改字会产生状态竞争，而且与现有经典 Win32 覆盖重叠。
- **暂不选择 Chromium DOM/调试协议。** 它需要目标明确开放调试连接或提供官方扩展入口，不能作为任意
  已运行桌面软件的默认能力；协议版本和多进程页面生命周期也应由独立 Extension 处理。
- **暂不选择 `DrawGlyphRun`。** 它只有排好位置的 glyph 数据，没有可直接用于 Dictionary 精确匹配的
  原文；在建立可靠上游关联前不能宣称通用文字替换。

首版只接受单一基础格式的 TextLayout；带多段格式、inline object 或无法取得创建证据的 layout 必须
原样放行。该限制要在合成合同中明确，不能为了增加命中而破坏目标布局。

## 验收

1. 合成 DirectWrite/Direct2D 目标以 `CreateTextLayout` 创建已知 Unicode 文字，再分别通过
   `DrawTextLayout` 和 TextLayout 自带的 `Draw` 绘制；Observe 能得到原文，Dictionary 替换改变输出，
   失败路径原样放行。
2. 同一已创建 layout 在停用后恢复原始像素，重新激活后可再次替换；Hook 不要求目标重建 layout。
3. 授权真实目标在用户打开代表性菜单或面板时必须出现 TextLayout 命中；零命中则否决该目标，不进入
   Bundle。命中后还必须完成一个已知标签的可见译文与停用恢复。

## Next

后续第一方资料评审选择了 Qt 作为下一有界技术族；结合“字典修改后立即看到结果”的产品约束，首版
没有采用需要目标主动处理 `LanguageChange` 的 Translator，而是进入
[Qt Painter 实时写回 Adapter](qt-painter-writeback-adapter.md)。它仍须回答同样的发布门槛：真实目标
入口命中、原文关联、Dictionary 可见替换与停用恢复必须同时成立；只有 glyph 或只成功注入仍直接
否决。

## References

- [Direct2D `DrawTextLayout`](https://learn.microsoft.com/en-us/windows/win32/api/d2d1/nf-d2d1-id2d1rendertarget-drawtextlayout)
- [DirectWrite `IDWriteTextLayout`](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nn-dwrite-idwritetextlayout)
- [Win32 `SetWindowTextW`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowtextw)
- [Chromium 调试协议](https://chromedevtools.github.io/devtools-protocol/)
