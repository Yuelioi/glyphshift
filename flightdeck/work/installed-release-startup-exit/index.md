# 已安装发行版启动即退出

Status: Finished

## Goal

定位并消除 GitHub Actions 发布安装包在 Windows 安装后界面短暂出现、随即退出的问题；确保普通启动与
管理员启动都能进入稳定可交互状态。按用户明确选择把 AI Profile 的 API Key 改为随 Profile 明文保存，
设置表单支持显示/隐藏；同时修复 Codex 订阅 Profile 的连接失败，并用与生产交付路径一致的候选验证。

## Current

三项实现已完成。Desktop Backend 会把有效 Dictionary `/2` 原子迁移为 `/3`；单个不可读词典会保留
原文件、进入脱敏 `artifactWarnings` 并被跳过，引用缺失词典的活动工作流在内存中安全停用，不再把
单文件错误升级为 Tauri setup panic。Desktop API 已提升到 v31，词典页会显示可恢复警告。

AI Profile `/4` 直接把 API Key 随 Profile 明文保存并通过 IPC 提供给编辑表单；字段默认遮蔽并
支持显示/隐藏。首次发布前清理已删除 Credential Manager、凭据迁移 seam 与相关依赖。

Codex 订阅默认模型已改为官方 `gpt-5.6-sol`，旧错误值 `gpt-sol-5.6` 会规范化；Provider 现在同时读取
Codex stdout JSON 事件与 stderr，因此模型不可用会报告具体原因。真实 Codex CLI 差分与同步 Release
WebView 产品连接测试均已通过。

验证期间错误运行了原始 `cargo test --workspace`，导致归档 UIA package 被调度。该结果不计入门禁，
会话已终止。UIA、UIA worker 和会间接构建它们的 OCR worker 现已移出 workspace 成员并进入
`exclude`；AGENTS、默认测试脚本与架构守卫共同禁止对这些归档 package 执行任何作业。
按用户后续明确要求，归档中的 20 个 UIA Rust 测试全部增加 `archive_uia_` 前缀与
`#[ignore = "archive-only UIA…"]`；唯一入口默认拒绝，只有显式 `-ArchiveUia` 才使用关键词加
`--ignored` 运行。当前只做静态计数 20/20，没有执行 UIA 测试。

归档安全的架构守卫与 `scripts/test.ps1` 活动包门禁已通过。正式 NSIS candidate 已生成：桌面壳、安装器
与 Runtime manifest 哈希全部匹配；安装器内含唯一 Runtime Bundle `/3` 清单，9 个活动 Adapter，0 个
UIA/OCR 归档条目，0 个合成 `test-target.exe`。candidate 未签名，尚未执行安装器，因此当前系统安装与
卸载记录未被改动。

用户删除旧安装并明确授权后，unsigned NSIS 已以当前用户静默安装成功，安装/卸载记录、EXE、
Uninstaller 与唯一 Runtime Bundle `/3` 均存在且哈希匹配。普通权限默认工作区启动存活通过；隔离安装版
smoke 又验证 Dictionary `/2 → /3` 保留 revision/译文、Profile `/4` 明文 Key 默认遮蔽并可显示/隐藏、
真实 Codex `gpt-5.6-sol` 连接成功。测试 Profile 已删除，测试进程已退出，安装保留。

## Next

None

## Progress

- 已把用户可见症状与隐私边界记录为独立 Work，避免与发布文档补全任务混合。
- 已建立约 6 秒的无人值守启动存活探针，能同时检查进程存活和响应式可见窗口。
- 已安装版原数据根 2/2 红灯，退出码均为 101；空数据根 1/1 绿灯。
- 单个合成 Dictionary `/2` 1/1 红灯；相同内容仅改为 `/3` 后 1/1 绿灯。
- tag 检查确认 `v0.2.0` 已包含 Dictionary `/3` 变更；既有合同只明确拒绝更老的未发布 `/1`，没有
  覆盖已存在 `/2` 数据的升级启动。
- 已核对 AI Profile 存储实现与合同：Profile 元数据属于数据根，Windows Credential Manager 只保存
  密钥；定向 Rust 合同 1/1 通过，证实明文密钥不落入 Profile 文件，但删除数据根会删除 Profile 索引。
- 用户明确选择不再使用 Windows Credential Manager：Key 随 Profile 明文保存，设置表单必须支持
  显示/隐藏，删除 Profile 不再需要外部凭据清理。
- Codex CLI 已登录 ChatGPT；`gpt-sol-5.6` 最小调用稳定返回 400，官方与本机 Catalog 的
  `gpt-5.6-sol` 使用相同调用成功返回 `OK`，连接失败已缩到错误模型 ID。
- AI Rust 29/29、Desktop Backend 10/10、Dictionary Distribution 9/9、Desktop Shell 76/76、前端
  生产构建与 Playwright 100/100 通过；设置页双尺寸视觉检查和机械检测无发现。
- `scripts/review-app.ps1` 同步 Release 已验证 9 个 Runtime Adapter；原 `/2` 复现数据迁移为 `/3`，
  进程存活、窗口响应、stderr 为 0；真实 WebView Playwright 与产品内 Codex 连接测试 1/1 通过。
- 原始全 workspace Cargo 命令误调度归档 UIA 后被用户纠正并终止；UIA、UIA worker、依赖它们的 OCR
  worker 已进入 workspace `exclude`，默认测试脚本引用为 0，架构检查只保留发布泄漏与归档边界负向守卫。
- 归档 UIA 测试标记静态检查 20/20：全部默认 ignored 且使用 `archive_uia_` 关键词；手工入口必须显式
  提供 `-ArchiveUia`，本轮未运行这些测试。
- 架构守卫和 `scripts/test.ps1` 活动包全量门禁通过，执行输出不含 UIA、UIA worker 或 OCR worker。
- 正式 unsigned NSIS candidate 构建成功；候选三项哈希一致，安装器 19 个条目中包含 1 个 Runtime
  manifest，归档 UIA/OCR 与合成测试目标均为 0。
- 用户授权后的 NSIS 静默安装退出码为 0；安装后 Runtime 哈希一致，普通启动存活通过。
- 隔离安装版 Playwright 1/1 通过：Dictionary `/2` 原子迁移为 `/3` 并保留 revision/译文，Profile `/4`
  明文 Key 可显示/隐藏且保持，真实 Codex 连接成功；stderr 为 0，测试进程和测试 Profile 已清理。

## References

- [发布根文档对齐](../release-root-docs/index.md)
- [桌面应用行为设置](../desktop-app-behavior-settings/index.md)
- [通用产品交付](../glyphshift/index.md)
