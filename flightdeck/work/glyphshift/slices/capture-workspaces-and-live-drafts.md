# 可恢复探针工作区与实时草稿

Status: Complete

## Outcome

把一次性的“开始/停止捕获会话”改造成可长期作业的 Probe Workspace：同一软件可保存多个命名
工作区；运行时持续生成技术记录和纯字典草稿；用户可以暂停、恢复、搜索、分页、批量处理、直接
编辑译文并在 Adapter 支持时实时预览，最后导出记录、草稿或有效 Dictionary `/2`。

## Confirmed product decisions

- 一个软件可以拥有多个持久 Probe Workspace，但同一时刻只允许一个工作区占用目标 Runtime。
- Runtime 状态使用 `running`、`paused`、`interrupted`；不再把“停止”作为主作业动作。
- 应用重新打开时保留工作区、草稿、筛选与当前页；上次仍在运行的工作区恢复为 `interrupted`，
  用户重新连接目标后继续，不伪造仍在监听。
- Capture Catalog 是不可伪造的技术事实；“排除”只改变 Draft membership，不删除捕获证据。
- Draft 只拥有 `source + translation + included` 编辑状态；Adapter、次数和时间只属于 Catalog。
- 选中的 Adapter 全部支持 `TextReplace` 时，新工作区默认开启 Live Preview；编辑后发布新的完整
  Runtime generation，暂停观察不卸载预览。
- UI 严格复用管理页语言：页头、工作区选择/状态、顶部搜索与批量工具条、带复选框的表格、底部
  分页；创建与导出设置进入 Modal，不使用卡片式控制台。

## Performance contract

- 基准规模是每工作区 5,000 条；容量上限继续有界。
- 前端只持有当前页 20/50/100 行；搜索、排序和分页由 Rust Module 完成。
- 运行时最多每 1 秒发布一次增量快照；桌面端先轮询轻量 revision，只有 revision 变化时才重取
  当前页，编辑中的输入和已选行不因刷新被重建。
- Capture 快照使用双槽滚动写入；任意时刻至少保留上一份可校验快照。重启读取 revision 较新的
  有效槽并把未生成 Draft 的 source 补入草稿。
- 分页表不叠加虚拟滚动。Nuxt UI `UTable` 只渲染当前页；若未来增加无分页连续浏览模式，再使用
  `@tanstack/vue-virtual`，不把两套窗口化模型混用。

## Deep Module seam

`glyphshift-capture` 拥有 `CaptureWorkspaceStore`：

- `open/recover` 隐藏 workspace 文件、双槽 catalog、草稿 merge 与崩溃恢复。
- `create/list/summary` 管理多个命名工作区和轻量 revision。
- `synchronize/query/apply/export` 隐藏全量数据、搜索分页、批量 mutation、CSV/JSON 与
  Dictionary `/2` 生成。
- Target Runtime 热路径仍只做有界 `try_send`；定时 checkpoint、暂停门控和 dropped 统计留在
  Capture Module 内部。

Vue 与 Tauri DTO 不读取内部文件，也不接收完整 5,000 条 catalog。

## Delivery

### 1. Durable workspace and runtime control

- [x] 新增 Workspace schema、双槽实时 Catalog、Draft persistence 与 interrupted recovery。
- [x] Controller/Target Runtime 增加 capture pause/resume 控制；暂停不卸载 Adapter 或 Preview。
- [x] Capture Runtime 同时请求可用的 `TextObserve` 与 `TextReplace`，并支持发布 Draft generation。

### 2. Query, mutation and export

- [x] 实现 Catalog/Draft 后端搜索、分页、排序和轻量 revision 查询。
- [x] 实现 include/exclude、清空译文、逐条译文编辑和批量 mutation。
- [x] 实现 Catalog CSV/JSON、Draft CSV/JSON 与 translated-only Dictionary `/2` 导出。

### 3. Management-table UI

- [x] 新建工作区 Modal：名称、软件、Adapter 多选与默认 Live Preview。
- [x] Catalog/Draft Tabs 复用顶部搜索、复选框、批量工具条、完整表格与底部分页。
- [x] Running/Paused/Interrupted 状态、暂停/继续、预览 generation 与 1 秒刷新可见且不跳行。
- [x] 导出设置放 Modal/菜单，普通表格不暴露内部文件路径。

## Verification

- [x] Capture Module 合同覆盖 5,000 条查询、双槽恢复、暂停、批量 mutation 与导出。
- [x] Controller 到 Target Runtime 的暂停/继续与 Preview generation 有合成合同。
- [x] Playwright 覆盖 960×640、1440×900、搜索、多选、分页、编辑和自动刷新稳定性。
- [x] 性能检查证明前端 DOM 行数受 page size 限制，1 秒刷新不传输完整 catalog。
- [x] 工作树内不含 `local-test/` 跟踪内容；本轮未 staged/未提交。

## Current

可恢复 Probe Workspace 已闭合：双槽实时快照、暂停/继续、Live Preview、搜索分页、
批量处理、导出和管理表 UI 均已落地。5,000 条用例只向 Vue 传递当前页 50 行。
启动前真实 Runtime 门禁还锁定了重连时 revision 归零的问题；Capture Sink 现在会续接同一
session 的 revision、条目和 dropped 计数，重复运行合同与真实 Adapter 捕获合同均通过。
实机窗口复核又修复了 `UApp` 虚拟根未承载高度 class 导致表格撑出视口的问题：
现在表格独立滚动、右侧滚动条始终可见且可拖动，底部分页固定可见。技术目录加入 Draft
译文的可编辑投影，行内修改直接触发已有 Live Preview generation。

## Next

后续如增加无分页连续浏览，再以 `@tanstack/vue-virtual` 增加单一窗口化模型；
当前分页管理表无需叠加虚拟滚动。
