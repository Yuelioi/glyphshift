# 桌面 UI/UX 一致性优化

Status: Finished

## Goal

在不改变 Glyphshift 既有产品模型的前提下，系统检查并修复桌面端主要表面的视觉与交互不一致，
让列表、详情、设置、帮助和 Probe 在层级、密度、组件语法、状态反馈、键盘操作与窄窗口行为上形成
一套可预测的 Windows 桌面体验。

## Current

本轮 UI/UX 一致性优化已完成。紧凑管理表保留对象与操作上下文，功能性元数据纳入语义字体角色；
Probe 详情动作收敛并隐藏临时资产内部标识；Help 改为任务式恢复入口和渐进 Adapter 目录；应用壳层、
共享工具栏与详情表面补齐主内容跳转、可访问名称、heading 层级和焦点恢复。实现已通过受影响回归、
生产构建、反模式扫描与代表性宽窄/深浅实机复核，用户已确认本阶段暂告完成。

## Next

None.

## Progress

- 已确认本阶段先检查 UI 统一性与 UX，再按审计结果拆分修复。
- 已确认本机截图、视频和运行日志只进入 `local-test/evidence/`，不成为仓库资源。
- 已用两路独立评审完成主要列表、详情、Settings 与 Help 的设计/体验检查；设计评审 Playwright 3/3 通过。
- Impeccable detector 唯一一次执行为 0 findings；机械浏览器检查提供代表性证据，但 Help 注入、紧凑
  Settings 几何与全量点击目标统计因审计 harness 有界终止而保留为后续补测。
- 已把问题收敛为 0 项 P0、2 项 P1、3 项 P2；不建议重做应用骨架。
- 已确认第一批合并处理两项 P1；Probe 详情、Help 信息架构与全局 heading 语义留在后续 Slice。
- [紧凑表格上下文与语义字体阶梯](slices/compact-tables-and-type-ramp.md) 已完成：64 项定向 Playwright、
  production build、Impeccable 0 findings 与 960×640 深浅色代表性复核通过。
- [Probe 任务动作与临时命名](slices/probe-task-actions-and-naming.md) 已完成：23 项 Probe/AI 定向回归、
  production build、Impeccable 0 findings 与 1440/960 深浅色代表性复核通过。
- [Help 任务入口与全局键盘语义](slices/help-and-keyboard-semantics.md) 已完成：受影响 Playwright 48 项、
  production build、Impeccable 0 findings，以及 1440/960 深浅色与键盘代表性复核通过。
- 三个实现 Slice 全部完成；本轮未改变既有产品模型、Probe 生命周期或 Adapter 技术文档 URL 合同。
- 用户已确认 UI/UX 升级阶段暂告完成，并以本次提交作为阶段边界。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [成熟设计系统校准基线](references/design-system-benchmarks.md)
- [基线审计](references/baseline-audit.md)
