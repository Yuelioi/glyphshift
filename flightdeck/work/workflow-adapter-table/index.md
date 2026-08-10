# 工作流拦截方式表格化

Status: Finished

## Goal

新建或编辑工作流时，拦截方式使用一张连续、可扫描的选择表；Platform 与 Technology 作为数据列，
表头提供清晰的一键全选与取消全选，并保持每个软件目标的独立选择状态。

## Current

工作流 Adapter 目录已收敛为一张连续表，Platform 与 Technology 成为数据列；表头复选框支持
未选、部分选中、全选和取消全选，且操作只影响当前软件目标。

## Next

None

## Progress

- 截图确认问题是 Adapter 目录的结构与批量操作缺失，不是目录数据或分类解析错误。
- Playwright 合同先稳定失败于“没有连续表”，改造后已验证单表、分类列和全选/取消全选。
- 960×640 开发界面确认阅读顺序、列对齐、长分类与半选状态清晰，表格在更窄内容区可横向滚动。
- 工作流页面 10 项 Playwright 回归、生产构建与 Impeccable 最终布局检测全部通过。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
