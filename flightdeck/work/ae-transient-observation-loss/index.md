# AE 临时界面观测丢失

Status: Open

## Goal

让 AE 2020 与 AE 2022 的合成设置对话框和多级菜单稳定进入探针 Observation Index；
不重新启用 UIA，并用同步构建的 Desktop Release 与 Runtime Bundle 完成实机验收。

## Current

根因已修复并在 AE 2022 实机通过：`CaptureBatchProducer::drain` 原来每 250ms 持有写锁，
热路径 `try_observe` 在这个窗口不等待而直接丢弃。现在生产端只把已验证的未编号观测写入
有界队列，由单一消费端按出队顺序分配 sequence；drain 不再与 Native Adapter 入队互斥。

新 Desktop Release 与 Runtime Bundle 已同步构建并通过 9 个 Adapter 的真实加载校验。新
Runtime 注入全新的 AE 2022 进程后，合成设置对话框、启用态合成菜单和二级菜单均进入
Observation Index，本机精确回归脚本已转绿。正在运行的 AE 2020 仍驻留旧 Runtime DLL，
需要重启目标进程后才能完成同版本实机复验。

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

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [AE 2020 渲染路径实证](../../knowledge/rendering/ae2020-text-render-map.md)
- [GDI 字形索引处理](../../knowledge/rendering/glyph-index-encoding.md)
