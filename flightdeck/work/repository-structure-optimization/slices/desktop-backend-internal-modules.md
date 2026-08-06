# Desktop Backend 内部 module

Status: Complete

## Goal

保持 `glyphshift-desktop-backend` 的公共类型、方法签名、Serde 视图、持久化 schema、错误值和原子写入
语义不变，把 2851 行单体实现按 Dictionary、Workflow、Software/Extension、Snapshot 与共享 Storage
seam 组织成深 module。

## Baseline

- `src/lib.rs`：2851 行纯生产实现，没有内嵌测试。
- 外部合同测试：`desktop_backend_contract.rs` 499 行、8 项，当前全绿。
- 根文件同时包含 30 余个公共 DTO、11 个私有持久 artifact、`DesktopBackend` 全部用例、Workflow
  编译、Extension 校验、视图映射和文件 I/O。

## Plan

- [x] 提取 Dictionary 纵向 module：公共 DTO、package 映射、安装摘要、CRUD 与文件交换。
- [x] 提取 Workflow 纵向 module：公共 DTO、artifact、编译/激活、Runtime Intent 与状态持久化。
- [x] 提取 Software/Extension 纵向 module：软件 DTO、Extension artifact、Runtime Spec、校验与目录操作。
- [x] 提取 Snapshot 与共享 Storage seam；根只保留错误、Backend 状态、Environment、打开/组合和 re-export。
- [x] 运行 Backend 合同、Desktop Shell 回归、package/workspace Clippy、格式、架构与 diff check。
- [x] 按 Standards/Spec 自审公共 API、持久 schema、错误字面量和原子写盘行为。

## Result

- `dictionary.rs` 776 行：Dictionary DTO、v2 package 映射、安装摘要、CRUD、文件交换和加载。
- `workflow.rs` 938 行：Workflow DTO/artifact、编译/激活、Runtime Intent 和状态持久化。
- `software.rs` 818 行：Software/Extension DTO/artifact、启动加载、Runtime Spec、目录修改和校验。
- `snapshot.rs` 109 行；`storage.rs` 51 行；根 `lib.rs` 209 行。

根通过 `pub use` 保留原 crate import surface；Software 的 `load()` 隐藏 Extension 扫描、校验、默认
locale、Adapter requirement 收集和 desktop-state 过滤，根只组合各 feature 的加载结果。

验证通过：Backend 合同 8/8、Desktop Shell 44/44、全 workspace Clippy `-D warnings`、Rust 格式、
架构检查与 `git diff --check`。

Standards 自审确认没有公开 feature module、内部协作不超过 `pub(super)`、所有文件低于 1200 行，
Storage 只包含安全路径/JSON/原子发布。Spec 自审逐项比较了公共类型/方法多重集和四个持久 schema
标识；BackendError、Dictionary v2、Workflow 编译/激活以及原子写盘代码均为原实现迁移，合同测试全绿。

## Decisions

- 根通过 `pub use` 保持现有 crate API；不要求调用者改 import path。
- 不建立只逐方法转发的 facade 对象；各 feature module 直接为同一个 `DesktopBackend` 实现用例。
- Shared Storage 只隐藏安全 ID、JSON 读取与原子发布，不吸收 feature schema 或业务校验。
- Snapshot 是跨功能读模型，可依赖三类 feature view；feature module 不依赖 Snapshot。

## Boundaries

- 不改变 `glyphshift.extension/1`、`glyphshift.workflow/3`、Workflow/Desktop state schema。
- 不改变 Dictionary v2 package、绝对路径要求、16 MiB 导入上限或安全 artifact ID 规则。
- 不改变 Workflow 编译、占用冲突、revision、启停恢复与 Adapter requirement 语义。
- 不改变外部 crate 名、依赖或公开方法集合。
