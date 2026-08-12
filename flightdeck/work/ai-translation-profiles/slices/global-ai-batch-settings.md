# 全局 AI 批次设置

Status: Complete

## Goal

把 AI 单批条目上限与输入 Token 预算作为唯一的应用级设置持久化。用户切换任何 Profile 时，Dictionary
与 Probe 的一键翻译都继续使用相同批次策略；Profile 只保留供应商、模型、连接、并发、凭据和过滤规则。

## Steps

- [x] 在 AppSettings 中加入默认 100 条与全局 16,000 输入 Token 预算，并验证读写边界。
- [x] 从 AI Profile artifact、视图和 resolved profile 移除每批条目与字符字段。
- [x] Translation Job 接收显式 Batch Policy，以条目数和保守 Token 估算共同切批。
- [x] Settings 增加全局控件，Profile 编辑器移除重复字段，预览引用全局值。
- [x] 更新产品契约、中英文文案及定向 Rust / Playwright 回归。

## Verification

- AppSettings round-trip 保存 120 条与 32,000 Token；越界策略被拒绝且不覆盖当前值。
- 合成任务在条目上限 100 时仍因 420 Token 测试预算拆成 `2 + 2 + 1`，证明两项约束独立生效。
- AI Rust 14 项、桌面壳 71 项、Settings / AI Playwright 13 项及生产构建通过。
- Rust 格式、架构检查与界面反模式检测通过；真实桌面 IPC 返回全局 100 / 16,000，Profile 不再返回旧字段。
