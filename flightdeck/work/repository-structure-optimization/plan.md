# 全仓代码结构优化计划

## Stage 1：基线与拆分规则

- [x] 将前序成果提交为 checkpoint `bbebea9`，从干净工作树开始结构优化。
- [x] 统计代码文件物理行数、workspace crate 数量、Adapter 家族数量和内部依赖中心性，形成
  [仓库结构基线](references/repository-structure-baseline.md)。
- [x] 完成[超长文件职责与拆分图](slices/large-file-decomposition-map.md)：记录首批生产文件的 interface、
  implementation 职责、调用者、内嵌测试和候选内部 seam。
- [x] 定义[crate 家族与允许的依赖方向](references/crate-families-and-dependency-directions.md)，明确 `core`、Adapter 平台、Adapter 实现、Runtime 编排、产品
  外壳和测试支持各自含义；先形成设计结论，不立即批量移动目录。

## Stage 2：桌面产品内部 module

- [x] 完成[桌面 Tauri 壳内部 module](slices/desktop-shell-internal-modules.md)：保持 `run()` 与 Tauri 命令名
  稳定，把请求/视图映射、应用编排、命令注册、权限与窗口生命周期、字体缓存及内嵌测试按变化原因组织。
- [x] 完成[Desktop Backend 内部 module](slices/desktop-backend-internal-modules.md)；把持久化制品映射、Dictionary 操作、Workflow 操作与软件目录
  编排收进深 module，避免顶层 facade 只是逐方法转发。
- [x] 完成[Desktop Runtime 内部 module](slices/desktop-runtime-internal-modules.md)：拆分 Bundle 验证、Runtime Pool、单目标生命周期和协议错误映射；保持现有公共
  interface 与兼容错误语义。

## Stage 3：核心语义与 Runtime 编排

- [x] 完成[Translation 内部 module](slices/translation-internal-modules.md)：拆解快照/字体策略与
  Workspace 编辑模型；确认它们是否是同一变化原因，避免只按类型数量切文件。
- [x] 完成[Isolated Worker Host 内部 module](slices/isolated-worker-host-internal-modules.md)：拆解进程监督、重启策略、Placement 组合和协议解码，
  并通过 Host interface 验证行为。
- [x] 复核并拆解 Capture、Controller Windows、Session、Protocol 与 Target Runtime 的大文件；优先
  迁移内嵌测试和形成私有 module，未扩大公共 interface。

## Stage 4：前端与测试结构

- [x] 完成[Desktop Playwright spec 拆分](slices/desktop-playwright-specs.md)：按桌面功能面拆分
  `desktop.spec.ts`，共享确定性 fixture 只保留一份，测试仍从用户可见 interface 验证行为。
- [x] 复核 `CaptureView.vue`、`useWorkspace.ts` 与本地化目录；只在状态/副作用/展示职责确实独立时拆分
  composable 或子视图。
- [x] 将大型 Rust 合同测试与 Windows test-support 宿主按 interface 场景组织（Session 合同、Windows
  Host 与 Desktop Runtime Windows 合同已完成），删除结构调整后重复的
  内部细节断言。

## Stage 5：crate 拓扑与物理目录

- [x] 基于前述 module 结果决定是否把 crate 物理归类为 `core`、`adapters`、`runtime` 等目录家族；保持
  crate 名称、ABI 与发布身份稳定，并评估迁移噪音是否值得。
- [x] 逐项评审小 crate：用删除测试判断其是否提供深 interface 或真实部署 seam；不得仅因行数少而合并
  Adapter SDK、Native ABI、Worker SDK 或测试 Adapter。
- [x] 将最终依赖方向写入架构检查，避免重新形成横向依赖和万能共享层。

## Stage 6：全仓验证与维护门槛

- [x] 每个切片运行受影响测试；阶段完成时运行完整 workspace、全 workspace Clippy、格式和架构检查。
- [x] 涉及桌面命令、模型或界面时运行生产构建与仓库 Playwright；只使用确定性 fixture。
- [x] 更新物理行数与 crate 依赖基线；保留“超长文件需评审”规则，不以机械行数限制替代 module 设计。

## Stage 7：Adapter 家族物理入口复核

- [x] 完成 [Adapter 家族物理归档](slices/physical-adapter-family.md)：将平台/契约与具体技术实现分别迁入
  `crates/adapters/platform/` 和 `crates/adapters/implementations/`，保持 package、ABI 与制品合同稳定。
- [x] 更新所有 Cargo/path 引用、架构扫描与仓库导航；清理不属于 workspace 的空 Adapter 脚手架。
- [x] 完成[迁移后全仓验证](slices/post-adapter-full-validation.md)、隐私检查与最终 Review，再将本 Work
  恢复为 Finished；本阶段不提交。

## Stage 8：非 Adapter crate 边界与命名复核

- [x] 完成[非 Adapter crate 边界与命名复核](slices/non-adapter-crate-boundaries-and-naming.md)：结合 Rust/Cargo
  官方资料，区分 package 名、lib crate 名、目录名与依赖 alias，不把四者混成一个重命名问题。
- [x] 对 29 个非 Adapter package 逐项执行删除测试，记录真实发布/ABI/进程/编译 seam、调用者与合并候选。
- [x] 完成[crate 家族短目录迁移](slices/crate-family-short-paths.md)：对有收益的物理归档和内部发布策略实施
  并验证；package 改名与 crate 合并因收益不足而明确拒绝。

## Stage 9：家族迁移后全仓验证

- [x] 完成[家族短目录迁移后全仓验证](slices/post-family-full-validation.md)：完整 Rust workspace、Clippy、
  Desktop build、Playwright、metadata/lockfile、目录、文档链接与隐私门禁全部通过。
