# Adapter 迁移后全仓验证

Status: Complete

## Goal

证明 Adapter 物理归档没有破坏全 workspace 构建/测试、Clippy、桌面产品、Playwright、隐私约束或前序
结构优化成果，并完成最终 Standards/Spec 审核。

## Current

物理迁移后的完整 Rust、桌面与结构门禁全部通过。验证没有启动或读取真实软件，没有把本机证据写入
跟踪目录，也没有暂存或提交。

## Steps

- [x] 运行完整 workspace test、全 workspace Clippy、格式、架构与 diff check。
- [x] 运行桌面生产构建和仓库 Playwright 全套确定性测试。
- [x] 复核 1200 行门槛、Cargo/package 集合、旧路径残留、Git rename 语义与文档链接。
- [x] 扫描 staged/tracked 路径和文本，确保没有 `local-test/` 或机器专属信息。
- [x] 完成最终 Review，更新 Work/Plan/Deck 为 Finished；保持工作树未提交。

## Next

None.

## Evidence

- `scripts/test.ps1` 完整 54-package workspace tests 通过；需要真实软件或显式授权的合同保持 ignored。
- `cargo clippy --workspace --all-targets -- -D warnings`、`cargo fmt --all -- --check`、架构检查和
  `git diff --check` 通过。
- Desktop `npm run build` 通过；仓库 Playwright 60/60 通过。Vite 仅保留既有的大 chunk warning；两个自动
  生成声明文件已恢复到验证前内容，`Cargo.lock` 也无变化。
- Cargo metadata 为 54 个 package；25 个 Adapter 精确分成 4 个 platform 与 21 个 implementations。
  55 个 manifest 的 239 个 path 属性均指向存在的相对路径，迁移前平铺路径残留为 0。
- 236 个代码文件中 0 个达到 1200 行；最大文件 977 行，前序结构优化门槛保持。
- 13 份相关导航文档的 66 个本地链接全部存在。
- Git 暂存区、tracked `local-test/` 和该目录 status 均为 0；变更文本只含 15 个明确合成的路径解析
  fixture，没有开发机路径、用户名或本机证据链接。

## Final review

- Standards：Adapter 真实目录入口与 package 家族一致；依赖层仍由 54-package partition/allow-list 独立
  约束。源码扫描根据 Cargo metadata 派生 package 根，目录移动不再要求同步一份脆弱路径清单。
- Spec：迁移树文件集合与 checkpoint 完全一致；45 个非 manifest 文件逐字节一致，25 个 manifest 只改
  path，另三个深度敏感相对路径经过精确替换。完整 Rust 与 UI 回归未发现 ABI、制品、授权、fail-open、
  schema 或用户行为偏差。

## Residual risks

- 需要显式授权的真实 Host/本机 DLL 合同仍按设计 ignored；本 Slice 没有读取真实软件证据。
- Vite 仍报告既有的大 JavaScript chunk warning。
- 按用户要求，全部结构成果保持未提交。
