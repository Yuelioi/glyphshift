# Session 内部 module

Status: Complete

## Goal

保持 Session 的公开 ports/report/status/error 与 start/update/stop 状态机不变，把部署合同值对象和有状态
SessionManager 按变化原因拆开；1274 行外部合同测试留给后续测试结构切片。

## Baseline

- `src/lib.rs` 1122 行，无内嵌测试；前 709 行是 target/recipe/host report/status/error 合同，后 413 行
  是 SessionRecord 与 SessionManager 状态机。
- `tests/session_contract.rs` 1274 行，从公开 interface 覆盖 18 项场景，是后续大型合同测试切片。

## Plan

- [x] 提取 contract：target、ports、bound adapter/feature、report、phase/status/error。
- [x] 提取 manager：registry resolve、controller/host 生命周期、generation 与 stop/release。
- [x] 根私有化 module 并保持公开 re-export。
- [x] 运行 18 项合同、直接下游、Clippy、格式、架构和 diff。
- [x] Standards/Spec 自审公开 surface、start/update/stop、loss policy 与失败隔离。

## Result

- 根 `lib.rs` 从 1122 行降为 7 行，只声明私有 Contract/Manager 并稳定 re-export。
- `contract.rs` 705 行：Target/Recipe ports、Bound Adapter/Feature、Host reports、phase/status/error。
- `manager.rs` 422 行：SessionRecord、Registry resolve、start/update/control/health/stop/release 状态机。
- 跨 module 字段与辅助只使用 `pub(super)`，crate 外字段私有性和公共 API 不变。

验证通过：Session 合同 18/18；4 个直接下游 package 的确定性测试全部通过，12 项授权 Desktop Runtime
合同保持 ignored；package Clippy `-D warnings`、Rust 格式、架构检查与 `git diff --check` 通过。

Standards 自审确认 Contract 是一套完整部署语言而非 value-object 碎片，Manager 是唯一状态所有者；两
文件均低于 1200 行。Spec 自审将两段实现去除 crate 内可见性后与 `HEAD` 逐段比较完全一致，公开类型/
函数多重集一致；start/update/stop、controller loss policy、generation、target exit 和 host 失败隔离由
原 18 项合同及四个下游编排测试集覆盖。

## Decisions

- Ports/report/status 是部署合同；Manager 是该合同的唯一有状态执行器，二者变化原因不同。
- 不拆每个 report/value object；它们共同定义 Host 与 Session 的一套语言。
