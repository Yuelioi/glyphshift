# 属性数值文本的翻译后对齐

## 结论与证据边界

本次输入只有字典原文列表，未提供目标软件名称、适配器或翻译后画面。以下是源码调查与实现建议，不能作为该画面错位的已确认根因。没有修改产品代码或运行真实目标。

属性翻译与布局应分开处理：模板保留数值文本，适配器保留布局语义。整串自然排列、整串居中、数字独立列是不同目标，不能统一靠补空格实现。

## 已确认的源码行为

- GDI 的 `call_with_decision` 替换时清除原 glyph-index 标志、将 spacing 置空，但继续使用原 x/y/rect。这样避免把英文逐字间距套给中文，却不会修正宿主先按英文测量得到的坐标。[源码](../../../../crates/adapters/implementations/native/gdi-native/src/lib.rs)
- DrawText 的两个 detour 把译文传给原函数，保留 rect/format，因此宿主传入的矩形对齐标志仍然存在。不能笼统说所有适配器都丢失对齐。[源码](../../../../crates/adapters/implementations/native/draw-text-native/src/lib.rs)
- DirectWrite 的 `replacement_layout` 使用记录的 format 与创建时最大宽高重新创建布局，没有把当前原 layout 创建后独立修改的对齐、宽高等状态复制到译文布局。此为明确的潜在缺口，但尚未证明与用户场景相关。[源码](../../../../crates/adapters/implementations/native/directwrite-native/src/lib.rs)

Microsoft 说明 ExtTextOut 的坐标语义由 DC 对齐设置决定，lpDx 为空时使用默认字间距；其 rect 是裁剪/背景范围，不能直接当成控件布局框。[ExtTextOutW](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-exttextoutw)

DrawText 在指定矩形内按格式标志排版，支持居中与尺寸计算。[DrawTextW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-drawtextw)

DirectWrite layout 自身具有宽高、格式与测量信息；修复时应从当前 layout 读取需要继承的状态，而非只依赖原 format。[IDWriteTextLayout](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nn-dwrite-idwritetextlayout)

## 建议处理

1. 自然排列：替换属性名并原样拼回分隔符、空白及数值，让同一次文字排版计算中文后的数值位置。
2. 整串居中/靠右：有矩形与对齐语义时继承它们；只有已算好的左起点时，需要同一文本决策覆盖测量与绘制，或获得可信锚点后按译文宽度重算。不能从裁剪矩形猜锚点。
3. 数字固定列：优先沿用宿主已有的独立数值控件/绘制位置。若源是一个完整字符串，则需额外的属性区宽度或数值锚点约定；模板替换本身不提供列布局。
4. 中文超宽：优先短译名和正确的字体度量；按具体控件决定省略、换行或有限缩放。避免全局缩字或补空格掩盖问题。
5. DirectWrite 定向修复候选：迁移当前 layout 的对齐、段落方向、宽高与换行等可证明等价的状态，复杂格式继续透传；同时检查宿主是否使用原 layout 的 metrics 决定绘制 origin。

## 后续验证

先确认目标软件、命中适配器、错位种类，再建立合成宿主。覆盖自然排列/居中/靠右/数字列，数值 0→9→10→100，字体替换、DPI、窄框裁剪、第二代译文和停用恢复。使用坐标/度量断言及可见像素检查，不能仅检查译文字符串。未获这些证据前不标记修复完成。
