# AI 翻译始终确认

Status: Finished

## Goal

删除“翻译前询问”设备偏好，让词典与探针每次发起 AI 翻译都先显示确认 Modal；同时澄清安装版与同步
Review 版的数据根边界，避免把隔离审阅数据误判为真实 AI 配置丢失。

## Current

“翻译前询问”已从设置页和 App Settings 合同删除；词典、探针和预览入口每次创建 AI 任务前都会显示
确认 Modal。旧设置中的 false 值可以安全读取但会被忽略，下一次保存设置时不再写回该字段。同步
Review 继续使用隔离数据根；后续工作已把安装版工作区切到独立品牌目录。

## Next

None

## Progress

- 用户确认 AI 翻译属于低频且可能计费的操作，应当始终确认，不再提供跳过确认的选择。
- 已确认同步 Review 启动器与安装版不是同一数据实例；Review 隔离也避免把真实明文 API Key 复制进
  本机测试证据目录。
- 红灯验证旧 false 值曾绕过确认且设置页仍显示开关；修复后两条合同通过。
- 安装版 AI Profile artifact 仍存在且可读；隔离 Review 工作区为空，证明配置未删除而是数据根不同。
- 桌面设置 Rust 5/5、AI Playwright 15/15、Settings Playwright 8/8、生产前端构建和 Rust 格式检查
  通过；960×640 与 1440×900 视觉检查通过，UI 机械检查无发现。
- 同步 Review Release 重建启动成功，Runtime Bundle 验证 9 个 Adapter；未运行归档 UIA 测试。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [AI 翻译配置](../ai-translation-profiles/index.md)
