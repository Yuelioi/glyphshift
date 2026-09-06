# AE 临时界面观测丢失

Status: Open

## Goal

让 AE 2020 与 AE 2022 的合成设置对话框和多级菜单稳定进入探针 Observation Index；
不重新启用 UIA，并用同步构建的 Desktop Release 与 Runtime Bundle 完成实机验收。

## Current

AE 效果参数遗漏已修复并通过实机验证：`FileCaptureSink` 原来每条新观测都会重新开始一秒
空闲等待，持续重绘使 checkpoint 长期无法写出。现在使用单调时钟截止时间周期性落盘，
持续生产测试要求生产未停止前至少发布两次新 revision；旧实现失败，修复后 capture 的
21 项测试全部通过。

同步 Release 与 Runtime Bundle 已构建并通过 9 个 Adapter 加载校验，使用原用户数据恢复
现有探针。Playwright 验证截图中 13 个原先缺失的效果参数全部被 GDI+ 捕获；通过产品词典
同步接口补入 20 个词条后，AE 实际窗口显示对应中文。该验证使用确定性中文词条验证回写，
没有调用外部 AI 服务。原 AI 配置与合成内容保留，实机证据只在本机忽略目录。

发布前性能对比覆盖千条与五万条合成目录：持续输入期间新版每秒发布 checkpoint，旧版不更新；
两版入队均保持微秒级且没有新增丢弃。新版后台 CPU 在这两个短时样本中分别低于单核 1% 和约
3.5%。写盘仍由后台所有者执行，正常周期性 I/O 有开销，不承诺所有机器与负载下零性能影响。
capture 21 项、目标进程宿主 checkpoint 合同 1 项以及发布内容 4 项通过；桌面版本已同步为
`0.2.3`，准备提交并推送对应 annotated tag。

根因已修复并在 AE 2022 实机通过：`CaptureBatchProducer::drain` 原来每 250ms 持有写锁，
热路径 `try_observe` 在这个窗口不等待而直接丢弃。现在生产端只把已验证的未编号观测写入
有界队列，由单一消费端按出队顺序分配 sequence；drain 不再与 Native Adapter 入队互斥。

新 Desktop Release 与 Runtime Bundle 已同步构建并通过 9 个 Adapter 的真实加载校验。新
Runtime 注入全新的 AE 2022 进程后，合成设置对话框、启用态合成菜单和二级菜单均进入
Observation Index，本机精确回归脚本已转绿。正在运行的 AE 2020 仍驻留旧 Runtime DLL，
需要重启目标进程后才能完成同版本实机复验。

桌面审阅脚本现显式启用 Tauri 的生产 `custom-protocol`，同步 Release Shell 已通过实际 WebView
回归从嵌入资源加载界面，不再意外依赖本地 Vite 服务。

## Next

重启 AE 2020，让它加载本次候选 Runtime，并重复合成设置对话框、启用态菜单和二级菜单的
精确回归；若通过则关闭本 Work。稠密 AE 重绘仍会累计部分 dropped 计数，但本轮要求的短暂
表面均已捕获；后续如要改善指标，需要把无效绘制输入与真实队列饱和分开计数。

## Progress

- 最新 AE 2020 探针中 9 个 Adapter 已全部加载，GDI/GDI+ 观测仍在增长。
- 可见译文证明二级菜单调用已命中 Hook 与词典，排除 Runtime 整体失活、Adapter 未装载与
  GDI 字形索引整体解码失败。
- 实机目录在菜单打开的相邻 revision 间丢弃计数增加 2，但已替换的两个原文均未出现在目录。
- 回归与历史提交已确认：旧的目标内 `FileCaptureSink` 路线曾捕获这些文本；后来引入跨进程
  observation batch 后，生产端在 drain 写锁竞争时按设计丢弃。
- 当前截图中没有活动合成，多个菜单项处于禁用态；禁用态是否另有未覆盖绘制路径仍待修复后
  独立验收。
- 新增并发回归在旧实现稳定失败，修复后 `glyphshift-capture` 19 项测试全部通过；Runtime、
  进程宿主与控制器的相关合同测试也全部通过。
- 同步 Release 候选的 Desktop EXE、Runtime manifest 与 9 个 Adapter 哈希均已验证；AE 2022
  目标进程确认加载该候选的新 Runtime 与全部 Adapter。
- AE 2022 实机已捕获 `Composition Settings...`、`Composition Name:`、`Save Frame As` 与
  `Responsive Design — Time`；来源分别覆盖 ExtTextOut、GDI+ 与 DrawText，精确脚本通过。
- 修正原始 Cargo 审阅构建缺少 Tauri `custom-protocol` 的问题；同步 Release 的 WebView 已确认
  加载 `tauri.localhost` 嵌入资源，拒绝连接回归转绿。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [AE 2020 渲染路径实证](../../knowledge/rendering/ae2020-text-render-map.md)
- [GDI 字形索引处理](../../knowledge/rendering/glyph-index-encoding.md)
