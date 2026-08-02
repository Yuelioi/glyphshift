# 独立仓库迁移

Status: Complete

## Objective

把通用实现提升为独立仓库根目录，不携带旧产品源码、旧 GUI、构建缓存或兼容层，并证明目录
提升没有改变现有 Module interface 和用户行为。

## Decisions

- 采用复制迁移，原仓库暂不删除，直到新仓库验收完成。
- 原 `v2/` 内容直接提升为根目录；Cargo workspace 不增加指向旧仓库的 path dependency。
- 本机 evidence、machine 配置和设计材料只复制到忽略目录。
- 有价值的旧词典保存在明确的 source archive，不作为产品 Runtime 输入。
- 不创建 V1/V2 compatibility Module；需要历史数据时使用一次性显式导入 seam。

## Acceptance

- [x] 所有 Cargo package 的 manifest 都位于当前仓库。
- [x] 架构检查、workspace test、Clippy 和 fmt 通过。
- [x] 桌面生产构建与仓库 Playwright 37 项回归全部通过。
- [x] staged path 不包含 `target/local-test/` 文件；staged text 不包含本机路径、用户名、PID 或本地截图链接。
- [x] 新仓库有独立初始提交，原仓库只保留用户既有未提交内容。

## Next

迁移 Slice 已结束；继续每个 Workflow Target 的独立有序词典编排。
