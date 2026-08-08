# GTK 3 Pango 绘制时写回

Status: Complete

## Outcome

动态 GTK 3 普通文本可以在公开 `gtk_render_layout` 绘制入口以 Layout 副本完成 Dictionary 精确替换、
同进程更新和停用恢复；目标原 Layout 始终不被修改。Adapter 已进入正式 Runtime Bundle 与中文目录。

## Result

- 纯逻辑合同 6/6 通过，覆盖普通文本、全范围样式、局部/未知属性 fail-open、无效 UTF-8、重入、超长文本、
  observe-only 与回调失败。
- Native Host 缺少 GTK/Pango 导出时的拒绝激活合同 1/1 通过，不会在不兼容进程中猜测模块或符号。
- 动态 GTK 3 / Pango 离屏宿主确认普通文本与全范围样式均产生译文像素；局部 byte-index 样式保持原文，
  停用后像素恢复原文。
- Pango 属性迭代器会在 `INT_MAX` 产生退化终止区间；实现因此检查首个有效范围是否覆盖
  `0..INT_MAX`，没有使用会误判全范围样式的“是否存在下一段”判断。
- 正式控制器注入链在后台离屏目标中完成第一代与第二代 Dictionary 发布，两代各记录 31 次
  `Matched + Replaced`，停止 Runtime 后最终绘制恢复原文。
- 动态 GTK 3 官方参考目标也完成跨进程命中，记录 602 次绘制与 6 次字典替换；这只证明技术入口，
  不单独形成具体软件支持声明。
- 正式 Bundle 构建已包含版本化 GTK 3 Adapter 文件并校验清单哈希；本机运行库、PNG 与原始日志只位于
  `local-test/`，不进入仓库。

## Delivery

- [x] 建立动态 GTK 3 合成宿主，覆盖普通文本、全范围样式和局部 byte-index 样式。
- [x] Adapter 只复制并修改满足护栏的 `PangoLayout`；局部样式范围 fail-open。
- [x] 验证第一代译文、第二代 Dictionary 更新、停用恢复、重入和对象释放合同。
- [x] 使用动态 GTK 3 参考目标验证跨进程注入、绘制命中和字典匹配。
- [x] 加入 Runtime Bundle 构建、中文目录与仓库测试入口。

## Boundaries

- 只覆盖动态 GTK 3 的公开 `gtk_render_layout`；GTK 2、GTK 4、静态链接和私有 GTK 符号不在本切片。
- 不修改目标 `PangoLayout`，不跟踪或遍历目标控件，不重写 markup/mnemonic/link 的局部属性索引。
- Adapter 进入 Bundle 表示该文字技术可供能力探测，不表示所有 GTK 3 软件或所有界面文字已经验证支持。
- 机器路径、进程信息、截图、二进制和原始日志只进入本地测试证据目录。

## References

- [GTK/Pango 实时写回候选评审](../references/gtk-pango-adapter-primary-source-review.md)
