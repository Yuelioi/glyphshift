# 非 Adapter crate 边界与命名审计

> 审计日期：2026-08-06
> 范围：22 个 `crates/` package、2 个应用 package、5 个 `test-support/` package。
> 方法：Cargo metadata 的 target/依赖/调用者、源码规模、产物类型和删除测试；命名依据见
> [Rust/Cargo crate 命名与 workspace 最佳实践](rust-crate-naming-and-workspace-best-practices.md)。

## 结论

- **不合并这 29 个 package。** 每个 package 至少保留了一个可验证边界：多调用者低层合同、独立
  binary/cdylib、进程与权限隔离、不同依赖集合、产品组合根或确定性测试制品。删除任何候选都会把实现
  复制到多个调用者，或迫使低层合同依赖高层实现。
- **Cargo package 名继续保留 `glyphshift-`。** Cargo 没有 package namespace；前缀让 `cargo -p`、
  metadata、日志和制品在 workspace 外仍有上下文。改成 `protocol`、`session` 等通用身份只减少字符，
  不减少概念。
- **物理目录改用家族和短叶名。** 例如 package `glyphshift-protocol` 放到
  `crates/runtime/protocol/`，package `glyphshift-domain` 放到 `crates/core/domain/`。目录路径不是 package
  身份，可以承担仓库导航而不制造 Cargo/ABI 迁移。
- **不做全仓 dependency alias 扫荡。** alias 是 consumer 局部名字，适合名称冲突或同一 package 多版本；
  对同一 workspace 全面换名会让 manifest key、源码 crate 名和 package 日志出现两套术语。确有局部语义
  收益时再单独引入。
- **所有 workspace package 显式禁止发布。** 当前仓库没有发布元数据或外部 registry 合同；统一
  `publish = false` 才能让“内部 package”成为 Cargo 可执行的事实。未来要发布的 SDK 可显式退出该默认值，
  并继续保留有项目上下文的 package 名。

## 逐 package 删除测试

“调用者”包含 normal dependency；括号内补充只在 dev/test 出现的调用者。

| package | 直接调用者或产物 | 删除后会发生什么 | 结论与目标目录 |
| --- | --- | --- | --- |
| `glyphshift-domain` | 32 个 normal consumer | 基础领域类型会复制到几乎所有层，或形成新的万能共享包 | 保留；`core/domain` |
| `glyphshift-translation` | 6（另 7 个 dev） | 字体映射、快照与 workspace 编辑语义会散入产品和 runtime | 保留；`core/translation` |
| `glyphshift-capture` | 11 | 捕获观察、目录和批处理模型会在 Controller、Worker、Target 间复制 | 保留；`core/capture` |
| `glyphshift-decision` | 2 | Runtime Kernel 与 Target 合同诊断会各自实现选择策略 | 保留；`core/decision` |
| `glyphshift-workflow` | 2 | Backend 与 Shell 将重复 workflow 验证和状态语义 | 保留；`core/workflow` |
| `glyphshift-extension` | 4（另 1 个 dev） | Controller、Protocol、Service、Desktop Runtime 会重复扩展描述 | 保留；`core/extension` |
| `glyphshift-dictionary-package` | 2（另 1 个 dev） | 稳定 package schema 会并入分发或产品实现 | 保留；`dictionary/package` |
| `glyphshift-dictionary-distribution` | 2 | 签名、摘要、版本与下载策略会泄入 Backend/Shell | 保留；`dictionary/distribution` |
| `glyphshift-runtime-contract` | 11 | publication/identity/wire 合同会被多个进程与产品各自复制 | 保留；`runtime/contract` |
| `glyphshift-runtime-kernel` | Target Runtime（Service dev） | 合并进 contract 会反转依赖；合并进 Target 会迫使无头 E2E 依赖 unsafe cdylib | 保留；`runtime/kernel` |
| `glyphshift-protocol` | 5 | 多个 Host/Service 将重复连接与 Controller 线协议 | 保留；`runtime/protocol` |
| `glyphshift-session` | 4 | 会话状态机与 runtime contract 编排会散入多个 Host | 保留；`runtime/session` |
| `glyphshift-controller-sdk` | Host、Windows Controller、测试插件 | 跨进程 SDK 会被实现 package 吞入，测试插件失去低层依赖面 | 保留；`runtime/controller/sdk` |
| `glyphshift-controller-host` | Desktop Runtime | 虽为单一生产调用者，但独立进程启动、插件校验和协议适配提供安全且可独测的边界 | 保留；`runtime/controller/host` |
| `glyphshift-controller-windows` | Desktop Shell；独立 bin | Windows Controller 是平台进程制品并隔离 unsafe/系统 API | 保留；`runtime/controller/windows` |
| `glyphshift-isolated-worker-sdk` | UIA Worker、Worker Host、测试 Worker | Worker wire SDK 是三方共享合同 | 保留；`runtime/worker/sdk` |
| `glyphshift-isolated-worker-host` | Desktop Runtime（UIA Worker dev） | 进程监督、重启和 placement 会泄入产品 runtime | 保留；`runtime/worker/host` |
| `glyphshift-target-runtime-contract` | Controller Windows、Target Host、Target Runtime | 部署描述、诊断与注入合同会在控制端/目标端分叉 | 保留；`runtime/targets/contract` |
| `glyphshift-target-process-host` | Desktop Runtime、Worker Host | 目标进程生命周期和协议桥会被两处复制 | 保留；`runtime/targets/process-host` |
| `glyphshift-target-runtime` | 独立 cdylib/rlib | 是目标进程内动态制品，包含受审计 unsafe 边界 | 保留；`runtime/targets/runtime` |
| `glyphshift-desktop-runtime` | Desktop Shell | 单一调用者不等于伪边界；它隔离进程池、Native Host、协议与 unsafe 依赖，并有独立合同测试 | 保留；`runtime/desktop` |
| `glyphshift-desktop-backend` | Desktop Runtime、Desktop Shell | 产品持久化与字典/workflow 编排不应进入 Tauri Shell 或进程 runtime | 保留；`product/desktop-backend` |
| `glyphshift-desktop-shell` | Tauri staticlib/cdylib/rlib/bin | GUI 应用组合根和发布制品 | 保留在 `apps/glyphshift-desktop/src-tauri` |
| `glyphshift-service` | lib + 无头集成测试 | 无头组合根独立于 GUI Shell，并验证动态组合 | 保留在 `apps/glyphshift-service` |
| `glyphshift-reference-adapters` | Service dev | 确定性 Adapter fixture；并入生产会形成 production → test 反向依赖 | 保留在 `test-support/reference-adapters` |
| `glyphshift-test-controller-plugin` | 独立测试 bin | Controller 进程协议 fixture | 保留在 `test-support/controller-plugin` |
| `glyphshift-test-isolated-worker` | 独立测试 bin | Worker 进程协议 fixture | 保留在 `test-support/isolated-worker` |
| `glyphshift-windows-host` | Windows Runtime Target（另 2 个 dev） | 可见窗口/字体宿主为 Windows 合同测试提供唯一可控实现 | 保留在 `test-support/windows-host` |
| `glyphshift-windows-runtime-target` | 独立测试 bin | 目标进程注入和可见性测试需要真实进程制品 | 保留在 `test-support/windows-runtime-target` |

## 命名操作收益/代价矩阵

| 操作 | 收益 | 代价/风险 | 决策 |
| --- | --- | --- | --- |
| 目录改为 `runtime/protocol` 等 | 导航直接表达家族；不改 Cargo/API/ABI 身份 | 更新 workspace、path dependency 和深度敏感测试路径 | **实施** |
| `[package].name` 去掉 `glyphshift-` | `cargo -p` 少输入若干字符 | 通用名无项目上下文；改变 Package ID、默认 target/制品名、lockfile 与自动化 | **拒绝** |
| `[lib].name` 去前缀 | Rust 路径变短 | 改变所有 consumer 的公开 crate 名；动态制品尤其敏感 | **拒绝** |
| 全仓 dependency alias | 可局部缩短 `use` 路径 | 同一概念出现 package 名/alias 两套词汇，约 371 处源码引用迁移 | **不批量实施** |
| 合并 Runtime 合同、Kernel、Protocol、Session | 减少 manifest 数量 | 高扇出合同依赖实现、feature/unsafe/进程边界扩大 | **拒绝** |
| workspace `publish = false` | 防止内部 package 被意外发布；把内部属性变成机器检查 | 未来公开 SDK 需显式调整策略 | **实施** |

## 目标物理结构

```text
crates/
  adapters/{platform,implementations}/<short-name>
  core/{domain,translation,capture,decision,workflow,extension}
  dictionary/{package,distribution}
  runtime/
    {protocol,contract,kernel,session,desktop}
    controller/{sdk,host,windows}
    worker/{sdk,host}
    targets/{contract,process-host,runtime}
  product/desktop-backend
```

`core` 只是一层物理目录，不创建名为 `core` 的 crate，也不放宽架构依赖方向。应用和 test-support 保持
现有顶层入口。

## 自审

- **Standards：** 结论区分了 package、lib crate、consumer alias、目录四层；没有用行数或单一调用者
  机械判断边界，也没有把 `core` 变成万能依赖层。
- **Spec：** 回答了“能否叫 runtime/protocol”与“内部包是否需要前缀”：目录可以且应简化，Cargo 身份
  保留前缀；内部属性通过 `publish = false` 明确，而非靠通用 package 名暗示。
- **删除测试：** 29 个 package 均记录删除后的复杂度去向；当前没有证据充分的合并候选。
