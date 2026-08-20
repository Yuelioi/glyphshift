# 探针创建来源统一计划

## Stage 1：创建合同与所有权

- [x] [统一 Probe creation request](slices/unified-probe-creation.md)：目标来源支持资料库 Software 或当前
  程序，词典来源支持已有 Dictionary 或系统命名的临时 Dictionary。
- [x] 复用现有 ownership ledger，覆盖四种来源组合、创建失败补偿、引用保护、保留与崩溃恢复。
- [x] 两项资产都复用时创建普通 Probe；任一临时资产存在时才保留临时状态。

## Stage 2：单一产品入口

- [x] 合并“快速测试”和“新建探针任务”，以一个 Modal 展示独立的目标来源与词典来源选择。
- [x] 默认选择成本最低的可用路径：已有资料可直接复用；临时词典不要求名称，源语言与目标语言常显，
  Adapter 由系统选择。
- [x] 当前程序沿用显式前台捕获与 preflight，不显示进程标识、程序路径以外的内部 Runtime 事实。

## Stage 3：验证与收尾

- [x] Desktop Shell 定向合同覆盖四种来源组合和所有权边界。
- [x] Probe Playwright 覆盖空资料库、已有资料、当前程序、临时词典、中英文与紧凑视口。
- [x] 删除重复入口和失效文案，更新 PRODUCT、DESIGN 与旧 Quick Probe Slice 的后续说明。

## Corrective slice：创建器可用性

- [x] 资料库为空时保持 Software/Dictionary 来源 Tab 可进入，并呈现明确空态和替代路径。
- [x] 任务名称与临时词典源语言/目标语言常显，不再归入折叠的高级选项。
- [x] Active Process 来源支持运行中可见软件列表、刷新和快捷键捕获三项交互。
- [x] Desktop API `/25` 提供去重排序的运行中可见软件候选，不向 GUI 暴露 PID。

## Corrective slice：资产创建与清理

- [x] 临时 Probe 清理在 ownership ledger 存在但 Probe Run 已缺失时，释放悬空运行态并继续回收资产。
- [x] Software 页头只保留一个“新建软件”，Modal 内统一运行中列表、按键捕获与手动路径。
- [x] Dictionary 新建和设置复用元数据表单；主要字段常显，低频发布字段折叠到“更多”。
- [x] 标签和作者使用 Nuxt UI 项目输入；发布版本使用三个数字字段生成。
- [x] 受影响的 Rust 边界与前端生产构建通过；不升级为全仓测试。
- [x] Playwright 定向验证 Software 单入口/前台捕获与 Dictionary 完整元数据页面合同。

## Corrective slice：Probe 与 Dictionary 联动

- [x] Probe 详情常显绑定 Dictionary、语言、词条数、关系说明和打开入口。
- [x] Probe 设置允许在释放连接后切换唯一 Dictionary binding；切换不复制或删除词典内容。
- [x] 联合表支持单条或多选非空译文写入绑定 Dictionary，并支持移除译文但保留 Observation。
- [x] Desktop API `/26` 增加原子批量词条 upsert，Probe Run update 合同纳入 `dictionaryId`。
- [x] Core、Desktop Backend 与 Desktop Shell 定向门禁通过。
- [x] Playwright 定向验证空资料库临时 Probe 与 Probe 详情/Dictionary 联动页面合同。
