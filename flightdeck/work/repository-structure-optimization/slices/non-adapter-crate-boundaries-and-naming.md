# 非 Adapter crate 边界与命名复核

Status: Complete

## Goal

判断 29 个非 Adapter workspace package 中哪些应继续独立、哪些可以合并为更深 module、哪些只需要物理
目录归档，以及 `glyphshift-` package 前缀是否应保留或只在局部缩短使用名。

## Current

已完成官方资料调研与 29 个非 Adapter package 的逐项删除测试。结论是不合并 package，也不把 Cargo
身份批量改成 `protocol`、`session` 等通用名；保留 `glyphshift-` 作为工具、日志与未来 registry 的稳定
上下文，同时把目录改成 `crates/runtime/protocol` 等短家族路径。仓库 package 均属内部制品，因此后续
统一声明 `publish = false`。

## Decisions

- package 名、Rust 代码中的 crate 名、Cargo dependency key/alias 和物理目录名分别评审。
- 只在删除 crate 后复杂度不会跨调用者复制时合并；ABI、`cdylib`/binary、进程、权限、协议或独立发布
  owner 默认视为真实 seam，但仍需用实际调用者验证。
- `runtime`、`protocol` 等短名是否清晰，要同时考虑 workspace 外诊断、制品、日志和未来发布，不只看
  `use` 语句长度。
- 审计阶段不改 Cargo package identity、外部 interface、ABI 或产品行为，也不提交。
- 全仓 dependency alias 会产生 package/源码两套术语与约 371 处迁移，不作为结构整理的默认动作。
- `core` 只可作为物理目录；不创建 `core` crate，避免与标准库命名冲突及万能共享层。

## Steps

- [x] 汇总 Rust/Cargo 官方命名、workspace、package/lib/alias 机制和发布约束。
- [x] 生成 29 个非 Adapter package 的 crate type、target、直接依赖/调用者、源码规模和公开 surface 清单。
- [x] 对 `protocol`、Runtime contract/kernel/target/session、Controller/Worker、Dictionary 与产品 crate
  分组执行删除测试。
- [x] 分别给出“保留前缀”“局部 alias”“物理归档”“package 改名”“crate 合并”的收益/代价矩阵。
- [x] 自审结论并更新 Plan；仅对证据充分的实施项创建后续 Slice。

## Evidence

- [Rust/Cargo crate 命名与 workspace 最佳实践](../references/rust-crate-naming-and-workspace-best-practices.md)
- [非 Adapter crate 边界与命名审计](../references/non-adapter-crate-boundary-audit.md)
- `cargo metadata --no-deps --format-version 1`：54 个 workspace package，其中 29 个非 Adapter；逐项采集
  targets、normal/dev consumer 与依赖集合。

## Review

- Standards：审计将编译/产物 seam 与文件导航分开；单一调用者没有被机械当作合并理由。
- Spec：`runtime/protocol` 被确认适合作为物理路径，但不牺牲 Cargo package 的项目上下文。
- 风险复核：目录移动需同步 55 个 manifest、4 个 PowerShell 深度路径及 5 类
  `CARGO_MANIFEST_DIR` 本地证据路径；实施切片将逐项核对。

## Next

实施 [crate 家族短目录迁移](crate-family-short-paths.md)，保持 package/lib/ABI 身份不变，并同步内部
`publish = false` 策略。
