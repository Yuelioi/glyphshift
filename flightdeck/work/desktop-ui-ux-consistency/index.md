# 桌面 UI/UX 一致性优化

Status: Finished

## Goal

在不改变 Glyphshift 既有产品模型的前提下，系统检查并修复桌面端主要表面的视觉与交互不一致，
让列表、详情、设置、帮助和 Probe 在层级、密度、组件语法、状态反馈、键盘操作与窄窗口行为上形成
一套可预测的 Windows 桌面体验。

## Current

术语、恢复文案与说明层级均已完成普通用户化。页面、设置分区、字段和弹窗中复述标题或可见控件的
description 已移除；共享页头与表单分区不渲染空说明，无说明设置行从 76px 压缩为 56px。保留内容只
包含快捷键操作、UAC 后果、费用/隐私、非显然作用范围、危险操作与失败恢复。

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
- 发布前用户反馈产品文案仍面向开发者；第五阶段改为以第一次使用者能否立即理解和行动为验收标准。
- 第五阶段已建立产品文案规范并完成设置、AI 任务、工作流、词典、探针、帮助、错误与删除确认改写；
  移除普通界面的修订号、预览代次和发布哈希，并为兼容方式说明补齐中英文。
- 凭据 Rust 合同 3/3、production build、机械扫描通过；Playwright 全量首轮 86/97 后仅发现旧断言，
  定向修复组 48/50、最终两项 2/2、AI 12/12、跨表面 64/66 与最终 Settings 2/2 均通过；独立复核 PASS。
- 用户追加“说明必须提供新信息”的要求；第六阶段清理显而易见的页头、分区和字段说明。
- 第六阶段已让 shared page/detail header、utility shell、form section 和 form row 原生支持无说明状态；
  设置宽窄截图密度清晰，无空节点或异常留白。production build、Playwright 全量 97/97、最终视觉用例
  与独立复核 PASS。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [成熟设计系统校准基线](references/design-system-benchmarks.md)
- [基线审计](references/baseline-audit.md)
- [面向普通用户的产品文案规范](references/product-copy-guidelines.md)
