# 超长文件职责与拆分图

Status: Complete

## Goal

对首批超长生产文件逐一识别现有 interface、implementation 职责、调用者、测试位置和变化原因，选出
能提高 locality、保持 interface 稳定且可独立验证的第一个拆分切片。

## Delivery

- [x] 记录桌面 Tauri 壳、Desktop Backend、Desktop Runtime、Translation 与 Isolated Worker Host 的
  公共 interface、主要调用者和内部职责簇。
- [x] 对 Capture、Controller Windows 与 Capture Workspace 区分“实现过长”和“内嵌测试导致文件过长”。
- [x] 对每个候选执行删除测试：如果抽出的 module 删除后复杂度只散回调用者，说明它有深度；若只剩
  转发，则调整 seam。
- [x] 为第一拆分候选固定兼容面、测试命令、预计文件结构和回滚边界。

## Responsibility map

| 候选 | 现有 interface 与调用者 | implementation 职责簇 | 删除测试与候选内部 seam |
|---|---|---|---|
| Desktop Tauri Shell | crate 对外只有 `run()`，由 `main.rs` 调用；前端通过固定 Tauri 命令名和 Serde 字段使用 | 启动/提权、错误映射、请求与视图、`DesktopApplication`、Runtime 展示映射、字体目录、命令、快捷取词、Builder 与 35 项根测试 | 删除字体目录后缓存格式、去重、扫描和写盘会散回启动与刷新路径，因此形成深 module；应用方法应按 Probe/Dictionary/Workflow/Software 变化原因纵向组织 |
| Desktop Backend | `DesktopBackend` 与创建/编辑/视图类型；由 Desktop Runtime 和 Shell 调用 | 产品快照、软件目录、Dictionary、Workflow、持久制品映射、路径与原子写入 | 持久化 module 删除后 JSON、校验、路径和恢复逻辑会散回所有写操作；适合保留 Backend facade，把存储与领域操作收进私有深 module |
| Desktop Runtime | `RuntimeBundle`、`DesktopRuntimePool`、`DesktopRuntime` 与状态/回执类型；由 Shell 调用 | Bundle 清单与制品验证、Pool 对账、单目标生命周期、Controller/Session 错误映射 | Bundle 验证和单目标生命周期各自隐藏多依赖与失败语义，是两个高 leverage 内部 seam；不新增公共 port |
| Translation | Snapshot、Font Policy、Workspace interface；13 个内部 crate 调用 | 确定性快照/摘要、字体规则/摘要、来源合并、Workspace 编辑/冲突/事件 | 删除 Snapshot/Font 与 Workspace 任一职责簇都会把不同算法散回多个调用者；应在 crate 内分 module 并保持公共 re-export |
| Isolated Worker Host | `ProcessIsolatedWorker`、`IsolatedWorkerHost`、`HybridAdapterHost`；Desktop Runtime 与 UIA Worker 使用 | 子进程协议、监督/重启、健康与 drain、Host 激活、Target/Worker 混合 Placement | 进程监督和 Placement 组合分别隐藏真实跨进程 seam；拆内部 module，不能合并掉 Worker SDK/Host crate |

## 文件膨胀分类

| 文件 | 实现与测试判断 | 处理顺序 |
|---|---|---|
| `glyphshift-capture/src/lib.rs` | 约 941 行实现 + 398 行内嵌测试 | 先把测试归到 Capture interface，再复核 Catalog/Ingress/File Sink 是否需要内部 module |
| `glyphshift-controller-windows/src/lib.rs` | 约 1040 行实现 + 244 行内嵌测试 | 先迁测试；之后按可执行文件识别/提权与 Controller 进程编排复核 |
| `glyphshift-capture/src/workspace.rs` | 约 904 行实现 + 362 行内嵌测试 | 当前已经是独立 Workspace module，先拆测试场景，不因总行数继续拆 implementation |

## Desktop Shell 方案比较

### 方案 A：横向技术分层

`models.rs` 放全部请求/视图，`application.rs` 放状态与操作，`commands.rs` 放 Tauri wrapper，
`platform.rs` 放 Windows 与文件系统逻辑。优点是迁移机械、依赖直观；缺点是一次 Probe、Dictionary 或
Workflow 变化仍需横跨三到四个文件，commands 容易退化成大批浅转发，locality 提升有限。

### 方案 B：纵向功能 module + 小型平台 module

根 `lib.rs` 保留 `run()`、Builder 组合和共享状态；Probe、Dictionary、Workflow、Software 各自收纳
请求/视图、`DesktopApplication` 方法和命令 wrapper；字体、启动/提权、快捷取词等稳定平台能力各自
形成小 interface。它需要谨慎使用 `pub(super)` 维持私有协作，但功能变化集中、测试可穿过同一命令或
内部 interface，depth 与 locality 更好。

选择方案 B。不会一次性搬完 3000 行；先提取依赖最少且删除测试成立的 `font_catalog`，验证私有 module
模式，再按功能逐步迁移。

## Current

职责、调用者、删除测试和桌面壳两种方案均已记录。首个 implementation seam 选择字体目录：两个入口
隐藏缓存 schema、读取/写入、去重和 Windows 注册表扫描；测试随 module 迁移。实际实施与后续桌面壳
拆分由[桌面 Tauri 壳内部 module](desktop-shell-internal-modules.md)继续承接。

## Next

进入[桌面 Tauri 壳内部 module](desktop-shell-internal-modules.md)，按已选纵向方案继续 Probe 功能面，
保持命令名、序列化字段、错误码与 `run()` interface 不变。

## Boundaries

- 本切片只产出职责图和首个实施设计；implementation 变更由后续切片承接。
- 不用“每 500 行一个文件”或按类型名字母分组；module 必须对应稳定变化原因。
- 不把只有一个实现的私有 helper 抬升为公共 port；测试可使用私有内部 seam。

## References

- [仓库结构基线](../references/repository-structure-baseline.md)
- [稳定上下文](../context.md)
- [阶段计划](../plan.md)
