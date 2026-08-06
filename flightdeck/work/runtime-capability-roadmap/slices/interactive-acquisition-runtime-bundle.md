# 交互式 Acquisition Worker 正式 Bundle 接入

Status: Finished

## Outcome

Runtime Bundle 当前 `/3` 可声明并验证一次性 Acquisition Worker 及其 support files；Desktop Runtime 只按 Adapter ID 取得受
信 Host，不向 Shell 暴露制品路径。正式构建包含 `windows.uia.acquire` executable，缺件、篡改、重复
ID 或越界路径均在启动 Worker 前拒绝。

## Scope

- Bundle manifest 增加必填 `acquisition_workers`，每项只声明 Adapter ID、文件与 SHA-256；空列表必须
  显式声明，未发布 schema 不接受缺字段的历史形态。
- `RuntimeBundle` 在加载时验证路径、摘要、ID 及唯一性，并提供有界 timeout 的 Host factory。
- Runtime Bundle 构建脚本复制并声明 UIA 一次性 Worker；输出目录精确文件集合继续由 manifest 驱动。
- 用纯 Bundle 合同与本地正式 Bundle 合同覆盖正常、缺字段、缺件、篡改、重复和未知 ID。

## Non-goals

- 不在本切片中选择前台 Target、签发 Controller grant、注册全局快捷键或创建外部呈现窗口。
- 不把 Acquisition ID 塞入持续 Runtime `Feature`，也不把它冒充 `TextObserve` / `TextReplace`。
- 不加入生产 Target Frame Source、OCR 引擎或在线 Translation Provider。

## Verification

- [x] `/3` manifest 必须显式声明 `acquisition_workers` 与每项的 `support_files`，缺字段稳定拒绝。
- [x] 受验证 Artifact 可按 Adapter ID 创建 Host，未知 ID 与重复声明稳定拒绝。
- [x] 缺件、摘要不匹配和越界路径在进程启动前失败。
- [x] Debug Runtime Bundle 包含且只包含 manifest 声明的 UIA Acquisition Worker。
- [x] Workspace test、Clippy、fmt、architecture checks 与代码自审通过。

## Review

- `acquisition_workers` 是当前未发布 `/2` schema 的必填字段；无 Worker 时也必须显式写空列表。未来
  真正发布后的兼容只通过明确 schema 版本与迁移策略引入，不在当前解析器里静默补缺字段。
- 已验证制品在 Bundle 内保持 opaque；Desktop Runtime 只公开稳定 Adapter ID 列表与有界 Host factory，
  Shell 无法取得 executable 路径、摘要或自行绕过 Bundle 校验启动进程。
- 单元合同覆盖旧清单、正常创建、未知/重复 ID、缺件、篡改摘要与越界路径。真实 Debug Bundle 声明
  1 个 `windows.uia.acquire` Worker，manifest 声明与输出目录精确匹配 14/14 个文件。
- `glyphshift-desktop-runtime` 17 个常规测试、现有真实 Bundle 合同、60 包 workspace 测试、workspace
  Clippy `-D warnings`、fmt、architecture checks 与 diff whitespace 检查均通过。
- 全仓首轮回归暴露两份旧合成 fixture 未声明新字段；已改为显式空数组并保持原错误优先级。现在只有
  专门的 schema 合同省略字段并断言拒绝，没有保留静默兼容路径。

## Next

进入 [Desktop Acquisition Session](interactive-desktop-acquisition-session.md)：复用 Controller 当前
target grant，并把 opaque target、point selection 和一次性 Worker 生命周期封装在 Runtime 内，不让
Tauri command 接触进程标识、grant payload 或制品路径。
