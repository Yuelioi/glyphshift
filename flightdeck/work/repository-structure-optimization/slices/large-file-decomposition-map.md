# 超长文件职责与拆分图

Status: Open

## Goal

对首批超长生产文件逐一识别现有 interface、implementation 职责、调用者、测试位置和变化原因，选出
能提高 locality、保持 interface 稳定且可独立验证的第一个拆分切片。

## Delivery

- [ ] 记录桌面 Tauri 壳、Desktop Backend、Desktop Runtime、Translation 与 Isolated Worker Host 的
  公共 interface、主要调用者和内部职责簇。
- [ ] 对 Capture、Controller Windows 与 Capture Workspace 区分“实现过长”和“内嵌测试导致文件过长”。
- [ ] 对每个候选执行删除测试：如果抽出的 module 删除后复杂度只散回调用者，说明它有深度；若只剩
  转发，则调整 seam。
- [ ] 为第一拆分候选固定兼容面、测试命令、预计文件结构和回滚边界。

## Current

物理行数基线、crate 数量和内部依赖中心性已经完成。桌面 Tauri 壳、Desktop Runtime、Capture、
Controller Windows 和 Capture Workspace 都含有显著内嵌测试；Desktop Backend、Translation 与
Isolated Worker Host 的实现体本身超过 1200 行。尚未决定第一个拆分对象，也未移动源文件。

## Next

从桌面 Tauri 壳开始，沿 `run()`、Tauri 命令、`DesktopApplication`、请求/视图映射、窗口生命周期和
内嵌测试六类职责追踪调用关系；以保持外部命令名与序列化合同为前提，写出至少两个内部 module 方案并
按 depth、locality 和测试 seam 比较。

## Boundaries

- 本切片只产出职责图和首个实施设计，不修改产品行为或公共 interface。
- 不用“每 500 行一个文件”或按类型名字母分组；module 必须对应稳定变化原因。
- 不把只有一个实现的私有 helper 抬升为公共 port；测试可使用私有内部 seam。

## References

- [仓库结构基线](../references/repository-structure-baseline.md)
- [稳定上下文](../context.md)
- [阶段计划](../plan.md)
