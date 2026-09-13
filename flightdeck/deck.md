# Flightdeck

## Open Work

- [DirectWrite 修复与 0.4.1 发布](work/ae2025-text-capture/index.md)

- **Focus** [Rust 模块结构整理](work/rust-module-refactor/index.md)

- [单软件工作流统一](work/workflow-unification/index.md)

- [探针与字典流程统一](work/probe-dictionary-flow/index.md)
- [Houdini 与其他软件文字覆盖](work/software-text-coverage/index.md)
- [适配器边界与 x86 规划](work/adapter-coverage-and-x86/index.md)
- [Alias / Qt Quick 适配器](work/alias-text-support/index.md)
- [游戏文字适配器](work/game-text-adapters/index.md)
- [AE 临时界面观测丢失](work/ae-transient-observation-loss/index.md)
- [发布根文档对齐](work/release-root-docs/index.md)

## Project links

- [软件兼容状态](knowledge/rendering/software-compatibility.md)
- [领域语言](../CONTEXT.md)
- [产品契约](../PRODUCT.md)
- [设计系统](../DESIGN.md)

## 验证纪律

- 开发循环只运行受影响 package、合同与页面的定向测试，并在约 20 秒没有新输出时主动回报状态。
- 大 Slice 自审运行相关边界回归，不默认升级为全仓测试。
- 全仓测试只用于提交前、发布前，或 workspace 拓扑、公共 ABI / wire contract 发生实质变化时；必须
  使用 `scripts/test.ps1` 的显式活动包边界，禁止原始 `cargo test --workspace` 把归档 UIA 纳入作业。

## 适配器策略

- 当前阶段只扩展通用适配器能力，暂时禁止新增软件专属适配器。适配器 ID、目录和运行时选择逻辑必须描述框架、渲染接口或稳定文字入口，不能按某个软件、进程名或软件 ID 建立专属分支。
- 允许在现有通用适配器内增加经过实机验证的精确版本 / ABI / C++ 命名空间 profile，例如 Qt 的特定 ABI；这类 profile 只是通用适配器的兼容能力，不得演变成软件专属适配器或任意通配识别。
- 新增兼容 profile 必须保持已有 profile 和旧软件行为不回退：保留原路径，新增条件精确命中，未知配置继续 fail-closed；至少运行覆盖旧 profile、新 profile 与未知配置拒绝的定向回归测试。
- 某软件当前找不到通用入口时，记录为“尚不支持 / 继续研究可抽象的通用入口”，不要以软件专属适配器绕过该限制。只有用户明确解除本规则后，才能设计或实现软件专属适配器。
