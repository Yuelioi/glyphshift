# 家族短目录迁移后全仓验证

Status: Complete

## Goal

从全仓入口验证 47 个 crate 的短目录迁移和内部发布策略没有改变编译、测试、GUI、ABI/制品、文档导航
或隐私边界，并完成最终结构 Review。

## Steps

- [x] 运行完整 workspace tests、全 workspace Clippy、格式与架构检查。
- [x] 运行 Desktop 生产构建与仓库 Playwright CLI 全套测试。
- [x] 复核 Cargo metadata、lockfile、目录残留、代码行数门槛与本地文档链接。
- [x] 复核未跟踪/暂存状态和隐私模式；不提交。
- [x] 更新 Work/Deck 为完成态。

## Next

None.

## Evidence

- 当前 `runtime/targets/` 最终路径下，`scripts/test.ps1` 完整 54-package workspace 通过；需要真实软件、
  本机 Qt DLL 或显式授权进程的合同按设计保持 ignored。
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all -- --check`、
  `architecture-tests/check.ps1` 与 `git diff --check` 通过。
- Desktop `npm run build` 通过；Playwright CLI 60/60 通过。Vite 只报告既有的大 chunk warning；构建生成的
  两个本机依赖布局声明已恢复，`Cargo.lock` 无变化。
- Cargo metadata：54 个 package、47 个 `crates/` package、允许发布 0；迁移前后 package target 和
  dependency identity 差异 0。55 个 manifest 的 239 个 path 属性全部存在。
- 47 个 package 叶目录均不再重复 `glyphshift-*`；旧 `crates/glyphshift-*` 与
  `adapters/{platform,implementations}/glyphshift-adapter-*` 路径残留为 0。
- `crates/runtime/targets/` 未被 Git ignore。单数 `runtime/target/` 在终审时被发现会命中 `**/target`，已
  在最终验证前更正并重新运行完整 Rust 门禁。
- 236 个代码文件中 0 个达到 1200 行，最大文件 977 行。37 份相关 Markdown 的 92 个本地链接全部存在；
  同时修复一条既有的模型复核失效链接。
- staged、tracked `local-test/`、`Cargo.lock` 状态和生成声明状态均为 0；变更文本没有仓库绝对
  路径或本地用户名。

## Final review

- Standards：目录家族改善导航但不替代依赖门禁；`core` 只是目录，ABI/cdylib/process/protocol seam 与
  54-package 精确依赖合同保持。
- Spec：用户提出的 `runtime/protocol` 已落实；内部 package 通过 `publish = false` 明确，Cargo identity
  保留 `glyphshift-` 以维持工具、日志、制品和潜在 registry 上下文。
- 完整性：迁移前后文件 hash、package targets 与 dependencies 一致，完整 Rust/UI 回归无行为差异。

## Verification environment note

第二次完整 Rust 重跑前，Cargo 增量缓存耗尽本机磁盘并触发 PDB/rustc I/O 错误；这不是测试断言失败。
只清理了可再生的 `target/debug/incremental`（约 11.7 GiB），保留 `local-test/`，随后用
`CARGO_INCREMENTAL=0` 重新运行完整 workspace 并通过。

## Residual risks

- 显式授权的真实 Host/本机 DLL 合同仍按设计 ignored，本切片未读取真实软件证据。
- Vite 保留既有的大 JavaScript chunk warning。
- 全部结构成果按用户要求保持未提交。
