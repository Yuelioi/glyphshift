# 探针创建来源统一计划

## Stage 1：创建合同与所有权

- [x] [统一 Probe creation request](slices/unified-probe-creation.md)：目标来源支持资料库 Software 或当前
  程序，词典来源支持已有 Dictionary 或系统命名的临时 Dictionary。
- [x] 复用现有 ownership ledger，覆盖四种来源组合、创建失败补偿、引用保护、保留与崩溃恢复。
- [x] 两项资产都复用时创建普通 Probe；任一临时资产存在时才保留临时状态。

## Stage 2：单一产品入口

- [x] 合并“快速测试”和“新建探针任务”，以一个 Modal 展示独立的目标来源与词典来源选择。
- [x] 默认选择成本最低的可用路径：已有资料可直接复用；临时词典不要求名称，语言和 Adapter 进入
  高级选项。
- [x] 当前程序沿用显式前台捕获与 preflight，不显示进程标识、程序路径以外的内部 Runtime 事实。

## Stage 3：验证与收尾

- [x] Desktop Shell 定向合同覆盖四种来源组合和所有权边界。
- [x] Probe Playwright 覆盖空资料库、已有资料、当前程序、临时词典、中英文与紧凑视口。
- [x] 删除重复入口和失效文案，更新 PRODUCT、DESIGN 与旧 Quick Probe Slice 的后续说明。
