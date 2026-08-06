# Capture Workspace 测试 seam

Status: Complete

## Goal

保持 Probe Run 持久文档、连接状态恢复、Dictionary join、筛选/分页/导出与清空观察语义不变，把
362 行内嵌测试按公开 Store 场景迁移；生产 `ProbeRunStore` 保持为一个约 900 行深 interface。

## Baseline

- `capture/src/workspace.rs` 1266 行，其中生产 904 行、内嵌测试 362 行。
- 生产模块已把双槽文档、Capture Catalog、Dictionary snapshot、查询和导出封装在一个 Store interface；
  暂无超过 1200 行的纯生产实现，也没有需要跨模块开放的第二所有者。
- 5 项测试覆盖 Dictionary join/不持久化翻译、断连恢复/ignore、查询与完整导出、连接期设置限制、清空。

## Plan

- [x] 提取共享临时 Store/run fixture。
- [x] 按 join/lifecycle、query/export、settings/clear 场景拆分 5 项测试。
- [x] 保持生产实现逐行不变，运行 Capture 与直接下游回归。
- [x] 运行 Clippy、格式、架构和 diff；自审持久 schema、连接状态与导出语义。

## Result

- `workspace.rs` 从 1266 行降至 907 行；前 904 行生产实现与 `HEAD` 逐行完全一致。
- 共享临时 Store/run fixture 30 行；join/lifecycle 143 行、query/export 95 行、settings/clear 98 行。
- `ProbeRunStore` 继续作为唯一持久文档、revision、Capture Catalog 和 Dictionary join 所有者。

验证通过：Capture 全部 15/15（含 Workspace 5/5）、package Clippy `-D warnings`、Rust 格式、架构检查
与 `git diff --check`。前一 Capture 生产切片已经跑过 11 个直接下游 package；本切片只迁移测试，未
重复执行同一批下游行为测试。

Standards 自审确认最大 Workspace 文件 907 行，fixture 只存确定性临时数据，未引入新的生产可见项。
Spec 自审确认生产实现逐行不变，原 5 项公开 Store 场景仅改变 module 路径；持久 schema、断连恢复、
Dictionary 不被 ignore 修改、筛选/完整导出和连接期设置限制均由同一断言覆盖。

## Decisions

- 保留 `ProbeRunStore` 为一个深 module；其方法共同维护同一份 run 文档、revision 与观察目录所有权。
- 不按 CRUD 方法机械拆生产文件，否则会把同步、双槽读写和状态恢复复制到多个浅模块。
