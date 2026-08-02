# 纯字典、常用 Adapter 与捕获探针

Status: Complete

## Outcome

让 Dictionary 成为唯一、可复用的纯翻译资产；扩展有真实观察价值的 Windows 文字 Adapter；
允许用户监听一个已授权软件，把去重原文与对应 Adapter 技术来源保存为 Capture Catalog，并导出
不携带技术字段的 Dictionary Draft。

## Confirmed seams

- `glyphshift-dictionary-package` 唯一拥有 Dictionary `/2` codec、校验与 mutation。
- `glyphshift-workflow::resolve` 把纯 Dictionary 编译进 Runtime；当前对 Software 的全部内部路由生效。
- Region Binding 属于 Workflow Target，不属于 Dictionary。没有真实区域信号时只允许 `all`。
- Native Adapter 已能把 Adapter ID 与 UTF-16 原文送入 Target Runtime；捕获在 Decision 前记录，
  但必须有界、非阻塞并保持 fail-open。
- Capture Catalog 保存技术证据；Dictionary Draft 只保存 `source + translation`，两者不双写字段。
- 真实进程输入、日志、截图和捕获结果只放 `target/local-test/`；tracked tests 只使用合成进程和文本。

## Delivery

### 1. Pure Dictionary `/2`

- [x] Dictionary artifact entry 改为非空 `source` 与非空 `translation`。
- [x] 删除公开 DTO、Tauri model、Vue editor 中的 Location、Context、keep/空译文。
- [x] 同一 Dictionary 内按 source 校验唯一；冲突诊断按 source 与 Dictionary 优先级报告。
- [x] Workflow 将每个纯词条编译到 Software 声明的所有内部路由，不把路由泄漏回 payload。
- [x] 更新现有 `/2` 本机词典；迁移备份与验证证据只进入 `target/local-test/evidence/`。

### 2. Common Adapter coverage

- [x] 消除 Windows PowerShell 5.1 对 Runtime Bundle 中文 presentation 的错误解码。
- [x] 增加观察优先的 `DrawTextW/DrawTextExW` 与 `TextOutW` Adapter，或用探针证据明确否决。
- [x] 为并行 Adapter 增加重复观测归并合同，避免同一绘制链重复替换或重复计数。
- [x] DirectWrite、UI Automation、Qt/Electron 等只在探针证明 seam 可观察/可写回后进入 Catalog。

### 3. Capture probe

- [x] 建立 Capture Session：显式目标进程、Adapter Plan、开始/停止和有界状态。
- [x] 建立 Capture Catalog：按规范化原文与 Adapter 去重，记录出现次数和首次/最后观测时间。
- [x] Target Runtime 的热路径只做有界入队；Controller/Desktop 负责持久化与展示。
- [x] 从 Capture Catalog 导出 Dictionary Draft，译文初始为空仅作为编辑状态，不能保存为空 Dictionary。
- [x] 提供合成端到端合同与本机原生 Adapter smoke 流程。

## Verification

- [x] 相关 Rust package 合同与 workspace tests 通过。
- [x] Frontend build 与仓库 Playwright CLI 全部通过。
- [x] Runtime fail-open、停止捕获、容量上限、重复文本和多 Adapter 情况有合同覆盖。
- [x] worktree 范围不含 `target/local-test/`、本机路径、用户名、PID 或本地证据链接。

## Current

三项纵切已交付：Dictionary `/2` 为纯 `source + translation`；Runtime Bundle 提供四个常用
GDI/GDI+ seam；桌面“探针”页可启动观察会话并生成 Capture Catalog 与 Dictionary Draft。

## Next

在真实授权第三方软件上按需运行探针，使用 Capture Catalog 决定下一批 Adapter seam；未取得
可观察证据前不增加 DirectWrite、UI Automation 或框架专用 Adapter。

## Results

- 本机现有四份 Dictionary `/2` 共 2285 条已原地升级，备份与报告仅保存在 local-test evidence。
- 新增 `TextOutW`、`DrawTextW/DrawTextExW`，连同现有 `ExtTextOutW`、`GdipDrawString` 进入 Bundle。
- 原生像素合同、真实 Adapter 捕获合同、workspace tests、Clippy、架构检查、frontend build 与
  Playwright 24 项全部通过。
- 剩余风险：不同第三方软件的覆盖率取决于其真实渲染技术，需由探针结果逐项决定后续 Adapter。
