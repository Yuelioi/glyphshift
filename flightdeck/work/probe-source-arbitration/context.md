# 稳定上下文

## 产品事实

- Workflow 与 Probe 当前以 Software 为粒度互斥；本 Work 不引入共享 Hook、多订阅游标或新的
  Target Execution 租约模型。
- 用户需要的是两种不同运行模式：Workflow 持续写回；Probe 观察原文并可选实时预览。冲突应说明
  当前目标已被另一模式占用，并给出“先停用工作流/释放探针”的恢复动作。
- UIA 是结构化可访问性观察源，不证明屏幕最终绘制路径，也不能写回。它对 GDI/GDI+/DrawText
  未覆盖的标准控件仍有独立兜底价值。
- 可写回绘制 Adapter 的 Observation 同时证明原文和可应用译文的 seam；与 observe-only 来源重叠时，
  应成为探针的首选来源。
- 持续观察器与绘制 Hook 并发执行；“观察器最后”是确定性的结果排序、去重和容量仲裁，不是等待其他
  Adapter 结束后才启动。

## 架构约束

- Core、Desktop 与 GUI 不按 Adapter ID、软件品牌或可执行文件名分支。
- 来源优先级必须从 Registry/Descriptor 的稳定能力推导，例如 Feature、Apply Model 与 Placement；
  不能维护 UIA/GDI 名称排序表。
- 原始 Observation 与 provenance 仍应有界、可诊断。首选来源不等于删除其他来源，也不能让低优先级
  Worker 的失败影响目标进程内写回 Adapter。
- Dictionary 继续只保存 `source + translation`，不能加入 Adapter 优先级或来源技术字段。
- Runtime 实际所有权与冲突是短期状态，不进入 Software、Dictionary 或 Workflow 持久模型。

## 验收语义

- 工作流占用目标时连接探针，界面明确说明“此软件正由工作流使用”，不再显示通用激活失败。
- 同一规范化原文同时来自可写回与 observe-only Adapter 时，联合表以可写回来源为首选，并仍可展示
  完整来源证据；只被 observe-only 观察到的原文继续出现。
- Adapter 选择顺序和事件到达顺序不能改变首选来源结果。
- 所有新行为从深 Module 的现有 Interface 测试，不把排序规则复制到 Vue、Tauri 与 Runtime 多处。
