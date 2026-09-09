# 本机实验知识提炼与清理

Status: Finished

## Goal

从本机实验提炼脱敏、可复用的已验证知识，清理可重建缓存，并按全局根变量加软件名隔离后续构建，整理本机目录与配置入口。

## Current

本轮按用户要求再次整理知识、缓存及本机测试目录。此前完成记录不能代表后续实验没有重新积累；本轮重新盘点后清理仓库旧 target、旧桌面及实验构建缓存，并仅按本项目 package 清理共享 Cargo 根，未删除其他项目缓存。正在被进程引用的审阅目录保留。

Cargo 入口统一为全局根目录下的 `glyphshift/`，显式 override 保持精确；构建、测试、子进程与本机配置入口使用同一解析结果。`local-test/` 根只保留导航、机器入口和 tools/config/software/cache/evidence 分区，历史根脚本归档并更新显式引用，活动实验按软件存放。配置不再保存长期 PID。

已提炼 [缓存文字刷新与恢复](../../knowledge/rendering/retained-text-refresh.md) 和 [本机产物存放](../../knowledge/workflow/local-artifacts.md)。Houdini 未解问题仍在下一项 Work，不提升为已验证知识。本机操作清单、原始证据和历史配置只留在忽略目录。

本轮核对：清理约 66.4 GiB 逻辑文件大小，local-test 约 3.34 GiB；保留一个仍被进程引用的旧审阅目录。Cargo 路径合同、33 个 PowerShell 文件语法、9 个迁移 Python 脚本语法及 12 个 Playwright 配置加载通过。未重建全部缓存或重新启动目标。

第二轮将 94 个证据根散落文件按验证日志、整理记录和软件调查分组，清空 11 项已经失效的软件路径设置并保留本机历史值；Houdini 采样输出改用专属目录。原始证据内容、视频工程与已保留的审阅目录未删除。

## Next

None. 后续软件覆盖在独立 Work 继续。

## 历史整理记录

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
