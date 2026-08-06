# Desktop Runtime 内部 module

Status: Complete

## Goal

保持 Runtime Bundle v2、公开类型/方法、Controller/Session/Target 失败映射、Native/Worker 信任校验与
运行生命周期不变，把 Desktop Runtime 按 Bundle、Pool 和单目标 Runtime 三个深 interface 拆分，并把
964 行内嵌测试按 seam 迁移。

## Baseline

- `src/lib.rs` 2781 行：约 1817 行实现、964 行内嵌测试。
- 14 项确定性单元测试全绿；12 项本机/授权 Runtime 合同保持 ignored，输入输出仍只在
  `target/local-test/`。
- `tests/windows_runtime_contract.rs` 1117 行，属于后续大型合同测试结构切片，不在本切片混入实机值。

## Plan

- [x] 把内嵌测试拆为共享 fixture、Bundle、Pool 和公开合同场景，避免生成单一超长 `tests.rs`。
- [x] 提取 Bundle：manifest、artifact 信任/路径/hash、Adapter presentation、Worker catalog 和 discover。
- [x] 提取 Runtime Pool：ManagedRuntime/Factory seam、状态、Workflow reconcile 与 Capture 所有权。
- [x] 提取单目标 Runtime：连接、Session 生命周期、publish/control/stop 和错误映射。
- [x] 根只保留稳定 re-export、共享错误与 session trace re-export。
- [x] 运行 Runtime 14 项、12 项 ignored 枚举、Backend/Shell 回归、workspace Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审 Bundle schema、公开 API、错误映射、fail-open/abandon 和锁定/停止顺序。

## Result

- `bundle.rs` 496 行：Bundle v2 manifest、规范路径/hash、Native inspect 前校验、Worker catalog、
  Adapter presentation 和 discover。
- `pool.rs` 601 行：ManagedRuntime/Factory 消费侧、状态、Workflow reconcile、Capture 所有权和移除。
- `target.rs` 656 行：单目标连接、Session、publish/control/stop、Drop/abandon 和错误映射。
- 根 `lib.rs` 82 行，只保留稳定 re-export、共享错误和 trace re-export。
- 内嵌测试拆成 324 行共享 fixture、Bundle 104 行、Pool 484 行、公开合同 47 行。

验证通过：14/14 确定性测试；12 项本机/授权合同仍正常枚举为 ignored；Backend 8/8、Shell 44/44、
全 workspace Clippy `-D warnings`、Rust 格式、架构检查与 `git diff --check`。

Standards 自审确认没有公开内部 module、所有文件低于 1200 行、测试不含本机输入；Bundle 的唯一
`unsafe` Native inspect 仍位于规范路径与 SHA-256 校验之后。Spec 自审确认公开类型/方法多重集和
`glyphshift.runtime-bundle/2` 未变，协议/Session/Controller 错误映射、stop/abandon/Drop 顺序均为
原实现迁移并由 Pool/Bundle 合同覆盖。

## Decisions

- Bundle 负责“哪些 bytes 可以被加载”；Target Runtime 负责“一个已验证组合如何连接和运行”；
  Pool 负责“多个软件/Workflow/Capture 的所有权与协调”。
- 不把 Native/Worker Host 合并进 Desktop Runtime；它们仍是独立部署 seam。
- 不改变 `unsafe` 检查位置：Native inspect 之前仍必须完成规范路径和 SHA-256 验证。
- 本切片不执行或改写任何 ignored 实机测试。
