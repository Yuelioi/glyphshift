# UIA Runtime Bundle 集成

Status: Complete

## Outcome

`windows.uia.observe` 已作为正式 Runtime Bundle 的隔离 Worker Adapter 进入 Desktop Probe catalog。
它只声明 `TextObserve + ObserveOnly + IsolatedWorker`，不会出现在 Workflow 的翻译 Adapter 列表，也不
修改 Dictionary。

## Delivered

- Bundle `/2` 增加可选 `isolated_workers` 清单；每个 Worker artifact 都先经过相对路径、SHA-256 与
  文件存在性验证，再进入 Worker artifact catalog。
- 隔离 Worker 清单只接受 `text-observe`，Descriptor 固定为 `ObserveOnly + IsolatedWorker`；不能通过
  清单把写回能力伪装成进程外观察器。
- 构建脚本编译并复制 UIA Worker，生成版本化文件名和哈希，同时把 UIA 的中文名称、说明、技术分类
  与目标信息放入现有 Adapter 展示目录。
- Desktop 只在当前 Controller recipe 确实包含隔离 Adapter 时签发临时 target grant；无关的 Console
  或翻译配方不会扩大授权。隔离 Worker 只允许在有 Desktop capture owner 的探针会话启动。
- `HybridAdapterHost` 保留权限拒绝和超时等原始失败，不再把所有隔离 Worker 启动失败折叠成泛化
  unavailable。
- observe-only 探针允许合法的初始 publication generation `0`；producer generation 仍必须非零，后续
  publication update 仍受单调约束。

## Verification

- 本地正式 Bundle 构建通过，catalog 合同证明 Console 与 UIA Observer 都能在 Probe 中发现，但二者
  均不会进入 `translation_adapter_ids`。
- 删除 Bundle 副本中的 Worker artifact 后，`RuntimeBundle::open` 在启动任何 Worker 前以
  `BundleUnavailable` 拒绝。
- Desktop 经正式 Bundle 对确定性标准控件目标启动 UIA Worker，capture 得到 label、单行 value 和多行
  document 三类公开文本；密码文本未进入 catalog。
- 高完整性 UAC 合同、Provider 永久阻塞恢复、完整窗口树重建、全 workspace、Clippy `-D warnings`、
  fmt 与架构门禁均通过。所有实机输入输出仍只位于本地测试目录。

## Next

回到 [Observation Stream Identity](observation-stream-identity.md)：先盘点现有 Native ABI、Runtime
observation、Capture 与 diagnostics 的信号，再冻结多流身份和独立 cursor 合同。
