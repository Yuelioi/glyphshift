# Adapter 家族物理归档

Status: Complete

## Goal

让所有正式 Adapter crate 都能从一个稳定物理入口定位，同时保留既有 Cargo package 名、Native ABI、
进程/部署 seam、制品命名与依赖方向。

## Current

全部 25 个正式 Adapter crate 已迁入统一物理入口：4 个平台/契约 crate 位于
`crates/adapters/platform/`，21 个技术实现 crate 位于 `crates/adapters/implementations/`。三个只含空
`src/`、没有 manifest/Git 条目的中断脚手架已清理。Cargo metadata 仍为 54 个 workspace package、25 个
Adapter package。

## Decisions

- `platform/` 只拥有跨实现复用的平台/契约能力：SDK、Registry、Native ABI 与 Native Host。
- `implementations/` 拥有 21 个具体文字技术 descriptor、Native 实现与 UIA Worker。
- 目录只表达能力家族；架构层级继续由 package 名和依赖门禁表达，不把二者混成一棵树。
- 不改 package 名、crate type、feature、ABI、bundle 产物或代码内容；路径迁移应是机械变换。
- 架构源码扫描应尽量从 Cargo metadata 的 package manifest 派生，避免再次绑定平铺路径。

## Steps

- [x] 验证 25 个源目录、目标目录和 Cargo path 引用的精确集合。
- [x] 迁移目录并机械重写所有 manifest 中的相对 path dependency。
- [x] 更新根 workspace、脚本、架构扫描、文档链接和 crate 导航。
- [x] 清理三个已验证为空、无 manifest、未跟踪的脚手架目录。
- [x] 运行 metadata、格式、架构、受影响测试；审查 diff 与旧路径残留。
- [x] 保存切片结论，把全仓验证与隐私检查交给独立收口 Slice。

## Next

None. 后续执行[迁移后全仓验证](post-adapter-full-validation.md)。

## Evidence

- `cargo metadata --no-deps --format-version 1`：54 个 package，25 个正式 Adapter package。
- 三个额外同前缀目录只有空 `src/`，不含 `Cargo.toml`，也不在 Git 或 workspace 中。
- 25 个 workspace member 与 79 个 manifest path 属性机械重写；55 个 manifest 归一化后与 checkpoint
  内容一致，说明除物理路径外没有语义字段变化。
- 迁移树与 checkpoint 文件集合完全一致：45 个非 manifest 文件逐字节一致，25 个 manifest 仅 path
  属性变化；另有两个测试脚本和一个 UIA 本地证据路径只按新增目录深度调整。
- 迁移前的 Adapter 平铺路径引用和根级同前缀目录均为 0；239 个 Cargo path 属性全部存在、
  均为相对正斜杠路径。
- `architecture-tests/check.ps1`、`cargo fmt --all -- --check`、`git diff --check` 通过。
- 全部 25 个 Adapter package 的 `cargo test` 通过；3 项需要显式本机软件/进程授权的合同保持 ignored。

## Review

- Standards：物理目录只表达 Adapter 能力家族；依赖层仍由 package partition/allow-list 表达。架构源码
  扫描现由 Cargo metadata 派生 package 根，不再绑定平铺路径。
- Spec：package 名、crate type、ABI、实现源码、测试集合和依赖集合保持；唯一非 manifest 修改是新增
  目录深度所要求的三个相对路径修正，真实本机证据仍只落入 `target/local-test/`。
