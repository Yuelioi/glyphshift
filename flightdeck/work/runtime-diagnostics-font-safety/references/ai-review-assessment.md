# AI 评审路由：当前收敛项

临时目录中的三份架构评审和通用升级提示已经按当前代码复核。本页只保存可复用结论，不复制临时
原文，也不把未经验证的建议当作产品事实。

## 采纳

- 有界 Decision Trace：解释 Adapter、字典命中、字体决策、Pass 和 fail-open 原因。
- `all_observations` 的高影响提示，以及有真实字体信号支持时的图标/符号字体保护。
- 在现有 Observation Index 上补充必要且有界的 Adapter 证据，而不是建立第二种探针数据。
- 将已有 Translation/Font Policy digest 用于更准确的部署 ACK 和诊断，但不重造快照身份。
- 以有界状态转换日志记录启停、发布、ACK mismatch 和 Runtime failure；不采用事件溯源。

## 已经实现，不重复建设

- Effective Workflow Intent 与 desired/actual reconcile。
- Translation Snapshot 和 Font Policy 的确定性 digest。
- Capture 有界队列、批量 checkpoint、暂停恢复和 dropped observation 计数。
- Native Adapter 的重入检测、panic 隔离和 fail-open。
- Adapter Feature、Apply Model、Placement、ABI、架构和 artifact hash 描述。

## 不采纳

- 自动超时关闭 `all_observations`。
- 把窗口、坐标、Context、字体或探针元数据放进 Dictionary Entry。
- 全局 substring/fuzzy/AI Runtime 匹配。
- Probe Overlay 或 Dictionary Draft 双写。
- 用统一 Renderer 抹平 inline-render、retained-object 和 external-protocol 的不同执行语义。
- 为减少 crate 数量而合并具有不同生命周期和信任职责的 Module。
