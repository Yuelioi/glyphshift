# 紧凑表格上下文与语义字体阶梯

Status: Complete

## Goal

让 960×640 的桌面窄窗口仍能同时辨认当前对象和访问行操作，并把功能性说明、状态与表格辅助信息从
9–10px 的零散字号收敛到可复用的语义字体阶梯；保持高密度桌面表格，不改造成移动端卡片。

## Delivery

- [x] 为工作流、软件、词典、Probe 运行与 Probe 条目表建立共享的选择列、身份列和操作列固定语法。
- [x] 工作流保留诊断与编辑直接入口，把低频复制/删除折叠进具名“更多操作”菜单。
- [x] 横向可滚动表格提供可聚焦区域和可访问名称，键盘进入后仍保留对象身份与操作上下文。
- [x] 建立 caption、label、metadata、body、section title、page title 语义字号变量，并接入 Nuxt UI 与共享管理组件。
- [x] 清理主要管理流程中的 9px 功能性说明；10px 仅保留非关键计数、徽标或紧凑装饰信息。

## Acceptance

- 960×640 下把工作流表横向滚动到最右端时，当前行名称和操作入口仍位于表格可视区域内。
- 工作流行最多持续展示诊断、编辑与一个“更多操作”触发器；复制和删除仍可用且可访问名称包含对象名。
- 表格滚动区域可由键盘聚焦，并具有清晰的 focus-visible 状态。
- 共享功能性 metadata 计算字号不低于 11px；页面标题、分区标题、正文、标签与 caption 的职责可由 token 识别。
- 定向 Playwright、桌面生产构建、Impeccable 检查与 960×640 深/浅色代表性复核通过。

## Current

首批两项 P1 已完成。共享列协议固定选择、身份与操作上下文，工作流行持续展示诊断、编辑和更多操作；
Nuxt UI、共享管理组件及主要管理流程已接入 10/11/12/13/20px 的语义字体角色，源码不再使用
8–10px 任意值承载功能性文案。

## Verification

- Desktop production build 通过。
- `component-system` 与 `management` 19 项 Playwright 通过；Workflow、Dictionary、Probe、Settings
  受影响页面 45 项 Playwright 通过。
- 本地 960×640 深/浅色代表性截图确认工作流、软件、词典的固定列层级、行密度与文本清晰度正常；
  原始证据只在 `local-test/evidence/`。
- Impeccable 最终 detector 为 0 findings；`git diff --check` 通过。

## Next

另建后续 Slice 处理 Probe 详情动作/临时命名、Help 信息架构与全局键盘入口/heading 语义；不在本 Slice
继续扩大范围。
