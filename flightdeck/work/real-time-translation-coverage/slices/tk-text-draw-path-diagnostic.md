# Tk 文字绘制链对照诊断

Status: Complete

## Outcome

动态 Tk 8.6 的公开文字绘制链没有比现有 GDI `TextOutW` Adapter 产生可验证增量，本切片作
**No-Go**。实验 Adapter 已撤回，不进入 Runtime Bundle、目录或软件支持声明。

## Delivery

- [x] 建立不显示在桌面上的确定性动态 Tk 8.6 宿主，覆盖 classic Label 与 Canvas 短文本。
- [x] 用正式 GDI `TextOutW` 记录原文完整度、Dictionary 命中、中文像素、第二代热更新与停用恢复。
- [x] 建立只挂公开 Tk 导出的诊断 Adapter，以同一宿主、Dictionary 和像素判据复测。
- [x] 比较三条完整短文本、两代中文 fallback 像素、热更新、停用恢复与无模块拒绝。
- [x] 按 Go gate 判定无增量并撤回实验实现；生产 Bundle 与目录保持不变。

## Result

- GDI 与 Tk 两路都稳定观察到 3/3 个完整 Dictionary source，没有出现字体 run 碎片导致的失配。
- 第一代与第二代中文译文在两路中均与目标软件原生设置相同中文时逐像素一致；Tk 自身 fallback 没有
  产生现有 GDI 无法达到的结果。
- 两路均在不重启目标的情况下使用第二代 Dictionary，并在停用后恢复到同一原文像素。
- Tk 纯逻辑合同 5/5、无模块拒绝 1/1 和隐藏双路运行合同 1/1 通过；这些实验代码随后按 No-Go 规则
  从工作区撤回，只保留可移植结论。

## Evaluated design

- `Tk_ComputeTextLayout` / `Tk_FreeTextLayout` 只维护原 layout 到 UTF-8 原文与公开布局参数的映射；
  不修改或接管应用 layout。
- `Tk_DrawTextLayout` 仅对 `firstChar = 0` 且 `lastChar < 0` 的完整绘制临时计算译文 layout；局部选区
  与不明确索引放行原文。
- `Tk_DrawChars` 只覆盖未经过已知完整 layout 的直接 UTF-8 绘制，并用重入护栏避免二次翻译。
- Dictionary 每次绘制重新决策，因此第二代更新无需改控件状态；停用时直接调用原函数。

## Go gate

以下至少一项必须成立，并且热更新与恢复合同同时通过：

1. GDI 观察得到碎片而 Tk 得到稳定完整词条，导致只有 Tk 能精确命中 Dictionary；
2. GDI 写回在原字体 run 中缺字或错误分段，而 Tk 临时 layout 能正确选择 fallback 并产生目标像素。

两路结果等价，因此本切片以 No-Go 完成并移除实验实现。

## Decision

不建设 Tk 专用 Adapter。动态 Tk 8.6 软件继续由现有 GDI `TextOutW` / `ExtTextOutW` 能力探测；如果
未来真实软件出现“Tk 层完整原文而 GDI 层稳定碎片化”的反例，再用该反例重新打开本切片，不预留生产
代码或用户选项。

## Boundaries

- 不启动或弹出任何可见第三方目标；所有运行验证必须后台、隐藏或离屏。
- 首轮只接受动态 Tk 8.6 x64；静态链接、Tk 9、私有结构、签名扫描和软件专用地址表不在范围内。
- 不修改 Tcl 变量、控件 `-text`、窗口标题、输入值或应用资源。
- 机器路径、进程信息、截图、二进制和原始日志只进入 `target/local-test/`。

## References

- [GTK/Pango 之后的通用文字入口评审](../references/post-gtk-next-adapter-primary-source-review.md)
