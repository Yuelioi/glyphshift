# Flightdeck

## Open Work

- **Focus** [单软件工作流统一](work/workflow-unification/index.md)

- [探针与字典流程统一](work/probe-dictionary-flow/index.md)
- [Houdini 与其他软件文字覆盖](work/software-text-coverage/index.md)
- [适配器边界与 x86 规划](work/adapter-coverage-and-x86/index.md)
- [Alias / Qt Quick 适配器](work/alias-text-support/index.md)
- [游戏文字适配器](work/game-text-adapters/index.md)
- [AE 临时界面观测丢失](work/ae-transient-observation-loss/index.md)
- [发布根文档对齐](work/release-root-docs/index.md)

## Project links

- [领域语言](../CONTEXT.md)
- [产品契约](../PRODUCT.md)
- [设计系统](../DESIGN.md)

## 验证纪律

- 开发循环只运行受影响 package、合同与页面的定向测试，并在约 20 秒没有新输出时主动回报状态。
- 大 Slice 自审运行相关边界回归，不默认升级为全仓测试。
- 全仓测试只用于提交前、发布前，或 workspace 拓扑、公共 ABI / wire contract 发生实质变化时；必须
  使用 `scripts/test.ps1` 的显式活动包边界，禁止原始 `cargo test --workspace` 把归档 UIA 纳入作业。
