# 任务表格与帮助工作台

Status: Finished

## Goal

让翻译任务的统计和任务列表复用产品统一的管理表格能力，并把帮助页改成真正可操作的分区指南，使
高频任务、筛选、分页和直接入口在紧凑桌面窗口中仍然清晰。

## Current

翻译任务统计和任务列表已经复用共享管理表格，具备搜索、协议或状态筛选、显示列和完整分页；模型身份
与详情操作在紧凑窗口中固定，低频请求与批次诊断按行展开。帮助页已经分为使用指南、AI 翻译、故障
排查和技术与兼容四个 Tab，默认六步路径可以带首次用户完成实际界面翻译。

## Next

None.

## Progress

- 统计和任务记录均接入 `ManagementTableFrame`、`UTable` 与独立持久化显示列设置。
- 帮助补齐首次工作路径、长期维护、后台 AI 行为、Token 解释、故障入口和 Adapter 渐进详情。
- 中英文与产品/设计契约已同步；前端全量 95/95、机械扫描和独立 UI 审阅通过。
- 同步 Release 桌面壳与 Runtime Bundle 已由统一审阅脚本重建、校验并启动，进程响应且无 stderr。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [GUI 约定](../../knowledge/gui/nuxt-ui-tauri-vite-setup.md)
