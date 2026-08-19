# 实验盘点与清理

## Goal

把 `local-test/` 中的材料按实验族而非按文件逐项审阅，先形成可复核的技术结论与清理判定，再执行
不可逆删除。

## Current

分类、提炼和清理均已完成。正式实现覆盖的本地原型、一次性诊断、编译缓存、Runtime Bundle、原始
证据、授权输入与上游源码副本已删除；GTK、SDL、Raylib、Unity、Qt、DirectWrite/GDI 与 Web 会话中
仍有独立复现价值的部分按最小源码原则保留，精确清单只留在本机工作区。

## Decision rules

- `Delete`：一次性诊断、简单 smoke、空目录、可重建产物、依赖缓存、重复 Runtime、可重新下载源码。
- `Source only`：仍有复现价值且没有等价 tracked harness 的复杂实验；只保留手写源码、入口和说明。
- `Evidence only until distilled`：原始证据只保留到对应结论被核实并脱敏写入研究稿，随后删除。
- `Promote`：跨任务复用且由源码或多项证据验证的结论，提升到 `flightdeck/knowledge/`。

## Next

None.

## Verification

- 删除前逐项确认结论已进入研究稿或已有 tracked 权威来源。
- 删除后检查保留目录只含允许的源码/说明扩展名和必要小型资源。
- 最终检查 `target/` 不存在、`git status --short` 无本机材料、tracked 文本无新机器标识。
