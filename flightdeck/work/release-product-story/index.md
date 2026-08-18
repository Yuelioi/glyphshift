# 发布前产品表达收敛

Status: Finished

## Goal

在发布前把 Glyphshift 的产品表达收敛为用户可理解、可验证的一条路径：设置页减少分区并合并 AI
执行偏好与 AI Profile；帮助页先解释怎么开始和怎么恢复；中英文 README 说明产品解决的问题、核心
用法与边界；用匿名化的真实界面截图形成发布素材。

## Current

设置页已从六个大分区收敛为四个，帮助页已有首次使用与故障恢复路径，中英文 README 已改为用户
发布文案。5 张匿名化关键页面截图已由 Playwright CLI 生成并完成两轮检查；同步 Release Shell 与
Runtime Bundle 已构建、校验并启动。

## Next

None

## Progress

- 已创建发布前 Work，并将其设为当前 Focus；AE 实机复验 Work 保持 Open。
- 已用红灯合同锁定 AI 分区合并、帮助首屏顺序、应用/权限合并与双语 README 结构。
- 已完成设置与帮助的中英文实现、双语 README 和两轮截图验证；界面机械检测无发现。
- 完整桌面 Playwright 92 项、全仓 Rust、全 workspace Clippy、Rust 格式与生产前端构建通过。
- 同步 Release 已通过 EXE/Runtime manifest 哈希校验并启动；嵌入页面可用且 stderr 为空。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
