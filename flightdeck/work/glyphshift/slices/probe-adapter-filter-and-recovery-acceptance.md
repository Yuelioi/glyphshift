# Probe Adapter 筛选与持续作业验收

Status: Complete

## Goal

在不修改 Runtime 协议、不引入 Observation Stream Identity 的前提下，补齐现有 Probe Run 的
Adapter 级筛选和持续作业发布验收，使四个 Adapter 同时观测、数千条记录和应用重启后的工作仍然
可定位、可继续、可诊断。

## Decisions

- Adapter 筛选只使用 Observation Index 已保存的 Adapter 事实，不把筛选条件写入 Dictionary。
- 每个 Probe Run 恢复自己的查询/筛选视图；视图状态不是翻译资产，也不能改变 Runtime 捕获范围。
- 暂停只停止新增 Observation；搜索、分页、行内编辑、批量处理和导出继续可用。
- 重启恢复的是 Probe Intent、Dictionary Binding、Observation Index 和视图状态，不恢复旧 PID、
  Runtime handle 或伪造的 running 状态。
- 候选文本流身份、样本历史、持久 Stream Binding 与重匹配需要新的 Adapter 证据和 Runtime 合同，
  归入运行时能力升级 Roadmap，不在本 Slice 中用 `location` 或调用地址替代。

## Delivery

- [x] 为 Probe 联合查询增加可选 Adapter 过滤，并保持后端分页和 5,000 条有界渲染。
- [x] 在详情页搜索区提供清晰的 Adapter 多选/清除筛选，不增加第二张技术目录表。
- [x] 为命名 Probe Run 持续保存并恢复查询、Adapter 筛选、页码和每页数量。
- [x] 收敛目标不存在、应用重启与恢复入口的 ready/interrupted 文案和诊断。

## Verification

- [x] 合同测试证明 Adapter 筛选不改变 Dictionary、Observation Index 或导出全集。
- [x] Playwright 覆盖多 Adapter、5,000 条、筛选后分页、多选和视图恢复。
- [x] 桌面验收证明暂停期间 Observation 不增长，但搜索、编辑、批量操作和导出仍可用。
- [x] 桌面重启验收证明任务和视图可恢复，目标重新发现并完成 Runtime ACK 前不显示 running。

## Current

Probe 联合查询已接受当前 Run 内的多 Adapter 条件；未知 Adapter 会被拒绝，筛选只改变当前后端
分页结果，不修改 Dictionary、Observation Index 或完整导出。详情页在同一工具区提供多选菜单，
并按 Probe Run 恢复搜索、Adapter 筛选、页码和每页数量。运行中或暂停中的任务重新打开存储后均
恢复为 interrupted，只有重新连接并取得 Runtime 结果后才会回到 running。

生产构建、30 项 Playwright、全 Rust workspace、Clippy、fmt 与架构检查通过；Playwright 同时覆盖
暂停后的行内编辑、导出可用性、5,000 条后端分页和刷新后的视图恢复。

## Next

本 Slice 已完成；返回 Glyphshift Work 的 Dictionary Catalog 执行指针。
