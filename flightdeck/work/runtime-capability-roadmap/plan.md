# 运行时能力升级计划

## Stage 1：多进程目标

- [x] [定义 Process Family 授权、发现、身份、退出和部分失败合同](slices/process-family-controller-inventory.md)。
- [x] 用合成 Controller/Target Instance 验证一个软件的主进程与子进程状态聚合。
- [x] 在授权目标软件上保存本地证据并决定首个生产 Extension。

## Stage 2：执行与观察复用

- [x] 将当前独占规则评估为单一 Target Execution、一个 Publication Owner 与多个只读 Subscriber；
  在多游标 Observation Stream 交付前保留现有互斥。
- [ ] 定义 Adapter 提供的 Observation Stream Identity、样本历史、匹配置信度与持久 Binding 合同。
- [ ] 验证目标重启后按当前实例重匹配 Stream，并分别报告 matched、degraded 与 unmatched。
- [ ] 证明 Probe、诊断和 Workflow 共享 Hook 时不会重复注入、竞争写回或提升错误 Feature 状态。

## Stage 3：Windows Adapter 覆盖

- [ ] 先研究并原型验证 DirectWrite Adapter 的 observe/replace Seam。
- [ ] 评估 UI Automation observe-only Adapter 与绘制观察的去重关系。
- [ ] 只在结构化 Adapter 无法覆盖且用户明确选择时评估 Window Capture、OCR observe-only 与
  Change Trigger Policy 组成的兜底。
- [ ] 按真实缺口分别决定 Direct2D、Direct3D、OpenGL 或 Vulkan 是否立项。

## Stage 4：原生稳定性与布局

- [ ] 研究 SEH/VEH、Watchdog、熔断和运行成本，选择能保持 fail-open 的最小方案。
- [ ] 以真实截断案例验证字体度量和 `LayoutAdjust`，不建立万能 Renderer。
- [ ] 仅在原位写回不可行且有真实需求时评估独立 Overlay / External Window Apply Model。

## Stage 5：有界 Transform Profile

- [ ] 定义捕获正规化、保护/排除和结果修正的有序、版本化、可预览规则合同。
- [ ] 为每个 Operator 固定输入类型、执行预算、冲突诊断和 fail-open 语义，不开放任意脚本。

## Stage 6：可选匹配与创作辅助

- [ ] 只有精确匹配出现可复现缺口时才设计 Dictionary Match Policy 和冲突诊断。
- [ ] 将模糊/AI 能力限制为 Probe 离线建议，并保留人工确认与确定性 Runtime Publication。
