# Translation 内部 module

Status: Complete

## Goal

保持公开 API、Generation、Snapshot/Font Policy digest、Workspace 冲突优先级和序列化结果不变，
把不可变运行输入与可编辑来源编排按变化原因拆成深 module。

## Baseline

- `src/lib.rs` 1593 行，前 718 行为 Translation Snapshot/Font Policy，后 874 行为 Workspace 编辑模型。
- Snapshot/Font Policy 面向 Decision/Adapter 消费；Workspace 面向来源扫描、用户覆盖、冲突视图与事件。
- 现有 `tests/workspace_contract.rs` 从公开 interface 覆盖 generation、冲突和发布行为。

## Plan

- [x] 提取 Snapshot/Font Policy module，保持 digest 字节算法与遍历顺序原样。
- [x] 提取 Workspace module，只通过公开 Translation Snapshot interface 构建运行快照。
- [x] 根 facade 私有化 module，并原样 re-export 既有公开 surface。
- [x] 运行 Translation 合同、直接下游回归、package Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审公开 API、digest、generation、冲突优先级与序列化语义。

## Result

- 根 `lib.rs` 从 1593 行降为 8 行，只声明两个私有 module 并 re-export 原公开 surface。
- `snapshot.rs` 716 行，封装不可变 Translation Snapshot、Font Policy、Adapter scope 和确定性 digest。
- `workspace.rs` 878 行，封装来源层、用户编辑、冲突视图、事件与 catalog 序列化。
- 两段实现与 `HEAD` 对应源码逐行比较完全一致，只有 module import/facade 发生变化。

验证通过：Translation Workspace 合同 10/10；13 个直接下游 package 的确定性测试全部通过，授权实机
合同仍按原定义 ignored；Translation Clippy `-D warnings`、Rust 格式、架构检查与 `git diff --check`
全部通过。

Standards 自审确认两个 module 都低于 1200 行，根没有业务转发层，也未公开内部 module。Spec 自审以
逐行搬迁比较确认 digest 字节算法、Generation 推进、来源优先级、冲突排序、事件和 JSON 转义原样；
公开类型/方法继续由 crate 根 re-export，全部直接消费方编译与合同测试通过。

## Decisions

- Snapshot 与 Font Policy 同属不可变运行输入，共享确定性 digest 实现与 Adapter scope 语义。
- Workspace 是可编辑来源编排：Source layer、用户覆盖、冲突、事件和 catalog 序列化属于同一变化原因。
- 不为字段或小类型建立浅 module；本切片只建立两个消费模型清晰的边界。
