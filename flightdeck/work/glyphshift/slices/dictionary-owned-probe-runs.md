# Dictionary-owned Probe Run

Status: Complete

## Outcome

把 Probe Workspace 收敛为可恢复的 Probe Run：每个任务只绑定一个 Software、一个 Dictionary
和一组 Adapter；Dictionary 是唯一翻译内容资产，Observation Index 是内部技术证据 sidecar。
用户在探针详情看到一张联合表，可以从已有 Dictionary 继续观察，也可以新建空 Dictionary 后立即
开始探针，不再导入、转换或同步 Dictionary Draft。

## Confirmed product decisions

- 一个 Probe Run 必须绑定且只绑定一个 Dictionary。
- 从零创建探针时先创建空 Dictionary；从已有 Dictionary 开始时直接绑定，不复制 entries。
- 非空译文立即成为 Dictionary Entry；清空已观测条目的译文会删除 Dictionary Entry，但保留
  Observation Index 记录并回到待翻译状态。
- 忽略/恢复只修改 Observation Index，不创建空译文或 keep entry。
- 探针管理是任务列表；探针详情与 Dictionary 详情读取同一内容资产，但探针详情额外投影技术证据。
- 内容变化与证据变化分别使用 dictionary revision 和 observation revision；纯观测更新不能把
  Dictionary 标记为 modified。
- UI 只显示一张可搜索、可多选、可分页、可行内编辑的联合表，不再暴露“技术目录/字典草稿”Tabs。

## Delivery

### 1. Domain and storage

- [x] Probe Run document 只保存任务身份、Dictionary 引用、运行状态和轻量计数。
- [x] Observation Index 保持双槽 checkpoint、5,000+ 条有界捕获和 interrupted recovery。
- [x] 删除持久 Dictionary Draft 及其导出/同步合同。

### 2. Dictionary composition

- [x] Desktop Module 以单一 Interface 联合查询 Dictionary Entries 与 Observation Index。
- [x] 行内编辑、清空和批量操作原子修改绑定 Dictionary；预览从 Dictionary 生成。
- [x] 新建任务支持选择现有 Dictionary 或创建空 Dictionary。

### 3. Desktop UI

- [x] 探针管理与详情使用明确的列表/返回导航，不用工作区下拉承担管理。
- [x] 详情只保留一张联合表和一个状态模型：待翻译、已翻译、未观测、已忽略。
- [x] 保留搜索、多选、批量忽略/恢复/清空、分页、持续刷新、滚动条和导出。

## Verification

- [x] Module 合同覆盖已有 Dictionary 接续探针、从零建 Dictionary、clear 保留证据、ignore 不污染
  Dictionary、5,000 条分页及重启恢复。
- [x] Playwright 覆盖管理页、联合表、行内编辑、批量操作、分页与 960×640/1440×900。
- [x] Desktop build、Rust workspace tests、fmt、Clippy 和架构检查通过。

## Current

Dictionary-owned Probe Run 已贯通 Capture Module、Desktop/Tauri、Vue 与测试。旧一次性 Capture
commands 和 Dictionary Draft 类型已移除；管理表连续背景、空态填充及末行边界同步修正。

## Next

启动本地桌面 App 做手动体验；后续实机观察证据仍只写入 `target/local-test/evidence/`。
