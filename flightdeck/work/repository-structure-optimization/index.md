# 全仓代码结构梳理与优化

Status: Open

## Goal

在不改变产品行为、Adapter ABI、Runtime Bundle 合同和安全失败语义的前提下，让代码库更容易定位、
修改与验证：拆解职责堆叠的超长文件，把测试放到合适的 seam，通过明确的 crate 家族与依赖方向降低
54 个 workspace crate 的导航成本，并避免产生新的浅模块或万能 `core`。

## Current

前序 Adapter 覆盖、桌面加固和技术文档入口已在 checkpoint `bbebea9` 固化。当前基线包含 158 个代码
文件，物理行数超过 1200 的有 11 个；其中 8 个是生产路径，但拆出内嵌测试后，首批真正超过 1200 行
实现体的是桌面 Tauri 壳、桌面后端、桌面运行时、翻译与隔离 Worker Host。

Rust workspace 当前有 54 个 crate，其中 25 个属于 Adapter 家族。`glyphshift-domain`、Adapter SDK、
Adapter Registry、Native ABI、Translation 与 Capture 是高复用依赖；Target Runtime、Native Host、
Desktop Runtime 和 Desktop Shell 则编排大量内部依赖。现有架构测试对每个 crate 的直接依赖做精确
约束，因此不能先移动目录或合并 crate，再事后解释依赖。

物理目录分成 `core`、`adapters`、`runtime` 等家族是候选方向，不是既定答案。`core` 若采用，只表示
低依赖、稳定语义的目录家族，不会合并成一个万能 crate；Adapter 仍按文字技术划分，而不是按软件品牌
划分。

## Next

完成[超长文件职责与拆分图](slices/large-file-decomposition-map.md)：结合
[结构基线](references/repository-structure-baseline.md)逐项记录首批生产文件的接口、实现职责、调用者、
内嵌测试和可验证 seam，然后按[计划](plan.md)选出第一个无行为变化的拆分切片。

## Progress

- 已创建当前成果 checkpoint `bbebea9`，工作树从干净状态进入本 Work。
- 已用物理行数重新统计代码文件，避免 PowerShell 行统计忽略空行导致优先级偏差。
- 已盘点 workspace crate 数量、Adapter 家族数量、内部依赖出入度和现有架构依赖合同。
- 已确认行数只用于分流审查；拆分由接口深度、职责局部性、变化原因和测试 seam 决定。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [仓库结构基线](references/repository-structure-baseline.md)
- [Adapter 按文字技术划分的既有结论](../../knowledge/research/market-landscape.md)
- [产品契约](../../../PRODUCT.md)
- [领域语言](../../../CONTEXT.md)
- [桌面设计契约](../../../DESIGN.md)
