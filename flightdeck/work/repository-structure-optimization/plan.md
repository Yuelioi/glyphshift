# 全仓代码结构优化计划

## Stage 1：基线与拆分规则

- [x] 将前序成果提交为 checkpoint `bbebea9`，从干净工作树开始结构优化。
- [x] 统计代码文件物理行数、workspace crate 数量、Adapter 家族数量和内部依赖中心性，形成
  [仓库结构基线](references/repository-structure-baseline.md)。
- [ ] 完成[超长文件职责与拆分图](slices/large-file-decomposition-map.md)：记录首批生产文件的 interface、
  implementation 职责、调用者、内嵌测试和候选内部 seam。
- [ ] 定义 crate 家族与允许的依赖方向，明确 `core`、Adapter 平台、Adapter 实现、Runtime 编排、产品
  外壳和测试支持各自含义；先形成设计结论，不立即批量移动目录。

## Stage 2：桌面产品内部 module

- [ ] 收敛桌面 Tauri 壳：保持 `run()` 与 Tauri 命令名稳定，把请求/视图映射、应用编排、命令注册、
  权限与窗口生命周期、字体缓存及内嵌测试放入按变化原因组织的内部 module。
- [ ] 复核 Desktop Backend 的 interface；把持久化制品映射、Dictionary 操作、Workflow 操作与软件目录
  编排收进深 module，避免顶层 facade 只是逐方法转发。
- [ ] 复核 Desktop Runtime 的 Bundle 验证、Runtime Pool、单目标生命周期和协议错误映射；保持现有公共
  interface 与兼容错误语义。

## Stage 3：核心语义与 Runtime 编排

- [ ] 拆解 Translation 的快照/字体策略与 Workspace 编辑模型；确认它们是否是同一变化原因，避免只按
  类型数量切文件。
- [ ] 拆解 Isolated Worker Host 的进程监督、重启策略、Placement 组合和协议解码，并通过 Host interface
  验证行为。
- [ ] 复核 Capture、Controller Windows、Session、Protocol 与 Target Runtime 的大文件；优先迁移内嵌
  测试和形成私有 module，不扩大公共 interface。

## Stage 4：前端与测试结构

- [ ] 按桌面功能面拆分 Playwright `desktop.spec.ts`，共享确定性 fixture 只保留一份，测试仍从用户可见
  interface 验证行为。
- [ ] 复核 `CaptureView.vue`、`useWorkspace.ts` 与本地化目录；只在状态/副作用/展示职责确实独立时拆分
  composable 或子视图。
- [ ] 将大型 Rust 合同测试与 Windows test-support 宿主按 interface 场景组织，删除结构调整后重复的
  内部细节断言。

## Stage 5：crate 拓扑与物理目录

- [ ] 基于前述 module 结果决定是否把 crate 物理归类为 `core`、`adapters`、`runtime` 等目录家族；保持
  crate 名称、ABI 与发布身份稳定，并评估迁移噪音是否值得。
- [ ] 逐项评审小 crate：用删除测试判断其是否提供深 interface 或真实部署 seam；不得仅因行数少而合并
  Adapter SDK、Native ABI、Worker SDK 或测试 Adapter。
- [ ] 将最终依赖方向写入架构检查，避免重新形成横向依赖和万能共享层。

## Stage 6：全仓验证与维护门槛

- [ ] 每个切片运行受影响测试；阶段完成时运行完整 workspace、全 workspace Clippy、格式和架构检查。
- [ ] 涉及桌面命令、模型或界面时运行生产构建与仓库 Playwright；只使用确定性 fixture。
- [ ] 更新物理行数与 crate 依赖基线；保留“超长文件需评审”规则，不以机械行数限制替代 module 设计。
