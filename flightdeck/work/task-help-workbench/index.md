# 任务表格与帮助工作台

Status: Finished

## Goal

让翻译任务的统计和任务列表复用产品统一的管理表格能力，并把帮助页改成真正可操作的分区指南，使
高频任务、筛选、分页和直接入口在紧凑桌面窗口中仍然清晰。

## Current

翻译任务已使用与工作流、软件一致的满宽管理页头和任务清单图标；三个 Tab 直接占满剩余工作区，
统计和任务列表不再重复显示分区标题与说明。

## Next

None.

## Progress

- 统计和任务记录均接入 `ManagementTableFrame`、`UTable` 与独立持久化显示列设置。
- 帮助补齐首次工作路径、长期维护、后台 AI 行为、Token 解释、故障入口和 Adapter 渐进详情。
- 中英文与产品/设计契约已同步；前端全量 95/95、机械扫描和独立 UI 审阅通过。
- 同步 Release 桌面壳与 Runtime Bundle 已由统一审阅脚本重建、校验并启动，进程响应且无 stderr。
- 已收到统计页信息层级反馈：Profile 应是筛选维度，协议不应默认突出。
- Profile 筛选、协议默认隐藏、列偏好迁移、定向 Playwright、机械扫描和独立 UI 复核均通过；最新版
  同步 Release App 已重新启动。
- 用户要求翻译任务改用与工作流、软件一致的满宽管理骨架，并移除 Tab 下重复说明。
- `TranslationTasksView` 已切换到 `ManagementPageHeader` 与满宽管理工作区，顶部使用任务清单图标；统计与
  任务列表复用共享表格并填满剩余高度，不再重复标题和说明。
- 前端构建、翻译任务定向 Playwright、发布内容合同及 1440×900 / 960×640 双尺寸视觉验证通过；独立
  UI 复核无阻碍交付问题。
- 根据空闲“当前任务”页面反馈，空状态与运行中任务均改用共享 `ManagementWorkspaceSurface`；空状态
  使用居中的 `UEmpty`，移除横跨页面的分割线式布局。构建、空闲/运行任务定向 Playwright 和视觉检查通过。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [GUI 约定](../../knowledge/gui/nuxt-ui-tauri-vite-setup.md)
