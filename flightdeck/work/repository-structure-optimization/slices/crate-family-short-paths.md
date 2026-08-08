# crate 家族短目录迁移

Status: Complete

## Goal

把 47 个 `crates/` package 统一放入可导航的能力家族，并用短叶目录消除重复的 `glyphshift-*` /
`glyphshift-adapter-*` 路径噪音；同时把内部发布策略固化为 Cargo metadata，不改变 package/lib 名、ABI、
target 名或产品行为。

## Decisions

- 目标结构以[非 Adapter crate 边界与命名审计](../references/non-adapter-crate-boundary-audit.md)为准。
- 只移动 `crates/` 下 package；`apps/` 和 `test-support/` 继续作为独立组合根和 fixture 根。
- 移动前记录精确 source → destination 映射并验证所有路径均位于 `<repo>/crates`、源存在、目标不存在。
- manifest path 以移动后 package root 重新计算，不做字符串猜测；深度敏感脚本与本地证据路径逐项检查。
- `local-test/` 仍是唯一机器特定证据根，不移动、不跟踪、不写入文档具体值。
- 本切片不提交。

## Steps

- [x] 迁移 25 个 Adapter 与 22 个非 Adapter package 到短家族路径。
- [x] 重写 workspace members、所有 path dependency 与深度敏感测试/脚本路径。
- [x] 为 54 个 workspace package 继承 `publish = false`，并在架构检查中防止遗漏。
- [x] 运行 metadata、内容完整性、目录残留、格式、架构和受影响测试。
- [x] 自审变更与隐私边界并更新 Plan/Work。

## Evidence

- 精确迁移表包含 47 个不相交 source → destination；执行前验证全部源位于 `<repo>/crates`、源存在、
  目标不存在且目标无重复。迁移时对每个 package 的所有文件做 SHA-256 前后比对，内容差异为 0。
- 55 个 Cargo manifest 按移动前解析出的绝对 dependency target 重新计算相对路径；迁移前后 metadata 的
  54 个 package、target name/kind 与 dependency name/kind/rename 对比差异为 0。
- `crates/` 顶层仅剩 `adapters`、`core`、`dictionary`、`product`、`runtime`；旧平铺目录和带完整 package
  前缀的 Adapter 叶目录均为 0。最终 Review 发现单数 `runtime/target/` 会命中 `**/target` 忽略规则，已在
  继续验证前修正为可跟踪的 `runtime/targets/`。
- Cargo metadata：54 个 package，`publish = false` 为 54，允许发布为 0；架构检查已增加发布策略门禁。
- 深度敏感的 Controller Host、Worker Host、Target Runtime 脚本均从新目录成功执行：4 项 Controller
  process、3 项 Worker unit、8 项 Worker process、1 项 Target unit 和 1 项确定性 Target contract 通过；
  3 项需授权 Native DLL 的 Target 合同保持 ignored。
- `cargo fmt --all -- --check` 与 `architecture-tests/check.ps1` 通过。

## Review

- Standards：物理目录只表达导航，现有精确依赖 partition/allow-list 继续约束真实方向；没有创建
  `core/common/utils` 聚合 crate，也没有合并 ABI、进程或协议 seam。
- Spec：用户期望的 `runtime/protocol` 和无重复前缀目录已经落实；Cargo package/lib/target 名全部稳定，
  “内部 package”由可执行的 `publish = false` 表达。
- 完整性：迁移前后 package/target/dependency 集合和文件 hash 一致；唯一有意的非路径 manifest 变化是
  发布策略。`local-test/` 只保存本机迁移证据并继续被 Git ignore。
- Git 可跟踪性：`git check-ignore crates/runtime/targets/contract/Cargo.toml` 返回未忽略；没有用
  `runtime/target/` 与 Cargo 构建输出目录竞争。
- 剩余风险：尚需执行完整 workspace tests/Clippy、桌面生产构建和 Playwright，作为独立最终验证切片。

## Next

执行[家族短目录迁移后全仓验证](post-family-full-validation.md)，完成后关闭本 Work；不提交。
