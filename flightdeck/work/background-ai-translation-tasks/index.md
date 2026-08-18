# 后台 AI 翻译任务

Status: Finished

## Goal

把 AI 翻译从页面内等待改成全局唯一的后台任务：用户可以离开词典或探针继续其他工作，随时查看真实
批次、耗时和 Token；任务只锁定目标词典写入，并新增复用本机 ChatGPT 登录的 Codex 订阅 Profile。

## Current

后台任务、词典锁、Codex 订阅、真实 usage、三 Tab 工作台与 Profile 推理控制均已交付。DeepSeek V4
高推理根因已由官方文档、wire body 和真实对照共同确认：旧 Profile 未发送控制，模型默认 high thinking；
提示词没有要求思考，重复约束已精简但不是 12k 推理的主因。

Profile schema 3 默认关闭翻译推理并迁移旧 Profile；Responses、Chat/Compatible 和 Codex 均发送真实
关闭/自动/强度字段，任务与历史保存启动档位。真实 ds 16 条对照从 12,315 reasoning、110.6 秒降为
0 reasoning、0.97 秒，16/16 完成；真实 Codex 关闭推理也返回 0 reasoning。

## Next

None.

## Progress

- 唯一任务与不排队合同由 Core 测试覆盖；第二次发起在 UI 直接回到当前任务。
- 目标词典只读、其他词典仍可写由 Desktop 合同覆盖；真正退出会确认并在重启后记录中断。
- Codex CLI 直连与 Glyphshift Rust Provider 均完成真实合成烟测；没有读取或复制账户凭据。
- 前端 94 条 Playwright、相关 Rust 三个 package 全部通过。
- 当前任务、每批、模型统计与历史均显示输入、缓存输入、输出、推理和总 Token；独立视觉 verdict 为
  PASS。
- 最新同步 Release App 已启动并保持运行，嵌入页面加载正常，stderr 为空。
- 用户实机反馈后的 B5 已把任务页分为三个 Tab；停止和整体进度位于当前任务首屏，批次与历史详情均
  按需展开，标题栏使用任务清单图标。
- B5 的 AI 定向 11/11、Playwright 全套 94/94、机械 detector 与独立视觉审阅均通过；最新 Release
  App 已重新启动。
- B6 请求合同先复现默认缺少 reasoning 字段，再由 Profile schema 3 与协议映射修复；真实 ds 16 条
  reasoning 从 12,315 降为 0，真实 Codex reasoning 也为 0。
- B6 相关 Rust 三个 package 全部通过，前端构建、AI 12/12、机械 detector、Profile 960×640 截图和
  独立 UI review 通过；最新同步 Release App 已启动且 stderr 为空。
