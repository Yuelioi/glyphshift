# 本机实验知识提炼与清理

Status: Finished

## Goal

从 `local-test/` 的本机实验中提炼可复用、脱敏且可验证的技术知识；删除结论简单或已由正式实现覆盖的
实验，复杂实验只保留最小源码与必要说明；最后删除整个可再生成的 `target/`。

## Current

本机实验已经完成脱敏审读和逐族判定，脱敏研究稿与跨任务 Knowledge 均已落盘。`local-test/` 已从
构建、Runtime、第三方源码、授权输入和原始证据混合区收敛为最小手写源码集；浏览器 profile 与其他
敏感运行状态已删除。整个 `target/` 已删除，后续 IL2CPP negative harness 也已改写为使用仓库统一的
本机证据根，不会再次创建旧的本机测试根。

保留集、生成目录、PowerShell 语法、Unity 静态合同、Markdown 链接、Git ignore、staged/tracked 路径和
隐私模式扫描均通过；没有运行全仓测试，也没有重建桌面 App。

## Next

None.

## Progress

- 冻结清理原则：简单实验在结论落盘后删除；复杂实验只保留最小源码和必要说明；原始证据不进入
  tracked 文档；`target/` 最终完整删除。
- 完成[本机实验技术发现](references/local-experiment-findings.md)，记录证据分级、各技术族结论、失败
  边界、tracked 对照和清理判定；本机源码与证据的精确清单不进入 tracked 文档。
- 将跨任务复用的 Adapter 晋级门槛与实现约束提升为
  [项目 Knowledge](../../knowledge/rendering/runtime-text-adapter-validation.md)。
- 删除整个 `target/` 及全部本机生成物；`local-test/` 最终只保留有独立复现价值的手写源码和控制文件。
- 修复 Unity IL2CPP negative harness 的旧证据根，并通过 `-ValidateOnly` 静态合同。

## References

- [上下文](context.md)
- [计划](plan.md)
- [既有本地测试根迁移](../local-test-persistence/index.md)
