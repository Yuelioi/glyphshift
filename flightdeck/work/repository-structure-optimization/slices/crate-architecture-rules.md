# Crate 家族与依赖方向门禁

Status: Complete

## Goal

把 54-package 家族覆盖与 L0-L5/Adapter/Test 依赖方向变成可执行检查，并补齐现有逐 package 精确依赖
合同遗漏，使依赖约束不受物理目录布局影响，持续阻止跨层反向依赖和万能共享层扩散。

## Baseline

- 现有 `architecture-tests/check.ps1` 精确检查 48/54 个 package 的 normal dependencies。
- 缺少 GTK3/Pango、Qt Painter 四个 package、`glyphshift-workflow` 和 Windows Runtime target。
- 精确依赖列表可阻止未审计依赖，但若维护者同时更新列表，脚本没有第二层语义规则说明新方向是否合法。

## Plan

- [x] 补齐 6 个 package 的精确 normal-dependency 合同，达到 54/54。
- [x] 声明物理能力家族与依赖层级两个完整 partition，并与 Cargo workspace 做无遗漏/无重复比较。
- [x] 校验每条 workspace normal dependency 的 layer 允许集合；production 禁止依赖 Test 或具体 Adapter。
- [x] 用三条合成故障案例证明 L0 → Host、Host → concrete Adapter、Product → Test 会被拒绝。
- [x] 运行架构检查、全 workspace metadata、PowerShell parser、格式与 diff check，做 Standards/Spec 自审。

## Rules

- L0 仅依赖 L0；L1 可依赖 L0-L1；L2 可依赖 L0-L2。
- Adapter implementation 可依赖 L0-L2 和本技术 Adapter companion。
- L3/L4/L5 可逐层向下依赖，但不得静态依赖 concrete Adapter；生产 Runtime 通过 registry/bundle 发现实现。
- Test 可依赖 production 与 Test；其他层不得依赖 Test。

## Result

- 精确 normal-dependency 合同补齐 GTK3/Pango、Qt Painter、Workflow 与 Windows Runtime target，54 次调用、
  54 个唯一 package；新增 package 未登记合同时会因 workspace coverage 失败。
- Package Family 与 Dependency Layer 各自构成 54-package 的完整 partition；缺失、额外或重复归类都会失败。
- 每条 workspace normal dependency 经过 layer allow-list；concrete Adapter 只允许 L0-L2/同技术 Adapter，
  L3-L5 production 不得静态依赖它，任意 production 不得依赖 Test。
- 三条禁止方向和三条允许方向作为轻量 self-test 随门禁执行。PowerShell parser、Cargo metadata、workspace
  Rust 格式、架构脚本与 diff check 通过。

## Review

- Standards：精确依赖负责变更审计，layer rule 负责语义方向，二者不是同一检查；Adapter/Test 清单在两个
  partition 间复用，避免复制漂移，没有把 family 当作 dependency rank。
- Spec：Runtime 仍通过 Registry/Bundle 发现具体 Adapter；SDK/contract 不反向依赖 Host，Desktop/Service
  不依赖 Shell，test-support 只处于允许的依赖末端。
